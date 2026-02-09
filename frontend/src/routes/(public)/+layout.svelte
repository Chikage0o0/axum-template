<script lang="ts">
  import { page } from "$app/state";
  import { toast } from "svelte-sonner";
  import { auth } from "$lib/features/auth/state/auth";
  import AuthShell from "$lib/shared/components/auth-shell.svelte";

  let { children } = $props();
  let pathname = $derived(page.url.pathname);

  $effect(() => {
    const flash = $auth.flash;
    if (!flash) return;
    toast.warning(flash.title, { description: flash.message });
    auth.clearFlash();
  });
</script>

{#key pathname}
  <AuthShell>
    {@render children()}
  </AuthShell>
{/key}
