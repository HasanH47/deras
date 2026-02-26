<script lang="ts">
  import { toggleMode, mode } from "mode-watcher";
  import { Button } from "$lib/components/ui/button";
  import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
  } from "$lib/components/ui/tooltip";
  import { Sun, Moon } from "@lucide/svelte";
</script>

<Tooltip>
  <TooltipTrigger>
    <Button variant="ghost" size="icon" class="h-8 w-8" onclick={toggleMode}>
      {#if mode.current === "dark"}
        <Sun class="h-4 w-4" />
      {:else}
        <Moon class="h-4 w-4" />
      {/if}
    </Button>
  </TooltipTrigger>
  <TooltipContent>
    {mode.current === "dark" ? "Light Mode" : "Dark Mode"}
  </TooltipContent>
</Tooltip>
