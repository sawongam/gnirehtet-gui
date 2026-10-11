# Gnirehtet Rust — Error Handling Strategy

**Upstream pin:** Genymobile/gnirehtet `@1eb2e58` / v2.5.1  
**Evidence:** `relay-rust/src/`.  
**MVP:** relay is a **sidecar process**; orchestrator errors ≠ packet-path panics in the GUI process.

Facts and proposals are separated. Wire-protocol behavior must not change when restructuring error types.

---

## FACTS — Current error types

### 1. Binary / ADB layer — `execution_error.rs`

```text
CommandExecutionError
├── ProcessIo(ProcessIoError)       # Command::status/output failed to spawn/IO
├── ProcessStatus(ProcessStatusError)  # exited non-zero or signal
└── Io(io::Error)                   # From<io::Error> — used when relay returns IO
```

Supporting types:

- `Cmd { command, args }` — Display as `adb ["-s", …]`
- `Termination::Value(i32)` / `Signal(i32)` (unix)
- Manual `Display` + `Error::source` (pre-thiserror style, edition 2018)

**Used by:** all `cmd_*` in `main.rs` returning `Result<(), CommandExecutionError>`; `Command::execute`.

**CLI parse errors:** `cli_args::CommandLineArguments::parse` → `Result<Self, String>` (comment: “never need to inspect them”).

**Exit codes (FACT `main.rs`):**

| Situation | Exit |
|-----------|------|
| Unknown command | 1 |
| Bad arguments | 2 |
| Execution error | 3 |
| Ctrl+C in `run` | 0 via `exit(0)` |

### 2. Relay library — mostly `std::io::Result`

| API | Error style |
|-----|-------------|
| `lib.rs` `relay(port)` | `io::Result<()>` |
| `Relay::run` / `poll_loop` | `io::Result<()>` — success path does not return |
| `Selector::register` / `poll` | `io::Result` |
| `TunnelServer::create` / accept | `io::Result`; accept errors logged, not always fatal |
| `Router::create_connection` | `io::ErrorKind::Other` for unsupported protocol |
| Buffers / packetizer | `io::Result` or custom `ErrorKind::Other` (“buffer full”) |
| `ClientChannel::send_to_client` | `WouldBlock` when client buffer full (**backpressure**, not fatal) |

**No** domain error enum inside `relay/`. Failures are IO-ish or logged-and-continued.

### 3. Panics as control flow (FACT — important for sidecar choice)

| Location | Pattern |
|----------|---------|
| `client.rs` `on_ready` | `Err(_) => panic!("Unexpected unhandled error")` |
| `tcp_connection.rs` / `udp_connection.rs` | same |
| `client.rs` `process_pending` | panic on non-WouldBlock send failure |
| Header helpers | `panic!("Not a TCP packet")` etc. on programmer violations |
| `cli_args` port parse | `parse().unwrap()` on `-p` value |
| `get_adb_path` | `into_string().expect("invalid ADB value")` |

**Implication:** unexpected IO in a connection handler aborts the **entire relay process**. Acceptable when relay is a child; unacceptable if linked into a GUI.

### 4. Logging vs errors

- Many paths **log + close connection** instead of returning errors upward (`TunnelServer::on_ready`, client read/write failures).
- `AdbMonitor` logs and calls `repair_adb_daemon()` rather than returning to the caller (`monitor` loop swallows).
- `async_start` logs start failures on a background thread — caller of `run` still blocks in relay unaware.

### 5. Bridging relay IO into CLI

`cmd_relay`:

```rust
relaylib::relay(port)?;  // io::Error
Ok(())
```

`From<io::Error> for CommandExecutionError` maps this to `CommandExecutionError::Io`.

---

## PROPOSALS — Layered strategy

Align with crate boundaries (`CRATE_STRUCTURE.md`) and sidecar MVP.

```
┌─────────────────────────────────────────┐
│ Frontends (cli / desktop)               │
│  anyhow (or plain Display) at edges     │
│  map to exit codes / UI toasts          │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│ gnirehtet-controller                    │
│  SessionError (thiserror)               │
│  wraps AdbError + RelayProcessError     │
└─────────────┬─────────────┬─────────────┘
              │             │
┌─────────────▼──────┐  ┌───▼──────────────────────────┐
│ gnirehtet-adb      │  │ Relay child (process)        │
│ AdbError (thiserror│  │ exit status / spawn IO       │
│ ≈ today’s types)   │  │ (optional: parse stderr)     │
└────────────────────┘  └──────────────────────────────┘

gnirehtet-relay (in-child):
  keep io::Result at public run();
  Phase 3: reduce panics → close+log
```

### Library crates: `thiserror`

**Why:** structured matching for GUI (show “adb not found” vs “device unauthorized”); stable `source()` chains. Matches existing hand-written enums.

**Proposed `AdbError`** (evolve `CommandExecutionError`):

