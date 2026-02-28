<script lang="ts">
	import {
		cancelJob,
		getJobQueueOrder,
		getJobReports,
		getJobs,
		getJobStates,
		queueJob,
	} from "@api/jobs";
	import type {
		JobExecutionReport,
		JobState,
		RegistryJob,
	} from "@bindings/bindings";
	import Button from "@components/Button.svelte";
	import Icon from "@components/Icon.svelte";
	import Progress from "@components/Progress.svelte";
	import { addSourceEventListener } from "@lib/utils/api";
	import { onMount } from "svelte";
	import { SvelteMap } from "svelte/reactivity";
	import { match, P } from "ts-pattern";

	interface JobUiState extends JobState {
		current?: bigint;
		total?: bigint;
	}

	let jobs: Array<RegistryJob> = $state([]);
	let jobStates: SvelteMap<string, JobUiState> = $state(new SvelteMap());
	let jobQueue: Array<string> = $state([]);
	let jobReports: SvelteMap<string, JobExecutionReport> = $state(
		new SvelteMap(),
	);

	onMount(async () => {
		jobs = await getJobs();

		jobStates = new SvelteMap(
			Object.entries(await getJobStates()),
		) as SvelteMap<string, JobUiState>;

		jobQueue = await getJobQueueOrder();
		jobReports = new SvelteMap(Object.entries(await getJobReports()));
	});
</script>

<svelte:window
	onload={() =>
	addSourceEventListener(
		new EventSource("/api/events"),
		"job-event",
		(event) => {
			match(event)
				.with(
					{ kind: "stateAdded" },
					(e) => jobStates.set(e.source, e.state as JobUiState),
				)
				.with({ kind: "stateUpdated" }, (e) => {
					const previousState = jobStates.get(e.source);
					if (!previousState) {
						return;
					}

					jobStates.set(e.source, {
						...previousState,
						...e.state,
					});
				})
				.with(
					{ kind: "reportUpdated" },
					(e) => jobReports.set(e.jobId, e.report),
				)
				.with({ kind: "stateRemoved" }, (e) => jobStates.delete(e.source))
				.with({ kind: "orderUpdated" }, (e) => (jobQueue = e.queue))
				.with({ kind: "progress" }, (e) => {
					const previousState = jobStates.get(e.source);
					if (!previousState) {
						return;
					}

					jobStates.set(e.source, {
						...previousState,
						current: e.current,
						total: e.total,
					});
				})
				.otherwise(() => {});
		},
	)}
/>

<div class="p-4 max-w-4xl">
	<ul class="space-y-2">
		{#each jobs as job}
			{@const [key, state] = jobStates.entries().find(([_, state]) => state.jobId === job.id)
			?? []}
			<li>
				<div
					data-id={job.id}
					class={["bg-base-100 overflow-hidden rounded-theme shadow-md"]}
				>
					<div class="flex divide-x-2 divide-base-text/25 w-full">
						{#each Object.entries(job.steps) as [step, description]}
							{@const max = state && state.currentStep === Number(step)
							? Number(state.total) || 100
							: 100}

							{@const value = state
							? state.currentStep === Number(step)
								? Number(state.current) || 0
								: state.currentStep > Number(step)
								? 100
								: 0
							: 0}

							<Progress
								{max}
								{value}
								class={[
									"rounded-none! w-full",
									state?.status !== "inProgress" && "opacity-50",
								]}
							/>
						{/each}
					</div>
					<div class="flex justify-between items-center pl-2 pr-3">
						<div>
							<p class="font-bold text-lg">{job.name}</p>
							<p>{job.description}</p>
						</div>
						<div>
							<Button
								variant="primary"
								onclick={async () => {
									if (
										state?.status === "inProgress"
										|| state?.status === "pending"
									) {
										await cancelJob(key!);
										return;
									}

									const id = await queueJob(job.id);

									jobQueue = [...jobQueue, id];
									jobStates.set(id, {
										jobId: job.id,
										status: "pending",
										currentStep: 1,
										values: {},
									});
								}}
							>
								{#if state?.status === "inProgress"}
									<Icon name="square-fill" />
								{:else if state?.status === "pending"}
									<Icon name="close-fill" />
								{:else}
									<Icon name="play-fill" />
								{/if}
							</Button>
						</div>
					</div>
				</div>
			</li>
		{/each}
	</ul>
</div>
