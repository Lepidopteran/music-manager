<script lang="ts">
	import { buildPath } from "@lib/router";
	import { type PageInfo, pageManager } from "@state";
	import { pageContext } from "./Page.svelte";

	interface Props extends PageInfo {
		path: string;
	}

	let {
		path,
		...metadata
	}: Props = $props();

	let previousPath: string | null = null;

	const manager = pageManager();
	const parentContext = pageContext();

	if (!parentContext) {
		throw new Error("PageAlias must be used inside a Page component");
	}

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

		parentContext?.aliases.add(normalizedPath);
		parentContext?.childPages.set(
			normalizedPath,
			parentContext
				? { ...parentContext.metadata, ...metadata }
				: metadata,
		);

		previousPath = normalizedPath;
	});
</script>
