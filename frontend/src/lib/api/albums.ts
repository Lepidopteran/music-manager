import type { Album } from "#bindings/Album";
import type { DatabaseSong } from "#bindings/DatabaseSong";
import { fetchJson } from "#utils/api";

export async function getAlbums(): Promise<Array<Album>> {
	return await fetchJson<Array<Album>>("/api/albums/");
}

export async function getAlbum(id: string): Promise<Album> {
	return await fetchJson<Album>(`/api/albums/${id}`);
}

export async function getAlbumSongs(albumId: string): Promise<Array<DatabaseSong>> {
	return await fetchJson<Array<DatabaseSong>>(`/api/albums/${albumId}/songs`);
}
