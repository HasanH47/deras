<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import { addDownload } from "$lib/commands";
  import type { DownloadTask } from "$lib/types/models";
  import { LinkIcon, Download } from "@lucide/svelte";

  let {
    open = $bindable(false),
    url = $bindable(""),
    onAdded,
  }: {
    open: boolean;
    url: string;
    onAdded: (task: DownloadTask) => void;
  } = $props();

  let savePath = $state("~/Downloads");
  let loading = $state(false);

  async function handleSubmit() {
    if (!url.trim()) return;
    loading = true;
    try {
      const task = await addDownload(url.trim(), savePath.trim());
      onAdded(task);
      url = "";
      open = false;
    } catch (e) {
      console.error("Failed to add download:", e);
    } finally {
      loading = false;
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title class="flex items-center gap-2">
        <LinkIcon class="h-5 w-5 text-primary" />
        URL Detected in Clipboard
      </Dialog.Title>
      <Dialog.Description>
        We found a URL in your clipboard. Want to download it?
      </Dialog.Description>
    </Dialog.Header>
    <div class="flex flex-col gap-3 py-2">
      <Input
        bind:value={url}
        placeholder="https://example.com/file.zip"
        class="font-mono text-xs"
      />
      <Input bind:value={savePath} placeholder="~/Downloads" class="text-sm" />
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => (open = false)}>Dismiss</Button>
      <Button
        onclick={handleSubmit}
        disabled={loading || !url.trim()}
        class="gap-2"
      >
        <Download class="h-4 w-4" />
        Download
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
