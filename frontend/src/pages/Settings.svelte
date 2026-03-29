<script lang="ts">
	import Button from "@components/Button.svelte";
	import Icon from "@components/Icon.svelte";
	import Page from "@components/routing/Page.svelte";
	import { buildPath } from "@lib/router";
	import { routeManager } from "@state";
	import { prefersReducedMotion } from "svelte/motion";
	import { fade } from "svelte/transition";
	import Directories from "./settings/Directories.svelte";
	import Jobs from "./settings/Jobs.svelte";

	const routeState = routeManager();
	const { current: currentRoute } = $derived(routeState);
	const navigationPath = $derived(
		currentRoute?.resolvedPath.split("/").filter(Boolean) ?? [],
	);

	let settingsPage: ReturnType<typeof Page> | null = $state(null);
</script>

<Page
	bind:this={settingsPage}
	path="/settings"
	name="Settings"
	icon="settings_3"
	navigation={{ position: "bottom" }}
>
	<div
		class="p-4"
		hidden={routeState.current?.path !== "/settings"}
	>
		<h2 class="text-2xl py-2">Server</h2>
		<ul class="border bg-base rounded-theme border-base-content/10">
			{#each settingsPage?.childPages() || [] as [path, { name, icon }]}
				<li>
					<a
						href={path}
						onclick={(event) => {
							event.preventDefault();
							routeState.goTo(
								(event.target as HTMLAnchorElement).getAttribute("href") as string,
							);
						}}
						class={[
							"font-semibold px-4 flex items-center gap-3 py-2 transition hover:bg-primary/10 hover:text-primary",
							routeState.current?.path === path && "text-primary bg-primary/20",
						]}
						data-active={routeState.current?.path === path}
					>
						<Icon name={icon!} size="1.25em" />
						{name}
					</a>
				</li>
			{/each}
		</ul>
	</div>
	{#if navigationPath.length > 1}
		<div
			in:fade={{ duration: prefersReducedMotion.current ? 0 : 200 }}
			class="p-4"
		>
			<a
				href={buildPath(navigationPath.slice(0, -1))}
				onclick={(event) => {
					event.preventDefault();
					routeState.goTo(
						(event.target as HTMLAnchorElement).getAttribute("href") as string,
					);
				}}
			>
				<Button title="Back" class="pointer-events-none shadow-lg">
					<Icon name="arrow_left" />
				</Button>
			</a>
		</div>
	{/if}
	<Jobs />
	<Directories />
</Page>
