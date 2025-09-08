CREATE TABLE `songs_new` (
    `id` TEXT NOT NULL PRIMARY KEY,
    `path` TEXT NOT NULL,
    `title` TEXT,
    `artist` TEXT,
    `album` TEXT,
    `album_artist` TEXT,
    `genre` TEXT,
    `year` TEXT,
    `track_number` TEXT,
		`disc_number` TEXT, `mood` TEXT,
    UNIQUE (`path`) ON CONFLICT REPLACE
);

INSERT INTO `songs_new` (`id`, `path`, `title`, `artist`, `album`, `album_artist`, `genre`, `year`, `track_number`, `disc_number`, `mood`)
SELECT `uuid`, `path`, `title`, `artist`, `album`, `album_artist`, `genre`, `year`, `track_number`, `disc_number`, `mood`
FROM `songs`;

DROP TABLE `songs`;
ALTER TABLE `songs_new` RENAME TO `songs`;
