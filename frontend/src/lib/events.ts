import type { TaskEvent } from "./models";

declare global {
	interface EventSourceEventMap {
		"task-event": MessageEvent<TaskEvent>;
	}
}
