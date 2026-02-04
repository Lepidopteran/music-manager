import type { JobReport } from "@bindings/bindings";
import { fetchJson, fetchText } from "@lib/utils/api";

export async function getJobs(): Promise<JobReport[]> {
	return await fetchJson<JobReport[]>("/api/tasks");
}

export async function startJob(id: string): Promise<void> {
	await fetchText(`/api/job/${id}/start`, { method: "POST" });
}

export async function stopTask(id: string): Promise<void> {
	await fetchText(`/api/job/${id}/stop`, { method: "POST" });
}
