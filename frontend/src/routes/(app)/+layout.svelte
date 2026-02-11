<script lang="ts">
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { page } from "$app/state";
  import MonitorIcon from "@lucide/svelte/icons/monitor";
  import MoonIcon from "@lucide/svelte/icons/moon";
  import SunIcon from "@lucide/svelte/icons/sun";
  import AppSidebar from "$lib/app/components/app-sidebar.svelte";
  import {
    deleteCurrentSessionHandler,
    getCurrentUserHandler,
    refreshSessionHandler,
  } from "$lib/api/generated/client";
  import { toAuthUser } from "$lib/features/auth/model/user-helpers";
  import { auth } from "$lib/features/auth/state/auth";
  import * as Breadcrumb from "$lib/shadcn/components/ui/breadcrumb/index.js";
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/shadcn/components/ui/dropdown-menu/index.js";
  import { Separator } from "$lib/shadcn/components/ui/separator/index.js";
  import * as Sidebar from "$lib/shadcn/components/ui/sidebar/index.js";
  import { mode, setMode, userPrefersMode } from "mode-watcher";

  let { children } = $props();

  let pathname = $derived(page.url.pathname);
  let syncedToken = $state<string | null>(null);
  let ensuringSession = false;

  type ThemeMode = "light" | "dark" | "system";
  let preferredMode = $derived(userPrefersMode.current as ThemeMode);
  let appliedMode = $derived(mode.current ?? "light");
  let themeModeLabel = $derived.by(() => {
    if (preferredMode === "system") {
      return `跟随系统（当前${appliedMode === "dark" ? "深色" : "浅色"}）`;
    }

    return preferredMode === "dark" ? "深色" : "浅色";
  });

  let breadcrumb = $derived.by(() => {
    if (pathname.startsWith("/users")) {
      return { section: "管理", page: "用户管理" };
    }

    if (pathname === "/settings") {
      return { section: "管理", page: "设置" };
    }

    return { section: "管理", page: "仪表盘" };
  });

  $effect(() => {
    if ($auth.isAuthenticated || ensuringSession) return;

    let cancelled = false;
    ensuringSession = true;

    void (async () => {
      try {
        const refreshed = await refreshSessionHandler();
        if (cancelled) return;
        auth.login(refreshed.token);
      } catch {
        if (cancelled) return;
        auth.logout({ reason: "manual" });
        await goto(resolve("/login"));
      } finally {
        ensuringSession = false;
      }
    })();

    return () => {
      cancelled = true;
    };
  });

  $effect(() => {
    const token = $auth.token;
    const currentUser = $auth.user;
    if (!token) {
      syncedToken = null;
      return;
    }
    if (token === syncedToken && currentUser) return;

    let cancelled = false;
    void (async () => {
      try {
        const currentUser = await getCurrentUserHandler();
        if (cancelled) return;
        const mapped = toAuthUser(currentUser);
        if (mapped) {
          auth.syncUser(mapped);
        }
      } finally {
        if (!cancelled) syncedToken = token;
      }
    })();

    return () => {
      cancelled = true;
    };
  });

  async function handleLogout() {
    try {
      await deleteCurrentSessionHandler();
    } catch {
      // 忽略服务端退出失败，始终清理本地状态。
    }
    auth.logout({ reason: "manual" });
    await goto(resolve("/login"));
  }

  function setThemeMode(nextMode: ThemeMode) {
    setMode(nextMode);
  }
</script>

<Sidebar.Provider>
  <AppSidebar
    currentPath={pathname}
    currentUser={$auth.user}
    currentRole={$auth.role}
    onLogout={handleLogout}
  />
  <Sidebar.Inset class="aurora-surface">
    <header class="flex h-16 shrink-0 items-center gap-2 border-b">
      <div class="flex w-full items-center justify-between gap-2 px-4">
        <div class="flex items-center gap-2">
          <Sidebar.Trigger class="-ms-1" />
          <Separator orientation="vertical" class="me-2 data-[orientation=vertical]:h-4" />
          <Breadcrumb.Root>
            <Breadcrumb.List>
              <Breadcrumb.Item class="hidden md:block">
                <Breadcrumb.Link href={resolve("/")}>{breadcrumb.section}</Breadcrumb.Link>
              </Breadcrumb.Item>
              <Breadcrumb.Separator class="hidden md:block" />
              <Breadcrumb.Item>
                <Breadcrumb.Page>{breadcrumb.page}</Breadcrumb.Page>
              </Breadcrumb.Item>
            </Breadcrumb.List>
          </Breadcrumb.Root>
        </div>

        <div class="flex items-center gap-2">
          <DropdownMenu.Root>
            <DropdownMenu.Trigger>
              {#snippet child({ props })}
                <Button
                  {...props}
                  variant="outline"
                  size="icon"
                  aria-label={`主题模式：${themeModeLabel}`}
                  title={`主题模式：${themeModeLabel}`}
                >
                  {#if preferredMode === "system"}
                    <MonitorIcon class="size-4" />
                  {:else if appliedMode === "dark"}
                    <MoonIcon class="size-4" />
                  {:else}
                    <SunIcon class="size-4" />
                  {/if}
                </Button>
              {/snippet}
            </DropdownMenu.Trigger>

            <DropdownMenu.Content align="end" sideOffset={8}>
              <DropdownMenu.Label>主题模式</DropdownMenu.Label>
              <DropdownMenu.Separator />
              <DropdownMenu.RadioGroup value={preferredMode}>
                <DropdownMenu.RadioItem value="light" onclick={() => setThemeMode("light")}>
                  <SunIcon />
                  浅色
                </DropdownMenu.RadioItem>
                <DropdownMenu.RadioItem value="dark" onclick={() => setThemeMode("dark")}>
                  <MoonIcon />
                  深色
                </DropdownMenu.RadioItem>
                <DropdownMenu.RadioItem value="system" onclick={() => setThemeMode("system")}>
                  <MonitorIcon />
                  跟随系统
                </DropdownMenu.RadioItem>
              </DropdownMenu.RadioGroup>
            </DropdownMenu.Content>
          </DropdownMenu.Root>
        </div>
      </div>
    </header>

    <div class="flex flex-1 flex-col p-4">
      <main class="flex-1">
        {#key pathname}
          <section class="route-enter">
            {@render children()}
          </section>
        {/key}
      </main>
    </div>
  </Sidebar.Inset>
</Sidebar.Provider>
