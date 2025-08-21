import type { TaskEvent, TaskInfo } from "@lib/models";
import { fetchJson, fetchText } from "@lib/utils/api";

declare global {
	interface EventSourceEventMap {
		"task-event": MessageEvent<TaskEvent>;
	}
}

export async function getTasks(): Promise<TaskInfo[]> {
	return await fetchJson<TaskInfo[]>("/api/tasks");
}

export async function startTask(id: string): Promise<void> {
	await fetchText(`/api/tasks/${id}/start`);
}

export async function stopTask(id: string): Promise<void> {
	await fetchText(`/api/tasks/${id}/stop`);
}

export async function getEventStream(): Promise<EventSource> {
	return new EventSource("/api/tasks/events");
}
