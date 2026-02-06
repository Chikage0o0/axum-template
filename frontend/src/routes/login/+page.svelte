<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth } from "$lib/stores/auth";
  import { createSession } from "$lib/utils/settings";

  let password = $state("");
  let submitting = $state(false);
  let error = $state<string | null>(null);

  async function submit() {
    error = null;
    const p = password.trim();
    if (!p) {
      error = "请输入管理员密码";
      return;
    }

    submitting = true;
    try {
      const res = await createSession({ password: p });
      auth.login(res.token);
      await goto("/settings");
    } catch (e) {
      error = e instanceof Error ? e.message : "登录失败";
    } finally {
      submitting = false;
    }
  }
</script>

<div class="mx-auto max-w-md space-y-4">
  <h1 class="text-2xl font-semibold tracking-tight">Login</h1>
  <p class="text-sm" style="color: var(--muted)">模板只提供最小登录能力：换取 Bearer Token。</p>

<form
    class="rounded-xl border p-4 space-y-3"
    style="border-color: var(--border); background: var(--card)"
    onsubmit={(e: SubmitEvent) => {
      e.preventDefault();
      void submit();
    }}
  >
    <label class="block text-sm">
      Password
      <input
        class="mt-1 w-full rounded-md border px-3 py-2"
        style="border-color: var(--border); background: rgba(255, 255, 255, 0.04)"
        type="password"
        bind:value={password}
        autocomplete="current-password"
        disabled={submitting}
      />
    </label>

    {#if error}
      <div class="text-sm" style="color: #fecaca">{error}</div>
    {/if}

    <button
      class="w-full rounded-md px-3 py-2 text-sm font-medium"
      style="background: var(--accent); color: #05202b"
      type="submit"
      disabled={submitting}
    >
      {submitting ? "Signing in..." : "Sign in"}
    </button>
  </form>
</div>
