# Gnirehtet Rust — Concurrency Model

**Upstream pin:** Genymobile/gnirehtet `@1eb2e58` / v2.5.1  
**Evidence paths:** relative to `relay-rust/`.  
**MVP constraint:** relay runs as a **managed sidecar/child process**; GUI/CLI orchestrate via process + ADB, not in-process mio embed.

---

## FACTS — Current mio selector model

### Single-threaded relay event loop

`relay/relay.rs`:

1. `Selector::create()` → wraps `mio::Poll` (`relay/selector.rs`).
2. `TunnelServer::create(port, &mut selector)` registers the `TcpListener` with a readable edge-triggered handler.
3. `poll_loop`:
   - Computes timeout from next UDP cleaning deadline (`CLEANING_INTERVAL_SECONDS = 60`, related to `udp_connection::IDLE_TIMEOUT_SECONDS = 120`).
   - `retry_on_intr!(selector.poll(&mut events, timeout))` (`relay/interrupt.rs`).
   - On deadline: `tunnel_server.borrow_mut().clean_up(selector)`.
   - Else: `selector.run_handlers(&events)`.

`Selector` (**FACT** `selector.rs`):

- `handlers: Slab<Rc<dyn EventHandler>>` keyed by `mio::Token`.
- `register` / `reregister` / `deregister`; deferred `tokens_to_remove` cleaned after the current event batch (avoids use-after-remove during nested callbacks).
- Handlers receive `&mut Selector` + `Event` and typically `Rc::clone` + `RefCell::borrow_mut` on the owner (`TunnelServer`, `Client`, `TcpConnection`, `UdpConnection`).

**Consequence:** the entire packet path is **monothreaded** and uses **`Rc<RefCell<_>>`**, hence **`!Send` / `!Sync`**. DEVELOP.md states this matches the Java NIO design and avoids lock contention on packets.

### Registered channels (facts)

| Handle | Type | Registration site |
|--------|------|-------------------|
| Server | `mio::net::TcpListener` | `tunnel_server.rs` |
| Device client | `mio::net::TcpStream` | `client.rs` (`PollOpt::level`) |
| Remote TCP | `mio::net::TcpStream` | `tcp_connection.rs` |
| Remote UDP | `mio::net::UdpSocket` | `udp_connection.rs` |

Interest toggling implements backpressure: client buffer full → TCP connections park packets via `PacketSource` / `pending_packet_sources` (`client.rs`, `packet_source.rs`) until client socket is writable again.

### Threads outside the selector (facts)

| Location | Threading | Lifetime |
|----------|-----------|----------|
| `Relay::run` | Caller thread (usually main) blocked in `poll_loop` | Until process death / poll error |
| `async_start` (`main.rs`) | `thread::spawn` → `cmd_start` | Fire-and-forget; errors logged |
| `cmd_autorun` | `thread::spawn` → `cmd_autostart` → `AdbMonitor::monitor` | Infinite |
| `AdbMonitor::monitor` | Sync loop on its thread; `thread::sleep` on repair | Infinite; no cancel |
| `cmd_run` Ctrl+C | `ctrlc` handler thread/callback | `cmd_stop` then **`exit(0)`** |

There is **no** shared queue between AdbMonitor and the relay except the OS (devices connect via adb reverse to the listening port).

### Cancellation & shutdown today (facts)

| Component | Cancel API? | Actual stop |
|-----------|-------------|-------------|
| `poll_loop` | None | Kill process; or rare `io::Error` from `poll` |
| `AdbMonitor::monitor` | None | Kill process / abandon thread |
| `cmd_run` | Ctrl+C | Stop Android client via adb; `exit(0)` — **does not** gracefully unwind relay |
| Connections | `close` / expiry | Local to selector thread |

**Surprise:** even “clean” Ctrl+C in `run` does not return from `relaylib::relay`; it aborts the process.

---

## FACTS — Why in-process embed is hard (motivates MVP sidecar)

1. `Rc<RefCell<T>>` relay graph cannot move to another thread or into `async` tasks without redesign.
2. `poll` has no built-in wake pipe for “stop from UI thread”.
3. Handler `panic!` on unexpected errors (`client.rs`, `tcp_connection.rs`, `udp_connection.rs`) would take down the **whole GUI process** if embedded.
4. `logger::init` / `exit(0)` patterns assume a dedicated process.

Sidecar isolation turns these into child-process problems (acceptable for MVP).

---

## PROPOSALS — Recommended model for CLI + GUI (MVP)

### Architecture: process isolation for the relay

