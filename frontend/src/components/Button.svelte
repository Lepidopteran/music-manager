<script lang="ts">
	import type { Snippet } from "svelte";
	import type { Action } from "svelte/action";
	import type { HTMLButtonAttributes } from "svelte/elements";

	interface Props extends HTMLButtonAttributes {
		toggleable?: boolean;
		active?: boolean;
		variant?:
			| "primary"
			| "base"
			| "secondary"
			| "ghost"
			| "info"
			| "success"
			| "warning"
			| "error"
			| "none";
		children?: Snippet;
	}

	let {
		type = "button",
		toggleable = false,
		active = $bindable(false),
		variant = "base",
		children,
		...rest
	}: Props = $props();

	const toggleButton: Action = (node) => {
		const toggle = () => {
			active = !active;
		};

		$effect(() => {
			if (!toggleable) {
				return () => {};
			}

			node.addEventListener("click", toggle);

			return () => {
				node.removeEventListener("click", toggle);
			};
		});
	};
</script>

<button
	{...rest}
	class={[
		"btn inset-shadow-xs inset-shadow-highlight/25",
		variant !== "base" && `btn-${variant}`,
		active && "btn-active",
		rest.class,
	]}
	role={toggleable ? "switch" : "button"}
	aria-checked={toggleable ? active : undefined}
	data-active={active || undefined}
	use:toggleButton
	{type}
>
	{@render children?.()}
</button>

<style>
	@layer components {
		.btn {
			color: var(--base-950);
			padding: 0.5rem;
			background-color: var(--base-400);
			justify-content: center;
			align-items: center;
			gap: 0.5rem;
			display: inline-flex;
			border-radius: var(--radius-theme);
			cursor: pointer;

			&:disabled {
				cursor: not-allowed;
				opacity: 0.5;
			}

			@media (hover: hover) {
				&:hover {
					background-color: var(--base-500);
				}
			}

			&.btn-active {
				background-color: var(--base-500);
			}

			@media (prefers-reduced-motion: no-preference) {
				&:active {
					scale: 0.95;
				}

				transition: 0.1s ease-in-out;
			}

			&.btn-none {
				background-color: transparent;

				@media (hover: hover) {
					&:hover {
						background-color: transparent;
					}
				}

				&.btn-active {
					background-color: transparent;
				}
			}

			&.btn-ghost {
				color: var(--base-950);
				background-color: transparent;
				@media (hover: hover) {
					&:hover {
						background-color: var(--base-400);
					}
				}
				&.btn-active {
					background-color: var(--base-400);
				}
			}

			&.btn-primary {
				color: var(--primary-950);
				background-color: var(--primary-400);
				@media (hover: hover) {
					&:hover {
						background-color: var(--primary-500);
					}
				}
				&.btn-active {
					background-color: var(--primary-500);
				}
			}

			&.btn-info {
				color: var(--color-info-text);
				background-color: var(--color-info);

				@media (hover: hover) {
					&:hover {
						background-color: oklch(from var(--color-info) calc(l * 1.25) c h);
					}
				}

				&.btn-active {
					background-color: oklch(from var(--color-info) calc(l * 1.25) c h);
				}
			}

			&.btn-success {
				color: var(--color-success-text);
				background-color: var(--color-success);

				@media (hover: hover) {
					&:hover {
						background-color: oklch(
							from var(--color-success)
							calc(l * 1.25)
							c h
						);
					}
				}

				&.btn-active {
					background-color: oklch(from var(--color-success) calc(l * 1.25) c h);
				}
			}

			&.btn-warning {
				color: var(--color-warning-text);
				background-color: var(--color-warning);

				@media (hover: hover) {
					&:hover {
						background-color: oklch(
							from var(--color-warning)
							calc(l * 1.25)
							c h
						);
					}
				}

				&.btn-active {
					background-color: oklch(from var(--color-warning) calc(l * 1.25) c h);
				}
			}

			&.btn-error {
				color: var(--color-error-text);
				background-color: var(--color-error);

				@media (hover: hover) {
					&:hover {
						background-color: oklch(from var(--color-error) calc(l * 1.25) c h);
					}
				}

				&.btn-active {
					background-color: oklch(from var(--color-error) calc(l * 1.25) c h);
				}
			}
		}
	}
</style>
