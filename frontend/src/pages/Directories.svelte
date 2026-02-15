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
	import ServerDirectoryExplorer from "@components/input/ServerDirectory.svelte";
	import type { AppState, PageComponentProps } from "@lib/state/app.svelte";
	import ServerDirectory from "@components/input/ServerDirectory.svelte";
	import Table from "@components/table/Table.svelte";

	import type { ContentValueReturnType } from "@components/table/table.svelte";

	let newDirectoryModalOpen = $state(false);
	let deleteDirectoryModalOpen = $state(false);

	let selectedDirectory: Directory | undefined = $state();

	let newDirectory: NewDirectory = $state({
		displayName: null,
		path: "",
	});

	let directories: Array<Directory> = $state([]);

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
		directories = await getDirectories();
	});

	let props: PageComponentProps = $props();
</script>

<div class="flex flex-col gap-4 p-4">
	<h1 class="text-3xl font-bold">Directories</h1>
	<p>This is the list of directories used to scan and manage your music</p>
	<p>Go ahead and add some</p>
	<Table
		data={directories}
		columns={[
			{
				accessorKey: "path",
				header: "Path",
				meta: {
					alignment: "left",
				},
			},
			{
				accessorKey: "displayName",
				header: "Display Name",
				meta: {
					alignment: "left",
				},
			},
			{
				accessorKey: "pathSize",
				header: "Size",
				cell: ({ getValue }) =>
					getValue() ? formatBytes(getValue() as number) : "-",
				meta: {
					alignment: "right",
				},
			},
			{
				accessorKey: "freeSpace",
				header: "Free Space",
				cell: ({ getValue }) =>
					getValue() ? formatBytes(getValue() as number) : "-",
				meta: {
					alignment: "right",
				},
			},
			{
				accessorKey: "totalSpace",
				header: "Total Space",
				cell: ({ getValue }) =>
					getValue() ? formatBytes(getValue() as number) : "-",
				meta: {
					alignment: "right",
				},
			},
		]}
	/>
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
			<TextInput placeholder="Music..." bind:value={newDirectory.displayName} />
		</label>
		<label class="block">
			<span class="block text-sm text-base-950/75">Path</span>
			<ServerDirectory bind:value={newDirectory.path} />
		</label>
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
