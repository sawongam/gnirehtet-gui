#!/usr/bin/env node
// Checks that every place holding the app version agrees, and prints it.
//   apps/desktop/src-tauri/tauri.conf.json
//   apps/desktop/src-tauri/Cargo.toml (and its Cargo.lock entry)
//   apps/desktop/package.json (and package-lock.json)
//
// With --notes <file>, also writes the CHANGELOG.md section for that version
// to <file>, failing if the section is missing or empty.
//
// Usage: node scripts/release-info.mjs [--notes <file>]

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(join(ROOT, path), "utf8");
const json = (path) => JSON.parse(read(path));

function cargoVersion() {
  const match = read("apps/desktop/src-tauri/Cargo.toml").match(/^\[package\][^[]*?^version\s*=\s*"([^"]+)"/ms);
  if (!match) throw new Error("No [package] version in apps/desktop/src-tauri/Cargo.toml");
  return match[1];
}

function cargoLockVersion() {
  const match = read("Cargo.lock").match(/^name = "gnirehtet-desktop"\nversion = "([^"]+)"/m);
  if (!match) throw new Error("No gnirehtet-desktop entry in Cargo.lock");
  return match[1];
}

function changelogSection(version) {
  const lines = read("CHANGELOG.md").split("\n");
  const start = lines.findIndex((l) => l.startsWith(`## [${version}]`));
  if (start < 0) throw new Error(`CHANGELOG.md has no "## [${version}]" section`);
  const end = lines.findIndex((l, i) => i > start && l.startsWith("## ["));
  const body = lines
    .slice(start + 1, end < 0 ? undefined : end)
    .filter((l) => !/^\[[^\]]+\]: /.test(l)) // link reference definitions
    .join("\n")
    .trim();
  if (!body) throw new Error(`CHANGELOG.md section for ${version} is empty`);
  return body;
}

function main() {
  const argv = process.argv.slice(2);
  let notes = null;
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === "--notes") notes = argv[++i];
    else throw new Error(`Unknown argument: ${argv[i]}`);
  }

  const sources = {
    "tauri.conf.json": json("apps/desktop/src-tauri/tauri.conf.json").version,
    "Cargo.toml": cargoVersion(),
    "Cargo.lock": cargoLockVersion(),
    "package.json": json("apps/desktop/package.json").version,
    "package-lock.json": json("apps/desktop/package-lock.json").version,
  };
  const versions = new Set(Object.values(sources));
  if (versions.size !== 1) {
    const list = Object.entries(sources).map(([file, v]) => `  ${file}: ${v}`).join("\n");
    throw new Error(`Version mismatch:\n${list}`);
  }
  const [version] = versions;
  const homepage = json("apps/desktop/src-tauri/tauri.conf.json").bundle.homepage;

  if (notes) {
    const body = changelogSection(version);
    writeFileSync(notes, `${body}\n\nInstallation steps: [docs/INSTALL.md](${homepage}/blob/v${version}/docs/INSTALL.md)\n`);
  }
  console.log(version);
}

try {
  main();
} catch (error) {
  console.error(`error: ${error.message}`);
  process.exit(1);
}