```
GUI or CLI process                      Child process
─────────────────                      ─────────────
UI / argv
   │
   ▼
controller::RunSession / AutorunSession
   │  spawn                            
   ├──► std::process::Command ────────► gnirehtet relay -p PORT
   │                                      mio poll_loop (unchanged)
   ├──► AdbClient (sync) ─────────────► adb binary / adbd :5037
   └──► AdbMonitor (optional thread) ─► track-devices
```

- **Relay concurrency:** unchanged — one OS thread inside the child, mio selector.
- **Orchestrator concurrency:** few OS threads (monitor, optional log-pump readers); sync APIs preferred.
- **No tokio required** for MVP.

### CLI verb mapping to concurrency

| Verb | Threads / processes (proposal) |
|------|--------------------------------|
| `relay` | One child (or foreground process) running poll_loop |
| `tunnel` / `start` / `stop` | Sync adb `Command` on caller thread |
| `run` | Relay **child** + sync/async_start for client; supervisor waits on child; stop = adb stop + child kill |
| `autorun` | Relay child + dedicated AdbMonitor thread with **stop flag** |

Mirror upstream semantics; change only the **process boundary** for `run` (child instead of in-process `relaylib::relay` + `exit`).

### Cancellation / shutdown (proposal)

#### Relay child

1. Prefer `Child::kill()` / SIGTERM then wait with timeout (platform-specific niceties later).
2. Accept that mio loop has no graceful drain in Phase 1–2 — TCP connections drop; Android client reconnects via `PersistentRelayTunnel` (documented in DEVELOP.md on the app side).
3. Do **not** require wake-fd inside mio for MVP.

#### AdbMonitor

**PROPOSAL:** `monitor_until(stop: &AtomicBool)` or check a `Receiver<()>` after each packet / on IO error before reconnect sleep.  
**FACT gap:** current `monitor()` cannot stop.

#### GUI close

1. Set monitor stop → join (timeout).
2. `session.stop()`: adb `stop` for known serials → kill relay child → wait.
3. Never call `process::exit` from library code.

### Logging bridge

- Child inherits or pipes stdout/stderr; GUI reads lines on a background thread into the UI log model.
- Libraries keep using `log` macros; only frontends install loggers (`logger.rs` stays binary-side).

### Multi-device concurrency

**FACT:** one relay listener, many `Client`s; AdbMonitor starts one client per new serial.  
**PROPOSAL:** unchanged with sidecar — one relay child per host session; controller tracks `HashSet<String>` serials for stop-all.

Port conflicts: only one relay per port; controller owns the port for the session.

---

## PROPOSALS — Post-MVP in-process path (non-blocking for MVP)

If product later requires embed:

1. Keep packet path on a **dedicated `std::thread`**.
2. Add a mio-wakeable stop source registered on `Selector` (pipe/`Registration`) — smallest seam around `poll_loop`, not a tokio rewrite.
3. Cross-thread API: `Send` control messages only (`Start`, `Stop`); never share `Rc<RefCell<Client>>` with UI.
4. Feature-gate; default remains sidecar.

**Still avoid:** converting the selector to tokio `TcpListener` in the same PR as product features.

---

## Sync vs async evaluation

| Approach | Fit for packet path | Fit for MVP orchestrator |
|----------|---------------------|--------------------------|
| Current mio 0.6 loop | **Best** — already correct | N/A (child process) |
| std threads + process | N/A | **Best for MVP** |
| tokio runtime wrapping mio | High cost, little gain Phase 1–2 | Optional later for GUI async |
| async ADB only | N/A | Nice-to-have; sync `Command` is enough |

**Decision:** Phase 1–2 = **mio stays in relay binary**; orchestrator = **stdlib process + threads**. Phase 3 = mio upgrade optional; tokio not on the packet path unless explicitly resourced as a rewrite.

---

## Testability implications

| Layer | How to test concurrency |
|-------|-------------------------|
| Selector handlers | Existing unit tests stay single-threaded |
| AdbMonitor framing | Unit tests already use buffers (`adb_monitor.rs` tests) |
| Controller | Mock `AdbExecutor`; mock/stub `RelayProcess` (don't spawn) |
| Integration | Spawn real relay binary on port 0 / free port; local TCP client |

---

## Explicit non-changes (wire / concurrency)

- **PROTOCOL — DO NOT** change client id handshake timing relative to first payload without Android coordination.
- Do not add worker-pool parallelism inside packet processing (ordering / TCB assumptions are single-threaded).
- Do not wrap `RefCell` borrow failures in recovery — fix logic bugs; sidecar limits blast radius.
