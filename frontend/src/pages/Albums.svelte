<script lang="ts">
	import GridList from "@components/GridList.svelte";
	import CoverView from "@components/music/CoverView.svelte";
	import Page from "@components/routing/Page.svelte";
	import {
		editedSongs,
		groupManager,
		routeManager,
		selectedSongs,
	} from "@state";
	import { SvelteSet } from "svelte/reactivity";

	const routeState = routeManager();
	const groupState = groupManager();
	const selected = selectedSongs();
	const edited = editedSongs();

	let page: ReturnType<typeof Page>;
	let selectedAlbums = new SvelteSet();
	let activeAlbum: string | null = $state(null);

	let gridList: ReturnType<typeof GridList>;

	if (!groupState.tracked.includes("album")) {
		groupState.track("album");
	}

	const albums = $derived(groupState.groups.get("album")!);

	$effect(() => {
		if (activeAlbum) {
			console.log(activeAlbum);
			routeState.goTo(`/albums/${activeAlbum}`);
		}
	});

	$effect(() => {
		selected.clear();
		for (const album of selectedAlbums) {
			for (const song of albums.get(album as string)!) {
				selected.add(song.id);
			}
		}
	});
</script>

<svelte:window />

<Page
	bind:this={page}
	path={"/albums"}
	name="Albums"
	icon="album_2"
	navigation
	displayEditor
>
	<div class="h-full">
		{#if groupState.groups.has("album") && groupState.groups.get("album")!.length() > 0}
			{#if activeAlbum}
				<div>
					<a href="../">
						Go back
					</a>
				</div>
				<GridList
					data={albums.get(activeAlbum)!.sort((a, b) =>
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
			{:else}
				<GridList
					bind:this={gridList}
					class="m-2 gap-2 overflow-y-auto h-full"
					selected={selectedAlbums}
					getKey={(([album]) => album)}
					data={albums.entries().sort((a, b) => a[0].localeCompare(b[0]))}
					onActivate={([album]) => {
						selectedAlbums.clear();

						activeAlbum = album;
					}}
					columnWidth={128}
				>
					{#snippet item({ data })}
						{@const [album, songs] = data}
						<CoverView
							src="/api/albums/{album}/cover-art/front.jpg"
							class="mb-1 rounded-theme shadow-lg shadow-shade/25 size-32 mx-auto"
						/>
						<div class="truncate text-center">
							{edited.get(songs[0].id)?.album || album}
						</div>
					{/snippet}
				</GridList>
			{/if}
		{:else if groupState.inProgress.includes("album")}
			<div class="h-full flex items-center justify-center">
				<p class="text-sm text-current/50">Loading albums...</p>
			</div>
		{:else}
			<div class="h-full flex items-center justify-center">
				<p class="text-sm text-current/50">No albums found...</p>
			</div>
		{/if}
	</div>
</Page>
