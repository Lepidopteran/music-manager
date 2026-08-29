<script lang="ts" generics="T">
	import { attributeChanged } from "@attachments/mutation";
	import { resizeOccurred } from "@attachments/resize";
	import { pressedKeys } from "@state";
	import { watch } from "@utils/reactivity/watch.svelte";
	import { onMount, type Snippet, untrack } from "svelte";
	import type { ClassValue, HTMLAttributes } from "svelte/elements";
	import { prefersReducedMotion } from "svelte/motion";
	import { SvelteSet } from "svelte/reactivity";

	const uuid = $props.id();

	const keyContext = pressedKeys();

	interface Props {
		id?: string;
		data: T[];
		columnWidth: number;
		cellHeight?: number;
		class?: ClassValue;
		multiSelect?: boolean;
		selected?: Set<unknown>;
		itemLabel?: (item: T, index: number) => string;
		getKey?: (item: T, index: number) => unknown;
		onActivate?: (
			option: T,
		) => void;
		item: Snippet<
			[{ data: T; index: number; selected: boolean; focused: boolean }]
		>;
		"aria-label"?: string;
	}

	let focusedRow = $state(-1);
	let focusedColumn = $state(-1);

	let focusedKey: unknown | null = $state(null);

	let gridFocused = $state(false);

	let keyMap: Map<unknown, number> = $derived.by(() => {
		const map = new Map();

		for (const [index, item] of data.entries()) {
			const key = getKey(item, index);
			map.set(key, index);
		}

		return map;
	});

	let columnCount = $derived.by(() => {
		return Math.floor(
			containerWidth
				/ (columnWidth + columnGap),
		) || 1;
	});

	let gridElement: HTMLDivElement;
	let styleDeclaration: null | CSSStyleDeclaration = $state(null);
	let containerWidth = $state(0);
	let containerHeight = $state(0);

	let {
		rowGap,
		columnGap,
	} = $derived.by(() => {
		if (!styleDeclaration) {
			return {
				rowGap: 0,
				columnGap: 0,
			};
		}

		const value = (px: string) => {
			return Number.parseFloat(px.replace("px", "").trim()) || 0;
		};

		return {
			rowGap: value(styleDeclaration.rowGap),
			columnGap: ((value(styleDeclaration.paddingRight)
				+ value(styleDeclaration.paddingLeft)) / 2)
				+ value(styleDeclaration.marginLeft)
				+ value(styleDeclaration.marginRight)
				+ value(styleDeclaration.borderLeft)
				+ value(styleDeclaration.borderRight)
				+ value(styleDeclaration.columnGap),
		};
	});

	export function moveToIndex(
		index: number,
		scrollOptions?: ScrollIntoViewOptions,
	) {
		moveTo(Math.floor(index / columnCount), index % columnCount, scrollOptions);
	}

	export function moveTo(
		row: number,
		column: number,
		scrollOptions?: ScrollIntoViewOptions,
	) {
		const index = layoutData[row][column];
		const item = data[index];

		if (index === undefined) {
			console.warn(
				"Position is out of bounds",
				row,
				column,
				layoutData[row][column],
			);

			return;
		}

		focusedRow = row;
		focusedColumn = column;
		focusedKey = getKey(item, index);

		document.getElementById(`${id}-item-${index}`)
			?.scrollIntoView(
				{
					behavior: prefersReducedMotion.current ? "auto" : "smooth",
					block: "nearest",
					...scrollOptions,
				},
			);
	}

	export function selectFocused() {
		selectIndex(layoutData[focusedRow][focusedColumn]);
	}

	export function toggleFocused() {
		const index = layoutData[focusedRow][focusedColumn];
		toggleIndex(index);
	}

	export function selectRow(row: number) {
		for (let i = 0; i < columnCount; i++) {
			const index = layoutData[row][i];
			if (index) {
				selectIndex(index);
			}
		}
	}

	function toggleIndex(index: number) {
		const item = data[index];

		if (item) {
			const key = getKey(item, index);
			if (selected.has(key)) {
				deselectIndex(index);
			} else {
				selectIndex(index);
			}
		}
	}

	export function selectFocusedRow() {
		selectRow(focusedRow);
	}

	export function activateFocused() {
		const index = layoutData[focusedRow][focusedColumn];
		onItemActivate(
			data[index],
		);
	}

	export function deselectIndex(index: number) {
		const item = data[index];

		if (item) {
			const key = getKey(item, index);
			selected.delete(key);
		}
	}

	export function select(keys: Array<unknown>) {
		for (const key of keys) {
			const index = keyMap.get(key);
			if (index !== undefined) {
				selectIndex(index);
			}
		}
	}

	export function selectIndex(index: number) {
		const item = data[index];

		if (item) {
			const key = getKey(item, index);
			selected.add(key);
		}
	}

	function onkeydown(event: KeyboardEvent) {
		const keys: Record<string, () => void> = {
			"ArrowLeft": () => {
				moveTo(
					Math.max(0, focusedRow),
					Math.max(0, focusedColumn - 1),
				);
				if (event.shiftKey) {
					selectFocused();
				}
			},
			"ArrowRight": () => {
				moveTo(
					Math.max(0, focusedRow),
					Math.min(columnCount - 1, focusedColumn + 1),
				);
				if (event.shiftKey) {
					selectFocused();
				}
			},
			"ArrowUp": () => {
				moveTo(
					Math.max(0, focusedRow - 1),
					Math.max(0, focusedColumn),
				);
				if (event.shiftKey) {
					selectFocused();
				}
			},
			"ArrowDown": () => {
				moveTo(
					Math.min(layoutData.length - 1, focusedRow + 1),
					Math.max(0, focusedColumn),
				);
				if (event.shiftKey) {
					selectFocused();
				}
			},
			"Home": () => {
				if (event.ctrlKey) {
					moveTo(0, 0);
				} else {
					moveTo(focusedRow, 0);
				}
			},
			"End": () => {
				if (event.ctrlKey) {
					moveTo(layoutData.length - 1, columnCount - 1);
				} else {
					moveTo(focusedRow, columnCount - 1);
				}
			},
			" ": () => {
				toggleFocused();
			},
			"Enter": activateFocused,
		};

		if (keys[event.key]) {
			event.preventDefault();
			keys[event.key]();
		}
	}

	let {
		id = `grid-list-${uuid}`,
		data,
		class: className,
		item,
		getKey = (_, index) => index,
		onActivate: onItemActivate = () => {},
		itemLabel = (_, index) => `Grid item ${index + 1}`,
		columnWidth,
		selected = $bindable(new SvelteSet()),
		multiSelect = true,
		...rest
	}: Props = $props();

	export const selectedItems = selected;

	let layoutData = $derived.by(() => {
		const rows = [];
		for (
			let rowIndex = 0;
			rowIndex < Math.ceil(data.length / columnCount);
			rowIndex++
		) {
			const row = [];

			for (let cellIndex = 0; cellIndex < columnCount; cellIndex++) {
				const index = rowIndex * columnCount + cellIndex;
				if (index < data.length) {
					row.push(index);
				}
			}

			rows.push(row);
		}

		return rows;
	});

	const keys = pressedKeys();

	onMount(() => {
		styleDeclaration = getComputedStyle(gridElement);
		containerWidth = gridElement.offsetWidth;
		containerHeight = gridElement.offsetHeight;
	});

	watch([() => columnCount, () => data], () => {
		if (focusedKey !== undefined && focusedKey !== null) {
			const index = keyMap.get(focusedKey);

			if (index !== undefined) {
				moveToIndex(index, {
					behavior: "auto",
				});
			}
		}
	});

	watch(() => data, () => {
		for (const key of selected.values()) {
			const index = keyMap.get(key);
			if (index === undefined) {
				selected.delete(key);
			}
		}
	});
