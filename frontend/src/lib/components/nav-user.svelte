<script lang="ts">
  import ChevronsUpDownIcon from "@lucide/svelte/icons/chevrons-up-down";
  import KeyRoundIcon from "@lucide/svelte/icons/key-round";
  import LogOutIcon from "@lucide/svelte/icons/log-out";
  import UserPenIcon from "@lucide/svelte/icons/user-pen";
  import { AvatarBeam } from "svelte-boring-avatars";
  import { toast } from "svelte-sonner";
  import { ApiError } from "$lib/api/mutator";
  import {
    getCurrentUserHandler,
    patchCurrentUserPasswordHandler,
    patchUserHandler,
    type UserResponse,
  } from "$lib/api/generated/client";
  import {
    CreateUserRequest as CreateUserRequestSchema,
    PatchUserRequest as PatchUserRequestSchema,
  } from "$lib/api/generated/schemas";
  import {
    detailsToFieldErrors,
    hasFieldError,
    mergeFieldErrors,
    toFieldErrorItems,
    type FieldErrors,
    zodErrorToFieldErrors,
  } from "$lib/forms/field-errors";
  import { validatePasswordChangeForm } from "$lib/forms/password-change";
  import { auth } from "$lib/stores/auth";
  import {
    buildCurrentUserPatchPayload,
    toAuthUser,
    type CurrentUserDraft,
  } from "$lib/utils/user-helpers";
  import PasswordInput from "./password-input.svelte";
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import * as Dialog from "$lib/shadcn/components/ui/dialog/index.js";
  import * as Field from "$lib/shadcn/components/ui/field/index.js";
  import { Input } from "$lib/shadcn/components/ui/input/index.js";
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
    onLogout: () => void | Promise<void>;
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

  let profileDialogOpen = $state(false);
  let passwordDialogOpen = $state(false);

  let currentUser = $state<UserResponse | null>(null);
  let loadingCurrentUser = $state(false);
  let savingCurrentUser = $state(false);
  let profileFieldErrors = $state<FieldErrors>({});
  let currentUserDraft = $state<CurrentUserDraft>({
    display_name: "",
    email: "",
    phone: "",
    avatar_url: "",
  });

  let currentPassword = $state("");
  let newPassword = $state("");
  let confirmPassword = $state("");
  let changingPassword = $state(false);
  let passwordFieldErrors = $state<FieldErrors>({});

  function syncCurrentUserDraft(nextUser: UserResponse) {
    currentUserDraft = {
      display_name: nextUser.display_name,
      email: nextUser.email,
      phone: nextUser.phone ?? "",
      avatar_url: nextUser.avatar_url ?? "",
    };
  }

  function invalidProfile(...keys: string[]): boolean {
    return hasFieldError(profileFieldErrors, ...keys);
  }

  function profileErrorItems(...keys: string[]) {
    return toFieldErrorItems(profileFieldErrors, ...keys);
  }

  function invalidPassword(...keys: string[]): boolean {
    return hasFieldError(passwordFieldErrors, ...keys);
  }

  function passwordErrorItems(...keys: string[]) {
    return toFieldErrorItems(passwordFieldErrors, ...keys);
  }

  function resetPasswordForm() {
    currentPassword = "";
    newPassword = "";
    confirmPassword = "";
    passwordFieldErrors = {};
  }

  async function loadCurrentUser() {
    loadingCurrentUser = true;
    profileFieldErrors = {};
    try {
      const me = await getCurrentUserHandler();
      currentUser = me;
      syncCurrentUserDraft(me);
    } catch (e) {
      currentUser = null;
      toast.error(e instanceof Error ? e.message : "加载当前用户信息失败");
    } finally {
      loadingCurrentUser = false;
    }
  }

  async function openProfileDialog() {
    profileDialogOpen = true;
    await loadCurrentUser();
  }

  function openPasswordDialog() {
    resetPasswordForm();
    passwordDialogOpen = true;
  }

  async function submitCurrentUserProfile() {
    if (!currentUser) {
      toast.error("当前用户不存在，无法保存");
      return;
    }

    const draftCheck = CreateUserRequestSchema.pick({
      display_name: true,
      email: true,
      phone: true,
      avatar_url: true,
    }).safeParse({
      display_name: currentUserDraft.display_name.trim(),
      email: currentUserDraft.email.trim(),
      phone: currentUserDraft.phone.trim() || undefined,
      avatar_url: currentUserDraft.avatar_url.trim() || undefined,
    });
    profileFieldErrors = draftCheck.success ? {} : zodErrorToFieldErrors(draftCheck.error);
    if (Object.keys(profileFieldErrors).length > 0) {
      return;
    }

    const result = buildCurrentUserPatchPayload(currentUser, currentUserDraft);
    if (!result.ok) {
      if (result.message.includes("display_name")) {
        profileFieldErrors = { display_name: [result.message] };
      } else if (result.message.includes("email")) {
        profileFieldErrors = { email: [result.message] };
      } else {
        toast.warning(result.message);
      }
      return;
    }

    const payloadCheck = PatchUserRequestSchema.safeParse(result.payload);
    if (!payloadCheck.success) {
      profileFieldErrors = zodErrorToFieldErrors(payloadCheck.error);
      return;
    }

    savingCurrentUser = true;
    try {
      const updated = await patchUserHandler(currentUser.id, result.payload);
      currentUser = updated;
      syncCurrentUserDraft(updated);
      profileFieldErrors = {};

      const mapped = toAuthUser(updated);
      if (mapped) {
        auth.syncUser(mapped);
      }

      toast.success("个人信息已更新");
      profileDialogOpen = false;
    } catch (e) {
      if (e instanceof ApiError) {
        const mapped = detailsToFieldErrors(e.body?.details);
        profileFieldErrors = mergeFieldErrors(profileFieldErrors, mapped);
        if (Object.keys(mapped).length > 0) {
          return;
        }
      }
      toast.error(e instanceof Error ? e.message : "更新个人信息失败");
    } finally {
      savingCurrentUser = false;
    }
  }

  async function submitPasswordChange() {
    const { payload, errors } = validatePasswordChangeForm({
      currentPassword,
      newPassword,
      confirmPassword,
    });
    passwordFieldErrors = errors;
    if (!payload) {
      return;
    }

    changingPassword = true;
    try {
      await patchCurrentUserPasswordHandler(payload);
      toast.success("密码已更新，请重新登录");
      passwordDialogOpen = false;
      resetPasswordForm();
      await onLogout();
    } catch (e) {
      if (e instanceof ApiError) {
        const mapped = detailsToFieldErrors(e.body?.details);
        passwordFieldErrors = mergeFieldErrors(passwordFieldErrors, mapped);
        if (Object.keys(mapped).length > 0) {
          return;
        }
      }
      toast.error(e instanceof Error ? e.message : "修改失败");
    } finally {
      changingPassword = false;
    }
  }
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

        <DropdownMenu.Item onclick={openProfileDialog}>
          <UserPenIcon />
          编辑信息
        </DropdownMenu.Item>
        <DropdownMenu.Item onclick={openPasswordDialog}>
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

