<script lang="ts">
  import { goto } from "$app/navigation";
  import { toast } from "svelte-sonner";
  import { createSessionHandler } from "$lib/api/generated/client";
  import { CreateSessionRequest } from "$lib/api/generated/schemas";
  import {
    detailsToFieldErrors,
    hasFieldError,
    mergeFieldErrors,
    toFieldErrorItems,
    type FieldErrors,
    zodErrorToFieldErrors,
  } from "$lib/shared/forms/field-errors";
  import { ApiError } from "$lib/api/mutator";
  import { auth } from "$lib/features/auth/state/auth";
  import PasswordInput from "$lib/shared/components/password-input.svelte";
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import * as Card from "$lib/shadcn/components/ui/card/index.js";
  import * as Field from "$lib/shadcn/components/ui/field/index.js";
  import { Input } from "$lib/shadcn/components/ui/input/index.js";

  let username = $state("");
  let password = $state("");
  let submitting = $state(false);
  let fieldErrors = $state<FieldErrors>({});

  function invalid(...keys: string[]): boolean {
    return hasFieldError(fieldErrors, ...keys);
  }

  function errorItems(...keys: string[]) {
    return toFieldErrorItems(fieldErrors, ...keys);
  }

  async function submit() {
    const u = username.trim();
    const p = password.trim();

    const parsed = CreateSessionRequest.safeParse({ username: u, password: p });
    if (!parsed.success) {
      fieldErrors = zodErrorToFieldErrors(parsed.error);
      return;
    }

    fieldErrors = {};

    submitting = true;
    try {
      const res = await createSessionHandler({ username: u, password: p });
      auth.login(res.token);
      await goto("/settings");
    } catch (e) {
      if (e instanceof ApiError) {
        fieldErrors = mergeFieldErrors(fieldErrors, detailsToFieldErrors(e.body?.details));
        if (invalid("username", "password")) {
          return;
        }
      }
      toast.error(e instanceof Error ? e.message : "登录失败");
    } finally {
      submitting = false;
    }
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
      <Field.Field data-invalid={invalid("username") || undefined}>
        <Field.Label for="username">用户名</Field.Label>
        <Input
          id="username"
          bind:value={username}
          autocomplete="username"
          disabled={submitting}
          aria-invalid={invalid("username")}
        />
        <Field.Error errors={errorItems("username")} />
      </Field.Field>

      <Field.Field data-invalid={invalid("password") || undefined}>
        <Field.Label for="password">密码</Field.Label>
        <PasswordInput
          id="password"
          bind:value={password}
          autocomplete="current-password"
          disabled={submitting}
          aria-invalid={invalid("password")}
        />
        <Field.Error errors={errorItems("password")} />
      </Field.Field>

      <Button class="w-full" type="submit" disabled={submitting}>
        {submitting ? "登录中..." : "登录"}
      </Button>
    </form>
  </Card.Content>
</Card.Root>
