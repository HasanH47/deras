<script lang="ts">
  import {
    Download,
    CheckCircle,
    ListFilter,
    Plus,
    Settings,
    Film,
    Music,
    FileText,
    Archive,
    AppWindow,
    Image as ImageIcon,
    FileQuestion,
    Activity,
    Shield,
  } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { Separator } from "$lib/components/ui/separator";
  import ThemeToggle from "./ThemeToggle.svelte";
  import SettingsDialog from "./SettingsDialog.svelte";
  import SiteManagerDialog from "./SiteManagerDialog.svelte";
  import BatchDownloadDialog from "./BatchDownloadDialog.svelte";
  import type { FilterMode } from "$lib/types/models";

  let {
    activeFilter,
    onFilterChange,
    onAddClick,
  }: {
    activeFilter: FilterMode;
    onFilterChange: (filter: FilterMode) => void;
    onAddClick: () => void;
  } = $props();

  let settingsOpen = $state(false);
  let siteManagerOpen = $state(false);
  let batchOpen = $state(false);

  const navItems: {
    label: string;
    filter: FilterMode;
    icon: typeof Download;
  }[] = [
    { label: "All Downloads", filter: "all", icon: ListFilter },
    { label: "Downloading", filter: "downloading", icon: Download },
    { label: "Completed", filter: "completed", icon: CheckCircle },
    { label: "Analytics", filter: "analytics", icon: Activity },
  ];

  const categoryItems: {
    label: string;
    filter: FilterMode;
    icon: typeof Download;
  }[] = [
    { label: "Videos", filter: "Video", icon: Film },
    { label: "Audio", filter: "Audio", icon: Music },
    { label: "Documents", filter: "Document", icon: FileText },
    { label: "Archives", filter: "Archive", icon: Archive },
    { label: "Applications", filter: "Application", icon: AppWindow },
    { label: "Images", filter: "Image", icon: ImageIcon },
    { label: "Others", filter: "Other", icon: FileQuestion },
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
    <div class="flex items-center gap-1">
      <Button
        variant="ghost"
        size="icon"
        class="h-9 w-9"
        onclick={() => (siteManagerOpen = true)}
      >
        <Shield class="h-4 w-4" />
        <span class="sr-only">Site Manager</span>
      </Button>
      <Button
        variant="ghost"
        size="icon"
        class="h-9 w-9"
        onclick={() => (settingsOpen = true)}
      >
        <Settings class="h-4 w-4" />
        <span class="sr-only">Settings</span>
      </Button>
      <ThemeToggle />
    </div>
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

  <div class="px-3 py-2">
    <h2
      class="mb-2 px-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground"
    >
      Categories
    </h2>
    <nav class="flex flex-col gap-1">
      {#each categoryItems as item}
        <button
          class="flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors {activeFilter ===
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
  </div>

  <!-- Add Download Button -->
  <div class="p-3 pb-4 space-y-2">
    <Button class="w-full gap-2" onclick={onAddClick}>
      <Plus class="h-4 w-4" />
      New Download
    </Button>
    <Button
      variant="secondary"
      class="w-full gap-2 text-xs h-8"
      onclick={() => (batchOpen = true)}
    >
      <ListFilter class="h-3 w-3" />
      Batch Download
    </Button>
  </div>
</aside>

<SettingsDialog bind:open={settingsOpen} />
<SiteManagerDialog bind:open={siteManagerOpen} />
<BatchDownloadDialog bind:open={batchOpen} onAdded={() => {}} />
