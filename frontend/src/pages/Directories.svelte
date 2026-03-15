<script lang="ts">
	import type { Directory } from "@lib/bindings/Directory";
	import type { NewDirectory } from "@lib/bindings/NewDirectory";

	import Button from "@components/Button.svelte";
	import Modal from "@components/Modal.svelte";
	import TextInput from "@components/TextInput.svelte";
	import { formatBytes } from "@utils/bytes";
	import { onMount } from "svelte";

	import {
		createDirectory,
		deleteDirectory,
		getDirectories,
	} from "@api/directory";
	import ServerDirectory from "@components/input/ServerDirectory.svelte";
	import Table from "@components/table/Table.svelte";

	import Icon from "@components/Icon.svelte";
	import { onMediumScreen } from "@utils/screen";

	let newDirectoryModalOpen = $state(false);
	let deleteDirectoryModalOpen = $state(false);

	let rowSelection: Record<string, boolean> = $state({});
	let selectedDirectories = $derived(
		Object.keys(rowSelection)
			.filter((key) => rowSelection[key])
			.map((key) => {
				return directories.find((directory) => directory.name === key);
			})
			.filter((directory) => directory !== undefined),
	);

	let directoriesToBeDeleted: Set<string> = $state(new Set());

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
		directories = [...directories, directory];
	}

	async function handleDeleteDirectory() {
		deleteDirectoryModalOpen = false;
		directoriesToBeDeleted = new Set(selectedDirectories.map((d) => d.name));

		for (const directory of selectedDirectories) {
			await deleteDirectory(directory.name);
			directories = directories.filter((d) => d.name !== directory.name);
			directoriesToBeDeleted.delete(directory.name);
		}
	}

	onMount(async () => {
		directories = await getDirectories();
	});

	$inspect(rowSelection);
	$inspect(directories);
</script>

<div class="flex flex-col gap-4 p-4">
	<h1 class="text-3xl font-bold">Directories</h1>
	<p>This is the list of directories used to scan and manage your music</p>
	<p>Go ahead and add some</p>
	<Table
		data={directories}
		class="select-none"
		options={{
			getRowId: (row) => row.name,
			enableRowSelection: (row) => !directoriesToBeDeleted.has(row.original.name),
			state: {
				rowSelection,
				columnVisibility: {
					totalSpace: onMediumScreen.current,
					freeSpace: onMediumScreen.current,
				},
			},
			onRowSelectionChange: (updater) => {
				rowSelection = typeof updater === "function"
					? updater(rowSelection)
					: updater;
			},
		}}
		columns={[
			{
				accessorKey: "path",
				header: "Path",
				enableHiding: false,
				meta: {
					truncate: "start",
					alignment: "left",
				},
			},
			{
				accessorKey: "displayName",
				header: "Display Name",
				enableHiding: false,
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
				enableHiding: true,
				header: "Total Space",
				cell: ({ getValue }) =>
					getValue() ? formatBytes(getValue() as number) : "-",
				meta: {
					alignment: "right",
				},
			},
		]}
	/>
	<div class="flex gap-4">
		<Button
			variant="primary"
			class="w-40"
			onclick={() => (newDirectoryModalOpen = true)}
		>
			<span aria-hidden="true">+</span>
			Add Directory
		</Button>
		<Button
			variant="error"
			class="w-30"
			onclick={() => (deleteDirectoryModalOpen = true)}
			disabled={!selectedDirectories.length}
		>
			<Icon name="delete" aria-hidden="true" />
			Remove
		</Button>
	</div>
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
	title={`Remove ${
		selectedDirectories.length > 1
			? `${selectedDirectories.length} directories`
			: `${selectedDirectories[0]?.displayName || "directory"}`
	}?`}
	bind:open={deleteDirectoryModalOpen}
>
	Are you sure you want to remove {
		selectedDirectories.length > 1
		? `${selectedDirectories.length} directories`
		: `${`"${selectedDirectories[0]?.displayName}"` || "directory"}`
	}?
	<br />
	This will not delete the directory from the server, it will just be removed
	from the list
	<div class="flex gap-2">
		<Button variant="error" class="font-bold" onclick={handleDeleteDirectory}
		>Delete</Button>
		<Button onclick={() => (deleteDirectoryModalOpen = false)} class="py-0">
			Cancel
		</Button>
	</div>
</Modal>
