<script lang="ts">
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";
  import { ApiError } from "$lib/api/mutator";
  import {
    getSettingsHandler,
    patchSettingsHandler,
    type PatchSettingsRequest as PatchSettingsRequestDto,
    type SettingsResponse,
  } from "$lib/api/generated/client";
  import { PatchSettingsRequest as PatchSettingsRequestSchema } from "$lib/api/generated/schemas";
  import { type FieldErrors, zodErrorToFieldErrors } from "$lib/shared/forms/field-errors";
  import { useFieldErrors } from "$lib/shared/forms/use-field-errors.svelte";
  import PasswordInput from "$lib/shared/components/password-input.svelte";
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
  const settingsFieldErrors = useFieldErrors<string>();

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
    return settingsFieldErrors.invalid(...keys);
  }

  function settingsErrorItems(...keys: string[]) {
    return settingsFieldErrors.items(...keys);
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
      settingsFieldErrors.setErrors(localErrors);
      return;
    }

    settingsFieldErrors.clearErrors();
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
        settingsFieldErrors.clearErrors();
        return;
      }

      const payloadCheck = PatchSettingsRequestSchema.safeParse(payload);
      if (!payloadCheck.success) {
        settingsFieldErrors.setErrors(zodErrorToFieldErrors(payloadCheck.error));
        return;
      }

      const updated = await patchSettingsHandler(payload);
      settings = updated;
      checkIntervalSecs = String(updated.app.check_interval_secs);
      welcomeMessage = updated.app.welcome_message;
      exampleApiBase = updated.integrations.example_api_base;
      exampleApiKey = "";
      settingsFieldErrors.clearErrors();
    } catch (e) {
      if (e instanceof ApiError) {
        settingsFieldErrors.mergeApiDetails(e.body?.details);
        if (Object.keys(settingsFieldErrors.errors).length > 0) {
          return;
        }
      }
      toast.error(e instanceof Error ? e.message : "保存失败");
    } finally {
      saving = false;
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
</div>
