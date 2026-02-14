import type { Attachment } from "svelte/attachments";
import { match } from "ts-pattern";

export function preventKeyboardScrollHandler(event: KeyboardEvent): void {
	match(event.key).with("ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", () =>
		event.preventDefault(),
	);
}

export const preventKeyboardScroll: Attachment<HTMLElement> = (element) => {
	element.addEventListener("keydown", preventKeyboardScrollHandler);

	return () => {
		element.removeEventListener("keydown", preventKeyboardScrollHandler);
	};
};
