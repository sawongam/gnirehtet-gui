#!/usr/bin/env node
// Prints the app version: [workspace.package] version in the root Cargo.toml.
// Fails if Cargo.lock is stale or a second copy of the version has crept into
// tauri.conf.json or package.json.
//
// With --notes <file>, also writes the release notes to <file>: a Downloads
// table followed by the CHANGELOG.md section for that version, failing if the
// section is missing or empty.
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

// The Downloads table at the top of the release notes. File names must match
// the ones the Release workflow gives the installers.
function downloads(version, homepage) {
  const file = (suffix) => `gnirehtet-gui-${version}-${suffix}`;
  const macOpen =
    "Open it and drag the app to *Applications*. The app is not notarized: the first time, right-click it and choose **Open**.";
  const rows = [
    ["Windows 10, 11", "windows-x64-setup.exe",
      "Run the installer. SmartScreen warns about an unknown publisher: *More info* → *Run anyway*."],
    ["macOS, Apple Silicon (M1 and later)", "macos-apple-silicon.dmg", macOpen],
    ["macOS, Intel", "macos-intel.dmg", macOpen],
    ["Debian, Ubuntu, Mint", "linux-x64.deb", `\`sudo apt install ./${file("linux-x64.deb")}\``],
    ["Fedora, openSUSE", "linux-x64.rpm", `\`sudo dnf install ./${file("linux-x64.rpm")}\` (\`zypper install\` on openSUSE)`],
    ["Other Linux", "linux-x64.AppImage", "Make it executable (`chmod +x`) and run it."],
  ];
  const guide = `${homepage}/blob/v${version}/docs/INSTALL.md`;
  return [
    "## Downloads",
    "",
    "| System | File | How to install |",
    "| --- | --- | --- |",
    ...rows.map(([system, suffix, how]) => `| ${system} | \`${file(suffix)}\` | ${how} |`),
    "",
    `The app also needs **adb** (Android platform-tools) on the computer and USB debugging on the phone. The [installation guide](${guide}) covers both.`,
    "",
    "Verify a download against `SHA256SUMS.txt`.",
  ].join("\n");
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
    writeFileSync(notes, `${downloads(version, homepage)}\n\n## What's changed\n\n${body}\n`);
  }
  console.log(version);
}

try {
  main();
} catch (error) {
  console.error(`error: ${error.message}`);
  process.exit(1);
}
