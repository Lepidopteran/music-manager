import type { Attachment } from "svelte/attachments";

export function resizeOccurred(
	callback: (entry: ResizeObserverEntry, observer: ResizeObserver) => void,
	options?: ResizeObserverOptions,
): Attachment {
	return (element) => {
		const observer = new ResizeObserver(([entry], observer) => {
			callback(entry, observer);
		});

		observer.observe(element, options);

		return () => observer.disconnect();
	};
}
