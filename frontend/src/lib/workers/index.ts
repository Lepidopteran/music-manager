import type { GroupWorkerRequest, GroupWorkerResponse } from "./group";
import GroupWorker from "./group?worker";

/**
 * Utility class that wraps a native {@link https://developer.mozilla.org/en-US/docs/Web/API/Worker|Worker} to add types.
 * @template T - The type of the message that the worker will receive
 * @template O - The type of the message that the worker will send
 */

export class WebWorker<I, O> {
	#worker: Worker;

	constructor(worker: Worker) {
		this.#worker = worker;
	}

	postMessage(message: I): void {
		this.#worker.postMessage(message);
	}

	onMessage(callback: (event: MessageEvent<O>) => void): void {
		this.#worker.onmessage = callback;
	}

	onError(callback: (error: ErrorEvent) => void): void {
		this.#worker.onerror = callback;
	}

	terminate(): void {
		this.#worker.terminate();
	}
}

export type { GroupWorkerRequest, GroupWorkerResponse } from "./group";
export class GroupWebWorker extends WebWorker<GroupWorkerRequest, GroupWorkerResponse> {
	constructor() {
		super(new GroupWorker());
	}
}
