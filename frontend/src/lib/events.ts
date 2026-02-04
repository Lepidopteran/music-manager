import type { JobManagerEvent } from "@bindings/bindings";
import type { TaskEvent } from "@bindings/TaskEvent";

declare global {
	interface EventSourceEventMap {
		"task-event": MessageEvent<TaskEvent>;
		"job-event": MessageEvent<JobManagerEvent>;
	}
}
