import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import process from "node:process";
import { fileURLToPath } from "node:url";

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
        // The demo's paths are fixed when it's built. Rebuild it if it's
        // missing or was built for another base, e.g. by a Pages build.
        const index = new URL("./static/demo/index.html", import.meta.url);
        const backend = `src="${process.env.BASE_PATH ?? ""}/demo/backend.js"`;
        if (!existsSync(index) || !readFileSync(index, "utf8").includes(backend)) {
          execFileSync("node", ["scripts/build-demo.mjs"], {
            cwd: fileURLToPath(new URL(".", import.meta.url)),
            stdio: "inherit",
          });
        }
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
