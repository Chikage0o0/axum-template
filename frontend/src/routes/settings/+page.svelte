<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";
  import { auth } from "$lib/stores/auth";
  import {
    changeAdminPassword,
    getSettings,
    patchSettings,
    type PatchSettingsRequest,
    type SettingsResponse,
  } from "$lib/utils/settings";
  import * as Alert from "$lib/shadcn/components/ui/alert/index.js";
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import * as Card from "$lib/shadcn/components/ui/card/index.js";
  import { Input } from "$lib/shadcn/components/ui/input/index.js";
  import { Label } from "$lib/shadcn/components/ui/label/index.js";

  let settings = $state<SettingsResponse | null>(null);
  let loading = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);

  let checkIntervalSecs = $state("3600");
  let welcomeMessage = $state("");
  let exampleApiBase = $state("");
  let exampleApiKey = $state("");

  async function reload() {
    error = null;
    loading = true;
    try {
      const s = await getSettings();
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

  function num(raw: string, label: string): number {
    const v = Number(raw);
    if (!Number.isFinite(v)) throw new Error(`${label} 必须是数字`);
    return v;
  }

  async function save() {
    if (!settings) {
      toast.error("配置尚未加载");
      return;
    }

    saving = true;
    try {
      const payload: PatchSettingsRequest = {};
      const app: NonNullable<PatchSettingsRequest["app"]> = {};
      const integrations: NonNullable<PatchSettingsRequest["integrations"]> = {};

      const interval = Math.trunc(num(checkIntervalSecs, "check_interval_secs"));
      if (interval < 10) throw new Error("check_interval_secs 不能小于 10");
      if (interval !== settings.app.check_interval_secs) app.check_interval_secs = interval;

      const msg = welcomeMessage.trim();
      if (!msg) throw new Error("welcome_message 不能为空");
      if (msg !== settings.app.welcome_message) app.welcome_message = msg;

      const base = exampleApiBase.trim();
      if (!base) throw new Error("example_api_base 不能为空");
      if (base !== settings.integrations.example_api_base) integrations.example_api_base = base;

      const key = exampleApiKey.trim();
      if (key) integrations.example_api_key = key;

      if (Object.keys(app).length) payload.app = app;
      if (Object.keys(integrations).length) payload.integrations = integrations;

      if (!Object.keys(payload).length) {
        return;
      }

      const updated = await patchSettings(payload);
      settings = updated;
      checkIntervalSecs = String(updated.app.check_interval_secs);
      welcomeMessage = updated.app.welcome_message;
      exampleApiBase = updated.integrations.example_api_base;
      exampleApiKey = "";
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "保存失败");
    } finally {
      saving = false;
    }
  }

  let currentPassword = $state("");
  let newPassword = $state("");
  let confirmPassword = $state("");
  let changing = $state(false);

  async function changePassword() {
    const cur = currentPassword;
    const next = newPassword;
    const confirm = confirmPassword;

    if (!cur.trim()) {
      toast.error("当前密码不能为空");
      return;
    }
    if (next.trim().length < 8) {
      toast.error("新密码长度不能小于 8");
      return;
    }
    if (next !== confirm) {
      toast.error("两次输入的新密码不一致");
      return;
    }

    changing = true;
    try {
      await changeAdminPassword({ current_password: cur, new_password: next });
      toast.success("密码已更新，请重新登录");
      auth.logout({ reason: "manual" });
      await goto("/login");
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "修改失败");
    } finally {
      changing = false;
    }
  }

  onMount(() => {
    void reload();
  });
</script>

<div class="space-y-6">
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
          <div class="space-y-2">
            <Label for="check_interval_secs">app.check_interval_secs</Label>
            <Input
              id="check_interval_secs"
              type="number"
              bind:value={checkIntervalSecs}
            />
          </div>

          <div class="space-y-2">
            <Label for="example_api_base">integrations.example_api_base</Label>
            <Input id="example_api_base" bind:value={exampleApiBase} />
          </div>

          <div class="space-y-2 sm:col-span-2">
            <Label for="welcome_message">app.welcome_message</Label>
            <Input id="welcome_message" bind:value={welcomeMessage} />
          </div>

          <div class="space-y-2 sm:col-span-2">
            <Label for="example_api_key">integrations.example_api_key（留空不修改）</Label>
            <Input
              id="example_api_key"
              type="password"
              bind:value={exampleApiKey}
              placeholder={settings.integrations.example_api_key_is_set ? "已设置" : "未设置"}
            />
          </div>
        </div>
      {:else}
        <p class="text-muted-foreground text-sm">尚未加载配置。</p>
      {/if}
    </Card.Content>
  </Card.Root>

  <Card.Root>
    <Card.Header>
      <Card.Title>修改管理员密码</Card.Title>
    </Card.Header>
    <Card.Content>
      <form
        class="grid gap-4"
        onsubmit={(e: SubmitEvent) => {
          e.preventDefault();
          void changePassword();
        }}
      >
        <div class="space-y-2">
          <Label for="current_password">当前密码</Label>
          <Input
            id="current_password"
            type="password"
            bind:value={currentPassword}
            disabled={changing}
            autocomplete="current-password"
          />
        </div>

        <div class="space-y-2">
          <Label for="new_password">新密码（至少 8 位）</Label>
          <Input
            id="new_password"
            type="password"
            bind:value={newPassword}
            disabled={changing}
            autocomplete="new-password"
          />
        </div>

        <div class="space-y-2">
          <Label for="confirm_password">确认新密码</Label>
          <Input
            id="confirm_password"
            type="password"
            bind:value={confirmPassword}
            disabled={changing}
            autocomplete="new-password"
          />
        </div>

        <div class="flex justify-end">
          <Button type="submit" disabled={changing}>
            {changing ? "更新中..." : "更新密码"}
          </Button>
        </div>
      </form>
    </Card.Content>
  </Card.Root>
</div>
