<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { auth } from "$lib/stores/auth";
  import { changeAdminPassword, getSettings, patchSettings, type PatchSettingsRequest, type SettingsResponse } from "$lib/utils/settings";

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
      error = "配置尚未加载";
      return;
    }

    error = null;
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
      error = e instanceof Error ? e.message : "保存失败";
    } finally {
      saving = false;
    }
  }

  let currentPassword = $state("");
  let newPassword = $state("");
  let confirmPassword = $state("");
  let changing = $state(false);
  let pwError = $state<string | null>(null);

  async function changePassword() {
    pwError = null;
    const cur = currentPassword;
    const next = newPassword;
    const confirm = confirmPassword;

    if (!cur.trim()) {
      pwError = "当前密码不能为空";
      return;
    }
    if (next.trim().length < 8) {
      pwError = "新密码长度不能小于 8";
      return;
    }
    if (next !== confirm) {
      pwError = "两次输入的新密码不一致";
      return;
    }

    changing = true;
    try {
      await changeAdminPassword({ current_password: cur, new_password: next });
      auth.logout({ reason: "manual" });
      await goto("/login");
    } catch (e) {
      pwError = e instanceof Error ? e.message : "修改失败";
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
    <h1 class="text-2xl font-semibold tracking-tight">Settings</h1>
    <p class="mt-1 text-sm" style="color: var(--muted)">运行期配置来自 DB：system_config，保存后热更新。</p>
  </div>

  {#if error}
    <div class="rounded-lg border p-3 text-sm" style="border-color: var(--border); background: rgba(239, 68, 68, 0.10)">
      {error}
    </div>
  {/if}

  <section class="rounded-xl border p-4 space-y-3" style="border-color: var(--border); background: var(--card)">
    <div class="flex items-center justify-between gap-3">
      <h2 class="font-medium">运行期配置</h2>
      <div class="flex items-center gap-2">
        <button class="rounded-md border px-3 py-1 text-sm" style="border-color: var(--border)" disabled={loading || saving} onclick={reload}>
          {loading ? "Loading..." : "Reload"}
        </button>
        <button class="rounded-md px-3 py-1 text-sm font-medium" style="background: var(--accent); color: #05202b" disabled={loading || saving} onclick={save}>
          {saving ? "Saving..." : "Save"}
        </button>
      </div>
    </div>

    {#if settings}
      <div class="grid gap-3 sm:grid-cols-2">
        <label class="block text-sm">
          app.check_interval_secs
          <input class="mt-1 w-full rounded-md border px-3 py-2" style="border-color: var(--border); background: rgba(255, 255, 255, 0.04)" type="number" bind:value={checkIntervalSecs} />
        </label>

        <label class="block text-sm">
          integrations.example_api_base
          <input class="mt-1 w-full rounded-md border px-3 py-2" style="border-color: var(--border); background: rgba(255, 255, 255, 0.04)" bind:value={exampleApiBase} />
        </label>

        <label class="block text-sm sm:col-span-2">
          app.welcome_message
          <input class="mt-1 w-full rounded-md border px-3 py-2" style="border-color: var(--border); background: rgba(255, 255, 255, 0.04)" bind:value={welcomeMessage} />
        </label>

        <label class="block text-sm sm:col-span-2">
          integrations.example_api_key（覆盖设置，留空不修改）
          <input class="mt-1 w-full rounded-md border px-3 py-2" style="border-color: var(--border); background: rgba(255, 255, 255, 0.04)" type="password" bind:value={exampleApiKey} placeholder={settings.integrations.example_api_key_is_set ? "已设置" : "未设置"} />
        </label>
      </div>
    {:else}
      <p class="text-sm" style="color: var(--muted)">尚未加载</p>
    {/if}
  </section>

  <section class="rounded-xl border p-4 space-y-3" style="border-color: var(--border); background: var(--card)">
    <h2 class="font-medium">修改管理员密码</h2>
    <form
      class="grid gap-3"
      onsubmit={(e: SubmitEvent) => {
        e.preventDefault();
        void changePassword();
      }}
    >
      <label class="block text-sm">
        当前密码
        <input class="mt-1 w-full rounded-md border px-3 py-2" style="border-color: var(--border); background: rgba(255, 255, 255, 0.04)" type="password" bind:value={currentPassword} disabled={changing} />
      </label>
      <label class="block text-sm">
        新密码（>= 8）
        <input class="mt-1 w-full rounded-md border px-3 py-2" style="border-color: var(--border); background: rgba(255, 255, 255, 0.04)" type="password" bind:value={newPassword} disabled={changing} />
      </label>
      <label class="block text-sm">
        确认新密码
        <input class="mt-1 w-full rounded-md border px-3 py-2" style="border-color: var(--border); background: rgba(255, 255, 255, 0.04)" type="password" bind:value={confirmPassword} disabled={changing} />
      </label>

      {#if pwError}
        <div class="text-sm" style="color: #fecaca">{pwError}</div>
      {/if}

      <button class="rounded-md px-3 py-2 text-sm font-medium" style="background: var(--accent); color: #05202b" disabled={changing} type="submit">
        {changing ? "Submitting..." : "Update password"}
      </button>
    </form>
  </section>
</div>
