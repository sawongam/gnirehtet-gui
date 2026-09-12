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
| `start_relay` | `session.start_relay(port)` — inherit stdio; then `owned_relay_pid()` / `session_port()` / `owns_relay()` |
| `start_relay` + LogLine | `session.start_relay_with_stdio(port)` → `RelayStdio` (take under lock, pump outside) |
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

Keep: UI event types (`DeviceChanged`, `RelayState`, `LogLine`, `Error`), APK resource resolution if still Desktop-owned. For child log streaming use `start_relay_with_stdio` (below); plain `start_relay` inherits stdio.

## Child stdio / LogLine (`start_relay_with_stdio`)

Two start paths (same port policy + `PORT_IN_USE` probe **before** ownership):

| API | Stdio | Use when |
|-----|-------|----------|
| `start_relay(port?)` | **inherit** | spawn/stop-first; no child LogLine |
| `start_relay_with_stdio(port?) -> RelayStdio` | **piped** | Desktop streams child stdout/stderr as `LogLine` |

```rust
use std::io::{BufRead, BufReader};
use std::thread;
use gnirehtet_controller::{ControllerError, RelayStdio, SessionController};

/// Under the session mutex: claim ownership + take pipes, then **drop the guard**
/// before reading. Desktop owns the single LogLine pump — controller does not
/// spawn reader threads that would double-emit lines.
fn start_relay_and_pump_logs(state: &OrchestratorState, port: Option<u16>) -> Result<(), ControllerError> {
    let stdio: RelayStdio = {
        let mut session = state.session.lock().unwrap();
        session.start_relay_with_stdio(port)?
        // MutexGuard dropped here — do not read pipes while holding the lock.
    };

    let RelayStdio { stdout, stderr } = stdio;
    thread::spawn(move || {
        for line in BufReader::new(stdout).lines().flatten() {
            // app.emit("LogLine", …)
            let _ = line;
        }
    });
    thread::spawn(move || {
        for line in BufReader::new(stderr).lines().flatten() {
            // app.emit("LogLine", …)
            let _ = line;
        }
    });
    Ok(())
}
```

**Rules for Desktop:**

1. Take `RelayStdio` under the lock briefly; pump **outside** `Mutex<SessionController>`.
2. Single LogLine pump — do not also attach a second reader to the same child.
3. `stop_relay` / `clear_owned_relay` / `Drop` still kill the session-owned child (pipes EOF).
4. Plain `start_relay` remains for paths that do not need captured logs.

## Sanity checks after migrate

```text
cargo check -p gnirehtet-desktop
cargo test -p gnirehtet-controller -p gnirehtet-adb
```

Confirm `apps/desktop/src-tauri/Cargo.toml` has **no** `gnirehtet-relay` dependency.

## Public API cheat-sheet

```rust
use gnirehtet_controller::{
    ControllerConfig, ControllerError, RelayStdio, RunOptions, SessionController,
    AdbDevice, AdbStatus, VpnOptions, DEFAULT_RELAY_PORT,
};

let mut s = SessionController::from_env();
s.ensure_adb()?;
let devices = s.list_devices()?;
s.start_relay(None)?;                 // inherit stdio; session port or 31416
assert!(s.owns_relay());
s.stop_relay()?;

// LogLine path (piped); ownership still on session:
let stdio: RelayStdio = s.start_relay_with_stdio(None)?;
// move stdio off the mutex before reading lines…
s.clear_owned_relay();                // idempotent; quit path
```

Ping Rust Architect if a method is missing or signatures fight serde/IPC.
