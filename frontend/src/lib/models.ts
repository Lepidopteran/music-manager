import type { DatabaseSong } from "@lib/bindings/DatabaseSong";
import type { SongMetadata } from "@lib/bindings/SongMetadata";

export type Song = DatabaseSong & SongMetadata;
