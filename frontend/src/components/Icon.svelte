<script lang="ts">
	import type { HTMLAttributes } from "svelte/elements";
	import { type Icon } from "virtual:icons";

	interface Props extends HTMLAttributes<SVGElement> {
		name: Icon;
		hFlip?: boolean;
		vFlip?: boolean;
		size?: number | string;
		rotate?: number;
	}

	let {
		name,
		size = "1em",
		hFlip = false,
		vFlip = false,
		rotate = 0,
		class: className,
		...rest
	}: Props = $props();
</script>

{#await import("virtual:icons")}
	<div
		class={[className]}
		style:width={size}
		style:height={size}
		style:transform={`rotate(${rotate}deg)`}
	>
	</div>
{:then { icons, iconToSVG }}
	{@const renderData = iconToSVG(icons[name], {
		rotate,
		hFlip,
		vFlip,
		width: size,
		height: size,
	})}

	<svg {...renderData.attributes} class={[className]} {...rest}>
		{@html renderData.body}
	</svg>
{/await}
