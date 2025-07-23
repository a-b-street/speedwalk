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
      },
    },
  },
  plugins: [svelte(), wasm()],
  server: {
    fs: {
      allow: ["./", "../backend/pkg"]
    }
  }
})
