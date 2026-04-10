<script lang="ts" generics="D, V">
	import type { CellContext, HeaderContext } from "@tanstack/table-core";
	import type { Component, ComponentProps, Snippet } from "svelte";
	import type { ContentValueReturnType } from "./table.svelte";

	type Header = HeaderContext<D, V>;
	type Cell = CellContext<D, V>;

	type ContentValue<T> = string | ((context: T) => ContentValueReturnType<T>);

	type Content =
		| { kind: "header"; context: Header; value?: ContentValue<Header> }
		| { kind: "cell"; context: Cell; value?: ContentValue<Cell> };

	type Props = {
		content: Content;
	};

	function isSnippet<Params = unknown>(
		value: unknown,
	): value is { snippet: Snippet<[Params]>; params?: Params } {
		return typeof value === "object" && value !== null && "snippet" in value;
	}

	function isComponent<Comp extends Component>(
		value: unknown,
	): value is { component: Comp; props?: ComponentProps<Comp> } {
		return typeof value === "object" && value !== null && "component" in value;
	}

	let { content }: Props = $props();
</script>

{#if typeof content.value === "string"}
	{content.value}
{:else if typeof content.value === "function"}
	{@const { kind, context, value } = content}

	{#if kind === "header"}
		{@const result = value(context)}

		{#if typeof result === "string"}
			{result}
		{:else if isComponent(result)}
			{@const { component: Component, props } = result}
			<Component {...props} />
		{:else if isSnippet(result)}
			{@render result.snippet(result.params)}
		{/if}
	{:else if kind === "cell"}
		{@const result = value(context)}

		{#if typeof result === "string"}
			{result}
		{:else if isComponent(result)}
			{@const { component: Component, props } = result}
			<Component {...props} />
		{:else if isSnippet(result)}
			{@render result.snippet(result.params)}
		{/if}
	{/if}
{/if}
