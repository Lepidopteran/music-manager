import type { Song, SongMetadata } from "@lib/models";
import type { Icons } from "@lib/icons";
import { SvelteMap } from "svelte/reactivity";

import UniversalRouter, {
	type ResolveContext,
	type Route,
} from "universal-router";
import { getSongs } from "@api/song";
import { type Component, untrack } from "svelte";
import type {
	SongWorkerRequest,
	SongWorkerResponse,
} from "@lib/worker-messages";
import { match } from "ts-pattern";

export type Item =
	| { type: "song"; song: Song }
	| { type: "group"; label: string; songs: Song[] };

export interface Page extends Route {
	path: string;
	name: string;
	component?: Component<{
		app: AppState;
		visible: boolean;
		[key: string]: unknown;
	}>;
	props?: Record<string, unknown>;
	icon?: Icons;
	children?: Array<Page>;
	action?: () => PageAction;
}

export interface PageAction {
	name: string;
	path: string;
	callback?: (app: AppState) => void;
}

export interface PageComponentProps {
	app: AppState;
	visible: boolean;
	[key: string]: unknown;
}

export class AppState {
	private _router: UniversalRouter;
	private _path = $state("/");
	private _name = $state("Home");
	private _fetchingTracks = $state(false);
	private _organizingArtists = $state(false);
	private _organizingAlbums = $state(false);
	private _tracks: Array<Song> = $state([]);
	private _editedTracks: SvelteMap<string, Song> = $state(new SvelteMap());
	private _artists: SvelteMap<string, Array<Song>> = $state(new SvelteMap());
	private _albums: SvelteMap<string, Array<Song>> = $state(new SvelteMap());
	private _selectedItem: Item | null = $state(null);

	private _worker: Worker = new Worker(
		new URL("../workers/song.ts", import.meta.url),
	);

	autoOrganizeArtists = $state(false);
	autoOrganizeAlbums = $state(false);

	constructor(routes: Array<Route>) {
		this._router = new UniversalRouter(routes);
		this._fetchingTracks = true;

		$inspect(`Fetching tracks: ${this._fetchingTracks}`);
		$inspect(`Organizing artists: ${this._organizingArtists}`);
		$inspect(`Organizing albums: ${this._organizingAlbums}`);

		this._worker.onmessage = (event: MessageEvent<SongWorkerResponse>) => {
			const { data } = event;

			match(data)
				.with({ type: "initialize" }, (data) => {
					this._tracks = data.payload;
				})
				.with({ type: "groupArtists" }, (data) => {
					for (const [key, value] of data.payload) {
						this._artists.set(key, value);
					}

					this._organizingArtists = false;
				})
				.with({ type: "groupAlbums" }, (data) => {
					for (const [key, value] of data.payload) {
						this._albums.set(key, value);
					}

					this._organizingAlbums = false;
				})
				.exhaustive();
		};

		$effect(() => {
			if (this._tracks.length > 0) {
				if (this.autoOrganizeArtists) {
					this.scheduleOrganizeArtists();
				}

				if (this.autoOrganizeAlbums) {
					this.scheduleOrganizeAlbums();
				}
			}
		});

		this.fetchTracks();
	}

	extendTrackInfo(id: number, info: SongMetadata) {
		const track = this._tracks.find((track) => track.id === id);

		if (track) {
			Object.assign(track, info);
		} else {
			throw new Error("Track not found");
		}
	}

	editTrack(track: Song) {
		const original = this._tracks.find((t) => t.id === track.id) as Song;

		if (
			this._editedTracks.has(track.id.toString()) &&
			JSON.stringify(original) === JSON.stringify(track)
		) {
			this._editedTracks.delete(track.id.toString());

			return;
		}

		this._editedTracks.set(track.id.toString(), track);
	}

	async fetchTracks() {
		const tracks: Array<Song> = (await getSongs()) as Array<Song>;
		this._sendMessage({
			type: "initialize",
			payload: tracks,
		});
	}

	async changePage(input: string | ResolveContext) {
		const { path, name, callback } = (await this._router.resolve(input)) as PageAction;

		this._path = path;
		this._name = name;

		if (callback) {
			callback(this);
		}
	}

	scheduleOrganizeArtists() {
		this._artists.clear();
		this._sendMessage({ type: "groupArtists" });
		this._organizingArtists = true;
	}

	scheduleOrganizeAlbums() {
		this._albums.clear();
		this._sendMessage({ type: "groupAlbums" });
		this._organizingAlbums = true;
	}

	private _sendMessage(message: SongWorkerRequest) {
		this._worker.postMessage(message);
	}

	get path() {
		return this._path;
	}

	get name() {
		return this._name;
	}

	get tracks() {
		return this._tracks;
	}

	get artists() {
		return this._artists;
	}

	get albums() {
		return this._albums;
	}

	get editedTracks() {
		return this._editedTracks;
	}

	get organizingArtists() {
		return this._organizingArtists;
	}

	get organizingAlbums() {
		return this._organizingAlbums;
	}

	get fetchingTracks() {
		return this._fetchingTracks;
	}

	get selectedItem() {
		return this._selectedItem;
	}

	set selectedItem(item: Item | null) {
		if (isItemEqual(this._selectedItem, item)) {
			return;
		}

		this._selectedItem = item;
	}
}

export function isSong(item: Item): item is Extract<Item, { type: "song" }> {
	return (item).type === "song";
}

export function isGroup(item: Item): item is Extract<Item, { type: "group" }> {
	return (item).type === "group";
}

export function isItemEqual(a: Item | null, b: Item | null): boolean {
	if (a === null || b === null) {
		return a === b;
	}

	if (isSong(a) && isSong(b)) {
		return a.song.id === b.song.id;
	}

	if (isGroup(a) && isGroup(b)) {
		return a.label === b.label;
	}

	return false;
}
