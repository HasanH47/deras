<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import { Label } from "$lib/components/ui/label";
  import { Switch } from "$lib/components/ui/switch";
  import { Separator } from "$lib/components/ui/separator";
  import { setGlobalSpeedLimit, setScheduleConfig } from "$lib/commands";
  import { Settings, Clock } from "@lucide/svelte";

  let {
    open = $bindable(false),
  }: {
    open: boolean;
  } = $props();

  // Default 0 means Unlimited
  let speedLimitMB = $state<number>(0);

  // Scheduler state
  let scheduleEnabled = $state<boolean>(false);
  let scheduleStart = $state<string>("00:00");
  let scheduleEnd = $state<string>("06:00");

  let loading = $state(false);

  async function handleSave() {
    loading = true;
    try {
      // Convert MB/s to Bytes/s
      let bps = speedLimitMB > 0 ? Math.floor(speedLimitMB * 1024 * 1024) : 0;
      await setGlobalSpeedLimit(bps);
      await setScheduleConfig(scheduleEnabled, scheduleStart, scheduleEnd);

      // Save to localStorage to persist across App restarts
      localStorage.setItem("deras_global_speed_limit", bps.toString());
      localStorage.setItem(
        "deras_schedule_enabled",
        scheduleEnabled.toString(),
      );
      localStorage.setItem("deras_schedule_start", scheduleStart);
      localStorage.setItem("deras_schedule_end", scheduleEnd);

      open = false;
    } catch (e) {
      console.error("Failed to set speed limit:", e);
    } finally {
      loading = false;
    }
  }

  // Load from localStorage on open
  $effect(() => {
    if (open) {
      const savedLimit = localStorage.getItem("deras_global_speed_limit");
      if (savedLimit) {
        speedLimitMB = parseInt(savedLimit, 10) / (1024 * 1024);
      }

      const savedSchEnabled = localStorage.getItem("deras_schedule_enabled");
      if (savedSchEnabled !== null) {
        scheduleEnabled = savedSchEnabled === "true";
      }
      const savedSchStart = localStorage.getItem("deras_schedule_start");
      if (savedSchStart) scheduleStart = savedSchStart;

      const savedSchEnd = localStorage.getItem("deras_schedule_end");
      if (savedSchEnd) scheduleEnd = savedSchEnd;
    }
  });
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title class="flex items-center gap-2">
        <Settings class="h-5 w-5 text-primary" />
        Settings
      </Dialog.Title>
      <Dialog.Description>
        Configure global application settings.
      </Dialog.Description>
    </Dialog.Header>
    <div class="flex flex-col gap-4 py-4">
      <div class="space-y-2">
        <Label for="speedLimit">Global Speed Limit (MB/s)</Label>
        <div class="flex items-center gap-2">
          <Input
            id="speedLimit"
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
          Set to 0 for unlimited network bandwidth usage.
        </p>
      </div>

      <Separator />

      <div class="space-y-4">
        <div class="flex items-center justify-between mt-2">
          <div class="space-y-0.5">
            <Label class="flex items-center gap-2">
              <Clock class="h-4 w-4 text-primary" />
              Download Scheduler
            </Label>
            <p class="text-xs text-muted-foreground">
              Only allow downloads to be active during a specific time window.
            </p>
          </div>
          <Switch bind:checked={scheduleEnabled} />
        </div>

        {#if scheduleEnabled}
          <div
            class="grid grid-cols-2 gap-4 animate-in fade-in slide-in-from-top-2"
          >
            <div class="space-y-2">
              <Label for="scheduleStart">Start Time</Label>
              <Input
                id="scheduleStart"
                type="time"
                bind:value={scheduleStart}
              />
            </div>
            <div class="space-y-2">
              <Label for="scheduleEnd">End Time</Label>
              <Input id="scheduleEnd" type="time" bind:value={scheduleEnd} />
            </div>
          </div>
          <p class="text-xs text-muted-foreground">
            Downloads will automatically pause outside of this window.
          </p>
        {/if}
      </div>
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
      <Button onclick={handleSave} disabled={loading}>Save Settings</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
