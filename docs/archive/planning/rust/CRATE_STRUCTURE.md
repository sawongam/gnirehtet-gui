# Gnirehtet Rust — Proposed Crate / Workspace Structure

**Upstream pin:** Genymobile/gnirehtet `@1eb2e58` / v2.5.1  
**Evidence root:** `relay-rust/`  
**MVP rule:** orchestration spawns the relay as a **child process**; libraries support that (and future reuse), not an in-process embed requirement.

Facts vs proposals are labeled. **Wire protocol** (4-byte client id + raw IPv4 stream over the reverse-tunneled TCP) must not change.

---

## FACTS — What exists today

Single Cargo package:

```toml
# relay-rust/Cargo.toml
[package] name = "gnirehtet" version = "2.5.1"
[lib] name = "relaylib" path = "src/lib.rs"
# default binary: src/main.rs → gnirehtet
```

| Unit | Path | Role |
|------|------|------|
| `relaylib` | `src/lib.rs` + `src/relay/*` | Blocking `relay(port)`; private packet path; re-exports `byte_buffer` |
| binary | `src/main.rs` + `cli_args` + `execution_error` + `adb_monitor` + `logger` | All CLI verbs + ADB |

Natural seams already visible in the tree (not yet crates):

1. **Relay loop** — `relay/relay.rs`, `selector.rs`, `tunnel_server.rs`, `client.rs`, connections, headers.
2. **ADB orchestration** — `main.rs` `exec_adb` / `cmd_*` + `adb_monitor.rs`.
3. **CLI** — `Command` trait, `cli_args.rs`, `logger.rs`, `main`.

---

## PROPOSALS — Name evaluation

Suggested names from the brief: `gnirehtet-core`, `gnirehtet-adb`, `gnirehtet-controller`, `gnirehtet-cli`, `gnirehtet-desktop`.

| Suggested | Verdict | Evidence-based preference |
|-----------|---------|---------------------------|
| `gnirehtet-core` | **Weak** | Upstream lib is specifically a **relay** (`relaylib`, `Relay`, `relay/`), not a generic “core”. “Core” invites dumping ADB/CLI into it. |
| `gnirehtet-relay` | **Prefer** | Matches `relaylib` / `relay::Relay` / DEVELOP.md “relay server”. |
| `gnirehtet-adb` | **Good** | Clear home for `AdbMonitor`, `exec_adb`, install/start/stop/tunnel. |
| `gnirehtet-controller` | **Good** | Session/process supervisor mirroring CLI verbs; owns sidecar child + adb calls. |
| `gnirehtet-cli` | **Good** | Thin argv → controller (or → child `gnirehtet` binary during transition). |
| `gnirehtet-desktop` | **OK** | GUI frontend; same controller API as CLI; name can be app-specific later. |

**Recommended workspace names (proposal):**

1. `gnirehtet-relay`
2. `gnirehtet-adb`
3. `gnirehtet-controller`
4. `gnirehtet-cli`
5. `gnirehtet-desktop` (GUI; can lag CLI)

Optional later (not MVP): `gnirehtet-common` only if buffer/error types are shared without pulling mio — avoid premature crate.

---

## Proposed dependency graph

```
gnirehtet-desktop ──┐
                    ├──► gnirehtet-controller ──┬──► gnirehtet-adb
gnirehtet-cli ──────┘                          │
                                               └──► (spawns process) gnirehtet-relay binary
                                                        ▲
                                                        │ links
                                                   gnirehtet-relay library
                                                   (mio packet path)
```

**MVP:** `controller` depends on `adb` as a **library**. It depends on the relay **binary on PATH / bundled path**, not on linking `relaylib` into the GUI process.

**Post-MVP (optional feature):** `controller` may offer `embed-relay` that links `gnirehtet-relay` and runs `Relay::run` on a dedicated thread — explicitly non-MVP.

Forbidden edges:

- `gnirehtet-adb` ↛ `gnirehtet-relay` (today `adb_monitor` → `relaylib::byte_buffer` is a **fact to break**).
- `gnirehtet-relay` ↛ `gnirehtet-adb` / CLI / ctrlc / logger init.
- GUI/CLI ↛ `relay::*` internals (headers, `TcpConnection`, etc.).

