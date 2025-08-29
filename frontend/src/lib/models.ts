export interface Album {
	title: string;
	tracks: Array<DatabaseSong>;
	barcode: string | null;
	catalogNumber: string | null;
	comment: string | null;
	country: string | null;
	artist: string | null;
	label: string | null;
	date: Date | null;
	originalDate: Date | null;
}

export interface DatabaseSong {
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


export interface SongMetadata {
	album?: string;
	albumArtist?: string;
	albumSort?: string;
	artist?: string;
	artistSort?: string;
	artists?: string;
	barcode?: string;
	bpm?: string;
	catalogNumber?: string;
	comment?: string;
	composer?: string;
	composerSortOrder?: string;
	conductor?: string;
	copyright?: string;
	director?: string;
	discNumber?: string;
	discTotal?: string;
	encodedBy?: string;
	encoderSettings?: string;
	engineer?: string;
	genre?: string;
	grouping?: string;
	key?: string;
	isrc?: string;
	language?: string;
	license?: string;
	lyricist?: string;
	lyrics?: string;
	mood?: string;
	movement?: string;
	movementNumber?: string;
	movementTotal?: string;
	musicBrainzRecordingId?: string;
	musicBrainzTrackId?: string;
	musicBrainzReleaseId?: string;
	musicBrainzReleaseGroupId?: string;
	musicBrainzArtistId?: string;
	musicBrainzReleaseArtistId?: string;
	musicBrainzWorkId?: string;
	originalAlbum?: string;
	originalArtist?: string;
	originalFileName?: string;
	originalReleaseDate?: string;
	performer?: string;
	producer?: string;
	label?: string;
	releaseDate?: string;
	recordingDate?: string;
	title?: string;
	titleSort?: string;
	trackNumber?: string;
	trackTotal?: string;
	website?: string;
	work?: string;
	writer?: string;
	year?: string;
	setSubtitle?: string;
	showName?: string;
	trackSubtitle?: string;
	originalLyricist?: string;
	albumTitleSortOrder?: string;
	showNameSortOrder?: string;
	arranger?: string;
	mixDj?: string;
	mixEngineer?: string;
	musicianCredits?: string;
	publisher?: string;
	internetRadioStationName?: string;
	internetRadioStationOwner?: string;
	remixer?: string;
	popularimeter?: string;
	parentalAdvisory?: string;
	flagCompilation?: string;
	flagPodcast?: string;
	fileType?: string;
	fileOwner?: string;
	taggingTime?: string;
	length?: string;
	originalMediaType?: string;
	encoderSoftware?: string;
	encodingTime?: string;
	replayGainAlbumGain?: string;
	replayGainAlbumPeak?: string;
	replayGainTrackGain?: string;
	replayGainTrackPeak?: string;
	audioFileUrl?: string;
	audioSourceUrl?: string;
	commercialInformationUrl?: string;
	copyrightUrl?: string;
	radioStationUrl?: string;
	paymentUrl?: string;
	publisherUrl?: string;
	integerBpm?: string;
	color?: string;
	podcastDescription?: string;
	podcastSeriesCategory?: string;
	podcastUrl?: string;
	podcastGlobalUniqueId?: string;
	podcastKeywords?: string;
	description?: string;
	script?: string;
	appleXid?: string;
	appleId3v2ContentGroup?: string;

	/**
	 * Contains unknown fields that couldn't be mapped easily with [lofty](https://crates.io/crates/lofty)
	 *
	 * Currently these properties are ignored when writing metadata.
	 */
	unknown?: Record<string, string>;
}

export interface Song extends SongMetadata {
	id: number;
}

export type TaskStatus = "idle" | "running" | "stopped";

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
