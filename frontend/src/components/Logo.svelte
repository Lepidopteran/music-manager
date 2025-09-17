<script lang="ts">
	import type { ClassValue } from "svelte/elements";

	interface Props {
		class?: ClassValue;
		size?: number | string;
		[key: string]: unknown;
	}

	const { class: className, size = "2.5rem", ...rest }: Props = $props();
</script>

<div
	class={["inline-flex", className]}
	{...rest}
	style:width={size}
	style:height={size}
>
	<svg
		width="512"
		height="512"
		viewBox="0 0 512 512"
		version="1.1"
		xmlns:xlink="http://www.w3.org/1999/xlink"
		class="size-full"
		xmlns="http://www.w3.org/2000/svg"
	>
		<defs id="defs1">
			<linearGradient id="linearGradient19">
				<stop
					style="stop-color:#a94fdd;stop-opacity:1;"
					offset="0"
					id="stop18"
				/>
				<stop
					style="stop-color:#24ff90;stop-opacity:1;"
					offset="1"
					id="stop19"
				/>
			</linearGradient>
			<filter
				id="b"
				width="1.037125"
				height="1.037125"
				x="-0.018562509"
				y="-0.018562509"
				style="color-interpolation-filters:sRGB"
			>
				<feGaussianBlur stdDeviation="3.7125017" id="feGaussianBlur1" />
			</filter>
			<linearGradient
				xlink:href="#linearGradient19"
				id="linearGradient18"
				x1="123.29063"
				y1="92.074959"
				x2="409.62277"
				y2="397.49896"
				gradientUnits="userSpaceOnUse"
			/>
			<filter
				id="filter19"
				width="1.0394248"
				height="1.0394248"
				x="-0.019712399"
				y="-0.019712399"
				style="color-interpolation-filters:sRGB"
			>
				<feGaussianBlur stdDeviation="3.7125017" id="feGaussianBlur19" />
			</filter>
			<mask maskUnits="userSpaceOnUse" id="mask12">
				<path
					id="path13"
					style="fill:#ffffff;stroke-width:1.38469"
					d="M 176.15185,108.90857 V 272.3812 c -35.39923,-9.02047 -76.33142,13.85001 -76.3311,52.12453 -2.4e-4,29.91417 26.88276,54.16444 60.0447,54.16438 33.1619,6e-5 60.04489,-24.25021 60.04464,-54.16438 l 0.008,-186.92914 125.89333,34.43213 v 146.68517 c -35.44694,-9.10947 -76.45579,13.78543 -76.45548,52.09169 -2.5e-4,29.91412 26.88276,54.16443 60.0447,54.16435 33.16191,8e-5 60.04491,-24.25023 60.04466,-54.16435 l 0.13307,-215.47775 c -6.1e-4,-9.86328 -6.59882,-18.50743 -16.11279,-21.10904 L 203.80886,87.799495 c -12.94582,-3.541138 -27.65645,5.793467 -27.65701,21.109075 z"
				/>
			</mask>
			<filter
				style="color-interpolation-filters:sRGB"
				id="filter13"
				x="-0.029242979"
				y="-0.02507659"
				width="1.058486"
				height="1.0501532"
			>
				<feGaussianBlur stdDeviation="3.5305733" id="feGaussianBlur13" />
			</filter>
			<filter
				style="color-interpolation-filters:sRGB"
				id="filter14"
				x="-0.029242979"
				y="-0.02507659"
				width="1.058486"
				height="1.0501532"
			>
				<feGaussianBlur stdDeviation="3.5305733" id="feGaussianBlur14" />
			</filter>
		</defs>
		<circle
			cx="256"
			cy="256"
			r="240"
			style="display:inline;opacity:0.25;fill:#000000;filter:url(#b)"
			id="shadow"
		/>
		<g id="disc" style="display:inline">
			<circle
				style="opacity:1;fill:#808080;fill-opacity:1"
				id="discPlate"
				cx="256"
				cy="256"
				r="240"
			/>
			<circle
				cx="256"
				cy="256"
				r="226"
				style="opacity:1;fill:#ffffff;fill-opacity:0.325;filter:url(#filter19)"
				id="discUnder"
			/>
			<circle cx="256" cy="256" r="130" style="fill:#555555" id="discLabel" />
			<circle style="fill:#bbbbbb" id="discHole" cx="256" cy="256" r="58" />
		</g>
		<g id="note">
			<path
				id="noteShadow"
				style="opacity:0.25;fill:#000000;filter:url(#filter14)"
				d="M 176.15185,108.90857 V 272.3812 c -35.39923,-9.02047 -76.33142,13.85001 -76.3311,52.12453 -2.4e-4,29.91417 26.88276,54.16444 60.0447,54.16438 33.1619,6e-5 60.04489,-24.25021 60.04464,-54.16438 l 0.008,-186.92914 125.89333,34.43213 v 146.68517 c -35.44694,-9.10947 -76.45579,13.78543 -76.45548,52.09169 -2.5e-4,29.91412 26.88276,54.16443 60.0447,54.16435 33.16191,8e-5 60.04491,-24.25023 60.04466,-54.16435 l 0.13307,-215.47775 c -6.1e-4,-9.86328 -6.59882,-18.50743 -16.11279,-21.10904 L 203.80886,87.799495 c -12.94582,-3.541138 -27.65645,5.793467 -27.65701,21.109075 z"
			/>
			<path
				id="noteBase"
				style="fill:#404040"
				d="M 176.15185,108.90857 V 272.3812 c -35.39923,-9.02047 -76.33142,13.85001 -76.3311,52.12453 -2.4e-4,29.91417 26.88276,54.16444 60.0447,54.16438 33.1619,6e-5 60.04489,-24.25021 60.04464,-54.16438 l 0.008,-186.92914 125.89333,34.43213 v 146.68517 c -35.44694,-9.10947 -76.45579,13.78543 -76.45548,52.09169 -2.5e-4,29.91412 26.88276,54.16443 60.0447,54.16435 33.16191,8e-5 60.04491,-24.25023 60.04466,-54.16435 l 0.13307,-215.47775 c -6.1e-4,-9.86328 -6.59882,-18.50743 -16.11279,-21.10904 L 203.80886,87.799495 c -12.94582,-3.541138 -27.65645,5.793467 -27.65701,21.109075 z"
			/>
			<path
				id="noteBevel"
				style="fill:#ffffff;filter:url(#filter13)"
				d="M 176.15185,108.90857 V 272.3812 c -35.39923,-9.02047 -76.33142,13.85001 -76.3311,52.12453 -2.4e-4,29.91417 26.88276,54.16444 60.0447,54.16438 33.1619,6e-5 60.04489,-24.25021 60.04464,-54.16438 l 0.008,-186.92914 125.89333,34.43213 v 146.68517 c -35.44694,-9.10947 -76.45579,13.78543 -76.45548,52.09169 -2.5e-4,29.91412 26.88276,54.16443 60.0447,54.16435 33.16191,8e-5 60.04491,-24.25023 60.04466,-54.16435 l 0.13307,-215.47775 c -6.1e-4,-9.86328 -6.59882,-18.50743 -16.11279,-21.10904 L 203.80886,87.799495 c -12.94582,-3.541138 -27.65645,5.793467 -27.65701,21.109075 z"
				mask="url(#mask12)"
			/>
		</g>
		<circle
			style="mix-blend-mode:multiply;fill:url(#linearGradient18)"
			id="color"
			cx="256"
			cy="256"
			r="240"
		/>
	</svg>
</div>
