import type { Attachment } from "svelte/attachments";

export function mutationOccurred(
	callback: (mutations: MutationRecord[], observer: MutationObserver) => void,
	options?: MutationObserverInit,
): Attachment {
	return (element) => {
		const observer = new MutationObserver((entries, observer) => {
			callback(entries, observer);
		});

		observer.observe(element, options);

		return () => observer.disconnect();
	};
}

export function attributeChanged(
	callback: (name: string, oldValue: string | null, newValue: string | null) => void,
	options?: MutationObserverInit,
): Attachment {
	return mutationOccurred((mutations, _) => {
		for (const mutation of mutations) {
			if (mutation.type === "attributes") {
				callback(
					mutation.attributeName!,
					mutation.oldValue,
					(mutation.target as Element).getAttribute(mutation.attributeName!),
				);
			}
		}
	}, {
		attributes: true,
		...options,
	});
}
