<script lang="ts">
	import type { Album, Song } from "@lib/models";
	import MissingCover from "./MissingCover.svelte";

	import { isAlbum } from "@utils/model-guards";
	import { untrack } from "svelte";

	let failedToLoad = $state(false);
	let image: HTMLImageElement;
	let container: HTMLDivElement;

	interface Props {
		item: Album | Song;
		artType?: "front" | "back";
		imageWidth?: number | null;
		imageHeight?: number | null;
		onLoading?: () => void;
		onError?: () => void;
		onLoad?: () => void;
		alt?: string;
		lazy?: boolean;
		[key: string]: unknown;
	}

	let {
		alt,
		artType = "front",
		imageHeight = $bindable(),
		item,
		lazy = true,
		onError,
		onLoad,
		onLoading,
		imageWidth = $bindable(),
		...rest
	}: Props = $props();

	let src: string = $state("");

	function onload() {
		container.classList.remove("motion-safe:animate-pulse");
		image.classList.remove("opacity-0", "pointer-events-none");

		image.hidden = false;
		failedToLoad = false;

		imageWidth = image.naturalWidth;
		imageHeight = image.naturalHeight;

		if (onLoad) {
			onLoad();
		}
	}

	function onerror(event: Event) {
		event.preventDefault();
		container.classList.remove("motion-safe:animate-pulse");

		image.hidden = true;
		failedToLoad = true;

		imageWidth = null;
		imageHeight = null;

		if (onError) {
			onError();
		}
	}

	$effect(() => {
		container.classList.add("motion-safe:animate-pulse");
		image.classList.add("opacity-0", "pointer-events-none");

		imageHeight = null;
		imageWidth = null;

		src = isAlbum(item)
			? `/api/albums/${encodeURIComponent(untrack(() => item.title))}/cover-art/${artType}.jpg`
			: `/api/songs/${item.id}/cover-art/${artType}.jpg`;

		if (onLoading) {
			onLoading();
		}
	});
</script>

<div
	bind:this={container}
	class={`size-64 bg-base-950/25 overflow-hidden relative motion-safe:animate-pulse ${rest.class || ""}`}
>
	<img
		{onload}
		{onerror}
		alt={alt || `Cover art for ${item.title}`}
		bind:this={image}
		class="size-full object-cover motion-safe:transition duration-300 ease-in-out"
		decoding="async"
		loading={lazy ? "lazy" : "eager"}
		{src}
	/>

	{#if failedToLoad}
		<MissingCover class="text-primary" />
	{/if}
</div>
