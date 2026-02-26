<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import type { DownloadTask } from "$lib/types/models";
  import { Activity } from "@lucide/svelte";

  let { downloads }: { downloads: DownloadTask[] } = $props();

  // Keep 60 seconds of history
  const HISTORY_SIZE = 60;
  let history = $state<number[]>(Array(HISTORY_SIZE).fill(0));

  let lastTotalBytes = 0;
  let currentSpeed = $state(0);
  let intervalId: number;

  function formatSpeed(bytesPerSec: number): string {
    if (bytesPerSec === 0) return "0 B/s";
    const k = 1024;
    const sizes = ["B/s", "KB/s", "MB/s", "GB/s"];
    const i = Math.floor(Math.log(bytesPerSec) / Math.log(k));
    return (
      parseFloat((bytesPerSec / Math.pow(k, i)).toFixed(1)) + " " + sizes[i]
    );
  }

  onMount(() => {
    // Initialize starting point
    lastTotalBytes = downloads
      .filter((d) => d.state.type === "Downloading")
      .reduce((acc, d) => acc + d.downloaded_bytes, 0);

    intervalId = setInterval(() => {
      // Calculate current total
      const currentTotalBytes = downloads
        .filter((d) => d.state.type === "Downloading")
        .reduce((acc, d) => acc + d.downloaded_bytes, 0);

      // Diff
      let speed = currentTotalBytes - lastTotalBytes;
      if (speed < 0) speed = 0; // Guard against restarts/removals

      currentSpeed = speed;
      lastTotalBytes = currentTotalBytes;

      // Update history array (shift left, push new)
      history = [...history.slice(1), speed];
    }, 1000) as unknown as number;
  });

  onDestroy(() => {
    if (intervalId) clearInterval(intervalId);
  });

  // Calculate SVG path based on history
  let maxSpeed = $derived(Math.max(...history, 1024 * 1024)); // Minimum scale of 1MB/s
  let points = $derived(
    history
      .map((val, i) => {
        const x = (i / (HISTORY_SIZE - 1)) * 100; // 0 to 100%
        const y = 100 - (val / maxSpeed) * 100; // 0 to 100% (inverted for SVG)
        return `${x},${y}`;
      })
      .join(" "),
  );
</script>

<div class="flex h-full flex-col p-4 gap-6">
  <div class="flex items-center gap-3">
    <div
      class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-primary/20 text-primary"
    >
      <Activity class="h-5 w-5" />
    </div>
    <div>
      <h2 class="text-xl font-bold tracking-tight">Global Network Traffic</h2>
      <p class="text-sm text-muted-foreground">
        Aggregated throughput of all active downloads
      </p>
    </div>
  </div>

  <!-- Main Chart Card -->
  <div
    class="flex flex-col rounded-xl border bg-card text-card-foreground shadow-sm overflow-hidden"
  >
    <div class="p-6 pb-2">
      <div
        class="text-sm font-medium text-muted-foreground uppercase tracking-wider mb-1"
      >
        Current Speed
      </div>
      <div
        class="text-4xl font-extrabold tracking-tighter text-primary tabular-nums"
      >
        {formatSpeed(currentSpeed)}
      </div>
    </div>
    <div class="relative mt-4 h-[200px] w-full p-6 pt-0">
      <!-- Grid Lines -->
      <div class="absolute inset-0 px-6 pb-6 pt-0">
        <div
          class="h-full w-full border-b border-l border-muted/50 flex flex-col justify-between"
        >
          <div class="w-full border-t border-muted/30"></div>
          <div class="w-full border-t border-muted/30"></div>
          <div class="w-full border-t border-muted/30"></div>
          <div class="w-full border-t border-muted/30"></div>
        </div>
      </div>

      <!-- SVG Chart -->
      <div class="relative h-full w-full">
        <svg
          class="h-full w-full overflow-visible"
          preserveAspectRatio="none"
          viewBox="0 0 100 100"
        >
          <!-- Gradient Fill -->
          <defs>
            <linearGradient id="chart-gradient" x1="0" y1="0" x2="0" y2="1">
              <stop
                offset="0%"
                stop-color="var(--primary)"
                stop-opacity="0.3"
              />
              <stop
                offset="100%"
                stop-color="var(--primary)"
                stop-opacity="0.0"
              />
            </linearGradient>
          </defs>

          <polygon
            points="0,100 {points} 100,100"
            fill="url(#chart-gradient)"
            class="transition-all duration-300 ease-in-out"
          />
          <polyline
            {points}
            fill="none"
            stroke="var(--primary)"
            stroke-width="2"
            vector-effect="non-scaling-stroke"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="transition-all duration-300 ease-in-out drop-shadow-sm"
          />
        </svg>
      </div>

      <!-- Y-Axis Scale Label -->
      <div
        class="absolute right-8 top-0 text-[10px] text-muted-foreground font-medium"
      >
        {formatSpeed(maxSpeed)}
      </div>
      <div
        class="absolute right-8 bottom-6 text-[10px] text-muted-foreground font-medium"
      >
        0 B/s
      </div>
    </div>
  </div>

  <!-- Legend / Info -->
  <div class="grid gap-4 mt-2 sm:grid-cols-2">
    <div
      class="rounded-lg border bg-card p-4 shadow-sm flex items-center justify-between"
    >
      <span class="text-sm font-medium text-muted-foreground"
        >Active Connections</span
      >
      <span class="text-xl font-bold"
        >{downloads.filter((d) => d.state.type === "Downloading").length}</span
      >
    </div>
    <div
      class="rounded-lg border bg-card p-4 shadow-sm flex items-center justify-between"
    >
      <span class="text-sm font-medium text-muted-foreground">Peak Speed</span>
      <span class="text-xl font-bold">{formatSpeed(maxSpeed)}</span>
    </div>
  </div>
</div>
