// Tauri doesn't have a Node.js server to do proper SSR
// so we use adapter-static with a fallback to index.html to put the site in SPA mode
// See: https://svelte.dev/docs/kit/single-page-apps
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import process from "node:process";

// The landing site (apps/site) embeds this UI as a live demo. It builds it
// with DEMO_BASE set to the path the demo is served from, into its own folder
// so the Tauri build is left alone.
const demoBase = process.env.DEMO_BASE;

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      pages: demoBase ? "build-demo" : "build",
      fallback: "index.html",
    }),
    paths: {
      base: demoBase ?? "",
    },
  },
};

export default config;
