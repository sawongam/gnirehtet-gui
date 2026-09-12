# Slice Report — Phase 2 start: Adb discovery + controller sidecar

**Date:** 2026-09-12  
**Agent:** Gnirehtet Rust Architect / Phase 2 executor  
**Branch:** `dev` @ `/workspace/gnirehtet-gui-push`  
**Upstream pin:** Genymobile/gnirehtet `@1eb2e58` / v2.5.1  
**Prior:** Phase 1 AdbClient extract (committed); this slice left **unstaged** for Lead Architect (`sawongam`)

## 1. What was changed

### A. `gnirehtet-adb` — Desktop-priority discovery/health

- `AdbClient::ensure_adb()` — `adb start-server` then `adb version` using configured `adb_path` (`ADB` env / `AdbConfig`)
- `AdbClient::list_devices()` — `adb devices -l` → `Vec<AdbDevice>`
- Types: `AdbStatus { path, version }`, `AdbDevice { serial, state, model?, product? }`
- `parse_adb_devices_l(&str)` — pure parser (unit-tested without a device)
- Error accessors + `CommandExecutionError::adb_ux_code_hint()` → `ADB_MISSING` / `ADB_PATH_INVALID` for Desktop ERROR_UX mapping

### B. `gnirehtet-controller` — Phase 2 start (workspace member)

- New crate depending on **`gnirehtet-adb` only** (no `gnirehtet-relay` / relaylib)
- `SessionController`: session-scoped relay child + ADB verb mirrors
- `start_relay(port?)` / `stop_relay()` / `clear_owned_relay()` (+ `Drop`)
- `probe_relay_port` / `spawn_relay` — bind probe on `127.0.0.1` before ownership
- Mirrors: `ensure_adb`, `list_devices`, `install`, `start`, `stop`, `reset_tunnel`, `run`
- `run(serial, opts)` = start relay child + `AdbClient::start` (install/tunnel/start) — **not** in-process relaylib
- Root `Cargo.toml`: enabled `crates/gnirehtet-controller`; preserved Desktop + adb + relay + cli members

## 2. Why

- Desktop Phase 0 stubs need first-class `list_devices` / `ensure_adb` on the adb crate ASAP (steering).
- Phase 2 goal: GUI-safe sidecar orchestration without linking mio/relay into the desktop process.
- PORT_IN_USE must surface **before** claiming ownership (PHASE0_ACCEPTANCE P0-P2); quit must clear owned port (P0-Q1).

## 3. Files / modules affected

| Path | Role |
|------|------|
| `crates/gnirehtet-adb/src/client.rs` | `ensure_adb`, `list_devices`, `AdbStatus`, `AdbDevice`, parser + tests |
| `crates/gnirehtet-adb/src/error.rs` | accessors, `adb_ux_code_hint`, UX hint tests |
| `crates/gnirehtet-adb/src/lib.rs` | re-exports |
| `crates/gnirehtet-controller/Cargo.toml` | new member; dep `gnirehtet-adb` |
| `crates/gnirehtet-controller/src/{lib,error,relay,session}.rs` | controller API |
| `crates/gnirehtet-controller/README.md` | crate summary |
| `Cargo.toml` | workspace members include controller |
| `docs/architecture/rust/SLICE_REPORT.md` | this report |

## 4. How it works

### ensure_adb / list_devices

1. `ensure_adb`: `exec_adb(["start-server"])` then capture `adb version` stdout (first line → `AdbStatus.version`).
2. `list_devices`: capture `adb devices -l`; `parse_adb_devices_l` skips header/blank lines; columns = serial + state; optional `model:` / `product:` from `-l` tokens.
3. Empty device list is **Ok([])** — UI maps to `NO_DEVICES`. Unauthorized/offline states are raw adb tokens for Desktop chips.
4. Spawn failure `NotFound` → hint `ADB_MISSING`; other process-IO → `ADB_PATH_INVALID`.

### Controller relay ownership