<Dialog.Root bind:open={profileDialogOpen}>
  <Dialog.Content class="sm:max-w-xl">
    <Dialog.Header>
      <Dialog.Title>当前用户信息</Dialog.Title>
    </Dialog.Header>

    <form
      class="grid gap-3 md:grid-cols-2"
      onsubmit={(event: SubmitEvent) => {
        event.preventDefault();
        void submitCurrentUserProfile();
      }}
    >
      <Field.Field data-invalid={invalidProfile("display_name") || undefined}>
        <Field.Label for="sidebar_display_name">display_name *</Field.Label>
        <Input
          id="sidebar_display_name"
          bind:value={currentUserDraft.display_name}
          disabled={savingCurrentUser || loadingCurrentUser || !currentUser}
          aria-invalid={invalidProfile("display_name")}
        />
        <Field.Error errors={profileErrorItems("display_name")} />
      </Field.Field>

      <Field.Field data-invalid={invalidProfile("email") || undefined}>
        <Field.Label for="sidebar_email">email *</Field.Label>
        <Input
          id="sidebar_email"
          bind:value={currentUserDraft.email}
          disabled={savingCurrentUser || loadingCurrentUser || !currentUser}
          aria-invalid={invalidProfile("email")}
        />
        <Field.Error errors={profileErrorItems("email")} />
      </Field.Field>

      <Field.Field data-invalid={invalidProfile("phone") || undefined}>
        <Field.Label for="sidebar_phone">phone</Field.Label>
        <Input
          id="sidebar_phone"
          bind:value={currentUserDraft.phone}
          disabled={savingCurrentUser || loadingCurrentUser || !currentUser}
          aria-invalid={invalidProfile("phone")}
        />
        <Field.Error errors={profileErrorItems("phone")} />
      </Field.Field>

      <Field.Field data-invalid={invalidProfile("avatar_url") || undefined}>
        <Field.Label for="sidebar_avatar_url">avatar_url</Field.Label>
        <Input
          id="sidebar_avatar_url"
          bind:value={currentUserDraft.avatar_url}
          disabled={savingCurrentUser || loadingCurrentUser || !currentUser}
          aria-invalid={invalidProfile("avatar_url")}
        />
        <Field.Error errors={profileErrorItems("avatar_url")} />
      </Field.Field>

      <div class="md:col-span-2 flex items-center justify-between gap-2">
        <p class="text-muted-foreground text-xs">
          {loadingCurrentUser ? "正在加载当前用户信息..." : ""}
        </p>
        <div class="flex items-center gap-2">
          <Button
            type="button"
            variant="outline"
            disabled={savingCurrentUser || loadingCurrentUser || !currentUser}
            onclick={() => {
              if (currentUser) {
                syncCurrentUserDraft(currentUser);
                profileFieldErrors = {};
              }
            }}
          >
            重置
          </Button>
          <Button type="submit" disabled={savingCurrentUser || loadingCurrentUser || !currentUser}>
            {savingCurrentUser ? "保存中..." : "保存修改"}
          </Button>
        </div>
      </div>
    </form>
  </Dialog.Content>
