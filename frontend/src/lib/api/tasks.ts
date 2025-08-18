import type { TaskEvent, TaskInfo } from "@lib/models";
import { fetchJson } from "@lib/utils/api";

declare global {
	interface EventSourceEventMap {
		"task-event": MessageEvent<TaskEvent>;
	}
}

export async function getTasks(): Promise<TaskInfo[]> {
	return await fetchJson<TaskInfo[]>("/api/tasks");
}

export async function getTask(id: string): Promise<TaskInfo> {
	return await fetchJson<TaskInfo>(`/api/tasks/${id}`);
}

export async function startTask(id: string): Promise<void> {
	await fetchJson<void>(`/api/tasks/${id}`);
}

export async function stopTask(id: string): Promise<void> {
	await fetchJson<void>(`/api/tasks/${id}`);
}

export async function getEventStream(): Promise<EventSource> {
	return new EventSource("/api/tasks/events");
}
