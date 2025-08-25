<script lang="ts">
	import { icons as mingcute } from "@iconify-json/mingcute";
	import json from "@iconify-json/mingcute/icons.json";

	import {
		iconToHTML,
		iconToSVG,
		replaceIDs,
		getIconData,
	} from "@iconify/utils";
	import type { ClassValue } from "svelte/elements";

	type IconName = keyof typeof json.icons;

	interface Props {
		name: IconName;
		hFlip?: boolean;
		vFlip?: boolean;
		rotate?: number | string;
		size?: number | string;
		class?: ClassValue;
		[key: string]: unknown;
	}

	let {
		name,
		size = "1em",
		hFlip = false,
		vFlip = false,
		class: className,
		...rest
	}: Props = $props();

	const iconData = getIconData(mingcute, name);

	if (!iconData) {
		throw new Error(`Icon ${name} not found`);
	}

	const renderData = iconToSVG(iconData, {
		hFlip,
		vFlip,
		width: size,
		height: size,
	});
</script>

<svg
	{...renderData.attributes}
	class={[className]}
>
	{@html renderData.body}
</svg>
