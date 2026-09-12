# Phase 1 Slice Report — ADB extract + workspace vendor

**Date:** 2026-09-12  
**Agent:** Gnirehtet Rust Architect / Phase 1 executor  
**Upstream pin:** Genymobile/gnirehtet `@1eb2e58` / v2.5.1  
**Evidence:** `/workspace/gnirehtet-rust-src/relay-rust/`

## 1. What was changed

- Created a Cargo workspace under `/workspace/gnirehtet-gui-push` coexisting with Desktop’s `apps/desktop/src-tauri`.
- Added `crates/gnirehtet-adb`: reusable `AdbClient` / `AdbConfig` / `VpnOptions`, local `ByteBuffer`, `AdbMonitor`, and execution errors — **no** dependency on `relaylib`.
- Vendored relay packet path as `crates/gnirehtet-relay` (`relaylib`) with minimal Cargo wiring (added missing `chrono` dep used by upstream `relay.rs`).
- Added thin `crates/gnirehtet-cli` binary `gnirehtet` that dispatches CLI verbs to `AdbClient` and calls `relaylib::relay(port)` for `relay`/`run`/`autorun` parity.
- Unified `AdbMonitor` daemon restart to use configurable `adb_path` (`ADB` env / `AdbConfig`), breaking the hardcoded `"adb"` FACT.
- Placeholders: `crates/gnirehtet-controller/README.md`, `resources/README.md` (layout alignment).

## 2. Why

- Phase 1 goal: GUI (and later controller) can call `AdbClient` without parsing argv and without linking mio/relay.
- Break false `adb_monitor` → `relaylib::byte_buffer` coupling (REFACTOR_PLAN §1.1).
- Preserve CLI verb semantics and Apache-2.0 Genymobile headers on moved sources.
- Desktop MVP remains sidecar/`externalBin` — desktop crate does **not** depend on `gnirehtet-relay`.

## 3. Files / modules affected

| Path | Role |
|------|------|
| `/workspace/gnirehtet-gui-push/Cargo.toml` | Workspace members: desktop + adb + relay + cli |
| `/workspace/gnirehtet-gui-push/crates/gnirehtet-adb/` | Phase 1 focus library |
| `…/gnirehtet-adb/src/lib.rs` | Public re-exports |
| `…/gnirehtet-adb/src/client.rs` | `AdbClient`, `AdbConfig`, `VpnOptions` |
| `…/gnirehtet-adb/src/buffer.rs` | Local `ByteBuffer` copy (from upstream `relay/byte_buffer.rs`) |
| `…/gnirehtet-adb/src/monitor.rs` | `AdbMonitor` + `adb_path` / `adbd_addr` |
| `…/gnirehtet-adb/src/error.rs` | Former `execution_error.rs` |
| `/workspace/gnirehtet-gui-push/crates/gnirehtet-relay/` | Vendored `src/relay/**` + `lib.rs` as `relaylib` |
| `/workspace/gnirehtet-gui-push/crates/gnirehtet-cli/` | Thin CLI (`main.rs`, `cli_args.rs`, `logger.rs`) |
| `/workspace/gnirehtet-gui-push/docs/architecture/rust/SLICE_REPORT.md` | This report |

## 4. How it works

- `AdbClient::from_env()` reads `ADB` / `GNIREHTET_APK` like upstream `get_adb_path` / `get_apk_path`.
- Methods wrap upstream argv verbatim: `install`, `uninstall`, `reinstall`, `tunnel`, `start`, `stop`, `restart`, `must_install_client`, `exec_adb`, `async_start`, `monitor`.
- `AdbMonitor` frames `host:track-devices` with the **local** `ByteBuffer`; `start_adb_daemon` uses `&self.adb_path`.
- CLI `relay` / `run` / `autorun` still call **in-process** `relaylib::relay(port)` for Phase 1 parity. GUI must spawn stock/`externalBin` gnirehtet later (not link this path).

## 5. Tests performed

```text
cargo test -p gnirehtet-adb -p gnirehtet-relay -p gnirehtet-cli
```

- `gnirehtet-adb`: **9 passed** (buffer + monitor packet tests)
- `gnirehtet-relay`: **30 passed**, 1 ignored (upstream parity)
- `gnirehtet-cli`: **10 passed** (cli_args)
- Smoke: `./target/debug/gnirehtet` prints full verb usage
- Confirmed: no `relaylib` import in `gnirehtet-adb` sources (comment-only mention)
- Confirmed: `apps/desktop/src-tauri` does not depend on `gnirehtet-relay`

## 6. Known limitations

- CLI `run` still uses Ctrl+C → `exit(0)` (upstream FACT); child-based run is Phase 2.
- `AdbMonitor::monitor` remains an infinite blocking loop (no cancel flag yet — Phase 2.3).
- `autorun` / multi-device monitor kept for CLI parity only; **not** MVP GUI feature work.
- mio stays at **0.6** (no bump).
- Workspace root warns that `[profile.release]` in `apps/desktop/src-tauri/Cargo.toml` is ignored (Desktop-owned; move profiles to workspace root later).
- No device/APK integration smoke on this box (no handset / APK in `resources/` yet).
- `gnirehtet-cli` is an extra workspace member beyond the locked Desktop layout sketch; needed for verb parity. Desktop must not depend on it in-process for relay.

## 7. Follow-ups

1. Phase 2: `gnirehtet-controller` with `RelayProcess` / `RunSession` (spawn relay binary; GUI-safe).
2. Stoppable `AdbMonitor` (`AtomicBool` / channel).
3. Resolve APK to absolute path before any child spawn; populate `resources/`.
4. Move release profile from desktop package into workspace root (Desktop).
5. Optional: drop in-process `relaylib` from CLI once child spawn path is proven.
6. Lead Architect (`sawongam`) commit/integration — files left unstaged by this slice.

## Success criteria checklist

- [x] `cargo test` / `cargo build` green for new crates
- [x] Adb code does not import `relaylib::byte_buffer`
- [x] Public `AdbClient` covers install/start/stop/tunnel (+ helpers)
- [x] `SLICE_REPORT.md` written

## Integration note (Desktop concurrent)

Workspace root `Cargo.toml` is shared with Desktop (`apps/desktop/src-tauri`).
If Desktop regenerates the root with Phase 1 members commented out, re-enable:

```toml
members = [
  "apps/desktop/src-tauri",
  "crates/gnirehtet-adb",
  "crates/gnirehtet-relay",
  "crates/gnirehtet-cli",
]
```

Keep Desktop `[profile.release]` at workspace root. Do not make `gnirehtet-desktop` depend on `gnirehtet-relay`.
