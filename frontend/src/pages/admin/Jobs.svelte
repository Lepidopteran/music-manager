<script lang="ts">
	import { getTasks, startTask, stopTask } from "@api/tasks";
	import Event from "@components/Event.svelte";
	import Progress from "@components/Progress.svelte";
	import { eventSource } from "@lib/state/server-events.svelte";
	import { addSourceEventListener } from "@lib/utils/api";
	import { onMount } from "svelte";
	import Button from "@components/Button.svelte";
	import Icon from "@components/Icon.svelte";
	import type { PageComponentProps } from "@lib/state/app.svelte";
	import type { RegistryJob, JobState } from "@bindings/bindings";
	import { getJobs, getJobStates, queueJob } from "@api/jobs";
	import { match, P } from "ts-pattern";
	import { SvelteMap } from "svelte/reactivity";

	interface JobUiState extends JobState {
		current?: bigint;
		total?: bigint;
	}

	let jobs: Array<RegistryJob> = $state([]);
	let jobStates: SvelteMap<string, JobUiState> = $state(new SvelteMap());

	addSourceEventListener(eventSource, "job-event", (event) => {
		match(event)
			.with({ kind: "stateAdded" }, (e) => {
				jobStates.set(e.source, e.state as JobUiState);
			})
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
			.with({ kind: "stateRemoved" }, (e) => jobStates.delete(e.source))
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
	});

	onMount(async () => {
		jobs = await getJobs();

		const states = Object.entries(await getJobStates());
		jobStates = new SvelteMap(states) as SvelteMap<string, JobUiState>;
	});

	let props: PageComponentProps = $props();
</script>

<div>
	<ul class="space-y-2">
		{#each jobs as job}
			{@const state = jobStates
				.values()
				.find((state) => state.jobId === job.id)}
			<li>
				<div
					data-id={job.id}
					class={`bg-base-100 max-w-4xl mx-auto overflow-hidden rounded-theme shadow-md`}
				>
					<div class="flex divide-x-2 divide-base-text/25 w-full">
						{#each Object.entries(job.steps) as [step, description]}
							{@const max =
								state && state.currentStep === Number(step)
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
					<div class="flex justify-between items-center pr-4">
						<div>
							<p class="font-bold text-lg pt-2 pl-2">{job.name}</p>
							<p class="pl-2">{job.description}</p>
						</div>
						<div>
							<Button
								variant="primary"
								disabled={state?.status === "pending"}
								onclick={async () => {
									const state = await queueJob(job.id);
									jobStates.set(state[0], state[1] as JobUiState);
								}}
							>
								{#if state?.status === "inProgress"}
									<Icon name="square-fill" />
								{:else if state?.status === "pending"}
									<Icon name="stopwatch-fill" />
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
