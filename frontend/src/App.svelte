<script lang="ts">
	import Button from "@components/Button.svelte";
	import Icon from "@components/Icon.svelte";
	import Logo from "@components/Logo.svelte";
	import Editor from "@components/music/Editor.svelte";
	import Page from "@components/Page.svelte";
	import { Pane, PaneGroup, PaneResizer } from "paneforge";
	import { prefersReducedMotion } from "svelte/motion";
	import { fade } from "svelte/transition";

	import { onSmallScreen } from "@lib/utils/screen";
	import { AppState } from "@state/app.svelte";

	import type {
		GroupKey,
		PageInfo,
		PageManager,
		ResolvedRoute,
		Route,
		SongGroups,
	} from "@lib/state";

	import {
		GroupManager,
		Router,
		setLegacyAppState,
		setPageManager,
		setSongGroups,
	} from "@lib/state";

	import Jobs from "@pages/admin/Jobs.svelte";
	import Albums from "@pages/Albums.svelte";
	import Directories from "@pages/Directories.svelte";

	let theme = $state("dark");
	let menuOpen = $state(true);

	const app = new AppState();
	setLegacyAppState(app);

	let groupWorkerKeys: Array<GroupKey> = $state([]);
	let trackedGroups: Array<GroupKey> = $state([]);

	const groups: SongGroups = $state({
		track: (group: GroupKey) => groupManager.track(group),
		untrack: (group: GroupKey) => groupManager.untrack(group),
		get tracked() {
			return trackedGroups;
		},

		get inProgress() {
			return Array.from(groupWorkerKeys);
		},
	});

	const groupManager: GroupManager = new GroupManager({
		maxActiveWorkers: 3,
		onTrack: () => trackedGroups = groupManager.tracked,
		onUntrack: () => trackedGroups = groupManager.tracked,
		onRemove: () => trackedGroups = groupManager.tracked,
		onWorkerStart: () => groupWorkerKeys = groupManager.workerKeys,
		onWorkerFinish() {
			Object.assign(groups, groupManager.groups);
			groupWorkerKeys = groupManager.workerKeys;
		},
	});

	setSongGroups(groups);

	$effect(() => {
		if (app.tracks.size !== 0) {
			groupManager.songs = Array.from(app.tracks.values());
		}
	});

	let routes: Array<Route<PageInfo>> = $state([]);
	const router = new Router<PageInfo>([], {
		onRouteAdd(router) {
			routes = router.routes;
		},

		onRouteRemove(router) {
			routes = router.routes;
		},
	});

	let page: ResolvedRoute<PageInfo> | undefined = $state();

	const pageManager: PageManager = $state({
		goTo,
		addPage: (page) => {
			router.addRoute(page);
		},
		removePage: (path) => {
			router.removeRoute(path);
		},
		get current() {
			return page;
		},
	});

	setPageManager(pageManager);

	$effect(() => {
		const { pathname } = window.location;
		if (page !== undefined || !router.hasRoute(pathname)) {
			return;
		}

		goTo(pathname);
	});

	function goTo(path: string, addToHistory = true) {
		const resolvedPage = router.resolve(path);
		if (!resolvedPage) {
			return;
		}

		if (addToHistory) {
			window.history.pushState({}, "", path);
		}

		resolvedPage.metadata?.callback?.();
		page = resolvedPage;
	}

	let editorPane: ReturnType<typeof Pane> | null = $state(null);
	let editorEnabled = $derived(page?.metadata?.displayEditor || false);

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
</script>

<svelte:window onpopstate={() => goTo(window.location.pathname, false)} />

<div class="grid grid-cols-[auto_1fr] grid-rows-[auto_1fr] overflow-hidden h-full">
	<header
		class="col-start-1 col-end-3 row-start-1 h-14 flex gap-4 justify-between items-center px-2 shadow-lg"
		hidden={page?.metadata?.hideHeader}
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
		class={`col-start-1 row-start-2 row-end-3 bg-base-200 transition-all duration-300 shadow-lg z-10 ${
			menuOpen ? "translate-x-0" : "-translate-x-full"
		}`}
	>
		<nav hidden={page?.metadata?.hideNavigation}>
			{#each routes.filter(({ metadata }) =>
				metadata !== undefined && !metadata.hideNavigation
			) as { path, metadata }}
				{@const { name, icon } = metadata!}
				<a
					href={path as string}
					onclick={(event) => {
						event.preventDefault();
						goTo((event.target as HTMLAnchorElement).getAttribute("href") as string);
					}}
					class="font-semibold px-4 flex items-center gap-3 py-2 transition hover:bg-base-600/20 hover:text-primary data-active:text-primary data-active:bg-primary/20"
					data-active={path === page?.path || undefined}
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
				<Page path="/" name="Albums" icon="album_2" displayEditor>
					<Albums />
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
					"shadow-lg shadow-black/25",
					!editorEnabled
						? "opacity-0 duration-500 transition-all pointer-events-none"
						: "",
				]}
			>
				{#if app.selectedItem && editorEnabled}
					<div
						transition:fade={{
							duration: prefersReducedMotion.current ? 0 : 200,
						}}
						class={[
							"h-full",
							onSmallScreen.current ? "rounded-t-theme-xl overflow-hidden" : "",
						]}
					>
						<Editor bind:selectedItem={app.selectedItem} />
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
