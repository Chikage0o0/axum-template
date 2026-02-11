<script lang="ts">
  import { resolve } from "$app/paths";
  import type { Component } from "svelte";
  import * as Sidebar from "$lib/shadcn/components/ui/sidebar/index.js";

  let {
    items,
    currentPath,
  }: {
    items: {
      title: string;
      url: string;
      icon: Component;
      isActive?: boolean;
    }[];
    currentPath: string;
  } = $props();

  function itemActive(url: string): boolean {
    if (url === "#") return false;
    if (url === "/") return currentPath === "/";
    return currentPath.startsWith(url);
  }
</script>

<Sidebar.Group>
  <Sidebar.GroupLabel>平台</Sidebar.GroupLabel>
  <Sidebar.Menu>
    {#each items as mainItem (mainItem.title)}
      <Sidebar.MenuItem>
        <Sidebar.MenuButton
          tooltipContent={mainItem.title}
          isActive={mainItem.isActive || itemActive(mainItem.url)}
        >
          {#snippet child({ props })}
            <a href={resolve(mainItem.url as Parameters<typeof resolve>[0])} {...props}>
              <mainItem.icon />
              <span>{mainItem.title}</span>
            </a>
          {/snippet}
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>
    {/each}
  </Sidebar.Menu>
</Sidebar.Group>
