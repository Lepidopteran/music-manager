<script lang="ts">
	import { legacyAppState, songGroups } from "@lib/context";
	import type { Song } from "@lib/models";
	import { isGroup, isSong } from "@state/app.svelte";

	const app = legacyAppState();
	const groups = songGroups();
	function isSelectedItem(item: string | Song) {
		if (!app.selectedItem) {
			return false;
		}

		if (isGroup(app.selectedItem) && typeof item === "string") {
			return app.selectedItem.label === item;
		} else if (isSong(app.selectedItem) && typeof item === "object") {
			return app.selectedItem.song.id === item.id;
		}

		return false;
	}
</script>

<div class="flex flex-col overflow-y-auto h-full">
	{#if groups.album && groups.album?.length() > 0}
		{@const albums = groups.album}
		{#each albums as [group, tracks]}
			<details>
				<summary
					class="cursor-pointer hover:bg-primary/5 select-none bg-base-100 px-2 py-1 data-[edited=true]:bg-yellow-500/25 data-[selected=true]:bg-primary/25"
					aria-label={group}
					data-selected={isSelectedItem(group)}
					data-edited={tracks.some((track) => app.editedTracks.has(track.id))}
					onclick={() => (app.selectedItem = {
						type: "group",
						label: group,
						songs: tracks.map(
							(song) => app.editedTracks.get(song.id) || song,
						),
					})}
				>
					{group}
				</summary>
				<ul>
					{#each tracks as track}
						{@const trackIdString = track.id}
						<li
							class="pl-4 py-1 data-[selected=true]:bg-primary/25 data-[edited=true]:bg-yellow-500/10 select-none cursor-pointer hover:bg-primary/5"
							aria-label={`${track.title} by ${track.artist}`}
							data-selected={isSelectedItem(track)}
							data-edited={app.editedTracks.has(track.id)}
							onclick={() => (app.selectedItem = {
								type: "song",
								song: app.editedTracks.get(trackIdString) || track,
							})}
						>
							{
								app.editedTracks.get(trackIdString)?.title
								|| track.title
								|| track.path
							}
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
