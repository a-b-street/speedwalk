import { defineConfig } from "vite";
import { resolve } from "path";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import wasm from "vite-plugin-wasm";

export default defineConfig({
  base: "/speedwalk/",
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        land: resolve(__dirname, "land.html"),
      },
    },
  },
  plugins: [svelte(), wasm()],
  server: {
    port: 5174,
    fs: {
      allow: ["./", "../backend/pkg"]
    }
  }
})
