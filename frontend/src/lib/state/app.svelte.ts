import Home from "@pages/Albums.svelte";
import type { Song, SongMetadata } from "@lib/models";
import type { Icons } from "@lib/icons";

import UniversalRouter, {
	type ResolveContext,
	type Route,
} from "universal-router";
import { getSongs } from "@api/song";

export interface Page extends Route {
	icon?: Icons;
}

export class AppState {
	private _router: UniversalRouter;
	private _path = $state("/");
	private _name = $state("Home");
	private _pageComponent = $state(Home);
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
		const { path, name, pageComponent } = await this._router.resolve(input);

		this._path = path;
		this._name = name;
		this._pageComponent = pageComponent;
	}

	get path() {
		return this._path;
	}

	get name() {
		return this._name;
	}

	get pageComponent() {
		return this._pageComponent;
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
