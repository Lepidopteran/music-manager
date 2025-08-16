<script lang="ts">
	import type { Directory, NewDirectory } from "@lib/models";

	import Icon from "@iconify/svelte";
	import Button from "@components/Button.svelte";
	import Modal from "@components/Modal.svelte";
	import TextInput from "@components/TextInput.svelte";

	import { formatBytes } from "@utils/bytes";
	import { onMount } from "svelte";

	import {
		getDirectories,
		createDirectory,
		deleteDirectory,
	} from "@api/directory";
	import ServerDirectoryExplorer from "@components/input/ServerDirectoryExplorer.svelte";

	let newDirectoryModalOpen = $state(false);
	let deleteDirectoryModalOpen = $state(false);

	let selectedDirectory: Directory | undefined = $state();

	let newDirectory: NewDirectory = $state({
		path: "",
		name: "",
	});

	const directories: Array<Directory> = $state([]);

	async function handleNewDirectory() {
		const directory = await createDirectory(newDirectory);
		if (!directory) {
			return;
		}

		newDirectoryModalOpen = false;
		directories.push(directory);
	}

	async function handleDeleteDirectory() {
		if (!selectedDirectory) return;
		await deleteDirectory(selectedDirectory.name);
		directories.splice(directories.indexOf(selectedDirectory), 1);
		deleteDirectoryModalOpen = false;
		selectedDirectory = undefined;
	}

	onMount(async () => {
		directories.push(...(await getDirectories()));
	});
</script>

<div class="flex flex-col gap-4 p-4">
	<h1 class="text-3xl font-bold">Directories</h1>
	<p>This is the list of directories used to scan and manage your music</p>
	<p>Go ahead and add some</p>
	<table
		class="w-full border border-base-600/15 rounded-theme overflow-hidden border-separate border-spacing-0 table-fixed shadow-lg max-w-3xl"
	>
		<caption class="p-2 text-left text-lg">Directories</caption>

		<colgroup>
			<col />
			<col class="w-12 md:w-24" />
			<col class="w-0 collapse md:w-28 md:visible" />
			<col class="w-32" />
			<col class="w-0 collapse md:w-28 md:visible" />
		</colgroup>
		<thead class="bg-base-300 border-inherit">
			<tr
				class="border-inherit text-sm text-primary-800 *:first:rounded-tl-theme *:last:rounded-tr-theme shadow-md"
			>
				<th class="p-cell text-left">Location</th>
				<th class="p-cell text-left">Name</th>
				<th class="p-cell text-right">Path Size</th>
				<th class="p-cell text-right">Drive Usage</th>
				<th class="p-cell text-right">Drive Total</th>
			</tr>
		</thead>
		<tbody
			class="divide-y divide-primary-600/15 border-inherit inset-shadow-xs inset-shadow-highlight/25"
		>
			{#snippet row(
				path: string,
				name: string,
				pathSize: number,
				freeSpace: number,
				totalSpace: number,
			)}
				<tr class="border-inherit *:overflow-hidden">
					<td class="p-cell truncate">{path}</td>
					<td class="p-cell">{name}</td>
					<td class="p-cell text-right">{formatBytes(pathSize)}</td>
					<td class="p-cell text-right">{formatBytes(freeSpace)}</td>
					<td class="p-cell text-right">{formatBytes(totalSpace)}</td>
				</tr>
			{/snippet}
			{#each directories as directory}
				{@render row(
					directory.path,
					directory.name,
					directory.pathSize,
					directory.freeSpace,
					directory.totalSpace,
				)}
			{/each}
		</tbody>
	</table>

	<Button
		color="primary"
		class="font-bold w-40"
		onclick={() => (newDirectoryModalOpen = true)}>Add Directory</Button
	>
</div>

<Modal title="Add Directory" bind:open={newDirectoryModalOpen} class="w-1/3">
	<div class="flex flex-col gap-4">
		<TextInput
			label="Name"
			required
			placeholder="Name"
			bind:value={newDirectory.name}
		/>
		<ServerDirectoryExplorer label="Location" required bind:value={newDirectory.path} />
		<Button color="primary" onclick={handleNewDirectory}>Add</Button>
	</div>
</Modal>
<Modal
	title={`Remove ${selectedDirectory?.name}?`}
	bind:open={deleteDirectoryModalOpen}
	class="w-1/3"
>
	Are you sure you want to remove this directory?
	<br />
	This will not delete the directory, it will just be removed from the list
	<div class="flex gap-2">
		<Button color="primary" onclick={handleDeleteDirectory}>Delete</Button>
		<Button onclick={() => (deleteDirectoryModalOpen = false)} class="py-0">
			Cancel
		</Button>
	</div>
</Modal>

<style>
	th,
	td {
		white-space: nowrap;
		overflow: hidden;
	}
</style>
