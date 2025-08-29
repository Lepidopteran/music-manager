<script lang="ts">
	import Icon from "@components/Icon.svelte";
	import Button from "@components/Button.svelte";
	import Directories from "@pages/Directories.svelte";
	import Home from "@pages/Albums.svelte";
	import Tasks from "@pages/admin/Tasks.svelte";
	import Logo from "./components/Logo.svelte";
	import { AppState, type Page } from "@lib/state/app.svelte";

	let menuOpen = $state(true);
	let theme = $state("dark");

	const routes: Array<Page> = [
		{
			path: "/",
			name: "Albums",
			icon: "music-fill",
			component: Home,
			action() {
				return {
					path: this.path,
					name: this.name,
					pageComponent: Home,
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

	$effect(() => {
		document.documentElement.dataset.theme = theme;
	});
</script>

<svelte:window
	onpopstate={() => app.changePage({ pathname: window.location.pathname })}
	onresize={() => (menuOpen = window.innerWidth > 650)}
	onload={() => (menuOpen = window.innerWidth > 650)}
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
		{#each routes as route}
			<div hidden={route.path !== app.path}>
				<route.component app={app} />
			</div>
		{/each}
	</main>
</div>
