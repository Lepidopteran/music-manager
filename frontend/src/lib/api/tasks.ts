import type { TaskInfo } from "@bindings/TaskInfo";
import { fetchJson, fetchText } from "@lib/utils/api";

export async function getTasks(): Promise<TaskInfo[]> {
	return await fetchJson<TaskInfo[]>("/api/tasks");
}

export async function startTask(id: string): Promise<void> {
	await fetchText(`/api/tasks/${id}/start`, { method: "POST" });
}

export async function stopTask(id: string): Promise<void> {
	await fetchText(`/api/tasks/${id}/stop`, { method: "POST" });
}
