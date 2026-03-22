import { describe, expect, test } from "vitest";

import { Router } from "./router";

describe("Router", () => {
	test("Child can get parent", () => {
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

		const [parent, child] = pages;
		const childParent = child.parent();
		expect(childParent).toBeDefined();
		expect(childParent?.path).toEqual(parent.path);
	});

	test("Parent can get children", () => {
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

		const [parent, child] = pages;
		expect(parent.children().length).toEqual(1);
		expect(parent.children()[0].path).toEqual(child.path);
	});

	// It's not what is sounds like
	test("Children can add children to parent", () => {
		const router = new Router([
			{
				path: "/",
				children: [],
			},
			{
				path: "/about",
				children: [
					{
						path: "/:id",
					},
				],
			},
		]);

		router.addRouteWithParentPath("/about/:id", {
			path: ":name",
		});

		const route = router.resolve("/about/1/kaisen");
		expect(route).toBeDefined();
		expect(route?.path).toEqual("/about/:id/:name");
		expect(route?.children().length).toEqual(0);

		router.addRouteWithParentPath(route!.path, {
			path: ":birthday",
		});

		expect(route?.params).toEqual({
			id: "1",
			name: "kaisen",
		});

		expect(route?.children().length).toEqual(1);

		const childChildRoute = router.resolve("/about/1/kaisen/2000-01-01");
		expect(childChildRoute).toBeDefined();
		expect(childChildRoute?.path).toEqual("/about/:id/:name/:birthday");
		expect(childChildRoute?.children().length).toEqual(0);

		expect(childChildRoute?.params).toEqual({
			id: "1",
			name: "kaisen",
			birthday: "2000-01-01",
		});
	});
});
