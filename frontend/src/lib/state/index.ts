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
