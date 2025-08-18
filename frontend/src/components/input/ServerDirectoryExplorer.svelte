<script lang="ts">
	import { getServerDirectoryFolders } from "@api/directory";
	import { uniqueId } from "@lib/utils/counter-id";
	import { untrack } from "svelte";

	import TextInput from "@components/TextInput.svelte";
	import Icon from "@iconify/svelte";

	interface Props {
		value: string;
		required?: boolean;
		label?: string;
	}

	const componentName = uniqueId("directory-explorer");

	let { label, required, value = $bindable("/") }: Props = $props();
	let activeIndex = $state(-1);

	let level = $derived.by(() => {
		const parts = value
			.split("/")
			.filter(
				(part, index, parts) =>
					index !== 0 && part.length > 0 && part !== parts[parts.length - 1],
			);

		return parts.length;
	});

	let folders: Promise<Array<string>> = $derived.by(() => {
		const dir = untrack(() => value);

		untrack(() => (activeIndex = level > 0 ? -1 : 0));
		if (level === 0) {
			return getServerDirectoryFolders("/");
		}

		if (!dir.endsWith("/")) {
			return getServerDirectoryFolders(dir.split("/").slice(0, -1).join("/"));
		}

		return getServerDirectoryFolders(dir);
	});

	async function handleDirectoryClick(directory: string) {
		const parts = value.split("/");
		value = parts.slice(0, -1).join("/") + `/${directory}/`;
	}

	async function handleBackClick() {
		const parts = value.split("/");
		const deep = parts[parts.length - 2].length > 1 ? 2 : 1;

		value = parts.slice(0, -deep).join("/") + "/";
	}

	async function onkeydown(event: KeyboardEvent) {
		if (event.key === "Enter") {
			event.preventDefault();
			document
				.getElementById(`${componentName}-option-${activeIndex}`)
				?.click();
		}

		if (event.key === "ArrowDown") {
			event.preventDefault();
			const folderList = await folders;

			activeIndex = Math.min(activeIndex + 1, folderList.length - 1);
			document
				.getElementById(`${componentName}-option-${activeIndex}`)
				?.scrollIntoView({
					block: "nearest",
				});
		}

		if (event.key === "ArrowUp") {
			event.preventDefault();

			activeIndex = Math.max(activeIndex - 1, level > 0 ? -1 : 0);
			document
				.getElementById(`${componentName}-option-${activeIndex}`)
				?.scrollIntoView({
					block: "nearest",
				});
		}
	}

	$inspect(folders, level, value);
</script>

<div class="space-y-2">
	<TextInput
		id={`${componentName}-input`}
		{label}
		{required}
		{onkeydown}
		aria-controls={`${componentName}-listbox`}
		bind:value
	/>
	<ul
		role="listbox"
		aria-labelledby={`${componentName}-input`}
		aria-activedescendant={`${componentName}-option-${activeIndex}`}
		tabindex="0"
		aria-orientation="vertical"
		aria-multiselectable="false"
		id={`${componentName}-listbox`}
		class="overflow-y-auto h-[200px] rounded-theme shadow-md shadow-black/25 border border-base-600/15 divide-y divide-base-600/15"
	>
		{#if level > 0}
			<li
				class="w-full inset-shadow-xs inset-shadow-base-950/25 text-left p-2 bg-base-200 aria-selected:bg-base-300/50 cursor-pointer"
				role="option"
				id={`${componentName}-option--1`}
				aria-selected={activeIndex === -1}
				aria-label="Go up one directory"
				onmouseover={() => (activeIndex = -1)}
				onfocus={() => (activeIndex = -1)}
				onclick={() => handleBackClick()}
				onkeydown={(event) => {
					if (event.key === "Enter") {
						event.preventDefault();
						document
							.getElementById(`${componentName}-option--1`)
							?.click();
					}
				}}
			>
				../
			</li>
		{/if}
		{#await folders}
			<li role="none">Loading...</li>
		{:then folders}
			{#each folders as folder, index}
				{@const directory = folder.split("/").pop()}
				<li
					role="option"
					id={`${componentName}-option-${index}`}
					aria-selected={index === activeIndex}
					class="w-full inset-shadow-xs inset-shadow-base-950/25 text-left p-2 bg-base-200 aria-selected:bg-base-300/50 cursor-pointer"
					onmouseover={() => (activeIndex = index)}
					onfocus={() => (activeIndex = index)}
					onclick={() => handleDirectoryClick(directory as string)}
					onkeydown={(event) => {
						if (event.key === "Enter") {
							event.preventDefault();
							document
								.getElementById(`${componentName}-option-${index}`)
								?.click();
						}
					}}
				>
					<Icon icon="mdi:folder" class="inline mr-1" inline={true} />
					{directory}
				</li>
			{/each}
		{:catch error}
			<li role="none" class="bg-error">Error: {error.body}</li>
		{/await}
	</ul>
</div>