</script>

<div
	{id}
	bind:this={gridElement}
	class={[
		"grid-list outline-none focus:outline-solid outline outline-primary/25 outline-offset-2",
		className,
	]}
	role="grid"
	aria-label="Grid container"
	aria-rowcount={data.length / columnCount}
	aria-colcount={columnCount}
	{onkeydown}
	onfocus={() => {
		gridFocused = true;
	}}
	onblur={() => {
		gridFocused = false;
	}}
	tabindex="0"
	{...rest}
	{@attach attributeChanged((name, oldValue, newValue) => {
		if (["class", "style"].includes(name) && oldValue !== newValue) {
			styleDeclaration = getComputedStyle(gridElement);
		}
	})}
	{@attach resizeOccurred((entry) => {
		containerWidth = entry.contentRect.width;
		containerHeight = entry.contentRect.height;
	})}
>
	{#each layoutData as row, rowIndex (rowIndex)}
		<div
			role="row"
			class="grid-list-row"
			style:grid-template-columns="repeat({columnCount}, minmax({columnWidth}px, 1fr))"
		>
			{#each row as index, columnIndex (index)}
				{@const cellData = data[index]}
				{@const key = getKey(cellData, index)}
				{@const focused = focusedRow === rowIndex && focusedColumn === columnIndex}
				{@const isSelected = selected.has(key)}
				<button
					role="gridcell"
					id={`${id}-item-${index}`}
					aria-label={itemLabel(cellData, index)}
					aria-selected={isSelected}
					data-active={focused}
					data-index={index}
					class={[
						"p-2 w-full text-center hover:bg-primary/15 cursor-pointer outline-none truncate rounded-theme shadow shadow-shade/25 inset-shadow-sm inset-shadow-highlight/25",
						gridFocused && focused
						&& "outline-2 outline-solid outline-offset-2 outline-primary/50",
						isSelected
						&& "bg-primary/25 hover:bg-primary/30 inset-shadow-shade/50",
					]}
					onclick={(_) => {
						moveToIndex(index);
						if (
							!multiSelect || !keyContext.has("control")
						) {
							selected.clear();
						}

						toggleIndex(index);
					}}
					ondblclick={(_) => {
						onItemActivate(cellData);
					}}
					tabindex="-1"
				>
					{@render item({
						data: cellData,
						index: columnIndex,
						selected: isSelected,
						focused: focused,
					})}
				</button>
			{/each}
		</div>
	{/each}
</div>

<style>
	@layer components {
		.grid-list {
			display: grid;
			gap: calc(var(--spacing) * 2);
		}

		.grid-list-row {
			display: grid;
			gap: inherit;
		}
	}
</style>
