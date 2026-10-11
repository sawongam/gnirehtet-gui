# REUSE_ANALYSIS.md

**Upstream pin:** `/workspace/gnirehtet-upstream` @ `1eb2e58` / v2.5.1  
**Goal:** What a desktop GUI successor can keep, wrap, or must rewrite.  
**Policy cited:** `/workspace/gnirehtet-gui-docs/MIGRATION_PLAN.md` (Reuse → Wrap → Refactor → Rewrite); `/workspace/gnirehtet-gui-docs/TECH_DECISIONS.md` TD-01/TD-02.  
**Network truth:** `/workspace/gnirehtet-net-docs/*` (protocol & failure modes are not reopened here unless contradicted).

---

## 0. Rust Architect recommendations (to reconcile)

The following recommendations were supplied for reconciliation against `relay-rust` evidence:

1. **Break `adb_monitor` → `relaylib::byte_buffer` coupling.**  
2. **MVP: keep the relay as a sidecar child process (not an in-process lib).**  
3. **Prefer crate name `gnirehtet-relay` over `gnirehtet-core`.**

### Verification against `/workspace/gnirehtet-upstream/relay-rust`

| # | Verdict | Evidence |
|---|---------|----------|
| 1 | **Confirmed real coupling.** `adb_monitor` lives in the **binary** crate (`main.rs` `mod adb_monitor`) but imports `relaylib::byte_buffer::ByteBuffer`. `lib.rs` re-exports `byte_buffer` with no other public consumers in-tree. | `adb_monitor.rs:18`; `lib.rs:18` `pub use crate::relay::byte_buffer`; binary-only modules `main.rs:23-26`. `ByteBuffer` itself is a generic ring buffer (`byte_buffer.rs:20-37`) with no packet/relay semantics — coupling is accidental packaging, not domain necessity. |
| 2 | **Confirmed: sidecar is the low-risk MVP.** `pub fn relay(port)` runs an **infinite** mio poll loop with **no** stop/cancel API. CLI already treats relay as a long-lived blocking call (`cmd_relay`). In-process link would share fate with the GUI and leave no clean shutdown short of killing a thread (unsafe with mio/`Rc`). | `lib.rs:23-25`; `relay.rs:57-77` `loop { … selector.poll … }`; `main.rs:482-485`; TECH_DECISIONS TD-02 Option A. |
| 3 | **Confirmed naming preference aligns with crate shape.** Upstream package is `gnirehtet` with **lib name** `relaylib` (`Cargo.toml` `[lib] name = "relaylib"`). A successor split should name the event-loop crate after what it is — a **relay** — not a vague “core”. MIGRATION_PLAN Stage M2 already lists `gnirehtet-relay`, `gnirehtet-adb`, `gnirehtet-cli`. | `Cargo.toml:7-10`; `MIGRATION_PLAN.md` Stage M2. |

**Disposition of the three points in this analysis:** adopt all three as reuse boundaries (see §2 and §5).

---

## 1. What the GUI successor can keep (ship / vendor)

| Asset | Disposition | Why | Evidence |
|-------|-------------|-----|----------|
| `app/` APK (`com.genymobile.gnirehtet`) | **Reuse** (ship; fork only for Android policy) | Intent-controllable VpnService; reconnect already implemented; Apache-2.0 | Manifest START/STOP; `GnirehtetActivity`; TD-01 |
| Wire protocol (4-byte BE id + raw IPv4 stream) | **Reuse unchanged** | Device + both relays agree; changing it forks the APK | PROTOCOL.md; `RelayTunnel.readClientId`; `client.rs` `pending_id_bytes` |
| `relay-rust` event loop (`relay/` modules) | **Reuse as binary sidecar** | Hard TCP/UDP userspace NAT; upstream-recommended | README; `tcp_connection.rs`; TD-02 |
| CLI command **semantics** (`run`/`start`/`stop`/`tunnel`/`relay`/`autorun`) | **Wrap** | Known-good adb sequences | `main.rs` `COMMANDS`; Java `Main.Command` |
| `AdbMonitor` algorithm | **Extract & reuse** after decoupling buffer | Correct `track-devices` framing + “new device” diff | `adb_monitor.rs` + unit tests |
| Java relay | **Keep available, not default** | Fallback / readable twin | README Flavors |
| `IPPacketOutputStream` + IPv4 filter | **Reuse inside APK** | Required for TUN correctness | `IPPacketOutputStream.java`; `Forwarder` |
| License headers / Apache-2.0 | **Retain** | Legal | `LICENSE` |

