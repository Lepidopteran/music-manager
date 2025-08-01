<script lang="ts">
	import Explorer from "@components/music/Explorer.svelte";
	import Details from "@components/music/Details.svelte";

	import type { Album, Song } from "@lib/models";
	import { getAlbums } from "@api/album";

	let selectedItem: Album | Song | null = $state(null);
</script>

{#await getAlbums()}
	<div
		class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4 overflow-y-auto"
	>
		{#each { length: 10 }}
			<div class="inline-flex flex-col gap-2 items-center">
				<div
					class="size-64 bg-primary/25 relative rounded-lg shadow-lg animate-pulse"
				></div>
				<div class="w-24 h-4 bg-neutral-50/25 animate-pulse rounded-lg"></div>
			</div>
		{/each}
	</div>
{:then albums}
	<div class="grid md:grid-cols-2 h-full">
		<Explorer
			{albums}
			onItemChange={(item) => (selectedItem = item)}
			class="h-full"
		/>
		<Details {selectedItem} />
	</div>
{/await}
