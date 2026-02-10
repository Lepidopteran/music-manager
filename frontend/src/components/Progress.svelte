<script lang="ts">
	import type { HTMLProgressAttributes } from "svelte/elements";

	interface Props extends HTMLProgressAttributes {}

	let { value = $bindable(0), max = 100, ...rest }: Props = $props();
</script>

<progress
	{max}
	{value}
	{...rest}
	class={["h-2 rounded-theme-sm bg-base-400 overflow-hidden", rest.class || ""]}
>
	{(Number(value) * 100) / Number(max)}%
</progress>

<style>
	progress {
		-webkit-appearance: none;
		-moz-appearance: none;
		appearance: none;

		&::-webkit-progress-bar,
		&::-moz-progress-bar {
			background-color: var(--color-base-400);
		}

		&::-webkit-progress-value,
		&::-moz-progress-bar {
			background-color: var(--color-primary);
		}
	}
</style>
