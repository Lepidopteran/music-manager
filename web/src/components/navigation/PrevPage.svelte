<script lang="ts">
	import Button from "@components/Button.svelte";
	import Icon from "@components/Icon.svelte";
	import { buildPath } from "@lib/router";
	import { routeManager } from "@state";

	const routeState = routeManager();
	const { current: currentRoute } = $derived(routeState);
	const navigationPath = $derived(
		currentRoute?.resolvedPath.split("/").filter(Boolean) ?? [],
	);
</script>

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
