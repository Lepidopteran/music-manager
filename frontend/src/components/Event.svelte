<script lang="ts">
	import type { ClassValue } from "svelte/elements";
	import type { TaskEvent } from "@lib/models";

	interface Props {
		event: TaskEvent;
		class?: ClassValue;
	}

	let { event, class: className, ...rest }: Props = $props();

</script>

<div
	{...rest}
	class={`log-entry ${className || ""}`}
	data-kind={event.kind}
>
	<div class="truncate">
		{event.message}
	</div>
	<div>
		{event.timestamp.toLocaleTimeString()}
	</div>
</div>

<style>
	.log-entry {
		--log-bg-color: var(--color-base-200);
		--log-text-color: var(--color-base-text);

		color: var(--log-text-color);
		background-color: var(--log-bg-color);
		padding: 0.5rem;
		border-radius: var(--radius-theme-sm);
	}

	.log-entry[data-kind="error"] {
		--log-bg-color: var(--color-error);
		--log-text-color: var(--color-black);
	}

	.log-entry[data-kind="warning"] {
		--log-bg-color: var(--color-warning);
		--log-text-color: var(--color-black);
	}

	.log-entry[data-kind="info"] {
		--log-bg-color: var(--color-info);
		--log-text-color: var(--color-black);
	}
</style>
