<script lang="ts">
	import type { Snippet } from "svelte";
	import type { Attachment } from "svelte/attachments";
	import type { HTMLAttributes } from "svelte/elements";
	import type {
		Derivable,
		Placement,
		FlipOptions,
		OffsetOptions,
		InlineOptions,
		ShiftOptions,
		SizeOptions,
		FloatingElement,
		Middleware,
	} from "@floating-ui/dom";

	import {
		computePosition,
		autoUpdate,
		flip,
		offset,
		shift,
		inline,
		size,
	} from "@floating-ui/dom";

	const uuid = $props.id();

	interface Props extends Omit<HTMLAttributes<HTMLDivElement>, "popover"> {
		open?: boolean;
		children?: Snippet;
		placement?: Placement;
		offset?: OffsetOptions;
		popoverMode?: "hint" | "manual";
		reference?: Element | string;
		size?: SizeOptions | Derivable<SizeOptions>;
		flip?: boolean | FlipOptions | Derivable<FlipOptions>;
		shift?: boolean | ShiftOptions | Derivable<ShiftOptions>;
		inline?: boolean | InlineOptions | Derivable<InlineOptions>;
	}

	let {
		id = `popover-${uuid}`,
		open = $bindable(false),
		popoverMode: popover = "manual",
		offset: offsetOptions,
		flip: flipOptions,
		shift: shiftOptions,
		size: sizeOptions,
		inline: inlineOptions,
		placement = "bottom",
		reference,
		children,
		...rest
	}: Props = $props();

	/** Apply middleware if options are enabled or provided */
	function lazyMiddleware<T>(
		factory: (options?: T | Derivable<T>) => Middleware,
		options?: boolean | T | Derivable<T>,
	): Middleware | null {
		if (!options) {
			return null;
		} else if (options === true) {
			return options ? factory() : null;
		} else {
			return factory(options);
		}
	}

	const popoverAttachment: Attachment<FloatingElement> = (popover) => {
		const referenceElement =
			typeof reference === "string"
				? document.getElementById(reference) ||
					document.querySelector(reference)
				: reference;

		console.debug("popoverAttachment", referenceElement, reference);

		if (!referenceElement) {
			return;
		}

		const cleanUp = autoUpdate(referenceElement, popover, () => {
			computePosition(referenceElement, popover, {
				placement,
				strategy: "fixed",
				middleware: [
					lazyMiddleware(offset, offsetOptions),
					lazyMiddleware(flip, flipOptions),
					lazyMiddleware(shift, shiftOptions),
					lazyMiddleware(inline, inlineOptions),
					lazyMiddleware(size, sizeOptions),
				].filter(Boolean),
			}).then(({ x, y }) => {
				popover.style.left = `${x}px`;
				popover.style.top = `${y}px`;
			});
		});

		return () => {
			cleanUp();
		};
	};

	let popoverRef: HTMLDivElement;

	$effect(() => {
		if (open) {
			popoverRef.showPopover();
		} else {
			popoverRef.hidePopover();
		}
	});

	export function popup() {
		open = true;
	}

	export function popdown() {
		open = false;
	}

	export function toggle() {
		open = !open;
	}

	export function focus() {
		popoverRef.focus();
	}

	export function blur() {
		popoverRef.blur();
	}
</script>

<div
	{id}
	{popover}
	bind:this={popoverRef}
	{@attach popoverAttachment}
	{...rest}
>
	{@render children?.()}
</div>
