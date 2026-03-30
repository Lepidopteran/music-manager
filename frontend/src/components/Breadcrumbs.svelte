<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";

	interface Props extends HTMLAttributes<HTMLUListElement> {
		crumbs?: Array<string>;
		crumb?: Snippet<[{ text: string; index: number }]>;
	}

	let {
		crumb,
		crumbs = $bindable([]),
		...rest
	}: Props = $props();
</script>

<ul {...rest}>
	{#each crumbs as text, index}
		<li>
			{#if crumb}
				{@render crumb({ text, index })}
			{:else}
				{text}
			{/if}
		</li>
	{/each}
</ul>

<style>
	@layer components {
		ul {
			list-style: none;
			display: flex;
			align-items: center;

			& > li {
				display: flex;
				align-items: center;
				&:not(:first-child)::before {
					content: "";
					width: calc(var(--spacing) * 1.5);
					height: calc(var(--spacing) * 1.5);
					display: inline-block;
					border-right: 1px solid;
					border-top: 1px solid;
					color: oklch(from var(--color-base-content) l c h / 0.5);
					transform: rotate(45deg);
				}
			}
		}
	}
</style>
