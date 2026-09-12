# Gnirehtet Rust — Current Architecture Map & Target Rationale

**Upstream pin:** Genymobile/gnirehtet `@1eb2e58` / **v2.5.1**  
**Source under analysis:** `relay-rust/` (paths below are relative to that tree unless noted).  
**Canonical doc home (later):** `docs/architecture/rust/`

This document separates **FACTS** (what the upstream code does) from **PROPOSALS** (what we recommend). Anything that would change the **client↔relay wire protocol** is called out explicitly.

---

## FACTS — Current architecture

### 1. Crate / target layout

`Cargo.toml` defines a single package `gnirehtet` 2.5.1 (edition 2018) with two targets:

| Target | Path | Public surface |
|--------|------|----------------|
| Library `relaylib` | `src/lib.rs` | `pub fn relay(port: u16) -> io::Result<()>` + `pub use crate::relay::byte_buffer` |
| Binary `gnirehtet` | `src/main.rs` | Full CLI + ADB orchestration |

Dependencies: `mio = "0.6"`, `slab = "0.4"`, `log = "0.4"`, `chrono = "0.4"`, `byteorder = "1.3"`, `rand = "0.7"`, `ctrlc` (used from binary).

```
src/
├── lib.rs                 # relaylib entry: Relay::new(port).run()
├── main.rs                # Command trait + cmd_* + exec_adb + ctrlc
├── cli_args.rs            # CommandLineArguments, PARAM_*, DEFAULT_PORT=31416
├── execution_error.rs     # CommandExecutionError, ProcessIo/Status, Cmd
├── adb_monitor.rs         # host:track-devices over TCP 127.0.0.1:5037
├── logger.rs              # SimpleLogger via log facade
└── relay/                 # packet path (almost entirely private)
    ├── mod.rs             # pub use Relay; pub mod byte_buffer
    ├── relay.rs           # poll_loop over Selector
    ├── selector.rs        # mio::Poll + slab::Slab<Rc<dyn EventHandler>>
    ├── tunnel_server.rs   # TcpListener → Client
    ├── client.rs          # device tunnel TCP + Router + id handshake
    ├── router.rs          # ConnectionId → TcpConnection | UdpConnection
    ├── tcp_connection.rs  # TCP TCB / state machine (~840 LOC)
    ├── udp_connection.rs  # UDP + IDLE_TIMEOUT_SECONDS = 120
    ├── packet_source.rs / packetizer.rs / stream_buffer.rs / …
    └── ipv4_* / tcp_header / udp_header / transport_header / …
```

`../DEVELOP.md` documents Java NIO ↔ Rust mio parity: one selector, monothreaded packet handling, L3↔L5 NAT-like translation.

### 2. Module boundaries today

| Concern | Lives in | Linked how |
|---------|----------|------------|
| CLI dispatch | `main.rs` (`COMMANDS`, `trait Command`) | binary only |
| Arg parsing | `cli_args.rs` | binary only |
| ADB process cmds | `main.rs` (`exec_adb`, `cmd_install`/`start`/`stop`/`tunnel`/…) | binary only |
| Device watch | `adb_monitor.rs` | binary; uses `relaylib::byte_buffer::ByteBuffer` |
| Errors (process) | `execution_error.rs` | binary only |
| Logging init | `logger.rs` | binary only |
| Relay packet path | `relay/*` via `lib.rs` | library; binary calls `relaylib::relay(port)` |

**Library vs binary gap:** almost all reusable orchestration is trapped in the binary. The lib exposes only blocking `relay(port)`.

### 3. Relay lifecycle (facts)

1. `Relay::new(port)` (`relay/relay.rs`) stores port.
2. `run()` creates `Selector`, `TunnelServer::create(port, &mut selector)`, then `poll_loop` forever.
3. `poll_loop`: `selector.poll` with timeout toward UDP cleaning (`CLEANING_INTERVAL_SECONDS = 60`, driven by `IDLE_TIMEOUT_SECONDS`); then `selector.run_handlers(&events)`.
4. There is **no** stop flag, wake fd, or graceful shutdown API. The loop only exits if `poll` returns a non-Interrupted error.
5. Binary `cmd_relay` → `relaylib::relay(port)?` blocks the calling thread.
6. Binary `cmd_run`: `async_start(...)` on a thread, installs Ctrl+C handler that `cmd_stop`s then **`exit(0)`**, then blocks in `cmd_relay`. Process exit is the shutdown mechanism.

