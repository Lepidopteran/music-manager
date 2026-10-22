<script lang="ts">
	import Icon from "#components/Icon.svelte";
	import { type Icon as IconType } from "virtual:icons";

	type SettingsPage = {
		name: string;
		icon: IconType;
		path: string;
	};

	const settingPages: Array<SettingsPage> = [
		{
			name: "Directories",
			icon: "folder",
			path: "directories",
		},
		{
			name: "Jobs",
			icon: "play",
			path: "jobs",
		},
	];
</script>

<div class="p-4">
	<h2 class="text-2xl py-2">Server</h2>
	<ul class="border bg-base rounded-theme border-base-content/10">
		{#each settingPages as { name, icon, path }}
			<li>
				<a
					href="/settings/{path}"
					class={[
						"font-semibold px-4 flex items-center gap-3 py-2 transition hover:bg-primary/10 hover:text-primary",
					]}
				>
					<Icon name={icon} size="1.25em" />
					{name}
				</a>
			</li>
		{/each}
	</ul>
</div>
