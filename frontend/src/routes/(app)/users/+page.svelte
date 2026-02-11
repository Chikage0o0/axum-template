<script lang="ts">
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";
  import { ApiError } from "$lib/api/mutator";
  import {
    createUserHandler,
    deleteUserHandler,
    getUsersHandler,
    patchUserHandler,
    type CreateUserRequest,
    type PatchUserRequest,
    type UserResponse,
  } from "$lib/api/generated/client";
  import {
    CreateUserRequest as CreateUserRequestSchema,
    PatchUserRequest as PatchUserRequestSchema,
  } from "$lib/api/generated/schemas";
  import { type FieldErrors, zodErrorToFieldErrors } from "$lib/shared/forms/field-errors";
  import { useFieldErrors } from "$lib/shared/forms/use-field-errors.svelte";
  import { auth } from "$lib/features/auth/state/auth";
  import { buildUserPatchPayload } from "$lib/features/auth/model/user-helpers";
  import * as AlertDialog from "$lib/shadcn/components/ui/alert-dialog/index.js";
  import { Badge } from "$lib/shadcn/components/ui/badge/index.js";
  import * as Card from "$lib/shadcn/components/ui/card/index.js";
  import * as Empty from "$lib/shadcn/components/ui/empty/index.js";
  import * as Field from "$lib/shadcn/components/ui/field/index.js";
  import { Input } from "$lib/shadcn/components/ui/input/index.js";
  import * as Sheet from "$lib/shadcn/components/ui/sheet/index.js";
  import { Skeleton } from "$lib/shadcn/components/ui/skeleton/index.js";
  import { Switch } from "$lib/shadcn/components/ui/switch/index.js";
  import * as Table from "$lib/shadcn/components/ui/table/index.js";
  import TruncateText from "$lib/shared/components/truncate-text.svelte";
  import UserRowActions from "$lib/app/components/user-row-actions.svelte";
  import UserAvatar from "$lib/shared/components/user-avatar.svelte";
  import FormDialogShell from "$lib/shared/components/form-dialog-shell.svelte";
  import LoadStatePanel from "$lib/shared/components/load-state-panel.svelte";
  import UserProfileFields from "$lib/shared/components/user-profile-fields.svelte";
  import UsersToolbar from "$lib/app/components/users-toolbar.svelte";
  import PageHeader from "$lib/shared/components/page-header.svelte";

  type SheetMode = "create" | "edit";
  type UserFormDraft = {
    username: string;
    display_name: string;
    email: string;
    phone: string;
    avatar_url: string;
    is_active: boolean;
  };

  const emptyDraft = (): UserFormDraft => ({
    username: "",
    display_name: "",
    email: "",
    phone: "",
    avatar_url: "",
    is_active: true,
  });

  let users = $state<UserResponse[]>([]);
  let listLoading = $state(false);
  let listError = $state<string | null>(null);
  let permissionDenied = $state(false);

  let sheetOpen = $state(false);
  let sheetMode = $state<SheetMode>("create");
  let sheetSubmitting = $state(false);
  const sheetFieldErrors = useFieldErrors<string>();
  let selectedUser = $state<UserResponse | null>(null);
  let draft = $state<UserFormDraft>(emptyDraft());

  let deletingUserId = $state<string | null>(null);

  // AlertDialog 状态
  let deleteDialogOpen = $state(false);
  let deleteTarget = $state<UserResponse | null>(null);

  const sheetTitle = $derived(sheetMode === "create" ? "新增用户" : "编辑用户");
  const sheetSubmitLabel = $derived(sheetMode === "create" ? "创建用户" : "保存修改");
  const activeUserCount = $derived(users.filter((user) => user.is_active).length);

  function isSelfRow(userId: string): boolean {
    return $auth.user?.sub === userId;
  }

  function invalidSheet(...keys: string[]): boolean {
    return sheetFieldErrors.invalid(...keys);
  }

  function sheetErrorItems(...keys: string[]) {
    return sheetFieldErrors.items(...keys);
  }

  function openCreateSheet() {
    sheetMode = "create";
    selectedUser = null;
    draft = emptyDraft();
    sheetFieldErrors.clearErrors();
    sheetOpen = true;
  }

  function openEditSheet(user: UserResponse) {
    sheetMode = "edit";
    selectedUser = user;
    draft = {
      username: user.username ?? "",
      display_name: user.display_name,
      email: user.email,
      phone: user.phone ?? "",
      avatar_url: user.avatar_url ?? "",
      is_active: user.is_active,
    };
    sheetFieldErrors.clearErrors();
    sheetOpen = true;
  }

  function closeSheet() {
    sheetOpen = false;
    sheetFieldErrors.clearErrors();
    selectedUser = null;
    draft = emptyDraft();
    sheetMode = "create";
  }

  async function reloadUsers() {
    listLoading = true;
    listError = null;
    permissionDenied = false;

    try {
      users = await getUsersHandler({ include_deleted: false });
    } catch (e) {
      users = [];
      if (e instanceof ApiError && e.status === 403) {
        permissionDenied = true;
        return;
      }
      listError = e instanceof Error ? e.message : "加载用户列表失败";
    } finally {
      listLoading = false;
    }
  }

  function makeCreatePayloadFromDraft():
    | { ok: true; payload: CreateUserRequest }
    | { ok: false; errors: FieldErrors } {
    const payloadRaw = {
      username: draft.username.trim() || undefined,
      display_name: draft.display_name.trim(),
      email: draft.email.trim(),
      phone: draft.phone.trim() || undefined,
      avatar_url: draft.avatar_url.trim() || undefined,
    };

    const check = CreateUserRequestSchema.safeParse(payloadRaw);
    if (!check.success) {
      return { ok: false, errors: zodErrorToFieldErrors(check.error) };
    }

    return { ok: true, payload: check.data };
  }

  function makePatchPayloadFromDraft():
    | { ok: true; payload: PatchUserRequest }
    | { ok: false; message: string; errors?: FieldErrors } {
    if (!selectedUser) {
      return { ok: false, message: "未找到要编辑的用户" };
    }

    const built = buildUserPatchPayload({
      mode: "admin-edit",
      current: selectedUser,
      draft,
    });
    if (!built.ok) {
      if (built.message.includes("display_name")) {
        return { ok: false, message: built.message, errors: { display_name: [built.message] } };
      }
      if (built.message.includes("email")) {
        return { ok: false, message: built.message, errors: { email: [built.message] } };
      }
      return { ok: false, message: built.message };
    }

    const check = PatchUserRequestSchema.safeParse(built.payload);
    if (!check.success) {
      return { ok: false, message: "表单校验失败", errors: zodErrorToFieldErrors(check.error) };
    }

    return { ok: true, payload: check.data };
  }

  async function submitSheet() {
    sheetFieldErrors.clearErrors();

    if (sheetMode === "edit" && selectedUser && isSelfRow(selectedUser.id) && !draft.is_active) {
      sheetFieldErrors.setErrors({ is_active: ["当前登录账号不允许自禁用"] });
      toast.error("当前登录账号不允许自禁用");
      return;
    }

    sheetSubmitting = true;
    try {
      if (sheetMode === "create") {
        const result = makeCreatePayloadFromDraft();
        if (!result.ok) {
          sheetFieldErrors.setErrors(result.errors);
          return;
        }
        await createUserHandler(result.payload);
        toast.success("用户创建成功");
      } else {
        if (!selectedUser) {
          toast.error("未找到要编辑的用户");
          return;
        }

        const result = makePatchPayloadFromDraft();
        if (!result.ok) {
          if (result.errors) {
            sheetFieldErrors.setErrors(result.errors);
          } else {
            toast.warning(result.message);
          }
          return;
        }
        await patchUserHandler(selectedUser.id, result.payload);
        toast.success("用户更新成功");
      }

      closeSheet();
      await reloadUsers();
    } catch (e) {
      if (e instanceof ApiError) {
        sheetFieldErrors.mergeApiDetails(e.body?.details);
        if (Object.keys(sheetFieldErrors.errors).length > 0) {
          return;
        }
      }
      toast.error(e instanceof Error ? e.message : "保存失败");
    } finally {
      sheetSubmitting = false;
    }
  }

  /** 打开删除确认对话框 */
  function requestDelete(user: UserResponse) {
    if (isSelfRow(user.id)) {
      toast.warning("当前登录账号不允许自删除");
      return;
    }
    deleteTarget = user;
    deleteDialogOpen = true;
  }

  /** 确认删除 */
  async function confirmDelete() {
    if (!deleteTarget) return;
    const user = deleteTarget;

    deletingUserId = user.id;
    deleteDialogOpen = false;
    try {
      await deleteUserHandler(user.id);
      toast.success("用户已逻辑删除");
      await reloadUsers();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "删除失败");
    } finally {
      deletingUserId = null;
      deleteTarget = null;
    }
  }

  onMount(() => {
    void reloadUsers();
  });
