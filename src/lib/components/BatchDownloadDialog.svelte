<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { addDownload } from "$lib/commands";
  import type { DownloadTask } from "$lib/types/models";
  import {
    ListPlus,
    FolderDown,
    Link2,
    CheckSquare,
    Square,
  } from "@lucide/svelte";

  let {
    open = $bindable(false),
    onAdded,
  }: {
    open: boolean;
    onAdded: (task: DownloadTask) => void;
  } = $props();

  let rawText = $state("");
  let savePath = $state("~/Downloads");
  let loading = $state(false);

  // Parsed links
  let parsedLinks = $state<{ url: string; selected: boolean }[]>([]);

  function handleParse() {
    // Basic regex for http/https URLs
    const regex = /(https?:\/\/[^\s"'<>\n\r]+)/gi;
    const matches = rawText.match(regex);

    if (matches) {
      // Deduplicate
      const uniqueUrls = Array.from(new Set(matches));
      parsedLinks = uniqueUrls.map((url) => ({ url, selected: true }));
    } else {
      parsedLinks = [];
    }
  }

  function toggleAll() {
    const allSelected = parsedLinks.every((p) => p.selected);
    parsedLinks = parsedLinks.map((p) => ({ ...p, selected: !allSelected }));
  }

  async function handleStartAll() {
    const selectedUrls = parsedLinks
      .filter((p) => p.selected)
      .map((p) => p.url);
    if (selectedUrls.length === 0) return;

    loading = true;
    try {
      for (const url of selectedUrls) {
        // Adding them iteratively
        const task = await addDownload(url.trim(), savePath.trim());
        onAdded(task);
      }
      open = false;
      rawText = "";
      parsedLinks = [];
    } catch (e) {
      console.error("Failed to start batch downloads", e);
    } finally {
      loading = false;
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="sm:max-w-2xl max-h-[85vh] flex flex-col">
    <Dialog.Header>
      <Dialog.Title class="flex items-center gap-2">
        <ListPlus class="h-5 w-5 text-primary" />
        Batch Download
      </Dialog.Title>
      <Dialog.Description>
        Paste a block of text containing multiple URLs. We'll automatically
        extract and queue them.
      </Dialog.Description>
    </Dialog.Header>

    <div class="flex-1 overflow-y-auto pr-4 py-4 flex flex-col gap-4">
      <div class="space-y-2">
        <Label for="batchText">Raw Text Source</Label>
        <textarea
          id="batchText"
          class="flex min-h-[150px] w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
          placeholder="Paste anything here... e.g. `<a href='https://example.com/file1.zip'>File 1</a>`"
          bind:value={rawText}
          oninput={handleParse}
        ></textarea>
      </div>

      <div class="space-y-2">
        <Label for="batchSavePath" class="flex items-center gap-2">
          <FolderDown class="h-4 w-4" /> Save Directory
        </Label>
        <Input
          id="batchSavePath"
          bind:value={savePath}
          placeholder="~/Downloads"
        />
      </div>

      {#if parsedLinks.length > 0}
        <div class="space-y-2 mt-2 border-t pt-4">
          <div class="flex items-center justify-between">
            <Label class="text-primary font-semibold"
              >Extracted Links ({parsedLinks.filter((l) => l.selected)
                .length}/{parsedLinks.length})</Label
            >
            <Button
              variant="ghost"
              size="sm"
              class="h-8 text-xs"
              onclick={toggleAll}
            >
              Toggle All
            </Button>
          </div>
          <div
            class="max-h-[200px] overflow-y-auto space-y-1 rounded-md border p-2 bg-muted/20"
          >
            {#each parsedLinks as link, i}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="flex items-start gap-2 p-1.5 hover:bg-muted rounded-sm cursor-pointer"
                onclick={() => (link.selected = !link.selected)}
              >
                <div class="mt-0.5 text-primary">
                  {#if link.selected}
                    <CheckSquare class="h-4 w-4" />
                  {:else}
                    <Square class="h-4 w-4 text-muted-foreground" />
                  {/if}
                </div>
                <div
                  class="flex-1 break-all text-xs font-mono leading-tight {link.selected
                    ? 'text-foreground'
                    : 'text-muted-foreground'}"
                >
                  {link.url}
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>

    <Dialog.Footer
      class="pt-2 border-t mt-2 flex justify-between sm:justify-between items-center w-full"
    >
      <div class="text-xs text-muted-foreground">
        {#if parsedLinks.length > 0}
          Ready to queue {parsedLinks.filter((p) => p.selected).length} files.
        {:else}
          Waiting for valid links...
        {/if}
      </div>
      <div class="flex gap-2">
        <Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
        <Button
          onclick={handleStartAll}
          disabled={loading ||
            parsedLinks.length === 0 ||
            parsedLinks.filter((p) => p.selected).length === 0}
        >
          {loading ? "Queueing..." : "Start Batch"}
        </Button>
      </div>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
