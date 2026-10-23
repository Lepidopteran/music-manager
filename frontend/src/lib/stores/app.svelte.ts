import { getAlbums, getAlbumSongs } from "#api/albums";
import { getSongs } from "#api/song";
import type { Album } from "#bindings/Album";
import type { DatabaseSong } from "#bindings/DatabaseSong";
import { SvelteMap, SvelteSet } from "svelte/reactivity";

export class AppState {
	#albums = $state<Map<string, Album>>(new Map());
	#songs = $state<Map<string, DatabaseSong>>(new Map());
	#albumSongIdMap = $derived.by(() => {
		const map = new Map<string, string[]>();

		for (const song of this.songs.values()) {
			if (!song.albumId) continue;

			const ids = map.getOrInsert(song.albumId, []);
			ids.push(song.id);
		}

		return map;
	});

	#selectedSongs = new SvelteSet<string>();
	#editedSongs = new SvelteMap<string, DatabaseSong>();

	constructor() {
		$effect.root(() => {
			$inspect(this.#albums, this.#albumSongIdMap);
		});
	}

	async refreshSongs() {
		try {
			this.#songs = new Map((await getSongs()).map((song) => [song.id, song]));
		} catch (error) {
			console.error(error);
		}
	}

	async refreshAlbums() {
		try {
			this.#albums = new Map((await getAlbums()).map((album) => [album.id, album]));
		} catch (error) {
			console.error(error);
		}
	}

	get albums() {
		if (this.#albums.size === 0) {
			this.refreshAlbums();
		}

		return this.#albums;
	}

	getAlbumSongIds(albumId: string): string[] | undefined {
		return this.#albumSongIdMap.get(albumId);
	}

	get songs(): Map<string, DatabaseSong> {
		if (this.#songs.size === 0) {
			this.refreshSongs();
		}

		return this.#songs;
	}

	get selectedSongs() {
		return this.#selectedSongs;
	}

	get editedSongs() {
		return this.#editedSongs;
	}
}

export const app = new AppState();
