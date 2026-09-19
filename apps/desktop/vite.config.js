import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
// @ts-expect-error type error without @types/node package
import process from "node:process";
const host = process.env.TAURI_DEV_HOST;

// Tailwind via PostCSS (@tailwindcss/postcss) — avoids @tailwindcss/vite
// mis-parsing Svelte virtual style modules as CSS.
export default defineConfig(() => ({
  plugins: [sveltekit()],
  clearScreen: false,
  css: {
    postcss: "./postcss.config.js",
  },
  server: {
    port: 1420,
    strictPort: true,
    host: host || "127.0.0.1",
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
