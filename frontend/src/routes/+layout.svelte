<script lang="ts">
  import "./layout.css";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import MoonIcon from "@lucide/svelte/icons/moon";
  import SunIcon from "@lucide/svelte/icons/sun";
  import { auth } from "$lib/stores/auth";
  import { ModeWatcher, toggleMode } from "mode-watcher";

  let { children } = $props();

  const nav = [
    { href: "/settings", label: "Settings" }
  ] as const;

  let isLoginRoute = $derived(page.url.pathname.startsWith("/login"));

  $effect(() => {
    if (isLoginRoute) return;
    if (!$auth.isAuthenticated) {
      void goto("/login");
    }
  });

  $effect(() => {
    if (!isLoginRoute) return;
    if (!$auth.flash) return;
    // 为了保持模板最小化：这里用 alert 作为示例；实际项目建议用 toast。
    alert(`${$auth.flash.title}\n\n${$auth.flash.message}`);
    auth.clearFlash();
  });

  async function handleLogout() {
    auth.logout({ reason: "manual" });
    await goto("/login");
  }
</script>

<ModeWatcher />

<div class="min-h-dvh">
  <header class="sticky top-0 z-10 border-b" style="border-color: var(--border); background: color-mix(in oklch, var(--background) 86%, transparent); backdrop-filter: blur(10px)">
    <div class="mx-auto flex max-w-5xl items-center justify-between gap-3 px-4 py-3">
      <div class="leading-tight">
        <div class="font-semibold tracking-tight">PROJECT_NAME</div>
        <div class="text-xs" style="color: var(--muted-foreground)">Template: conventions over business</div>
      </div>

      {#if !isLoginRoute}
        <nav class="flex items-center gap-3" aria-label="Primary">
          {#each nav as item}
            <a
              class="text-sm hover:underline"
              style="color: {page.url.pathname === item.href ? 'var(--primary)' : 'var(--foreground)'}"
              href={item.href}
              >{item.label}</a
            >
          {/each}
        </nav>
      {/if}

      <div class="flex items-center gap-2">
        <button
          class="relative inline-flex size-9 items-center justify-center rounded-md border"
          style="border-color: var(--border); background: var(--card)"
          onclick={toggleMode}
          type="button"
          aria-label="切换主题"
        >
          <SunIcon class="size-4 rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
          <MoonIcon class="absolute size-4 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
        </button>

        {#if $auth.isAuthenticated}
          <button
            class="rounded-md border px-3 py-1 text-sm"
            style="border-color: var(--border); background: var(--card)"
            onclick={handleLogout}
          >
            Logout
          </button>
        {/if}
      </div>
    </div>
  </header>

  <main class="mx-auto max-w-5xl px-4 py-8">
    {@render children()}
  </main>
</div>
