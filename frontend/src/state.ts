import type { DatabaseSong } from "@lib/bindings/DatabaseSong";
import type { Song } from "@lib/models";
import type { ResolvedRoute, Route, Router } from "@lib/router";
import { createContext } from "svelte";
import type { Icon } from "virtual:icons";

type Songs = Map<string, Song>;

export const [groupManager, setGroupManager] = createContext<GroupManager>();
export const [pageManager, setPageManager] = createContext<PageManager>();
export const [songs, setSongs] = createContext<Songs>();
export const [editedSongs, setEditedSongs] = createContext<Songs>();
export const [selectedSongs, setSelectedSongs] = createContext<Set<string>>();

export interface PageInfo {
	name?: string;
	hideHeader?: boolean;
	hideNavigation?: boolean;
	displayEditor?: boolean;
	icon?: Icon;
	callback?: () => void;
}

export interface PageManager {
	router: Router<PageInfo>;
	pages: Array<Route<PageInfo>>;
	current?: ResolvedRoute<PageInfo>;
	goTo: (path: string, addToHistory?: boolean) => void;
}

/**
 * Type for song group keys.
 */
export type GroupKey = keyof DatabaseSong;

/**
 * Interface defining the structure for song groups.
 */
export interface GroupManager {
	groups: Map<GroupKey, GroupedSongs>;

	/**
	 * Adds a key to keep track of to group songs.
	 * @param group - The key to track.
	 */
	track: (group: GroupKey) => void;

	/**
	 * Removes a key from being tracked.
	 * @param group - The key to untrack.
	 */
	untrack: (group: GroupKey) => void;

	/**
	 * Array containing the keys of all groups that currently have songs tracked.
	 */
	tracked: GroupKey[];

	/**
	 * Array containing the keys of all groups that are currently in progress.
	 */
	inProgress: GroupKey[];
}

/**
 * Represents a collection of grouped songs.
 */
export class GroupedSongs {
	// NOTE: Can't use actual private field here because Object can't interact with it
	// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes/Private_elements
	private _groups: Record<string, Array<Song>>;

	constructor(
		groups: Record<string, Array<Song>>,
	) {
		this._groups = groups;
	}

	/**
	 * Returns the number of song groups.
	 */
	length(): number {
		return Object.keys(this._groups).length;
	}

	/**
	 * Retrieves songs for a given group key.
	 * @param key - The group key to retrieve songs for.
	 * @returns An array of songs associated with the group key.
	 */
	get(key: string): Array<Song> | undefined {
		return this._groups[key];
	}

	/**
	 * Checks if a specific group key exists in the collection.
	 * @param key - The group key to check.
	 * @returns True if the group key exists, false otherwise.
	 */
	has(key: string): boolean {
		return key in this._groups;
	}

	entries() {
		return Object.entries(this._groups);
	}

	keys() {
		return Object.keys(this._groups);
	}

	values() {
		return Object.values(this._groups);
	}
}
