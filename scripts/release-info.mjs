#!/usr/bin/env node
// Prints the app version: [workspace.package] version in the root Cargo.toml.
// Fails if Cargo.lock is stale or a second copy of the version has crept into
// tauri.conf.json or package.json.
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
  const match = read("Cargo.toml").match(/^\[workspace\.package\][^[]*?^version\s*=\s*"([^"]+)"/ms);
  if (!match) throw new Error("No [workspace.package] version in Cargo.toml");
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

  const version = cargoVersion();
  const lock = cargoLockVersion();
  if (lock !== version) throw new Error(`Cargo.lock has ${lock}, Cargo.toml has ${version}. Run cargo check.`);
  const tauri = json("apps/desktop/src-tauri/tauri.conf.json");
  for (const [file, data] of [["tauri.conf.json", tauri], ["package.json", json("apps/desktop/package.json")]]) {
    if ("version" in data) throw new Error(`Remove "version" from ${file}; the root Cargo.toml holds it.`);
  }
  const homepage = tauri.bundle.homepage;

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
