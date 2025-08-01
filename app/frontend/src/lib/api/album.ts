import type { Album } from "../models";
import { fetchJson } from "../utils/api";

export async function getAlbums(): Promise<Album[]> {
	return await fetchJson<Album[]>("/api/albums/");
}
