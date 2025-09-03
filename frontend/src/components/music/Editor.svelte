<script lang="ts">
	import TextInput from "@components/TextInput.svelte";
	import Icon from "@components/Icon.svelte";
	import Cover from "./Cover.svelte";

	import { isGroup, isSong, type Item } from "@lib/state/app.svelte";
	import type { Song } from "@lib/models";

	const excludedFields: Array<keyof Song> = [
		"title",
		"artist",
		"id",
		"path",
		"unknown",
	];

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

	$inspect(selectedItem).with((type, value) => {
		if (type === "update" && value && isGroup(value)) {
			console.table(
				value.songs.map((song) => ({
					...song,
				})),
			);
		}
	});
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
				placeholder={isSong(selectedItem) ? "Title..." : "Album Title..."}
				aria-label={isSong(selectedItem) ? "Song title" : "Album title"}
				bind:value={
					() =>
						isSong(selectedItem)
							? selectedItem.song.title
							: selectedItem.songs.at(0)?.album,
					(value) =>
						isGroup(selectedItem)
							? (selectedItem.songs = selectedItem.songs.map((song) => ({
									...song,
									album: value,
								})))
							: (selectedItem.song.title = value)
				}
				{suffixChild}
			></TextInput>
			<TextInput
				variant="ghost"
				class="text-center block w-full"
				placeholder={isSong(selectedItem) ? "Artist..." : "Album Artist..."}
				aria-label={isSong(selectedItem) ? "Song artist" : "Album artist"}
				bind:value={
					() =>
						isSong(selectedItem)
							? selectedItem.song.artist
							: selectedItem.songs[0].albumArtist,
					(value) =>
						isGroup(selectedItem)
							? (selectedItem.songs = selectedItem.songs.map((song) => ({
									...song,
									albumArtist: value,
								})))
							: (selectedItem.song.artist = value)
				}
				{suffixChild}
			></TextInput>
		</div>
		<div class="space-y-2 mt-2 px-2 md:w-3/5 mx-auto">
			{#if isSong(selectedItem)}
				{#each Object.entries(selectedItem.song) as [key, value]}
					{#if value && !excludedFields.includes(key as keyof Song)}
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
				{@const keys = [
					...new Set(selectedItem.songs.flatMap((song) => Object.keys(song))),
				].sort() as Array<keyof Song>}

				{#each keys as key}
					{#if !excludedFields.includes(key) && selectedItem.songs.every((song) => song[key] !== null && song[key] !== undefined)}
						<TextInput
							class="w-full"
							label={renameField(key)}
							floatingLabel={true}
							{suffixChild}
							bind:value={
								() =>
									selectedItem.songs.every(
										(song) =>
											song[key] === selectedItem.songs[0][key],
									)
										? selectedItem.songs[0][key]?.toString()
										: `Different across (${selectedItem.songs.length}) tracks`,
								(newValue) =>
									(selectedItem.songs = selectedItem.songs.map((song) => ({
										...song,
										[key]: newValue,
									})))
							}
						/>
					{/if}
				{/each}
			{/if}
		</div>
	{/if}
</div>
