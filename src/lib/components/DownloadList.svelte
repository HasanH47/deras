<script lang="ts">
  import type { DownloadTask, DownloadState } from "$lib/types/models";
  import { removeDownload } from "$lib/commands";
  import { Progress } from "$lib/components/ui/progress";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import {
    Trash2,
    FileDown,
    FileCheck2,
    Pause,
    AlertCircle,
    Clock,
  } from "@lucide/svelte";

  let {
    downloads,
    onRemove,
  }: {
    downloads: DownloadTask[];
    onRemove: (id: string) => void;
  } = $props();

  function getStateType(state: DownloadState): string {
    return state.type;
  }

  function getStateBadgeVariant(
    state: DownloadState,
  ): "default" | "secondary" | "destructive" | "outline" {
    switch (state.type) {
      case "Downloading":
        return "default";
      case "Completed":
        return "secondary";
      case "Error":
        return "destructive";
      default:
        return "outline";
    }
  }

  function getProgress(task: DownloadTask): number {
    if (task.total_bytes <= 0) return 0;
    return Math.round((task.downloaded_bytes / task.total_bytes) * 100);
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
  }

  async function handleRemove(id: string) {
    try {
      await removeDownload(id);
      onRemove(id);
    } catch (e) {
      console.error("Failed to remove download:", e);
    }
  }
</script>

{#if downloads.length === 0}
  <div
    class="flex flex-1 flex-col items-center justify-center gap-3 text-muted-foreground"
  >
    <FileDown class="h-12 w-12 opacity-30" />
    <p class="text-sm">No downloads yet</p>
    <p class="text-xs">Click "New Download" to get started</p>
  </div>
{:else}
  <div class="flex flex-col gap-2">
    {#each downloads as task (task.id)}
      <div
        class="group flex items-center gap-4 rounded-lg border border-border bg-card p-4 transition-colors hover:border-primary/30"
      >
        <!-- Icon -->
        <div
          class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-secondary"
        >
          {#if task.state.type === "Completed"}
            <FileCheck2 class="h-5 w-5 text-green-400" />
          {:else if task.state.type === "Downloading"}
            <FileDown class="h-5 w-5 text-primary animate-pulse" />
          {:else if task.state.type === "Paused"}
            <Pause class="h-5 w-5 text-yellow-400" />
          {:else if task.state.type === "Error"}
            <AlertCircle class="h-5 w-5 text-destructive" />
          {:else}
            <Clock class="h-5 w-5 text-muted-foreground" />
          {/if}
        </div>

        <!-- Info -->
        <div class="flex min-w-0 flex-1 flex-col gap-1.5">
          <div class="flex items-center gap-2">
            <span class="truncate text-sm font-medium">{task.filename}</span>
            <Badge
              variant={getStateBadgeVariant(task.state)}
              class="shrink-0 text-[10px]"
            >
              {getStateType(task.state)}
            </Badge>
          </div>

          {#if task.state.type === "Downloading" || task.state.type === "Paused"}
            <Progress value={getProgress(task)} class="h-1.5" />
            <div class="flex items-center justify-between">
              <span class="text-[11px] text-muted-foreground">
                {formatBytes(task.downloaded_bytes)} / {formatBytes(
                  task.total_bytes,
                )}
              </span>
              <span class="text-[11px] text-muted-foreground"
                >{getProgress(task)}%</span
              >
            </div>
          {:else}
            <span class="text-[11px] text-muted-foreground">
              {task.total_bytes > 0
                ? formatBytes(task.total_bytes)
                : "Unknown size"}
            </span>
          {/if}
        </div>

        <!-- Actions -->
        <Button
          variant="ghost"
          size="icon"
          class="h-8 w-8 shrink-0 text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100 hover:text-destructive"
          onclick={() => handleRemove(task.id)}
        >
          <Trash2 class="h-4 w-4" />
        </Button>
      </div>
    {/each}
  </div>
{/if}
