<script lang="ts">
	import { page } from "$app/state";
	import Button from "#components/Button.svelte";
	import Icon from "#components/Icon.svelte";

	const navigationPath = $derived(
		page.url.pathname.split("/").filter(Boolean) ?? [],
	);
</script>

<a href={navigationPath.slice(0, -1).join("/").replace(/\/$/, "")}>
	<Button title="Back" class="pointer-events-none shadow-lg">
		<Icon name="arrow_left" />
	</Button>
</a>
