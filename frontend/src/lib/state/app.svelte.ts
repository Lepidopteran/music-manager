import { getSongs } from "#api/song";
import type { Song } from "#lib/models.ts";
import { SvelteMap, SvelteSet } from "svelte/reactivity";

export class AppState {
	#songs = new SvelteMap<string, Song>();
	selectedSongs = new SvelteSet<string>();
	editedSongs = new SvelteMap<string, Song>();

	constructor() {
	}

	async refreshSongs() {
		try {
			for (
				const [id, song] of (await getSongs()).map(song => [song.id, song as Song] as const)
			) {
				this.#songs.set(id, song);
			}
		} catch (error) {
			console.error(error);
		}
	}

	get songs() {
		if (this.#songs.size === 0) {
			this.refreshSongs();
		}

		return this.#songs;
	}
}

export const app = new AppState();
