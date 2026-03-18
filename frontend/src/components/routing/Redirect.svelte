<script lang="ts">
	import { buildPath } from "@lib/router";
	import { type RedirectMetadata, routeManager } from "@state";
	import { pageContext } from "./Page.svelte";

	interface Props {
		redirectTo?: string;
		path: string;
	}

	let {
		path,
		redirectTo,
	}: Props = $props();

	let previousPath: string | null = null;

	const manager = routeManager();
	const parentContext = pageContext();
	const normalizedPath = $derived(buildPath(path.split("/")));

	$effect(() => {
		if (
			previousPath
			&& previousPath !== normalizedPath
			&& manager.router.hasRoute(previousPath)
		) {
			manager.router.removeRoute(previousPath);
			parentContext?.aliases.delete(previousPath);
		}

		if (!redirectTo && parentContext?.parentPath === undefined) {
			throw new Error("Redirect must have a redirectTo");
		}

		const metadata: RedirectMetadata = {
			redirectTo: redirectTo ?? parentContext!.parentPath,
		};

		manager.router.addRoute(
			{
				path: normalizedPath,
				metadata: {
					kind: "redirect",
					...metadata,
				},
			},
		);

		previousPath = normalizedPath;
	});
</script>
