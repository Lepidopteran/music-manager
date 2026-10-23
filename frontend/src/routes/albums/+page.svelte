<script lang="ts">
	import GridList from "#components/GridList.svelte";
	import CoverView from "#components/music/CoverView.svelte";
	import { app } from "#lib/stores/app.svelte.ts";
	import { goto } from "$app/navigation";
	import { SvelteSet } from "svelte/reactivity";

	let selectedAlbums = $state(new SvelteSet<string>());
	let gridList = $state<ReturnType<typeof GridList>>(null as any);
	const { selectedSongs } = $derived(app);

	$effect(() => {
		selectedSongs.clear();
		for (const albumId of selectedAlbums) {
			const ids = app.getAlbumSongIds(albumId);

			if (ids) {
				for (const id of ids) {
					selectedSongs.add(id);
				}
			}
		}
	});

	$inspect(selectedAlbums);
</script>

<svelte:window />

{#if app.albums.size > 0}
	<GridList
		bind:this={gridList}
		class={"m-2 gap-2 overflow-y-auto h-full"}
		selected={selectedAlbums}
		getKey={([id]) => id}
		data={Array.from(app.albums)}
		onActivate={([albumId]) => {
			selectedAlbums.clear();
			goto(`/albums/${albumId}`);
		}}
		multiSelect
		columnWidth={128}
	>
		{#snippet item({ data })}
			{@const [_, { title }] = data}
			<CoverView
				src="/api/albums/{encodeURIComponent(title)}/cover-art/front.jpg"
				class="mb-1 rounded-theme shadow-lg shadow-shade/25 size-32 mx-auto"
			/>
			<div class="truncate text-center">
				{title}
			</div>
		{/snippet}
	</GridList>
{:else}
	<div class="h-full flex items-center justify-center">
		<p class="text-sm text-current/50">No albums found...</p>
	</div>
{/if}