---

## 2. Wrap vs rewrite (decision table)

| Capability | MVP approach | Later | Do **not** rewrite unless… |
|------------|--------------|-------|----------------------------|
| Packet relay (TCP/UDP) | **Sidecar:** spawn `gnirehtet relay -p PORT` (or stock binary `relay` verb) | Optional embed after cooperative `stop` API | Measured TCP defect unfixable; or forced by unportable mio |
| adb reverse / start / stop | **Wrap** CLI semantics in a host orchestrator process/module | Shared `gnirehtet-adb` crate | — |
| Device monitoring | **Wrap** `AdbMonitor` logic (after buffer split) | Same | — |
| Android VPN client | **Reuse APK** | Small fork (FGS types, `onRevoke`) | VpnService abandoned by platform |
| Desktop UI | **New** | — | N/A (does not exist upstream) |
| IPv6 | **Out of scope** (see IPV6_ANALYSIS.md) | Separate ADR | Product requires it |

### MVP sidecar shape (Architect #2)

```text
GUI process
  ├─ host-orchestrator (new): builds argv / watches exit codes
  │     └─ child: gnirehtet-relay-or-stock  ["relay", "-p", "31416"]
  ├─ optional child/thread: adb monitor (new crate or port of AdbMonitor)
  └─ adb CLI for reverse / am start / install   (same strings as main.rs)
Device: unchanged APK
```

**Why not in-process `relaylib` for MVP:**

1. No shutdown: `relay()` never returns except on fatal I/O (`relay.rs` infinite loop).
2. Ctrl+C path today is **process exit** after `cmd_stop` (`main.rs:370-378`), not graceful relay teardown.
3. Crash isolation: a panic in TCP code (`client.rs` `panic!("Unexpected unhandled error")`) must not kill the GUI.
4. Matches TECH_DECISIONS TD-02 recommendation A.

In-process remains a **Stage M3+** option once a stop/health API exists.

---

## 3. Coupling hotspots (boundaries to cut)

### H1 — `adb_monitor` → `relaylib::byte_buffer` (Architect #1)

1. **File/module:** `relay-rust/src/adb_monitor.rs` ↔ `relay-rust/src/lib.rs` / `relay/byte_buffer.rs`
2. **What it does:** Uses `ByteBuffer` to reassemble adbd length-prefixed frames.
3. **Why it matters:** Pulling “adb monitor” into a GUI crate today would either (a) depend on the entire relay library or (b) require forking the buffer.
4. **Whether reusable:** Logic yes; packaging **no**.
5. **Refactor required:** Move `ByteBuffer` to a tiny shared util crate **or** duplicate/inline a 1 KiB buffer inside `gnirehtet-adb`. Stop re-exporting `byte_buffer` from the relay lib unless the relay itself needs it public (today only adb_monitor uses the re-export).
6. **Risk:** Low technical risk; medium if ignored (forces GUI→relaylib dependency).
7. **Evidence:** `adb_monitor.rs:18`; `lib.rs:17-18`; no other `relaylib::byte_buffer` references in binary besides adb_monitor (verified by source layout: only `adb_monitor.rs` imports it).

### H2 — CLI + relay in one Cargo package

