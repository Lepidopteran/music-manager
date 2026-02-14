<script lang="ts" generics="T">
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";

	const uuid = $props.id();

	interface Props extends Omit<HTMLAttributes<HTMLUListElement>, "aria-busy"> {
		options: T[];
		busy?: boolean;
		option: Snippet<[T, number]>;
		optionLabel?: (option: T, index: number) => string;
		isOptionDisabled?: (option: T, index: number) => boolean;
		onIndexChange?: (index: number) => void;
		onOptionSelect?: (option: T, index: number) => void;
		onOptionActivate: (
			option: T,
			index: number,
			originalEvent: MouseEvent,
		) => void;
	}

	let selectedIndex = $state(-1);
	let selectedId: string = $derived.by(() => {
		return `${id}-option-${selectedIndex}`;
	});

	let selectedOptionElement: null | HTMLButtonElement = $derived.by(() => {
		return document.getElementById(selectedId) as HTMLButtonElement;
	});

	let disabledOptions = $derived.by(() => {
		return options
			.filter((option, index) => isOptionDisabled(option, index))
			.map((_, index) => index);
	});

	export function selectOption(
		index: number,
		scrollOptions?: ScrollIntoViewOptions,
	) {
		selectedIndex = index;
		selectedOptionElement?.scrollIntoView({
			block: "center",
			...scrollOptions,
		});

		onOptionSelect?.(options[index], index);
	}

	export function selectNext(scrollOptions?: ScrollIntoViewOptions) {
		const nextIndex = Math.min(selectedIndex + 1, options.length - 1);
		if (disabledOptions.includes(nextIndex)) {
			if (nextIndex === options.length - 1) {
				invalidateSelection();
				return;
			}

			selectNext(scrollOptions);
			return;
		}

		selectOption(nextIndex, scrollOptions);
	}

	export function selectPrevious(scrollOptions?: ScrollIntoViewOptions) {
		const previousIndex = Math.max(selectedIndex - 1, 0);

		if (disabledOptions.includes(previousIndex)) {
			if (previousIndex === 0) {
				invalidateSelection();
				return;
			}

			selectPrevious(scrollOptions);
			return;
		}

		selectOption(previousIndex, scrollOptions);
	}

	export function selectFirst(scrollOptions?: ScrollIntoViewOptions) {
		if (options.length === 0) {
			invalidateSelection();
			return;
		}

		if (disabledOptions.includes(0)) {
			selectNext(scrollOptions);
			return;
		}

		selectOption(0, scrollOptions);
	}

	export function selectLast(scrollOptions?: ScrollIntoViewOptions) {
		if (options.length === 0) {
			invalidateSelection();
			return;
		}

		if (disabledOptions.includes(options.length - 1)) {
			selectPrevious(scrollOptions);
			return;
		}

		selectOption(options.length - 1, scrollOptions);
	}

	export function activateSelected() {
		selectedOptionElement?.click();
	}

	export function invalidateSelection() {
		selectedIndex = -1;
	}

	let {
		id = `listbox-${uuid}`,
		busy = false,
		options,
		option,
		optionLabel = (_, index) => `Option ${index + 1}`,
		onIndexChange,
		onOptionSelect,
		onOptionActivate,
		isOptionDisabled = () => false,
		...rest
	}: Props = $props();
</script>

<ul {id} role="listbox" aria-busy={busy} tabindex="0" {...rest}>
	{#each options as item, index}
		<li class="inset-shadow-xs inset-shadow-base-950/25">
			<button
				role="option"
				id={`${id}-option-${index}`}
				aria-label={optionLabel(item, index)}
				aria-selected={selectedIndex === index}
				aria-disabled={isOptionDisabled(item, index)}
				disabled={isOptionDisabled(item, index)}
				onclick={(e) => onOptionActivate(item, index, e)}
				data-index={index}
				tabindex="-1"
				class={[
					"p-2 w-full text-left hover:bg-base-300/50 cursor-pointer outline-none truncate",
					selectedIndex === index ? "font-bold bg-base-300" : "",
				]}
			>
				{@render option(item, index)}
			</button>
		</li>
	{/each}
</ul>

<style>
	@layer components {
		@media (prefers-reduced-motion: no-preference) {
			button {
				transition:
					background-color 0.2s ease-in-out,
					color 0.2s ease-in-out,
					font-weight 50ms ease-in-out;
			}
		}

		ul {
			overflow-y: auto;
		}
	}
</style>
