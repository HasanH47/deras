<script lang="ts">
  import "$lib/../app.css";
  import type { Snippet } from "svelte";
  import { Toaster } from "svelte-sonner";
  import { ModeWatcher } from "mode-watcher";
  import * as Tooltip from "$lib/components/ui/tooltip";

  let { children }: { children: Snippet } = $props();
</script>

<ModeWatcher />

<Tooltip.Provider>
  <div
    class="flex h-screen w-screen overflow-hidden bg-background text-foreground"
  >
    {@render children()}
  </div>
</Tooltip.Provider>

<Toaster
  position="bottom-right"
  richColors
  theme="dark"
  toastOptions={{
    class: "border-border bg-card text-foreground",
  }}
/>
