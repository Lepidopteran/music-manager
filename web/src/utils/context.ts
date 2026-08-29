import { createContext } from "svelte";

/**
 * Helper function to create a context with safe getter that returns undefined instead of throwing
 * when accessed outside of the context provider.
 */
export function createSafeContext<T>() {
	const [getRaw, set] = createContext<T>();

	function get(): T | undefined {
		try {
			return getRaw();
		} catch {
			return undefined;
		}
	}

	return [get, set] as const;
}
