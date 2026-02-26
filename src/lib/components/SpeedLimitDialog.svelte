<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import { Label } from "$lib/components/ui/label";
  import { setDownloadSpeedLimit } from "$lib/commands";
  import { Gauge } from "@lucide/svelte";

  let {
    open = $bindable(false),
    taskId,
    currentLimit,
  }: {
    open: boolean;
    taskId: string;
    currentLimit: number | null | undefined;
  } = $props();

  // Default 0 means Unlimited
  let speedLimitMB = $state<number>(0);
  let loading = $state(false);

  async function handleSave() {
    loading = true;
    try {
      let bps =
        speedLimitMB > 0 ? Math.floor(speedLimitMB * 1024 * 1024) : null;
      await setDownloadSpeedLimit(taskId, bps);
      open = false;
    } catch (e) {
      console.error("Failed to set speed limit:", e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (open) {
      if (currentLimit && currentLimit > 0) {
        speedLimitMB = parseFloat((currentLimit / (1024 * 1024)).toFixed(2));
      } else {
        speedLimitMB = 0;
      }
    }
  });
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title class="flex items-center gap-2">
        <Gauge class="h-5 w-5 text-primary" />
        Task Speed Limit
      </Dialog.Title>
      <Dialog.Description>
        Configure download speed limit for this specific task.
      </Dialog.Description>
    </Dialog.Header>
    <div class="flex flex-col gap-4 py-4">
      <div class="space-y-2">
        <Label for="taskSpeedLimit">Speed Limit (MB/s)</Label>
        <div class="flex items-center gap-2">
          <Input
            id="taskSpeedLimit"
            type="number"
            min="0"
            step="0.1"
            bind:value={speedLimitMB}
          />
          <span class="text-sm text-muted-foreground whitespace-nowrap">
            {speedLimitMB === 0 ? "Unlimited" : "MB/s"}
          </span>
        </div>
        <p class="text-xs text-muted-foreground">
          Set to 0 to use the global limit (if any) or unlimited bandwidth.
        </p>
      </div>
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
      <Button onclick={handleSave} disabled={loading}>Set Limit</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
