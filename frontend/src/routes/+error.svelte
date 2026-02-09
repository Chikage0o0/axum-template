<script lang="ts">
  import { browser } from "$app/environment";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import CompassIcon from "@lucide/svelte/icons/compass";
  import CornerLeftUpIcon from "@lucide/svelte/icons/corner-left-up";
  import CircleXIcon from "@lucide/svelte/icons/circle-x";
  import HouseIcon from "@lucide/svelte/icons/house";
  import LogInIcon from "@lucide/svelte/icons/log-in";
  import Settings2Icon from "@lucide/svelte/icons/settings-2";
  import TriangleAlertIcon from "@lucide/svelte/icons/triangle-alert";
  import { Badge } from "$lib/shadcn/components/ui/badge/index.js";
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import { Separator } from "$lib/shadcn/components/ui/separator/index.js";
  import * as Empty from "$lib/shadcn/components/ui/empty/index.js";
  import { composeDocumentTitle } from "$lib/shared/utils/page-title";

  let { status } = $props<{ status: number }>();

  let isNotFound = $derived(status === 404);
  let requestedPath = $derived(page.url.pathname);
  let firstSegment = $derived(requestedPath.split("/").filter(Boolean)[0] ?? "");
  const quickLinks = [
    {
      href: "/",
      label: "回到仪表盘",
      description: "返回系统首页，继续常用操作",
      icon: HouseIcon,
      segment: "",
    },
    {
      href: "/settings",
      label: "打开设置",
      description: "查看系统配置或更新运行参数",
      icon: Settings2Icon,
      segment: "settings",
    },
    {
      href: "/login",
      label: "前往登录",
      description: "账号状态变化时可重新登录",
      icon: LogInIcon,
      segment: "login",
    },
  ] as const;
  let recommendedHref = $derived.by(() => {
    if (!firstSegment) return "/";
    return quickLinks.find((item) => item.segment === firstSegment)?.href ?? "/";
  });
  let title = $derived(isNotFound ? "页面不存在" : "页面暂时不可用");
  let documentTitle = $derived(composeDocumentTitle(title));
  let description = $derived.by(() => {
    if (isNotFound) {
      return `请求路径 ${requestedPath} 不存在，请检查地址是否正确。`;
    }

    return "系统遇到异常，请稍后再试。";
  });

  async function goBackOrFallback() {
    if (browser && window.history.length > 1) {
      window.history.back();
      return;
    }

    await goto(recommendedHref);
  }
</script>

<svelte:head>
  <title>{documentTitle}</title>
</svelte:head>

<main class="bg-background min-h-dvh px-4 py-6 sm:px-6 sm:py-10">
  <div class="mx-auto flex min-h-[calc(100dvh-4rem)] w-full max-w-5xl items-center">
    {#if isNotFound}
      <section
        class="relative w-full overflow-hidden rounded-3xl border bg-card/80 p-6 shadow-sm backdrop-blur sm:p-8"
      >
        <div
          class="pointer-events-none absolute -right-16 -top-16 size-44 rounded-full bg-[color-mix(in_oklch,var(--chart-1)_22%,transparent)] blur-3xl"
        ></div>
        <div
          class="pointer-events-none absolute -bottom-20 -left-20 size-56 rounded-full bg-[color-mix(in_oklch,var(--chart-2)_18%,transparent)] blur-3xl"
        ></div>

        <div class="relative grid gap-8 lg:grid-cols-[1.15fr_0.85fr]">
          <div class="space-y-5">
            <div class="flex items-center gap-2">
              <Badge variant="secondary" class="font-mono tracking-[0.2em]">HTTP {status}</Badge>
              <span class="text-muted-foreground text-xs">Not Found</span>
            </div>

            <div class="space-y-3">
              <div class="flex items-center gap-2">
                <CircleXIcon class="text-muted-foreground size-5" />
                <h1 class="text-3xl font-semibold tracking-tight sm:text-4xl">
                  你访问的页面走丢了
                </h1>
              </div>
              <p class="text-muted-foreground max-w-xl text-sm leading-6 sm:text-base">
                地址
                <span class="bg-muted rounded px-1.5 py-0.5 font-mono text-xs sm:text-sm"
                  >{requestedPath}</span
                >
                当前不存在。你可以返回主流程，或从右侧快捷入口继续操作。
              </p>
            </div>

            <div class="flex flex-wrap items-center gap-2">
              <Button href={recommendedHref}>
                <CompassIcon class="size-4" />
                继续浏览
              </Button>
              <Button variant="outline" type="button" onclick={goBackOrFallback}>
                <CornerLeftUpIcon class="size-4" />
                返回上一页
              </Button>
            </div>
          </div>

          <aside class="bg-background/70 rounded-2xl border p-4 sm:p-5">
            <p class="text-muted-foreground mb-3 text-xs tracking-[0.18em]">常用入口</p>
            <div class="space-y-2">
              {#each quickLinks as item}
                <a
                  href={item.href}
                  class="hover:bg-muted/70 focus-visible:ring-ring flex items-start gap-3 rounded-xl border px-3 py-2.5 transition-colors focus-visible:ring-2 focus-visible:outline-none"
                >
                  <item.icon class="mt-0.5 size-4 shrink-0" />
                  <span class="min-w-0 flex-1">
                    <span class="block text-sm font-medium">{item.label}</span>
                    <span class="text-muted-foreground block text-xs leading-5"
                      >{item.description}</span
                    >
                  </span>
                  {#if item.href === recommendedHref}
                    <Badge variant="outline" class="text-[10px]">推荐</Badge>
                  {/if}
                </a>
              {/each}
            </div>
          </aside>
        </div>
      </section>
    {:else}
      <Empty.Root class="w-full rounded-3xl border bg-card/80 p-6 shadow-sm backdrop-blur sm:p-8">
        <Empty.Header>
          <Empty.Media variant="icon" class="text-destructive">
            <TriangleAlertIcon class="size-6" />
          </Empty.Media>
          <p class="text-muted-foreground font-mono text-xs tracking-[0.24em]">HTTP {status}</p>
          <Empty.Title>{title}</Empty.Title>
          <Empty.Description>{description}</Empty.Description>
        </Empty.Header>

        <Separator class="mx-auto my-1 max-w-md" />

        <Empty.Content>
          <div class="flex flex-wrap items-center justify-center gap-2">
            <Button href={recommendedHref}>返回可用页面</Button>
            <Button variant="outline" type="button" onclick={goBackOrFallback}>返回上一页</Button>
          </div>
        </Empty.Content>
      </Empty.Root>
    {/if}
  </div>
</main>
