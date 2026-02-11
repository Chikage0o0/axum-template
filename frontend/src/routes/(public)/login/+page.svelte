<script lang="ts">
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { toast } from "svelte-sonner";
  import { createSessionHandler } from "$lib/api/generated/client";
  import { CreateSessionRequest } from "$lib/api/generated/schemas";
  import { useApiFormSubmit } from "$lib/shared/forms/use-api-form-submit.svelte";
  import { zodErrorToFieldErrors } from "$lib/shared/forms/field-errors";
  import { useFieldErrors } from "$lib/shared/forms/use-field-errors.svelte";
  import { auth } from "$lib/features/auth/state/auth";
  import PasswordInput from "$lib/shared/components/password-input.svelte";
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import * as Card from "$lib/shadcn/components/ui/card/index.js";
  import * as Field from "$lib/shadcn/components/ui/field/index.js";
  import { Input } from "$lib/shadcn/components/ui/input/index.js";

  let identifier = $state("");
  let password = $state("");
  let submitting = $state(false);
  const apiSubmit = useApiFormSubmit();
  const fieldErrors = useFieldErrors<"identifier" | "password">();

  async function submit() {
    const i = identifier.trim();
    const p = password.trim();

    const parsed = CreateSessionRequest.safeParse({ identifier: i, password: p });
    if (!parsed.success) {
      fieldErrors.setErrors(zodErrorToFieldErrors(parsed.error));
      return;
    }

    fieldErrors.clearErrors();

    await apiSubmit.run(
      async () => {
        const res = await createSessionHandler({ identifier: i, password: p });
        auth.login(res.token);
        await goto(resolve("/settings"));
      },
      {
        setSubmitting(next) {
          submitting = next;
        },
        onFieldErrors(details) {
          fieldErrors.mergeApiDetails(details);
          return fieldErrors.invalid("identifier", "password");
        },
        onUnknownError(error) {
          toast.error(error instanceof Error ? error.message : "登录失败");
        },
      },
    );
  }
</script>

<Card.Root>
  <Card.Header>
    <Card.Title>登录</Card.Title>
  </Card.Header>

  <Card.Content>
    <form
      class="space-y-4"
      onsubmit={(e: SubmitEvent) => {
        e.preventDefault();
        void submit();
      }}
    >
      <Field.Field data-invalid={fieldErrors.invalid("identifier") || undefined}>
        <Field.Label for="identifier">账号（邮箱/用户名/手机号）</Field.Label>
        <Input
          id="identifier"
          bind:value={identifier}
          autocomplete="username"
          disabled={submitting}
          aria-invalid={fieldErrors.invalid("identifier")}
        />
        <Field.Error errors={fieldErrors.items("identifier")} />
      </Field.Field>

      <Field.Field data-invalid={fieldErrors.invalid("password") || undefined}>
        <Field.Label for="password">密码</Field.Label>
        <PasswordInput
          id="password"
          bind:value={password}
          autocomplete="current-password"
          disabled={submitting}
          aria-invalid={fieldErrors.invalid("password")}
        />
        <Field.Error errors={fieldErrors.items("password")} />
      </Field.Field>

      <Button class="w-full" type="submit" disabled={submitting}>
        {submitting ? "登录中..." : "登录"}
      </Button>
    </form>
  </Card.Content>
</Card.Root>
