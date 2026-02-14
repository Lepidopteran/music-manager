<script lang="ts">
	import { getServerDirectoryFolders } from "@api/directory";
	import Icon from "@components/Icon.svelte";
	import ListBox from "@components/ListBox.svelte";
	import Popover from "@components/Popover.svelte";
	import TextInput from "@components/TextInput.svelte";
	import { untrack, type ComponentProps } from "svelte";
	import { match } from "ts-pattern";

	let listBoxRef: ReturnType<typeof ListBox>;
	let popoverRef: ReturnType<typeof Popover>;
	let selectedIndex = $state(-1);

	const uuid = $props.id();
	const id = `server-directory-input-${uuid}`;

	interface Props extends ComponentProps<typeof TextInput> {
		value?: string;
	}

	let { value = $bindable(""), class: className, required }: Props = $props();

	let explorerOpen = $state(false);

	type WordSegment = {
		kind: "text" | "highlight";
		value: string;
	};

	function highlightWords(text: string): Array<WordSegment> {
		if (!query.trim()) return [{ kind: "text", value: text }];
		const words = query.toLowerCase().trim().split(/\s+/);

		let result: Array<WordSegment> = [{ kind: "text", value: text }];

		for (const word of words) {
			const temp: Array<WordSegment> = [];

			for (const segment of result) {
				if (segment.kind === "highlight") {
					temp.push(segment);
					continue;
				}

				const regex = new RegExp(`(${word})`, "gi");
				const parts = segment.value.split(regex);

				for (const part of parts) {
					if (part.toLowerCase() === word)
						temp.push({ kind: "highlight", value: part });
					else temp.push({ kind: "text", value: part });
				}
			}

			result = temp;
		}

		return result;
	}

	let folders: Array<string> = $state([]);
	let fetching = $state(false);
	let level = $derived.by(() => {
		const parts = value
			.split("/")
			.filter(
				(part, index, parts) =>
					index !== 0 && part.length > 0 && part !== parts[parts.length - 1],
			);

		return parts.length;
	});

	let query = $derived.by(() => {
		return value.split("/").pop() ?? "";
	});

	let filteredFolders = $derived.by(() => {
		const normalizedQuery = query.trim();
		if (normalizedQuery.length === 0) {
			return folders;
		}

		const queryWords = normalizedQuery
			.split(/\s+/)
			.filter((word) => word.length > 0);

		return folders
			.map((folder) => {
				let score = 0;

				for (const word of queryWords) {
					const criteria: Array<[boolean, number]> = [
						[folder.includes(word), 1],
						[folder.toLowerCase().includes(word.toLowerCase()), 0.5],
						[folder.split(" ").some((w) => w.startsWith(word)), 0.25],
					];

					for (const [match, rank] of criteria) {
						if (!match) continue;
						score += rank;
					}
				}

				return { folder, score };
			})
			.filter((result) => result.score > 0)
			.sort((a, b) => b.score - a.score)
			.map((result) => result.folder);
	});

	$effect(() => {
		if (query.trim().length !== 0) {
			listBoxRef.selectFirst({ behavior: "instant" });
		}
	});

	$effect(() => {
		const dir = untrack(() => value);
		let cancelled = false;

		(async () => {
			try {
				let targetDir = "/";
				if (level !== 0) {
					targetDir = dir.endsWith("/")
						? dir
						: dir.split("/").slice(0, -1).join("/");
				}

				fetching = true;
				const result = await getServerDirectoryFolders(targetDir);
				if (cancelled) {
					return;
				}

				listBoxRef.selectFirst({ behavior: "instant" });
				folders = result;
				fetching = false;
			} catch (err) {
				if (!cancelled) {
					// @ts-expect-error
					error = err?.body ?? err;
				}
			}
		})();

		return () => {
			cancelled = true;
		};
	});

	$inspect(folders.length, filteredFolders.length, explorerOpen);
</script>

<TextInput
	{id}
	{required}
	class={["w-full", className]}
	suffixDecorative={false}
	role="combobox"
	aria-autocomplete="list"
	aria-expanded={explorerOpen}
	aria-controls={`server-directory-listbox-${uuid}`}
	aria-activedescendant={selectedIndex >= 0 && explorerOpen
		? `server-directory-listbox-${uuid}-option-${selectedIndex}`
		: null}
	onfocus={() => (explorerOpen = true)}
	onblur={(e) => {
		if (!e.relatedTarget) {
			explorerOpen = false;
		}
	}}
	onkeydown={(e) => {
		match(e.key)
			.with("ArrowUp", () => {
				e.preventDefault();
				listBoxRef.selectPrevious();
			})
			.with("ArrowDown", () => {
				e.preventDefault();
				listBoxRef.selectNext();
			})
			.with("Home", () => {
				e.preventDefault();
				listBoxRef.selectFirst();
			})
			.with("End", () => {
				e.preventDefault();
				listBoxRef.selectLast();
			})
			.with("Enter", () => {
				if (explorerOpen) {
					listBoxRef.activateSelected();
				}
			})
			.with("Escape", () => {
				if (explorerOpen) {
					explorerOpen = false;
				}
			})
			.otherwise(() => {
				if (!explorerOpen) {
					explorerOpen = true;
				}
			});
	}}
	bind:value
></TextInput>
<Popover
	reference={id}
	tabindex={-1}
	class="bg-base-200 backdrop-blur-sm shadow-lg rounded-theme border border-base-300/50"
	bind:this={popoverRef}
	bind:open={explorerOpen}
	offset={8}
	flip
	size={{
		apply({ rects, elements, availableHeight }) {
			Object.assign(elements.floating.style, {
				maxHeight: `${Math.min(availableHeight, 200)}px`,
				width: `${rects.reference.width}px`,
			});
		},
	}}
>
	<ListBox
		id={`server-directory-listbox-${uuid}`}
		bind:this={listBoxRef}
		tabindex={-1}
		busy={fetching}
		options={level > 0 && !query ? ["..", ...filteredFolders] : filteredFolders}
		onIndexChange={(index) => (selectedIndex = index)}
		optionLabel={(folder) =>
			`Go to ${folder === ".." ? "parent" : folder} directory`}
		onOptionActivate={(folder) => {
			const parts = value.split("/");

			if (folder === "..") {
				const deep = parts[parts.length - 2].length > 1 ? 2 : 1;

				value = parts.slice(0, -deep).join("/") + "/";
			} else {
				value = parts.slice(0, -1).join("/") + `/${folder}/`;
			}

			if (document.activeElement?.id !== id) {
				document.getElementById(id)?.focus();
			}

			explorerOpen = false;
		}}
		class="focus:border border-primary w-full"
	>
		{#snippet option(folder)}
			<Icon
				name="folder-fill"
				class="inline mr-1"
				inline={true}
				aria-hidden="true"
			/>
			{#each highlightWords(folder) as fragment}
				{@const { kind, value } = fragment}
				{#if kind === "highlight"}
					<span class="font-bold text-primary">{value}</span>
				{:else}
					{value}
				{/if}
			{/each}
		{/snippet}
	</ListBox>
</Popover>
