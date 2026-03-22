<script lang="ts">
	import type { HTMLAttributes } from "svelte/elements";

	interface Props extends HTMLAttributes<HTMLDivElement> {}

	let { class: className, ...props }: Props = $props();
</script>

<div class={["bg-primary-500 bg-blend-multiply", className]} {...props}></div>

<style>
	div {
		aspect-ratio: 1 / 1;
		background-position: center;
		background-size: 125%;
		background-color: var(--color-primary-500);
		background-blend-mode: multiply;
		background-image: url("../../assets/missing-album.svg");
	}
</style>