### 4. Client↔relay wire protocol (facts — do not change lightly)

Documented in `DEVELOP.md` and implemented in `client.rs` / Android side:

1. After TCP accept, relay writes a **4-byte client id** (big-endian via `binary::to_byte_array`) before any IP payload (`pending_id_bytes = 4`).
2. Thereafter, TCP payload is a **stream of raw IPv4 packets** (no length prefix between packets; framing by IP header length — see `Ipv4PacketBuffer`).
3. Transport: device reaches relay via `adb reverse localabstract:gnirehtet tcp:{port}` (`cmd_tunnel` in `main.rs`).
4. Bind address: relay listens on **127.0.0.1** only (`tunnel_server.rs` `start_socket`).

**Wire-protocol change flag:** any change to id handshake, packet framing, abstract socket name, or default port semantics is a **cross-repo protocol change** (Android app + relay). MVP must not touch these.

### 5. Ownership & concurrency (facts)

- Inside relay: `Rc<RefCell<T>>` + `Weak` throughout (`TunnelServer`, `Client`, `Router`, connections). Types are **`!Send` / `!Sync`**.
- Selector handlers are `Rc<dyn EventHandler>` closures that `borrow_mut` the owning object.
- Binary threads: `thread::spawn` in `async_start` and `cmd_autorun`; `AdbMonitor::monitor()` is an infinite sync loop (retries + optional `adb start-server`).
- No `Mutex`/`Arc` in the packet path; DEVELOP.md states this is intentional.

### 6. Multi-device (facts)

- One relay process/listener serves **many** `Client`s (`TunnelServer.clients: Vec<Rc<RefCell<Client>>>`).
- `autorun` / `autostart`: `AdbMonitor` callbacks call `async_start(Some(serial), …)` per newly connected device.
- Each device needs its own `adb reverse` (per-serial) onto the **same** host port.

### 7. I/O, logging, errors (facts)

- I/O: mio 0.6 sockets + std `Read`/`Write` into custom buffers; `retry_on_intr!` in `relay/interrupt.rs`.
- Logging: `log` macros with string `TAG`s; binary sets `SimpleLogger` at Info.
- Relay errors: predominantly `io::Result`; many handler paths `panic!("Unexpected unhandled error")` (`client.rs`, `tcp_connection.rs`, `udp_connection.rs`).
- CLI/ADB errors: `CommandExecutionError` (`execution_error.rs`); CLI parse errors are `String`.

---

## PROPOSALS — Target architecture

### MVP constraint (Lead Architect)

> **MVP keeps the relay as a managed sidecar / child process.**  
> Do **not** require in-process `relaylib` embed for MVP.  
> Library extraction is fine for reuse, but **GUI and CLI orchestration for MVP spawn and manage the relay process**.

Implications:

- Orchestrator talks to an existing (or built) `gnirehtet` / `gnirehtet-relay` **binary** via OS process API (`std::process::Command` / later a small process supervisor).
- Primary control plane = **same CLI verbs** the upstream binary already exposes: `run`, `start`, `stop`, `tunnel`, `relay` (plus install/uninstall as needed).
- In-process `RelayHandle` / embedding is a **post-MVP** option, not a gate.

### Target shape (extract/refactor, not rewrite)

Prefer splitting **concerns that already exist as modules** in relay-rust:

```
┌─────────────────────────────────────────────────────────┐
│ Frontends (MVP)                                         │
│  gnirehtet-cli  |  gnirehtet-desktop (GUI)              │
│  both: spawn/supervise relay child + drive ADB verbs    │
└────────────────────────────┬────────────────────────────┘
                             │ process spawn + argv / IPC-light
                             ▼
┌────────────────────────────────────────────────────────┐
│ gnirehtet-controller  (library)                        │
│  mirrors CLI semantics: run/start/stop/tunnel/relay    │
│  owns child Process for `relay`, Adb ops for devices   │
└───────────────┬─────────────────────┬──────────────────┘
                │                     │
                ▼                     ▼
┌───────────────────────┐   ┌────────────────────────────┐
│ gnirehtet-relay       │   │ gnirehtet-adb              │
│ (lib + thin binary)   │   │ AdbClient + AdbMonitor     │
│ mio packet path UNCHANGED in Phase 1–2               │
└───────────────────────┘   └────────────────────────────┘
```

