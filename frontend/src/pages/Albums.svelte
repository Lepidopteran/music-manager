<script lang="ts">
	import Explorer from "@components/music/Explorer.svelte";
	import type { PageComponentProps } from "@lib/state/app.svelte";

	let { app }: PageComponentProps = $props();
</script>

<Explorer items={app.albums} onItemChange={(item) => (app.selectedItem = item)} class="h-full"
></Explorer>
