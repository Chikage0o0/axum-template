<script lang="ts">
  let { children } = $props();
</script>

<main class="aurora-surface flex min-h-dvh w-full items-center justify-center px-4">
  <div class="route-enter route-enter--auth w-full max-w-md space-y-4">
    {@render children()}
  </div>
</main>
