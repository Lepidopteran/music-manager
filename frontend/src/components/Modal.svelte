<script lang="ts">
  import type { Snippet } from "svelte";
  import type { Action } from "svelte/action";
    import Button from "./Button.svelte";

  let dialog: HTMLDialogElement;

  interface Props {
    children?: Snippet;
    title?: string;
    showTitle?: boolean;
    showClose?: boolean;
		canSoftClose?: boolean;
    open?: boolean;
    [key: string]: unknown;
  }

  let {
    children,
    title,
    showTitle = true,
    showClose = true,
		canSoftClose = true,
    open = $bindable(false),
    ...rest
  }: Props = $props();

  const closeAction: Action = (node) => {
    const close = (event: MouseEvent) => {
      const { left, right, top, bottom } = node.getBoundingClientRect();
      if (
        event.clientX < left ||
        event.clientX > right ||
        event.clientY < top ||
        event.clientY > bottom 
      ) {
				if (!canSoftClose) return;
        open = false;
      }
    };

    $effect(() => {
      node.addEventListener("click", close);

      return () => {
        node.removeEventListener("click", close);
      };
    });
  };

  $effect(() => {
    open ? dialog.showModal() : dialog.close();
  });
</script>

<dialog
	{...rest}
  class={`m-auto max-sm:w-11/12 bg-base-200 max-w-lg rounded-theme-lg shadow-lg inset-shadow-xs inset-shadow-highlight/25 backdrop:backdrop-blur ${rest.class || ""}`}
  bind:this={dialog}
  use:closeAction
>
  {#if showTitle}
    <div class="flex gap-4 py-2 px-4 items-center justify-between shadow">
      <h1 class="text-xl font-bold">{title}</h1>
      {#if showClose}
        <Button
          onclick={() => (open = false)}
          class="btn btn-ghost"
          aria-label="Close"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="1.5"
            stroke="currentColor"
            class="w-6 h-6"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </Button>
      {/if}
    </div>
  {/if}
  <div class="h-full p-4 overflow-y-auto">
    {@render children?.()}
  </div>
</dialog>
