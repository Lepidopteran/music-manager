<script lang="ts">
	import Button from "#components/Button.svelte";
	import Icon from "#components/Icon.svelte";
	import { page } from "$app/state";

	const navigationPath = $derived(
		page.url.pathname.split("/").filter(Boolean) ?? [],
	);
</script>

<a
	href='/{navigationPath.length > 1 ? navigationPath.slice(0, -1).join("/") : "/"}'
>
	<Button title="Back" class="pointer-events-none shadow-lg">
		<Icon name="arrow_left" />
	</Button>
</a>
