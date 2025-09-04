import type { Directory } from "@bindings/Directory";
import type { NewDirectory } from "@bindings/NewDirectory";
import { fetchJson } from "@utils/api";

/**
 * Fetch the list of directories.
 * @returns Promise resolving to an array of directories.
 */
export async function getDirectories(): Promise<Array<Directory>> {
	return await fetchJson<Array<Directory>>("/api/directories/");
}

/**
 * Create a new directory via the API.
 * @param directory The directory to create.
 * @returns Promise resolving to the created directory.
 */
export async function createDirectory(
	directory: NewDirectory,
): Promise<Directory | null> {
	return await fetchJson<Directory | null>("/api/directories/", {
		method: "POST",
		body: JSON.stringify(directory),
	});
}

/**
 * Delete a directory via the API.
 * @param name The name of the directory to delete.
 * @returns Promise resolving when the directory is deleted.
 */
export async function deleteDirectory(name: string): Promise<void> {
	await fetchJson<void>(`/api/directories/${name}`, {
		method: "DELETE",
	});
}

/**
 * Get a list of folders in specific system directory.
 * @param name The name of the directory.
 * @returns Promise resolving to an array of files.
 */
export async function getServerDirectoryFolders(path: string): Promise<Array<string>> {
	return await fetchJson<Array<string>>(`/api/directories/filesystem//${path}`);
}
