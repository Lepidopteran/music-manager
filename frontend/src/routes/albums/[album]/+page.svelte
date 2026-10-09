<script lang="ts">
	import GridList from "#components/GridList.svelte";
	import CoverView from "#components/music/CoverView.svelte";

	import type { PageProps } from "./$types";

	import { editedSongs, groupManager, selectedSongs } from "#lib/state.ts";

	const selected = selectedSongs();
	const groupState = groupManager();
	const edited = editedSongs();
	const albums = $derived(groupState.groups.get("album"));

	let { params }: PageProps = $props();

	const { album } = $derived(params);
</script>

{#if albums}
	<GridList
		class="p-4"
		data={albums.get(album as string)!.sort((a, b) =>
			Number(a.trackNumber) - Number(b.trackNumber)
		)}
		getKey={(song) => song.id}
		selected={selected}
		columnWidth={128}
	>
		{#snippet item({ data })}
			<CoverView
				src="/api/songs/{data.id}/cover-art/front.jpg"
				class="mb-1 rounded-theme shadow-lg shadow-shade/25 size-32 mx-auto"
			/>
			<div class="truncate text-center">
				{edited.get(data.id)?.title || data.title}
			</div>
		{/snippet}
	</GridList>
{/if}
