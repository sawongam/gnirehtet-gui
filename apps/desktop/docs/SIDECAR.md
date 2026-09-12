# gnirehtet sidecar (externalBin)

MVP relay + CLI verbs use the **stock upstream `gnirehtet` binary** as a managed child process.
Do **not** link `relaylib` / `gnirehtet-relay` into the Tauri process.

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

Obtain from the upstream release zip (same pin as APK: Genymobile/gnirehtet **v2.5.1** / `1eb2e58`), rename with the triple suffix, `chmod +x` on Unix.

```bash
rustc -vV | sed -n 's/^host: //p'
```

## Enable bundling

`tauri.conf.json` ships with `bundle.externalBin: []` so CI/`cargo check` works without the binary.
When the sidecar is present, set:

```json
"externalBin": ["binaries/gnirehtet"]
```

## Runtime resolution (Phase 0)

1. `GNIREHTET_BIN` env override  
2. Packaged resource / sidecar path  
3. `src-tauri/binaries/gnirehtet-<triple>`  
4. `PATH`

Missing binary → `RELAY_START_FAILED` (ERROR_UX). Occupied port → `PORT_IN_USE` **before** ownership. Quit tears down **owned** child only (never foreign; never adb server).

## Plugin

`tauri-plugin-shell` is enabled. Process supervision (pipes → `LogLine`, session-scoped kill) lives in the Rust host-orchestrator.
