import type { Album } from "src/bindings/Album";
import { fetchJson } from "../../utils/api/api";

export async function getAlbums(): Promise<Album[]> {
	return await fetchJson<Album[]>("/api/albums/");
}
