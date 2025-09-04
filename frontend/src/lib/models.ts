import type { SongMetadata } from "@bindings/SongMetadata";

export interface Song extends SongMetadata {
	id: bigint;
	path: string;
}
