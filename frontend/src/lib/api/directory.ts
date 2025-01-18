import { fetchJson } from "../utils/api";
import type { Directory, NewDirectory } from "../models";

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
