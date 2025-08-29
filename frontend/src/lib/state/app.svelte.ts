import type { Song, SongMetadata } from "@lib/models";
import type { Icons } from "@lib/icons";

import UniversalRouter, {
	type ResolveContext,
	type Route,
} from "universal-router";
import { getSongs } from "@api/song";
import type { Component } from "svelte";

export interface Page extends Route {
	path: string;
	name: string;
	component?: Component<{ state: AppState; [key: string]: unknown }>;
	props?: Record<string, unknown>;
	icon?: Icons;
	children?: Array<Page>;
	action?: () => void;
}

export interface PageComponentProps {
	state: AppState;
	[key: string]: unknown;
}

export class AppState {
	private _router: UniversalRouter;
	private _path = $state("/");
	private _name = $state("Home");
	private _tracks = $state<Array<Song>>([]);
	private _updatedTracks = $state<Array<Song>>([]);
	private _fetchingTracks = $state(false);

	constructor(routes: Array<Route>) {
		this._router = new UniversalRouter(routes);
		this._fetchingTracks = true;

		$inspect(this.tracks);
		this.fetchTracks();
	}

	extendTrackInfo(id: number, info: SongMetadata) {
		let track = this._tracks.find((track) => track.id === id);

		if (track) {
			Object.assign(track, info);
		} else {
			throw new Error("Track not found");
		}
	}

	async fetchTracks() {
		this._tracks = (await getSongs()) as Array<Song>;
		this._fetchingTracks = false;
	}

	async changePage(input: string | ResolveContext) {
		const { path, name } = await this._router.resolve(input);

		this._path = path;
		this._name = name;
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

	get updatedTracks() {
		return this._updatedTracks;
	}

	get fetchingTracks() {
		return this._fetchingTracks;
	}
}
