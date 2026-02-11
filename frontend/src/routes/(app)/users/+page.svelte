<script lang="ts">
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";
  import EllipsisVerticalIcon from "@lucide/svelte/icons/ellipsis-vertical";
  import PencilIcon from "@lucide/svelte/icons/pencil";
  import RefreshCwIcon from "@lucide/svelte/icons/refresh-cw";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import UserPlusIcon from "@lucide/svelte/icons/user-plus";
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
  import {
    detailsToFieldErrors,
    hasFieldError,
    mergeFieldErrors,
    toFieldErrorItems,
    type FieldErrors,
    zodErrorToFieldErrors,
  } from "$lib/shared/forms/field-errors";
  import { auth } from "$lib/features/auth/state/auth";
  import { buildUserPatchPayload } from "$lib/features/auth/model/user-helpers";
  import * as Alert from "$lib/shadcn/components/ui/alert/index.js";
  import * as AlertDialog from "$lib/shadcn/components/ui/alert-dialog/index.js";
  import { Badge } from "$lib/shadcn/components/ui/badge/index.js";
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import * as Card from "$lib/shadcn/components/ui/card/index.js";
  import * as DropdownMenu from "$lib/shadcn/components/ui/dropdown-menu/index.js";
  import * as Empty from "$lib/shadcn/components/ui/empty/index.js";
  import * as Field from "$lib/shadcn/components/ui/field/index.js";
  import { Input } from "$lib/shadcn/components/ui/input/index.js";
  import * as Sheet from "$lib/shadcn/components/ui/sheet/index.js";
  import { Skeleton } from "$lib/shadcn/components/ui/skeleton/index.js";
  import { Switch } from "$lib/shadcn/components/ui/switch/index.js";
  import * as Table from "$lib/shadcn/components/ui/table/index.js";
  import * as Tooltip from "$lib/shadcn/components/ui/tooltip/index.js";
  import TruncateText from "$lib/shared/components/truncate-text.svelte";
  import UserAvatar from "$lib/shared/components/user-avatar.svelte";
  import UserProfileFields from "$lib/shared/components/user-profile-fields.svelte";

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
  let sheetFieldErrors = $state<FieldErrors>({});
  let selectedUser = $state<UserResponse | null>(null);
  let draft = $state<UserFormDraft>(emptyDraft());

  let deletingUserId = $state<string | null>(null);

  // AlertDialog 状态
  let deleteDialogOpen = $state(false);
  let deleteTarget = $state<UserResponse | null>(null);

  const sheetTitle = $derived(sheetMode === "create" ? "新增用户" : "编辑用户");

  function isSelfRow(userId: string): boolean {
    return $auth.user?.sub === userId;
  }

  function invalidSheet(...keys: string[]): boolean {
    return hasFieldError(sheetFieldErrors, ...keys);
  }

  function sheetErrorItems(...keys: string[]) {
    return toFieldErrorItems(sheetFieldErrors, ...keys);
  }

  function openCreateSheet() {
    sheetMode = "create";
    selectedUser = null;
    draft = emptyDraft();
    sheetFieldErrors = {};
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
    sheetFieldErrors = {};
    sheetOpen = true;
  }

  function closeSheet() {
    sheetOpen = false;
    sheetFieldErrors = {};
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
    sheetFieldErrors = {};

    if (sheetMode === "edit" && selectedUser && isSelfRow(selectedUser.id) && !draft.is_active) {
      sheetFieldErrors = { is_active: ["当前登录账号不允许自禁用"] };
      toast.error("当前登录账号不允许自禁用");
      return;
    }

    sheetSubmitting = true;
    try {
      if (sheetMode === "create") {
        const result = makeCreatePayloadFromDraft();
        if (!result.ok) {
          sheetFieldErrors = result.errors;
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
            sheetFieldErrors = result.errors;
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
        const mapped = detailsToFieldErrors(e.body?.details);
        sheetFieldErrors = mergeFieldErrors(sheetFieldErrors, mapped);
        if (Object.keys(mapped).length > 0) {
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
  <div class="flex flex-wrap items-center justify-between gap-3">
    <h1 class="text-2xl font-semibold tracking-tight">用户管理</h1>
    <div class="flex items-center gap-2">
      <Button variant="outline" disabled={listLoading} onclick={reloadUsers}>
        <RefreshCwIcon class={`size-4 ${listLoading ? "animate-spin" : ""}`} />
        {listLoading ? "刷新中..." : "刷新"}
      </Button>
      <Button onclick={openCreateSheet}>
        <UserPlusIcon class="size-4" />
        新增用户
      </Button>
    </div>
  </div>

  {#if permissionDenied}
    <Empty.Root>
      <Empty.Header>
        <Empty.Title>无权限访问用户管理</Empty.Title>
        <Empty.Description>仅管理员可访问此页面，请联系管理员分配权限。</Empty.Description>
      </Empty.Header>
    </Empty.Root>
  {:else}
    {#if listError}
      <Alert.Root variant="destructive">
        <Alert.Title>请求失败</Alert.Title>
        <Alert.Description>{listError}</Alert.Description>
      </Alert.Root>
    {/if}

    <Card.Root>
      <Card.Header class="space-y-2">
        <Card.Title>用户列表</Card.Title>
        <Card.Description>默认仅显示未删除用户；删除后用户将从此列表移除。</Card.Description>
      </Card.Header>

      <Card.Content>
        {#if listLoading}
          <!-- 骨架屏加载态 -->
          <div class="space-y-4">
            {#each Array.from({ length: 5 }, (_, index) => index) as index (index)}
              <div class="flex items-center gap-4">
                <Skeleton class="size-8 rounded-full" />
                <Skeleton class="h-4 w-24" />
                <Skeleton class="h-4 w-20" />
                <Skeleton class="h-4 w-36" />
                <Skeleton class="h-5 w-12 rounded-full" />
              </div>
            {/each}
          </div>
        {:else if users.length === 0}
          <Empty.Root class="min-h-44">
            <Empty.Header>
              <Empty.Title>暂无用户数据</Empty.Title>
              <Empty.Description>可以先新增一个用户。</Empty.Description>
            </Empty.Header>
            <Empty.Content>
              <Button onclick={openCreateSheet}>
                <UserPlusIcon class="size-4" />
                新增用户
              </Button>
            </Empty.Content>
          </Empty.Root>
        {:else}
          <Tooltip.Provider>
            <Table.Root class="table-fixed">
              <Table.Header>
                <Table.Row>
                  <Table.Head class="w-[30%]">用户</Table.Head>
                  <Table.Head class="w-[25%]">用户名</Table.Head>
                  <Table.Head class="w-[25%]">邮箱</Table.Head>
                  <Table.Head class="w-[10%]">状态</Table.Head>
                  <Table.Head class="w-[10%]"></Table.Head>
                </Table.Row>
              </Table.Header>

              <Table.Body>
                {#each users as user (user.id)}
                  <Table.Row class={isSelfRow(user.id) ? "bg-muted/40" : ""}>
                    <!-- 头像 + 显示名称 -->
                    <Table.Cell>
                      <div class="flex items-center gap-3 min-w-0">
                        <UserAvatar
                          src={user.avatar_url ?? ""}
                          alt={user.display_name}
                          email={user.email}
                          displayName={user.display_name}
                          id={user.id}
                        />
                        <div class="flex flex-col min-w-0">
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
                        <Badge
                          class="border-emerald-600/30 bg-emerald-500/10 text-emerald-600 dark:text-emerald-400"
                          >启用</Badge
                        >
                      {:else}
                        <Badge variant="destructive">禁用</Badge>
                      {/if}
                    </Table.Cell>
                    <Table.Cell>
                      <!-- 行操作下拉菜单 -->
                      <DropdownMenu.Root>
                        <DropdownMenu.Trigger>
                          {#snippet child({ props })}
                            <Button
                              {...props}
                              variant="ghost"
                              size="icon"
                              class="size-8"
                              aria-label="操作菜单"
                            >
                              <EllipsisVerticalIcon class="size-4" />
                            </Button>
                          {/snippet}
                        </DropdownMenu.Trigger>
                        <DropdownMenu.Content align="end">
                          <DropdownMenu.Item
                            disabled={deletingUserId === user.id}
                            onclick={() => openEditSheet(user)}
                          >
                            <PencilIcon class="size-4" />
                            编辑
                          </DropdownMenu.Item>
                          <DropdownMenu.Separator />
                          {#if isSelfRow(user.id)}
                            <Tooltip.Provider>
                              <Tooltip.Root>
                                <Tooltip.Trigger>
                                  {#snippet child({ props })}
                                    <div {...props}>
                                      <DropdownMenu.Item disabled>
                                        <Trash2Icon class="size-4" />
                                        删除
                                      </DropdownMenu.Item>
                                    </div>
                                  {/snippet}
                                </Tooltip.Trigger>
                                <Tooltip.Content>
                                  <p>当前登录账号不允许自删除</p>
                                </Tooltip.Content>
                              </Tooltip.Root>
                            </Tooltip.Provider>
                          {:else}
                            <DropdownMenu.Item
                              class="text-destructive focus:text-destructive"
                              disabled={deletingUserId === user.id}
                              onclick={() => requestDelete(user)}
                            >
                              <Trash2Icon class="size-4" />
                              {deletingUserId === user.id ? "删除中..." : "删除"}
                            </DropdownMenu.Item>
                          {/if}
                        </DropdownMenu.Content>
                      </DropdownMenu.Root>
                    </Table.Cell>
                  </Table.Row>
                {/each}
              </Table.Body>
            </Table.Root>
          </Tooltip.Provider>
        {/if}
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
        errors={sheetFieldErrors}
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

      <div class="md:col-span-2 flex justify-end gap-2 pt-1">
        <Button type="button" variant="outline" disabled={sheetSubmitting} onclick={closeSheet}>
          取消
        </Button>
        <Button type="submit" disabled={sheetSubmitting}>
          {sheetSubmitting ? "保存中..." : sheetMode === "create" ? "创建用户" : "保存修改"}
        </Button>
      </div>
    </form>
  </Sheet.Content>
</Sheet.Root>
