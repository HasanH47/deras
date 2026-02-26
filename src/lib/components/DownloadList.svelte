<script lang="ts">
  import type { DownloadTask, DownloadState } from "$lib/types/models";
  import {
    removeDownload,
    pauseDownload,
    resumeDownload,
    cancelDownload,
    moveDownload,
    forceStartDownload,
  } from "$lib/commands";
  import { Progress } from "$lib/components/ui/progress";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
  } from "$lib/components/ui/tooltip";
  import {
    Trash2,
    FileDown,
    FileCheck2,
    Pause,
    Play,
    X,
    AlertCircle,
    Clock,
    ChevronUp,
    ChevronDown,
    Zap,
    ShieldCheck,
  } from "@lucide/svelte";

  let {
    downloads,
    onRemove,
    onReorder,
    onChecksum,
  }: {
    downloads: DownloadTask[];
    onRemove: (id: string) => void;
    onReorder: () => void;
    onChecksum: (id: string, filename: string) => void;
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

  function getErrorMessage(state: DownloadState): string {
    if (state.type === "Error" && "message" in state) {
      return state.message;
    }
    return "";
  }

  async function handleRemove(id: string) {
    try {
      await removeDownload(id);
      onRemove(id);
    } catch (e) {
      console.error("Failed to remove download:", e);
    }
  }

  async function handlePause(id: string) {
    try {
      await pauseDownload(id);
    } catch (e) {
      console.error("Failed to pause download:", e);
    }
  }

  async function handleResume(id: string) {
    try {
      await resumeDownload(id);
    } catch (e) {
      console.error("Failed to resume download:", e);
    }
  }

  async function handleCancel(id: string) {
    try {
      await cancelDownload(id);
      onRemove(id);
    } catch (e) {
      console.error("Failed to cancel download:", e);
    }
  }

  async function handleMove(id: string, direction: "up" | "down") {
    try {
      await moveDownload(id, direction);
      onReorder();
    } catch (e) {
      console.error("Failed to move download:", e);
    }
  }

  async function handleForceStart(id: string) {
    try {
      await forceStartDownload(id);
    } catch (e) {
      console.error("Failed to force start download:", e);
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
          {:else if task.state.type === "Error"}
            <span class="truncate text-[11px] text-destructive">
              {getErrorMessage(task.state)}
            </span>
          {:else}
            <span class="text-[11px] text-muted-foreground">
              {task.total_bytes > 0 ? formatBytes(task.total_bytes) : "Queued"}
            </span>
          {/if}
        </div>

        <!-- Actions -->
        <div
          class="flex shrink-0 items-center gap-1 opacity-0 transition-opacity group-hover:opacity-100"
        >
          {#if task.state.type === "Pending"}
            <!-- Queue controls for pending items -->
            <Tooltip>
              <TooltipTrigger>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7 text-muted-foreground hover:text-foreground"
                  onclick={() => handleMove(task.id, "up")}
                >
                  <ChevronUp class="h-4 w-4" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>Move Up</TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7 text-muted-foreground hover:text-foreground"
                  onclick={() => handleMove(task.id, "down")}
                >
                  <ChevronDown class="h-4 w-4" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>Move Down</TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7 text-muted-foreground hover:text-yellow-400"
                  onclick={() => handleForceStart(task.id)}
                >
                  <Zap class="h-4 w-4" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>Force Start</TooltipContent>
            </Tooltip>
          {/if}

          {#if task.state.type === "Downloading"}
            <Tooltip>
              <TooltipTrigger>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-8 w-8 text-muted-foreground hover:text-yellow-400"
                  onclick={() => handlePause(task.id)}
                >
                  <Pause class="h-4 w-4" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>Pause</TooltipContent>
            </Tooltip>
          {/if}

          {#if task.state.type === "Paused" || task.state.type === "Error"}
            <Tooltip>
              <TooltipTrigger>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-8 w-8 text-muted-foreground hover:text-green-400"
                  onclick={() => handleResume(task.id)}
                >
                  <Play class="h-4 w-4" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>
                {task.state.type === "Paused" ? "Resume" : "Retry"}
              </TooltipContent>
            </Tooltip>
          {/if}

          {#if task.state.type === "Downloading" || task.state.type === "Paused" || task.state.type === "Pending"}
            <Tooltip>
              <TooltipTrigger>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-8 w-8 text-muted-foreground hover:text-destructive"
                  onclick={() => handleCancel(task.id)}
                >
                  <X class="h-4 w-4" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>Cancel</TooltipContent>
            </Tooltip>
          {/if}

          {#if task.state.type === "Completed"}
            <Tooltip>
              <TooltipTrigger>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-8 w-8 text-muted-foreground hover:text-primary"
                  onclick={() => onChecksum(task.id, task.filename)}
                >
                  <ShieldCheck class="h-4 w-4" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>Verify Checksum</TooltipContent>
            </Tooltip>
          {/if}

          {#if task.state.type === "Completed" || task.state.type === "Error"}
            <Tooltip>
              <TooltipTrigger>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-8 w-8 text-muted-foreground hover:text-destructive"
                  onclick={() => handleRemove(task.id)}
                >
                  <Trash2 class="h-4 w-4" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>Remove</TooltipContent>
            </Tooltip>
          {/if}
        </div>
      </div>
    {/each}
  </div>
{/if}
