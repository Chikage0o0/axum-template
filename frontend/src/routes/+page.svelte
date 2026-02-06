<script lang="ts">
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import * as Card from "$lib/shadcn/components/ui/card/index.js";
</script>

<div class="space-y-6">
  <div>
    <h1 class="text-2xl font-semibold tracking-tight">Dashboard</h1>
    <p class="text-muted-foreground mt-1 text-sm">这是精简后的控制台入口，主要操作集中在设置页。</p>
  </div>

  <Card.Root>
    <Card.Header>
      <Card.Title>系统配置</Card.Title>
      <Card.Description>管理运行期参数与管理员密码。</Card.Description>
    </Card.Header>
    <Card.Content>
      <Button href="/settings">进入 Settings</Button>
    </Card.Content>
  </Card.Root>
</div>
