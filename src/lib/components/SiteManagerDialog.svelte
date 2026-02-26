<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Table from "$lib/components/ui/table";
  import { Trash2, Plus, Globe } from "@lucide/svelte";
  import {
    getCredentials,
    saveCredential,
    deleteCredential,
  } from "$lib/commands";

  let { open = $bindable(false) } = $props();

  let credentials = $state<any[]>([]);
  let newDomain = $state("");
  let newUsername = $state("");
  let newPassword = $state("");

  async function loadCredentials() {
    try {
      credentials = await getCredentials();
    } catch (e) {
      console.error(e);
    }
  }

  async function handleAdd() {
    if (!newDomain || !newUsername || !newPassword) return;
    try {
      await saveCredential({
        domain: newDomain,
        username: newUsername,
        password: newPassword,
      });
      newDomain = "";
      newUsername = "";
      newPassword = "";
      await loadCredentials();
    } catch (e) {
      console.error(e);
    }
  }

  async function handleDelete(domain: string) {
    try {
      await deleteCredential(domain);
      await loadCredentials();
    } catch (e) {
      console.error(e);
    }
  }

  $effect(() => {
    if (open) {
      loadCredentials();
    }
  });
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="sm:max-w-[620px]">
    <Dialog.Header>
      <Dialog.Title>Site Manager</Dialog.Title>
      <Dialog.Description>
        Manage credentials for sites that require authentication (Basic Auth).
      </Dialog.Description>
    </Dialog.Header>

    <div class="grid gap-4 py-4">
      <div class="grid grid-cols-12 items-end gap-3">
        <div class="col-span-4 space-y-2">
          <Label for="domain">Domain</Label>
          <Input id="domain" placeholder="example.com" bind:value={newDomain} />
        </div>
        <div class="col-span-3 space-y-2">
          <Label for="username">User</Label>
          <Input id="username" placeholder="user" bind:value={newUsername} />
        </div>
        <div class="col-span-3 space-y-2">
          <Label for="password">Pass</Label>
          <Input
            id="password"
            type="password"
            placeholder="••••••"
            bind:value={newPassword}
          />
        </div>
        <div class="col-span-2">
          <Button onclick={handleAdd} class="w-full">
            <Plus class="h-4 w-4" />
          </Button>
        </div>
      </div>

      <div class="rounded-md border max-h-[400px] overflow-auto">
        <Table.Root>
          <Table.Header>
            <Table.Row>
              <Table.Head>Domain</Table.Head>
              <Table.Head>Username</Table.Head>
              <Table.Head class="text-right">Action</Table.Head>
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {#each credentials as cred}
              <Table.Row>
                <Table.Cell class="font-medium">
                  <div class="flex items-center gap-2">
                    <Globe class="h-4 w-4 text-muted-foreground" />
                    {cred.domain}
                  </div>
                </Table.Cell>
                <Table.Cell>{cred.username}</Table.Cell>
                <Table.Cell class="text-right">
                  <Button
                    variant="ghost"
                    size="icon"
                    onclick={() => handleDelete(cred.domain)}
                    class="text-destructive h-8 w-8"
                  >
                    <Trash2 class="h-4 w-4" />
                  </Button>
                </Table.Cell>
              </Table.Row>
            {:else}
              <Table.Row>
                <Table.Cell
                  colspan={3}
                  class="h-24 text-center text-muted-foreground"
                >
                  No credentials saved.
                </Table.Cell>
              </Table.Row>
            {/each}
          </Table.Body>
        </Table.Root>
      </div>
    </div>
  </Dialog.Content>
</Dialog.Root>