1. **File/module:** `Cargo.toml` package `gnirehtet` with both `src/main.rs` and `src/lib.rs`
2. **What it does:** One package builds CLI binary + `relaylib`.
3. **Why it matters:** GUI vendors the whole package even when it only wants the relay binary or only adb helpers.
4. **Reuse:** Sidecar can ignore this; library consumers cannot.
5. **Refactor required:** Split into `gnirehtet-relay` (lib+relay binary entry) and `gnirehtet-cli` / `gnirehtet-adb` (Architect #3 / MIGRATION_PLAN M2). Prefer **`gnirehtet-relay`**, not `gnirehtet-core` — “core” invites dumping orchestration, UI types, and protocol helpers into one ball of mud; the lib today is literally a relay (`pub fn relay`).
6. **Risk:** Low if rename is packaging-only; high if “core” becomes a junk drawer.
7. **Evidence:** `Cargo.toml` name `gnirehtet`, lib name `relaylib`; `lib.rs` only exposes `relay` + `byte_buffer`.

### H3 — Hardcoded abstract socket name + port defaults

1. **File/module:** `RelayTunnel.LOCAL_ABSTRACT_NAME = "gnirehtet"`; CLI `DEFAULT_PORT = 31416`; `cmd_tunnel` strings
2. **What it does:** Couples APK, adb reverse, and host listen address by convention.
3. **Why it matters:** GUI must not “innovate” on the name without shipping a matching APK.
4. **Reuse:** Treat as **protocol constants**; single shared constants module in orchestrator + document in PROTOCOL.md.
5. **Refactor required:** None for MVP; optional central constants crate later.
6. **Risk:** Medium if GUI and APK diverge.
7. **Evidence:** `RelayTunnel.java:32`; `cli_args.rs:23`; `main.rs:471-478`.

### H4 — Dual Java/Rust implementations

1. **File/module:** `relay-java/` vs `relay-rust/`
2. **What it does:** Parallel CLI + relay.
3. **Why it matters:** Drift risk; GUI should pick **one** primary (Rust).
4. **Reuse:** Rust primary; Java as oracle/tests/fallback.
5. **Refactor required:** Do not port GUI to Java relay.
6. **Risk:** Low if Java is not default.
7. **Evidence:** README “Use the Rust implementation”; mirrored `Main.java` / `main.rs`.

### H5 — Android reconnect vs host reverse ownership

1. **File/module:** `PersistentRelayTunnel` / `RelayTunnelProvider` vs host `cmd_tunnel`
2. **What it does:** Device retries connect forever; host must re-apply `adb reverse` after USB reattach.
3. **Why it matters:** GUI owns the control plane the APK cannot see (adb).
4. **Reuse:** Keep device reconnect; **wrap** host reverse on device-online events (FAILURE_MODES / RECONNECT_DESIGN).
5. **Refactor required:** Orchestrator state machine (device online → tunnel → start); not an APK rewrite.
6. **Risk:** High UX failure if GUI only starts relay and assumes reverse persists.
7. **Evidence:** `RelayTunnelProvider` 5s backoff; `cmd_start` always calls `cmd_tunnel`; FAILURE_MODES USB row.

### H6 — APK version gate hardcoded to `"9"`

1. **File/module:** `main.rs` / `Main.java` `REQUIRED_APK_VERSION_CODE = "9"`; `app/build.gradle` `versionCode 9`
2. **What it does:** Auto-reinstall if dumpsys versionCode differs.
3. **Why it matters:** GUI forks that bump versionCode must update the gate or disable it.
4. **Reuse:** Keep the idea; make the expected code a build-time constant shared with the APK.
5. **Refactor required:** Single source of truth in successor build.
6. **Risk:** Medium — silent reinstall loops or skipped upgrades.
7. **Evidence:** `main.rs:37`, `must_install_client`; `app/build.gradle` `versionCode 9`.

---

## 4. Reuse boundaries (recommended crate/process cut)

```text
┌─────────────────────────────────────────────────────────┐
│ Desktop GUI (new)                                       │
│  - never imports mio / tcp_connection                   │
│  - talks to orchestrator API only                       │
└───────────────────────────┬─────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────┐
│ gnirehtet-host / orchestrator (new, wrap CLI semantics) │
│  - install/start/stop/tunnel argv builders              │
│  - child process supervisor for relay sidecar           │
│  - device session state                                 │
└─────────────┬───────────────────────────┬───────────────┘
              │                           │
              │ spawn                     │ library
              ▼                           ▼
┌──────────────────────────┐   ┌──────────────────────────┐
│ gnirehtet-relay (sidecar │   │ gnirehtet-adb (extract)  │
│  binary + optional lib)  │   │  AdbMonitor - NO relay   │
│  = today's relay/ +      │   │  dependency; own buffer  │
│    lib.rs relay(port)    │   │  honor ADB env           │
│  name NOT gnirehtet-core │   └──────────────────────────┘
└──────────────────────────┘
              ▲
              │ TCP protocol unchanged
┌─────────────┴──────────────┐
│ Android APK (reuse app/)   │
└────────────────────────────┘
```

**Apache-2.0:** vendoring with attribution is fine; keep NOTICE/headers (`LICENSE`).

---

## 5. Finding-format entries (reuse-focused)

### F-REUSE-1 — Ship APK; do not rewrite VpnService

1. **File/module:** `app/src/main/java/com/genymobile/gnirehtet/*`
2. **What it does:** Full reverse-tether client.
3. **Why it matters:** Correctness surface (TUN, reconnect, packet split) is already paid for.
4. **Whether reusable:** **Yes** as artifact + optional maintenance fork.
5. **Refactor required:** Manifest/FGS/`onRevoke` when targeting modern SDKs (debt doc).
6. **Risk:** Medium (platform policy), low (protocol).
7. **Evidence:** 15 production classes; `Forwarder`+`PersistentRelayTunnel`; TD-01.

### F-REUSE-2 — Sidecar relay binary (Architect #2)

1. **File/module:** `relay-rust` `lib.rs` / `relay.rs` / CLI `cmd_relay`
2. **What it does:** Blocking `relay(port)`.
3. **Why it matters:** Clean process boundary for GUI.
4. **Whether reusable:** **Yes** as child process; **not** as MVP in-process lib.
5. **Refactor required:** Process supervisor (restart, port conflict, logs). Later: stop API if embedding.
6. **Risk:** Low for sidecar; high for premature embed.
7. **Evidence:** infinite `poll_loop`; no cancel token; TD-02.

### F-REUSE-3 — Break adb_monitor ↔ relaylib (Architect #1)

1. **File/module:** `adb_monitor.rs`, `byte_buffer.rs`, `lib.rs`
2. **What it does:** Accidental public re-export for a binary module.
3. **Why it matters:** Defines whether GUI can take AdbMonitor without linking the relay.
4. **Whether reusable:** After split, **yes**.
5. **Refactor required:** Remove `pub use byte_buffer` from relay lib (or move buffer to util); AdbMonitor self-contained.
6. **Risk:** Low.
7. **Evidence:** §0 table; `ByteBuffer` API is Read-into-slice only.

### F-REUSE-4 — Name the relay crate `gnirehtet-relay` (Architect #3)

1. **File/module:** successor packaging (maps to today’s `relaylib`)
2. **What it does:** Holds selector/client/router/tcp/udp.
3. **Why it matters:** Naming drives dependency hygiene.
4. **Whether reusable:** N/A (naming).
5. **Refactor required:** When splitting crates, use `gnirehtet-relay` (+ `gnirehtet-adb`, `gnirehtet-cli`). Avoid `gnirehtet-core`.
6. **Risk:** Low; bike-shedding only if delayed.
7. **Evidence:** `lib.rs` surface is `relay()`; MIGRATION_PLAN M2 already proposes `gnirehtet-relay`.

### F-REUSE-5 — Wrap CLI; do not re-string adb from UI widgets

1. **File/module:** `main.rs` `cmd_tunnel` / `cmd_start` / `cmd_stop`
2. **What it does:** Canonical adb argv.
3. **Why it matters:** Reverse + intent extras are easy to get subtly wrong (`--esa`, component name, abstract socket).
4. **Whether reusable:** **Semantics yes**; argv builder should be one module.
5. **Refactor required:** Orchestrator API mirroring verbs; UI calls `start(serial, dns, routes, port)`.
6. **Risk:** Medium if UI invents adb commands.
7. **Evidence:** `main.rs:471-478` reverse; `:421-436` am start; Java twin `Main.java:305-307`.

### F-REUSE-6 — Do not reuse root Gradle as the desktop build

1. **File/module:** root `build.gradle` / `settings.gradle`
2. **What it does:** Ties app + both relays to AGP 3.5 / Gradle 5.4.1 / jcenter.
3. **Why it matters:** Desktop GUI should build APK in CI optionally, not as a hard dep of the UI crate.
4. **Whether reusable:** Android subproject only.
5. **Refactor required:** Separate desktop build; invoke `cargo` / prebuilt APK.
6. **Risk:** High build break if GUI repo requires full Android Studio toolchain for every change.
7. **Evidence:** `settings.gradle` `include ':app', ':relay-java', ':relay-rust'`; AGP 3.5.0; MIGRATION_PLAN inventory row “Root Gradle multi-project — Do not require”.

### F-REUSE-7 — Java relay: reference only

1. **File/module:** `relay-java/`
2. **What it does:** Parity implementation.
3. **Why it matters:** Useful when reading TCPConnection without Rust lifetime noise.
4. **Whether reusable:** Fallback binary; not GUI default.
5. **Refactor required:** None.
6. **Risk:** Confusion if both ship in UI “advanced” settings without clear preference.
7. **Evidence:** README Flavors.

---

## 6. One-line GUI context (only)

A successor desktop GUI should **orchestrate** existing adb+APK+Rust-relay pieces; it should not reimplement the VPN client or the userspace TCP stack for MVP.

---

*End of REUSE_ANALYSIS.md*
