import type { DatabaseSong } from "@bindings/DatabaseSong";
import type { SongMetadata } from "@bindings/SongMetadata";
import { fetchJson } from "@utils/api";

export async function getSongs(): Promise<Array<DatabaseSong>> {
	return await fetchJson<Array<DatabaseSong>>("/api/songs/");
}

export async function getSongExtendedInfo(id: number): Promise<DatabaseSong> {
	return await fetchJson<DatabaseSong>(`/api/songs/${id}/file-info`);
}

export async function editSong(id: number, song: SongMetadata): Promise<void> {
	await fetchJson<void>(`/api/songs/${id}`, {
		method: "PUT",
		body: JSON.stringify(song),
	});
}
