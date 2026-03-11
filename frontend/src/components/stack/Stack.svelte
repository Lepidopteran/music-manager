<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";
	import { setStackItemProps } from "./context";

	interface Props extends HTMLAttributes<HTMLDivElement> {
		direction?: "top" | "bottom" | "left" | "right";
		offset?: number | string;
		children?: Snippet;
	}

	let nextIndex = $state(0);

	setStackItemProps({
		get index() {
			return nextIndex++;
		},
	});

	let { class: className, direction = "bottom", offset = 6, children, ...rest }:
		Props = $props();
</script>

<div
	style:--stack-offset={typeof offset === "number" ? `${offset}px` : offset}
	class={["stack", `stack-${direction}`, className]}
	{...rest}
>
	{@render children?.()}
	<style>
		@layer components {
			.stack .stack-item {
				grid-area: stack;
				width: 100%;
				height: 100%;
				z-index: var(--stack-index);
			}

			.stack-bottom .stack-item {
				margin-top: calc(var(--stack-index) * var(--stack-offset));
			}

			.stack-top .stack-item {
				margin-bottom: calc(var(--stack-index) * var(--stack-offset));
			}

			.stack-left .stack-item {
				margin-right: calc(var(--stack-index) * var(--stack-offset));
			}

			.stack-right .stack-item {
				margin-left: calc(var(--stack-index) * var(--stack-offset));
			}
		}
	</style>
</div>

<style>
	@layer components {
		.stack {
			--stack-index: 0;
			display: inline-grid;
			grid-template-areas: "stack";
		}
	}
</style>
