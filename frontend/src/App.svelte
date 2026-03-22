<script lang="ts">
	import Button from "@components/Button.svelte";
	import Icon from "@components/Icon.svelte";
	import Logo from "@components/Logo.svelte";
	import Editor from "@components/music/Editor.svelte";
	import Page from "@components/routing/Page.svelte";
	import { Pane, PaneGroup, PaneResizer } from "paneforge";
	import { prefersReducedMotion } from "svelte/motion";
	import { fade } from "svelte/transition";

	import { onSmallScreen } from "@utils/screen";

	import {
		GroupedSongs,
		type GroupKey,
		type GroupManager,
		type PageMetadata,
		type RouteManager,
		type RouteMetadata,
		setEditedSongs,
		setGroupManager,
		setRouteManager,
		setSelectedSongs,
		setSongs,
	} from "@state";

	import { getSongs } from "@api/song";
	import Redirect from "@components/routing/Redirect.svelte";
	import type { Song } from "@lib/models";
	import { type ResolvedRoute, type Route, Router } from "@lib/router";
	import { GroupWorker } from "@lib/workers";
	import Jobs from "@pages/admin/Jobs.svelte";
	import Directories from "@pages/Directories.svelte";
	import Music from "@pages/Music.svelte";
	import { watch } from "@utils/reactivity/watch.svelte";
	import { onMount } from "svelte";
	import { SvelteMap, SvelteSet } from "svelte/reactivity";

	let theme = $state("dark");
	let menuOpen = $state(true);

	const songs = new SvelteMap<string, Song>();
	setSongs(songs);

	const selectedSongs = new SvelteSet<string>();
	setSelectedSongs(selectedSongs);

	const editedSongs = new SvelteMap<string, Song>();
	setEditedSongs(editedSongs);

	class SongGroupManager implements GroupManager {
		#maxActiveWorkers: number = 3;
		#tracked: SvelteSet<GroupKey> = new SvelteSet();
		#workers: SvelteMap<GroupKey, GroupWorker> = new SvelteMap();
		#groups: SvelteMap<GroupKey, GroupedSongs> = new SvelteMap();

		constructor() {
			watch(() => songs.size, () => this.#update());
		}

		#update() {
			const trackedKeys = this.#tracked.values();

			let groupKey = trackedKeys.next().value;
			while (
				this.#workers.size < this.#maxActiveWorkers && groupKey !== undefined
			) {
				const worker = new GroupWorker();
				worker.onMessage(event => {
					const { grouped, key } = event.data;

					this.#groups.set(key, new GroupedSongs(grouped));
					this.#workers.delete(key);
				});

				worker.postMessage({
					key: groupKey,
					songs: $state.snapshot(songs.values().toArray()),
				});

				this.#workers.set(groupKey, worker);

				groupKey = trackedKeys.next().value;
			}
		}

		track(groupKey: GroupKey) {
			this.#tracked.add(groupKey);
			this.#update();
		}

		untrack(groupKey: GroupKey) {
			this.#tracked.delete(groupKey);
			this.#update();
		}

		get groups() {
			return this.#groups;
		}

		get tracked() {
			return this.#tracked.values().toArray();
		}

		get inProgress() {
			return this.#workers.keys().toArray();
		}
	}

	const groupManager = new SongGroupManager();

	setGroupManager(groupManager);

	class RouteState implements RouteManager {
		#routes: Array<Route<RouteMetadata>> = $state([]);
		#current: ResolvedRoute<RouteMetadata> | undefined = $state();
		router = new Router<RouteMetadata>([], {
			onRoutesUpdated: (router) => this.#routes = router.routes,
		});
		constructor() {
			$effect(() => {
				const { pathname } = window.location;
				if (this.#current !== undefined || !this.router.hasRoute(pathname)) {
					return;
				}

				this.goTo(pathname);
			});
		}

		goTo(path: string, addToHistory?: boolean): void {
			const resolvedRoute = this.router.resolve(path);
			if (!resolvedRoute || resolvedRoute.path === this.#current?.path) {
				return;
			}

			this.#current = resolvedRoute;

			if (resolvedRoute.metadata?.kind === "redirect") {
				return this.goTo(
					resolvedRoute.metadata.redirectTo,
					addToHistory,
				);
			}

			if (addToHistory && resolvedRoute.metadata?.kind === "page") {
				window.history.pushState({}, "", path);
			} else {
				window.history.replaceState({}, "", path);
			}
		}

		get current() {
			return this.#current;
		}

		get routes() {
			return this.#routes;
		}
	}

	const routeState = new RouteState();
	const { current: currentRoute, routes } = $derived(routeState);
	setRouteManager(routeState);
	$inspect(currentRoute);

	let editorPane: ReturnType<typeof Pane> | null = $state(null);
	let editorEnabled = $derived(
		routeState.current?.metadata?.kind === "page"
				&& routeState.current?.metadata?.displayEditor || false,
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

	onMount(async () => {
		for (
			const [id, song] of (await getSongs()).map(song =>
				[song.id, song as Song] as const
			)
		) {
			songs.set(id, song);
		}
	});
</script>

<svelte:window
	onpopstate={() => routeState.goTo(window.location.pathname, false)}
/>

<div class="grid grid-cols-[auto_1fr] grid-rows-[auto_1fr] overflow-hidden h-full">
	<header
		class="col-start-1 col-end-3 row-start-1 h-14 flex gap-4 justify-between items-center px-2 shadow-lg bg-base z-10"
		hidden={currentRoute?.metadata?.kind === "page" && currentRoute?.metadata?.hideHeader}
	>
		<div class="flex items-center gap-2">
			<Button
				color="ghost"
				toggleable={true}
				active={menuOpen}
				onclick={() => (menuOpen = !menuOpen)}
				class="group size-10 sm:hidden"
			>
				<Icon
					name="menu"
					class="text-2xl group-data-[active=true]:text-primary transition"
				/>
			</Button>
			<h1 class="text-2xl font-bold row-start-1 flex gap-2 items-center">
				<Logo class="p-1" /> Muusik
			</h1>
		</div>
		<div class="flex gap-4"></div>
		<div class="flex gap-4"></div>
	</header>
	<aside
		class={`col-start-1 row-start-2 row-end-3 bg-base transition-all duration-300 shadow-lg z-10 ${
			menuOpen ? "translate-x-0" : "-translate-x-full"
		}`}
	>
		<nav
			hidden={currentRoute?.metadata?.kind === "page"
			&& currentRoute?.metadata?.hideNavigation}
		>
			{#each routes.filter(({ metadata }) =>
				metadata !== undefined && metadata.kind === "page" && !metadata.hideNavigation
			) as { path, metadata }}
				{@const { name, icon } = metadata! as PageMetadata}
				<a
					href={path as string}
					onclick={(event) => {
						event.preventDefault();
						routeState.goTo(
							(event.target as HTMLAnchorElement).getAttribute("href") as string,
						);
					}}
					class="font-semibold px-4 flex items-center gap-3 py-2 transition hover:bg-primary/10 hover:text-primary data-active:text-primary data-active:bg-primary/20"
					data-active={path === currentRoute?.path || undefined}
				>
					{#if icon}
						<Icon name={icon} size="1.25em" />
					{/if}
					{name}
				</a>
			{/each}
		</nav>
	</aside>
	<main class="col-start-1 sm:col-start-2 col-end-3 row-start-2 overflow-y-auto h-full inset-shadow-xs shadow-lg inset-shadow-highlight/10">
		<PaneGroup
			direction={onSmallScreen.current ? "vertical" : "horizontal"}
			autoSaveId="mainPane"
		>
			<Pane minSize={onSmallScreen.current ? 0 : 30}>
				<Page path="/music" name="Music" icon="music" displayEditor>
					<Music />
					<Redirect path="/" />
				</Page>
				<Page path="/directories" name="Directories" icon="folder">
					<Directories />
				</Page>
				<Page path="/jobs" name="Jobs" icon="play">
					<Jobs />
				</Page>
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
						class={`transition transform lg:-rotate-90 ${
							editorPane?.isCollapsed() ? "rotate-180 lg:rotate-90" : ""
						}`}
					/>
				</div>
			</PaneResizer>
			<Pane
				collapsible={true}
				minSize={30}
				bind:this={editorPane}
				class={[
					"shadow-lg shadow-black/25 bg-base",
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
						<Editor />
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