</Dialog.Root>

<Dialog.Root bind:open={passwordDialogOpen}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>修改当前用户密码</Dialog.Title>
      <Dialog.Description>修改成功后会退出登录，请使用新密码重新登录。</Dialog.Description>
    </Dialog.Header>

    <form
      class="grid gap-4"
      onsubmit={(event: SubmitEvent) => {
        event.preventDefault();
        void submitPasswordChange();
      }}
    >
      <Field.Field data-invalid={invalidPassword("current_password") || undefined}>
        <Field.Label for="sidebar_current_password">当前用户密码</Field.Label>
        <PasswordInput
          id="sidebar_current_password"
          bind:value={currentPassword}
          disabled={changingPassword}
          autocomplete="current-password"
          aria-invalid={invalidPassword("current_password")}
        />
        <Field.Error errors={passwordErrorItems("current_password")} />
      </Field.Field>

      <Field.Field data-invalid={invalidPassword("new_password") || undefined}>
        <Field.Label for="sidebar_new_password">新密码</Field.Label>
        <Field.Description>至少 8 位</Field.Description>
        <PasswordInput
          id="sidebar_new_password"
          bind:value={newPassword}
          disabled={changingPassword}
          autocomplete="new-password"
          aria-invalid={invalidPassword("new_password")}
        />
        <Field.Error errors={passwordErrorItems("new_password")} />
      </Field.Field>

      <Field.Field data-invalid={invalidPassword("confirm_password") || undefined}>
        <Field.Label for="sidebar_confirm_password">确认新密码</Field.Label>
        <PasswordInput
          id="sidebar_confirm_password"
          bind:value={confirmPassword}
          disabled={changingPassword}
          autocomplete="new-password"
          aria-invalid={invalidPassword("confirm_password")}
        />
        <Field.Error errors={passwordErrorItems("confirm_password")} />
      </Field.Field>

      <div class="flex justify-end">
        <Button type="submit" disabled={changingPassword}>
          {changingPassword ? "更新中..." : "更新密码"}
        </Button>
      </div>
    </form>
  </Dialog.Content>
</Dialog.Root>
