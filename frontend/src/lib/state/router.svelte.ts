import type { IconKey } from "@lib/icons";
import type { MatchFunction, ParamData } from "path-to-regexp";
import type { Component } from "svelte";
import type { AppState } from "../app.svelte";

import { match } from "path-to-regexp";

interface Route extends PageProps {
	matcher: MatchFunction<ParamData>;
	parentIndex?: number;
}

export class Router {
	#routes: Array<Route> = $state([]);

	constructor(pages: Array<PageDefinition>) {
		this.addPages(pages);
	}

	addPages(pages: Array<PageDefinition>) {
		for (const page of pages) {
			const route = this.#route(page);
			this.#routes.push(route);

			const parentIndex = this.#routes.length - 1;
			if (page.children?.length) {
				for (const child of page.children) {
					this.#routes.push(this.#route(child, parentIndex));
				}
			}
		}
	}

	resolvePage(path: string): ResolvedPage | undefined {
		for (const [index, route] of this.#routes.entries()) {
			const { matcher, parentIndex, ...rest } = route;
			const match = matcher(path);
			if (!match) {
				continue;
			}

			return {
				...rest,
				index,
				params: match.params,
				children: () => {
					return this.#routes
						.filter((route) => route.parentIndex === index)
						.map((route) => this.#routeToPage(route, index));
				},
				parent: () => {
					if (index === 0 || !parentIndex) {
						return;
					}

					return this.#routeToPage(this.#routes[parentIndex], parentIndex);
				},
			};
		}
	}

	#route(page: PageDefinition, parentIndex?: number): Route {
		const { children, ...props } = page;
		return {
			matcher: match(
				parentIndex
					? `${this.#routes[parentIndex].path.replace(/\/$/, "")}/${props.path.replace(/^\//, "")}`
					: props.path,
				{
					end: !(children?.length && children.length > 0),
				},
			),
			...props,
			parentIndex,
		};
	}

	#routeToPage(route: Route, index: number): Page {
		const { matcher: _, ...rest } = route;
		return {
			children: () => {
				return this.#routes
					.filter((route) => route.parentIndex === index)
					.map((route) => this.#routeToPage(route, index));
			},
			parent: () => {
				if (index === 0 || route.parentIndex === undefined) {
					return;
				}

				return this.#routeToPage(this.#routes[route.parentIndex], route.parentIndex);
			},
			...rest,
		};
	}

	get pages() {
		return this.#routes.map((route, index) => this.#routeToPage(route, index));
	}
}

export interface PageComponentProps {
	app: AppState;
	visible: boolean;
	params?: ParamData;
	[key: string]: unknown;
}

export interface PageProps {
	path: string;
	name?: string;
	display?: boolean;
	hideHeader?: boolean;
	hideNavigation?: boolean;
	displayEditor?: boolean;
	icon?: IconKey;
	callback?: () => void;
	component?: Component<PageComponentProps>;
}

export interface Page extends PageProps {
	children(): Array<Page>;
	parent(): Page | undefined;
}

export interface ResolvedPage extends Page {
	index: number;
	params: ParamData;
}

export interface PageDefinition extends PageProps {
	children?: Array<PageDefinition>;
}