```rust
#[derive(Debug, thiserror::Error)]
pub enum AdbError {
    #[error(transparent)]
    ProcessIo(#[from] ProcessIoError),
    #[error(transparent)]
    ProcessStatus(#[from] ProcessStatusError),
    #[error("adb monitor: {0}")]
    Monitor(#[source] std::io::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

Keep `Cmd`, `ProcessIoError`, `ProcessStatusError` — they are good facts.

**Proposed `RelayProcessError`** (controller):

```rust
pub enum RelayProcessError {
    #[error("failed to spawn relay at {path}: {source}")]
    Spawn { path: PathBuf, source: std::io::Error },
    #[error("relay exited too early with {status}")]
    EarlyExit { status: ExitStatus },
    #[error("relay wait: {0}")]
    Wait(std::io::Error),
}
```

**Proposed `SessionError`:**

```rust
pub enum SessionError {
    Adb(#[from] AdbError),
    Relay(#[from] RelayProcessError),
    #[error("session already stopped")]
    AlreadyStopped,
}
```

### Binary / GUI edges: `anyhow` optional

- **CLI:** `main` can use `anyhow::Error` or map `SessionError` → exit 1/2/3 preserving upstream codes where possible.
- **GUI:** match on `AdbError` / `SessionError` variants for UX; avoid stringly-only errors for recoverable cases.
- Do **not** force `anyhow` inside `gnirehtet-relay` packet path.

### CLI parse errors

Replace `String` with a small `ArgsError` (thiserror) when touching `cli_args.rs` — low priority; behavior unchanged.

### Relay internal errors (proposal, phased)

| Phase | Action |
|-------|--------|
| 1–2 (MVP sidecar) | Leave `io::Result` + panics; child death = `RelayProcessError::EarlyExit` surfaced to UI |
| 3 | Change `on_ready` catch-all from `panic!` to `error!` + `self.close(selector)` |
| 3 | Keep asserts/panics for true invariant violations in header parsers (programmer errors) |

**Do not** invent a rich `RelayError` enum for every packet drop in Phase 1 — DEVELOP.md intentionally drops invalid packets with warnings (`router.rs`).

### Backpressure is not an error storm

**FACT:** `WouldBlock` + `"Client buffer full"` is flow control.  
**PROPOSAL:** document in API comments; controllers must not treat child stderr warnings as fatal unless the process exits.

---

## Mapping table: today → tomorrow

| Today | Crate | Tomorrow |
|-------|-------|----------|
| `CommandExecutionError` | adb | `AdbError` (thiserror; same variants) |
| `Result<_, String>` args | cli | `ArgsError` (optional) |
| `io::Error` from `relay()` | relay | unchanged public; process exit for MVP consumers |
| `panic!` in handlers | relay | Phase 3 → close+log |
| Logged AdbMonitor failures | adb | Optional callback `on_monitor_error`; still retry |
| `exit(3)` on execution error | cli | Map `SessionError` / `AdbError` |

---

## Process-level errors (MVP-critical)

Because the relay is a **child**:

1. **Spawn failure** (binary missing, permission) → hard error before any adb start (or after policy choice).
2. **Immediate exit** (port in use, poll create fail) → fail the session; surface stderr snippet if captured.
3. **Mid-session crash** (panic in handler) → GUI notifies; offer restart relay + `tunnel`/`start` (Android `PersistentRelayTunnel` may already retry — DEVELOP.md).
4. **Kill on stop** → ignore broken-pipe on log readers; treat wait success as clean.

Do not try to decode mio-internal failures over IPC in MVP — **exit status + logs** are enough.

---

## What not to do

- Do not wrap every `debug!` packet drop in `Result` (noise; changes hot paths).
- Do not make `gnirehtet-relay` depend on `anyhow`.
- Do not change Android-facing failure modes (e.g. silent no-op when client already started — **FACT** documented in `StartCommand` description) while “improving” errors.
- Do not treat `WouldBlock` as session failure.
- **PROTOCOL — DO NOT** change error bytes on the wire — there are none beyond TCP close; keep it that way.

---

## Test recommendations

| Area | Approach |
|------|----------|
| `ProcessStatusError` Display | Unit test with mocked `ExitStatus` if feasible |
| `AdbClient` | Trait `CommandRunner` returning canned `Output` / errors |
| Controller | Child stub that exits 1 → `EarlyExit` |
| Relay | Integration: bind failure on occupied port returns `io::Error` from `run` before loop |

---

## Summary decision

| Layer | Crate | Error tool |
|-------|-------|------------|
| Packet path | `gnirehtet-relay` | `io::Result` + log/close; panic hygiene in Phase 3 |
| ADB | `gnirehtet-adb` | `thiserror` enums (evolve `execution_error.rs`) |
| Session / sidecar | `gnirehtet-controller` | `thiserror` composing adb + process |
| CLI | `gnirehtet-cli` | map to exit codes; optional anyhow |
| GUI | `gnirehtet-desktop` | match structured errors; show logs from child |

This preserves upstream behavior, favors extract-then-reuse, and matches the sidecar MVP so GUI stability does not depend on eliminating relay panics first.
