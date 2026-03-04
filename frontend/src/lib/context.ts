import type { GroupedSongs } from "@lib/state/group";
import type { ResolvedRoute, RouteDefinition } from "@lib/state/router";
import type { GroupKey } from "@lib/workers";
import type { AppState } from "@state/app.svelte";
import { createContext } from "svelte";
import type { Icon } from "virtual:icons";

export const [legacyAppState, setLegacyAppState] = createContext<AppState>();

export interface SongGroups extends Partial<Record<GroupKey, GroupedSongs>> {
	track: (group: GroupKey) => void;
	untrack: (group: GroupKey) => void;
	tracked: GroupKey[];
	inProgress: GroupKey[];
}

export const [songGroups, setSongGroups] = createContext<SongGroups>();

export interface PageInfo {
	name?: string;
	hideHeader?: boolean;
	hideNavigation?: boolean;
	displayEditor?: boolean;
	icon?: Icon;
	callback?: () => void;
}

export interface PageManager {
	current?: ResolvedRoute<PageInfo>;
	goTo: (path: string, addToHistory?: boolean) => void;
	addPage: (page: RouteDefinition<PageInfo>) => void;
	removePage: (path: string) => void;
}

export const [pageManager, setPageManager] = createContext<PageManager>();
