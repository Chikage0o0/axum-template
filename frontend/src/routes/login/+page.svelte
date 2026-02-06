<script lang="ts">
  import { goto } from "$app/navigation";
  import { toast } from "svelte-sonner";
  import { auth } from "$lib/stores/auth";
  import { createSession } from "$lib/utils/settings";
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import * as Card from "$lib/shadcn/components/ui/card/index.js";
  import { Input } from "$lib/shadcn/components/ui/input/index.js";
  import { Label } from "$lib/shadcn/components/ui/label/index.js";

  let password = $state("");
  let submitting = $state(false);

  async function submit() {
    const p = password.trim();
    if (!p) {
      toast.error("请输入管理员密码");
      return;
    }

    submitting = true;
    try {
      const res = await createSession({ password: p });
      auth.login(res.token);
      await goto("/settings");
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "登录失败");
    } finally {
      submitting = false;
    }
  }
</script>

<Card.Root>
  <Card.Header>
    <Card.Title>登录</Card.Title>
  </Card.Header>

  <Card.Content>
    <form
      class="space-y-4"
      onsubmit={(e: SubmitEvent) => {
        e.preventDefault();
        void submit();
      }}
    >
      <div class="space-y-2">
        <Label for="password">管理员密码</Label>
        <Input
          id="password"
          type="password"
          bind:value={password}
          autocomplete="current-password"
          disabled={submitting}
        />
      </div>

      <Button class="w-full" type="submit" disabled={submitting}>
        {submitting ? "登录中..." : "登录"}
      </Button>
    </form>
  </Card.Content>
</Card.Root>
