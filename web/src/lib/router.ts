import type { MatchFunction, ParamData } from "path-to-regexp";

import { match } from "path-to-regexp";

interface InternalRoute<M> {
	path: string;
	matcher: MatchFunction<ParamData>;
	parentIndex?: number;
	metadata?: M;
}

export interface Route<M> {
	path: string;
	metadata?: M;
	children(): Array<Route<M>>;
	parent(): Route<M> | undefined;
}

export interface ResolvedRoute<M> extends Route<M> {
	index: number;
	params: ParamData;
	resolvedPath: string;
}

export interface RouteDefinition<M> {
	path: string;
	children?: Array<RouteDefinition<M>>;
	metadata?: M;
}

type RoutesUpdatedCallback<M> = (router: Router<M>) => void;

export interface RouterOptions<M> {
	/**
	 * Callback when a route has been added or deleted in the router.
	 */
	onRoutesUpdated?: RoutesUpdatedCallback<M>;
}

export class Router<M> implements RouterOptions<M> {
	#routes: Array<InternalRoute<M>> = [];
	onRoutesUpdated?: RoutesUpdatedCallback<M>;

	constructor(
		routes: Array<RouteDefinition<M>>,
		options?: RouterOptions<M>,
	) {
		Object.assign(this, options);
		for (const page of routes) {
			this.addRoute(page);
		}
	}

	hasRoute(path: string) {
		return this.#routes.some((route) => route.matcher(path));
	}

	getRouteIndex(path: string): number | null {
		const index = this.#routes.findIndex((route) => route.path === path);
		return index === -1 ? null : index;
	}

	addRouteWithParentPath(parentPath: string, def: RouteDefinition<M>) {
		const index = this.#routes.findIndex((route) => route.path === parentPath || route.matcher(parentPath));

		if (index === -1) {
			throw new Error(`Parent route not found: ${parentPath}`);
		}

		this.addRoute(def, index);
	}

	addRoute(def: RouteDefinition<M>, parentIndex?: number) {
		this.#routes.push(this.#internalRoute(def, parentIndex));
		this.onRoutesUpdated?.(this);

		const currentIndex = this.#routes.length - 1;
		if (def.children?.length) {
			for (const child of def.children) {
				this.addRoute(child, currentIndex);
			}
		}
	}

	removeRoute(path: string) {
		const index = this.#routes.findIndex((route) => route.path === path);
		this.removeRouteWithIndex(index);
	}

	removeRouteWithIndex(index: number) {
		for (
			const childIndex of this
				.#routes
				.filter((route) => route.parentIndex === index).map((_, index) => index)
		) {
			this.#routes.splice(childIndex, 1);
			this.onRoutesUpdated?.(this);
		}

		this.#routes.splice(index, 1);
		this.onRoutesUpdated?.(this);
	}

	resolve(path: string): ResolvedRoute<M> | undefined {
		for (const [index, route] of this.#routes.entries()) {
			const { matcher, parentIndex, ...rest } = route;
			const match = matcher(path);
			if (!match) {
				continue;
			}

			return {
				...rest,
				index,
				resolvedPath: match.path,
				params: match.params,
				children: () => {
					return this.#routes
						.filter((route) => route.parentIndex === index)
						.map((route, index) => this.#route(route, index));
				},
				parent: () => {
					if (parentIndex === undefined) {
						return;
					}

					return this.#route(this.#routes[parentIndex], parentIndex);
				},
			};
		}
	}

	#internalRoute(route: RouteDefinition<M>, parentIndex?: number): InternalRoute<M> {
		const { children: _, path, ...props } = route;

		const combinedPath = parentIndex !== undefined
			? buildPath([...this.#routes[parentIndex].path.split("/"), ...path.split("/")])
			: buildPath(path.split("/"));

		return {
			...props,
			parentIndex,
			path: combinedPath,
			matcher: match(combinedPath),
		};
	}

	#route(route: InternalRoute<M>, index: number): Route<M> {
		const { matcher: _, ...rest } = route;
		return {
			children: () => {
				return this.#routes
					.filter((route) => route.parentIndex === index)
					.map((route) => this.#route(route, index));
			},
			parent: () => {
				if (index === 0 || route.parentIndex === undefined) {
					return;
				}

				return this.#route(this.#routes[route.parentIndex], route.parentIndex);
			},
			...rest,
		};
	}

	get routes() {
		return this.#routes.map((route, index) => this.#route(route, index));
	}
}

export function buildPath(parts: Array<string>, absolute = true): string {
	return `${absolute ? "/" : ""}${parts.filter((part) => part !== "").join("/")}`;
}
