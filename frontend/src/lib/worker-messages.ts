import type { Song } from "./models";

export type SongWorkerRequest =
	| { type: "initialize"; payload: Song[] }
	| { type: "groupArtists"; }
	| { type: "groupAlbums"; };

export type SongWorkerResponse =
	| { type: "initialize"; payload: Map<string, Song> }
	| { type: "groupArtists"; payload: Map<string, Array<Song>> }
	| { type: "groupAlbums"; payload: Map<string, Array<Song>> };
