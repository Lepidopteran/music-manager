<script lang="ts" generics="D, V">
	import {
		getCoreRowModel,
		type ColumnDef,
		type TableOptions,
	} from "@tanstack/table-core";

	import type { HTMLAttributes } from "svelte/elements";
	import { createTable } from "./table.svelte";
	import CellContent from "./CellContent.svelte";
	import Checkbox from "@components/Checkbox.svelte";
	import { SvelteSet } from "svelte/reactivity";

	interface Props extends HTMLAttributes<HTMLTableElement> {
		columns: ColumnDef<D, V>[];
		data: D[];
		options?: Partial<TableOptions<D>>;
		/**
		 * Allows the ability to click and drag to select multiple rows.
		 *
		 * Requires selection to be enabled.
		 */
		scrubSelection?: boolean;
	}

	let isScrubbing = $state(false);
	let ignoredScrubRows: SvelteSet<string> = $state(new SvelteSet());

	function endScrubbing() {
		if (!isScrubbing) {
			return;
		}

		isScrubbing = false;
		ignoredScrubRows.clear();
	}

	let {
		columns,
		data,
		class: className,
		options,
		scrubSelection = true,
		...rest
	}: Props = $props();

	const tableOptions = $derived({
		data,
		columns,
		getCoreRowModel: getCoreRowModel(),
		...options,
	});

	const {
		getHeaderGroups,
		getRowModel,
		getIsAllRowsSelected,
		getIsSomeRowsSelected,
		getToggleAllRowsSelectedHandler,
	} = $derived(createTable(tableOptions));
	const { rows } = $derived(getRowModel());
	const headerGroups = $derived(getHeaderGroups());
</script>

<svelte:window
	onpointercancel={() => endScrubbing()}
	onpointerup={() => endScrubbing()}
/>

<table
	class={[
		className,
		"w-full border border-base-600/15 rounded-theme overflow-hidden border-separate border-spacing-0 table-fixed shadow-lg",
	]}
	onpointerup={() => endScrubbing()}
	onpointerdown={(event) => {
		if (!scrubSelection || event.button !== 0) {
			return;
		}

		isScrubbing = true;
	}}
	{...rest}
>
	<colgroup>
		{#if tableOptions.enableRowSelection}
			<col style="width: 32px;" />
		{/if}
		{#each headerGroups as { headers }}
			{#each headers as { column: { getSize } }}
				<col style:width={getSize()} class="min-w-0" />
			{/each}
		{/each}
	</colgroup>

	<thead class="bg-base-300 border-inherit">
		{#each headerGroups as { headers }}
			<tr class="border-inherit text-sm text-primary-800 shadow-md">
				{#if tableOptions.enableRowSelection}
					<th
						class="p-cell first:rounded-tl-theme last:rounded-tr-theme text-center"
					>
						<Checkbox
							indeterminate={getIsSomeRowsSelected()}
							disabled={!data.length}
							checked={getIsAllRowsSelected()}
							onchange={getToggleAllRowsSelectedHandler()}
						/>
					</th>
				{/if}

				{#each headers as { getContext, column: { columnDef: { header, meta } } }}
					<th
						class={[
							"p-cell first:rounded-tl-theme last:rounded-tr-theme",
							meta?.truncate !== false && "truncate",
						]}
						style:direction={meta?.truncate === "end" ? "rtl" : undefined}
						style:text-align={meta?.alignment || "left"}
					>
						<CellContent
							content={{
								kind: "header",
								context: getContext(),
								value: header,
							}}
						/>
					</th>
				{/each}
			</tr>
		{/each}
	</thead>
	<tbody
		class="divide-y divide-primary-600/15 border-inherit inset-shadow-xs inset-shadow-highlight/25"
	>
		{#each rows as row}
			<tr
				class={[
					"border-inherit first:border-transparent",
					row.getCanExpand() && "cursor-pointer",
					row.getIsSelected() && "bg-primary-500/10",
				]}
			>
				{#if tableOptions.enableRowSelection}
					<td class="p-cell w-min text-center">
						<Checkbox
							variant={row.getIsSelected() ? "primary" : "base"}
							indeterminate={row.getIsSomeSelected()}
							disabled={!row.getCanSelect()}
							checked={row.getIsSelected()}
							onchange={row.getToggleSelectedHandler()}
							onpointermove={() => {
								if (!isScrubbing || ignoredScrubRows.has(row.id)) {
									return;
								}

								ignoredScrubRows.add(row.id);
								row.getToggleSelectedHandler()(new Event("change"));
							}}
						/>
					</td>
				{/if}
				{#each row.getVisibleCells() as { getContext, column: { columnDef: { cell, meta } } }}
					<td
						class={[
							"p-cell truncate border-inherit border-t",
							meta?.truncate !== false && "truncate",
						]}
						style:direction={meta?.truncate === "start" ? "rtl" : undefined}
						style:text-align={meta?.alignment || "left"}
					>
						<CellContent
							content={{ kind: "cell", context: getContext(), value: cell }}
						/>
					</td>
				{/each}
			</tr>
		{/each}
	</tbody>
</table>

<style>
	th,
	td {
		white-space: nowrap;
		overflow: hidden;
	}
</style>
