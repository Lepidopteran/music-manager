<script lang="ts" generics="D, V">
	import { getCoreRowModel, type ColumnDef } from "@tanstack/table-core";

	import type { HTMLAttributes } from "svelte/elements";
	import { createTable } from "./table.svelte";
	import CellContent from "./CellContent.svelte";

	interface Props extends HTMLAttributes<HTMLTableElement> {
		columns: ColumnDef<D, V>[];
		data: D[];
	}

	let { columns, data, class: className, ...rest }: Props = $props();
	const { getHeaderGroups, getRowModel } = $derived(
		createTable({
			data,
			columns,
			getCoreRowModel: getCoreRowModel(),
			debugAll: true,
		}),
	);

	const { rows } = $derived(getRowModel());
	const headerGroups = $derived(getHeaderGroups());
</script>

<table
	class={[
		className,
		"w-full border border-base-600/15 rounded-theme overflow-hidden border-separate border-spacing-0 table-fixed shadow-lg max-w-3xl",
	]}
	{...rest}
>
	<colgroup>
		{#each headerGroups as { headers }}
			{#each headers as { column: { getSize } }}
				<col style={`width: ${getSize()}px;`} />
			{/each}
		{/each}
	</colgroup>

	<thead class="bg-base-300 border-inherit">
		{#each headerGroups as { headers }}
			<tr
				class="border-inherit text-sm text-primary-800 *:first:rounded-tl-theme *:last:rounded-tr-theme shadow-md"
			>
				{#each headers as { getContext, column: { columnDef: { header, meta } } }}
					<th class="p-cell" style:text-align={meta?.alignment}>
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
			<tr class="border-inherit">
				{#each row.getVisibleCells() as { getContext, column: { columnDef: { cell, meta } } }}
					<td class="p-cell trunecate" style:text-align={meta?.alignment}>
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
