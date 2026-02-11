<script lang="ts">
  import * as Alert from "$lib/shadcn/components/ui/alert/index.js";
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import * as Empty from "$lib/shadcn/components/ui/empty/index.js";

  let {
    loading = false,
    loadingContent,
    error = null,
    isEmpty = false,
    onRetry,
    onCreate,
    emptyTitle = "暂无数据",
    emptyDescription = "当前没有可展示的内容。",
    createLabel = "新增",
    children,
  }: {
    loading?: boolean;
    loadingContent?: import("svelte").Snippet;
    error?: string | null;
    isEmpty?: boolean;
    onRetry?: () => void | Promise<void>;
    onCreate?: () => void;
    emptyTitle?: string;
    emptyDescription?: string;
    createLabel?: string;
    children?: import("svelte").Snippet;
  } = $props();
</script>

{#if loading}
  {#if loadingContent}
    {@render loadingContent()}
  {:else}
    <div class="py-2 text-sm text-muted-foreground">加载中...</div>
  {/if}
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
