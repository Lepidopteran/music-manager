import { MediaQuery } from "svelte/reactivity";
export const onSmallScreen = new MediaQuery("(max-width: 639px)");
export const onMediumScreen = new MediaQuery("(min-width: 640px)");
export const onLargeScreen = new MediaQuery("(min-width: 1024px)");
export const onExtraLargeScreen = new MediaQuery("(min-width: 1280px)");
export const onExtraExtraLargeScreen = new MediaQuery("(min-width: 1536px)");
