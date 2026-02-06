<script lang="ts">
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";
  import { ApiError } from "$lib/api/mutator";
  import * as Alert from "$lib/shadcn/components/ui/alert/index.js";
  import { getCurrentUserHandler, patchUserHandler } from "$lib/api/generated/client";
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
  } from "$lib/forms/field-errors";
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import * as Card from "$lib/shadcn/components/ui/card/index.js";
  import * as Field from "$lib/shadcn/components/ui/field/index.js";
  import { Input } from "$lib/shadcn/components/ui/input/index.js";
  import { auth } from "$lib/stores/auth";
  import {
    buildCurrentUserPatchPayload,
    toAuthUser,
    type CurrentUserDraft,
    type User,
  } from "$lib/utils/user-helpers";

  let loading = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let currentUser = $state<User | null>(null);
  let fieldErrors = $state<FieldErrors>({});

  let draft = $state<CurrentUserDraft>({
    display_name: "",
    email: "",
    phone: "",
    avatar_url: "",
  });

  function syncDraftFromUser(user: User) {
    draft = {
      display_name: user.display_name,
      email: user.email,
      phone: user.phone ?? "",
      avatar_url: user.avatar_url ?? "",
    };
  }

  function invalid(...keys: string[]): boolean {
    return hasFieldError(fieldErrors, ...keys);
  }

  function errorItems(...keys: string[]) {
    return toFieldErrorItems(fieldErrors, ...keys);
  }

  async function reloadCurrentUser() {
    loading = true;
    error = null;
    fieldErrors = {};
    try {
      const me = await getCurrentUserHandler();
      currentUser = me;
      syncDraftFromUser(me);

      const mapped = toAuthUser(me);
      if (mapped) {
        auth.syncUser(mapped);
      }
    } catch (e) {
      currentUser = null;
      error = e instanceof Error ? e.message : "加载当前用户信息失败";
    } finally {
      loading = false;
    }
  }

  async function handleSubmit() {
    if (!currentUser) {
      toast.error("当前用户不存在，无法保存");
      return;
    }

    const draftCheck = CreateUserRequestSchema.pick({
      display_name: true,
      email: true,
      phone: true,
      avatar_url: true,
    }).safeParse({
      display_name: draft.display_name.trim(),
      email: draft.email.trim(),
      phone: draft.phone.trim() || undefined,
      avatar_url: draft.avatar_url.trim() || undefined,
    });
    fieldErrors = draftCheck.success ? {} : zodErrorToFieldErrors(draftCheck.error);
    if (Object.keys(fieldErrors).length > 0) {
      return;
    }

    const result = buildCurrentUserPatchPayload(currentUser, draft);
    if (!result.ok) {
      if (result.message.includes("display_name")) {
        fieldErrors = { display_name: [result.message] };
      } else if (result.message.includes("email")) {
        fieldErrors = { email: [result.message] };
      } else {
        toast.warning(result.message);
      }
      return;
    }

    const payloadCheck = PatchUserRequestSchema.safeParse(result.payload);
    if (!payloadCheck.success) {
      fieldErrors = zodErrorToFieldErrors(payloadCheck.error);
      return;
    }

    saving = true;
    try {
      const updated = await patchUserHandler(currentUser.id, result.payload);
      currentUser = updated;
      syncDraftFromUser(updated);

      const mapped = toAuthUser(updated);
      if (mapped) {
        auth.syncUser(mapped);
      }

      toast.success("个人信息已更新");
    } catch (e) {
      if (e instanceof ApiError) {
        const mapped = detailsToFieldErrors(e.body?.details);
        fieldErrors = mergeFieldErrors(fieldErrors, mapped);
        if (Object.keys(mapped).length > 0) {
          return;
        }
      }
      toast.error(e instanceof Error ? e.message : "更新个人信息失败");
    } finally {
      saving = false;
    }
  }

  onMount(() => {
    void reloadCurrentUser();
  });
</script>

<div class="space-y-6">
  <div class="flex flex-wrap items-end justify-between gap-3">
    <div>
      <h1 class="text-2xl font-semibold tracking-tight">个人信息</h1>
      <p class="text-muted-foreground mt-1 text-sm">仅支持修改当前登录用户信息。</p>
    </div>
    <Button variant="outline" disabled={loading || saving} onclick={reloadCurrentUser}>
      {loading ? "刷新中..." : "刷新"}
    </Button>
  </div>

  {#if error}
    <Alert.Root variant="destructive">
      <Alert.Title>加载失败</Alert.Title>
      <Alert.Description>{error}</Alert.Description>
    </Alert.Root>
  {/if}

  <Card.Root>
    <Card.Header>
      <Card.Title>编辑当前用户</Card.Title>
    </Card.Header>
    <Card.Content>
      <form
        class="grid gap-3 md:grid-cols-2"
        onsubmit={(event: SubmitEvent) => {
          event.preventDefault();
          void handleSubmit();
        }}
      >
        <Field.Field data-invalid={invalid("display_name") || undefined}>
          <Field.Label for="display_name">display_name *</Field.Label>
          <Input
            id="display_name"
            bind:value={draft.display_name}
            disabled={saving || !currentUser}
            aria-invalid={invalid("display_name")}
          />
          <Field.Error errors={errorItems("display_name")} />
        </Field.Field>
        <Field.Field data-invalid={invalid("email") || undefined}>
          <Field.Label for="email">email *</Field.Label>
          <Input
            id="email"
            bind:value={draft.email}
            disabled={saving || !currentUser}
            aria-invalid={invalid("email")}
          />
          <Field.Error errors={errorItems("email")} />
        </Field.Field>
        <Field.Field data-invalid={invalid("phone") || undefined}>
          <Field.Label for="phone">phone</Field.Label>
          <Input
            id="phone"
            bind:value={draft.phone}
            disabled={saving || !currentUser}
            aria-invalid={invalid("phone")}
          />
          <Field.Error errors={errorItems("phone")} />
        </Field.Field>
        <Field.Field data-invalid={invalid("avatar_url") || undefined}>
          <Field.Label for="avatar_url">avatar_url</Field.Label>
          <Input
            id="avatar_url"
            bind:value={draft.avatar_url}
            disabled={saving || !currentUser}
            aria-invalid={invalid("avatar_url")}
          />
          <Field.Error errors={errorItems("avatar_url")} />
        </Field.Field>

        <div class="md:col-span-2 flex justify-end gap-2">
          <Button
            type="button"
            variant="outline"
            disabled={saving || !currentUser}
            onclick={() => {
              if (currentUser) syncDraftFromUser(currentUser);
            }}
          >
            重置
          </Button>
          <Button type="submit" disabled={saving || !currentUser}>
            {saving ? "保存中..." : "保存修改"}
          </Button>
        </div>
      </form>
    </Card.Content>
  </Card.Root>
</div>
