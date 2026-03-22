<script lang="ts">
	import Icon from "@components/Icon.svelte";
	import Image from "@components/Image.svelte";
	import TextInput from "@components/TextInput.svelte";

	import Stack from "@components/stack/Stack.svelte";
	import StackItem from "@components/stack/StackItem.svelte";
	import type { Song } from "@lib/models";
	import { editedSongs, selectedSongs, songs } from "@state";
	import { SvelteSet } from "svelte/reactivity";
	import MissingCover from "./MissingCover.svelte";

	const excludedFields: Array<keyof Song> = [
		"id",
		"path",
		"unknown",
		"fileCreatedAt",
		"directoryId",
		"updatedAt",
		"addedAt",
	];

	interface Props {
		canEdit?: boolean;
	}

	function renameField(key: string) {
		return key
			.replace(/([A-Z])/g, " $1")
			.replace(/^./, (str) => str.toUpperCase());
	}

	const all = songs();
	const edited = editedSongs();
	const selected = selectedSongs();

	const songsWithNoCover: Set<string> = new SvelteSet();
	const songsInSelection: Array<Song> = $derived(
		selected.values().map(id => all.get(id) as Song)
			.toArray(),
	);
</script>

<div class={["space-y-2 relative h-full overflow-y-auto pt-6"]}>
	{#if songsInSelection.length > 0}
		{@const first = songsInSelection[0] as Song}
		{@const firstEdited = edited.get(first.id)}
		{@const keys = [...new Set(songsInSelection.flatMap((song) => Object.keys(song)))]
		.sort() as Array<keyof Song>}

		<div class="text-center text-sm">
			{#if songsInSelection.length === 1}
				{#if songsWithNoCover.has(songsInSelection[0].id)}
					<MissingCover
						class="size-64 rounded-theme shadow-lg shadow-black/25 mx-auto mb-1"
					/>
				{:else}
					<Image
						src="/api/songs/{songsInSelection[0].id}/cover-art/front.jpg"
						class="mb-1 mx-auto rounded-theme shadow-lg shadow-black/25 size-64"
					/>
				{/if}
			{:else}
				<Stack
					class="drop-shadow-xl drop-shadow-black/25"
					style={`height: calc(auto + ${selected.size * 3})px`}
					offset="4px"
				>
					{#each songsInSelection.filter((song) => !songsWithNoCover.has(song.id)) as song, index (song.id)}
						<StackItem index={index}>
							<Image
								src="/api/songs/{song.id}/cover-art/front.jpg"
								class="mb-1 mx-auto rounded-theme size-64 object-contain object-top"
								onError={() => {
									songsWithNoCover.add(song.id);
								}}
							/>
						</StackItem>
					{/each}
				</Stack>
			{/if}
		</div>
		<div class="space-y-2 mt-2 px-2 md:w-3/5 mx-auto">
			{#each keys as key}
				{#if !excludedFields.includes(key)
	&& songsInSelection.some((song) =>
		song[key] !== null && song[key] !== undefined
	)}
					<label class="w-full block">
						<span class="block text-sm text-base-950/50">
							{renameField(key)}
						</span>
						<TextInput
							class="w-full"
							placeholder={songsInSelection.some((song) => !song[key] || song[key] !== first[key])
							? "Difference across selected songs"
							: undefined}
							bind:value={() => {
								return firstEdited !== undefined && firstEdited[key] !== first[key]
									? firstEdited[key]?.toString()
									: songsInSelection.every(
											(song) => song[key] === first[key],
										)
									? songsInSelection[0][key]?.toString()
									: "";
							}, (newValue) => {
								for (const song of songsInSelection) {
									edited.set(song.id, {
										...song,
										...edited.get(song.id),
										[key]: newValue,
									});
								}
							}}
						>
							{#snippet suffixChild({ focused })}
								<Icon
									name="pencil"
									class="w-8"
									hidden={focused ? true : undefined}
								/>
							{/snippet}
						</TextInput>
					</label>
				{/if}
			{/each}
		</div>
	{/if}
</div>
