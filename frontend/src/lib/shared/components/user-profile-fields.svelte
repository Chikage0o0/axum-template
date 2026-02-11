<script lang="ts">
  import {
    hasFieldError,
    toFieldErrorItems,
    type FieldErrors,
  } from "$lib/shared/forms/field-errors";
  import * as Field from "$lib/shadcn/components/ui/field/index.js";
  import { Input } from "$lib/shadcn/components/ui/input/index.js";

  type UserProfileDraft = {
    display_name: string;
    email: string;
    phone: string;
    avatar_url: string;
  };

  let {
    draft = $bindable<UserProfileDraft>(),
    errors = {},
    disabled = false,
    idPrefix = "user-profile",
    class: className = "",
  }: {
    draft: UserProfileDraft;
    errors?: FieldErrors;
    disabled?: boolean;
    idPrefix?: string;
    class?: string;
  } = $props();

  function invalid(...keys: string[]): boolean {
    return hasFieldError(errors, ...keys);
  }

  function errorItems(...keys: string[]) {
    return toFieldErrorItems(errors, ...keys);
  }

  function fieldId(key: string): string {
    return `${idPrefix}_${key}`;
  }
</script>

<div class={`contents ${className}`}>
  <Field.Field data-invalid={invalid("display_name") || undefined}>
    <Field.Label for={fieldId("display_name")}>显示名称 *</Field.Label>
    <Input
      id={fieldId("display_name")}
      bind:value={draft.display_name}
      placeholder="例如：张三"
      {disabled}
      aria-invalid={invalid("display_name")}
    />
    <Field.Error errors={errorItems("display_name")} />
  </Field.Field>

  <Field.Field data-invalid={invalid("email") || undefined}>
    <Field.Label for={fieldId("email")}>邮箱 *</Field.Label>
    <Input
      id={fieldId("email")}
      type="email"
      bind:value={draft.email}
      placeholder="user@example.com"
      {disabled}
      aria-invalid={invalid("email")}
    />
    <Field.Error errors={errorItems("email")} />
  </Field.Field>

  <Field.Field data-invalid={invalid("phone") || undefined}>
    <Field.Label for={fieldId("phone")}>手机号</Field.Label>
    <Input
      id={fieldId("phone")}
      bind:value={draft.phone}
      placeholder="可选"
      {disabled}
      aria-invalid={invalid("phone")}
    />
    <Field.Error errors={errorItems("phone")} />
  </Field.Field>

  <Field.Field class="md:col-span-2 min-w-0" data-invalid={invalid("avatar_url") || undefined}>
    <Field.Label for={fieldId("avatar_url")}>头像链接</Field.Label>
    <Input
      id={fieldId("avatar_url")}
      bind:value={draft.avatar_url}
      placeholder="https://example.com/avatar.png"
      {disabled}
      aria-invalid={invalid("avatar_url")}
    />
    <Field.Error errors={errorItems("avatar_url")} />
  </Field.Field>
</div>
