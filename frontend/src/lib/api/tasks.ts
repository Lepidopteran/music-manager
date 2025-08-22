import type { TaskEvent, TaskInfo } from "@lib/models";
import { fetchJson, fetchText } from "@lib/utils/api";

export async function getTasks(): Promise<TaskInfo[]> {
	return await fetchJson<TaskInfo[]>("/api/tasks");
}

export async function startTask(id: string): Promise<void> {
	await fetchText(`/api/tasks/${id}/start`);
}

export async function stopTask(id: string): Promise<void> {
	await fetchText(`/api/tasks/${id}/stop`);
}
