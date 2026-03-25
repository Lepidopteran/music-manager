<script lang="ts">
	import { buildPath, type RouteDefinition } from "@lib/router";
	import { type PageMetadata, routeManager, type RouteMetadata } from "@state";
	import { createSafeContext } from "@utils/context";
	import { type Snippet } from "svelte";
	import type { ClassValue } from "svelte/elements";

	interface Props extends PageMetadata {
		path: string;
		class?: ClassValue;
		children: Snippet;
	}

	interface ChildPageDefinition {
		path: string;
		metadata: PageMetadata;
	}

	let {
		path,
		class: className,
		children,
		...pageMetadata
	}: Props = $props();

	let previousPath: string | null = null;

	const manager = routeManager();
	const parentContext = pageContext();

	const childDefinitionMap = new Map<string, ChildPageDefinition>();
	const combinedPath = $derived(
		parentContext
			? buildPath(
				[...parentContext.parentPath.split("/"), ...path.split("/")],
			)
			: buildPath(path.split("/")),
	);

	export const childPages = (): Array<[string, PageMetadata]> =>
		Array.from(
			childDefinitionMap.entries().map((
				[path, { metadata }],
			) => [path, metadata]),
		);

	setPageContext({
		childPages: childDefinitionMap,
		get parentPath() {
			return path;
		},
		get metadata() {
			return pageMetadata;
		},
	});

	$effect(() => {
		if (
			previousPath
			&& previousPath !== combinedPath
			&& manager.router.hasRoute(previousPath)
		) {
			manager.router.removeRoute(previousPath);
		}

		const def: RouteDefinition<RouteMetadata> = {
			path: combinedPath,
			metadata: {
				kind: "page",
				...parentContext
					? { ...parentContext.metadata, ...pageMetadata }
					: pageMetadata,
			},
		};

		if (parentContext) {
			parentContext.childPages.set(combinedPath, {
				path,
				metadata: pageMetadata,
			});
		} else {
			manager.router.addRoute(def);

			for (const { path, metadata } of childDefinitionMap.values()) {
				manager.router.addRouteWithParentPath(combinedPath, {
					path,
					metadata: {
						kind: "page",
						...metadata,
					},
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
		childPages: Map<string, ChildPageDefinition>;
		parentPath: string;
		metadata: PageMetadata;
	}

	export const [pageContext, setPageContext] = createSafeContext<PageContext>();
</script>

<div
	class={["h-full", className]}
	hidden={manager.current?.path !== combinedPath
	&& !childDefinitionMap.has(manager.current?.path ?? "")
	&& manager.current?.resolvedPath !== window.history.state?.previousPath}
>
	{@render children()}
</div>
