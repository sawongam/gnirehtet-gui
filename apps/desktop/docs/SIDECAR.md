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
| macOS | Upstream still **v2.2.1** only | `gnirehtet` |

**Do not silently mix pins.** Linux/Windows ship/APK line is **v2.5.1** / `1eb2e58`. macOS prebuilt rust zip remains **v2.2.1** — treat as a packaging exception (document in release notes; prefer build-from-source for macOS parity later). APK stays v2.5.1 for all hosts.

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
| macOS arm64 | `binaries/gnirehtet-aarch64-apple-darwin` |

```bash
# Linux v2.5.1 example
curl -fsSL -o /tmp/gnirehtet-rust-linux64-v2.5.1.zip \
  https://github.com/Genymobile/gnirehtet/releases/download/v2.5.1/gnirehtet-rust-linux64-v2.5.1.zip
unzip -p /tmp/gnirehtet-rust-linux64-v2.5.1.zip '*/gnirehtet' \
  > apps/desktop/src-tauri/binaries/gnirehtet-$(rustc -vV | sed -n 's/^host: //p')
chmod +x apps/desktop/src-tauri/binaries/gnirehtet-*
```

```bash
rustc -vV | sed -n 's/^host: //p'
```

`binaries/` is gitignored except README / `.gitkeep` — CI and `cargo check` must not require the file.

## Enable bundling (reversible; default off)

`tauri.conf.json` ships with:

```json
"externalBin": []
```

so `cargo check -p gnirehtet-desktop` and CI stay green **without** a committed sidecar.

When `binaries/gnirehtet-<triple>` is present and you want real installers to embed it, set:

```json
"externalBin": ["binaries/gnirehtet"]
```

(Tauri appends `-<triple>` / `.exe` at bundle time.)

To revert: restore `"externalBin": []`. Dev/runtime still finds the drop-in via `GNIREHTET_BIN` / search / `PATH` even when externalBin is empty.

**Do not auto-enable** from missing-binary-safe defaults — enabling with no triple-suffixed file breaks `tauri build`.

## Runtime resolution (Phase 0)

Same order as controller `default_gnirehtet_path` / sidecar search:

1. `GNIREHTET_BIN` env override  
2. Packaged resource / sidecar path (when externalBin enabled)  
3. `src-tauri/binaries/gnirehtet-<triple>`  
4. `PATH` → `gnirehtet`

Missing binary → `RELAY_START_FAILED` (ERROR_UX). Occupied port → `PORT_IN_USE` **before** ownership. Quit tears down **owned** child only (never foreign; never adb server).

## Plugin

`tauri-plugin-shell` is enabled. Process supervision (pipes → `LogLine`, session-scoped kill) lives in the Rust host-orchestrator.
