<script lang="ts">
	import type { Snippet } from "svelte";

	interface Props {
		separator?: string;
		crumbs?: Array<string>;
		crumb?: Snippet<[string, number, string[]]>;
		[key: string]: unknown;
	}

	let {
		children,
		separator = "",
		crumb: crumb,
		crumbs = $bindable([]),
		...rest
	}: Props = $props();
</script>

<div
	class={separator.length > 0 ? "" : "breadcrumb-box-separator"}
	style={separator.length > 0
		? `--breadcrumb-separator: \"${separator}\"`
		: undefined}
>
	<ul {...rest}>
		{#each crumbs as text, index}
			<li>
				{#if crumb}
					{@render crumb(text, index, crumbs)}
				{:else}
					{text}
				{/if}
			</li>
		{/each}
	</ul>
</div>

<style>
	:root {
		--breadcrumb-color: rgb(from var(--color-base-text) r g b / 0.5);
	}

	ul {
		list-style: none;
		display: flex;
		align-items: center;

		& > li {
			display: flex;
			align-items: center;
			&:not(:first-child)::before {
				content: var(--breadcrumb-separator);
				margin-inline: 0.5em;
				color: var(--breadcrumb-color);
				display: inline-block;
			}
		}
	}

	.breadcrumb-box-separator ul li:not(:first-child)::before {
		content: "";
		width: calc(var(--spacing) * 1.5);
		height: calc(var(--spacing) * 1.5);
		display: inline-block;
		border-right: 1px solid;
		border-top: 1px solid;
		color: var(--breadcrumb-color);
		transform: rotate(45deg);
	}
</style>
