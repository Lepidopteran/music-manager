<script lang="ts">
	const {...rest} = $props();
</script>

<div class={`overflow-hidden ${rest.class}`}>
  <svg
    width="512"
    height="512"
    class="size-full"
    viewBox="0 0 512 512"
    version="1.1"
		role="img"
		aria-label="Missing Cover Art"
  >
    <defs id="defs1">
      <filter
        id="b"
        width="1.0790255"
        height="1.0790255"
        x="-0.039512728"
        y="-0.039512728"
        style="color-interpolation-filters:sRGB"
      >
        <feGaussianBlur stdDeviation="5.433" id="feGaussianBlur1" />
      </filter>
    </defs>
    <rect style="fill:#bbbbbb" id="bg" width="512" height="512" />
    <circle
      cx="256"
      cy="256"
      r="175"
      style="opacity:0.25;fill:#000000;filter:url(#b)"
      id="shadow"
    />
    <g id="disc">
      <circle
        cx="256"
        cy="256"
        r="165"
        style="opacity:1;fill:#eeeeee;fill-opacity:0.325;filter:url(#b)"
        id="discUnder"
      />
      <circle
        style="opacity:0.75;fill:#777777"
        id="discPlate"
        cx="256"
        cy="256"
        r="175"
      />
      <circle cx="256" cy="256" r="95" style="fill:#555555" id="discLabel" />
      <circle style="fill:#bbbbbb" id="discHole" cx="256" cy="256" r="42.5" />
    </g>
    <rect
      style="mix-blend-mode:multiply; fill:currentColor"
      id="color"
      width="512"
      height="512"
      x="0"
      y="0"
    />
  </svg>
</div>
