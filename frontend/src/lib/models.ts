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

export interface TaskInfo {
	id: number;
	name: string;
	description: string;
}

export type TaskEventType =
	| "Initial"
	| "Info"
	| "Error"
	| "Warning"
	| "Progress"
	| "Complete"
	| "Start"
	| "Stop";

export interface TaskEvent {
	kind: TaskEventType;
	source: string;
	message: string;
	current?: number;
	total?: number;
}

export interface NewDirectory {
	name: string;
	path: string;
}

export interface Directory {
	name: string;
	path: string;
	pathSize: number;
	freeSpace: number;
	totalSpace: number;
}
