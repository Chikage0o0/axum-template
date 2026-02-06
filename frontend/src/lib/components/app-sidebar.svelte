<script lang="ts" module>
  import LayoutDashboardIcon from "@lucide/svelte/icons/layout-dashboard";
  import Settings2Icon from "@lucide/svelte/icons/settings-2";

  const data = {
    user: {
      name: "管理员",
      email: "admin@local",
      avatar: "",
    },
    navMain: [
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
    ],
  };
</script>

<script lang="ts">
  import CommandIcon from "@lucide/svelte/icons/command";
  import type { ComponentProps } from "svelte";
  import NavMain from "./nav-main.svelte";
  import NavUser from "./nav-user.svelte";
  import * as Sidebar from "$lib/shadcn/components/ui/sidebar/index.js";

  let {
    ref = $bindable(null),
    currentPath,
    onLogout,
    ...restProps
  }: ComponentProps<typeof Sidebar.Root> & {
    currentPath: string;
    onLogout: () => void;
  } = $props();
</script>

<Sidebar.Root bind:ref variant="inset" {...restProps}>
  <Sidebar.Header>
    <Sidebar.Menu>
      <Sidebar.MenuItem>
        <Sidebar.MenuButton size="lg" isActive={currentPath === "/"}>
          {#snippet child({ props })}
            <a href="/" {...props}>
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
    <NavMain items={data.navMain} {currentPath} />
  </Sidebar.Content>

  <Sidebar.Footer>
    <NavUser user={data.user} {onLogout} />
  </Sidebar.Footer>
</Sidebar.Root>
