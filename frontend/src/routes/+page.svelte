<script lang="ts">
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import * as Card from "$lib/shadcn/components/ui/card/index.js";
</script>

<div class="space-y-6">
  <div>
    <h1 class="text-2xl font-semibold tracking-tight">仪表盘</h1>
  </div>

  <Card.Root>
    <Card.Header>
      <Card.Title>系统配置</Card.Title>
    </Card.Header>
    <Card.Content>
      <Button href="/settings">进入设置</Button>
    </Card.Content>
  </Card.Root>
</div>
