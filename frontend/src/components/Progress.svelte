<script lang="ts">
	import type { HTMLProgressAttributes } from "svelte/elements";

	type Variant = "primary" | "secondary" | "base";

	interface Props extends HTMLProgressAttributes {
		variant?: Variant;
	}

	let {
		value = $bindable(0),
		max = 100,
		variant = "primary",
		class: className,
		...rest
	}: Props = $props();
</script>

<progress
	{max}
	{value}
	{...rest}
	class={[
		"h-2 rounded-theme-sm bg-base-400 overflow-hidden",
		className,
		variant && `text-${variant}`,
	]}
>
	{(Number(value) * 100) / Number(max)}%
</progress>

<style>
	@layer components {
		progress {
			&:indeterminate {
				background-image: repeating-linear-gradient(
					90deg,
					currentColor -1%,
					currentColor 10%,
					#0000 10%,
					#0000 90%
				);
				background-size: 200%;
				background-position-x: 15%;
				@media (prefers-reduced-motion: no-preference) {
					animation: progress 5s ease-in-out infinite;
				}

				@supports (-moz-appearance: none) {
					&::-moz-progress-bar {
						@media (prefers-reduced-motion: no-preference) {
							animation: progress 5s ease-in-out infinite;
							background-image: repeating-linear-gradient(
								90deg,
								currentColor -1%,
								currentColor 10%,
								transparent 10%,
								transparent 90%
							);
							background-size: 200%;
							background-position-x: 15%;
						}
					}
				}
			}

			@supports (-moz-appearance: none) {
				&::-moz-progress-bar {
					background-color: currentColor;
				}
			}

			@supports (-webkit-appearance: none) {
				&::-webkit-progress-bar {
					background-color: transparent;
				}

				&::-webkit-progress-value {
					background-color: currentColor;
				}
			}

			@keyframes progress {
				50% {
					background-position-x: -115%;
				}
			}
		}
	}
</style>
