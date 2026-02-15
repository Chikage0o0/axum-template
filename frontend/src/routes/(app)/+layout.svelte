<script lang="ts">
  import { resolve } from "$app/paths";
  import { page } from "$app/state";
  import MonitorIcon from "@lucide/svelte/icons/monitor";
  import MoonIcon from "@lucide/svelte/icons/moon";
  import SunIcon from "@lucide/svelte/icons/sun";
  import AppSidebar from "$lib/app/components/app-sidebar.svelte";
  import { deleteCurrentSessionHandler } from "$lib/api/generated/client";
  import { auth } from "$lib/features/auth/state/auth";
  import { useAuthBootstrap } from "$lib/features/auth/state/use-auth-bootstrap.svelte";
  import * as Breadcrumb from "$lib/shadcn/components/ui/breadcrumb/index.js";
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/shadcn/components/ui/dropdown-menu/index.js";
  import { Separator } from "$lib/shadcn/components/ui/separator/index.js";
  import * as Sidebar from "$lib/shadcn/components/ui/sidebar/index.js";
  import { mode, setMode, userPrefersMode } from "mode-watcher";

  let { children } = $props();

  let pathname = $derived(page.url.pathname);
  const authBootstrap = useAuthBootstrap();

  type ThemeMode = "light" | "dark" | "system";
  let preferredMode = $derived(userPrefersMode.current as ThemeMode);
  let appliedMode = $derived(mode.current ?? "light");
  let themeModeLabel = $derived.by(() => {
    if (preferredMode === "system") {
      return `跟随系统（当前${appliedMode === "dark" ? "深色" : "浅色"}）`;
    }

    return preferredMode === "dark" ? "深色" : "浅色";
  });

  let breadcrumb = $derived(page.data.breadcrumb ?? { section: "管理", page: "仪表盘" });

  async function handleLogout() {
    try {
      await deleteCurrentSessionHandler();
    } catch {
      // 忽略服务端退出失败，始终清理本地状态。
    }
    await authBootstrap.logoutAndRedirect();
  }

  function setThemeMode(nextMode: ThemeMode) {
    setMode(nextMode);
  }
</script>

<Sidebar.Provider>
  <AppSidebar
    currentPath={pathname}
    currentUser={$auth.user}
    currentPermissions={$auth.permissions}
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
        {#if authBootstrap.ensuringSession}
          <section class="route-enter flex min-h-24 items-center">
            <p class="text-muted-foreground text-sm">正在恢复会话...</p>
          </section>
        {:else}
          {#key pathname}
            <section class="route-enter">
              {@render children()}
            </section>
          {/key}
        {/if}
      </main>
    </div>
  </Sidebar.Inset>
</Sidebar.Provider>
