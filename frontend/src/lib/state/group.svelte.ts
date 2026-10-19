import type { DatabaseSong } from "#bindings/DatabaseSong";
import type { Song } from "#lib/models.ts";
import { app } from "#lib/state/app.svelte.ts";
import { watch } from "#lib/utils/reactivity/watch.svelte.ts";
import { SvelteMap, SvelteSet } from "svelte/reactivity";
import { GroupWebWorker } from "../workers";

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

export class SongGroupManager implements GroupManager {
	#maxActiveWorkers: number = 3;
	#tracked: SvelteSet<GroupKey> = new SvelteSet();
	#workers: SvelteMap<GroupKey, GroupWebWorker> = new SvelteMap();
	#groups: SvelteMap<GroupKey, GroupedSongs> = new SvelteMap();

	constructor() {
		$effect.root(() => {
			watch(() => app.songs.size, () => this.#update());
		});
	}

	#update() {
		const trackedKeys = this.#tracked.values();

		let groupKey = trackedKeys.next().value;
		while (
			this.#workers.size < this.#maxActiveWorkers && groupKey !== undefined
		) {
			const worker = new GroupWebWorker();
			worker.onMessage(event => {
				const { grouped, key } = event.data;

				this.#groups.set(key, new GroupedSongs(grouped));
				this.#workers.delete(key);
			});

			worker.postMessage({
				key: groupKey,
				songs: $state.snapshot(app.songs.values().toArray()),
			});

			this.#workers.set(groupKey, worker);

			groupKey = trackedKeys.next().value;
		}
	}

	track(groupKey: GroupKey) {
		this.#tracked.add(groupKey);
		this.#update();
	}

	untrack(groupKey: GroupKey) {
		this.#tracked.delete(groupKey);
		this.#update();
	}

	get groups() {
		return this.#groups;
	}

	get tracked() {
		return this.#tracked.values().toArray();
	}

	get inProgress() {
		return this.#workers.keys().toArray();
	}
}

export const songGroups = new SongGroupManager();
