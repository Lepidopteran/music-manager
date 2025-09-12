import type { Song } from "@lib/models";
import type { Icons } from "@lib/icons";
import { SvelteMap } from "svelte/reactivity";

import type { Component } from "svelte";
import type {
	SongWorkerRequest,
	SongWorkerResponse,
} from "@lib/worker-messages";
import { match } from "ts-pattern";
import { match as matchPath, type MatchFunction } from "path-to-regexp";
import type { SongMetadata } from "@bindings/SongMetadata";
import type { SongFile } from "@bindings/SongFile";
import { getSongs } from "@api/song";

export type Item =
	| { type: "song"; song: Song; fileInfo?: SongFile }
	| { type: "group"; label: string; songs: Song[] };

export interface Page {
	path: string;
	name: string;
	children?: Array<Page>;
	hidden?: boolean;
	hideNavigation?: boolean;
	hideHeader?: boolean;
	displayEditor?: boolean;
	props?: Record<string, unknown>;
	icon?: Icons;
	callback?: (app: AppState) => void;
	component?: Component<{
		app: AppState;
		visible: boolean;
		[key: string]: unknown;
	}>;
}

interface Route {
	path: string;
	name: string;
	children?: Array<Route>;
	hidden?: boolean;
	hideNavigation?: boolean;
	hideHeader?: boolean;
	displayEditor?: boolean;
	props?: Record<string, unknown>;
	icon?: Icons;
	callback?: (app: AppState) => void;
	matcher: MatchFunction<object>;
	component?: Component<{
		app: AppState;
		visible: boolean;
		[key: string]: unknown;
	}>;
}

export type PageResponse = null | {
	path: string;
	name: string;
	params: Record<string, string>;
	hidden?: boolean;
	hideHeader?: boolean;
	hideNavigation?: boolean;
	displayEditor?: boolean;
	props?: Record<string, unknown>;
	child?: PageResponse;
	icon?: Icons;
};

export interface PageComponentProps {
	app: AppState;
	visible: boolean;
	[key: string]: unknown;
}

function mapPageToRoute(page: Page): Route {
	return {
		...page,
		matcher: matchPath(page.path, {
			end: !(page.children && page.children.length > 0),
		}),
		children: page.children?.map(mapPageToRoute),
	};
}

export class AppState {
	private _fetchingTracks = $state(false);
	private _organizingArtists = $state(false);
	private _organizingAlbums = $state(false);
	private _tracks: SvelteMap<string, Song> = $state(new SvelteMap());
	private _editedTracks: SvelteMap<string, Song> = $state(new SvelteMap());
	private _artists: SvelteMap<string, Array<Song>> = $state(new SvelteMap());
	private _albums: SvelteMap<string, Array<Song>> = $state(new SvelteMap());
	private _selectedItem: Item | null = $state(null);
	private _routes: Array<Route>;
	private _page: PageResponse = $state(null);
	private _path = $derived(this._page?.path || "/");

	private _worker: Worker = new Worker(
		new URL("../workers/song.ts", import.meta.url),
	);

	autoOrganizeArtists = $state(false);
	autoOrganizeAlbums = $state(false);

	constructor(pages: Array<Page>) {
		this._routes = pages.map(mapPageToRoute);
		this._fetchingTracks = true;

		$inspect(`Fetching tracks: ${this._fetchingTracks}`);
		$inspect(`Organizing artists: ${this._organizingArtists}`);
		$inspect(`Organizing albums: ${this._organizingAlbums}`);

		this._worker.onmessage = (event: MessageEvent<SongWorkerResponse>) => {
			const { data } = event;

			match(data)
				.with({ type: "initialize" }, (data) => {
					this._tracks = new SvelteMap(data.payload);
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

		// TODO: Consider trying an alternative way to update updatedTracks
		$effect(() => {
			if (this._selectedItem) {
				this._editItem(this._selectedItem);
			}
		});

		$effect(() => {
			if (this._tracks.size > 0) {
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

	extendTrackInfo(id: string, info: SongMetadata) {
		const track = this._tracks.get(id);

		if (track) {
			Object.assign(track, info);
			this._tracks.set(id, track);
		} else {
			throw new Error("Track not found");
		}
	}

	async fetchTracks() {
		const tracks: Array<Song> = (await getSongs()) as Array<Song>;
		this._sendMessage({
			type: "initialize",
			payload: tracks,
		});
	}

	changePage(path: string) {
		this._page = this._resolveRoute(path, this._routes);
		const callback = this._routes.find((route) => route.path === path)?.callback;

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

	private _resolveRoute(path: string, routes: Array<Route>): PageResponse {
		for (const route of routes) {
			const match = route.matcher(path);

			if (match) {
				const params = (match.params || {}) as Record<string, string>;

				return {
					props: route.props,
					path: route.path,
					name: route.name,
					icon: route.icon,
					hidden: route.hidden,
					hideHeader: route.hideHeader,
					hideNavigation: route.hideNavigation,
					displayEditor: route.displayEditor,
					params,
					child: this._resolveRoute(
						path.slice(route.path.length) || "/",
						route.children || [],
					),
				};
			}
		}

		return null;
	}

	private _sendMessage(message: SongWorkerRequest) {
		this._worker.postMessage(message);
	}

	private _editItem(item: Item) {
		if (isGroup(item)) {
			for (const song of item.songs) {
				this._editItem({ type: "song", song });
			}

			return;
		}

		const original = this._tracks.get(item.song.id);

		if (
			original &&
			// @ts-expect-error
			Object.keys(original).every((key: keyof Song) => {
				// TODO: compare unknown field
				if (key === "unknown") {
					return true;
				}

				return original[key] === item.song[key];
			})
		) {
			this._editedTracks.delete(item.song.id.toString());
			return;
		}

		this._editedTracks.set(item.song.id.toString(), item.song);
	}

	get page() {
		return this._page;
	}

	get path() {
		return this._path;
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
	return item.type === "song";
}

export function isGroup(item: Item): item is Extract<Item, { type: "group" }> {
	return item.type === "group";
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
