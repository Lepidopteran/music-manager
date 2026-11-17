import type { Attachment } from "svelte/attachments";

export function dragSelection<T extends HTMLElement>(onDrag: (event: PointerEvent) => boolean): Attachment<T> {
	return (element) => {
		let dragged = false;
		let dragging = false;
		let startX = 0;
		let startY = 0;
		const DRAG_PIXEL_THRESHOLD = 3;

		const start = (event: PointerEvent) => {
			if (event.pointerType === "touch") return;

			dragging = true;
			startX = event.clientX;
			startY = event.clientY;

			event.preventDefault();
		};

		const drag = (event: PointerEvent) => {
			if (!dragging || dragged) return;

			const dx = Math.abs(event.clientX - startX);
			const dy = Math.abs(event.clientY - startY);

			if (dx > DRAG_PIXEL_THRESHOLD || dy > DRAG_PIXEL_THRESHOLD) {
				dragged = onDrag(event);
				event.preventDefault();
			}
		};

		const release = () => {
			dragged = false;
			dragging = false;
		};

		element.addEventListener("pointerdown", start);
		element.addEventListener("pointermove", drag);
		element.addEventListener("pointerup", release);
		element.addEventListener("pointercancel", release);

		window.addEventListener("pointerdown", start);
		window.addEventListener("pointerup", release);
		window.addEventListener("pointercancel", release);

		return () => {
			element.removeEventListener("pointerdown", start);
			element.removeEventListener("pointermove", drag);
			element.removeEventListener("pointerup", release);
			element.removeEventListener("pointercancel", release);
			window.removeEventListener("pointerdown", start);
			window.removeEventListener("pointerup", release);
			window.removeEventListener("pointercancel", release);
		};
	};
}
