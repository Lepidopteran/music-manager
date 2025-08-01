<script lang="ts">
	import type { Action } from "svelte/action";
	import type { Album, Song } from "@lib/models";
	import AlbumCover from "@components/music/Cover.svelte";
	import Button from "@components/Button.svelte";
	import Icon from "@iconify/svelte";
	interface Props {
		albums?: Array<Album>;
		sortBy?: "name";
		onItemChange?: (item: Album | Song) => void;
		[key: string]: unknown;
	}

	let {
		albums = [],
		sortBy = "name",
		albumClick,
		onItemChange,
		...rest
	}: Props = $props();

	let sortedAlbums = $state(sortAlbums(albums, sortBy));
	let selectedItem: Album | Song | null = $state(null);

	function sortAlbums(albums: Array<Album>, mode: "name"): Array<Album> {
		if (mode === "name") {
			return [...albums].sort((a, b) => a.title.localeCompare(b.title));
		}

		return [...albums];
	}

	let selectedElement: HTMLElement | null = null;

	const selectElement: Action = (node) => {
		const toggle = () => {
			if (selectedElement) {
				selectedElement.dataset.selected = "false";
			}

			selectedElement = node;
			selectedElement.dataset.selected = "true";
		};

		$effect(() => {
			node.addEventListener("click", toggle);

			return () => {
				node.removeEventListener("click", toggle);
			};
		});
	};

	function selectItem(item: Album | Song) {
		selectedItem = item;

		if (onItemChange) {
			onItemChange(item);
		}
	}
</script>

{#if albums.length}
	<div class={`flex flex-col overflow-y-auto ${rest.class}`}>
		{#each sortedAlbums as album}
			<details>
				<summary
					use:selectElement
					onclick={() => selectItem(album)}
					class="cursor-pointer hover:bg-primary/5 select-none bg-base-100 px-2 py-1 data-[selected=true]:bg-primary/25"
				>
					{album.title}
				</summary>
				<ul>
					{#each album.tracks as track}
						<li
							class="pl-4 py-1 data-[selected=true]:bg-primary/25"
							role="treeitem"
							aria-selected={selectedItem === track}
							onclick={() => selectItem(track)}
							aria-label={`${track.title} by ${track.artist}`}
							use:selectElement
						>
							{track.title}
						</li>
					{/each}
				</ul>
			</details>
		{/each}
	</div>
{:else}
	<div class="h-full flex items-center justify-center">
		No albums to be displayed :(
	</div>
{/if}
