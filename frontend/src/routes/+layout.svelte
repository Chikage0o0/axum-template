<script lang="ts">
  import "./layout.css";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import MonitorIcon from "@lucide/svelte/icons/monitor";
  import MoonIcon from "@lucide/svelte/icons/moon";
  import SunIcon from "@lucide/svelte/icons/sun";
  import { toast } from "svelte-sonner";
  import AppSidebar from "$lib/components/app-sidebar.svelte";
  import * as Breadcrumb from "$lib/shadcn/components/ui/breadcrumb/index.js";
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/shadcn/components/ui/dropdown-menu/index.js";
  import { Separator } from "$lib/shadcn/components/ui/separator/index.js";
  import { Toaster } from "$lib/shadcn/components/ui/sonner/index.js";
  import * as Sidebar from "$lib/shadcn/components/ui/sidebar/index.js";
  import { getCurrentUserHandler } from "$lib/api/generated/client";
  import { setupZodErrorMap } from "$lib/forms/zod-error-map";
  import { auth } from "$lib/stores/auth";
  import { composeDocumentTitleFromPageData } from "$lib/utils/page-title";
  import { toAuthUser } from "$lib/utils/user-helpers";
  import { ModeWatcher, mode, setMode, userPrefersMode } from "mode-watcher";

  setupZodErrorMap();

  let { children } = $props();

  let isLoginRoute = $derived(page.url.pathname.startsWith("/login"));
  let pathname = $derived(page.url.pathname);
  let documentTitle = $derived(composeDocumentTitleFromPageData(page.data));
  let syncedToken = $state<string | null>(null);
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
    if (pathname === "/settings") {
      return { section: "管理", page: "设置" };
    }

    return { section: "管理", page: "仪表盘" };
  });

  $effect(() => {
    if (isLoginRoute) return;
    if (!$auth.isAuthenticated) {
      void goto("/login");
    }
  });

  $effect(() => {
    if (isLoginRoute) {
      syncedToken = null;
      return;
    }

    const token = $auth.token;
    if (!token) {
      syncedToken = null;
      return;
    }
    if (token === syncedToken) return;

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

  $effect(() => {
    if (!isLoginRoute) return;
    const flash = $auth.flash;
    if (!flash) return;
    toast.warning(flash.title, { description: flash.message });
    auth.clearFlash();
  });

  async function handleLogout() {
    auth.logout({ reason: "manual" });
    await goto("/login");
  }

  function setThemeMode(nextMode: ThemeMode) {
    setMode(nextMode);
  }
</script>

<svelte:head>
  <title>{documentTitle}</title>
</svelte:head>

<ModeWatcher disableHeadScriptInjection />
<Toaster position="top-center" />

{#if isLoginRoute}
  <main class="aurora-surface flex min-h-dvh w-full items-center justify-center px-4">
    {#key pathname}
      <div class="route-enter route-enter--auth w-full max-w-md space-y-4">
        {@render children()}
      </div>
    {/key}
  </main>
{:else}
  <Sidebar.Provider>
    <AppSidebar currentPath={pathname} currentUser={$auth.user} onLogout={handleLogout} />
    <Sidebar.Inset class="aurora-surface">
      <header class="flex h-16 shrink-0 items-center gap-2 border-b">
        <div class="flex w-full items-center justify-between gap-2 px-4">
          <div class="flex items-center gap-2">
            <Sidebar.Trigger class="-ms-1" />
            <Separator orientation="vertical" class="me-2 data-[orientation=vertical]:h-4" />
            <Breadcrumb.Root>
              <Breadcrumb.List>
                <Breadcrumb.Item class="hidden md:block">
                  <Breadcrumb.Link href="/">{breadcrumb.section}</Breadcrumb.Link>
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
{/if}
