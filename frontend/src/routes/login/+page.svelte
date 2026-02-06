<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth } from "$lib/stores/auth";
  import { createSession } from "$lib/utils/settings";
  import * as Alert from "$lib/shadcn/components/ui/alert/index.js";
  import { Button } from "$lib/shadcn/components/ui/button/index.js";
  import * as Card from "$lib/shadcn/components/ui/card/index.js";
  import { Input } from "$lib/shadcn/components/ui/input/index.js";
  import { Label } from "$lib/shadcn/components/ui/label/index.js";

  let password = $state("");
  let submitting = $state(false);
  let error = $state<string | null>(null);

  async function submit() {
    error = null;
    const p = password.trim();
    if (!p) {
      error = "请输入管理员密码";
      return;
    }

    submitting = true;
    try {
      const res = await createSession({ password: p });
      auth.login(res.token);
      await goto("/settings");
    } catch (e) {
      error = e instanceof Error ? e.message : "登录失败";
    } finally {
      submitting = false;
    }
  }
</script>

<Card.Root>
  <Card.Header>
    <Card.Title>Login</Card.Title>
    <Card.Description>输入管理员密码获取 Bearer Token。</Card.Description>
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
        <Label for="password">Password</Label>
        <Input
          id="password"
          type="password"
          bind:value={password}
          autocomplete="current-password"
          disabled={submitting}
        />
      </div>

      {#if error}
        <Alert.Root variant="destructive">
          <Alert.Title>登录失败</Alert.Title>
          <Alert.Description>{error}</Alert.Description>
        </Alert.Root>
      {/if}

      <Button class="w-full" type="submit" disabled={submitting}>
        {submitting ? "Signing in..." : "Sign in"}
      </Button>
    </form>
  </Card.Content>
</Card.Root>
