import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import process from "node:process";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter(),
    paths: {
      // Empty on the custom domain and locally; "/gnirehtet-gui" if it's ever
      // served from sawongam.github.io/gnirehtet-gui again.
      base: process.env.BASE_PATH ?? "",
    },
  },
};

export default config;
