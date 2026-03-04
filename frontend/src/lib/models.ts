import type { DatabaseSong } from "@bindings/DatabaseSong";
import type { SongMetadata } from "@bindings/SongMetadata";

export type Song = DatabaseSong & SongMetadata;
