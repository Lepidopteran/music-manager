<script lang="ts">
	import Explorer from "@components/music/Explorer.svelte";
	import Editor from "@components/music/Editor.svelte";

	import type { Album, Song } from "@lib/models";
	import { getAlbums } from "@api/album";
	import type { PageComponentProps } from "@lib/state/app.svelte";

	let selectedItem: [string, Song[]] | Song | null = $state(null);

	let { app }: PageComponentProps = $props();
</script>

<div class="grid md:grid-cols-2 h-svh">
	<Explorer items={app.albums} onItemChange={(item) => (selectedItem = item)} ></Explorer>
	<Editor bind:selectedItem={selectedItem} />
</div>
