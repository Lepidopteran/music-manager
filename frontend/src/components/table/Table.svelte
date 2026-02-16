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

	interface Props extends HTMLAttributes<HTMLTableElement> {
		columns: ColumnDef<D, V>[];
		data: D[];
		options?: Partial<TableOptions<D>>;
	}

	let { columns, data, class: className, options, ...rest }: Props = $props();
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

<table
	class={[
		className,
		"w-full border border-base-600/15 rounded-theme overflow-hidden border-separate border-spacing-0 table-fixed shadow-lg",
	]}
	{...rest}
>
	<colgroup>
		{#if tableOptions.enableRowSelection}
			<col style="width: 32px;" />
		{/if}
		{#each headerGroups as { headers }}
			{#each headers as { column: { getSize } }}
				<col style={`width: ${getSize()}px;`} />
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
							checked={getIsAllRowsSelected()}
							onchange={getToggleAllRowsSelectedHandler()}
						/>
					</th>
				{/if}

				{#each headers as { getContext, column: { columnDef: { header, meta } } }}
					<th
						class="p-cell first:rounded-tl-theme last:rounded-tr-theme"
						style:text-align={meta?.alignment}
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
							checked={row.getIsSelected()}
							onchange={row.getToggleSelectedHandler()}
						/>
					</td>
				{/if}
				{#each row.getVisibleCells() as { getContext, column: { columnDef: { cell, meta } } }}
					<td
						class="p-cell trunecate border-inherit border-t"
						style:text-align={meta?.alignment}
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
