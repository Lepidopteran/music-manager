CREATE TABLE `songs` (
    `id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    `path` TEXT NOT NULL,
    `parent_path` TEXT NOT NULL,
    `title` TEXT,
    `artist` TEXT,
    `album` TEXT,
    `album_artist` TEXT,
    `genre` TEXT,
    `year` TEXT,
    `track_number` TEXT,
		`disc_number` TEXT,
    UNIQUE (`path`) ON CONFLICT REPLACE
);