---

## Crate responsibilities & public APIs

### 1. `gnirehtet-relay` (from today’s `relaylib` + `relay/`)

**FACT move:** `src/relay/**`, current `lib.rs`.

**Public (proposal — minimal Phase 1):**

```rust
// Keep existing entry for binary compatibility
pub fn relay(port: u16) -> std::io::Result<()>;

pub struct RelayConfig {
    pub port: u16,  // still binds 127.0.0.1:{port} — wire/deploy semantics unchanged
}

pub struct Relay;
impl Relay {
    pub fn new(config: RelayConfig) -> Self;
    /// Blocks; same as today’s Relay::run / poll_loop.
    pub fn run(&self) -> std::io::Result<()>;
}

// binary crate or bin target in same package:
// main → parse -p → Relay::run  (the sidecar executable)
```

**Private (stay crate-private):**  
`Selector`, `TunnelServer`, `Client`, `Router`, `TcpConnection`, `UdpConnection`, `Ipv4Packet*`, headers, `Packetizer`, `PacketSource`, `retry_on_intr!`.

**Do not publish** header parsers as a separate crate until a second in-tree consumer exists.

**Binary:** thin `gnirehtet-relay` or retain `gnirehtet relay` subcommand that only runs this loop (sidecar).

---

### 2. `gnirehtet-adb` (from `adb_monitor.rs` + ADB helpers in `main.rs`)

**FACT sources:**

- `adb_monitor.rs` — `AdbMonitor`, `AdbMonitorCallback`, track-devices framing
- `main.rs` — `get_adb_path`, `get_apk_path`, `exec_adb`, `create_adb_args`, `must_install_client`, `cmd_install`/`uninstall`/`start`/`stop`/`tunnel`, intent action strings, `REQUIRED_APK_VERSION_CODE`

**Public API (proposal):**

```rust
pub struct AdbConfig {
    pub adb_path: PathBuf,       // default env ADB or "adb"
    pub apk_path: PathBuf,       // env GNIREHTET_APK or "gnirehtet.apk"
    pub adbd_addr: SocketAddr,   // default 127.0.0.1:5037
}

pub struct VpnOptions<'a> {
    pub dns_servers: Option<&'a str>,
    pub routes: Option<&'a str>,
    pub port: u16,               // for reverse + intent extras if any
}

pub struct AdbClient { /* config */ }

impl AdbClient {
    pub fn install(&self, serial: Option<&str>) -> Result<(), AdbError>;
    pub fn uninstall(&self, serial: Option<&str>) -> Result<(), AdbError>;
    pub fn tunnel(&self, serial: Option<&str>, port: u16) -> Result<(), AdbError>;
    pub fn start(&self, serial: Option<&str>, opts: &VpnOptions) -> Result<(), AdbError>;
    pub fn stop(&self, serial: Option<&str>) -> Result<(), AdbError>;
    pub fn must_install_client(&self, serial: Option<&str>) -> Result<bool, AdbError>;
    // same shell am start intents as main.rs — no protocol change to the Android app
}

pub trait AdbMonitorCallback: Fn(&str) {}
pub struct AdbMonitor { /* … */ }
impl AdbMonitor {
    pub fn new(callback: Box<dyn AdbMonitorCallback>) -> Self;
    /// Blocking loop (fact). Proposal: add `run_until(&AtomicBool)` or channel stop.
    pub fn monitor(&mut self);
}
```

**Private:** packet length parsing helpers, `ByteBuffer` (local copy — **break** dependency on relay).

**Errors:** evolve `execution_error.rs` types here (see `ERROR_HANDLING.md`).

---

### 3. `gnirehtet-controller` (new; logic from `cmd_run` / `cmd_autorun` / `cmd_relay` composition)

**Purpose:** Mirror existing CLI semantics so GUI and CLI share one policy.

