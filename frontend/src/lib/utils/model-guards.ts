import type { DatabaseSong, Album } from "@lib/models";

export function isSong(input: DatabaseSong | Album): input is DatabaseSong {
	if (!input) return false;
	return "path" in input;
}

export function isAlbum(input: DatabaseSong | Album): input is Album {
	if (!input) return false;
	return "tracks" in input;
}
