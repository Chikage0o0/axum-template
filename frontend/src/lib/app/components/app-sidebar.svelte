<script lang="ts">
  import { resolve } from "$app/paths";
  import CommandIcon from "@lucide/svelte/icons/command";
  import LayoutDashboardIcon from "@lucide/svelte/icons/layout-dashboard";
  import Settings2Icon from "@lucide/svelte/icons/settings-2";
  import UsersIcon from "@lucide/svelte/icons/users";
  import type { ComponentProps } from "svelte";
  import NavMain from "./nav-main.svelte";
  import NavUser from "./nav-user.svelte";
  import * as Sidebar from "$lib/shadcn/components/ui/sidebar/index.js";
  import type { AuthUser } from "$lib/features/auth/model/auth-user";
  import type { AuthRole } from "$lib/features/auth/model/token-role";

  let {
    ref = $bindable(null),
    currentPath,
    currentUser,
    currentRole,
    onLogout,
    ...restProps
  }: ComponentProps<typeof Sidebar.Root> & {
    currentPath: string;
    currentUser: AuthUser | null;
    currentRole: AuthRole;
    onLogout: () => void;
  } = $props();

  const navMain = $derived.by(() => {
    const base = [
      {
        title: "仪表盘",
        url: "/",
        icon: LayoutDashboardIcon,
      },
      {
        title: "设置",
        url: "/settings",
        icon: Settings2Icon,
      },
    ];

    if (currentRole === "admin") {
      base.push({
        title: "用户管理",
        url: "/users",
        icon: UsersIcon,
      });
    }

    return base;
  });

  const sidebarUser = $derived.by(() => {
    const displayName = currentUser?.displayName?.trim() || "当前用户";
    return {
      name: displayName,
      email: currentUser?.email?.trim() || "未获取邮箱",
      avatar: "",
    };
  });
</script>

<Sidebar.Root bind:ref variant="inset" {...restProps}>
  <Sidebar.Header>
    <Sidebar.Menu>
      <Sidebar.MenuItem>
        <Sidebar.MenuButton size="lg" isActive={currentPath === "/"}>
          {#snippet child({ props })}
            <a href={resolve("/")} {...props}>
              <div
                class="bg-sidebar-primary text-sidebar-primary-foreground flex aspect-square size-8 items-center justify-center rounded-lg"
              >
                <CommandIcon class="size-4" />
              </div>
              <div class="grid flex-1 text-start text-sm leading-tight">
                <span class="truncate font-medium">PROJECT_NAME</span>
                <span class="truncate text-xs">管理控制台</span>
              </div>
            </a>
          {/snippet}
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>
    </Sidebar.Menu>
  </Sidebar.Header>

  <Sidebar.Content>
    <NavMain items={navMain} {currentPath} />
  </Sidebar.Content>

  <Sidebar.Footer>
    <NavUser user={sidebarUser} {onLogout} />
  </Sidebar.Footer>
</Sidebar.Root>
