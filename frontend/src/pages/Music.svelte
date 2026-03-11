<script lang="ts">
	import { editedSongs, selectedSongs, songGroups } from "@lib/state";

	const groups = songGroups();
	const selected = selectedSongs();
	const edited = editedSongs();

	if (!groups.tracked.includes("album")) {
		groups.track("album");
	}
</script>

<div class="flex flex-col overflow-y-auto h-full">
	{#if groups.album && groups.album?.length() > 0}
		{@const albums = groups.album}
		{#each albums.entries()
			.sort((
				[groupA],
				[groupB],
			) => groupA.localeCompare(groupB)) as [group, songs]}
			<details>
				<summary
					class={[
						"cursor-pointer hover:bg-primary/5 select-none bg-base-100 px-2 py-1",
						songs.every((song) => selected.has(song.id)) && "bg-primary/25"
						|| songs.every((song) =>
								edited.has(song.id) && edited.get(song.id)?.album !== group
							) && "bg-error/25"
						|| songs.some((track) => edited.has(track.id)) && "bg-yellow-500/25",
					]}
					aria-label={group}
					onclick={() => {
						selected.clear();
						for (const song of songs) {
							selected.add(song.id);
						}
					}}
				>
					{#if songs.every((song) => edited.has(song.id))}
						{edited.get(songs[0].id)?.album}
					{:else}
						{group}
					{/if}
				</summary>
				<ul>
					{#each songs as song}
						{@const editedSong = edited.get(song.id)}
						<li
							class={[
								"pl-4 py-1 select-none cursor-pointer hover:bg-primary/5",
								selected.has(song.id) && "bg-primary/25"
								|| editedSong && "bg-yellow-500/25",
							]}
							aria-label={`${song.title} by ${song.artist}`}
						>
							<button
								class="size-full text-left cursor-pointer"
								onclick={() => {
									selected.clear();
									selected.add(song.id);
								}}
							>
								{
									edited.get(song.id)?.title
									|| song.title
									|| song.path
								}
							</button>
						</li>
					{/each}
				</ul>
			</details>
		{/each}
	{:else if groups.inProgress.includes("album")}
		<div class="p-2">Organizing albums...</div>
	{:else}
		<div class="p-2">No albums found</div>
	{/if}
</div>
