import { createContext } from "svelte";

import type { Song } from "@lib/models";
import type { SongGroups } from "./group";
import type { PageManager } from "./page";

export * from "./group";
export * from "./page";

export const [songGroups, setSongGroups] = createContext<SongGroups>();
export const [pageManager, setPageManager] = createContext<PageManager>();
export const [songs, setSongs] = createContext<Map<string, Song>>();
export const [editedSongs, setEditedSongs] = createContext<Map<string, Song>>();
export const [selectedSongs, setSelectedSongs] = createContext<Set<string>>();

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
