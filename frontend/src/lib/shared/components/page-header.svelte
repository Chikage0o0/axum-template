<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    actions,
  }: {
    title: string;
    actions?: Snippet;
  } = $props();
</script>

<div class="flex flex-wrap items-center justify-between gap-3">
  <h1 class="text-2xl font-semibold tracking-tight">{title}</h1>
  {#if actions}
    {@render actions()}
  {/if}
</div>
