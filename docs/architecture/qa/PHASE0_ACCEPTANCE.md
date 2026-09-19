# PHASE0_ACCEPTANCE.md

| Field | Value |
|-------|-------|
| Status | Draft v0.1 |
| Date | 2026-09-12 |
| Project | gnirehtet desktop GUI |
| Scope | First runnable Desktop slice (Tauri scaffold + sidecar + log stream + APK resource) |
| Branch | `dev` (Lead Architect commits as sawongam — do not commit from this doc alone) |
| Related | [TEST_STRATEGY.md](./TEST_STRATEGY.md), [TEST_MATRIX.md](./TEST_MATRIX.md), [FAILURE_SCENARIOS.md](./FAILURE_SCENARIOS.md), [ERROR_CODE_MAP.md](./ERROR_CODE_MAP.md), [EVENT_STATUS_MAP.md](../ux/EVENT_STATUS_MAP.md), [ERROR_UX.md](../ux/ERROR_UX.md), [TEAM_BRIEF.md](../../TEAM_BRIEF.md), [ROADMAP.md](../ROADMAP.md) |

Concrete checklist Desktop / Rust must pass for the **first runnable slice**. No fabricated run results. Sharing remains three-layer healthy only (Relay Listening + Tunnel OK + Device VPN handshake) — see EVENT_STATUS_MAP.

**Phase 0 Desktop (this gate):** Tauri scaffold, sidecar spawn/stop, log stream, APK resource.  
**Phase 1 Rust (adjacent, not blocking every Phase 0 row):** AdbClient extract, break `byte_buffer` coupling, CLI parity; relay stays sidecar.  
**P0 SLAs (already binding where in scope):** relay death UI ≤3s; quit clears port **31416** (or configured session port); **no Sharing on intent-sent**.

---

## 1. Scaffold smoke (app launches, versions)

| ID | Check | Pass | Fail |
|----|-------|------|------|
| P0-S1 | App launches on lab host (Linux primary; Windows when artifact exists) | Window opens; no fatal panic on cold start | Crash / blank WebView without actionable message |
| P0-S2 | Version / build identity visible or queryable | App (or `--version` / about) exposes GUI build id; diagnostics or logs can show relay + APK path when resolved | Unknown binary with no version string for triage |
| P0-S3 | Tauri 2 + Svelte shell loads | Main UI shell renders (Idle-capable surface per EVENT_STATUS_MAP) | Frontend fails to mount |
| P0-S4 | Upstream pin recorded | Docs / bundle metadata still pin Genymobile/gnirehtet lineage used for sidecar + APK (see TEST_STRATEGY / RELEASE_PLAN) | Unpinned or mismatched undocumented binary |

**Out of this row set:** full USB E2E traffic proof (see §5).

---

## 2. Sidecar spawn/stop + log stream

| ID | Check | Pass | Fail | Lab (`29dc0fa` / PHASE0_LAB_NOTES) |
|----|-------|------|------|-------------------------------------|
| P0-R1 | Spawn relay sidecar (`gnirehtet relay` or equivalent managed child) | Child starts; orchestrator records PID/port; `RelayState` can become listening/ready when bind succeeds | UI claims relay up with no process | **PASS** — owns :31416, PID recorded |
| P0-R2 | Stop owned sidecar | `stop_relay` / Stop path terminates **our** child; port not left listening by our PID | Orphan child after Stop | **PASS** — stop ok, port free |
| P0-R3 | Log stream | Sidecar stdout/stderr line-buffered → `LogLine` events → UI log strip appends | Logs only in terminal; UI never sees lines; or log alone flips Sharing | **Partial** — pipes drained / pump wired; WebView strip not labbed |
| P0-R4 | Foreign process policy | Process we did not start on :31416 is **not** killed; surface `PORT_IN_USE` / foreign relay per DESKTOP_LIFECYCLE | Quiet kill of foreign `gnirehtet` | **PASS** — foreign bind → `PORT_IN_USE`, `owns_relay=false`, foreign not killed (both start paths) |
| P0-R5 | Relay death UI SLA | If our owned relay exits/crashes after ownership, UI reflects Error / Relay Error with `RELAY_CRASHED` (ERROR_UX) within **≤3s** | Stale healthy/Sharing >3s | **PASS** (headless) — `poll_owned_relay` ≤3s after SIGKILL; UI event path code-pass |

