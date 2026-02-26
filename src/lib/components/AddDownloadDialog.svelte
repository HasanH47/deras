<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { addDownload } from "$lib/commands";
  import type { DownloadTask } from "$lib/types/models";
  import { Link, FolderDown } from "@lucide/svelte";

  let {
    open = $bindable(false),
    onAdded,
  }: {
    open: boolean;
    onAdded: (task: DownloadTask) => void;
  } = $props();

  let url = $state("");
  let savePath = $state("~/Downloads");
  let loading = $state(false);
  let error = $state("");

  async function handleSubmit() {
    if (!url.trim()) {
      error = "Please enter a URL.";
      return;
    }

    loading = true;
    error = "";

    try {
      const task = await addDownload(url.trim(), savePath.trim());
      onAdded(task);
      url = "";
      error = "";
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>New Download</Dialog.Title>
      <Dialog.Description
        >Paste a URL to start downloading a file.</Dialog.Description
      >
    </Dialog.Header>

    <form onsubmit={handleSubmit} class="flex flex-col gap-4 py-2">
      <!-- URL Input -->
      <div class="flex flex-col gap-1.5">
        <label
          for="url-input"
          class="flex items-center gap-1.5 text-sm font-medium"
        >
          <Link class="h-3.5 w-3.5" />
          URL
        </label>
        <Input
          id="url-input"
          bind:value={url}
          placeholder="https://example.com/file.zip"
          type="url"
        />
      </div>

      <!-- Save Path Input -->
      <div class="flex flex-col gap-1.5">
        <label
          for="path-input"
          class="flex items-center gap-1.5 text-sm font-medium"
        >
          <FolderDown class="h-3.5 w-3.5" />
          Save to
        </label>
        <Input
          id="path-input"
          bind:value={savePath}
          placeholder="~/Downloads"
        />
      </div>

      {#if error}
        <p class="text-sm text-destructive">{error}</p>
      {/if}

      <Dialog.Footer>
        <Button type="button" variant="outline" onclick={() => (open = false)}>
          Cancel
        </Button>
        <Button type="submit" disabled={loading}>
          {loading ? "Adding..." : "Add Download"}
        </Button>
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>
