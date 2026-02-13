<script lang="ts" generics="T">
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";
	import { match } from "ts-pattern";

	const uuid = $props.id();

	interface Props extends Omit<
		HTMLAttributes<HTMLUListElement>,
		"aria-busy" | "aria-activedescendant"
	> {
		options: T[];
		busy?: boolean;
		option: Snippet<[T, number]>;
		optionLabel?: (option: T, index: number) => string;
		isOptionDisabled?: (option: T, index: number) => boolean;
		onOptionActivate: (
			option: T,
			index: number,
			originalEvent: MouseEvent,
		) => void;
	}

	let activeIndex = $state(0);
	let activeId: string = $derived.by(() => {
		return `${id}-option-${activeIndex}`;
	});

	let activeOption: null | HTMLButtonElement = $derived.by(() => {
		return document.getElementById(activeId) as HTMLButtonElement;
	});

	export function selectOption(
		index: number,
		scrollOptions?: ScrollIntoViewOptions,
	) {
		activeIndex = index;
		activeOption?.scrollIntoView({
			block: "center",
			behavior: "smooth",
			...scrollOptions,
		});
	}

	export function selectNext(scrollOptions?: ScrollIntoViewOptions) {
		selectOption(Math.min(activeIndex + 1, options.length - 1), scrollOptions);
	}

	export function selectPrevious(scrollOptions?: ScrollIntoViewOptions) {
		selectOption(Math.max(activeIndex - 1, 0), scrollOptions);
	}

	export function selectFirst(scrollOptions?: ScrollIntoViewOptions) {
		selectOption(0, scrollOptions);
	}

	export function selectLast(scrollOptions?: ScrollIntoViewOptions) {
		selectOption(options.length - 1, scrollOptions);
	}

	export function activateSelected() {
		activeOption?.click();
	}

	function onkeydown(event: KeyboardEvent) {
		match(event.key)
			.with("ArrowUp", () => {
				event.preventDefault();
				selectPrevious();
			})
			.with("ArrowDown", () => {
				event.preventDefault();
				selectNext();
			})
			.with("Home", () => {
				event.preventDefault();
				selectFirst();
			})
			.with("End", () => {
				event.preventDefault();
				selectLast();
			})
			.with("Enter", () => {
				event.preventDefault();
				activateSelected();
			});
	}

	let {
		id = `palette-${uuid}`,
		busy = false,
		options,
		option,
		optionLabel = (_, index) => `Option ${index + 1}`,
		onOptionActivate,
		isOptionDisabled = () => false,
		...rest
	}: Props = $props();
</script>

<ul
	{id}
	role="listbox"
	aria-busy={busy}
	aria-activedescendant={activeId}
	tabindex="0"
	{onkeydown}
	{...rest}
>
	{#each options as item, index}
		<li class="inset-shadow-xs inset-shadow-base-950/25">
			<button
				role="option"
				id={`${id}-option-${index}`}
				aria-label={optionLabel(item, index)}
				aria-selected={activeIndex === index}
				disabled={isOptionDisabled(item, index)}
				onclick={(e) => onOptionActivate(item, index, e)}
				data-index={index}
				tabindex="-1"
				class={[
					"p-2 w-full text-left hover:bg-base-300/50 cursor-pointer outline-none truncate",
					activeIndex === index ? "font-bold bg-base-300" : "",
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
