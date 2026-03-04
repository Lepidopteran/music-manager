import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { resolve } from "node:path";
import { defineConfig } from "vitest/config";
import icons from "./plugins/icons";

// https://vite.dev/config/
export default defineConfig({
	root: ".",
	resolve: {
		alias: {
			"@api": resolve(__dirname, "src/lib/api"),
			"@bindings": resolve(__dirname, "src/lib/bindings"),
			"@actions": resolve(__dirname, "src/lib/actions"),
			"@assets": resolve(__dirname, "src/assets"),
			"@components": resolve(__dirname, "src/components"),
			"@state": resolve(__dirname, "src/state"),
			"@lib": resolve(__dirname, "src/lib"),
			"@pages": resolve(__dirname, "src/pages"),
			"@utils": resolve(__dirname, "src/lib/utils"),
		},
	},
	server: {
		proxy: {
			"/api": { target: "http://localhost:3000", changeOrigin: true },
		},
	},
	build: {
		outDir: "../dist",
		emptyOutDir: true,
	},
	plugins: [icons(), svelte(), tailwindcss()],
	test: {
		expect: { requireAssertions: true },
		projects: [
			{
				extends: "./vite.config.ts",
				resolve: {
					conditions: ["browser", "node"],
				},
			},
		],
	},
});
