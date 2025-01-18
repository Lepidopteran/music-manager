<script lang="ts">
  interface Props {
    value?: number;
    max?: number;
    min?: number;
    high?: number;
    low?: number;
    [key: string]: unknown;
  }

  let {
    value = 0,
    max = 100,
    min = 0,
    high = 100,
    low = 0,
    ...rest
  }: Props = $props();
</script>

<div class="h-2 w-full rounded-theme-sm bg-base-400 overflow-hidden">
  <div
		{...rest}
    class={`h-2 bg-primary transition-all duration-500 ease-in-out ${rest.class}`}
    style={`width: ${((value - min) / (max - min)) * 100}%`}
    data-value={value}
    data-high={high <= value || undefined}
    data-low={low >= value || undefined}
    data-based={value <= high || value >= low || undefined}
  ></div>
</div>
