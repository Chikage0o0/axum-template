<script lang="ts">
  import { toast } from "svelte-sonner";
  import {
    getCurrentUserHandler,
    patchCurrentUserHandler,
    type UserResponse,
  } from "$lib/api/generated/client";
  import { CreateUserRequest as CreateUserRequestSchema } from "$lib/api/generated/schemas";
  import { PatchCurrentUserRequest as PatchCurrentUserRequestSchema } from "$lib/api/generated/schemas";
  import { zodErrorToFieldErrors } from "$lib/shared/forms/field-errors";
  import { useFieldErrors } from "$lib/shared/forms/use-field-errors.svelte";
  import { useApiFormSubmit } from "$lib/shared/forms/use-api-form-submit.svelte";
  import { auth } from "$lib/features/auth/state/auth";
  import {
    buildUserPatchPayload,
    toAuthUser,
    type CurrentUserDraft,
  } from "$lib/features/auth/model/user-helpers";
  import UserProfileFields from "$lib/shared/components/user-profile-fields.svelte";
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import * as Dialog from "$lib/shadcn/components/ui/dialog/index.js";

  let {
    open = $bindable(false),
  }: {
    open: boolean;
  } = $props();

  let currentUser = $state<UserResponse | null>(null);
  let loadingCurrentUser = $state(false);
  let savingCurrentUser = $state(false);
  const fieldErrors = useFieldErrors<string>();
  let draft = $state<CurrentUserDraft>({
    display_name: "",
    email: "",
    phone: "",
    avatar_url: "",
  });

  const apiSubmit = useApiFormSubmit();

  function syncDraft(nextUser: UserResponse) {
    draft = {
      display_name: nextUser.display_name,
      email: nextUser.email,
      phone: nextUser.phone ?? "",
      avatar_url: nextUser.avatar_url ?? "",
    };
  }

  async function loadCurrentUser() {
    loadingCurrentUser = true;
    fieldErrors.clearErrors();
    try {
      const me = await getCurrentUserHandler();
      currentUser = me;
      syncDraft(me);
    } catch (e) {
      currentUser = null;
      toast.error(e instanceof Error ? e.message : "加载当前用户信息失败");
    } finally {
      loadingCurrentUser = false;
    }
  }

  export async function openDialog() {
    open = true;
    await loadCurrentUser();
  }

  async function submit() {
    if (!currentUser) {
      toast.error("当前用户不存在，无法保存");
      return;
    }
    const userToUpdate = currentUser;

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
    fieldErrors.setErrors(draftCheck.success ? {} : zodErrorToFieldErrors(draftCheck.error));
    if (Object.keys(fieldErrors.errors).length > 0) {
      return;
    }

    const result = buildUserPatchPayload({
      mode: "self-edit",
      current: userToUpdate,
      draft,
    });
    if (!result.ok) {
      if (result.message.includes("display_name")) {
        fieldErrors.setErrors({ display_name: [result.message] });
      } else if (result.message.includes("email")) {
        fieldErrors.setErrors({ email: [result.message] });
      } else {
        toast.warning(result.message);
      }
      return;
    }

    const payloadCheck = PatchCurrentUserRequestSchema.safeParse(result.payload);
    if (!payloadCheck.success) {
      fieldErrors.setErrors(zodErrorToFieldErrors(payloadCheck.error));
      return;
    }

    await apiSubmit.run(
      async () => {
        const updated = await patchCurrentUserHandler(result.payload);
        currentUser = updated;
        syncDraft(updated);
        fieldErrors.clearErrors();

        const mapped = toAuthUser(updated);
        if (mapped) {
          auth.syncUser(mapped);
        }

        toast.success("个人信息已更新");
        open = false;
      },
      {
        setSubmitting(next) {
          savingCurrentUser = next;
        },
        onFieldErrors(details) {
          fieldErrors.mergeApiDetails(details);
          return Object.keys(fieldErrors.errors).length > 0;
        },
        onUnknownError(error) {
          toast.error(error instanceof Error ? error.message : "更新个人信息失败");
        },
      },
    );
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="sm:max-w-xl">
    <Dialog.Header>
      <Dialog.Title>编辑当前用户信息</Dialog.Title>
    </Dialog.Header>

    <form
      class="grid gap-3 md:grid-cols-2"
      onsubmit={(event: SubmitEvent) => {
        event.preventDefault();
        void submit();
      }}
    >
      <UserProfileFields
        bind:draft
        errors={fieldErrors.errors}
        disabled={savingCurrentUser || loadingCurrentUser || !currentUser}
        idPrefix="sidebar"
      />

      <div class="md:col-span-2 flex items-center justify-between gap-2">
        <p class="text-muted-foreground text-xs">
          {loadingCurrentUser ? "正在加载当前用户信息..." : ""}
        </p>
        <div class="flex items-center gap-2">
          <Button
            type="button"
            variant="outline"
            disabled={savingCurrentUser || loadingCurrentUser || !currentUser}
            onclick={() => {
              if (currentUser) {
                syncDraft(currentUser);
                fieldErrors.clearErrors();
              }
            }}
          >
            重置
          </Button>
          <Button type="submit" disabled={savingCurrentUser || loadingCurrentUser || !currentUser}>
            {savingCurrentUser ? "保存中..." : "保存修改"}
          </Button>
        </div>
      </div>
    </form>
  </Dialog.Content>
</Dialog.Root>
