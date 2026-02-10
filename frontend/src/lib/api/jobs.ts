import type {
	JobReportsResponse,
	JobStateResponse,
	RegistryJob,
} from "@bindings/bindings";
import { fetchJson, fetchText } from "@lib/utils/api";

export async function getJobs(): Promise<RegistryJob[]> {
	return await fetchJson<RegistryJob[]>("/api/jobs");
}

export async function getJobStates(): Promise<JobStateResponse> {
	return await fetchJson("/api/jobs/state");
}

export async function queueJob(id: string): Promise<[string, JobState]> {
	const stateId = await fetchJson<string>(`/api/jobs/${id}/queue`, {
export async function getJobReports(): Promise<JobReportsResponse> {
	return await fetchJson("/api/jobs/reports");
}

export async function getJobQueue(): Promise<string[]> {
	return await fetchJson<string[]>("/api/jobs/queue");
}

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

export async function cancelJob(stateId: string): Promise<void> {
	await fetchText(`/api/jobs/state/${stateId}/cancel`, { method: "POST" });
}
