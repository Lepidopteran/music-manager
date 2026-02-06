import type { JobState, RegistryJob } from "@bindings/bindings";
import { fetchJson, fetchText } from "@lib/utils/api";

export async function getJobs(): Promise<RegistryJob[]> {
	return await fetchJson<RegistryJob[]>("/api/jobs");
}

export async function getJobStates(): Promise<JobState[]> {
	return await fetchJson("/api/jobs/state");
}

export async function queueJob(id: string): Promise<[bigint, JobState]> {
	const stateId = await fetchJson<bigint>(`/api/jobs/${id}/queue`, {
		method: "POST",
	});

	return [
		stateId,
		{
			jobId: id,
			currentStep: 1,
			status: "pending",
			values: {},
		},
	];
}

export async function stopTask(id: string): Promise<void> {
	await fetchText(`/api/jobs/${id}/stop`, { method: "POST" });
}
