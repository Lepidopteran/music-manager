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

export interface Page extends Route {
	path: string;
	name: string;
	component?: Component<{ app: AppState; [key: string]: unknown }>;
	props?: Record<string, unknown>;
	icon?: Icons;
	children?: Array<Page>;
	action?: () => void;
}

export interface PageComponentProps {
	app: AppState;
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
	selectedItem: Song | [string, Song[]] | null = $state(null);

	private _worker: Worker = new Worker(
		new URL("../workers/song.ts", import.meta.url),
	);

	constructor(routes: Array<Route>) {
		this._router = new UniversalRouter(routes);
		this._fetchingTracks = true;

		$inspect(`Fetching tracks: ${this._fetchingTracks}`);
		$inspect(`Organizing artists: ${this._organizingArtists}`);
		$inspect(`Organizing albums: ${this._organizingAlbums}`);

		this._worker.onmessage = (event: MessageEvent<SongWorkerResponse>) => {
			const { type, payload } = event.data;

			console.log(type, payload);
			switch (type) {
				case "initialize":
					this._tracks = payload;
					break;
				case "groupArtists":
					for (const [key, value] of payload) {
						this._artists.set(key, value);
					}

					this._organizingArtists = false;
					break;
				case "groupAlbums":
					for (const [key, value] of payload) {
						this._albums.set(key, value);
					}

					this._organizingAlbums = false;
					break;
			}
		};

		$effect(() => {
			if (this._tracks.length > 0) {
				this.scheduleOrganizeAlbums();
				this.scheduleOrganizeArtists();
			}
		});

		$inspect(this._tracks.length, "Tracks");
		$inspect(this._artists.size, "Artists");
		$inspect(this._albums.size, "Albums");

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
		const { path, name } = await this._router.resolve(input);

		this._path = path;
		this._name = name;
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
}
