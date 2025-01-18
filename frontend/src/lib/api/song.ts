import type { Song } from "../models";
import { fetchJson } from "../utils/api";
export async function getSongs(): Promise<Array<Song>> {
	return await fetchJson<Array<Song>>("/api/songs/");
}
