import type { Song } from "@lib/models";
import { type GroupKey, GroupWorker } from "@lib/workers";

export type { GroupKey } from "@lib/workers";

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

/**
 * Class to manage grouping of songs using workers.
 */
export class GroupManager {
	#songs: Songs = [];
	#maxActiveWorkers: number;
	#tracked: Set<GroupKey> = new Set();
	#workers: Map<GroupKey, GroupWorker> = new Map();
	#groups: Map<GroupKey, GroupedSongs> = new Map();

	onTrack?: GroupKeyCallback;
	onUntrack?: GroupKeyCallback;
	onRemove?: GroupKeyCallback;
	onWorkerStop?: GroupKeyCallback;
	onWorkerStart?: GroupKeyCallback;
	onWorkerError?: WorkerErrorCallback;
	onWorkerFinish?: GroupKeyCallback;

	/**
	 * Constructor for the GroupManager class.
	 * @param songs - Input for the songs to be grouped.
	 * @param options - Options for configuring the group manager.
	 */
	constructor(
		options?: GroupManagerOptions,
		songs?: Songs | null,
	) {
		const { maxActiveWorkers, ...rest } = options ?? {};

		this.#maxActiveWorkers = maxActiveWorkers ?? 2;
		this.#songs = songs || [];

		Object.assign(this, rest);

		if (this.#getSongs().length > 0) {
			this.#update();
		}
	}

	/**
	 * Update method to manage and start workers.
	 */
	#update() {
		if (this.#maxActiveWorkers < 1) {
			throw new Error("maxActiveWorkers must be greater than 0");
		}

		const trackedKeys = this.#tracked.values();

		let groupKey = trackedKeys.next().value;
		while (this.#workers.size < this.#maxActiveWorkers && groupKey !== undefined) {
			const worker = new GroupWorker();
			worker.onMessage(event => {
				const { grouped, key } = event.data;

				this.#groups.set(key, new GroupedSongs(grouped));
				this.#workers.delete(key);
				this.onWorkerFinish?.(key);
			});

			worker.onError((event) => {
				if (groupKey === undefined) {
					return;
				}

				this.onWorkerError?.(groupKey, event);
			});

			worker.postMessage({ key: groupKey, songs: this.#getSongs() });
			this.#workers.set(groupKey, worker);
			this.onWorkerStart?.(groupKey);

			groupKey = trackedKeys.next().value;
		}
	}

	/**
	 * Get the current list of songs.
	 */
	#getSongs() {
		return typeof this.#songs === "function" ? this.#songs() : this.#songs;
	}

	/**
	 * Track a group key.
	 * @param groupKey - The key to track.
	 */
	track(groupKey: GroupKey) {
		this.#tracked.add(groupKey);
		this.onTrack?.(groupKey);
		this.#update();
	}

	/**
	 * Untrack a group key.
	 * @param groupKey - The key to untrack.
	 */
	untrack(groupKey: GroupKey) {
		this.#tracked.delete(groupKey);
		this.onUntrack?.(groupKey);
		this.#update();
	}

	/**
	 * Remove a group key and terminate its worker if it currently running.
	 * @param groupKey - The key to remove.
	 */
	remove(groupKey: GroupKey) {
		const worker = this.#workers.get(groupKey);
		if (worker !== undefined) {
			worker.terminate();
			this.#workers.delete(groupKey);
			this.onWorkerStop?.(groupKey);
		}

		this.untrack(groupKey);
		this.#groups.delete(groupKey);
		this.onRemove?.(groupKey);
	}

	/**
	 * songs used for grouping.
	 * @param songs - New songs.
	 */
	set songs(songs: Songs) {
		this.#songs = songs;
		this.#update();
	}

	/**
	 * tracked group keys.
	 */
	get tracked() {
		return Array.from(this.#tracked.values());
	}

	/**
	 * Getter for the maximum number of active workers.
	 */
	get maxActiveWorkers() {
		return this.#maxActiveWorkers;
	}

	/**
	 * Active group keys that are in workers.
	 */
	get workerKeys() {
		return Array.from(this.#workers.keys());
	}

	/**
	 * Grouped songs.
	 */
	get groups(): Partial<Record<GroupKey, GroupedSongs>> {
		return Object.fromEntries(this.#groups.entries());
	}
}
