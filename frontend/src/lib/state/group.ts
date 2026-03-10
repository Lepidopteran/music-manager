import type { Song } from "@lib/models";
import { type GroupKey } from "@lib/workers";

/**
 * Represents the type of a group key used for grouping.
 */
export type { GroupKey } from "@lib/workers";

/**
 * Interface defining the structure for song groups.
 */
export interface SongGroups extends Record<GroupKey, GroupedSongs | undefined> {
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
 * Callback function for group key events.
 */
type GroupKeyCallback = (key: GroupKey) => void;

/**
 * Callback function for worker error events.
 */
type WorkerErrorCallback = (key: GroupKey, error: ErrorEvent) => void;

/**
 * Options for the GroupManager class.
 */
export interface GroupManagerOptions {
	/**
	 * Maximum number of active workers.
	 */
	maxActiveWorkers: number;
	/**
	 * Callback function when a group key is tracked.
	 */
	onTrack?: GroupKeyCallback;
	/**
	 * Callback function when a group key is untracked.
	 */
	onUntrack?: GroupKeyCallback;
	/**
	 * Callback function when a group key is removed.
	 */
	onRemove?: GroupKeyCallback;
	/**
	 * Callback function when a worker stops.
	 */
	onWorkerStop?: GroupKeyCallback;
	/**
	 * Callback function when a worker starts.
	 */
	onWorkerStart?: GroupKeyCallback;
	/**
	 * Callback function when a worker encounters an error.
	 */
	onWorkerError?: WorkerErrorCallback;
	/**
	 * Callback function when a worker finishes.
	 */
	onWorkerFinish?: GroupKeyCallback;
}

/**
 * Type for song input which can be a function returning songs or an array of songs.
 */
type Songs = (() => Array<Song>) | Array<Song>;

/**
 * Represents a collection of grouped songs.
 */
export class GroupedSongs implements Iterable<[string, Array<Song>]> {
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

	[Symbol.iterator](): Iterator<[string, Array<Song>]> {
		return Object.entries(this._groups)[Symbol.iterator]();
	}
}
