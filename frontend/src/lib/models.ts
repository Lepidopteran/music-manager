import type { SongMetadata } from "@bindings/SongMetadata";

export interface Song extends SongMetadata {
	id: string;
	path: string;
}
