<script lang="ts">
	interface Props {
		label?: string;
		value?: string;
		placeholder?: string;
		pattern?: string;
		required?: boolean;
		[rest: string]: unknown;
	}

	let {
		label,
		value = $bindable(""),
		placeholder,
		pattern,
		required = false,
		...rest
	}: Props = $props();
</script>

<div class="flex flex-col">
	{#if label}
		<div class="flex justify-between">
			<label class="label" for="text-input">
				<span class="text-base-950">
					{label}:
					{#if required}
						<span class="text-red-500 font-bold select-none">*</span>
					{/if}
				</span>
			</label>
		</div>
	{/if}
	<input
		class="bg-base-300 px-2 py-1 inset-shadow-sm inset-shadow-black/25 rounded-theme focus-visible:outline-1 focus-visible:outline-primary"
		type="text"
		id="text-input"
		{required}
		{pattern}
		{placeholder}
		bind:value
		{...rest}
	/>
</div>
