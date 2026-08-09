<script lang="ts">
	import Image from "@components/Image.svelte";
	import type { ClassValue } from "svelte/elements";
	import MissingCover from "./MissingCover.svelte";

	interface Props {
		src?: string;
		width?: string | number;
		height?: string | number;
		class?: ClassValue;
		onError?: () => void;
	}

	let { src, width, onError, height, class: className }: Props = $props();
	let hasCover = $state(true);
</script>

{#if hasCover}
	<Image
		{src}
		{width}
		{height}
		loading="lazy"
		onError={() => {
			hasCover = false;
			onError?.();
		}}
		class={className}
	/>
{:else}
	<MissingCover class={className} />
{/if}
