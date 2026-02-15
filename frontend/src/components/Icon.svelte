<script lang="ts">
	import { icons as mingcute } from "@iconify-json/mingcute";
	import { iconToSVG, getIconData } from "@iconify/utils";

	import type { Icons } from "@lib/icons";
	import type { HTMLAttributes } from "svelte/elements";

	interface Props extends HTMLAttributes<SVGElement> {
		name: Icons;
		hFlip?: boolean;
		vFlip?: boolean;
		rotate?: number | string;
		size?: number | string;
	}

	let {
		name,
		size = "1em",
		hFlip = false,
		vFlip = false,
		class: className,
		...rest
	}: Props = $props();

	const iconData = $derived.by(() => {
		let data = getIconData(mingcute, name);
		if (!data) {
			throw new Error(`Icon "${name}" not found`);
		}

		return data;
	});

	const renderData = $derived(
		iconToSVG(iconData, {
			hFlip,
			vFlip,
			width: size,
			height: size,
		}),
	);
</script>

<svg {...renderData.attributes} class={[className]} {...rest}>
	{@html renderData.body}
</svg>
