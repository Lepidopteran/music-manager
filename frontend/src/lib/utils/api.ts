// Interface for fetch errors
export interface FetchError extends Error {
	status: number;
	statusText: string;
	body: string;
}

// Utility function to handle fetch requests
export async function fetchJson<T>(
	url: string,
	options?: RequestInit,
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
		return await response.json();
	} catch (error) {
		console.error(`Failed to fetch: ${url}`, error);
		throw error;
	}
}
