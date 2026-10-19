import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import process from "node:process";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter(),
    paths: {
      // "/gnirehtet-gui" on GitHub Pages; empty for a custom domain or locally.
      base: process.env.BASE_PATH ?? "",
    },
  },
};

export default config;
