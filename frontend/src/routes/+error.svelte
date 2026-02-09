<script lang="ts">
  import { page } from "$app/state";
  import CircleXIcon from "@lucide/svelte/icons/circle-x";
  import TriangleAlertIcon from "@lucide/svelte/icons/triangle-alert";
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import * as Empty from "$lib/shadcn/components/ui/empty/index.js";
  import { composeDocumentTitle } from "$lib/utils/page-title";

  let { status } = $props<{ status: number }>();

  let isNotFound = $derived(status === 404);
  let requestedPath = $derived(page.url.pathname);
  let title = $derived(isNotFound ? "页面不存在" : "页面暂时不可用");
  let documentTitle = $derived(composeDocumentTitle(title));
  let description = $derived.by(() => {
    if (isNotFound) {
      return `请求路径 ${requestedPath} 不存在，请检查地址是否正确。`;
    }

    return "系统遇到异常，请稍后再试。";
  });
</script>

<svelte:head>
  <title>{documentTitle}</title>
</svelte:head>

<div class="content-flow">
  <Empty.Root class="min-h-[calc(100dvh-12rem)] border bg-card/70 backdrop-blur-sm">
    <Empty.Header>
      <Empty.Media variant="icon" class={isNotFound ? "text-muted-foreground" : "text-destructive"}>
        {#if isNotFound}
          <CircleXIcon class="size-6" />
        {:else}
          <TriangleAlertIcon class="size-6" />
        {/if}
      </Empty.Media>
      <p class="text-muted-foreground font-mono text-xs tracking-[0.24em]">HTTP {status}</p>
      <Empty.Title>{title}</Empty.Title>
      <Empty.Description>{description}</Empty.Description>
    </Empty.Header>

    <Empty.Content>
      <div class="flex flex-wrap items-center justify-center gap-2">
        <Button href="/">返回首页</Button>
        <Button variant="outline" type="button" onclick={() => history.back()}>返回上一页</Button>
      </div>
    </Empty.Content>
  </Empty.Root>
</div>
