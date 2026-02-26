<script lang="ts">
  import Sidebar from "$lib/components/Sidebar.svelte";
  import DownloadList from "$lib/components/DownloadList.svelte";
  import AddDownloadDialog from "$lib/components/AddDownloadDialog.svelte";
  import type { DownloadTask } from "$lib/types/models";
  import { getDownloads } from "$lib/commands";
  import { onMount } from "svelte";

  type FilterMode = "all" | "downloading" | "completed";

  let downloads = $state<DownloadTask[]>([]);
  let activeFilter = $state<FilterMode>("all");
  let showAddDialog = $state(false);

  let filteredDownloads = $derived.by(() => {
    switch (activeFilter) {
      case "downloading":
        return downloads.filter(
          (d) =>
            d.state.type === "Downloading" ||
            d.state.type === "Paused" ||
            d.state.type === "Pending",
        );
      case "completed":
        return downloads.filter((d) => d.state.type === "Completed");
      default:
        return downloads;
    }
  });

  async function loadDownloads() {
    try {
      downloads = await getDownloads();
    } catch (e) {
      console.error("Failed to load downloads:", e);
    }
  }

  function handleDownloadAdded(task: DownloadTask) {
    downloads = [task, ...downloads];
    showAddDialog = false;
  }

  function handleDownloadRemoved(id: string) {
    downloads = downloads.filter((d) => d.id !== id);
  }

  onMount(() => {
    loadDownloads();
  });
</script>

<Sidebar
  {activeFilter}
  onFilterChange={(f) => (activeFilter = f)}
  onAddClick={() => (showAddDialog = true)}
/>

<main class="flex flex-1 flex-col overflow-hidden">
  <!-- Header -->
  <header
    class="flex h-14 shrink-0 items-center justify-between border-b border-border px-6"
  >
    <h1 class="text-lg font-semibold capitalize">
      {activeFilter === "all" ? "All Downloads" : activeFilter}
    </h1>
    <span class="text-sm text-muted-foreground"
      >{filteredDownloads.length} items</span
    >
  </header>

  <!-- Content -->
  <div class="flex-1 overflow-y-auto p-4">
    <DownloadList
      downloads={filteredDownloads}
      onRemove={handleDownloadRemoved}
    />
  </div>
</main>

<AddDownloadDialog bind:open={showAddDialog} onAdded={handleDownloadAdded} />
