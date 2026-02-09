<script lang="ts">
  import "./layout.css";
  import { page } from "$app/state";
  import { Toaster } from "$lib/shadcn/components/ui/sonner/index.js";
  import { setupZodErrorMap } from "$lib/shared/forms/zod-error-map";
  import { composeDocumentTitleFromPageData } from "$lib/shared/utils/page-title";
  import { ModeWatcher } from "mode-watcher";

  setupZodErrorMap();

  let { children } = $props();
  let documentTitle = $derived(composeDocumentTitleFromPageData(page.data));
</script>

<svelte:head>
  <title>{documentTitle}</title>
</svelte:head>

<ModeWatcher disableHeadScriptInjection />
<Toaster position="top-center" />

{@render children()}
