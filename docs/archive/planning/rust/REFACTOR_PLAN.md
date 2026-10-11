# Gnirehtet Rust — Phased Minimal-Risk Refactor Plan

**Upstream pin:** Genymobile/gnirehtet `@1eb2e58` / v2.5.1  
**Evidence:** `relay-rust/` as analyzed.  
**Lead constraints:** MVP = **relay as managed sidecar/child process**; extract/refactor over rewrite; mio modernization = **Phase 3**; orchestrator mirrors CLI verbs `run` / `start` / `stop` / `tunnel` / `relay`.

Legend: **FACT** = current code behavior; **PROPOSAL** = planned change.  
**Wire protocol:** any step that would change client↔relay framing is marked **PROTOCOL — DO NOT**.

---

## Guiding rules

1. Prefer **move files / expose APIs** over rewriting `tcp_connection.rs` / `selector.rs`.
2. Ship a working CLI after every phase (behavior parity with upstream commands).
3. GUI MVP talks to **controller + child relay process**, not in-process `relaylib`.
4. Do not bump mio / edition as a drive-by (Phase 3 only, unless build/security breaker).
5. Keep Android app and wire protocol fixed (`client.rs` id handshake, `Ipv4PacketBuffer` framing, `cmd_tunnel` abstract name).

---

## Phase 0 — Inventory freeze & characterization (no behavior change)

**Goal:** Safety net before moves.

| Step | Action | Risk |
|------|--------|------|
| 0.1 | Document pin `1eb2e58` / v2.5.1 in repo notes | None |
| 0.2 | Run existing unit tests: `cli_args`, `adb_monitor`, `byte_buffer`, header tests under `relay/` | Low |
| 0.3 | Manual smoke: `relay`, `tunnel`, `start`/`stop`, `run` on one device | Low |
| 0.4 | List public symbols today (`lib.rs` only exports `relay` + `byte_buffer`) | None |

**Do not change:** any `relay/*` packet logic.

---

## Phase 1 — Extract ADB + CLI seams (still one package OK)

**Goal:** Reusable ADB API without touching mio path. Relay remains startable as today (`gnirehtet relay` / `relaylib::relay`).

### 1.A Move ADB out of `main.rs` (**PROPOSAL**)

**FACT sources to relocate:**

- `src/adb_monitor.rs`
- `src/execution_error.rs` (or rename into adb errors)
- Functions in `main.rs`: `get_adb_path`, `get_apk_path`, `create_adb_args`, `exec_adb`, `must_install_client`, `cmd_install`, `cmd_uninstall`, `cmd_reinstall`, `cmd_start`, `cmd_stop`, `cmd_tunnel`, `async_start` (thread wrapper)

**PROPOSAL structure (in-tree first):**

```text
src/adb/mod.rs          # AdbClient, AdbConfig, VpnOptions
src/adb/monitor.rs      # from adb_monitor.rs
src/adb/error.rs        # from execution_error.rs
src/adb/buffer.rs       # ByteBuffer copy — BREAK dep on relaylib::byte_buffer
```

| Step | Detail | Risk |
|------|--------|------|
| 1.1 | Copy/move `ByteBuffer` into adb module; point monitor at it | Low — **breaks false coupling** |
| 1.2 | `AdbClient` methods wrapping current `cmd_*` bodies verbatim | Low if argv identical |
| 1.3 | `main.rs` Command impls call `AdbClient` | Low |
| 1.4 | Keep intent strings / package name / versionCode `"9"` identical | **PROTOCOL-adjacent** (Android) — do not “clean up” |

**What NOT to change early:** `REQUIRED_APK_VERSION_CODE`, `com.genymobile.gnirehtet` component names, `dumpsys` parsing heuristics.

### 1.B Thin CLI over AdbClient + relay (**PROPOSAL**)

| Step | Detail | Risk |
|------|--------|------|
| 1.5 | `cmd_relay` still calls `relaylib::relay(port)` in-process **inside the CLI binary** (fact parity). For **GUI**, do not link this path. | Low |
| 1.6 | Introduce `RelayChild::spawn(relay_argv)` used by a new `RunSession` — CLI `run` can switch to spawn-self or spawn same binary with `relay` subcommand | Medium — must preserve env, cwd, port |

**MVP CLI `run` strategy (proposal):**  
Prefer `Command::new(current_exe).args(["relay", "-p", …])` child + adb start/stop — matches GUI model and avoids `exit(0)` killing a future embedder. Keep a feature/`GNIREHTET_RUN_INPROCESS=1` escape hatch during transition if needed.

**Risk note:** Today `cmd_run` Ctrl+C → `exit(0)` (**FACT** `main.rs`). Child-based run fixes that for GUI; CLI should adopt the same.

---

## Phase 2 — Controller library + sidecar-first orchestration

**Goal:** `gnirehtet-controller` (module or crate) mirrors verbs; GUI can depend on it.

| Verb | Parity requirement |
|------|-------------------|
| `relay` | Start relay **process**; wait/kill |
| `tunnel` | Same `adb reverse localabstract:gnirehtet tcp:{port}` |
| `start` / `stop` | Same install-if-needed + intents |
| `run` | Relay child + start client; stop tears down both |
| `autorun` | Relay child + `AdbMonitor` thread |

