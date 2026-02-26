import { expect, test } from "vitest";

import { Router } from "./router.svelte";

const { pages } = new Router([
	{
		path: "/",
		name: "home",
		children: [
			{
				path: "/about",
				name: "about",
			},
		],
	},
]);

$inspect(pages);

test("Child can get parent", () => {
	expect(pages[1].parent()?.name).toEqual(pages[0].name);
});

test("Parent can get children", () => {
	expect(pages[0].children()[0].name).toEqual(pages[1].name);
});
