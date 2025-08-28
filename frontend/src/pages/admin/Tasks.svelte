<script lang="ts">
	import { getTasks, startTask, stopTask } from "@api/tasks";
	import Event from "@components/Event.svelte";
	import Progress from "@components/Progress.svelte";
	import type { TaskEvent, TaskInfo } from "@lib/models";
	import { eventSource } from "@lib/state/server-events.svelte";
	import { addSourceEventListener } from "@lib/utils/api";
	import { onMount } from "svelte";
	import Button from "@components/Button.svelte";
	import Icon from "@components/Icon.svelte";

	let tasks: Array<TaskInfo> = $state([]);
	const events: Array<[TaskEvent, number]> = $state([]);

	addSourceEventListener(eventSource, "task-event", (event) => {
		events.push([event, events.length]);

		tasks = tasks.map((task) => {
			if (task.id !== event.source) {
				return task;
			}

			switch (event.kind) {
				case "stop":
					return { ...task, status: "stopped" };
				case "progress":
				case "start":
					return { ...task, status: "running" };
				case "complete":
					return { ...task, status: "idle" };
			}

			return task;
		});
	});

	$effect(() => {
		const element = document.querySelector(
			`[data-log-index=\"${events.length - 1}\"]`,
		);

		$inspect(element, events.length - 1);
		element?.scrollIntoView({ behavior: "instant", block: "center" });
	});

	onMount(async () => {
		tasks = await getTasks();
	});
</script>

<div>
	<ul class="space-y-2">
		{#each tasks as task}
			{@const logs = events.filter(([event, _]) => event.source === task.id)}
			<li>
				<div
					data-id={task.id}
					class={`bg-base-100 max-w-4xl mx-auto overflow-hidden rounded-theme shadow-md`}
				>
					<div class="flex divide-x-2 divide-base-text/25 w-full">
						{#each { length: task.steps }, step}
							{@const [lastProgress, _] = logs.findLast(
								([event, _]) =>
									(event.kind === "progress" && event.step === step + 1) ||
									(event.kind === "progress" && !event.step),
							) ?? [null, null]}
							<Progress
								value={task.status === "running"
									? (lastProgress?.current ?? 0)
									: 0}
								max={lastProgress?.total ?? 100}
								class="rounded-none! w-full"
							/>
						{/each}
					</div>
					<div class="flex justify-between items-center pr-4">
						<div>
							<p class="font-bold text-lg pt-2 pl-2">{task.name}</p>
							<p class="pl-2">{task.description}</p>
						</div>
						<div>
							<Button
								aria-label={task.status === "running"
									? `Stop ${task.name}`
									: `Start ${task.name}`}
								variant="primary"
								onclick={async () => {
									if (task.status !== "running") {
										startTask(task.id);
									} else {
										stopTask(task.id);
									}
								}}
							>
								{#if task.status === "running"}
									<Icon name="stop-fill" />
								{:else}
									<Icon name="play-fill" />
								{/if}
							</Button>
						</div>
					</div>
					<details class="space-y-2 mt-2 px-2">
						<summary
							class="flex items-center gap-2 text-sm font-bold text-base-950/50 cursor-pointer select-none overflow-hidden"
						>
							Event Logs <span class="text-base-text/25 truncate w-32"
								>{logs.at(-1)?.[0].message}</span
							>
						</summary>
						<div class="space-y-2 mt-2 p-2 overflow-y-auto h-48">
							{#each logs as event}
								<Event data-log-index={event[1]} event={event[0]} />
							{/each}
						</div>
					</details>
				</div>
			</li>
		{/each}
	</ul>
</div>
