import type { DatabaseSong } from "../models";
import { fetchJson } from "../utils/api";
export async function getSongs(): Promise<Array<DatabaseSong>> {
	return await fetchJson<Array<DatabaseSong>>("/api/songs/");
}
