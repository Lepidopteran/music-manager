import {
	createTable as createCoreTable,
	type RowData,
	type TableOptions,
	type TableOptionsResolved,
	type TableState,
} from "@tanstack/table-core";
import type { Component, ComponentProps, Snippet } from "svelte";

declare module "@tanstack/table-core" {
	interface ColumnMeta<TData extends RowData, TValue> {
		alignment?: "left" | "center" | "right"; 
		truncate?: boolean | "start" | "end";
	}
}

export type ContentValueReturnType<
	Params = never,
	Comp extends Component = Component,
> =
	| string
	| { snippet: Snippet<[Params]>; params?: Params }
	| { component: Comp; props?: ComponentProps<Comp> };

export function createTable<D extends RowData>(options: TableOptions<D>) {
	const resolvedOptions: TableOptionsResolved<D> = {
		state: {},
		onStateChange: () => {},
		renderFallbackValue: null,
		mergeOptions(defaultOptions, options) {
			return {
				...defaultOptions,
				...options,
			};
		},
		...options,
	};

	const table = createCoreTable(resolvedOptions);
	let state = $state<TableState>(table.initialState);

	function updateOptions() {
		table.setOptions((prev) => {
			return {
				...prev,
				...options,
				state: { ...state, ...options.state },

				onStateChange: (updater) => {
					if (typeof updater === "function") {
						state = updater(state);
					} else {
						state = { ...state, ...updater };
					}

					resolvedOptions.onStateChange?.(updater);
				},
			};
		});
	}

	updateOptions();

	$effect.pre(() => {
		updateOptions();
	});

	return table;
}
