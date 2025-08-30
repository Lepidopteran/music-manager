<script lang="ts">
	import type { Action } from "svelte/action";
	import type { Song } from "@lib/models";
	import { SvelteMap } from "svelte/reactivity";
	import type { ClassValue } from "svelte/elements";
	type Album = SvelteMap<string, Song[]>;

	// TODO: Add sorting
	// TODO: Add search
	// TODO: Add filtering

	interface Props {
		items?: Album | null;
		sortBy?: "name";
		onItemChange?: (item: [string, Song[]] | Song) => void;
		class?: ClassValue;
		[key: string]: unknown;
	}

	let {
		items = null,
		sortBy = "name",
		albumClick,
		class: className,
		onItemChange,
		...rest
	}: Props = $props();

	let sortedItems = $derived.by(() => {
		if (items instanceof SvelteMap) {
			return new Map([...items.entries()].sort((a, b) => a[0].localeCompare(b[0])));
		}
	});

	let selectedItem: Song | [string, Song[]] | null = $state(null);
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

	function selectItem(item: Song | [string, Song[]]) {
		selectedItem = item;

		if (onItemChange) {
			onItemChange(item);
		}
	}
</script>

{#if sortedItems && sortedItems.size > 0}
	<div class={`flex flex-col overflow-y-auto ${className}`}>
		{#each sortedItems as [group, tracks]}
			<details>
				<summary
					use:selectElement
					onclick={() => selectItem([group, sortedItems.get(group) as Song[]])}
					class="cursor-pointer hover:bg-primary/5 select-none bg-base-100 px-2 py-1 data-[selected=true]:bg-primary/25"
				>
					{group}
				</summary>
				<ul>
					{#each tracks as track}
						<li
							class="pl-4 py-1 data-[selected=true]:bg-primary/25"
							role="treeitem"
							onclick={() => selectItem(track)}
							aria-label={`${track.title} by ${track.artist}`}
							use:selectElement
						>
							{track.title || track}
						</li>
					{/each}
				</ul>
			</details>
		{/each}
	</div>
{:else}
	<div class="h-full flex items-center justify-center">
		No items to be displayed :(
	</div>
{/if}