Cross-ref: FAILURE_SCENARIOS F-R1/F-R4; TEST_MATRIX M-18/M-20. Evidence: [PHASE0_LAB_NOTES.md](../desktop/PHASE0_LAB_NOTES.md).

---

## 3. Port 31416 ownership / `PORT_IN_USE` before claiming relay

| ID | Check | Pass | Fail |
|----|-------|------|------|
| P0-P1 | Default port | Session/default listen port is **31416** unless configured | Hardcoded conflict with documented default undocumented |
| P0-P2 | Bind failure before ownership | Occupied port → `Error` `PORT_IN_USE` only; do **not** set Relay Listening / ownedBySession | Chip or layer shows Relay healthy on bind fail |
| P0-P3 | Ownership gate | Relay layer “Listening” only after **we** successfully own the listener | Attaches to wrong process; pretend success |
| P0-P4 | Diagnostics | On port failure, logs include `stage=start_relay`, `port`, `error_code=PORT_IN_USE` (TEST_STRATEGY §8 fields) | Missing required log fields |

Cross-ref: EVENT_STATUS_MAP §2 / §4; ERROR_UX `PORT_IN_USE`; FAILURE_SCENARIOS F-R2; TEST_MATRIX M-17. Lab: P0-P2 covered by P0-R4 **PASS** (`29dc0fa`).

---

## 4. Quit while running → no orphan

| ID | Check | Pass | Fail | Lab (`29dc0fa` / PHASE0_LAB_NOTES) |
|----|-------|------|------|-------------------------------------|
| P0-Q1 | Quit with owned relay running | App quit hook tears down owned sidecar; configured port (**31416** default) is **free** before/as process exit completes | Listener remains; zombie child | **PASS** — `clear_owned_relay` → child gone, port free; poll `None` |
| P0-Q2 | Quit mid-start | Partial start does not leave orphan relay we spawned | Orphan after cancel/quit | **TBD** |
| P0-Q3 | adb server | Do **not** kill shared user `adb` server on quit by default (ERROR_UX / DESKTOP_LIFECYCLE) | Aggressive adb server kill as default | **PASS*** — adb PIDs unchanged; *no adb server present in this lab (`adb_note=no_adb_server_present_lab_ok`) |
| P0-Q4 | Must-log | `stage=stop\|shutdown`, `port`, success flags present | Silent teardown with no trail | Code-pass (orchestrator); full quit-hook log trail not separately labbed |

Cross-ref: EVENT_STATUS_MAP §4 “Quit while owning relay”; FAILURE_SCENARIOS F-R4; TEST_MATRIX M-23; P0 SLA. Evidence: [PHASE0_LAB_NOTES.md](../desktop/PHASE0_LAB_NOTES.md).

---

## 5. What is NOT required yet

Phase 0 acceptance **does not** require:

| Deferred | Notes |
|----------|-------|
| Full USB E2E tether (install → tunnel → START → traffic → STOP) | Lab P0 for later MVP gate (TEST_MATRIX M-01/M-02); ROADMAP Spike B may exist as spike, not this acceptance floor |
| Packaging GA (signed installers, notarization, full artifact set) | RELEASE_PLAN; packaging smoke can be separate |
| Three-layer Sharing proof on device | Needs Tunnel OK + Device VPN Active handshake; Phase 0 may only prove Idle / Starting / Error + **Relay** layer |
| Waiting for VPN / Interrupted chip polish | EVENT_STATUS_MAP chips exist; full VPN consent lab is post–Phase-0 slice |
| Multi-device simultaneous tether | Out of MVP |
| Wireless adb as primary | Out of MVP |
| IPv6 | Out of MVP |
| In-process `relaylib` / relay rewrite | Relay stays sidecar |
| Phase 1 Rust complete (AdbClient extract, `byte_buffer` decoupling, full CLI parity) | Tracked on Rust refactor plan; not every Phase 0 Desktop row blocked on it |
| macOS first-class / notarization | Best-effort per TEAM_BRIEF |
| Tray-first UX | Later per EVENT_STATUS_MAP |

