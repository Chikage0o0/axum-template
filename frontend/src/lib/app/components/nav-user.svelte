<script lang="ts">
  import ChevronsUpDownIcon from "@lucide/svelte/icons/chevrons-up-down";
  import KeyRoundIcon from "@lucide/svelte/icons/key-round";
  import LogOutIcon from "@lucide/svelte/icons/log-out";
  import UserPenIcon from "@lucide/svelte/icons/user-pen";
  import ProfileEditDialog from "./profile-edit-dialog.svelte";
  import PasswordChangeDialog from "./password-change-dialog.svelte";
  import UserAvatar from "$lib/shared/components/user-avatar.svelte";
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
    onLogout: () => void | Promise<void>;
  } = $props();

  const sidebar = useSidebar();
  let profileDialogOpen = $state(false);
  let passwordDialogOpen = $state(false);

  let profileDialog: ProfileEditDialog;
  let passwordDialog: PasswordChangeDialog;
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
            <UserAvatar
              src={user.avatar}
              alt={user.name}
              email={user.email}
              displayName={user.name}
            />
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
            <UserAvatar
              src={user.avatar}
              alt={user.name}
              email={user.email}
              displayName={user.name}
            />
            <div class="grid flex-1 text-start text-sm leading-tight">
              <span class="truncate font-medium">{user.name}</span>
              <span class="truncate text-xs">{user.email}</span>
            </div>
          </div>
        </DropdownMenu.Label>

        <DropdownMenu.Separator />

        <DropdownMenu.Item onclick={() => profileDialog.openDialog()}>
          <UserPenIcon />
          编辑信息
        </DropdownMenu.Item>
        <DropdownMenu.Item onclick={() => passwordDialog.openDialog()}>
          <KeyRoundIcon />
          修改密码
        </DropdownMenu.Item>

        <DropdownMenu.Separator />
        <DropdownMenu.Item variant="destructive" onclick={onLogout}>
          <LogOutIcon />
          退出登录
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  </Sidebar.MenuItem>
</Sidebar.Menu>

<ProfileEditDialog bind:this={profileDialog} bind:open={profileDialogOpen} />
<PasswordChangeDialog
  bind:this={passwordDialog}
  bind:open={passwordDialogOpen}
  onSuccess={onLogout}
/>
