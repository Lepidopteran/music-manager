<script lang="ts">
	import { watch } from "@utils/reactivity/watch.svelte";
	import type { ClassValue } from "svelte/elements";

	let failedToLoad = $state(false);
	let isLoading = $state(true);
	let isComplete = $state(false);

	interface Props {
		src: string;
		alt?: string;
		decoding?: "async" | "auto" | "sync";
		loading?: "lazy" | "eager";
		class?: ClassValue;
		width?: number | string | null;
		height?: number | string | null;
		imageWidth?: number | null;
		imageHeight?: number | null;
		onLoading?: () => void;
		onError?: () => void;
		onLoad?: () => void;
	}

	let {
		alt,
		src,
		imageHeight = $bindable(),
		imageWidth = $bindable(),
		class: className,
		onError,
		onLoad,
		onLoading,
		width,
		height,
		decoding,
		loading,
	}: Props = $props();

	watch(() => src, () => {
		failedToLoad = false;
		isComplete = false;
		isLoading = true;
		onLoading?.();
	});
</script>

<img
	onload={() => {
		isLoading = false;
		onLoading?.();
		onLoad?.();
	}}
	onerror={() => {
		failedToLoad = true;
		isLoading = false;
		onError?.();
	}}
	class={[
		"motion-safe:transition duration-300 ease-in-out",
		isLoading && "motion-safe:animate-pulse bg-base-content/25",
		className,
	]}
	bind:naturalWidth={imageWidth}
	bind:naturalHeight={imageHeight}
	hidden={failedToLoad}
	{alt}
	{width}
	{height}
	{decoding}
	{loading}
	{src}
/>
