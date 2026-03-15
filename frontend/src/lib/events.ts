import type { JobManagerEvent } from "src/bindings/bindings";

declare global {
	interface EventSourceEventMap {
		"job-event": MessageEvent<JobManagerEvent>;
	}
}
