<script lang="ts">
	import Button from "@components/Button.svelte";
	import Icon from "@components/Icon.svelte";
	import PrevPage from "@components/navigation/PrevPage.svelte";
	import Page from "@components/routing/Page.svelte";
	import { buildPath } from "@lib/router";
	import { routeManager } from "@state";
	import { prefersReducedMotion } from "svelte/motion";
	import { fade } from "svelte/transition";
	import { type Icon as IconType } from "virtual:icons";
</script>

{#snippet item(icon: IconType, name: string, path: string)}
	<li>
		<a
			href={path}
			class={[
				"font-semibold px-4 flex items-center gap-3 py-2 transition hover:bg-primary/10 hover:text-primary",
			]}
		>
			<Icon name={icon} size="1.25em" />
			{name}
		</a>
	</li>
{/snippet}

<div class="p-4">
	<h2 class="text-2xl py-2">Server</h2>
	<ul class="border bg-base rounded-theme border-base-content/10">
		<li>
			<a
				href="/settings/directories"
				class="font-semibold px-4 flex items-center gap-3 py-2 transition hover:bg-primary/10 hover:text-primary"
			>
				<Icon name="folder" size="1.25em" />
				Directories
			</a>
		</li>
		<li>
			<a
				href="/settings/jobs"
				class="font-semibold px-4 flex items-center gap-3 py-2 transition hover:bg-primary/10 hover:text-primary"
			>
				<Icon name="play" size="1.25em" />
				Jobs
			</a>
		</li>
	</ul>
</div>
