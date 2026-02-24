<script lang="ts">
	import type { HTMLInputAttributes } from "svelte/elements";

	interface Props extends Omit<HTMLInputAttributes, "type"> {
		variant?:
			| "primary"
			| "secondary"
			| "base"
			| "info"
			| "success"
			| "warning"
			| "error";
	}

	let { class: className, variant = "base", ...rest }: Props = $props();
</script>

<input
	type="checkbox"
	{...rest}
	class={[variant !== "base" && `checkbox-${variant}`, className]}
/>

<style>
	@layer components {
		input {
			border: 1px solid
				var(--input-color, color-mix(in oklab, currentColor 20%, #0000));
			padding: var(--spacing);
			flex-shrink: 0;
			cursor: pointer;
			color: var(--color-base-text);
			border-radius: var(--radius-theme);
			display: inline-block;
			position: relative;
			appearance: none;
			text-align: center;
			transition: background-color 0.2s, box-shadow 0.2s;

			--size: calc(var(--spacing, 0.25rem) * 6);
			width: var(--size);
			height: var(--size);

			&:before {
				--tw-content: "";
				content: var(--tw-content);
				display: block;
				width: 100%;
				height: 100%;
				rotate: 45deg;
				background-color: currentColor;
				opacity: 0;
				transition: clip-path 0.3s, opacity 0.1s, rotate 0.3s, translate 0.3s;
				transition-delay: 0.1s;
				clip-path: polygon(
					20% 100%,
					20% 80%,
					50% 80%,
					50% 80%,
					70% 80%,
					70% 100%
				);
				box-shadow: 0px 3px 0 0px oklch(100% 0 0 / calc(var(--depth) * 0.1))
					inset;
				font-size: 1rem;
				line-height: 0.75;
			}

			&:focus-visible {
				outline: 2px solid var(--input-color, currentColor);
				outline-offset: 2px;
			}

			&:checked,
			&[aria-checked="true"] {
				background-color: var(--input-color, #0000);

				&:before {
					opacity: 1;
					clip-path: polygon(
						20% 100%,
						20% 80%,
						50% 80%,
						50% 0%,
						70% 0%,
						70% 100%
					);
				}

				@media (forced-colors: active) {
					&:before {
						--tw-content: "✔︎";
						background: none;
						rotate: 0deg;
						clip-path: none;
					}
				}

				@media print {
					&:before {
						background: none;
						rotate: 0deg;
						--tw-content: "✔︎";
						clip-path: none;
					}
				}
			}

			&:indeterminate {
				background-color: var(
					--input-color,
					color-mix(in oklab, var(--color-base-content) 20%, #0000)
				);
				&:before {
					translate: 0 -35%;
					rotate: 0deg;
					opacity: 1;
					clip-path: polygon(
						20% 100%,
						20% 80%,
						50% 80%,
						50% 80%,
						80% 80%,
						80% 100%
					);
				}
			}

			&:disabled {
				cursor: not-allowed;
				opacity: 0.5;
			}

			&.checkbox-primary {
				--input-color: var(--color-primary);
				color: var(--color-primary-text);
			}

			&.checkbox-secondary {
				--input-color: var(--color-secondary);
				color: var(--color-secondary-text);
			}

			&.checkbox-success {
				--input-color: var(--color-success);
				color: var(--color-success-text);
			}

			&.checkbox-warning {
				--input-color: var(--color-warning);
				color: var(--color-warning-text);
			}

			&.checkbox-error {
				--input-color: var(--color-error);
				color: var(--color-error-text);
			}

			&.checkbox-info {
				--input-color: var(--color-info);
				color: var(--color-info-text);
			}
		}
	}
</style>
