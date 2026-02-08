<script lang="ts">
  import ChevronsUpDownIcon from "@lucide/svelte/icons/chevrons-up-down";
  import LogOutIcon from "@lucide/svelte/icons/log-out";
  import { AvatarBeam } from "svelte-boring-avatars";
  import * as Avatar from "$lib/shadcn/components/ui/avatar/index.js";
  import * as DropdownMenu from "$lib/shadcn/components/ui/dropdown-menu/index.js";
  import * as Sidebar from "$lib/shadcn/components/ui/sidebar/index.js";
  import { useSidebar } from "$lib/shadcn/components/ui/sidebar/index.js";

  let {
    user,
    onLogout,
  }: {
    user: {
      name: string;
      email: string;
      avatar: string;
    };
    onLogout: () => void;
  } = $props();

  const sidebar = useSidebar();
  const beamColors = [
    "var(--sidebar-primary)",
    "var(--chart-1)",
    "var(--chart-2)",
    "var(--chart-3)",
    "var(--chart-4)",
  ];

  const fallbackSeed = $derived.by(() => {
    const email = user.email.trim();
    if (email) return email;

    const name = user.name.trim();
    if (name) return name;

    return "current-user";
  });
</script>

<Sidebar.Menu>
  <Sidebar.MenuItem>
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        {#snippet child({ props })}
          <Sidebar.MenuButton
            {...props}
            size="lg"
            class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
          >
            <Avatar.Root class="size-8 rounded-lg">
              <Avatar.Image src={user.avatar} alt={user.name} />
              <Avatar.Fallback class="rounded-lg p-0">
                {#key fallbackSeed}
                  <AvatarBeam size={32} name={fallbackSeed} square={true} colors={beamColors} />
                {/key}
              </Avatar.Fallback>
            </Avatar.Root>
            <div class="grid flex-1 text-start text-sm leading-tight">
              <span class="truncate font-medium">{user.name}</span>
              <span class="truncate text-xs">{user.email}</span>
            </div>
            <ChevronsUpDownIcon class="ms-auto size-4" />
          </Sidebar.MenuButton>
        {/snippet}
      </DropdownMenu.Trigger>

      <DropdownMenu.Content
        class="w-(--bits-dropdown-menu-anchor-width) min-w-56 rounded-lg"
        side={sidebar.isMobile ? "bottom" : "right"}
        align="end"
        sideOffset={4}
      >
        <DropdownMenu.Label class="p-0 font-normal">
          <div class="flex items-center gap-2 px-1 py-1.5 text-start text-sm">
            <Avatar.Root class="size-8 rounded-lg">
              <Avatar.Image src={user.avatar} alt={user.name} />
              <Avatar.Fallback class="rounded-lg p-0">
                {#key fallbackSeed}
                  <AvatarBeam size={32} name={fallbackSeed} square={true} colors={beamColors} />
                {/key}
              </Avatar.Fallback>
            </Avatar.Root>
            <div class="grid flex-1 text-start text-sm leading-tight">
              <span class="truncate font-medium">{user.name}</span>
              <span class="truncate text-xs">{user.email}</span>
            </div>
          </div>
        </DropdownMenu.Label>

        <DropdownMenu.Separator />
        <DropdownMenu.Item variant="destructive" onclick={onLogout}>
          <LogOutIcon />
          退出登录
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  </Sidebar.MenuItem>
</Sidebar.Menu>
