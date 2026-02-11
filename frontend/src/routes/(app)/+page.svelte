<script lang="ts">
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import * as Card from "$lib/shadcn/components/ui/card/index.js";
  import PageHeader from "$lib/shared/components/page-header.svelte";
</script>

<div class="content-flow space-y-6">
  <PageHeader title="仪表盘" />

  <Card.Root>
    <Card.Header>
      <Card.Title>系统配置</Card.Title>
    </Card.Header>
    <Card.Content>
      <div class="flex flex-wrap gap-2">
        <Button href="/settings">进入设置</Button>
      </div>
    </Card.Content>
  </Card.Root>
</div>