1. Port policy: explicit `port` > session port > **31416**. No mid-session listen-port hot-swap while owned.
2. Before spawn: `TcpListener::bind(("127.0.0.1", port))` — `AddrInUse` → `ControllerError::PortInUse` (`ux_code() == PORT_IN_USE`); **no** child, **no** ownership.
3. Spawn `gnirehtet relay -p <port>` (path: `GNIREHTET_BIN` or `"gnirehtet"`). Brief `try_wait`; immediate exit → `RELAY_START_FAILED`, no ownership.
4. `stop_relay` / `Drop` / `clear_owned_relay` kill **only** the session child (foreign process on :31416 never killed — P0-R4).

### PHASE0_ACCEPTANCE scoring notes (spawn/stop present)

| Gate | Status in this slice |
|------|----------------------|
| P0-P2 Bind failure before ownership → `PORT_IN_USE` | **Covered** by `probe_relay_port` + unit test; ownership stays false |
| P0-P1 Default port 31416 | **Covered** (`DEFAULT_RELAY_PORT`) |
| P0-R2 Stop owned sidecar | **Covered** (`stop_relay` / `clear_owned_relay`) |
| P0-R4 Do not kill foreign :31416 | **Covered** (only owned `Child`) |
| P0-Q1 Quit clears owned port | **API ready** (`Drop` + `clear_owned_relay`); Desktop must call on quit hook — not wired into Tauri yet |
| P0-R5 Relay death UI ≤3s | **Not in this crate** — Desktop poll/events still own UI SLA |

## 5. Tests performed

```text
cargo test -p gnirehtet-adb -p gnirehtet-controller
cargo test -p gnirehtet-cli -p gnirehtet-relay
```

- `gnirehtet-adb`: **14 passed** (buffer, monitor, device parse ×3, UX hints ×2)
- `gnirehtet-controller`: **4 passed** (default port, PortInUse no ownership, clear idempotent, probe)
- `gnirehtet-cli` / `gnirehtet-relay`: still green (10 / 30+1 ignored)
- Confirmed: controller & desktop do not depend on `gnirehtet-relay`

## 6. Known limitations

- Port probe is TOCTOU vs spawn (MVP acceptable); immediate-exit check mitigates some races.
- `start_relay` does not parse child stderr for “Address already in use” beyond exit — pre-bind is primary signal.
- No stdout/stderr log bridge / events yet (`LogLine` / `RelayState` remain Desktop Phase 0 concern until wired).
- `AdbMonitor` cancel **not** added (MVP / Lead greenlight — not required for quit).
- CLI `run` still in-process relaylib; controller `run` is the child-based path for GUI.
- Absolute `GNIREHTET_BIN` validated as file; bare `gnirehtet` on PATH validated only at spawn time.
- Desktop orchestrator still has its own stub `ensure_adb`/`list_devices` — follow-up to call crate.

## 7. Follow-ups

1. Desktop: replace Phase 0 inline adb parsing with `gnirehtet-adb` / `SessionController` (map `adb_ux_code_hint` / `ControllerError::ux_code` to ERROR_UX).
2. Wire quit hook → `clear_owned_relay` (PHASE0 P0-Q1 end-to-end).
3. Optional: pipe relay child stdout/stderr into controller callbacks for `LogLine`.
4. Phase 2.3 later: stoppable `AdbMonitor` if autorun/GUI monitor needs cancel.
5. Resolve APK to absolute path before child-related install flows; populate `resources/`.
6. Lead Architect commit — **do not** commit from this agent (left unstaged).

## Success checklist

- [x] `ensure_adb` / `list_devices` on `gnirehtet-adb` with real adb shapes
- [x] ERROR_UX-mappable ADB failure hints
- [x] `gnirehtet-controller` workspace member, no relaylib dep
- [x] Session-owned relay start/stop + port policy + PORT_IN_USE before ownership
- [x] `run` prefers child relay + AdbClient
- [x] `cargo test -p gnirehtet-adb -p gnirehtet-controller` green
- [x] Desktop workspace members preserved; controller enabled
- [x] Unstaged for `sawongam`
