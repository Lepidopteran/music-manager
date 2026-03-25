<script lang="ts">
	import { buildPath } from "@lib/router";
	import { type RedirectMetadata, routeManager } from "@state";
	import { pageContext } from "./Page.svelte";

	interface Props {
		redirectTo?: string;
		absolute?: boolean;
		path: string;
	}

	let {
		path,
		redirectTo,
		absolute,
	}: Props = $props();

	let previousPath: string | null = null;

	const manager = routeManager();
	const parentContext = pageContext();
	const normalizedPath = $derived(
		parentContext
			? buildPath([...parentContext.parentPath.split("/"), ...path.split("/")])
			: buildPath(path.split("/")),
	);

	$effect(() => {
		if (
			previousPath
			&& previousPath !== normalizedPath
			&& manager.router.hasRoute(previousPath)
		) {
			manager.router.removeRoute(previousPath);
		}

		if (!redirectTo && parentContext?.parentPath === undefined) {
			throw new Error("Redirect must have a redirectTo");
		}

		const metadata: RedirectMetadata = {
			redirectTo: redirectTo
				? absolute !== true && parentContext
					? buildPath([
						...parentContext.parentPath.split("/"),
						...redirectTo.split("/"),
					])
					: redirectTo
				: parentContext!.parentPath,
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