---

## 6. Pass/fail against EVENT_STATUS_MAP chips

Source of truth: [EVENT_STATUS_MAP.md](../ux/EVENT_STATUS_MAP.md). **Never** show Sharing / Connected on intent-sent or relay-only.

| Chip | Phase 0 must prove? | Pass criteria (Phase 0) | Fail |
|------|---------------------|-------------------------|------|
| **Idle** | **Yes** | No owned session; relay not ours or stopped cleanly after Stop/Quit | Stuck Starting/Error with no recovery; Idle while our orphan holds port |
| **Starting** | **Yes** (if Start/run entry exists) | Shown while spawn/pipeline in progress; does not imply Sharing | Sharing shown during Starting |
| **Waiting for VPN** | No (optional if START path wired) | If shown: maps to `VPN_PERMISSION_PENDING`; not Sharing | Sharing while waiting for consent |
| **Sharing** | **Not claimable in Phase 0 without three layers** | If UI can show Sharing at all: only when Relay Listening **and** Tunnel OK **and** Device VPN Active (handshake). Phase 0 may **omit** Sharing entirely and only prove Idle/Starting/Error + relay layer | Sharing on relay-only, intent-sent, or log scrape alone |
| **Interrupted** | No | Defer full `TUNNEL_LOST` lab to later P0 matrix | N/A for Phase 0 floor |
| **Stopping** | Best-effort | Stop/quit path reaches Idle without orphan | Hang in Stopping |
| **Error** | **Yes** | Blocking errors use ERROR_UX codes (`PORT_IN_USE`, `RELAY_CRASHED`, …); chip title from ERROR_UX | Vague “failed”; fake green |

### Layer strip (Phase 0 focus)

| Layer | Phase 0 | Pass |
|-------|---------|------|
| Relay | **In scope** | Listening only when owned bind OK; Off when stopped; Error on crash/`PORT_IN_USE` before ownership |
| Tunnel | Not required for Phase 0 floor | Do not mark Tunnel OK without `adb reverse` success (when later wired) |
| Device VPN | Not required for Phase 0 floor | Do not mark Active on START intent alone |

---

## 7. Report format reminder (implementation tasks)

Per TEAM_BRIEF — every implementation task report must include:

1. **What** was changed  
2. **Why** it was changed  
3. **Files/modules** affected  
4. **How** it works  
5. **Tests** performed  
6. **Known limitations**  
7. **Follow-up** issues, if any  

QA sign-off notes for this checklist should mirror the same honesty: expectations only until lab evidence exists; cite ERROR_CODE_MAP / EVENT_STATUS_MAP; no invented upstream behavior.

---

## 8. Sign-off stub (no fabricated results)

| Gate | Owner | Result | Date | Evidence |
|------|-------|--------|------|----------|
| Scaffold smoke (§1) | Desktop | _TBD_ (no display lab) | 2026-09-19 | PHASE0_LAB_NOTES — P0-S1 not run |
| Sidecar + logs (§2) | Desktop + QA | **Near Pass** | 2026-09-19 | R1/R2/R4/R5 **PASS**; R3 **Partial** (WebView strip) |
| Port ownership (§3) | Desktop + QA | **PASS** (lab) | 2026-09-19 | Default :31416 + R4 foreign `PORT_IN_USE` before ownership |
| Quit / no orphan (§4) | Desktop + QA | **PASS** (P0-Q1 lab); Q2 TBD | 2026-09-19 | `clear_owned_relay` → child gone, port free (`29dc0fa`) |
| Chip honesty (§6) | Desktop + QA | **Pass** (no Sharing claimed) | 2026-09-19 | Relay-only; Sharing N/A |

**QA lab re-score (`29dc0fa`):** P0-R1/R2/R4/R5/Q1 **PASS**; P0-R3 **Partial**; no Sharing.

**Exit:** All Yes-required rows Pass or Waived-with-ticket; Sharing still not claimable without three layers. Remaining Phase 0 soft gaps: R3 WebView strip, Q2 mid-start quit, S1 GUI launch.
