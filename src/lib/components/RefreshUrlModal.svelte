<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { updateDownloadUrl } from "$lib/commands";
  import type { DownloadTask } from "$lib/types/models";

  let {
    open = $bindable(),
    task,
  }: { open: boolean; task: DownloadTask | null } = $props();

  let newUrl = $state("");
  let isLoading = $state(false);

  $effect(() => {
    if (open && task) {
      newUrl = task.url;
    }
  });

  async function handleUpdate() {
    if (!task || !newUrl) return;
    isLoading = true;
    try {
      await updateDownloadUrl(task.id, newUrl);
      open = false;
    } catch (e) {
      console.error("Failed to update URL:", e);
    } finally {
      isLoading = false;
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="sm:max-w-[500px]">
    <Dialog.Header>
      <Dialog.Title>Refresh Download Link</Dialog.Title>
      <Dialog.Description>
        Update the URL for "<strong>{task?.filename}</strong>". This is useful
        if the original link has expired.
      </Dialog.Description>
    </Dialog.Header>

    <div class="grid gap-4 py-4">
      <div class="grid gap-2">
        <Label for="url">New URL</Label>
        <Input
          id="url"
          placeholder="Paste new download link here..."
          bind:value={newUrl}
          onkeydown={(e) => e.key === "Enter" && handleUpdate()}
        />
      </div>
    </div>

    <Dialog.Footer>
      <Button variant="outline" onclick={() => (open = false)}>Cancel</Button>
      <Button onclick={handleUpdate} disabled={isLoading || !newUrl}>
        {isLoading ? "Updating..." : "Update Link"}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
