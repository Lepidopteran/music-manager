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
		`added_at` DATETIME DEFAULT NULL,
		`updated_at` DATETIME DEFAULT NULL,
		`file_created_at` DATETIME DEFAULT NULL,
		`directory_id` TEXT NOT NULL,
		FOREIGN KEY (`directory_id`) REFERENCES `directories` (`name`),
		UNIQUE (`path`) ON CONFLICT REPLACE
);  

INSERT INTO `songs_new` (`id`, `path`, `title`, `artist`, `album`, `album_artist`, `genre`, `year`, `track_number`, `disc_number`, `mood`, `added_at`, `updated_at`, `file_created_at`, `directory_id`)
SELECT `id`, `path`, `title`, `artist`, `album`, `album_artist`, `genre`, `year`, `track_number`, `disc_number`, `mood`, `added_at`, `updated_at`, `file_created_at`, `directory_id`
FROM `songs`;

DROP TABLE `songs`;
ALTER TABLE `songs_new` RENAME TO `songs`;
