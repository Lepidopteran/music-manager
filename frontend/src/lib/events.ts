import type { JobManagerEvent } from "@bindings/bindings";

declare global {
	interface EventSourceEventMap {
		"job-event": MessageEvent<JobManagerEvent>;
	}
}
