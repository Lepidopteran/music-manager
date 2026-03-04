import { createContext } from "svelte";

import type { GroupKey } from "@lib/workers";
import type { AppState } from "@state/app.svelte";
import type { GroupedSongs } from "./group";
import type { PageManager } from "./page";

export * from "./group";
export * from "./page";

export const [legacyAppState, setLegacyAppState] = createContext<AppState>();

export interface SongGroups extends Partial<Record<GroupKey, GroupedSongs>> {
	track: (group: GroupKey) => void;
	untrack: (group: GroupKey) => void;
	tracked: GroupKey[];
	inProgress: GroupKey[];
}

export const [songGroups, setSongGroups] = createContext<SongGroups>();

export const [pageManager, setPageManager] = createContext<PageManager>();
