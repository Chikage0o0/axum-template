<script lang="ts">
  import { goto } from "$app/navigation";
  $effect(() => {
    void goto("/settings");
  });
</script>

<p class="text-sm" style="color: var(--muted)">Redirecting...</p>
