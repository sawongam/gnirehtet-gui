# Tauri ↔ SessionController glue sketch (copy into Desktop)

**Status:** WIP note for Desktop migrate — do **not** treat as checked-in app code.  
**Crate:** `gnirehtet-controller` on `dev`  
**Rule:** Desktop must **not** depend on `gnirehtet-relay` / relaylib. Hold one `Mutex<SessionController>` (or equivalent) and delete duplicate spawn logic.

## Cargo (`apps/desktop/src-tauri/Cargo.toml`)

```toml
[dependencies]
gnirehtet-controller = { path = "../../../crates/gnirehtet-controller" }
# optional if you call AdbClient directly:
# gnirehtet-adb = { path = "../../../crates/gnirehtet-adb" }
```

## State

```rust
use std::sync::Mutex;
use gnirehtet_controller::SessionController;

pub struct OrchestratorState {
    pub session: Mutex<SessionController>,
}

impl Default for OrchestratorState {
    fn default() -> Self {
        Self {
            session: Mutex::new(SessionController::from_env()),
        }
    }
}
```

`from_env()` honors `ADB`, `GNIREHTET_APK`, `GNIREHTET_BIN` (same spirit as Phase 0 stubs).

## Error → ERROR_UX

```rust
use gnirehtet_controller::ControllerError;

fn map_controller_err(e: ControllerError) -> (String /* code */, String /* message */) {
    let code = e
        .ux_code()
        .unwrap_or("INTERNAL")
        .to_string();
    (code, e.to_string())
}
```

| Controller / ADB | ERROR_UX hint |
|------------------|---------------|
| `ControllerError::PortInUse` | `PORT_IN_USE` (**before** ownership — `owns_relay()` stays false) |
| `RelaySpawn` / `RelayStartFailed` / `GnirehtetNotFound` | `RELAY_START_FAILED` |
| `Adb` → `adb_ux_code_hint()` | `ADB_MISSING` / `ADB_PATH_INVALID` |
| Empty `list_devices()` | OK `[]` — UI maps to `NO_DEVICES` |

Emit your existing `Error` / `RelayState` / `LogLine` events from these codes; don’t invent new codes.

## Command mapping (orchestrator stubs → controller)

| Tauri command | Call |
|---------------|------|
| `ensure_adb` | `session.ensure_adb()` → `AdbStatus { path, version }` |
| `list_devices` | `session.list_devices()` → `Vec<AdbDevice>` |
| `install` | `session.install(serial)` |
| `start` | `session.start(serial, &VpnOptions { dns_servers, routes, port })` |
| `stop` | `session.stop(serial)` |
| `reset_tunnel` / tunnel | `session.reset_tunnel(serial, port)` |
| `start_relay` | `session.start_relay(port)` — then read `owned_relay_pid()`, `session_port()`, `owns_relay()` |
| `stop_relay` | `session.stop_relay()` (or ignore `NoOwnedRelay`) |
| `run` | `session.run(serial, &RunOptions { dns_servers, routes, port })` — **child** relay + adb start |
| quit / window close / `teardown_owned_relay` | **`session.clear_owned_relay()`** (P0-Q1) |

### Quit (required for PHASE0 P0-Q1)

```rust
pub fn teardown_owned_relay(state: &OrchestratorState) {
    if let Ok(mut session) = state.session.lock() {
        session.clear_owned_relay(); // kills only session-owned child
    }
}
```

Also fine: drop the `SessionController` (implements `Drop` → clears owned relay). Prefer an explicit quit hook so timing matches your `RelayState` event.

### PORT_IN_USE

```rust
match session.start_relay(port) {
    Err(e @ ControllerError::PortInUse { .. }) => {
        assert!(!session.owns_relay());
        // emit Error { code: "PORT_IN_USE", ... } — do not set owned=true
    }
    Ok(relay) => { /* emit RelayState running + pid */ }
    Err(e) => { /* map via ux_code() */ }
}
```

## What to delete from Desktop stubs

Once wired:

1. Local `Command::new("gnirehtet").args(["relay", …])` spawn/kill paths  
2. Duplicate bind probes (controller already probes `127.0.0.1:port` before ownership)  
3. Duplicate `adb devices -l` parsers (use `list_devices`)  
4. Any path that links or embeds `relaylib`

Keep: UI event types (`DeviceChanged`, `RelayState`, `LogLine`, `Error`), APK resource resolution if still Desktop-owned, and log streaming from the child if you attach pipes in Desktop (controller currently inherits stdio — open a follow-up if you need captured stdout).

## Child stdio / LogLine (gap)

`SessionController` / `spawn_relay` today do **not** expose piped stdout/stderr. If Phase 0 log streaming relied on pipes, either:

- keep a thin Desktop wrapper that spawns with pipes **or**
- ask Rust for a follow-up: `start_relay_with_stdio` / log callback

Don’t block migrate on that — spawn/stop/ownership can land first; LogLine can stay stubbed or use `log` facade lines.

## Sanity checks after migrate

```text
cargo check -p gnirehtet-desktop
cargo test -p gnirehtet-controller -p gnirehtet-adb
```

Confirm `apps/desktop/src-tauri/Cargo.toml` has **no** `gnirehtet-relay` dependency.

## Public API cheat-sheet

```rust
use gnirehtet_controller::{
    ControllerConfig, ControllerError, RunOptions, SessionController,
    AdbDevice, AdbStatus, VpnOptions, DEFAULT_RELAY_PORT,
};

let mut s = SessionController::from_env();
s.ensure_adb()?;
let devices = s.list_devices()?;
s.start_relay(None)?;                 // session port or 31416
assert!(s.owns_relay());
s.stop_relay()?;
s.clear_owned_relay();                // idempotent; quit path
```

Ping Rust Architect if a method is missing or signatures fight serde/IPC.
