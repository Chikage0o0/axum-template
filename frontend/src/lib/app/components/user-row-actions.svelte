<script lang="ts">
  import EllipsisVerticalIcon from "@lucide/svelte/icons/ellipsis-vertical";
  import PencilIcon from "@lucide/svelte/icons/pencil";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import type { UserResponse } from "$lib/api/generated/client";
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import * as DropdownMenu from "$lib/shadcn/components/ui/dropdown-menu/index.js";
  import * as Tooltip from "$lib/shadcn/components/ui/tooltip/index.js";

  let {
    row,
    currentUserId = null,
    deletingUserId = null,
    onEdit,
    onDelete,
  }: {
    row: UserResponse;
    currentUserId?: string | null;
    deletingUserId?: string | null;
    onEdit: (row: UserResponse) => void;
    onDelete: (row: UserResponse) => void;
  } = $props();

  let isSelf = $derived(currentUserId === row.id);
  let deleting = $derived(deletingUserId === row.id);
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger>
    {#snippet child({ props })}
      <Button {...props} variant="ghost" size="icon" class="size-8" aria-label="操作菜单">
        <EllipsisVerticalIcon class="size-4" />
      </Button>
    {/snippet}
  </DropdownMenu.Trigger>
  <DropdownMenu.Content align="end">
    <DropdownMenu.Item disabled={deleting} onclick={() => onEdit(row)}>
      <PencilIcon class="size-4" />
      编辑
    </DropdownMenu.Item>
    <DropdownMenu.Separator />
    {#if isSelf}
      <Tooltip.Provider>
        <Tooltip.Root>
          <Tooltip.Trigger>
            {#snippet child({ props })}
              <div {...props}>
                <DropdownMenu.Item disabled>
                  <Trash2Icon class="size-4" />
                  删除
                </DropdownMenu.Item>
              </div>
            {/snippet}
          </Tooltip.Trigger>
          <Tooltip.Content>
            <p>当前登录账号不允许自删除</p>
          </Tooltip.Content>
        </Tooltip.Root>
      </Tooltip.Provider>
    {:else}
      <DropdownMenu.Item
        class="text-destructive focus:text-destructive"
        disabled={deleting}
        onclick={() => onDelete(row)}
      >
        <Trash2Icon class="size-4" />
        {deleting ? "删除中..." : "删除"}
      </DropdownMenu.Item>
    {/if}
  </DropdownMenu.Content>
</DropdownMenu.Root>
