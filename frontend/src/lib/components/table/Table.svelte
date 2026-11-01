<script lang="ts" generics="D extends RowData, V">
	import { dragSelection } from "#attachments/drag";
	import Checkbox from "#components/Checkbox.svelte";

	import {
		type ColumnDef,
		columnSizingFeature,
		columnVisibilityFeature,
		createTable,
		FlexRender,
		type Row,
		type RowData,
		rowSelectionFeature,
		tableFeatures,
		type TableOptions,
	} from "@tanstack/svelte-table";

	import type { ClassValue } from "svelte/elements";

	interface ColumnMeta {
		alignment?: "left" | "center" | "right";
		truncate?: boolean | "start" | "end";
	}

	const features = tableFeatures({
		rowSelectionFeature,
		columnSizingFeature,
		columnVisibilityFeature,
		columnMeta: {} as ColumnMeta,
	});

	interface Props<D extends RowData> {
		columns: ColumnDef<typeof features, D>[];
		options?: Omit<
			TableOptions<typeof features, D>,
			| "data"
			| "columns"
			| "features"
		>;
		data: D[];
		class?: ClassValue;
	}

	let {
		columns,
		data,
		class: className,
		options,
		...rest
	}: Props<D> = $props();

	const table = $derived(createTable({
		features,
		columns,
		...options,
		get data() {
			return data;
		},
	}));

	const rowSelectionState = $derived(table.atoms.rowSelection.get());
	const hasRowSelection = $derived(rowSelectionState);
</script>

<table
	role="grid"
	class={[
		className,
		"w-full border border-base-content/15 rounded-theme overflow-hidden border-separate border-spacing-0 table-fixed shadow-lg",
	]}
	{...rest}
>
	<colgroup>
		{#if hasRowSelection}
			<col style="width: 32px" />
		{/if}
		{#each table.getHeaderGroups() as { headers }}
			{#each headers as header}
				<col style:width={header.getSize()} class="min-w-0" />
			{/each}
		{/each}
	</colgroup>

	<thead class="bg-base-300 border-inherit">
		{#each table.getHeaderGroups() as headerGroup (headerGroup.id)}
			<tr class="border-inherit text-sm shadow-md">
				{#if hasRowSelection}
					<th class="p-cell first:rounded-tl-theme last:rounded-tr-theme text-center">
						<Checkbox
							indeterminate={table.getIsSomeRowsSelected()}
							disabled={data.length === 0}
							checked={table.getIsAllRowsSelected()}
							onchange={table.getToggleAllRowsSelectedHandler()}
						/>
					</th>
				{/if}

				{#each headerGroup.headers as header}
					{@const meta = header.column.columnDef.meta}
					<th
						class={[
							"p-cell first:rounded-tl-theme last:rounded-tr-theme",
							meta?.truncate && "truncate",
						]}
						style:direction={meta?.truncate === "end" ? "rtl" : undefined}
						style:text-align={meta?.alignment ?? "left"}
					>
						<FlexRender {header} />
					</th>
				{/each}
			</tr>
		{/each}
	</thead>
	<tbody class="divide-y divide-primary/15 inset-shadow-xs inset-shadow-highlight/25">
		{#each table.getRowModel().rows as row (row.id)}
			<tr
				class={[
					"first:border-transparent",
					row.getIsSelected() && "bg-primary/10",
				]}
			>
				{#if hasRowSelection}
					{@const {
						id,
						getIsSelected,
						getToggleSelectedHandler,
						getIsSomeSelected,
						getCanSelect,
					} = row}
					<td
						id={id}
						class="p-cell w-min text-center"
						{@attach dragSelection((event) => {
							const checkbox = event.target instanceof Element
								? event.target.closest("input[type=checkbox]")
								: null;

							if (checkbox instanceof HTMLInputElement) {
								checkbox.click();
								return true;
							}

							return false;
						})}
					>
						<Checkbox
							variant={getIsSelected() ? "primary" : "base"}
							indeterminate={getIsSomeSelected()}
							disabled={!getCanSelect()}
							checked={getIsSelected()}
							onchange={getToggleSelectedHandler()}
						/>
					</td>
				{/if}
				{#each row.getVisibleCells() as cell (cell.id)}
					{@const meta = cell.column.columnDef.meta}
					<td
						class={[
							"p-cell truncate border-inherit border-t",
							meta?.truncate !== false && "truncate",
						]}
						style:direction={meta?.truncate === "start"
						? "rtl"
						: undefined}
						style:text-align={meta?.alignment ?? "left"}
					>
						<FlexRender {cell} />
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
