<script lang="ts">
	import Button from "@components/Button.svelte";
	import Directories from "@pages/Directories.svelte";
	import Albums from "@pages/Albums.svelte";
	import Tasks from "@pages/admin/Tasks.svelte";
	import Logo from "./components/Logo.svelte";
	import Icon from "@components/Icon.svelte";

	import { PaneGroup, Pane, PaneResizer } from "paneforge";

	import { AppState, type Page } from "@lib/state/app.svelte";
	import Editor from "@components/music/Editor.svelte";
	import { fade } from "svelte/transition";
	import { MediaQuery } from "svelte/reactivity";
	import { prefersReducedMotion } from "svelte/motion";

	let theme = $state("dark");
	let onMobile = new MediaQuery("(max-width: 650px)");
	let menuOpen = $state(true);

	const routes: Array<Page> = [
		{
			path: "/",
			name: "Albums",
			icon: "album-2-fill",
			component: Albums,
			action() {
				return {
					path: this.path,
					name: this.name,
					callback: (app) => {
						if (!app.autoOrganizeAlbums) {
							app.autoOrganizeAlbums = true;
						}
					}
				};
			},
		},
		{
			path: "/directories",
			name: "Directories",
			icon: "folder-fill",
			component: Directories,
			action() {
				return {
					path: this.path,
					name: this.name,
				};
			},
		},
		{
			path: "/tasks",
			name: "Tasks",
			icon: "play-fill",
			component: Tasks,
			action() {
				return {
					path: this.path,
					name: this.name,
				};
			},
		},
	];

	const app = new AppState(routes);

	async function handleNavitionClick(event: MouseEvent) {
		const { target } = event;

		if (!(target instanceof HTMLAnchorElement)) {
			return;
		}

		const path = target.getAttribute("href") as string;
		event.preventDefault();

		window.history.pushState({}, "", path);
		await app.changePage(path);
	}

	app.changePage({ pathname: window.location.pathname });

	let editorPane: ReturnType<typeof Pane> | null = $state(null);
	let editorEnabled = $derived(["/albums", "/songs", "/"].includes(app.path));

	$effect(() => {
		document.documentElement.dataset.theme = theme;
		if (!editorPane) {
			return;
		}

		menuOpen = !onMobile.current;

		if (!editorEnabled) {
			editorPane.collapse();
		} else {
			editorPane.expand();
		}
	});
</script>

<svelte:window
	onpopstate={() => app.changePage({ pathname: window.location.pathname })}
/>

<div
	class="grid grid-cols-[auto_1fr] grid-rows-[auto_1fr] overflow-hidden h-full"
>
	<header
		class="col-start-1 col-end-3 row-start-1 h-14 flex gap-4 justify-between items-center px-2 shadow-lg"
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
					name="menu-line"
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
		class={`col-start-1 row-start-2 row-end-3 bg-base-200 transition-all duration-300 shadow-lg z-10 ${menuOpen ? "translate-x-0" : "-translate-x-full"}`}
	>
		<nav>
			{#each routes as route}
				{@const icon = route.icon}
				<a
					href={route.path as string}
					onclick={handleNavitionClick}
					class="font-semibold px-4 flex items-center gap-3 py-2 transition hover:bg-base-600/20 hover:text-primary data-active:text-primary data-active:bg-primary/20"
					data-active={route.path === app.path || undefined}
				>
					{#if icon}
						<Icon name={icon} size="1.25em" />
					{/if}
					{route.name}
				</a>
			{/each}
		</nav>
	</aside>
	<main
		class="col-start-1 sm:col-start-2 col-end-3 row-start-2 overflow-y-auto h-full inset-shadow-xs shadow-lg inset-shadow-highlight/10"
	>
		<PaneGroup
			direction={onMobile.current ? "vertical" : "horizontal"}
			autoSaveId="mainPane"
		>
			<Pane minSize={onMobile.current ? 0 : 30}>
				{#each routes as route}
					<div class="h-full" hidden={route.path !== app.path}>
						<route.component {app} />
					</div>
				{/each}
			</Pane>
			<PaneResizer disabled={!editorEnabled}>
				<div
					class={[
						"size-full absolute z-[1] left-0 top-0",
						onMobile.current ? "pb-32" : "p-3",
						!editorEnabled && "pointer-events-none",
						editorPane?.isCollapsed()
							? onMobile.current
								? "translate-y-full"
								: "-translate-x-full"
							: "",
					]}
				></div>
				<div
					class={`max-lg:px-1 lg:py-1 absolute top-1/2 z-[1] -translate-y-1/2 left-1/2 -translate-x-1/2 rounded-theme bg-primary/50 inset-shadow-sm inset-shadow-white/25 backdrop-blur-lg transition-opacity ${editorPane?.isCollapsed() ? "opacity-0" : ""}`}
				>
					<Icon
						name="up-line"
						size="1.25em"
						class={`transition transform lg:-rotate-90 ${editorPane?.isCollapsed() ? "rotate-180 lg:rotate-90" : ""}`}
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
							onMobile.current ? "rounded-t-theme-xl overflow-hidden" : "",
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
