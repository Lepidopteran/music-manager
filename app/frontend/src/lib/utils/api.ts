// Utility function to handle fetch requests
export async function fetchJson<T>(
	url: string,
	options?: RequestInit,
): Promise<T> {
	const defaultOptions = {
		method: "GET",
		headers: {
			"Content-Type": "application/json",
		},
	};
	try {
		const response = await fetch(url, { ...defaultOptions, ...options });
		if (!response.ok) {
			throw new Error(`HTTP Error: ${response.status} ${response.statusText}`);
		}
		return await response.json();
	} catch (error) {
		console.error(`Error in fetchJson: ${url}`, error);
		throw error;
	}
}
