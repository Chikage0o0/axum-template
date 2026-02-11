<script lang="ts">
  import { toast } from "svelte-sonner";
  import { patchCurrentUserPasswordHandler } from "$lib/api/generated/client";
  import { useFieldErrors } from "$lib/shared/forms/use-field-errors.svelte";
  import { useApiFormSubmit } from "$lib/shared/forms/use-api-form-submit.svelte";
  import { validatePasswordChangeForm } from "$lib/shared/forms/password-change";
  import PasswordInput from "$lib/shared/components/password-input.svelte";
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import * as Dialog from "$lib/shadcn/components/ui/dialog/index.js";
  import * as Field from "$lib/shadcn/components/ui/field/index.js";

  let {
    open = $bindable(false),
    onSuccess,
  }: {
    open: boolean;
    onSuccess: () => void | Promise<void>;
  } = $props();

  let currentPassword = $state("");
  let newPassword = $state("");
  let confirmPassword = $state("");
  let changingPassword = $state(false);
  const fieldErrors = useFieldErrors<"current_password" | "new_password" | "confirm_password">();
  const apiSubmit = useApiFormSubmit();

  function reset() {
    currentPassword = "";
    newPassword = "";
    confirmPassword = "";
    fieldErrors.clearErrors();
  }

  export function openDialog() {
    reset();
    open = true;
  }

  async function submit() {
    const { payload, errors } = validatePasswordChangeForm({
      currentPassword,
      newPassword,
      confirmPassword,
    });
    fieldErrors.setErrors(errors);
    if (!payload) {
      return;
    }

    await apiSubmit.run(
      async () => {
        await patchCurrentUserPasswordHandler(payload);
        toast.success("密码已更新，请重新登录");
        open = false;
        reset();
        await onSuccess();
      },
      {
        setSubmitting(next) {
          changingPassword = next;
        },
        onFieldErrors(details) {
          fieldErrors.mergeApiDetails(details);
          return Object.keys(fieldErrors.errors).length > 0;
        },
        onUnknownError(error) {
          toast.error(error instanceof Error ? error.message : "修改失败");
        },
      },
    );
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>修改当前用户密码</Dialog.Title>
      <Dialog.Description>修改成功后会退出登录，请使用新密码重新登录。</Dialog.Description>
    </Dialog.Header>

    <form
      class="grid gap-4"
      onsubmit={(event: SubmitEvent) => {
        event.preventDefault();
        void submit();
      }}
    >
      <Field.Field data-invalid={fieldErrors.invalid("current_password") || undefined}>
        <Field.Label for="sidebar_current_password">当前用户密码</Field.Label>
        <PasswordInput
          id="sidebar_current_password"
          bind:value={currentPassword}
          disabled={changingPassword}
          autocomplete="current-password"
          aria-invalid={fieldErrors.invalid("current_password")}
        />
        <Field.Error errors={fieldErrors.items("current_password")} />
      </Field.Field>

      <Field.Field data-invalid={fieldErrors.invalid("new_password") || undefined}>
        <Field.Label for="sidebar_new_password">新密码</Field.Label>
        <Field.Description>至少 8 位</Field.Description>
        <PasswordInput
          id="sidebar_new_password"
          bind:value={newPassword}
          disabled={changingPassword}
          autocomplete="new-password"
          aria-invalid={fieldErrors.invalid("new_password")}
        />
        <Field.Error errors={fieldErrors.items("new_password")} />
      </Field.Field>

      <Field.Field data-invalid={fieldErrors.invalid("confirm_password") || undefined}>
        <Field.Label for="sidebar_confirm_password">确认新密码</Field.Label>
        <PasswordInput
          id="sidebar_confirm_password"
          bind:value={confirmPassword}
          disabled={changingPassword}
          autocomplete="new-password"
          aria-invalid={fieldErrors.invalid("confirm_password")}
        />
        <Field.Error errors={fieldErrors.items("confirm_password")} />
      </Field.Field>

      <div class="flex justify-end">
        <Button type="submit" disabled={changingPassword}>
          {changingPassword ? "更新中..." : "更新密码"}
        </Button>
      </div>
    </form>
  </Dialog.Content>
</Dialog.Root>
