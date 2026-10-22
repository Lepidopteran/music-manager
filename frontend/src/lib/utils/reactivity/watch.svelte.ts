import { untrack } from "svelte";

type Getter<T> = () => T;

function startWatch<T>(
	sources: Getter<T> | Array<Getter<T>>,
	effect: (
		values: T | Array<T>,
		previousValues: T | undefined | Array<T | undefined>,
	) => void | VoidFunction,
): void {
	let previousValues: T | undefined | Array<T | undefined> = Array.isArray(sources)
		? []
		: undefined;

	$effect(() => {
		const values = Array.isArray(sources) ? sources.map((source) => source()) : sources();

		const cleanup = untrack(() => effect(values, previousValues));
		previousValues = values;

		return cleanup;
	});
}

export function watch<T extends Array<unknown>>(
	sources: {
		[K in keyof T]: Getter<T[K]>;
	},
	effect: (
		values: T,
		previousValues: {
			[K in keyof T]: T[K] | undefined;
		},
	) => void | VoidFunction,
): void;

export function watch<T>(
	source: Getter<T>,
	effect: (value: T, previousValue: T | undefined) => void | VoidFunction,
): void;

export function watch<T>(
	sources: Getter<T> | Array<Getter<T>>,
	effect: (
		values: T | Array<T>,
		previousValues: T | undefined | Array<T | undefined>,
	) => void | VoidFunction,
): void {
	startWatch(sources, effect);
}
