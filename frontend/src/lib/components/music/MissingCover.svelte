<script lang="ts">
	import type { HTMLAttributes } from "svelte/elements";

	interface Props extends HTMLAttributes<HTMLDivElement> {}

	let { class: className, ...props }: Props = $props();
</script>

<div class={className} {...props}></div>

<style>
	div {
		aspect-ratio: 1 / 1;
		background-position: center;
		background-size: 125%;
		background-color: var(--color-secondary);
		background-blend-mode: multiply;
		background-image: url("../../assets/missing-album.svg");
	}
</style>
