// Interface for fetch errors
export interface FetchError extends Error {
	status: number;
	statusText: string;
	body: string;
}

export interface FetchOptions extends RequestInit {
	method: "GET" | "POST" | "PUT" | "DELETE";
}

export async function fetchJson<T>(
	url: string,
	options?: FetchOptions,
): Promise<T> {
	try {
		const response = await fetch(url, {
			method: "GET",
			headers: {
				"Content-Type": "application/json",
			},
			...options,
		});

		if (!response.ok) {
			throw Object.assign(new Error("JSON fetch failed..."), {
				status: response.status,
				statusText: response.statusText,
				body: await response.text(),
			}) as FetchError;
		}

		const json: T = await response.json();
		if (json && typeof json === "object") {
			for (const [key, value] of Object.entries(json)) {
				if (typeof value === "string") {
					const date = new Date(value);
					(json as Record<string, unknown>)[key] = isNaN(date.getTime())
						? value
						: date;
				}
			}
		}

		return json;
	} catch (error) {
		console.error(`Failed to fetch: ${url}`, error);
		throw error;
	}
}

export async function fetchText(
	url: string,
	options?: FetchOptions,
): Promise<string> {
	try {
		const response = await fetch(url, {
			method: "GET",
			...options,
		});

		if (!response.ok) {
			throw Object.assign(new Error("Text fetch failed..."), {
				status: response.status,
				statusText: response.statusText,
				body: await response.text(),
			}) as FetchError;
		}

		return await response.text();
	} catch (error) {
		console.error(`Failed to fetch: ${url}`, error);
		throw error;
	}
}

type MessageEventData<K extends keyof EventSourceEventMap> =
	EventSourceEventMap[K] extends MessageEvent<infer T> ? T : never;

/**
 * Adds an event listener to an EventSource for a specific event type.
 *
 * @param source - The EventSource to listen to.
 * @param event - The name of the event to listen for.
 * @param handler - A callback function that handles the parsed data from the event.
 */
export function addSourceEventListener<K extends keyof EventSourceEventMap>(
	source: EventSource,
	event: K,
	handler: (data: MessageEventData<K>) => void,
) {
	source.addEventListener(event, (rawEvent) => {
		if (rawEvent instanceof MessageEvent) {
			try {
				const parsed: MessageEventData<K> = JSON.parse(
					rawEvent.data,
					(_, value) => {
						const date = new Date(value);
						if (value && typeof value === "string" && !isNaN(date.getTime())) {
							return date;
						}

						return value;
					},
				);
				handler(parsed);
			} catch (error) {
				console.error(
					`Failed to parse EventSource message for event "${event}":`,
					error,
				);
			}
		}
	});
}
