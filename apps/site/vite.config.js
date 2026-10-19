import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import { readFileSync } from "node:fs";

// The app version, from the same place scripts/release-info.mjs reads it. The
// page asks GitHub for the latest release when it opens (src/lib/release.svelte.ts);
// this is what it shows until then, or if GitHub can't be reached.
const cargo = readFileSync(new URL("../../Cargo.toml", import.meta.url), "utf8");
const version = cargo.match(/^\[workspace\.package\][^[]*?^version\s*=\s*"([^"]+)"/ms)?.[1];
if (!version) throw new Error("No [workspace.package] version in Cargo.toml");

export default defineConfig({
  plugins: [
    {
      // The demo is a static SPA in static/demo/; serve its index for /demo/.
      name: "demo-index",
      configureServer(server) {
        server.middlewares.use((req, _res, next) => {
          if (req.url && /\/demo\/(\?.*)?$/.test(req.url)) req.url = req.url.replace("/demo/", "/demo/index.html");
          next();
        });
      },
    },
    sveltekit(),
  ],
  define: {
    __APP_VERSION__: JSON.stringify(version),
  },
  server: {
    port: 1440,
    strictPort: true,
  },
});
