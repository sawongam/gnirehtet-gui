#!/usr/bin/env node
// Prepares the files bundled with the desktop app:
//   resources/gnirehtet.apk                                  (Android client)
//   apps/desktop/src-tauri/binaries/gnirehtet-relay-<triple>[.exe] (relay)
//
// Both come from the pinned upstream gnirehtet release and are verified
// against known SHA-256 checksums. Where upstream publishes no prebuilt relay
// (macOS, ARM, ...), the relay is built from the vendored sources in
// crates/gnirehtet-relay (same version) with `cargo build -p gnirehtet-cli`.
//
// Usage: node scripts/fetch-upstream.mjs [--target <rust-triple>] [--from-source] [--force]

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { chmodSync, existsSync, mkdirSync, readFileSync, writeFileSync, copyFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { inflateRawSync } from "node:zlib";

const VERSION = "v2.5.1";
const BASE_URL = `https://github.com/Genymobile/gnirehtet/releases/download/${VERSION}`;

const RELEASES = {
  linux64: {
    file: `gnirehtet-rust-linux64-${VERSION}.zip`,
    sha256: "dee55499ca4fef00ce2559c767d2d8130163736d43fdbce753e923e75309c275",
    relay: "gnirehtet-rust-linux64/gnirehtet",
    apk: "gnirehtet-rust-linux64/gnirehtet.apk",
  },
  win64: {
    file: `gnirehtet-rust-win64-${VERSION}.zip`,
    sha256: "7f5b1063e7895182aa60def1437e50363c3758144088dcd079037bb7c3c46a1c",
    relay: "gnirehtet-rust-win64/gnirehtet.exe",
    apk: "gnirehtet-rust-win64/gnirehtet.apk",
  },
};

const APK_SHA256 = "c1ac2b869a48e3c836046aac5a168f3ade510288f3304a87ddb671315c564b9a";

/** Rust targets with an upstream prebuilt relay. */
const PREBUILT = {
  "x86_64-unknown-linux-gnu": "linux64",
  "x86_64-pc-windows-msvc": "win64",
};

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const CACHE = join(ROOT, "target", "upstream-cache");

function parseArgs(argv) {
  const args = { target: null, fromSource: false, force: false };
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === "--target") args.target = argv[++i];
    else if (arg === "--from-source") args.fromSource = true;
    else if (arg === "--force") args.force = true;
    else if (arg === "--help" || arg === "-h") {
      console.log("Usage: node scripts/fetch-upstream.mjs [--target <rust-triple>] [--from-source] [--force]");
      process.exit(0);
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }
  return args;
}

function hostTriple() {
  const output = execFileSync("rustc", ["-vV"], { encoding: "utf8" });
  const line = output.split("\n").find((l) => l.startsWith("host: "));
  if (!line) throw new Error("Cannot determine the Rust host target (rustc -vV)");
  return line.slice("host: ".length).trim();
}

const sha256 = (data) => createHash("sha256").update(data).digest("hex");

async function download(release) {
  const path = join(CACHE, release.file);
  if (existsSync(path) && sha256(readFileSync(path)) === release.sha256) return readFileSync(path);

  const url = `${BASE_URL}/${release.file}`;
  console.log(`Downloading ${url}`);
  const response = await fetch(url);
  if (!response.ok) throw new Error(`Download failed: HTTP ${response.status} for ${url}`);
  const data = Buffer.from(await response.arrayBuffer());
  const actual = sha256(data);
  if (actual !== release.sha256) {
    throw new Error(`Checksum mismatch for ${release.file}: expected ${release.sha256}, got ${actual}`);
  }
  mkdirSync(CACHE, { recursive: true });
  writeFileSync(path, data);
  return data;
}

/** Extract one file from a zip archive (stored or deflated entries). */
function unzipEntry(zip, name) {
  const eocd = zip.lastIndexOf(Buffer.from([0x50, 0x4b, 0x05, 0x06]));
  if (eocd < 0) throw new Error("Invalid zip archive");
  const entries = zip.readUInt16LE(eocd + 10);
  let offset = zip.readUInt32LE(eocd + 16);
  for (let i = 0; i < entries; i++) {
    const method = zip.readUInt16LE(offset + 10);
    const compressedSize = zip.readUInt32LE(offset + 20);
    const nameLength = zip.readUInt16LE(offset + 28);
    const extraLength = zip.readUInt16LE(offset + 30);
    const commentLength = zip.readUInt16LE(offset + 32);
    const localOffset = zip.readUInt32LE(offset + 42);
    const entryName = zip.toString("utf8", offset + 46, offset + 46 + nameLength);
    if (entryName === name) {
      const localNameLength = zip.readUInt16LE(localOffset + 26);
      const localExtraLength = zip.readUInt16LE(localOffset + 28);
      const start = localOffset + 30 + localNameLength + localExtraLength;
      const data = zip.subarray(start, start + compressedSize);
      if (method === 0) return Buffer.from(data);
      if (method === 8) return inflateRawSync(data);
      throw new Error(`Unsupported zip compression method ${method} for ${name}`);
    }
    offset += 46 + nameLength + extraLength + commentLength;
  }
  throw new Error(`${name} not found in archive`);
}

function write(path, data, executable) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, data);
  if (executable) chmodSync(path, 0o755);
  console.log(`Wrote ${path.slice(ROOT.length + 1)}`);
}

function buildRelayFromSource(target, host) {
  console.log(`Building the relay from source for ${target}`);
  const args = ["build", "--release", "--locked", "-p", "gnirehtet-cli"];
  if (target !== host) args.push("--target", target);
  execFileSync("cargo", args, { cwd: ROOT, stdio: "inherit" });
  const exe = target.includes("windows") ? "gnirehtet.exe" : "gnirehtet";
  return target === host ? join(ROOT, "target", "release", exe) : join(ROOT, "target", target, "release", exe);
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const host = hostTriple();
  const target = args.target ?? host;
  const suffix = target.includes("windows") ? ".exe" : "";

  const apkPath = join(ROOT, "resources", "gnirehtet.apk");
  const relayPath = join(ROOT, "apps", "desktop", "src-tauri", "binaries", `gnirehtet-relay-${target}${suffix}`);

  const prebuilt = args.fromSource ? null : PREBUILT[target];
  // The APK is identical in every release archive: reuse whichever we fetch.
  const zip = await download(RELEASES[prebuilt ?? "linux64"]);

  if (args.force || !existsSync(apkPath) || sha256(readFileSync(apkPath)) !== APK_SHA256) {
    const apk = unzipEntry(zip, RELEASES[prebuilt ?? "linux64"].apk);
    if (sha256(apk) !== APK_SHA256) throw new Error("Checksum mismatch for gnirehtet.apk");
    write(apkPath, apk, false);
  } else {
    console.log("resources/gnirehtet.apk is up to date");
  }

  if (prebuilt) {
    write(relayPath, unzipEntry(zip, RELEASES[prebuilt].relay), true);
  } else {
    const built = buildRelayFromSource(target, host);
    mkdirSync(dirname(relayPath), { recursive: true });
    copyFileSync(built, relayPath);
    chmodSync(relayPath, 0o755);
    console.log(`Wrote ${relayPath.slice(ROOT.length + 1)}`);
  }
  console.log(`Ready: gnirehtet ${VERSION} relay and client for ${target}.`);
}

main().catch((error) => {
  console.error(`error: ${error.message}`);
  process.exit(1);
});
