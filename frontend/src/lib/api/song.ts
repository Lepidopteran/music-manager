import type { DatabaseSong } from "@bindings/DatabaseSong";
import type { SongFile } from "@bindings/SongFile";
import type { SongMetadata } from "@bindings/SongMetadata";
import { fetchJson } from "@utils/api";

export async function getSongs(): Promise<Array<DatabaseSong>> {
	return await fetchJson<Array<DatabaseSong>>("/api/songs/");
}

export async function getSongFileInfo(id: string): Promise<SongFile> {
	return await fetchJson<SongFile>(`/api/songs/${id}/file-info`);
}

export async function editSong(id: string, song: SongMetadata): Promise<void> {
	await fetchJson<void>(`/api/songs/${id}`, {
		method: "PUT",
		body: JSON.stringify(song),
	});
}
