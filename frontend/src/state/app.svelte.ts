import type { Song } from "@lib/models";
import { SvelteMap } from "svelte/reactivity";

import { getSongs } from "@api/song";
import type { SongFile } from "@bindings/SongFile";
import type { SongMetadata } from "@bindings/SongMetadata";
import { type SongWorkerRequest, type SongWorkerResponse, songWorkerUrl } from "@lib/workers";
import { match } from "ts-pattern";

export type Item =
	| { type: "song"; song: Song; fileInfo?: SongFile }
	| { type: "group"; label: string; songs: Song[] };

export class AppState {
	#fetchingTracks = $state(false);
	#organizingArtists = $state(false);
	#organizingAlbums = $state(false);
	#tracks: SvelteMap<string, Song> = $state(new SvelteMap());
	#editedTracks: SvelteMap<string, Song> = $state(new SvelteMap());
	#artists: SvelteMap<string, Array<Song>> = $state(new SvelteMap());
	#albums: SvelteMap<string, Array<Song>> = $state(new SvelteMap());
	#selectedItem: Item | null = $state(null);

	#worker: Worker = new Worker(songWorkerUrl);

	autoOrganizeArtists = $state(false);
	autoOrganizeAlbums = $state(false);

	constructor() {
		this.#fetchingTracks = true;

		$inspect(`Fetching tracks: ${this.#fetchingTracks}`);
		$inspect(`Organizing artists: ${this.#organizingArtists}`);
		$inspect(`Organizing albums: ${this.#organizingAlbums}`);

		this.#worker.onmessage = (event: MessageEvent<SongWorkerResponse>) => {
			const { data } = event;

			match(data)
				.with({ type: "initialize" }, (data) => {
					this.#tracks = new SvelteMap(data.payload);
				})
				.with({ type: "groupArtists" }, (data) => {
					for (const [key, value] of data.payload) {
						this.#artists.set(key, value);
					}

					this.#organizingArtists = false;
				})
				.with({ type: "groupAlbums" }, (data) => {
					for (const [key, value] of data.payload) {
						this.#albums.set(key, value);
					}

					this.#organizingAlbums = false;
				})
				.exhaustive();
		};

		// TODO: Consider trying an alternative way to update updatedTracks
		$effect(() => {
			if (this.#selectedItem) {
				this.#editItem(this.#selectedItem);
			}
		});

		$effect(() => {
			if (this.#tracks.size > 0) {
				if (this.autoOrganizeArtists) {
					this.scheduleOrganizeArtists();
				}

				if (this.autoOrganizeAlbums) {
					this.scheduleOrganizeAlbums();
				}
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
		const tracks: Array<Song> = (await getSongs()) as Array<Song>;
		this.#sendMessage({
			type: "initialize",
			payload: tracks,
		});
	}

	scheduleOrganizeArtists() {
		this.#artists.clear();
		this.#sendMessage({ type: "groupArtists" });
		this.#organizingArtists = true;
	}

	scheduleOrganizeAlbums() {
		this.#albums.clear();
		this.#sendMessage({ type: "groupAlbums" });
		this.#organizingAlbums = true;
	}

	#sendMessage(message: SongWorkerRequest) {
		this.#worker.postMessage(message);
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

	get artists() {
		return this.#artists;
	}

	get albums() {
		return this.#albums;
	}

	get editedTracks() {
		return this.#editedTracks;
	}

	get organizingArtists() {
		return this.#organizingArtists;
	}

	get organizingAlbums() {
		return this.#organizingAlbums;
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
