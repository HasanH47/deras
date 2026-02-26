<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { verifyChecksum } from "$lib/commands";
  import { ShieldCheck, Loader2 } from "@lucide/svelte";

  let {
    open = $bindable(false),
    downloadId,
    filename,
  }: {
    open: boolean;
    downloadId: string;
    filename: string;
  } = $props();

  let hashType = $state<"md5" | "sha256">("sha256");
  let expectedHash = $state("");
  let loading = $state(false);
  let result = $state<boolean | null>(null);

  async function handleVerify() {
    if (!expectedHash.trim()) return;
    loading = true;
    result = null;
    try {
      result = await verifyChecksum(downloadId, hashType, expectedHash.trim());
    } catch (e) {
      console.error("Checksum verification failed:", e);
      result = false;
    } finally {
      loading = false;
    }
  }

  function resetAndClose() {
    expectedHash = "";
    result = null;
    open = false;
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title class="flex items-center gap-2">
        <ShieldCheck class="h-5 w-5 text-primary" />
        Verify Checksum
      </Dialog.Title>
      <Dialog.Description>
        Verify the integrity of <strong>{filename}</strong>
      </Dialog.Description>
    </Dialog.Header>
    <div class="flex flex-col gap-3 py-2">
      <div class="flex gap-2">
        <Button
          variant={hashType === "sha256" ? "default" : "outline"}
          size="sm"
          onclick={() => {
            hashType = "sha256";
            result = null;
          }}
        >
          SHA-256
        </Button>
        <Button
          variant={hashType === "md5" ? "default" : "outline"}
          size="sm"
          onclick={() => {
            hashType = "md5";
            result = null;
          }}
        >
          MD5
        </Button>
      </div>
      <Input
        bind:value={expectedHash}
        placeholder="Paste expected hash here..."
        class="font-mono text-xs"
      />

      {#if result !== null}
        <div class="flex items-center gap-2">
          {#if result}
            <Badge variant="secondary" class="bg-green-500/20 text-green-400">
              ✓ Match — File integrity verified
            </Badge>
          {:else}
            <Badge variant="destructive">
              ✗ Mismatch — File may be corrupted
            </Badge>
          {/if}
        </div>
      {/if}
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={resetAndClose}>Close</Button>
      <Button
        onclick={handleVerify}
        disabled={loading || !expectedHash.trim()}
        class="gap-2"
      >
        {#if loading}
          <Loader2 class="h-4 w-4 animate-spin" />
        {:else}
          <ShieldCheck class="h-4 w-4" />
        {/if}
        Verify
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
