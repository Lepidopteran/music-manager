import { icons as mingcute } from "@iconify-json/mingcute";
import type metadata from "@iconify-json/mingcute/metadata.json";
import { getIconData, IconifyIcon, parseIconSet } from "@iconify/utils";
import path from "node:path";
import { AST, parse } from "svelte/compiler";
import { match } from "ts-pattern";
import type { Plugin, ResolvedConfig } from "vite";

const virtualModuleId = "virtual:icons";
const resolvedVirtualModuleId = "\0" + virtualModuleId;

type MingcuteCategories = typeof metadata.categories;
function declaration(keys: Array<string>) {
	return `declare module "${virtualModuleId}" {
	import { IconifyIcon } from "@iconify/types";
	export const icons: Record<Icon, IconifyIcon>;
	export { iconToSVG } from "@iconify/utils";
	export type Icon = \n\t\t${keys.map((key) => `| "${key}"`).join("\n\t\t")};
}
`;
}

export default function icons(): Plugin {
	const available: Array<string> = [];
	const icons: Record<string, IconifyIcon> = {};
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

		async buildStart() {
			const { icons, metadata } = await import("@iconify-json/mingcute");
			const { categories } = metadata;

			const { Zodiac: _zodiac, Crypto: _crypto, ...included } = categories as MingcuteCategories;

			parseIconSet(icons, (name) => {
				if (name.endsWith("-line") || Object.values(included).every((category) => !category.includes(name))) {
					return;
				}

				available.push(name.replace(/-fill/g, "").replace(/-/g, "_"));
			});

			const outputDirectory = path.resolve(config.root, ".muusik");
			const outputFile = path.join(outputDirectory, "icons.d.ts");

			await this.fs.mkdir(outputDirectory, { recursive: true });
			await this.fs.writeFile(
				outputFile,
				declaration(available),
			);
		},

		transform(code, id) {
			if (id.endsWith(".svelte")) {
				const ast = parse(code, {
					modern: true,
				});

				walk(ast.fragment, (node) => {
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
							icons[value] = getIconData(mingcute, `${value.replace("_", "-")}-fill`) as IconifyIcon;
						}
					}
				});
			}
		},

		load(id) {
			if (id === resolvedVirtualModuleId) {
				this.info(`Bundled icons: ${Object.keys(icons).join(", ")}`);
				return `export const icons = ${JSON.stringify(icons)};\nexport { iconToSVG } from "@iconify/utils";`;
			}
		},
	};
}

type Node = AST.ElementLike | AST.Block | AST.Text | AST.Comment | AST.Tag;

function walk(fragment: AST.Fragment, callback: (node: Node) => void) {
	for (const childNode of fragment.nodes) {
		callback(childNode);

		match(childNode)
			.with(
				{ type: "Component" },
				{ type: "TitleElement" },
				{ type: "RegularElement" },
				{ type: "KeyBlock" },
				({ fragment }) => walk(fragment, callback),
			)
			.with({ type: "EachBlock" }, ({ body, fallback }) => {
				walk(body, callback);
				if (fallback) {
					walk(fallback, callback);
				}
			})
			.with({ type: "SnippetBlock" }, ({ body }) => {
				walk(body, callback);
			})
			.with({ type: "IfBlock" }, ({ consequent, alternate }) => {
				walk(consequent, callback);
				if (alternate) {
					walk(alternate, callback);
				}
			})
			.otherwise(() => {});
	}
}
