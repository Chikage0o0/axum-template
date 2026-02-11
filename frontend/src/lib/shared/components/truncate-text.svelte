<script lang="ts">
  import * as Tooltip from "$lib/shadcn/components/ui/tooltip/index.js";

  let { text, class: className = "" }: { text: string; class?: string } = $props();

  let el: HTMLSpanElement | undefined = $state();
  let truncated = $state(false);

  $effect(() => {
    if (!el) return;
    const check = () => {
      truncated = el!.scrollWidth > el!.clientWidth;
    };
    const ro = new ResizeObserver(check);
    ro.observe(el);
    check();
    return () => ro.disconnect();
  });
</script>

<Tooltip.Root>
  <Tooltip.Trigger>
    {#snippet child({ props })}
      <span bind:this={el} {...props} class="block truncate {className}">{text}</span>
    {/snippet}
  </Tooltip.Trigger>
  {#if truncated}
    <Tooltip.Content>
      <p>{text}</p>
    </Tooltip.Content>
  {/if}
</Tooltip.Root>
