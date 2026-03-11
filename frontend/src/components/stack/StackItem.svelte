<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";
	import { stackItemProps } from "./context";

	interface Props extends HTMLAttributes<HTMLDivElement> {
		index?: number;
		children?: Snippet;
	}

	const { class: className, index = stackItemProps().index, children, ...rest }:
		Props = $props();
</script>

<div
	class={["stack-item", className]}
	style:--stack-index={index}
	{...rest}
>
	{@render children?.()}
</div>
