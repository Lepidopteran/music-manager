<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLInputAttributes } from "svelte/elements";
	interface Props extends HTMLInputAttributes {
		label?: string;
		value?: string | null;
		variant?: "base" | "ghost";
		floatingLabel?: boolean;
		prefixChild?: Snippet;
		suffixChild?: Snippet;
		[rest: string]: unknown;
	}

	let {
		label,
		value = $bindable(""),
		variant = "base",
		floatingLabel = false,
		required = true,
		prefixChild,
		suffixChild,
		placeholder,
		pattern,
		class: className,
		...rest
	}: Props = $props();
</script>

<label
	class={`${floatingLabel ? "floating-label inset-shadow-sm inset-shadow-black/25 focus-within:outline-1 focus-within:outline-primary" : ""} ${`input-${variant}`} ${className || ""}`}
>
	{#if label}
		<span class="text-base-950 label">
			{label}
		</span>
	{/if}
	{#if prefixChild}
		<span class="user-select-none pointer-events-none">
			{@render prefixChild()}
		</span>
	{/if}
	<input
		class={`outline-0 ${!floatingLabel ? "rounded-theme inset-shadow-sm inset-shadow-black/25 focus-within:outline-1 focus-within:outline-primary" : ""}`}
		type="text"
		placeholder={placeholder || label}
		bind:value
		{...rest}
	/>
	{#if suffixChild}
		<span class="user-select-none pointer-events-none">
			{@render suffixChild()}
		</span>
	{/if}
</label>

<style>
	label {
		position: relative;
		align-items: center;
		padding: calc(var(--spacing) * 2) calc(var(--spacing));
		border-radius: var(--radius-theme);

		&:has(span:not(.label)) {
			display: flex;
		}

		& > input,
		& > span:not(.label) {
			padding: calc(var(--spacing) * 2) calc(var(--spacing));
		}

		& > input {
			text-align: inherit;
			display: block;
			width: 100%;
		}

		&.input-base > input,
		&.input-base > span:not(.label) {
			background-color: var(--color-base-300);
		}

		&.input-ghost {
			& > input,
			& > span:not(.label) {
				background-color: transparent;
				box-shadow: none;
			}

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

	.floating-label {

		&.input-base {
			background-color: var(--color-base-300);
		}

		&.input-ghost {
			background-color: transparent;
			box-shadow: none;
		}

		& > input {
			padding: calc(var(--spacing) * 2) calc(var(--spacing));
			display: block;
		}

		& > input::placeholder {
			transition: all 200ms ease-in-out;
		}

		& > span.label {
			position: absolute;
			pointer-events: none;
			user-select: none;
			top: 50%;
			transform: translateY(-50%);
			left: 0.5rem;
			opacity: 0;
			transition: all 200ms ease-in-out;
		}

		&:focus-within,
		&:not(:has(:placeholder-shown)) {
			& > input::placeholder {
				opacity: 0;
			}

			& > span:first-child {
				top: 0;
				transform: translateY(10%);
				font-size: var(--text-xs);
				line-height: var(--text-xs--line-height);
				font-weight: var(--font-weight-bold);
				opacity: 0.5;
			}
		}
	}
</style>
