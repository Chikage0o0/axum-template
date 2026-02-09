<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";
  import { ApiError } from "$lib/api/mutator";
  import {
    getSettingsHandler,
    patchCurrentUserPasswordHandler,
    patchSettingsHandler,
    type PatchSettingsRequest as PatchSettingsRequestDto,
    type SettingsResponse,
  } from "$lib/api/generated/client";
  import { PatchSettingsRequest as PatchSettingsRequestSchema } from "$lib/api/generated/schemas";
  import { validatePasswordChangeForm } from "$lib/forms/password-change";
  import {
    detailsToFieldErrors,
    hasFieldError,
    mergeFieldErrors,
    toFieldErrorItems,
    type FieldErrors,
    zodErrorToFieldErrors,
  } from "$lib/forms/field-errors";
  import { auth } from "$lib/stores/auth";
  import PasswordInput from "$lib/components/password-input.svelte";
  import * as Alert from "$lib/shadcn/components/ui/alert/index.js";
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import * as Card from "$lib/shadcn/components/ui/card/index.js";
  import * as Field from "$lib/shadcn/components/ui/field/index.js";
  import { Input } from "$lib/shadcn/components/ui/input/index.js";

  let settings = $state<SettingsResponse | null>(null);
  let loading = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);

  let checkIntervalSecs = $state("3600");
  let welcomeMessage = $state("");
  let exampleApiBase = $state("");
  let exampleApiKey = $state("");
  let settingsFieldErrors = $state<FieldErrors>({});

  async function reload() {
    error = null;
    loading = true;
    try {
      const s = await getSettingsHandler();
      settings = s;
      checkIntervalSecs = String(s.app.check_interval_secs);
      welcomeMessage = s.app.welcome_message;
      exampleApiBase = s.integrations.example_api_base;
      exampleApiKey = "";
    } catch (e) {
      error = e instanceof Error ? e.message : "加载失败";
    } finally {
      loading = false;
    }
  }

  function invalidSettings(...keys: string[]): boolean {
    return hasFieldError(settingsFieldErrors, ...keys);
  }

  function settingsErrorItems(...keys: string[]) {
    return toFieldErrorItems(settingsFieldErrors, ...keys);
  }

  async function save() {
    if (!settings) {
      toast.error("配置尚未加载");
      return;
    }

    const localErrors: FieldErrors = {};
    const intervalRaw = checkIntervalSecs.trim();
    const intervalNum = Number(intervalRaw);
    const welcomeMessageTrimmed = welcomeMessage.trim();
    const apiBaseTrimmed = exampleApiBase.trim();
    const apiKeyTrimmed = exampleApiKey.trim();

    if (!intervalRaw) {
      localErrors.check_interval_secs = ["check_interval_secs 不能为空"];
    } else if (!Number.isFinite(intervalNum)) {
      localErrors.check_interval_secs = ["check_interval_secs 必须是数字"];
    } else if (Math.trunc(intervalNum) < 10) {
      localErrors.check_interval_secs = ["check_interval_secs 不能小于 10"];
    }

    if (!welcomeMessageTrimmed) {
      localErrors.welcome_message = ["welcome_message 不能为空"];
    }

    if (!apiBaseTrimmed) {
      localErrors.example_api_base = ["example_api_base 不能为空"];
    }

    if (Object.keys(localErrors).length > 0) {
      settingsFieldErrors = localErrors;
      return;
    }

    settingsFieldErrors = {};
    saving = true;
    try {
      const payload: PatchSettingsRequestDto = {};
      const app: NonNullable<PatchSettingsRequestDto["app"]> = {};
      const integrations: NonNullable<PatchSettingsRequestDto["integrations"]> = {};

      const interval = Math.trunc(intervalNum);
      if (interval !== settings.app.check_interval_secs) app.check_interval_secs = interval;

      if (welcomeMessageTrimmed !== settings.app.welcome_message) {
        app.welcome_message = welcomeMessageTrimmed;
      }

      if (apiBaseTrimmed !== settings.integrations.example_api_base) {
        integrations.example_api_base = apiBaseTrimmed;
      }

      if (apiKeyTrimmed) integrations.example_api_key = apiKeyTrimmed;

      if (Object.keys(app).length) payload.app = app;
      if (Object.keys(integrations).length) payload.integrations = integrations;

      if (!Object.keys(payload).length) {
        settingsFieldErrors = {};
        return;
      }

      const payloadCheck = PatchSettingsRequestSchema.safeParse(payload);
      if (!payloadCheck.success) {
        settingsFieldErrors = zodErrorToFieldErrors(payloadCheck.error);
        return;
      }

      const updated = await patchSettingsHandler(payload);
      settings = updated;
      checkIntervalSecs = String(updated.app.check_interval_secs);
      welcomeMessage = updated.app.welcome_message;
      exampleApiBase = updated.integrations.example_api_base;
      exampleApiKey = "";
      settingsFieldErrors = {};
    } catch (e) {
      if (e instanceof ApiError) {
        const mapped = detailsToFieldErrors(e.body?.details);
        settingsFieldErrors = mergeFieldErrors(settingsFieldErrors, mapped);
        if (Object.keys(mapped).length > 0) {
          return;
        }
      }
      toast.error(e instanceof Error ? e.message : "保存失败");
    } finally {
      saving = false;
    }
  }

  let currentPassword = $state("");
  let newPassword = $state("");
  let confirmPassword = $state("");
  let changing = $state(false);
  let passwordFieldErrors = $state<FieldErrors>({});

  function invalidPassword(...keys: string[]): boolean {
    return hasFieldError(passwordFieldErrors, ...keys);
  }

  function passwordErrorItems(...keys: string[]) {
    return toFieldErrorItems(passwordFieldErrors, ...keys);
  }

  async function changePassword() {
    const { payload, errors } = validatePasswordChangeForm({
      currentPassword,
      newPassword,
      confirmPassword,
    });
    passwordFieldErrors = errors;

    if (!payload) {
      return;
    }

    changing = true;
    try {
      await patchCurrentUserPasswordHandler(payload);
      toast.success("密码已更新，请重新登录");
      auth.logout({ reason: "manual" });
      await goto("/login");
    } catch (e) {
      if (e instanceof ApiError) {
        const mapped = detailsToFieldErrors(e.body?.details);
        passwordFieldErrors = mergeFieldErrors(passwordFieldErrors, mapped);
        if (Object.keys(mapped).length > 0) {
          return;
        }
      }
      toast.error(e instanceof Error ? e.message : "修改失败");
    } finally {
      changing = false;
    }
  }

  onMount(() => {
    void reload();
  });
