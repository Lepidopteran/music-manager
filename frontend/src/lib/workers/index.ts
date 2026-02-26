import type { Song } from "@lib/models";

export const songWorkerUrl = new URL("./song.ts", import.meta.url);

export type SongWorkerRequest =
	| { type: "initialize"; payload: Song[] }
	| { type: "groupArtists" }
	| { type: "groupAlbums" };

export type SongWorkerResponse =
	| { type: "initialize"; payload: Map<string, Song> }
	| { type: "groupArtists"; payload: Map<string, Array<Song>> }
	| { type: "groupAlbums"; payload: Map<string, Array<Song>> };
