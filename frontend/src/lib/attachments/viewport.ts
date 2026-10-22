import type { Attachment } from "svelte/attachments";

export function isInViewport(
	callback: (inViewport: boolean) => void,
	options?: IntersectionObserverInit,
): Attachment {
	return (element) => {
		const observer = new IntersectionObserver(([entry]) => {
			callback(entry.isIntersecting);
		}, options);

		observer.observe(element);

		return () => observer.disconnect();
	};
}
