<script lang="ts">
	import GridList from "#components/GridList.svelte";
	import CoverView from "#components/music/CoverView.svelte";

	import type { PageProps } from "./$types";

	import { app } from "#lib/state/app.svelte.ts";
	import { songGroups } from "#lib/state/group.svelte.ts";

	const { editedSongs, selectedSongs } = $derived(app);
	const albums = $derived(songGroups.groups.get("album"));

	if (!songGroups.tracked.includes("album")) {
		songGroups.track("album");
	}

	let { params }: PageProps = $props();

	const { album } = $derived(params);
</script>

{#if albums && albums.has(album as string)}
	<GridList
		class="p-4"
		data={albums.get(album as string)!.sort((a, b) =>
			Number(a.trackNumber) - Number(b.trackNumber)
		)}
		getKey={(song) => song.id}
		selected={selectedSongs}
		columnWidth={128}
	>
		{#snippet item({ data })}
			<CoverView
				src="/api/songs/{data.id}/cover-art/front.jpg"
				class="mb-1 rounded-theme shadow-lg shadow-shade/25 size-32 mx-auto"
			/>
			<div class="truncate text-center">
				{editedSongs.get(data.id)?.title || data.title}
			</div>
		{/snippet}
	</GridList>
{/if}
