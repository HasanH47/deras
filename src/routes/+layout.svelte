<script lang="ts">
  import "$lib/../app.css";
  import type { Snippet } from "svelte";
  import { Toaster } from "svelte-sonner";

  let { children }: { children: Snippet } = $props();
</script>

<div
  class="flex h-screen w-screen overflow-hidden bg-background text-foreground"
>
  {@render children()}
</div>

<Toaster
  position="bottom-right"
  richColors
  theme="dark"
  toastOptions={{
    class: "border-border bg-card text-foreground",
  }}
/>
