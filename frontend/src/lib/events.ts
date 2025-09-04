import type { TaskEvent } from "@bindings/TaskEvent";

declare global {
	interface EventSourceEventMap {
		"task-event": MessageEvent<TaskEvent>;
	}
}
