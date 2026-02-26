<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { getTaskLogs } from "$lib/commands";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, tick } from "svelte";

  let { open = $bindable(false), taskId = "", filename = "" } = $props();

  let logs = $state<any[]>([]);
  let viewport = $state<HTMLElement | null>(null);

  async function loadLogs() {
    if (!taskId) return;
    try {
      logs = await getTaskLogs(taskId);
      await scrollToBottom();
    } catch (e) {
      console.error(e);
    }
  }

  async function scrollToBottom() {
    await tick();
    if (viewport) {
      viewport.scrollTop = viewport.scrollHeight;
    }
  }

  onMount(() => {
    const unlisten = listen("task_log", (event: any) => {
      const [id, entry] = event.payload;
      if (id === taskId) {
        logs.push(entry);
        scrollToBottom();
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  });

  $effect(() => {
    if (open && taskId) {
      loadLogs();
    }
  });
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="sm:max-w-[700px] h-[500px] flex flex-col">
    <Dialog.Header>
      <Dialog.Title>Download Logs</Dialog.Title>
      <Dialog.Description class="truncate">
        Real-time events for: <span class="text-foreground font-mono break-all"
          >{filename}</span
        >
      </Dialog.Description>
    </Dialog.Header>

    <div
      class="flex-1 min-h-0 bg-muted/30 rounded-md border mt-2 overflow-hidden"
    >
      <div
        bind:this={viewport}
        class="h-full overflow-y-auto p-4 space-y-1 font-mono text-[11px] leading-tight"
      >
        {#each logs as log}
          <div class="flex gap-3">
            <span class="text-muted-foreground shrink-0 w-20"
              >{new Date(log.timestamp).toLocaleTimeString()}</span
            >
            <span
              class={log.level === "error"
                ? "text-destructive"
                : log.level === "warn"
                  ? "text-yellow-500"
                  : "text-blue-400"}
            >
              [{log.level.toUpperCase()}]
            </span>
            <span class="text-foreground flex-1 break-all">{log.message}</span>
          </div>
        {:else}
          <div
            class="h-full flex items-center justify-center text-muted-foreground text-sm"
          >
            No logs available for this task.
          </div>
        {/each}
      </div>
    </div>

    <div class="flex justify-end mt-4">
      <Button variant="outline" size="sm" onclick={() => (open = false)}
        >Close</Button
      >
    </div>
  </Dialog.Content>
</Dialog.Root>
