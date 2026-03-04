import { createContext } from "svelte";

import type { AppState } from "@state/app.svelte";
import type { SongGroups } from "./group";
import type { PageManager } from "./page";

export * from "./group";
export * from "./page";

export const [legacyAppState, setLegacyAppState] = createContext<AppState>();

export const [songGroups, setSongGroups] = createContext<SongGroups>();

export const [pageManager, setPageManager] = createContext<PageManager>();