| Step | Detail | Risk |
|------|--------|------|
| 2.1 | `RelayProcess` wrap `std::process::Child`; pipe stdout/stderr | Low |
| 2.2 | `RunSession` / `AutorunSession` | Medium (lifecycle races) |
| 2.3 | Stoppable `AdbMonitor` (`AtomicBool` or channel checked between packets / on reconnect) | Medium — **FACT** loop is infinite with no cancel |
| 2.4 | Split Cargo crates if APIs stable (`CRATE_STRUCTURE.md`) | Low–medium (path/import churn) |
| 2.5 | Desktop MVP: call controller only | Low if Phase 2.1–2.2 done |

**What NOT to change:**

- `relay/selector.rs`, `tcp_connection.rs`, `udp_connection.rs`, header lifetime design (**FACT** `*Header<'a>` / `*HeaderData` split in DEVELOP.md).
- Listen address `127.0.0.1` (**FACT** `tunnel_server.rs`) — changing to `0.0.0.0` is a security/behavior change, not MVP.
- In-process embed APIs — **defer**.

**PROTOCOL — DO NOT:** alter client id bytes, packet stream format, abstract socket name, default port meaning.

---

## Phase 3 — Hardening & optional modernization

Only after CLI+GUI sidecar flows are stable.

| Step | Detail | When |
|------|--------|------|
| 3.1 | Replace handler `panic!` on unexpected IO with close+log (`client.rs`, `tcp_connection.rs`, `udp_connection.rs`) | After sidecar MVP — panics kill only the child |
| 3.2 | Edition 2021 bump | When deps allow; separate PR |
| 3.3 | mio 0.6 → 0.8/1.x | **High risk** — API (`Ready`, `PollOpt`, `Evented`) pervasive; isolate in `selector.rs` first |
| 3.4 | Optional `embed-relay` feature: dedicated thread + wake fd for in-process stop | Post-MVP product need |
| 3.5 | Tokio for ADB wrappers only (never packet path unless full rewrite accepted) | Optional |

**Breaker clause:** If mio 0.6 fails to build on a required target/toolchain with no patch, Phase 3.3 may pull earlier — still isolate to selector/register call sites; do **not** rewrite TCP state machine “while there”.

---

## Phase map vs crates

```
Phase 0  character single package
Phase 1  modules adb:: + cli uses AdbClient; relay code untouched
Phase 2  controller + sidecar sessions; optional workspace split
Phase 3  mio/edition/embed extras
```

---

## Risk register

| Risk | Severity | Mitigation |
|------|----------|------------|
| `run` via child breaks relative `gnirehtet.apk` path | Medium | Resolve APK to absolute path before spawn; document cwd |
| Double relay bind on port | Medium | Controller single-owner; detect `AddrInUse` from child stderr/exit |
| AdbMonitor thread leak on GUI close | Medium | Phase 2.3 stop flag; join with timeout |
| Splitting crates breaks gradle `cargo` wrappers (`build.gradle`) | Medium | Update gradle tasks in same PR as workspace split |
| Accidental protocol tweak while “cleaning” client id send | High | Code owners review on `client.rs` / `ipv4_packet_buffer.rs` |
| In-process embed attempted too early (`!Send` Rc/RefCell) | High | MVP policy: **sidecar only** |

---

## What NOT to change early (checklist)

- [ ] `relay/tcp_connection.rs` state machine
- [ ] `relay/selector.rs` handler model
- [ ] `Client` pending id / `PacketSource` backpressure
- [ ] UDP idle expiry constants (unless bugfix)
- [ ] mio version
- [ ] Wire protocol & `adb reverse` shape
- [ ] Replacing `log` + `SimpleLogger` with tracing (cosmetic)

---

## Suggested PR sequence

1. **PR-A:** `adb` module + local `ByteBuffer`; `main` delegates; tests green.  
2. **PR-B:** `RelayProcess` + CLI `run`/`autorun` via child; parity smoke.  
3. **PR-C:** `controller` API; CLI thin; document verbs.  
4. **PR-D:** GUI MVP against controller.  
5. **PR-E:** workspace crate split (if not done in C).  
6. **PR-F (Phase 3):** panic hygiene / mio — separate.

---

## Success criteria per phase

| Phase | Done when |
|-------|-----------|
| 0 | Smoke notes recorded against pin |
| 1 | GUI could call `AdbClient` without parsing argv; monitor does not import relay |
| 2 | GUI starts/stops reverse tether via controller without linking mio; verbs match CLI |
| 3 | Optional; no regression on packet path benchmarks / manual transfer test |

---

## Reconciliation note (Codebase Researcher, 2026-09-12)

Independent verification in `/workspace/gnirehtet-research/REUSE_ANALYSIS.md` §0 **confirmed** all three Rust Architect boundaries. Extra actionable finding folded here:

- **FACT:** `AdbMonitor::start_adb_daemon` hardcodes `"adb"` while CLI uses `get_adb_path()` / `ADB` env (`adb_monitor.rs` vs `main.rs`). **PROPOSAL:** unify on `AdbConfig::adb_path` in Phase 1 extract.
- ByteBuffer disposition: prefer **inline/copy into `gnirehtet-adb`** over premature `gnirehtet-util` (agrees with Researcher H1 option B and Phase 1.1).
- Android AGP/jcenter is **P0 for APK rebuilds** (Researcher TECHNICAL_DEBT) — not a Rust packet-path change; track with Android/Desktop, not Phase 1 mio work.
