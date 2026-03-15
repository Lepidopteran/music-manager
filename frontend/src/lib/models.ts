import type { DatabaseSong } from "src/bindings/DatabaseSong";
import type { SongMetadata } from "src/bindings/SongMetadata";

export type Song = DatabaseSong & SongMetadata;
