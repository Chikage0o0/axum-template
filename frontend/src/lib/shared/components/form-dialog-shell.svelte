<script lang="ts">
  import { Button } from "$lib/shadcn/components/ui/button/index.js";

  let {
    submitting = false,
    submitLabel,
    submittingLabel = "提交中...",
    cancelLabel = "取消",
    cancelDisabled = false,
    onCancel,
  }: {
    submitting?: boolean;
    submitLabel: string;
    submittingLabel?: string;
    cancelLabel?: string;
    cancelDisabled?: boolean;
    onCancel?: () => void;
  } = $props();
</script>

<div class="md:col-span-2 flex justify-end gap-2 pt-1">
  <Button
    type="button"
    variant="outline"
    disabled={submitting || cancelDisabled}
    onclick={onCancel}
  >
    {cancelLabel}
  </Button>
  <Button type="submit" disabled={submitting}>
    {submitting ? submittingLabel : submitLabel}
  </Button>
</div>
