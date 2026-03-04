import type { Song } from "@lib/models";
import { SvelteMap } from "svelte/reactivity";

import { getSongs } from "@api/song";
import type { SongFile } from "@bindings/SongFile";
import type { SongMetadata } from "@bindings/SongMetadata";

export type Item =
	| { type: "song"; song: Song; fileInfo?: SongFile }
	| { type: "group"; label: string; songs: Song[] };

export class AppState {
	#fetchingTracks = $state(false);
	#tracks: SvelteMap<string, Song> = $state(new SvelteMap());
	#editedTracks: SvelteMap<string, Song> = $state(new SvelteMap());
	#selectedItem: Item | null = $state(null);

	constructor() {
		this.#fetchingTracks = true;

		// TODO: Consider trying an alternative way to update updatedTracks
		$effect(() => {
			if (this.#selectedItem) {
				this.#editItem(this.#selectedItem);
			}
		});

		this.fetchTracks();
	}

	extendTrackInfo(id: string, info: SongMetadata) {
		const track = this.#tracks.get(id);

		if (track) {
			Object.assign(track, info);
			this.#tracks.set(id, track);
		} else {
			throw new Error("Track not found");
		}
	}

	async fetchTracks() {
		const songs: Array<Song> = (await getSongs()) as Array<Song>;
		this.#tracks = new SvelteMap(songs.map((song) => [song.id, song]));
	}

	#editItem(item: Item) {
		if (isGroup(item)) {
			for (const song of item.songs) {
				this.#editItem({ type: "song", song });
			}

			return;
		}

		const original = this.#tracks.get(item.song.id);

		if (
			original
			// @ts-expect-error
			&& Object.keys(original).every((key: keyof Song) => {
				// TODO: compare unknown field
				if (key === "unknown") {
					return true;
				}

				return original[key] === item.song[key];
			})
		) {
			this.#editedTracks.delete(item.song.id.toString());
			return;
		}

		this.#editedTracks.set(item.song.id.toString(), item.song);
	}

	get tracks() {
		return this.#tracks;
	}

	get editedTracks() {
		return this.#editedTracks;
	}

	get fetchingTracks() {
		return this.#fetchingTracks;
	}

	get selectedItem() {
		return this.#selectedItem;
	}

	set selectedItem(item: Item | null) {
		if (isItemEqual(this.#selectedItem, item)) {
			return;
		}

		this.#selectedItem = item;
	}
}

export function isSong(item: Item): item is Extract<Item, { type: "song" }> {
	return item.type === "song";
}

export function isGroup(item: Item): item is Extract<Item, { type: "group" }> {
	return item.type === "group";
}

export function isItemEqual(a: Item | null, b: Item | null): boolean {
	if (a === null || b === null) {
		return a === b;
	}

	if (isSong(a) && isSong(b)) {
		return a.song.id === b.song.id;
	}

	if (isGroup(a) && isGroup(b)) {
		return a.label === b.label;
	}

	return false;
}
