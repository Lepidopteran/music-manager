<script lang="ts">
	import type { Snippet } from "svelte";
	import type { Action } from "svelte/action";
	import type { HTMLButtonAttributes } from "svelte/elements";

	interface Props extends HTMLButtonAttributes {
		type?: "button" | "submit" | "reset";
		toggleable?: boolean;
		active?: boolean;
		color?: "primary" | "base" | "secondary" | "ghost" | "none";
		children?: Snippet;
		[key: string]: unknown;
	}

	let {
		type = "button",
		toggleable = false,
		active = false,
		color = "base",
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
	class={`btn inline-flex inset-shadow-xs inset-shadow-highlight/25 cursor-pointer rounded-theme ${color === "base" ? "" : `btn-${color}`} ${active ? "btn-active" : ""} ${rest.class || ""}`}
	role={toggleable ? "switch" : "button"}
	aria-checked={toggleable ? active : undefined}
	data-active={active || undefined}
	use:toggleButton
	{type}
>
	{@render children?.()}
</button>

<style>
	.btn {
		color: var(--base-950);
		padding: 0.5rem;
		background-color: var(--base-400);
		justify-content: center;
		align-items: center;
		gap: 0.5rem;

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

		&.btn-secondary {
			color: var(--secondary-950);
			background-color: var(--secondary-400);

			@media (hover: hover) {
				&:hover {
					background-color: var(--secondary-500);
				}
			}

			&.btn-active {
				background-color: var(--secondary-500);
			}
		}
	}
</style>
