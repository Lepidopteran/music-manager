import type { Song } from "@lib/models";
import type { GroupWorkerRequest, GroupWorkerResponse } from "./group";

/**
 * Utility class that wraps a native {@link https://developer.mozilla.org/en-US/docs/Web/API/Worker|Worker} to add types.
 * @template T - The type of the message that the worker will receive
 * @template O - The type of the message that the worker will send
 */
export class WebWorker<I, O> {
	#worker: Worker | null = null;

	constructor(scriptUrl: URL) {
		this.#worker = new Worker(scriptUrl, {
			type: "module",
		});
	}

	postMessage(message: I): void {
		if (this.#worker) {
			this.#worker.postMessage(message);
		} else {
			console.error("Worker is not initialized.");
		}
	}

	onMessage(callback: (event: MessageEvent<O>) => void): void {
		if (this.#worker) {
			this.#worker.onmessage = callback;
		} else {
			console.error("Worker is not initialized.");
		}
	}

	onError(callback: (error: ErrorEvent) => void): void {
		if (this.#worker) {
			this.#worker.onerror = callback;
		} else {
			console.error("Worker is not initialized.");
		}
	}

	terminate(): void {
		if (this.#worker) {
			this.#worker.terminate();
			this.#worker = null;
		}
	}
}

export const songWorkerUrl = new URL("./song.ts", import.meta.url);
export const groupWorkerUrl = new URL("./group.ts", import.meta.url);

export type { GroupKey, GroupWorkerRequest, GroupWorkerResponse } from "./group";
export class GroupWorker extends WebWorker<GroupWorkerRequest, GroupWorkerResponse> {
	constructor() {
		super(groupWorkerUrl);
	}
}

export type SongWorkerRequest =
	| { type: "initialize"; payload: Song[] }
	| { type: "groupArtists" }
	| { type: "groupAlbums" };

export type SongWorkerResponse =
	| { type: "initialize"; payload: Map<string, Song> }
	| { type: "groupArtists"; payload: Map<string, Array<Song>> }
	| { type: "groupAlbums"; payload: Map<string, Array<Song>> };
