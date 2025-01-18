import type { Action } from "svelte/action";


export const createTree: Action = (node: HTMLElement) => {
  const groups = node.querySelectorAll(
    "details",
  ) as NodeListOf<HTMLDetailsElement>;

  node.role = "tree";
  node.ariaMultiSelectable = "false";

  for (const group of groups) {
    const items = group.querySelectorAll(
      "[role=treeitem]",
    ) as NodeListOf<HTMLOptionElement>;

		group.dataset.index = Array.from(groups).indexOf(group).toString()

		group.dataset.expanded = group.open ? "true" : "false"
		group.ariaExpanded = group.open ? "true" : "false"

		group.tabIndex = 0

    for (const item of items) {
      item.ariaSelected = "false";
      item.dataset.selected = "false";
			item.tabIndex = -1
    }
  }

  $effect(() => {
    const keydownHandler = (event: KeyboardEvent) => {
			const { key } = event;
			const target = event.target as HTMLDetailsElement

			switch (key) {
				case " ":
				case "Enter": {
					event.preventDefault();
					target.open = !target.open;
					break;
				}
				case "ArrowUp": {
					event.preventDefault();
					const previousGroup = groups[Math.max(0, Array.from(groups).indexOf(target) - 1)];
					previousGroup.open = true;
					previousGroup.focus();

					break
				}
				case "ArrowDown":
					event.preventDefault();
					break;
				case "Home":
					event.preventDefault();
					Array.from(groups)[0].open = true;
					break;
				case "End":
					event.preventDefault();
					Array.from(groups)[Array.from(groups).length - 1].open = true;
					break;

			}
			
		};
    const toggleDetails = (event: Event) => {
      const group = event.target as HTMLDetailsElement;
      group.ariaExpanded = group.open ? "true" : "false";
      group.dataset.expanded = group.open ? "true" : "false";
    };

    for (const group of groups) {
      const items = group.querySelectorAll(
        "[role=treeitem]",
      ) as NodeListOf<HTMLElement>;

      let currentIndex = -1;

      group.addEventListener("toggle", toggleDetails);
      group.addEventListener("keydown", keydownHandler);

      for (const item of items) {
        item.addEventListener("keydown", (event: KeyboardEvent) => {
					const { key } = event
					const target = event.target as HTMLInputElement

          if (key === "ArrowUp") {
            event.preventDefault();
						const previousIndex = Math.max(0, currentIndex - 1);
            const previousItem = items[previousIndex];

						currentIndex = previousIndex;
						previousItem.focus();
          }
          if (key === "ArrowDown") {
            event.preventDefault();
            currentIndex = Math.min(items.length - 1, currentIndex + 1);
            items[currentIndex].focus();
          }
        });

				item.addEventListener("click", () => {
					item.ariaSelected = "true";
					item.dataset.selected = "true";
					item.focus();

					currentIndex = Array.from(items).indexOf(item);
				});
				item.addEventListener("focus", handleFocus);
				item.addEventListener("blur", handleBlur);
      }
    }

    return () => {
      for (const group of groups) {
        group.removeEventListener("toggle", toggleDetails);
        group.removeEventListener("keydown", keydownHandler);
      }
    };
  });
};

function handleFocus(event: FocusEvent) {
	const target = event.target as HTMLInputElement;
	target.ariaSelected = "true";
	target.dataset.selected = "true";
}

function handleBlur(event: FocusEvent) {
	const target = event.target as HTMLInputElement;
	target.ariaSelected = "false";
	target.dataset.selected = "false";
}
