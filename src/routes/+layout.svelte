<script lang="ts">
	import "$lib/../app.css";
	import type { Snippet } from "svelte";

	let { children }: { children: Snippet } = $props();
</script>

<div class="flex h-screen w-screen overflow-hidden bg-background text-foreground">
	{@render children()}
</div>
