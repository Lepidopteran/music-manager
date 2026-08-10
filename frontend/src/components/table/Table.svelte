<script lang="ts" generics="D, V">
	import {
		type ColumnDef,
		getCoreRowModel,
		type RowSelectionOptions,
		type TableOptions,
	} from "@tanstack/table-core";

	import Checkbox from "@components/Checkbox.svelte";
	import { watch } from "@utils/reactivity/watch.svelte";
	import type { ClassValue } from "svelte/elements";
	import { SvelteSet } from "svelte/reactivity";
	import { match } from "ts-pattern";
	import CellContent from "./CellContent.svelte";
	import { createTable } from "./table.svelte";

	let xIndex: number = $state(0);
	let yIndex: number = $state(0);

	interface SelectionOptions extends RowSelectionOptions<D> {
		/**
		 * Allows the ability to click and drag to select multiple rows.
		 *
		 * Requires selection to be enabled.
		 */
		enableScrubSelection?: boolean;
	}

	interface Props {
		columns: ColumnDef<D, V>[];
		data: D[];
		class?: ClassValue;
		rowSelection?: boolean | SelectionOptions;
		options?: Partial<TableOptions<D>>;
	}

	let isScrubbing = $state(false);
	let idleRows: SvelteSet<string> = $state(new SvelteSet());

	let {
		columns,
		data,
		class: className,
		options,
		rowSelection = false,
		...rest
	}: Props = $props();

	const { enableScrubSelection, ...selectionOptions } = $derived(
		{
			enableScrubSelection: true,
			enableRowSelection: true,
			...typeof rowSelection === "boolean"
				? { enableRowSelection: rowSelection }
				: rowSelection,
		},
	);

	const tableOptions = $derived({
		data,
		columns,
		getCoreRowModel: getCoreRowModel(),
		...selectionOptions,
		...options,
	});

	function startScrubbing() {
		if (isScrubbing || !enableScrubSelection) {
			return;
		}

		isScrubbing = true;
	}

	function endScrubbing() {
		if (!isScrubbing) {
			return;
		}

		isScrubbing = false;
		idleRows.clear();
	}

	function setYIndex(rowIndex: number) {
		yIndex = rowIndex;

		focusCell(rowIndex, xIndex);
	}

	function setXIndex(colIndex: number) {
		xIndex = colIndex;

		focusCell(yIndex, colIndex);
	}

	function focusCell(rowIndex: number, colIndex: number) {
		const row = tableRef.querySelector(
			`[aria-rowindex="${rowIndex + 1}"]`,
		) as HTMLElement;

		const cell = row.querySelector(
			`[aria-colindex="${colIndex + 1}"]`,
		) as HTMLElement;

		cell.focus();
	}

	let tableRef: HTMLTableElement;
	let annotationText: string = $state("");

	const {
		getHeaderGroups,
		getRowModel,
		getIsAllRowsSelected,
		getIsSomeRowsSelected,
		getToggleAllRowsSelectedHandler,
	} = $derived(createTable(tableOptions));
	const { rows } = $derived(getRowModel());
	const headerGroups = $derived(getHeaderGroups());

	$inspect(xIndex, yIndex);

	function toggleRowSelection(rowIndex: number) {
		if (rowIndex > -1) {
			const row = rows.at(rowIndex);

			if (row && !idleRows.has(row.id)) {
				row.getToggleSelectedHandler()(new Event("change"));

				if (isScrubbing) {
					idleRows.add(row.id);
				}
			}
		}
	}

	watch(() => yIndex, (rowIndex, prevRowIndex) => {
		if (rowIndex === prevRowIndex || rowIndex < 1) {
			return;
		}

		const row = rows.at(rowIndex);

		if (row) {
			annotationText = row.getIsSelected()
				? "Press Space to deselect row"
				: "Press Space to select row";
		} else {
			annotationText = "";
		}
	});
</script>

<svelte:window
	onpointercancel={() => endScrubbing()}
	onpointerup={() => endScrubbing()}
/>

<div
	class="sr-only"
	aria-live="polite"
	aria-atomic="true"
	aria-relevant="text additions"
>
	{annotationText}
</div>