| Verb | Upstream fact (`main.rs`) | Controller API (proposal) |
|------|---------------------------|---------------------------|
| `relay` | `cmd_relay` → block in `relaylib::relay` | `start_relay_process(port)` → `Child` + log pipes |
| `tunnel` | `cmd_tunnel` | `AdbClient::tunnel` |
| `start` | install-if-needed, tunnel, am start | `start_client(serial, opts)` |
| `stop` | am STOP intent | `stop_client(serial)` |
| `run` | async_start + ctrlc + relay | `RunSession::start` — spawn relay child, start client, on drop/stop: stop client + kill child |
| `autorun` | thread AdbMonitor + relay | `AutorunSession` — relay child + monitor thread |

```rust
pub struct RelayProcess {
    child: std::process::Child,
    // stdin unused; capture stdout/stderr for GUI log pane
}

pub struct RunSession { /* RelayProcess + serial + AdbClient + port */ }
impl RunSession {
    pub fn start(adb: AdbClient, serial: Option<String>, opts: VpnOptions, relay_bin: &Path) -> Result<Self, SessionError>;
    pub fn stop(self) -> Result<(), SessionError>; // stop client + terminate child
}

pub struct AutorunSession { /* … */ }
```

**MVP:** `relay_bin` points at built `gnirehtet` (invoke `relay -p …`) or dedicated relay executable. **No** `extern crate relaylib` inside the GUI process.

**Private:** retry/sleep timings (`thread::sleep(500ms)` after install — fact in `cmd_start`), ctrlc equivalent for CLI only.

---

### 4. `gnirehtet-cli`

**FACT move:** `cli_args.rs`, `logger.rs`, `Command` dispatch table from `main.rs`.

**Public:** none required (binary). May expose `CommandLineArguments::parse` for tests (already tested).

Depends on: `gnirehtet-controller` (+ thus adb). During Phase 0 transition, CLI may still be the monolith binary.

---

### 5. `gnirehtet-desktop`

Depends on: `gnirehtet-controller`, `gnirehtet-adb`.  
Spawns/supervises relay via controller. UI never imports mio or `relay::client`.

---

## What stays private (summary)

| Item | Crate | Visibility |
|------|-------|------------|
| mio `Selector` / handlers | relay | private |
| TCP TCB / `TcpState` | relay | private |
| `Ipv4Packet` zero-copy views | relay | private |
| Intent component names / action strings | adb | private constants (or `pub(crate)`) |
| track-devices framing | adb | private |
| Process argv construction for relay child | controller | private |
| `SimpleLogger` | cli (or desktop) | binary-only; libs use `log` only |

---

## Workspace layout (proposal)

```
gnirehtet-rust/                    # or keep under relay-rust workspace root
├── Cargo.toml                     # workspace
├── crates/
│   ├── gnirehtet-relay/
│   │   ├── Cargo.toml             # lib + bin
│   │   └── src/…                  # moved from relay-rust/src/relay + lib.rs
│   ├── gnirehtet-adb/
│   ├── gnirehtet-controller/
│   ├── gnirehtet-cli/
│   └── gnirehtet-desktop/         # optional in early phases
└── …
```

**Phase 0 alternative (lower risk):** keep one package, expose modules:

```text
relaylib::relay::*     (existing)
relaylib::adb::*       (move adb_monitor + helpers)
relaylib::controller::*
```

Then split packages once APIs stabilize. Prefer this if workspace tooling churn is costly — see `REFACTOR_PLAN.md`.

---

## Compatibility matrix

| Consumer | Needs relay lib linked? | Needs relay binary? | Needs adb lib? |
|----------|-------------------------|---------------------|----------------|
| MVP CLI | No (or yes only to *build* the sidecar) | Yes | Yes |
| MVP GUI | No | Yes (bundled) | Yes |
| Post-MVP embed | Yes | Optional | Yes |
| Unit tests for headers | Dev-dep / same crate | No | No |

---

## Wire-protocol checklist (must remain unchanged)

- [ ] 4-byte client id before payload (`client.rs`)
- [ ] Raw IPv4 stream framing (`ipv4_packet_buffer.rs`)
- [ ] `adb reverse localabstract:gnirehtet tcp:{port}`
- [ ] Listen `127.0.0.1`
- [ ] Default port `31416` (`cli_args::DEFAULT_PORT`)
- [ ] Android intent actions `com.genymobile.gnirehtet.START` / `.STOP`

Crate splits must not alter these.
