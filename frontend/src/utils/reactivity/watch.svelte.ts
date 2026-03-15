import { untrack } from "svelte";

type Getter<T> = () => T;

export function watch<T>(
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
