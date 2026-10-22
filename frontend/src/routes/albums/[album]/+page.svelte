<script lang="ts">
	import type { DatabaseSong } from "#bindings/DatabaseSong";
	import GridList from "#components/GridList.svelte";
	import CoverView from "#components/music/CoverView.svelte";
	import type { PageProps } from "./$types";

	import PrevPage from "#components/navigation/PrevPage.svelte";
	import { app } from "#stores/app";

	const { params }: PageProps = $props();
	const { editedSongs, selectedSongs } = $derived(app);
	const { album } = $derived(params);

	const songIds = $derived(app.getAlbumSongIds(album));
	const songs: DatabaseSong[] = $derived(
		songIds
			?.map((id) => editedSongs.get(id) ?? app.songs.get(id)).filter((song) =>
				song !== undefined
			)
			?? [],
	);
</script>

{#if songs.length === 0}
	<div class="h-full flex items-center justify-center">
		<p class="text-sm text-current/50">Album has no songs...</p>
	</div>
{:else}
	<GridList
		class="p-4"
		data={songs.sort((a, b) => Number(a.trackNumber) - Number(b.trackNumber))}
		getKey={(song: DatabaseSong) => song.id}
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
