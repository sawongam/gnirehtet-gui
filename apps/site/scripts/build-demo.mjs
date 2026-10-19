#!/usr/bin/env node
// Builds the desktop UI (apps/desktop) for the live demo on the landing page:
// served from <BASE_PATH>/demo/, with demo/backend.js loaded before the app
// so it runs against a scripted backend instead of Tauri.
//
// Usage: BASE_PATH=/gnirehtet-gui node scripts/build-demo.mjs

import { execFileSync } from "node:child_process";
import { cpSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const SITE = join(dirname(fileURLToPath(import.meta.url)), "..");
const DESKTOP = join(SITE, "../desktop");
const OUT = join(SITE, "static/demo");
const base = `${process.env.BASE_PATH ?? ""}/demo`;

execFileSync("pnpm", ["exec", "vite", "build"], {
  cwd: DESKTOP,
  stdio: "inherit",
  env: { ...process.env, DEMO_BASE: base },
});

rmSync(OUT, { recursive: true, force: true });
cpSync(join(DESKTOP, "build-demo"), OUT, { recursive: true });
cpSync(join(SITE, "demo/backend.js"), join(OUT, "backend.js"));

// A classic script in <head> runs before SvelteKit's module scripts start the app.
const index = join(OUT, "index.html");
const html = readFileSync(index, "utf8");
if (!html.includes("<head>")) throw new Error("No <head> in the demo's index.html");
writeFileSync(
  index,
  html.replace("<head>", `<head>\n    <script src="${base}/backend.js"></script>\n    <meta name="robots" content="noindex" />`),
);
console.log(`Demo written to ${OUT} (served from ${base}/)`);
