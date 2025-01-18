import type { Song, Album } from "@lib/models";

export function isSong(input: Song | Album): input is Song {
	if (!input) return false;
	return "path" in input;
}

export function isAlbum(input: Song | Album): input is Album {
	if (!input) return false;
	return "tracks" in input;
}
