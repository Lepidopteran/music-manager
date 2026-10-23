<script lang="ts">
	import "./layout.css";
	import favicon from "#assets/favicon.svg";

	import type { Icon as IconType } from "virtual:icons";

	import { page, page as currentPage } from "$app/state";

	import Icon from "#components/Icon.svelte";
	import Logo from "#components/Logo.svelte";
	import Editor from "#components/music/Editor.svelte";
	import PrevPage from "#components/navigation/PrevPage.svelte";
	import { app } from "#lib/stores/app.svelte.ts";
	import { onSmallScreen } from "#utils/screen";
	import { Pane, PaneGroup, PaneResizer } from "paneforge";
	import { prefersReducedMotion } from "svelte/motion";
	import { fade, scale } from "svelte/transition";

	let theme = $state("dark");
	let menuOpen = $state(true);

	const topNavigation: Page[] = [
		{
			name: "Albums",
			displayEditor: true,
			path: "/albums",
			icon: "album_2",
		},
	];

	const bottomNavigation: Page[] = [
		{
			name: "Settings",
			path: "/settings",
			icon: "settings_3",
		},
	];

	let editorPane: ReturnType<typeof Pane> | null = $state(null);
	let editorEnabled = $derived(
		topNavigation.find(item => currentPage.url.pathname.startsWith(item.path))
			?.displayEditor,
	);

	$effect(() => {
		document.documentElement.dataset.theme = theme;
		if (!editorPane) {
			return;
		}

		menuOpen = !onSmallScreen.current;

		if (!editorEnabled) {
			editorPane.collapse();
		} else {
			editorPane.expand();
		}
	});

	let { children } = $props();

	type Page = {
		name: string;
		path: string;
		hideHeader?: boolean;
		hideNavigation?: boolean;
		displayEditor?: boolean;
		icon: IconType;
	};
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>
<div class="p-1 grid grid-cols-[auto_1fr] grid-rows-[auto_1fr] overflow-hidden h-full gap-2">
	<header class="col-start-2 col-end-3 row-start-1 h-14 flex gap-4 justify-between items-center px-4 shadow-lg bg-base border-b border-base-content/10 rounded-b-theme inset-shadow-xs inset-shadow-highlight/25 z-10">
		<div class="flex items-center gap-2"></div>
		<div class="flex gap-4"></div>
		<div class="flex gap-4"></div>
	</header>

	<aside
		class={[
			"row-start-1 row-end-3 bg-base transition-all duration-300 shadow-lg z-10 h-full overflow-hidden rounded-r-theme pr-2 inset-shadow-highlight/50 inset-shadow-xs flex flex-col",
			menuOpen ? "translate-x-0" : "-translate-x-full",
		]}
	>
		<div class="flex items-center gap-2 p-2">
			<h1 class="text-2xl font-semibold row-start-1 flex gap-2 items-center">
				<Logo class="p-1" /> Muusik
			</h1>
		</div>
		{#snippet menuItem(item: Page)}
			{@const { name, path, icon } = item}
			<li>
				<a
					href={path as string}
					class={[
						"font-semibold px-4 flex border-base-content/10 border border-l-0 items-center gap-3 py-2 transition hover:bg-primary/10 inset-shadow-sm hover:text-primary rounded-r-theme-xl",
						currentPage.url.pathname === path || currentPage.url.pathname.startsWith(
								path,
							)
							? "text-primary bg-primary/20 inset-shadow-shade/50"
							: "inset-shadow-highlight/25",
					]}
				>
					<Icon name={icon} size="1.25em" />
					{name}
				</a>
			</li>
		{/snippet}
		<nav class="flex flex-col gap-2">
			<ul>
				{#each topNavigation as page}
					{@render menuItem(page)}
				{/each}
			</ul>
		</nav>
		<nav class="h-full py-4 flex flex-col">
			<ul class="mt-auto">
				{#each bottomNavigation as page}
					{@render menuItem(page)}
				{/each}
			</ul>
		</nav>
	</aside>
	<main class="col-start-1 sm:col-start-2 col-end-3 row-start-2 overflow-y-auto h-full shadow-lg inset-shadow-xs inset-shadow-highlight/10 rounded-theme shadow-shade/25">
		<PaneGroup
			direction={onSmallScreen.current ? "vertical" : "horizontal"}
			autoSaveId="mainPane"
		>
			<Pane minSize={onSmallScreen.current ? 0 : 30}>
				{#if page.url.pathname.split("/").filter(Boolean).length > 1}
					<div
						class="inline-block p-4"
						in:scale={{ duration: prefersReducedMotion.current ? 0 : 200 }}
					>
						<PrevPage />
					</div>
				{/if}

				{@render children()}
			</Pane>
			<PaneResizer disabled={!editorEnabled}>
				<div
					class={[
						"size-full absolute z-1 left-0 top-0",
						onSmallScreen.current ? "pb-32" : "p-3",
						!editorEnabled && "pointer-events-none",
						editorPane?.isCollapsed()
							? onSmallScreen.current
								? "translate-y-full"
								: "-translate-x-full"
							: "",
					]}
				>
				</div>
				<div
					class={[
						"max-lg:px-1 lg:py-1 absolute top-1/2 z-1 -translate-y-1/2 left-1/2 -translate-x-1/2 rounded-theme bg-primary/50 inset-shadow-sm inset-shadow-white/25 backdrop-blur-lg transition-opacity",
						editorPane?.isCollapsed() && "opacity-0",
					]}
				>
					<Icon
						name="up"
						size="1.25em"
						class={[
							"transition transform md:-rotate-90",
							editorPane?.isCollapsed() ? "rotate-180 md:rotate-90" : "",
						]}
					/>
				</div>
			</PaneResizer>
			<Pane
				collapsible={true}
				minSize={30}
				bind:this={editorPane}
				class={[
					"shadow-lg shadow-black/25 bg-base inset-shadow-xs inset-shadow-highlight/25 rounded-theme rounded-br-none",
					!editorEnabled
						? "opacity-0 duration-500 transition-all pointer-events-none"
						: "",
				]}
			>
				{#if editorEnabled}
					<div
						transition:fade={{
							duration: prefersReducedMotion.current ? 0 : 200,
						}}
						class={[
							"h-full",
							onSmallScreen.current ? "rounded-t-theme-xl overflow-hidden" : "",
						]}
					>
						{#if app.selectedSongs.size > 0}
							<Editor />
						{:else}
							<div class="h-full flex items-center justify-center">
								<p class="text-sm text-current/50">Select a item to edit...</p>
							</div>
						{/if}
					</div>
				{/if}
			</Pane>
		</PaneGroup>
	</main>
</div>

<style>
	:global {
		[data-pane-resizer] {
			display: flex;
			position: relative;
			align-items: center;
			justify-content: center;
		}
	}
</style>
