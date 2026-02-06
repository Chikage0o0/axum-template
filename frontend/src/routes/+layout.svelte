<script lang="ts">
  import "./layout.css";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import MoonIcon from "@lucide/svelte/icons/moon";
  import SunIcon from "@lucide/svelte/icons/sun";
  import AppSidebar from "$lib/components/app-sidebar.svelte";
  import * as Alert from "$lib/shadcn/components/ui/alert/index.js";
  import * as Breadcrumb from "$lib/shadcn/components/ui/breadcrumb/index.js";
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import { Separator } from "$lib/shadcn/components/ui/separator/index.js";
  import * as Sidebar from "$lib/shadcn/components/ui/sidebar/index.js";
  import { auth } from "$lib/stores/auth";
  import { ModeWatcher, toggleMode } from "mode-watcher";

  let { children } = $props();

  let isLoginRoute = $derived(page.url.pathname.startsWith("/login"));
  let pathname = $derived(page.url.pathname);

  let breadcrumb = $derived.by(() => {
    if (pathname === "/settings") {
      return { section: "Administration", page: "Settings" };
    }

    return { section: "Administration", page: "Dashboard" };
  });

  $effect(() => {
    if (isLoginRoute) return;
    if (!$auth.isAuthenticated) {
      void goto("/login");
    }
  });

  async function handleLogout() {
    auth.logout({ reason: "manual" });
    await goto("/login");
  }
</script>

<ModeWatcher />

{#if isLoginRoute}
  <main class="aurora-surface flex min-h-dvh w-full items-center justify-center px-4">
    <div class="w-full max-w-md space-y-4">
      {#if $auth.flash}
        <Alert.Root variant="destructive">
          <Alert.Title>{$auth.flash.title}</Alert.Title>
          <Alert.Description>{$auth.flash.message}</Alert.Description>
        </Alert.Root>
      {/if}

      {@render children()}
    </div>
  </main>
{:else}
  <Sidebar.Provider>
    <AppSidebar currentPath={pathname} onLogout={handleLogout} />
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
            <Button variant="outline" size="icon" onclick={toggleMode} aria-label="Toggle theme">
              <SunIcon class="size-4 rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
              <MoonIcon class="absolute size-4 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
            </Button>
          </div>
        </div>
      </header>

      <div class="flex flex-1 flex-col p-4">
        <main class="flex-1">
          {@render children()}
        </main>
      </div>
    </Sidebar.Inset>
  </Sidebar.Provider>
{/if}
