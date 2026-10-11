# gnirehtet sidecar (externalBin)

MVP relay + CLI verbs use the **stock upstream `gnirehtet` binary** as a managed child process.
Do **not** link `relaylib` / `gnirehtet-relay` into the Tauri process.

Controller resolution stays compatible with Phase 0:

1. `GNIREHTET_BIN` env override  
2. Sidecar / drop-in search (`binaries/gnirehtet-<triple>`, packaged sidecar path)  
3. `PATH` name `gnirehtet` (Windows: `gnirehtet.exe`)

## Upstream packaging pins

| Platform | Release zip pin | Binary **inside** the zip |
|----------|-----------------|---------------------------|
| Linux x86_64 | Genymobile **v2.5.1** (`gnirehtet-rust-linux64-v2.5.1.zip`) | `gnirehtet` (not the folder name) |
| Windows x86_64 | Genymobile **v2.5.1** (`gnirehtet-rust-win64-v2.5.1.zip`) | `gnirehtet.exe` |
| macOS | **deferred** this slice (not first-class in RC packaging) | — |

**Do not silently mix pins.** Linux/Windows ship/APK line is **v2.5.1** / `1eb2e58`. APK stays v2.5.1 for Linux/Windows hosts.

Extract the executable named `gnirehtet` / `gnirehtet.exe` from the zip — **not** the zip folder name (e.g. not `gnirehtet-rust-linux64`).

## Drop-in path (dev / CI)

Tauri 2 `bundle.externalBin` expects target-triple–suffixed binaries under `src-tauri/binaries/`:

```text
apps/desktop/src-tauri/binaries/gnirehtet-<target-triple>[.exe]
```

| Platform | File |
|----------|------|
| Linux x86_64 | `binaries/gnirehtet-x86_64-unknown-linux-gnu` |
| Windows x86_64 | `binaries/gnirehtet-x86_64-pc-windows-msvc.exe` |

```bash
# Linux v2.5.1 example
curl -fsSL -o /tmp/gnirehtet-rust-linux64-v2.5.1.zip \
  https://github.com/Genymobile/gnirehtet/releases/download/v2.5.1/gnirehtet-rust-linux64-v2.5.1.zip
unzip -p /tmp/gnirehtet-rust-linux64-v2.5.1.zip '*/gnirehtet' \
  > apps/desktop/src-tauri/binaries/gnirehtet-$(rustc -vV | sed -n 's/^host: //p')
chmod +x apps/desktop/src-tauri/binaries/gnirehtet-*
```

`binaries/` is gitignored except README / `.gitkeep` — CI clones must drop the file in or clear `externalBin` (below).

## Release packaging: `externalBin` **enabled**

**RC packaging** (see `docs/architecture/desktop/RC_PACKAGING.md`) ships with:

```json
"externalBin": ["binaries/gnirehtet"]
```

so `npm run tauri build` embeds the triple-suffixed sidecar into deb/rpm (Linux) or nsis/msi (Windows). Requires `binaries/gnirehtet-<triple>` present — otherwise `tauri build` fails.

### No-sidecar workflow (`cargo check` / CI without binary)

Prefer release config with sidecar present. If you temporarily need a green check **without** the binary:

1. Set `"externalBin": []`.
2. Run `cargo check -p gnirehtet-desktop` / CI unit checks.
3. Restore `"externalBin": ["binaries/gnirehtet"]` before RC installer builds.

Dev/runtime still finds the drop-in via `GNIREHTET_BIN` / search / `PATH` even when `externalBin` is empty.

## Runtime resolution (Phase 0)

1. `GNIREHTET_BIN` env override  
2. Packaged resource / sidecar path (when externalBin enabled)  
3. `src-tauri/binaries/gnirehtet-<triple>`  
4. `PATH` → `gnirehtet`

Missing binary → `RELAY_START_FAILED` (ERROR_UX). Occupied port → `PORT_IN_USE` **before** ownership. Quit tears down **owned** child only (never foreign; never adb server).

## Plugin

`tauri-plugin-shell` is enabled. Process supervision (pipes → `LogLine`, session-scoped kill) lives in the Rust host-orchestrator.
