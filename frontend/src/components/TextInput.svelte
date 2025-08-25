<script lang="ts">
	import type { HTMLInputAttributes } from "svelte/elements";
	interface Props extends HTMLInputAttributes {
		label?: string;
		value?: string;
		variant?: "base" | "ghost";
		floatingLabel?: boolean;
		[rest: string]: unknown;
	}

	let {
		label,
		value = $bindable(""),
		variant = "base",
		floatingLabel = false,
		required = true,
		placeholder,
		pattern,
		class: className,
		...rest
	}: Props = $props();
</script>

<label class={`${floatingLabel ? "floating-label" : ""} block`}>
	{#if label}
		<span class="text-base-950 block">
			{label}
		</span>
	{/if}
	<input
		class={`inset-shadow-sm inset-shadow-black/25 focus-visible:outline-1 focus-visible:outline-primary ${`input-${variant}`} ${className || ""}`}
		type="text"
		placeholder={placeholder || label}
		bind:value
		{...rest}
	/>
</label>

<style>
	input {
		padding: calc(var(--spacing) * 2) calc(var(--spacing));
		border-radius: var(--radius-theme);

		&:disabled,
		&[disabled] {
			cursor: not-allowed;
			opacity: 0.5;
		}

		&.input-base {
			background-color: var(--color-base-300);
		}

		&.input-ghost {
			background-color: transparent;
			box-shadow: none;
		}
	}

	.floating-label {
		position: relative;
		display: block;

		& > input {
			padding: calc(var(--spacing) * 4) calc(var(--spacing) * 2);
			display: block;
		}

		& > input::placeholder {
			transition: all 200ms ease-in-out;
		}

		& > span {
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

			& > span {
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