**Naming note:** suggested names `gnirehtet-core` / `gnirehtet-adb` / `gnirehtet-controller` / `gnirehtet-cli` / `gnirehtet-desktop` are evaluated in `CRATE_STRUCTURE.md`. Evidence favors **`gnirehtet-relay`** over vague `gnirehtet-core`.

### Sync vs async / tokio vs mio (proposal)

| Layer | Phase 1–2 | Phase 3+ |
|-------|-----------|----------|
| Packet path | **Keep mio 0.6 selector** (`relay/selector.rs`, `relay/relay.rs`) | Optional mio upgrade or isolated async experiment — **not** MVP |
| ADB process + monitor | Keep sync `Command` + sync TCP (as today) | Optional async wrappers if GUI wants; not required |
| Controller | Sync process supervisor + threads (mirror `main.rs`) | May add async API façade later |
| GUI | Own UI toolkit loop; talk to controller; **child process** for relay | Optional later: in-process embed behind feature flag |

**Do not** rewrite working networking code for style. mio/modernization is **Phase 3** unless a hard build/security breaker appears (none found at pin `1eb2e58` / v2.5.1 for normal Linux builds).

### Sidecar lifecycle (MVP proposal)

Mirrors upstream `cmd_run` / `cmd_relay` / `cmd_autorun` behavior without embedding:

1. Controller starts child: `gnirehtet relay [-p PORT]` (or dedicated relay binary with same behavior).
2. Controller runs ADB: `tunnel` + `start` (or monitor→start for autorun), same argv semantics as CLI.
3. On stop: ADB `stop` for device(s); then terminate relay child (SIGTERM/kill); wait for exit.
4. GUI observes: child stdout/stderr (log lines) + process exit status + AdbMonitor callbacks — **not** internal `Client` structs.

This matches today’s process model more closely than an in-process `Rc<RefCell<_>>` embed (which is `!Send` and hard to cancel — see `CONCURRENCY_MODEL.md`).

### What stays stable

- Wire protocol (id + raw IPv4 stream) — **no change**.
- Default port `31416`, abstract name `gnirehtet`, localhost bind.
- TCP/UDP connection semantics and cleaning intervals.
- CLI verb names and flag meanings (`-d`, `-r`, `-p`, serial).

### What changes (proposals only)

- Extract ADB + controller out of `main.rs` into libraries callable by CLI and GUI.
- Process supervisor API instead of `exit(0)` from Ctrl+C inside a shared process (GUI must not `exit` the whole app).
- Break `adb_monitor` → `relaylib::byte_buffer` dependency (duplicate small buffer or move buffer to a tiny shared util — **not** a protocol change).

---

## Evaluation snapshot

| Topic | Fact | Proposal |
|-------|------|----------|
| Sync vs async | Relay is sync mio loop; ADB sync | Keep; no tokio in packet path for Phases 1–2 |
| Library vs binary | Lib too thin; binary owns orchestration | Extract adb + controller; relay binary remains sidecar |
| Error strategy | Split `CommandExecutionError` vs `io::Result` + panics | See `ERROR_HANDLING.md` |
| Event/state | Selector tokens + connection state machines | Unchanged in core; controller adds ProcessState |
| Cancellation | None in relay; AdbMonitor infinite; Ctrl+C exits process | Kill/wait child; stoppable monitor loop |
| Multi-device | One relay, many clients; AdbMonitor | Same with sidecar; controller tracks serials |
| Testability | Some unit tests; adb/relay hard to mock | Trait-wrap AdbExecutor; relay stays integration-tested as process |

---

## Risks / surprises (facts)

1. **`Relay::run` never returns on success** — sidecar kill is the practical stop for MVP.
2. **`cmd_run` Ctrl+C calls `exit(0)`** — unsafe inside a GUI process; another reason MVP uses a **child** relay.
3. **`AdbMonitor::monitor` never returns** — needs a stop seam before GUI embed of monitor (can still run monitor on a thread with a stop flag without touching wire protocol).
4. **`adb_monitor` depends on `relaylib::byte_buffer`** — false coupling.
5. **Panics in I/O handlers** — a single unexpected error aborts the whole relay process (acceptable for sidecar isolation; bad for in-process embed).
6. **mio 0.6 / edition 2018** — old but building at pin; modernization deferred to Phase 3.
