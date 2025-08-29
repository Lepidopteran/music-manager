<script lang="ts">
	import Explorer from "@components/music/Explorer.svelte";
	import Editor from "@components/music/Editor.svelte";

	import type { Album, Song } from "@lib/models";
	import { getAlbums } from "@api/album";
	import type { PageComponentProps } from "@lib/state/app.svelte";

	let selectedItem: Album | Song | null = $state(null);

	let { app }: PageComponentProps = $props();
</script>

{#await getAlbums() then albums}
	<div class="grid md:grid-cols-2 h-full">
		<Explorer
			{albums}
			onItemChange={(item) => (selectedItem = item)}
			class="h-full"
		/>
		<Editor bind:selectedItem={selectedItem} />
	</div>
{/await}
