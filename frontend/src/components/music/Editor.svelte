<script lang="ts">
	import TextInput from "@components/TextInput.svelte";
	import Icon from "@components/Icon.svelte";
	import Cover from "./Cover.svelte";

	import { isSong, type Item } from "@lib/state/app.svelte";
	import type { Song } from "@lib/models";

	const excludedFields = ["title", "artist", "id", "path", "parentPath"];

	interface Props {
		selectedItem: Item | null;
		canEdit?: boolean;
	}

	function renameField(key: string) {
		return key
			.replace(/([A-Z])/g, " $1")
			.replace(/^./, (str) => str.toUpperCase());
	}

	let { selectedItem = $bindable() }: Props = $props();

	let imageHeight: number | null | undefined = $state();
	let imageWidth: number | null | undefined = $state();
	let failedToLoad = $state(false);

	function onCoverError() {
		failedToLoad = true;
	}

	function onCoverLoad() {
		failedToLoad = false;
	}

	function mapTracksToFields(songs: Song[]): Map<string, string> {
		if (!songs.length) return new Map();

		const map = new Map<string, string>();
		const first = songs.at(0);
		const rest = songs.slice(1);

		if (!rest.length) {
			for (const [key, value] of Object.entries(first as Song)) {
				if (!value || excludedFields.includes(key)) continue;
				map.set(key, value as string);
			}

			return map;
		}

		for (const track of rest) {
			for (const [key, value] of Object.entries(track)) {
				if (!value || excludedFields.includes(key)) continue;

				if (first && value === first[key as keyof Song]) {
					map.set(key, value);
				} else {
					map.set(key, `Different across (${songs.length}) tracks`);
				}
			}
		}

		return map;
	}
</script>

{#snippet suffixChild()}
	<Icon name="edit-3-line" />
{/snippet}

<div class={`space-y-2 relative h-full overflow-y-auto pt-6`}>
	{#if selectedItem}
		<div class="text-center text-sm">
			<Cover
				lazy={false}
				bind:imageHeight
				bind:imageWidth
				onError={onCoverError}
				onLoading={onCoverLoad}
				onLoad={onCoverLoad}
				item={selectedItem}
				class="mb-1 mx-auto rounded-theme shadow-lg shadow-black/25"
			/>

			{#if !imageHeight && !imageWidth && !failedToLoad}
				<div
					class="w-24 bg-base-950/25 text-transparent motion-safe:animate-pulse rounded-theme-lg mx-auto"
					aria-hidden="true"
				>
					x
				</div>
			{:else}
				<p
					aria-hidden={failedToLoad || (!imageHeight && !imageWidth)}
					aria-label={`Cover art size ${imageWidth} by ${imageHeight}.`}
					class={`duration-300 ease-in-out text-base-950/50 ${failedToLoad ? "invisible pointer-events-none" : ""}`}
				>
					{imageHeight} x {imageWidth}
				</p>
			{/if}
		</div>
		<div
			class="flex flex-col gap-2 mx-auto justify-center items-center md:w-3/5"
		>
			<TextInput
				variant="ghost"
				class="font-bold text-center text-2xl truncate w-full"
				placeholder={isSong(selectedItem)
					? "Title..."
					: "Album Title..."}
				value={isSong(selectedItem)
					? selectedItem.song.title
					: selectedItem.label}
				{suffixChild}
			></TextInput>
			<TextInput
				variant="ghost"
				class="text-center block w-full"
				placeholder={!Array.isArray(selectedItem)
					? "Artist..."
					: "Album Artist..."}
				value={isSong(selectedItem)
					? selectedItem.song.artist
					: selectedItem.songs[0].artist}
				{suffixChild}
			></TextInput>
		</div>
		<div class="space-y-2 mt-2 px-2 md:w-3/5 mx-auto">
			{#if isSong(selectedItem)}
				{#each Object.entries(selectedItem.song) as [key, value]}
					{#if value && !excludedFields.includes(key)}
						<TextInput
							class="w-full"
							label={renameField(key)}
							floatingLabel={true}
							{suffixChild}
							bind:value={selectedItem.song[key as keyof Song] as string}
						/>
					{/if}
				{/each}
			{:else}
				{#each mapTracksToFields(selectedItem.songs).entries() as [key, value]}
					<TextInput
						class="w-full"
						label={renameField(key)}
						floatingLabel={true}
						{suffixChild}
						{value}
					/>
				{/each}
			{/if}
		</div>
	{/if}
</div>
