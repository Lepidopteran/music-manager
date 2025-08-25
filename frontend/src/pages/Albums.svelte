<script lang="ts">
	import Explorer from "@components/music/Explorer.svelte";
	import Editor from "@components/music/Editor.svelte";

	import type { ClassValue } from "svelte/elements";
	import type { Album, Song } from "@lib/models";
	import { getAlbums } from "@api/album";

	interface Props {
		albums: Array<Album>;
		class?: ClassValue;
		[props: string]: unknown;
	}

	let { class: className, albums, ...rest }: Props = $props();
	let selectedItem: Album | Song | null = $state(null);
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