</script>

<div class="content-flow space-y-6">
  <div>
    <h1 class="text-2xl font-semibold tracking-tight">设置</h1>
  </div>

  {#if error}
    <Alert.Root variant="destructive">
      <Alert.Title>请求失败</Alert.Title>
      <Alert.Description>{error}</Alert.Description>
    </Alert.Root>
  {/if}

  <Card.Root id="runtime">
    <Card.Header class="space-y-3">
      <div class="flex flex-wrap items-center justify-between gap-3">
        <div>
          <Card.Title>运行期配置</Card.Title>
        </div>
        <div class="flex items-center gap-2">
          <Button variant="outline" disabled={loading || saving} onclick={reload}>
            {loading ? "加载中..." : "刷新"}
          </Button>
          <Button disabled={loading || saving} onclick={save}>
            {saving ? "保存中..." : "保存"}
          </Button>
        </div>
      </div>
    </Card.Header>
    <Card.Content>
      {#if settings}
        <div class="grid gap-4 sm:grid-cols-2">
          <Field.Field
            data-invalid={invalidSettings("check_interval_secs", "app.check_interval_secs") ||
              undefined}
          >
            <Field.Label for="check_interval_secs">app.check_interval_secs</Field.Label>
            <Input
              id="check_interval_secs"
              type="number"
              bind:value={checkIntervalSecs}
              aria-invalid={invalidSettings("check_interval_secs", "app.check_interval_secs")}
            />
            <Field.Error
              errors={settingsErrorItems("check_interval_secs", "app.check_interval_secs")}
            />
          </Field.Field>

          <Field.Field
            data-invalid={invalidSettings("example_api_base", "integrations.example_api_base") ||
              undefined}
          >
            <Field.Label for="example_api_base">integrations.example_api_base</Field.Label>
            <Input
              id="example_api_base"
              bind:value={exampleApiBase}
              aria-invalid={invalidSettings("example_api_base", "integrations.example_api_base")}
            />
            <Field.Error
              errors={settingsErrorItems("example_api_base", "integrations.example_api_base")}
            />
          </Field.Field>

          <Field.Field
            class="sm:col-span-2"
            data-invalid={invalidSettings("welcome_message", "app.welcome_message") || undefined}
          >
            <Field.Label for="welcome_message">app.welcome_message</Field.Label>
            <Input
              id="welcome_message"
              bind:value={welcomeMessage}
              aria-invalid={invalidSettings("welcome_message", "app.welcome_message")}
            />
            <Field.Error errors={settingsErrorItems("welcome_message", "app.welcome_message")} />
          </Field.Field>

          <Field.Field
            class="sm:col-span-2"
            data-invalid={invalidSettings("example_api_key", "integrations.example_api_key") ||
              undefined}
          >
            <Field.Label for="example_api_key">integrations.example_api_key</Field.Label>
            <Field.Description>留空不修改</Field.Description>
            <PasswordInput
              id="example_api_key"
              bind:value={exampleApiKey}
              placeholder={settings.integrations.example_api_key_is_set ? "已设置" : "未设置"}
              aria-invalid={invalidSettings("example_api_key", "integrations.example_api_key")}
            />
            <Field.Error
              errors={settingsErrorItems("example_api_key", "integrations.example_api_key")}
            />
          </Field.Field>
        </div>
      {:else}
        <p class="text-muted-foreground text-sm">尚未加载配置。</p>
      {/if}
    </Card.Content>
  </Card.Root>

  <Card.Root>
    <Card.Header>
      <Card.Title>修改当前用户密码</Card.Title>
    </Card.Header>
    <Card.Content>
      <form
        class="grid gap-4"
        onsubmit={(e: SubmitEvent) => {
          e.preventDefault();
          void changePassword();
        }}
      >
        <Field.Field data-invalid={invalidPassword("current_password") || undefined}>
          <Field.Label for="current_password">当前用户密码</Field.Label>
          <PasswordInput
            id="current_password"
            bind:value={currentPassword}
            disabled={changing}
            autocomplete="current-password"
            aria-invalid={invalidPassword("current_password")}
          />
          <Field.Error errors={passwordErrorItems("current_password")} />
        </Field.Field>

        <Field.Field data-invalid={invalidPassword("new_password") || undefined}>
          <Field.Label for="new_password">新密码</Field.Label>
          <Field.Description>至少 8 位</Field.Description>
          <PasswordInput
            id="new_password"
            bind:value={newPassword}
            disabled={changing}
            autocomplete="new-password"
            aria-invalid={invalidPassword("new_password")}
          />
          <Field.Error errors={passwordErrorItems("new_password")} />
        </Field.Field>

        <Field.Field data-invalid={invalidPassword("confirm_password") || undefined}>
          <Field.Label for="confirm_password">确认新密码</Field.Label>
          <PasswordInput
            id="confirm_password"
            bind:value={confirmPassword}
            disabled={changing}
            autocomplete="new-password"
            aria-invalid={invalidPassword("confirm_password")}
          />
          <Field.Error errors={passwordErrorItems("confirm_password")} />
        </Field.Field>

        <div class="flex justify-end">
          <Button type="submit" disabled={changing}>
            {changing ? "更新中..." : "更新密码"}
          </Button>
        </div>
      </form>
    </Card.Content>
  </Card.Root>
</div>
