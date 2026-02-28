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
}

export interface RouteDefinition<M> {
	path: string;
	children?: Array<RouteDefinition<M>>;
	metadata?: M;
}

type RouterCallback<M> = (router: Router<M>, path: string, index: number) => void;

export interface RouterOptions<M> {
	onRouteAdd?: RouterCallback<M>;
	onRouteRemove?: RouterCallback<M>;
}

export class Router<M> implements RouterOptions<M> {
	#routes: Array<InternalRoute<M>> = [];
	onRouteAdd?: RouterCallback<M>;
	onRouteRemove?: RouterCallback<M>;

	constructor(
		routes: Array<RouteDefinition<M>>,
		options?: RouterOptions<M>,
	) {
		Object.assign(this, options);
		for (const page of routes) {
			this.addRoute(page);
		}
	}

	addRoute(def: RouteDefinition<M>) {
		const path = def.path;
		const route = this.#internalRoute(def);

		this.#routes.push(route);
		this.onRouteAdd?.(this, path, this.#routes.length - 1);

		const parentIndex = this.#routes.length - 1;
		if (def.children?.length) {
			for (const child of def.children) {
				const { path } = child;
				this.#routes.push(this.#internalRoute(child, parentIndex));
				this.onRouteAdd?.(this, path, this.#routes.length - 1);
			}
		}
	}

	removeRoute(path: string) {
		const index = this.#routes.findIndex((route) => route.path === path);
		this.removeRouteWithIndex(index);
	}

	removeRouteWithIndex(index: number) {
		for (
			const [childIndex, childPath] of this
				.#routes
				.filter((route) => route.parentIndex === index)
				.map(({ path }, index) => [index, path] as const)
		) {
			this.removeRouteWithIndex(childIndex);
			this.onRouteRemove?.(this, childPath, childIndex);
		}

		const { path } = this.#routes[index];
		this.#routes.splice(index, 1);
		this.onRouteRemove?.(this, path, index);
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
				params: match.params,
				children: () => {
					return this.#routes
						.filter((route) => route.parentIndex === index)
						.map((route) => this.#route(route, index));
				},
				parent: () => {
					if (index === 0 || !parentIndex) {
						return;
					}

					return this.#route(this.#routes[parentIndex], parentIndex);
				},
			};
		}
	}

	#internalRoute(route: RouteDefinition<M>, parentIndex?: number): InternalRoute<M> {
		const { children, ...props } = route;
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
