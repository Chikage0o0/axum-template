<script lang="ts">
  import { AvatarBeam } from "svelte-boring-avatars";
  import * as Avatar from "$lib/shadcn/components/ui/avatar/index.js";
  import { cn } from "$lib/shadcn/utils";
  import { avatarBeamColors, buildAvatarSeed } from "$lib/shared/utils/avatar";

  let {
    src = "",
    alt = "用户头像",
    email = "",
    displayName = "",
    id = "",
    class: className = "",
  }: {
    src?: string;
    alt?: string;
    email?: string;
    displayName?: string;
    id?: string;
    class?: string;
  } = $props();

  const seed = $derived.by(() => buildAvatarSeed({ email, displayName, id }));
  const imageSrc = $derived(src.trim());
</script>

<Avatar.Root class={cn("size-8 shrink-0 rounded-full", className)}>
  {#if imageSrc}
    <Avatar.Image src={imageSrc} {alt} />
  {/if}
  <Avatar.Fallback class="rounded-full p-0">
    {#key seed}
      <AvatarBeam size={32} name={seed} square={true} colors={avatarBeamColors} />
    {/key}
  </Avatar.Fallback>
</Avatar.Root>