</script>

<div class="content-flow space-y-6">
  <!-- 页面标题栏 -->
  <PageHeader title="用户管理">
    {#snippet actions()}
      <UsersToolbar loading={listLoading} onRefresh={reloadUsers} onCreate={openCreateSheet} />
    {/snippet}
  </PageHeader>

  {#if permissionDenied}
    <Empty.Root>
      <Empty.Header>
        <Empty.Title>无权限访问用户管理</Empty.Title>
        <Empty.Description>仅管理员可访问此页面，请联系管理员分配权限。</Empty.Description>
      </Empty.Header>
    </Empty.Root>
  {:else}
    <Card.Root>
      <Card.Header class="space-y-3">
        <div class="flex flex-wrap items-start justify-between gap-3">
          <div class="space-y-1">
            <Card.Title>用户列表</Card.Title>
            <Card.Description>默认仅显示未删除用户；删除后用户将从此列表移除。</Card.Description>
          </div>
          <div class="flex flex-wrap gap-2">
            <Badge variant="secondary" class="font-normal">总计 {users.length}</Badge>
            <Badge class="border-success/30 bg-success/10 text-success"
              >启用 {activeUserCount}</Badge
            >
          </div>
        </div>
      </Card.Header>

      <Card.Content>
        <LoadStatePanel
          loading={listLoading}
          error={listError}
          isEmpty={users.length === 0}
          onRetry={reloadUsers}
          onCreate={openCreateSheet}
          emptyTitle="暂无用户数据"
          emptyDescription="可以先新增一个用户。"
          createLabel="新增用户"
        >
          {#snippet loadingContent()}
            <div class="space-y-3">
              {#each Array.from({ length: 5 }, (_, index) => index) as index (index)}
                <div
                  class="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-3 rounded-lg border bg-gradient-to-br from-card to-muted/30 p-3 sm:grid-cols-[minmax(0,3fr)_minmax(0,2fr)_minmax(0,2fr)_minmax(0,1fr)_auto]"
                >
                  <div class="flex min-w-0 items-center gap-3">
                    <Skeleton class="size-8 shrink-0 rounded-full" />
                    <div class="min-w-0 space-y-2">
                      <Skeleton class="h-4 w-24" />
                      <Skeleton class="h-3 w-16" />
                    </div>
                  </div>
                  <Skeleton class="hidden h-4 w-20 sm:block" />
                  <Skeleton class="hidden h-4 w-36 sm:block" />
                  <Skeleton class="hidden h-5 w-12 rounded-full sm:block" />
                  <div class="flex justify-end">
                    <Skeleton class="h-8 w-8 rounded-md" />
                  </div>
                </div>
              {/each}
            </div>
          {/snippet}

          <div class="space-y-3 md:hidden">
            {#each users as user (user.id)}
              <article
                class={`rounded-xl border bg-gradient-to-br p-4 shadow-sm transition-colors ${
                  isSelfRow(user.id)
                    ? "border-primary/30 from-primary/10 to-card"
                    : "from-card to-muted/30 hover:border-primary/20"
                }`}
              >
                <div class="flex items-start gap-3">
                  <UserAvatar
                    class="size-9"
                    src={user.avatar_url ?? ""}
                    alt={user.display_name}
                    email={user.email}
                    displayName={user.display_name}
                    id={user.id}
                  />
                  <div class="min-w-0 flex-1">
                    <TruncateText text={user.display_name} class="font-medium leading-tight" />
                    <div class="mt-2 flex flex-wrap items-center gap-2">
                      {#if user.is_active}
                        <Badge class="border-success/30 bg-success/10 text-success">启用</Badge>
                      {:else}
                        <Badge variant="destructive">禁用</Badge>
                      {/if}
                      {#if isSelfRow(user.id)}
                        <Badge variant="secondary">当前账号</Badge>
                      {/if}
                    </div>
                  </div>
                  <div class="shrink-0">
                    <UserRowActions
                      row={user}
                      currentUserId={$auth.user?.sub}
                      {deletingUserId}
                      onEdit={openEditSheet}
                      onDelete={requestDelete}
                    />
                  </div>
                </div>

                <dl class="mt-3 space-y-2 border-t pt-3 text-sm">
                  <div class="flex items-center justify-between gap-3">
                    <dt class="text-muted-foreground shrink-0">用户名</dt>
                    <dd class="min-w-0 flex-1 text-right">
                      <TruncateText
                        text={user.username ?? "-"}
                        class="text-foreground text-right"
                      />
                    </dd>
                  </div>
                  <div class="flex items-center justify-between gap-3">
                    <dt class="text-muted-foreground shrink-0">邮箱</dt>
                    <dd class="min-w-0 flex-1 text-right">
                      <TruncateText text={user.email} class="text-foreground text-right" />
                    </dd>
                  </div>
                </dl>
              </article>
            {/each}
          </div>

          <div class="hidden md:block">
            <Table.Root class="table-fixed">
              <Table.Header>
                <Table.Row>
                  <Table.Head class="w-[34%]">用户</Table.Head>
                  <Table.Head class="w-[20%]">用户名</Table.Head>
                  <Table.Head class="w-[28%]">邮箱</Table.Head>
                  <Table.Head class="w-[10%]">状态</Table.Head>
                  <Table.Head class="w-[8%]"></Table.Head>
                </Table.Row>
              </Table.Header>

              <Table.Body>
                {#each users as user (user.id)}
                  <Table.Row class={isSelfRow(user.id) ? "bg-muted/40" : "hover:bg-muted/20"}>
                    <Table.Cell>
                      <div class="flex min-w-0 items-center gap-3">
                        <UserAvatar
                          src={user.avatar_url ?? ""}
                          alt={user.display_name}
                          email={user.email}
                          displayName={user.display_name}
                          id={user.id}
                        />
                        <div class="flex min-w-0 flex-col">
                          <TruncateText
                            text={user.display_name}
                            class="font-medium leading-tight"
                          />
                          {#if isSelfRow(user.id)}
                            <span class="text-muted-foreground text-xs">当前账号</span>
                          {/if}
                        </div>
                      </div>
                    </Table.Cell>
                    <Table.Cell class="text-muted-foreground">
                      <TruncateText text={user.username ?? "-"} />
                    </Table.Cell>
                    <Table.Cell class="text-muted-foreground">
                      <TruncateText text={user.email} />
                    </Table.Cell>
                    <Table.Cell>
                      {#if user.is_active}
                        <Badge class="border-success/30 bg-success/10 text-success">启用</Badge>
                      {:else}
                        <Badge variant="destructive">禁用</Badge>
                      {/if}
                    </Table.Cell>
                    <Table.Cell>
                      <UserRowActions
                        row={user}
                        currentUserId={$auth.user?.sub}
                        {deletingUserId}
                        onEdit={openEditSheet}
                        onDelete={requestDelete}
                      />
                    </Table.Cell>
                  </Table.Row>
                {/each}
              </Table.Body>
            </Table.Root>
          </div>
        </LoadStatePanel>
      </Card.Content>
    </Card.Root>
  {/if}
</div>

<!-- 删除确认对话框 -->
<AlertDialog.Root bind:open={deleteDialogOpen}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>确认删除用户</AlertDialog.Title>
      <AlertDialog.Description>
        确定要删除用户「{deleteTarget?.display_name ??
          ""}」吗？删除后用户将从默认列表中隐藏（逻辑删除，可恢复）。
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel
        onclick={() => {
          deleteTarget = null;
        }}>取消</AlertDialog.Cancel
      >
      <AlertDialog.Action
        class="bg-destructive text-white hover:bg-destructive/90"
        onclick={() => {
          void confirmDelete();
        }}
      >
        确认删除
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>

<!-- 新增/编辑用户侧边抽屉 -->
<Sheet.Root bind:open={sheetOpen}>
  <Sheet.Content class="sm:max-w-2xl overflow-y-auto">
    <Sheet.Header>
      <Sheet.Title>{sheetTitle}</Sheet.Title>
      <Sheet.Description>
        {sheetMode === "create" ? "创建新用户，默认启用。" : "编辑用户信息，可切换是否启用。"}
      </Sheet.Description>
    </Sheet.Header>

    <form
      class="mt-1 grid gap-3 px-4 pb-4 md:grid-cols-2 [&>*]:min-w-0"
      onsubmit={(event: SubmitEvent) => {
        event.preventDefault();
        void submitSheet();
      }}
    >
      <UserProfileFields
        bind:draft
        errors={sheetFieldErrors.errors}
        disabled={sheetSubmitting}
        idPrefix="user"
      />

      <Field.Field data-invalid={invalidSheet("username") || undefined}>
        <Field.Label for="user_username">用户名</Field.Label>
        <Input
          id="user_username"
          placeholder="可选，用于登录"
          bind:value={draft.username}
          disabled={sheetSubmitting}
          aria-invalid={invalidSheet("username")}
        />
        <Field.Error errors={sheetErrorItems("username")} />
      </Field.Field>

      {#if sheetMode === "edit"}
        <Field.Field
          class="md:col-span-2 min-w-0"
          data-invalid={invalidSheet("is_active") || undefined}
        >
          <div class="flex items-center justify-between rounded-md border px-3 py-2">
            <div>
              <Field.Label for="user_is_active">启用状态</Field.Label>
              <Field.Description>禁用与删除是两种状态；删除会从默认列表隐藏。</Field.Description>
            </div>

            <Switch
              id="user_is_active"
              bind:checked={draft.is_active}
              disabled={sheetSubmitting || (selectedUser ? isSelfRow(selectedUser.id) : false)}
              aria-invalid={invalidSheet("is_active")}
            />
          </div>
          <Field.Error errors={sheetErrorItems("is_active")} />
        </Field.Field>
      {/if}

      <FormDialogShell
        submitting={sheetSubmitting}
        submitLabel={sheetSubmitLabel}
        submittingLabel="保存中..."
        onCancel={closeSheet}
      />
    </form>
  </Sheet.Content>
</Sheet.Root>
