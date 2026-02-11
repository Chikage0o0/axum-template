<script lang="ts">
  import * as Alert from "$lib/shadcn/components/ui/alert/index.js";
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import * as Empty from "$lib/shadcn/components/ui/empty/index.js";
  import { Skeleton } from "$lib/shadcn/components/ui/skeleton/index.js";

  let {
    loading = false,
    error = null,
    isEmpty = false,
    onRetry,
    onCreate,
    loadingRows = 4,
    emptyTitle = "暂无数据",
    emptyDescription = "当前没有可展示的内容。",
    createLabel = "新增",
    children,
  }: {
    loading?: boolean;
    error?: string | null;
    isEmpty?: boolean;
    onRetry?: () => void | Promise<void>;
    onCreate?: () => void;
    loadingRows?: number;
    emptyTitle?: string;
    emptyDescription?: string;
    createLabel?: string;
    children?: import("svelte").Snippet;
  } = $props();
</script>

{#if loading}
  <div class="space-y-4">
    {#each Array.from({ length: loadingRows }, (_, index) => index) as index (index)}
      <div class="flex items-center gap-4">
        <Skeleton class="size-8 rounded-full" />
        <Skeleton class="h-4 w-24" />
        <Skeleton class="h-4 w-20" />
        <Skeleton class="h-4 w-36" />
      </div>
    {/each}
  </div>
{:else if error}
  <div class="space-y-3">
    <Alert.Root variant="destructive">
      <Alert.Title>请求失败</Alert.Title>
      <Alert.Description>{error}</Alert.Description>
    </Alert.Root>
    {#if onRetry}
      <Button variant="outline" onclick={onRetry}>重试</Button>
    {/if}
  </div>
{:else if isEmpty}
  <Empty.Root class="min-h-44">
    <Empty.Header>
      <Empty.Title>{emptyTitle}</Empty.Title>
      <Empty.Description>{emptyDescription}</Empty.Description>
    </Empty.Header>
    {#if onCreate}
      <Empty.Content>
        <Button onclick={onCreate}>{createLabel}</Button>
      </Empty.Content>
    {/if}
  </Empty.Root>
{:else}
  {@render children?.()}
{/if}
