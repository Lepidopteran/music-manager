import { icons as mingcute } from "@iconify-json/mingcute";
import { getIconData, type IconifyIcon, parseIconSet } from "@iconify/utils";
import type { Node } from "estree";
import path from "node:path";
import { type AST, parse } from "svelte/compiler";
import { match } from "ts-pattern";
import type { Plugin, ResolvedConfig } from "vite";

import { walk } from "zimmerframe";

const virtualModuleId = "virtual:icons";
const resolvedVirtualModuleId = "\0" + virtualModuleId;

function declaration(keys: Array<string>) {
	return `declare module "${virtualModuleId}" {
	import { IconifyIcon } from "@iconify/types";
	export const icons: Record<Icon, IconifyIcon>;
	export { iconToSVG } from "@iconify/utils";
	export type Icon = \n\t\t${keys.map((key) => `| "${key}"`).join("\n\t\t")};
}
`;
}

export default function iconsPlugin(): Plugin {
	const available: Array<string> = [];
	const icons: Map<string, IconifyIcon> = new Map();
	let config: ResolvedConfig;

	return {
		name: "muusik-icons",

		resolveId(id) {
			if (id === virtualModuleId) {
				return resolvedVirtualModuleId;
			}
		},

		configResolved(resolvedConfig) {
			config = resolvedConfig;
		},

		handleHotUpdate({ server, file, modules }) {
			if (!file.endsWith(".svelte")) return;

			const mod = server.moduleGraph.getModuleById(resolvedVirtualModuleId);
			if (!mod) return;
			server.moduleGraph.invalidateModule(mod);

			return [...modules, mod];
		},

		transform(code, id) {
			if (id.endsWith(".svelte")) {
				const { fragment, instance, module: _module } = parse(code, {
					modern: true,
				});

				if (instance) {
					const { content } = instance;

					walk(content as Node, {}, {
						_(_, { next }) {
							next();
						},
						Literal(node) {
							const { value } = node;
							if (value && typeof value === "string" && available.includes(value)) {
								const icon = getIconData(
									mingcute,
									`${value.replace(/_/g, "-")}-fill`,
								);

								if (icon) {
									icons.set(value, icon);
								} else {
									console.warn(`Icon "${value}" not found`);
								}
							}
						},
					});
				}

				walkFragment(fragment, (node) => {
					if (node.type !== "Component") {
						return;
					}

					const iconValues = node
						.attributes
						.filter((attribute) =>
							attribute.type === "Attribute" && ["icon", "name"].includes(attribute.name)
							&& typeof attribute.value !== "boolean" && attribute.value
						)
						.flatMap((attribute) => {
							const value = (attribute as AST.Attribute).value;
							if (Array.isArray(value)) {
								return value
									.filter((value) => value.type === "Text")
									.map((value) => value.raw);
							}
						})
						.filter((value) => value !== undefined);

					for (
						const value of iconValues
					) {
						if (available.includes(value)) {
							icons.set(
								value,
								getIconData(
									mingcute,
									`${value.replace(/_/g, "-")}-fill`,
								) as IconifyIcon,
							);
						}
					}
				});
			}
		},

		load(id) {
			if (id === resolvedVirtualModuleId) {
				this.info(`Bundled icons: ${icons.keys().toArray().join(", ")}`);
				return `export const icons = ${
					JSON.stringify(Object.fromEntries(icons.entries()))
				};\nexport { iconToSVG } from "@iconify/utils";`;
			}
		},

		async buildStart() {
			const { icons } = await import("@iconify-json/mingcute");

			parseIconSet(icons, (name) => {
				if (name.endsWith("-line")) {
					return;
				}

				available.push(name.replace(/-/g, "_").replace(/_fill$/, ""));
			});

			const outputDirectory = path.resolve(config.root, ".muusik");
			const outputFile = path.join(outputDirectory, "icons.d.ts");

			await this.fs.mkdir(outputDirectory, { recursive: true });
			await this.fs.writeFile(
				outputFile,
				declaration(available),
			);
		},
	};
}

type SvelteNode = AST.ElementLike | AST.Block | AST.Text | AST.Comment | AST.Tag;

function walkFragment(fragment: AST.Fragment, callback: (node: SvelteNode) => void) {
	for (const childNode of fragment.nodes) {
		callback(childNode);

		match(childNode)
			.with(
				{ type: "Component" },
				{ type: "TitleElement" },
				{ type: "RegularElement" },
				{ type: "KeyBlock" },
				({ fragment }) => walkFragment(fragment, callback),
			)
			.with({ type: "EachBlock" }, ({ body, fallback }) => {
				walkFragment(body, callback);
				if (fallback) {
					walkFragment(fallback, callback);
				}
			})
			.with({ type: "SnippetBlock" }, ({ body }) => {
				walkFragment(body, callback);
			})
			.with({ type: "IfBlock" }, ({ consequent, alternate }) => {
				walkFragment(consequent, callback);
				if (alternate) {
					walkFragment(alternate, callback);
				}
			})
			.otherwise(() => {});
	}
}
