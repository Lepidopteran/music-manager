<script lang="ts">
	import type { Snippet } from "svelte";
	import type { ClassValue, HTMLAttributes } from "svelte/elements";

	import { type PageInfo, pageManager } from "@lib/state";

	interface Props extends PageInfo, HTMLAttributes<HTMLDivElement> {
		path: string;
		class?: ClassValue;
		children?: Snippet;
	}

	let { path, class: className, children, ...rest }: Props = $props();
	const pages = pageManager();

	$effect(() => {
		pages.addPage({
			path,
			metadata: {
				hideNavigation: rest.hideNavigation,
				hideHeader: rest.hideHeader,
				name: rest.name,
				displayEditor: rest.displayEditor,
				icon: rest.icon,
				callback: rest.callback,
			},
		});

		return () => {
			pages.removePage(path);
		};
	});
</script>

<div
	class={["h-full", className]}
	hidden={pages.current?.path !== path}
	{...rest}
>
	{@render children?.()}
</div>
