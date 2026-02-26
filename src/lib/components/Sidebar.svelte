<script lang="ts">
  import { Download, CheckCircle, ListFilter, Plus } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { Separator } from "$lib/components/ui/separator";
  import ThemeToggle from "./ThemeToggle.svelte";

  type FilterMode = "all" | "downloading" | "completed";

  let {
    activeFilter,
    onFilterChange,
    onAddClick,
  }: {
    activeFilter: FilterMode;
    onFilterChange: (filter: FilterMode) => void;
    onAddClick: () => void;
  } = $props();

  const navItems: {
    label: string;
    filter: FilterMode;
    icon: typeof Download;
  }[] = [
    { label: "All Downloads", filter: "all", icon: ListFilter },
    { label: "Downloading", filter: "downloading", icon: Download },
    { label: "Completed", filter: "completed", icon: CheckCircle },
  ];
</script>

<aside
  class="flex h-full w-60 shrink-0 flex-col border-r border-sidebar-border bg-sidebar text-sidebar-foreground"
>
  <!-- Logo / App Name -->
  <div class="flex h-14 items-center justify-between px-5">
    <div class="flex items-center gap-2">
      <Download class="h-5 w-5 text-primary" />
      <span class="text-base font-bold tracking-tight">Deras</span>
    </div>
    <ThemeToggle />
  </div>

  <Separator />

  <!-- Navigation -->
  <nav class="flex flex-1 flex-col gap-1 px-3 py-3">
    {#each navItems as item}
      <button
        class="flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors {activeFilter ===
        item.filter
          ? 'bg-sidebar-accent text-sidebar-accent-foreground'
          : 'text-muted-foreground hover:bg-sidebar-accent/50 hover:text-sidebar-accent-foreground'}"
        onclick={() => onFilterChange(item.filter)}
      >
        <item.icon class="h-4 w-4" />
        {item.label}
      </button>
    {/each}
  </nav>

  <!-- Add Download Button -->
  <div class="p-3">
    <Button class="w-full gap-2" onclick={onAddClick}>
      <Plus class="h-4 w-4" />
      New Download
    </Button>
  </div>
</aside>
