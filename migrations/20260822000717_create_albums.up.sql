CREATE TABLE albums (
	id TEXT NOT NULL PRIMARY KEY,
	title TEXT NOT NULL,
	artist TEXT DEFAULT NULL,
	artist_sort TEXT DEFAULT NULL,
	artists TEXT DEFAULT NULL,
	artists_sort TEXT DEFAULT NULL,
	original_release_date TEXT DEFAULT NULL,
	label TEXT DEFAULT NULL,
	barcode TEXT DEFAULT NULL,
	release_date TEXT DEFAULT NULL,
	disc_total TEXT DEFAULT NULL,
	musicbrainz_release_id TEXT DEFAULT NULL,
	musicbrainz_release_artist_id TEXT DEFAULT NULL,
	musicbrainz_release_group_id TEXT DEFAULT NULL,
	script TEXT DEFAULT NULL,
	language TEXT DEFAULT NULL,
	replaygain_album_gain TEXT DEFAULT NULL,
	replaygain_album_peak TEXT DEFAULT NULL,
	added_at DATETIME NOT NULL,
	updated_at DATETIME DEFAULT NULL
);

ALTER TABLE songs ADD COLUMN album_id TEXT REFERENCES albums(id);

CREATE INDEX idx_albums_title ON albums(title);
