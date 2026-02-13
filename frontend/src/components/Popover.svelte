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
	import type { Middleware } from "@floating-ui/dom";

	const uuid = $props.id();

	export const id = `popover-${uuid}`;

	interface Props extends Omit<
		HTMLAttributes<HTMLDivElement>,
		"id" | "popover"
	> {
		open?: boolean;
		children?: Snippet;
		popoverMode?: "hint" | "manual";
		reference?: HTMLElement;
		placement?: Placement;
		offset?: OffsetOptions;
		flip?: boolean | FlipOptions | Derivable<FlipOptions>;
		shift?: boolean | ShiftOptions | Derivable<ShiftOptions>;
		inline?: boolean | InlineOptions | Derivable<InlineOptions>;
		size?: SizeOptions | Derivable<SizeOptions>;
	}

	let {
		open: opened = $bindable(false),
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

	const popoverAttachment: Attachment<HTMLDivElement> = (popover) => {
		if (!reference) {
			return;
		}

		const cleanUp = autoUpdate(reference, popover, () => {
			computePosition(reference, popover, {
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
		if (opened) {
			popoverRef.showPopover();
		} else {
			popoverRef.hidePopover();
		}
	});

	export function popup() {
		opened = true;
	}

	export function popdown() {
		opened = false;
	}

	export function toggle() {
		opened = !opened;
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
