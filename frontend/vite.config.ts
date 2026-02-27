import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { resolve } from "node:path";
import { defineConfig } from "vitest/config";

const alias = {
	"@api": resolve(__dirname, "src/lib/api"),
	"@bindings": resolve(__dirname, "src/lib/bindings"),
	"@actions": resolve(__dirname, "src/lib/actions"),
	"@assets": resolve(__dirname, "src/assets"),
	"@components": resolve(__dirname, "src/components"),
	"@state": resolve(__dirname, "src/state"),
	"@lib": resolve(__dirname, "src/lib"),
	"@pages": resolve(__dirname, "src/pages"),
	"@utils": resolve(__dirname, "src/lib/utils"),
};

// https://vite.dev/config/
export default defineConfig({
	root: ".",
	resolve: {
		alias,
	},
	server: {
		proxy: {
			"/api": { target: "http://localhost:3000", changeOrigin: true },
		},
	},
	build: { outDir: "../dist", emptyOutDir: true },
	plugins: [svelte(), tailwindcss()],
	test: {
		expect: { requireAssertions: true },
		projects: [
			{
				extends: "./vite.config.ts",
				resolve: {
					alias,
					conditions: ["browser", "node"],
				},
			},
		],
	},
});
