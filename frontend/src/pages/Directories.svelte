<script lang="ts">
	import type { NewDirectory } from "@bindings/NewDirectory";
	import type { Directory } from "@bindings/Directory";

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
	import type { AppState, PageComponentProps } from "@lib/state/app.svelte";

	let newDirectoryModalOpen = $state(false);
	let deleteDirectoryModalOpen = $state(false);

	let selectedDirectory: Directory | undefined = $state();

	let newDirectory: NewDirectory = $state({
		displayName: null,
		path: "",
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

	let props: PageComponentProps = $props();
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
				name: string | null,
				pathSize: bigint | null,
				freeSpace: bigint | null,
				totalSpace: bigint | null,
			)}
				<tr class="border-inherit *:overflow-hidden">
					<td class="p-cell truncate">{path}</td>
					<td class="p-cell">{name}</td>
					<td class="p-cell text-right"
						>{pathSize ? formatBytes(pathSize) : "-"}</td
					>
					<td class="p-cell text-right"
						>{freeSpace ? formatBytes(freeSpace) : "-"}</td
					>
					<td class="p-cell text-right"
						>{totalSpace ? formatBytes(totalSpace) : "-"}</td
					>
				</tr>
			{/snippet}
			{#each directories as directory}
				{@render row(
					directory.path,
					directory.displayName,
					directory.pathSize,
					directory.freeSpace,
					directory.totalSpace,
				)}
			{/each}
		</tbody>
	</table>

	<Button
		variant="primary"
		class="font-bold w-40"
		onclick={() => (newDirectoryModalOpen = true)}>Add Directory</Button
	>
</div>

<Modal title="Add Directory" bind:open={newDirectoryModalOpen}>
	<div class="flex flex-col gap-4">
		<label class="block">
			<span class="block text-sm text-base-950/75">Display Name</span>
			<TextInput
				placeholder="Music..."
				bind:value={newDirectory.displayName}
			/>
		</label>
		<ServerDirectoryExplorer
			required
			bind:value={newDirectory.path}
		/>
		<Button variant="primary" onclick={handleNewDirectory}>Add</Button>
	</div>
</Modal>
<Modal
	title={`Remove ${selectedDirectory?.name}?`}
	bind:open={deleteDirectoryModalOpen}
>
	Are you sure you want to remove this directory?
	<br />
	This will not delete the directory, it will just be removed from the list
	<div class="flex gap-2">
		<Button variant="primary" onclick={handleDeleteDirectory}>Delete</Button>
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
