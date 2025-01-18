import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";

import { resolve } from "node:path";

// https://vite.dev/config/
export default defineConfig({
  root: ".",
  resolve: {
    alias: {
      "@api": resolve(__dirname, "src/lib/api"),
			"@actions": resolve(__dirname, "src/lib/actions"),
      "@assets": resolve(__dirname, "src/assets"),
      "@components": resolve(__dirname, "src/components"),
      "@lib": resolve(__dirname, "src/lib"),
      "@pages": resolve(__dirname, "src/pages"),
      "@utils": resolve(__dirname, "src/lib/utils"),
    },
  },
  server: {
    proxy: {
      "/api": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
    },
  },
  build: {
    outDir: "../dist",
    emptyOutDir: true,
  },
  plugins: [svelte(), tailwindcss()],
});