<table
	role="grid"
	bind:this={tableRef}
	aria-rowcount={rows.length}
	class={[
		className,
		"w-full border border-base-content/15 rounded-theme overflow-hidden border-separate border-spacing-0 table-fixed shadow-lg",
	]}
	aria-multiselectable={selectionOptions.enableRowSelection
	&& selectionOptions.enableMultiRowSelection !== false}
	onpointerup={() => endScrubbing()}
	onkeyup={(event) => {
		match(event)
			.with({ key: " " }, () => {
				endScrubbing();
			});
	}}
	onkeydown={(event) => {
		match(event)
			.with({ key: "ArrowLeft" }, () => setXIndex(Math.max(0, xIndex - 1)))
			.with({ key: "ArrowUp" }, () => {
				setYIndex(Math.max(0, yIndex - 1));
				if (isScrubbing) {
					toggleRowSelection(yIndex);
				}
			})
			.with(
				{ key: "ArrowRight" },
				() => setXIndex(Math.min(columns.length, xIndex + 1)),
			)
			.with(
				{ key: "ArrowDown" },
				() => {
					setYIndex(Math.min(rows.length, yIndex + 1));
					if (isScrubbing) {
						toggleRowSelection(yIndex);
					}
				},
			)
			.with({ key: " " }, () => {
				if (yIndex === 0 && selectionOptions.enableMultiRowSelection !== false) {
					console.log("toggle all");
					getToggleAllRowsSelectedHandler()(new Event("change"));

					return;
				}

				startScrubbing();
				toggleRowSelection(yIndex - 1);
			});
	}}
	onpointerdown={(event) => {
		if (
			event.button !== 0
		) {
			return;
		}

		startScrubbing();
	}}
	{...rest}
>
	<colgroup>
		{#if tableOptions.enableRowSelection}
			<col style="width: 32px" />
		{/if}
		{#each headerGroups as { headers }}
			{#each headers as { column: { getSize } }}
				<col style:width={getSize()} class="min-w-0" />
			{/each}
		{/each}
	</colgroup>

	<thead class="bg-base-300 border-inherit">
		{#each headerGroups as { headers }}
			<tr aria-rowindex="1" class="border-inherit text-sm shadow-md">
				{#if selectionOptions.enableRowSelection}
					<th
						tabindex={xIndex === 0 && yIndex === 0 ? 0 : -1}
						aria-colindex="1"
						onfocus={() => annotationText = "Press Space to select all rows"}
						onblur={() => annotationText = ""}
						class="p-cell first:rounded-tl-theme last:rounded-tr-theme text-center"
					>
						<span class="sr-only">Selected</span>
						{#if selectionOptions.enableMultiRowSelection !== false}
							<Checkbox
								tabindex={-1}
								indeterminate={getIsSomeRowsSelected()}
								disabled={!data.length}
								checked={getIsAllRowsSelected()}
								onchange={getToggleAllRowsSelectedHandler()}
							/>
						{/if}
					</th>
				{/if}

				{#each headers as { getContext, column: { columnDef: { header, meta } } }, colIndex}
					{@const offsetColIndex = colIndex + (selectionOptions.enableRowSelection ? 1 : 0)}
					<th
						role="columnheader"
						class={[
							"p-cell first:rounded-tl-theme last:rounded-tr-theme",
							meta?.truncate !== false && "truncate",
						]}
						aria-colindex={offsetColIndex + 1}
						tabindex={offsetColIndex === xIndex && yIndex === 0 ? 0 : -1}
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
	<tbody class="divide-y divide-primary/15 border-inherit inset-shadow-xs inset-shadow-highlight/25">
		{#each rows as row, rowIndex}
			<tr
				aria-rowindex={rowIndex + 2}
				class={[
					"border-inherit first:border-transparent",
					row.getCanExpand() && "cursor-pointer",
					row.getIsSelected() ? "bg-primary/10" : "has-hover:bg-primary/5",
				]}
			>
				{#if tableOptions.enableRowSelection}
					{@const isCellSelected = xIndex === 0 && yIndex === rowIndex + 1}
					<td
						role="gridcell"
						aria-selected={isCellSelected}
						aria-colindex="1"
						tabindex={isCellSelected ? 0 : -1}
						class="p-cell w-min text-center"
					>
						<Checkbox
							variant={row.getIsSelected() ? "primary" : "base"}
							indeterminate={row.getIsSomeSelected()}
							disabled={!row.getCanSelect()}
							checked={row.getIsSelected()}
							onchange={row.getToggleSelectedHandler()}
							tabindex={-1}
							onpointermove={() => {
								if (!isScrubbing || idleRows.has(row.id)) {
									return;
								}

								idleRows.add(row.id);
								row.getToggleSelectedHandler()(new Event("change"));
							}}
						/>
					</td>
				{/if}
				{#each row.getVisibleCells() as { getContext, column: { columnDef: { cell, meta } } }, colIndex}
					{@const offsetColIndex = colIndex + (selectionOptions.enableRowSelection ? 1 : 0)}
					{@const isCellSelected = row.getIsSelected() && yIndex === rowIndex + 1}
					<td
						role="gridcell"
						aria-colindex={offsetColIndex + 1}
						aria-selected={isCellSelected}
						tabindex={isCellSelected ? 0 : -1}
						style:direction={meta?.truncate === "start" ? "rtl" : undefined}
						style:text-align={meta?.alignment || "left"}
						class={[
							"p-cell truncate border-inherit border-t",
							meta?.truncate !== false && "truncate",
						]}
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

<style lang="postcss">
	@layer components {
		th,
		td {
			white-space: nowrap;
			overflow: hidden;

			outline: none;

			&:focus {
				box-shadow: inset 0 0 0 2px var(--color-primary);
			}
		}
	}
</style>
