<script lang="ts">
  import EyeIcon from "@lucide/svelte/icons/eye";
  import EyeOffIcon from "@lucide/svelte/icons/eye-off";
  import type { ComponentProps } from "svelte";
  import * as InputGroup from "$lib/shadcn/components/ui/input-group/index.js";

  type Props = Omit<ComponentProps<typeof InputGroup.Input>, "type" | "files">;

  let {
    ref = $bindable(null),
    value = $bindable(),
    class: className,
    disabled,
    ...restProps
  }: Props = $props();

  let visible = $state(false);
</script>

<InputGroup.Root class={className} data-disabled={disabled ? "true" : undefined}>
  <InputGroup.Input
    bind:ref
    bind:value
    type={visible ? "text" : "password"}
    {disabled}
    {...restProps}
  />
  <InputGroup.Addon align="inline-end">
    <InputGroup.Button
      type="button"
      size="icon-xs"
      aria-label={visible ? "隐藏密码" : "显示密码"}
      aria-pressed={visible}
      {disabled}
      onclick={() => {
        visible = !visible;
      }}
    >
      {#if visible}
        <EyeOffIcon aria-hidden="true" />
      {:else}
        <EyeIcon aria-hidden="true" />
      {/if}
    </InputGroup.Button>
  </InputGroup.Addon>
</InputGroup.Root>
