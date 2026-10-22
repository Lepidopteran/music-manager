import type { JobManagerEvent } from "#lib/bindings/bindings.ts";

declare global {
	interface EventSourceEventMap {
		"job-event": MessageEvent<JobManagerEvent>;
	}
}
