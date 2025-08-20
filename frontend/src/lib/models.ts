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

export type TaskStatus = "idle" | "started" | "stopped";

export interface TaskInfo {
	id: string;
	name: string;
	description: string;
	steps: number;
	status: TaskStatus;
	startedAt: Date | null;
	stoppedAt: Date | null;
	completedAt: Date | null;
}

export type TaskEventType =
	| "initial"
	| "info"
	| "error"
	| "warning"
	| "progress"
	| "complete"
	| "start"
	| "stop";

export interface TaskEvent {
	kind: TaskEventType;
	source: string;
	message: string;
	timestamp: Date;
	step?: number;
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
