import { describe, expect, test } from "vitest";

import { Router } from "./page";

const { routes: pages } = new Router([
	{
		path: "/",
		children: [
			{
				path: "/about",
			},
		],
	},
]);

describe("Router", () => {
	test("Child can get parent", () => {
		const [parent, child] = pages;
		const childParent = child.parent();
		expect(childParent).toBeDefined();
		expect(childParent?.path).toEqual(parent.path);
	});

	test("Parent can get children", () => {
		const [parent, child] = pages;
		expect(parent.children().length).toEqual(1);
		expect(parent.children()[0].path).toEqual(child.path);
	});
});
