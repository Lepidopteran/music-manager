import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vitest/config";
import bundleIcons from "./plugins/icons";

// https://vite.dev/config/
export default defineConfig({
	root: ".",
	server: {
		proxy: {
			"/api": { target: "http://localhost:3000", changeOrigin: true },
		},
	},
	resolve: {
		tsconfigPaths: true,
	},
	build: {
		outDir: "../dist",
		emptyOutDir: true,
	},
	plugins: [bundleIcons(), tailwindcss(), svelte()],
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
