# gnirehtet sidecar (externalBin)

MVP relay + CLI verbs use the **stock upstream `gnirehtet` binary** as a managed child process.
Do **not** link `relaylib` / `gnirehtet-relay` into the Tauri process.

## Drop-in path (dev / CI)

Tauri 2 `bundle.externalBin` expects target-triple–suffixed binaries under `src-tauri/binaries/`:

```text
apps/desktop/src-tauri/binaries/gnirehtet-<target-triple>[.exe]
```

Examples:

| Platform | File |
|----------|------|
| Linux x86_64 | `binaries/gnirehtet-x86_64-unknown-linux-gnu` |
| Windows x86_64 | `binaries/gnirehtet-x86_64-pc-windows-msvc.exe` |
| macOS arm64 | `binaries/gnirehtet-aarch64-apple-darwin` |

Obtain the binary from the same upstream release zip as the APK (e.g. `gnirehtet` inside `gnirehtet-rust-linux64-v2.5.1.zip`), rename with the triple suffix, and `chmod +x` on Unix.

Find your triple:

```bash
rustc -vV | sed -n 's/^host: //p'
```

## Runtime resolution (Phase 0 stub)

The host-orchestrator looks for `gnirehtet` in this order:

1. Bundled sidecar path (when packaged / when `binaries/` copy exists next to the app)
2. `GNIREHTET_BIN` environment override (dev convenience)
3. `PATH` (`which gnirehtet`)

If none are found, `start_relay` returns a clear `gnirehtet_not_found` error. CI may omit the binary; unit/smoke tests should assert the error path.

## Plugin

`tauri-plugin-shell` is enabled for sidecar/shell permissions. Process supervision (spawn, pipe stdout/stderr → `LogLine`, session-scoped kill) is implemented in the Rust host-orchestrator so we only kill relays **this process started**.
