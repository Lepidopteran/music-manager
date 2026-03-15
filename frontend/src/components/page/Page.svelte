<script lang="ts">
	import { buildPath, type RouteDefinition } from "@lib/router";
	import { type PageInfo, pageManager } from "@state";
	import { createSafeContext } from "@utils/context";
	import { type Snippet } from "svelte";
	import type { ClassValue } from "svelte/elements";
	import { SvelteSet } from "svelte/reactivity";

	interface Props extends PageInfo {
		path: string;
		class?: ClassValue;
		children: Snippet;
	}

	let {
		path,
		class: className,
		children,
		callback,
		displayEditor,
		hideHeader,
		hideNavigation,
		icon,
		name,
	}: Props = $props();

	const metadata: PageInfo = $derived({
		hideNavigation,
		hideHeader,
		name,
		displayEditor,
		icon,
		callback,
	});

	let previousPath: string | null = null;

	const manager = pageManager();
	const parentContext = pageContext();

	const aliases = new SvelteSet<string>();
	const childPages = new Map<string, PageInfo>();
	const combinedPath = $derived(
		parentContext
			? buildPath(
				[...parentContext.parentPath.split("/"), ...path.split("/")],
			)
			: buildPath(path.split("/")),
	);

	setPageContext({
		aliases,
		childPages,
		get parentPath() {
			return path;
		},
		get metadata() {
			return metadata;
		},
	});

	$effect(() => {
		if (
			previousPath
			&& previousPath !== combinedPath
			&& manager.router.hasRoute(previousPath)
		) {
			manager.router.removeRoute(previousPath);
			parentContext?.aliases.delete(previousPath);
		}

		const def: RouteDefinition<PageInfo> = {
			path: combinedPath,
			metadata: parentContext
				? { ...parentContext.metadata, ...metadata }
				: metadata,
		};

		if (parentContext) {
			parentContext.childPages.set(path, metadata);
		} else {
			manager.router.addRoute(def);

			for (const [childPath, childMetadata] of childPages.entries()) {
				manager.router.addRouteWithParentPath(combinedPath, {
					path: childPath,
					metadata: childMetadata,
				});
			}
		}

		previousPath = combinedPath;

		return () => {
			manager.router.removeRoute(combinedPath);
		};
	});
</script>

<script lang="ts" module>
	export interface PageContext {
		aliases: Set<string>;
		childPages: Map<string, PageInfo>;
		parentPath: string;
		metadata: PageInfo;
	}

	export const [pageContext, setPageContext] = createSafeContext<PageContext>();
</script>

<div
	class={["h-full", className]}
	hidden={manager.current?.path !== combinedPath
	&& !aliases.has(manager.current?.path ?? "")}
>
	{@render children()}
</div>
