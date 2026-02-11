<script lang="ts">
  import RefreshCwIcon from "@lucide/svelte/icons/refresh-cw";
  import UserPlusIcon from "@lucide/svelte/icons/user-plus";
  import { Button } from "$lib/shadcn/components/ui/button/index.js";

  let {
    loading = false,
    onRefresh,
    onCreate,
  }: {
    loading?: boolean;
    onRefresh: () => void | Promise<void>;
    onCreate: () => void;
  } = $props();
</script>

<div class="flex items-center gap-2">
  <Button variant="outline" disabled={loading} onclick={onRefresh}>
    <RefreshCwIcon class={`size-4 ${loading ? "animate-spin" : ""}`} />
    {loading ? "刷新中..." : "刷新"}
  </Button>
  <Button onclick={onCreate}>
    <UserPlusIcon class="size-4" />
    新增用户
  </Button>
</div>
