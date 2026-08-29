<script lang="ts" generics="T">
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";

	interface Props extends HTMLAttributes<HTMLUListElement> {
		data?: Array<T>;
		crumb?: Snippet<[{ item: T; index: number }]>;
	}

	let {
		crumb,
		data = $bindable([]),
		...rest
	}: Props = $props();
</script>

<ul {...rest}>
	{#each data as item, index}
		<li>
			{#if crumb}
				{@render crumb({ item, index })}
			{:else}
				{item}
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
			gap: calc(var(--spacing) * 1.5);

			& > li {
				display: flex;
				align-items: center;
				gap: calc(var(--spacing) * 1.5);
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
