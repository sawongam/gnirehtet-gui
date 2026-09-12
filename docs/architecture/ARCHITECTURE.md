# Architecture

**Project:** Open-source desktop GUI successor / front-end for Genymobile’s [gnirehtet](https://github.com/Genymobile/gnirehtet)  
**Role of this doc:** System context, module boundaries, and interfaces.  
**Upstream pin (verified):** `Genymobile/gnirehtet` @ `1eb2e58` (2023-07-09), tagged release **v2.5.1**  
**Status:** Draft v0.1 — provisional until stack validation gates pass.

---

## 1. Problem frame

Gnirehtet already provides reverse tethering (device uses the computer’s network) over `adb`, without root. Upstream is **CLI-only** on the host (“The application has no UI, and is intended to be controlled from the computer only” — README). This project adds a **modern cross-platform desktop GUI** while preserving the proven networking path where practical.

---

## 2. Facts from upstream (source-verified)

| Fact | Evidence |
|------|----------|
| Two host components + one device client | README, DEVELOP.md |
| Android client uses `VpnService` to capture raw IPv4 packets | DEVELOP.md; `GnirehtetService.java` |
| Client connects via Unix abstract local socket `gnirehtet`, after `adb reverse localabstract:gnirehtet tcp:<port>` | `RelayTunnel.java`; `cmd_tunnel` in `relay-rust/src/main.rs` |
| Default relay listen port `31416` | `cli_args.rs` `DEFAULT_PORT` |
| On accept, relay writes a 4-byte client id; client must read it before considering itself connected | DEVELOP.md; `RelayTunnel.readClientId` |
| Payload is raw IPv4 packets (TCP/UDP); **no IPv6** | README |
| Relay does L3↔L5 translation (userspace TCP state for TCP; header strip/add for UDP) | DEVELOP.md |
| Relay implementations: `relay-java/` (Java 8 NIO) and `relay-rust/` (mio) | README, DEVELOP.md |
| Upstream **recommends Rust** relay (lower CPU/RAM, no JRE) | README “Which one to choose?” |
| Rust crate exposes library entry `relaylib::relay(port)` plus a rich CLI binary | `lib.rs`, `main.rs` |
| CLI commands: `install`, `uninstall`, `reinstall`, `run`, `autorun`, `start`, `autostart`, `stop`, `restart`, `tunnel`, `relay` | `main.rs` `COMMANDS` |
| Device control via intents `com.genymobile.gnirehtet.START` / `.STOP` on `GnirehtetActivity` | `AndroidManifest.xml`, `GnirehtetActivity.java` |
| Optional CLI params: serial, `-d` DNS, `-r` routes, `-p` port | `cli_args.rs` |
| Env overrides: `ADB`, `GNIREHTET_APK` | README / `main.rs` |
| License: **Apache License 2.0** | `LICENSE` |
| Upstream not actively maintained (major blockers only) | README |

---

## 3. Target system context

```text
┌─────────────────────────────────────────────────────────────┐
│ Desktop host                                                │
│  ┌──────────────┐   commands/events   ┌──────────────────┐  │
│  │ desktop-ui   │◄───────────────────►│ host-orchestrator│  │
│  │ (Tauri webview│                     │ (Rust preferred) │  │
│  │  + Svelte)   │                     └────────┬─────────┘  │
│  └──────────────┘                              │            │
│         │                                      │ adb spawn  │
│         │ lifecycle                            ▼            │
│         │                              ┌──────────────┐     │
│         └─────────────────────────────►│ relay-core   │     │
│                    (sidecar or in-proc)│ (gnirehtet   │     │
│                                        │  relay-rust) │     │
│                                        └──────┬───────┘     │
│                                               │ TCP :31416  │
└───────────────────────────────────────────────┼─────────────┘
                                                │ adb reverse
                                                ▼
┌─────────────────────────────────────────────────────────────┐
│ Android device                                              │
│  abstract socket "gnirehtet" ◄── VpnService (APK client)    │
│  raw IPv4 packets to/from TUN                               │
└─────────────────────────────────────────────────────────────┘
```

External dependency that remains **out of process and user-provided**: platform `adb` (Android SDK platform-tools). The GUI must discover and surface adb health; it should not vendor a full SDK.

---

## 4. Module boundaries

### 4.1 `android-client` (reuse first)

**Owns:** VPN capture, tunnel reconnect (`PersistentRelayTunnel`), packet framing to/from relay, on-device VPN permission UX.

**Does not own:** Host UI, adb orchestration, relay networking.

**Boundary rule:** Treat the APK as a **compatibility surface**. Prefer shipping/building the upstream APK (or a minimal fork only when forced by Android API breakage). Do not rewrite VpnService for the GUI MVP.

**Public control surface (stable for orchestrator):**

- Install/uninstall APK package `com.genymobile.gnirehtet`
- `adb reverse localabstract:gnirehtet tcp:<port>`
- Start: `am start -a com.genymobile.gnirehtet.START -n com.genymobile.gnirehtet/.GnirehtetActivity` (+ extras for DNS/routes as upstream already supports)
- Stop: action `STOP` on the same activity

### 4.2 `relay-core` (reuse Rust relay)

**Owns:** Accepting client TCP connections, assigning client ids, IPv4 packet parse/route, TCP/UDP connection mapping, L3↔L5 translation.

**Library surface today:** `relaylib::relay(port: u16) -> io::Result<()>` — blocking event loop (mio).

**Integration modes (ordered preference for reliability):**

1. **MVP:** Run upstream `gnirehtet` binary / `relay` subcommand as a **managed child process** (Tauri sidecar or orchestrator-spawned).
2. **Later:** Link `relaylib` in-process behind a worker thread **only after** dependency modernization and a clean shutdown API exist (today: Ctrl+C oriented; no graceful stop API in `lib.rs`).

**Does not own:** adb device monitoring UI policy (though upstream CLI’s `adb_monitor.rs` may be reused or reimplemented in orchestrator).

### 4.3 `host-orchestrator`

**Owns:** Device discovery (`adb devices -l`), serial selection, install/start/stop/tunnel/relay lifecycle, log aggregation, error taxonomy for the UI, APK path resolution, adb path resolution.

**Interface (conceptual; exact IPC TBD in implementation):**

```text
list_devices() -> Device[]
ensure_adb() -> AdbStatus
install(serial)
start(serial, options: {dns?, routes?, port?})
stop(serial)
reset_tunnel(serial, port?)
start_relay(port?) / stop_relay()
subscribe(events: DeviceChanged | RelayState | LogLine | Error)
```

**MVP implementation strategy:** Thin wrapper around the **existing CLI command semantics** (same adb invocations as `main.rs`) rather than a ground-up reimplementation. That preserves behavior parity and reduces rewrite risk.

### 4.4 `desktop-shell`

**Owns:** Windowing, settings persistence, presentation of devices/relay/logs, user workflows (one-click run, multi-device later), packaging.

**Provisional stack:** Tauri 2 (Rust host) + Svelte/TypeScript frontend.

**Does not own:** Packet processing. UI talks only to `host-orchestrator` commands/events.

### 4.5 `protocol` (documentation + conformance tests)

Document the wire protocol as a first-class module even if code stays in `relay-core` / client:

1. Client TCP connect (via adb reverse to host port).
2. Server → client: big-endian `u32` client id.
3. Bidirectional: raw IPv4 packets (length implied by IP header / stream reassembly as implemented).

Conformance tests protect reuse across any future relay modernization.

---

## 5. Process & trust boundaries

| Boundary | Trust notes |
|----------|-------------|
| UI ↔ orchestrator | Same app process (Tauri invoke) or local IPC; not network-exposed |
| Orchestrator ↔ adb | Spawns local `adb`; user must have authorized the device |
| Relay ↔ device client | Localhost TCP via adb reverse; not a general internet service |
| On-device VPN permission | Always user-granted on device; GUI cannot skip this |

---

## 6. Non-goals (architecture level)

- Replacing `adb` with a custom USB stack in MVP
- IPv6 support in MVP (upstream explicitly lacks it)
- Rewriting the userspace TCP relay “for cleanliness” without measured need
- Making the Android app the primary UX (host GUI is the product surface)

---

## 7. Consistency rules for other agents

1. **Never invent upstream behavior** — cite path + revision.
2. Prefer **reuse → wrap → refactor → rewrite** in that order.
3. Any proposal that changes the client↔relay packet contract needs a protocol ADR and dual-compat plan.
4. Desktop stack remains **provisional** until validation gates in `TECH_DECISIONS.md` pass.
