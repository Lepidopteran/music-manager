<script lang="ts">
  import type { Album, Song } from "@lib/models";
  import Cover from "./Cover.svelte";
  import { isSong } from "@lib/utils/model_guards";

  const excludedFields = ["title", "artist", "id", "path", "parentPath"];

  interface Props {
    selectedItem: Album | Song | null;
    [props: string]: unknown;
  }

  function renameField(key: string) {
    return key
      .replace(/([A-Z])/g, " $1")
      .replace(/^./, (str) => str.toUpperCase());
  }

  let { selectedItem, ...rest }: Props = $props();

  let imageHeight: number | null | undefined = $state();
  let imageWidth: number | null | undefined = $state();
  let failedToLoad = $state(false);

  function onCoverError() {
    failedToLoad = true;
  }

  function onCoverLoad() {
    failedToLoad = false;
  }

  function mapTracksToFields(tracks: Array<Song>): Map<string, string> {
    if (!tracks.length) return new Map();

    const map = new Map<string, string>();
    const first = tracks.at(0);
    const rest = tracks.slice(1);

    if (!rest.length) {
      for (const [key, value] of Object.entries(first as Song)) {
        if (!value || excludedFields.includes(key)) continue;
        map.set(key, value);
      }

      return map;
    }

    for (const track of rest) {
      for (const [key, value] of Object.entries(track)) {
        if (!value || excludedFields.includes(key)) continue;

        if (value === first?.[key as keyof Song]) {
          map.set(key, value);
        } else {
          map.set(key, `Different across (${tracks.length}) tracks`);
        }
      }
    }

    return map;
  }
</script>

<div
  class="flex flex-col items-center justify-center gap-2 relative h-full overflow-y-auto pt-2 bg-base-100"
  {...rest}
>
  {#if selectedItem}
    <div class="flex flex-col items-center text-sm">
      <Cover
        lazy={false}
        bind:imageHeight
        bind:imageWidth
        onError={onCoverError}
        onLoading={onCoverLoad}
        onLoad={onCoverLoad}
        item={selectedItem}
        class="mb-1 rounded-theme shadow-lg shadow-black/25"
      />

      {#if !imageHeight && !imageWidth && !failedToLoad}
        <div
          class="w-24 bg-base-950/25 text-transparent motion-safe:animate-pulse rounded-theme-lg"
          aria-hidden="true"
        >
          x
        </div>
      {:else}
        <p
          aria-hidden={failedToLoad || (!imageHeight && !imageWidth)}
          aria-label={`Cover art size ${imageWidth} by ${imageHeight}.`}
          class={`duration-300 ease-in-out text-base-950/50 ${failedToLoad ? "invisible pointer-events-none" : ""}`}
        >
          {imageHeight} x {imageWidth}
        </p>
      {/if}
    </div>
    <h2 class="text-2xl font-bold text-center">{selectedItem.title}</h2>
    <p class="text-center">{selectedItem.artist}</p>
    <div class="flex flex-col gap-2 w-full md:w-1/2">
      {#if isSong(selectedItem)}
        {#each Object.entries(selectedItem) as [key, value]}
          {#if value && !excludedFields.includes(key)}
            <div
              class="p-2 bg-base/25 rounded-theme shadow-black/25 inset-shadow-sm inset-shadow-highlight/25 shadow grow shrink-0"
            >
              <p class="text-sm font-bold text-base-950/50">
                {renameField(key)}
              </p>
              <p class="text truncate">{value}</p>
            </div>
          {/if}
        {/each}
      {:else}
        {#each mapTracksToFields(selectedItem.tracks).entries() as [key, value]}
          <div
            class="p-2 bg-base/25 rounded-theme shadow-black/25 inset-shadow-sm inset-shadow-highlight/25 shadow grow shrink-0"
          >
            <p class="text-sm font-bold text-base-950/50">
              {renameField(key)}
            </p>
            <p class="text truncate">
              {value}
            </p>
          </div>
        {/each}
      {/if}
    </div>
  {/if}
</div>
