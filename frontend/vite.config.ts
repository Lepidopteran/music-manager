import adapter from "@sveltejs/adapter-static";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { playwright } from "@vitest/browser-playwright";
import { defineConfig } from "vitest/config";
import bundleIcons from "./plugins/icons.ts";

export default defineConfig({
	plugins: [
		bundleIcons(),
		tailwindcss(),
		sveltekit({
			alias: {
				"@api/*": "src/api/*",
				"@state": "src/state.ts",
				"@attachments/*": "src/attachments/*",
				"@assets/*": "src/assets/*",
				"@components/*": "src/components/*",
				"@lib/*": "src/lib/*",
				"@pages/*": "src/pages/*",
				"@utils/*": "src/utils/*",
			},
			typescript: {
				config: config => {
					config.include.push("../.muusik/*.ts");
				},
			},
			compilerOptions: {
				// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
				runes: ({ filename }) => filename.split(/[/\\]/).includes("node_modules") ? undefined : true,
			},
			outDir: "../dist",
			adapter: adapter({
				fallback: "index.html",
			}),
		}),
	],
	server: {
		proxy: {
			"/api": { target: "http://localhost:3000", changeOrigin: true },
		},
	},
	resolve: {
		tsconfigPaths: true,
	},
	test: {
		expect: { requireAssertions: true },
		projects: [
			{
				extends: "./vite.config.ts",
				test: {
					name: "client",
					browser: {
						enabled: true,
						provider: playwright(),
						instances: [{ browser: "chromium", headless: true }],
					},
					include: ["src/**/*.svelte.{test,spec}.{js,ts}"],
					exclude: ["src/lib/server/**"],
				},
			},

			{
				extends: "./vite.config.ts",
				test: {
					name: "server",
					environment: "node",
					include: ["src/**/*.{test,spec}.{js,ts}"],
					exclude: ["src/**/*.svelte.{test,spec}.{js,ts}"],
				},
			},
		],
	},
});
