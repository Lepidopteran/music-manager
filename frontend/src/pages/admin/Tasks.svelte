<script lang="ts">
	import { getTasks } from "@api/tasks";
	import Event from "@components/Event.svelte";
	import Progress from "@components/Progress.svelte";
	import type { TaskEvent, TaskInfo } from "@lib/models";
	import { taskEventsSource } from "@lib/state/event-sources.svelte";
	import { addSourceEventListener } from "@lib/utils/api";
	import { prefersReducedMotion } from "svelte/motion";
	import { slide } from "svelte/transition";
	import { onMount } from "svelte";
	import Button from "@components/Button.svelte";
	import Icon from "@iconify/svelte";

	let tasks: Array<TaskInfo> = $state([]);

	const events: Array<TaskEvent> = $state([]);
	addSourceEventListener(taskEventsSource, "task-event", (event) => {
		events.push(event);
	});

	$effect(() => {
		document
			.querySelector(`[data-log-index=\"${events.length - 1}\"]`)
			?.scrollIntoView({ behavior: "instant", block: "center" });
	});

	onMount(async () => {
		tasks = await getTasks();
	});
</script>

<div>
	<ul class="space-y-2">
		{#each tasks as task}
			{@const logs = events.filter((e) => e.source === task.id)}
			<li>
				<div
					data-id={task.id}
					class="bg-base-100 max-w-4xl mx-auto overflow-hidden rounded-theme shadow-md"
				>
					<div class="flex divide-x-2 divide-base-text/25 w-full">
						{#each { length: task.steps }, step}
							{@const lastProgress = logs.findLast(
								(e) => e.kind === "progress",
							)}
							<Progress
								value={lastProgress?.current ?? 0}
								max={lastProgress?.total ?? 100}
								class="rounded-none! w-full"
							/>
						{/each}
					</div>
					<p class="font-bold text-lg pt-2 pl-2">{task.name}</p>
					<p class="pl-2">{task.description}</p>
					<details class="space-y-2 mt-2 px-2">
						<summary
							class="flex items-center gap-2 text-sm font-bold text-base-950/50 overflow-hidden"
						>
							Event Logs <span
								transition:slide={{
									duration: prefersReducedMotion ? 0 : 200,
									axis: "y",
								}}
								class="text-base-text/25 truncate w-32"
								>{logs.at(-1)?.message}</span
							>
						</summary>
						<div class="space-y-2 mt-2 p-2 overflow-y-auto h-48">
							{#each logs as event, index}
								<Event data-log-index={index} {event} class="truncate" />
							{/each}
						</div>
					</details>
				</div>
			</li>
		{/each}
	</ul>
</div>
