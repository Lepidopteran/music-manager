<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLInputAttributes } from "svelte/elements";

	interface Props extends HTMLInputAttributes {
		value?: string | null;
		variant?: "base" | "ghost";
		prefixChild?: Snippet;
		suffixChild?: Snippet;
		prefixDecorative?: boolean;
		suffixDecorative?: boolean;
	}

	let {
		value = $bindable(""),
		variant = "base",
		required = true,
		prefixChild,
		suffixChild,
		prefixDecorative = true,
		suffixDecorative = true,
		placeholder,
		pattern,
		class: className,
		...rest
	}: Props = $props();
</script>

<div
	class={[
		"inset-shadow-sm inset-shadow-black/25 focus-within:outline-1 focus-within:outline-primary",
		`input-${variant}`,
		className,
	]}
>
	{#if prefixChild}
		<span
			class={["user-select-none", prefixDecorative && "pointer-events-none"]}
		>
			{@render prefixChild()}
		</span>
	{/if}
	<input type="text" class="outline-none" {placeholder} bind:value {...rest} />
	{#if suffixChild}
		<span
			class={["user-select-none", suffixDecorative && "pointer-events-none"]}
		>
			{@render suffixChild()}
		</span>
	{/if}
</div>

<style>
	@layer components {
		div {
			position: relative;
			align-items: center;
			border-radius: var(--radius-theme);
			display: inline-flex;
			background-color: var(--color-base-300);

			& > input {
				text-align: inherit;
				display: block;
				width: 100%;
				padding: calc(var(--spacing) * 2) calc(var(--spacing));
			}

			&.input-ghost {
				background-color: transparent;
				box-shadow: none;

				&:has(input:not(:disabled):not([disabled])):hover {
					background-color: rgb(from var(--color-base-300) r g b / 50%);
				}
			}
		}

		input {
			transition: all 50ms linear;

			&:disabled,
			&[disabled] {
				cursor: not-allowed;
				opacity: 0.5;
			}
		}
	}
</style>
