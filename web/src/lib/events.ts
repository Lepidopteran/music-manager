import type { JobManagerEvent } from "@lib/bindings/bindings";

declare global {
	interface EventSourceEventMap {
		"job-event": MessageEvent<JobManagerEvent>;
	}
}
