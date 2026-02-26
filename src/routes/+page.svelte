<script lang="ts">
  import Sidebar from "$lib/components/Sidebar.svelte";
  import DownloadList from "$lib/components/DownloadList.svelte";
  import AddDownloadDialog from "$lib/components/AddDownloadDialog.svelte";
  import NewDownloadModal from "$lib/components/NewDownloadModal.svelte";
  import ChecksumDialog from "$lib/components/ChecksumDialog.svelte";
  import type { DownloadTask } from "$lib/types/models";
  import {
    getDownloads,
    listenToProgress,
    listenToClipboardUrl,
  } from "$lib/commands";
  import type { DownloadProgressPayload } from "$lib/commands";
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";

  type FilterMode = "all" | "downloading" | "completed";

  let downloads = $state<DownloadTask[]>([]);
  let activeFilter = $state<FilterMode>("all");
  let showAddDialog = $state(false);

  // Clipboard auto-detect modal
  let showClipboardModal = $state(false);
  let clipboardUrl = $state("");

  // Checksum dialog
  let showChecksumDialog = $state(false);
  let checksumTaskId = $state("");
  let checksumFilename = $state("");

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

  function handleClipboardDownloadAdded(task: DownloadTask) {
    downloads = [task, ...downloads];
    showClipboardModal = false;
    clipboardUrl = "";
  }

  function handleDownloadRemoved(id: string) {
    downloads = downloads.filter((d) => d.id !== id);
  }

  function handleChecksum(id: string, filename: string) {
    checksumTaskId = id;
    checksumFilename = filename;
    showChecksumDialog = true;
  }

  function handleProgressUpdate(payload: DownloadProgressPayload) {
    downloads = downloads.map((d) => {
      if (d.id === payload.id) {
        const prev = d.state.type;
        const next = payload.state.type;

        if (prev !== next) {
          if (next === "Completed") {
            toast.success(`Download complete: ${d.filename}`);
          } else if (next === "Error" && "message" in payload.state) {
            toast.error(`Download failed: ${d.filename}`, {
              description: payload.state.message,
            });
          }
        }

        return {
          ...d,
          state: payload.state,
          downloaded_bytes: payload.downloaded_bytes,
          total_bytes: payload.total_bytes,
        };
      }
      return d;
    });
  }

  onMount(() => {
    loadDownloads();

    const unlistenProgress = listenToProgress(handleProgressUpdate);
    const unlistenClipboard = listenToClipboardUrl((url: string) => {
      clipboardUrl = url;
      showClipboardModal = true;
    });

    return () => {
      unlistenProgress.then((unlisten) => unlisten());
      unlistenClipboard.then((unlisten) => unlisten());
    };
  });
</script>

<Sidebar
  {activeFilter}
  onFilterChange={(f) => (activeFilter = f)}
  onAddClick={() => (showAddDialog = true)}
/>

<main class="flex flex-1 flex-col overflow-hidden">
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

  <div class="flex-1 overflow-y-auto p-4">
    <DownloadList
      downloads={filteredDownloads}
      onRemove={handleDownloadRemoved}
      onReorder={loadDownloads}
      onChecksum={handleChecksum}
    />
  </div>
</main>

<AddDownloadDialog bind:open={showAddDialog} onAdded={handleDownloadAdded} />

<NewDownloadModal
  bind:open={showClipboardModal}
  bind:url={clipboardUrl}
  onAdded={handleClipboardDownloadAdded}
/>

<ChecksumDialog
  bind:open={showChecksumDialog}
  downloadId={checksumTaskId}
  filename={checksumFilename}
/>
