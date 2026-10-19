<script lang="ts">
	import GridList from "#components/GridList.svelte";
	import CoverView from "#components/music/CoverView.svelte";
	import PrevPage from "#components/navigation/PrevPage.svelte";
	import { app } from "#lib/state/app.svelte.ts";
	import { songGroups } from "#lib/state/group.svelte.ts";
	import { goto } from "$app/navigation";
	import { onMount } from "svelte";
	import { SvelteSet } from "svelte/reactivity";

	const { selectedSongs, editedSongs } = $derived(app);

	let selectedAlbums = new SvelteSet();

	let gridList: ReturnType<typeof GridList>;

	const albums = $derived(songGroups.groups.get("album"));

	$effect(() => {
		selectedSongs.clear();
		for (const album of selectedAlbums) {
			for (const song of albums!.get(album as string)!) {
				selectedSongs.add(song.id);
			}
		}
	});

	onMount(() => {
		if (!songGroups.tracked.includes("album")) {
			songGroups.track("album");
		}
	});
</script>

<svelte:window />

{#if songGroups.groups.has("album") && songGroups.groups.get("album")!.length() > 0}
	<GridList
		bind:this={gridList}
		class={"m-2 gap-2 overflow-y-auto h-full"}
		selected={selectedAlbums}
		getKey={(([album]) => album)}
		data={albums!.entries().sort((a, b) => a[0].localeCompare(b[0]))}
		onActivate={([album]) => {
			selectedAlbums.clear();
			goto(`/albums/${album}`);
		}}
		columnWidth={128}
	>
		{#snippet item({ data })}
			{@const [album, songs] = data}
			<CoverView
				src="/api/albums/{encodeURIComponent(album)}/cover-art/front.jpg"
				class="mb-1 rounded-theme shadow-lg shadow-shade/25 size-32 mx-auto"
			/>
			<div class="truncate text-center">
				{editedSongs.get(songs[0].id)?.album || album}
			</div>
		{/snippet}
	</GridList>
{:else if songGroups.inProgress.includes("album")}
	<div class="h-full flex items-center justify-center">
		<p class="text-sm text-current/50">Loading albums...</p>
	</div>
{:else}
	<div class="h-full flex items-center justify-center">
		<p class="text-sm text-current/50">No albums found...</p>
	</div>
{/if}
