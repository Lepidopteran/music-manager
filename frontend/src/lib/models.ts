export interface Album {
	title: string;
	tracks: Array<Song>;
	barcode: string | null;
	catalogNumber: string | null;
	comment: string | null;
	country: string | null;
	artist: string | null;
	label: string | null;
	date: Date | null;
	originalDate: Date | null;
}

export interface Song {
	id: number;
	path: string;
	title: string | null;
	artist: string | null;
	album: string | null;
	albumArtist: string | null;
	genre: string | null;
	trackNumber: string | null;
	discNumber: string | null;
	year: string | null;
}

export interface NewDirectory {
	name: string;
	path: string;
}

export interface Directory {
	name: string;
	path: string;
	freeSpace: number;
	totalSpace: number;
}
