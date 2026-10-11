# TEST_STRATEGY.md

| Field | Value |
|-------|-------|
| Status | Draft v0.1 |
| Date | 2026-09-12 |
| Project | gnirehtet desktop GUI |
| Upstream | Genymobile/gnirehtet @ 1eb2e58 / v2.5.1 (Apache-2.0) |
| Related | [TEST_MATRIX.md](./TEST_MATRIX.md), [FAILURE_SCENARIOS.md](./FAILURE_SCENARIOS.md), [RELEASE_PLAN.md](./RELEASE_PLAN.md), [CROSS_PLATFORM.md](./CROSS_PLATFORM.md) |

---

## 1. Purpose

Define how we validate a Tauri 2 + Svelte/TS + Rust host-orchestrator desktop GUI that reuses the upstream Android VpnService APK and Rust relay as a managed sidecar. ADB remains an external, user-provided dependency (PATH or configured). Goal: ship a trustworthy MVP for **one active tether session** on Linux and Windows (macOS best-effort) with measurable recovery, diagnostics, and clear failure UX.

**Out of MVP UI (test as negative / post-MVP expansion):** simultaneous multi-device tether; wireless-adb as primary path.

**In MVP test scope:** multiple devices *present* and requiring explicit selection; USB primary.

---

## 2. Principles

1. **Reuse, don’t retest the world.** Treat upstream APK + relay as known-good at v2.5.1; focus QA on orchestration, lifecycle, packaging, and UX around them. Spot-check APK/relay integration, not rewrite assumptions.
2. **Risk-based.** Prioritize USB happy path, port conflict, relay death, quit-while-running, APK mismatch, multi-device selection (named MVP acceptance themes).
3. **Observable recovery.** Every failure mode has detect → symptom → recovery SLA → must-log fields → pass/fail (see FAILURE_SCENARIOS.md).
4. **No fake green.** Docs describe expectations only; no invented run results.
5. **IPv4 only, no root.** VPN grant is on-device user action; tests must never assume silent VPN enable.
6. **Diagnostics first.** If a bug can’t be triaged from the diagnostics pack, the pack is incomplete.
7. **CI vs lab split.** Deterministic unit/integration in CI; device/USB/OEM/soak in lab.

---

## 3. Test levels

| Level | What | Where | Owner |
|-------|------|-------|-------|
| **Unit** | Orchestrator pure logic: port selection, state machine transitions, intent URI builders, error taxonomy mapping, APK version compare | CI | Eng |
| **Integration** | Host ↔ sidecar relay IPC; `adb` CLI wrappers (`list_devices`, `ensure_adb`, install/start/stop intents, `adb reverse`); mock adb fixture | CI (+ optional lab) | Eng + QA |
| **E2E** | Full USB tether: start relay → reverse → START intent → traffic → STOP → teardown | Lab | QA |
| **Manual / exploratory** | OEM UX quirks, VPN dialog wording, tray/quit paths, accessibility | Lab | QA |
| **Compat** | Android 10–16 (+ API 21 smoke if hardware available), OEM skins, Linux/Windows/macOS packaging | Lab | QA |
| **Soak** | ≥4h continuous tether; reconnect storms; quit/restart cycles | Lab | QA |
| **Security / packaging** | Code signing, quarantine, PATH vs bundled bits, LICENSE/attribution | CI gates + release | Release |

### Conceptual API under test

`list_devices`, `ensure_adb`, `install`, `start`, `stop`, `reset_tunnel`, `start_relay` / `stop_relay`, `subscribe` (events).

Default relay port: **31416**. Device control: `adb reverse localabstract:gnirehtet tcp:<port>`; START/STOP intents on `com.genymobile.gnirehtet/.GnirehtetActivity`. Env: `ADB`, `GNIREHTET_APK`.

---

## 4. Environments

| Env | Contents | Use |
|-----|----------|-----|
| **CI** | Linux runner; mocked adb; unit + integration; lint; license check | PR gate |
| **Lab-Linux** | Ubuntu LTS + udev rules; real USB devices | P0 E2E |
| **Lab-Windows** | Win10/11; Google USB driver / OEM driver matrix | P0 E2E |
| **Lab-macOS** | Best-effort; notarization when signing available | P1/P2 |
| **Device farm** | Pixel / Samsung / Xiaomi / OnePlus; Android 10–16; one API 21+ legacy if available | Compat |
| **Negative fixtures** | Occupied :31416; killed relay; unauthorized adb; missing adb; wrong APK version | Failure scenarios |

Env vars for tests: `ADB` (override binary), `GNIREHTET_APK` (override APK path). Product must honor both.

---

## 5. Entry / exit criteria

### Entry (MVP test start)

- [ ] Build produces installable artifacts for Linux + Windows (macOS optional for MVP gate)
- [ ] Bundled APK matches declared upstream pin (v2.5.1 / 1eb2e58 lineage)
- [ ] Relay sidecar starts and binds configurable port (default 31416)
- [ ] Orchestrator exposes list/ensure/install/start/stop/reset/relay/subscribe
- [ ] Logging emits required fields (see §8)
- [ ] Diagnostics pack exportable from UI or CLI

### Exit (MVP release candidate)

- [ ] All **P0** matrix cells Pass or Waived-with-ticket (TEST_MATRIX.md)
- [ ] All named MVP acceptance themes covered and passing (happy path USB; port conflict; relay killed externally; quit while running; APK version mismatch; multi-device require selection)
- [ ] FAILURE_SCENARIOS.md P0 rows have verified recovery SLAs
- [ ] Soak: ≥4h Linux **or** Windows with no silent session death; UI reflects state within SLA
- [ ] RELEASE_PLAN.md checklist signed
- [ ] Known limitations documented (incl. macOS best-effort, no simultaneous multi-tether)
- [ ] No open Sev-1/Sev-2 without mitigation

### Exit (post-MVP expansions — explicit)

- Simultaneous multi-device tether UI + tests
- Wireless adb as primary supported path
- Full macOS parity (notarization + USB reliability gate)

---

## 6. Risk-based priorities

| Priority | Themes | Rationale |
|----------|--------|-----------|
| **P0** | USB happy path; adb missing/unauthorized; port occupied; relay killed; quit-while-running; VPN grant/revoke; multi-device selection required; APK mismatch; Linux+Windows packaging install | Blocks core value or corrupts trust |
| **P1** | Device disappear/reconnect; adb daemon stop/crash; app restart on device; device reboot; network failure mid-session; Windows USB drivers; Linux udev; diagnostics pack completeness | Common field failures |
| **P2** | API 21 smoke; OEM variants; macOS best-effort; malformed relay traffic; upgrade paths; optional bundled platform-tools policy; soak beyond 4h | Compat / hardening |

---

## 7. MVP vs post-MVP scope

| In MVP | Post-MVP (mark tests deferred) |
|--------|--------------------------------|
| One active tether session | N concurrent tether sessions |
| Multiple devices listed; user must select one | Auto-balance / multi-start |
| USB debugging primary | Wireless adb primary |
| System PATH adb or configured path; APK+relay bundled | Official full SDK vendoring (if ever) |
| Linux + Windows primary | macOS first-class |
| IPv4 only | IPv6 |
| Manual VPN permission | Guided automation where OEM allows |

---

## 8. Logging & diagnostics strategy

### Required log fields (every orchestrator stage)

| Field | Description |
|-------|-------------|
| `timestamp` | ISO-8601 UTC |
| `serial` | adb serial or `none` |
| `stage` | e.g. `ensure_adb`, `install`, `start_relay`, `reverse`, `start_intent`, `stop`, `reset_tunnel` |
| `port` | relay TCP port |
| `adb_exit_code` | integer or `n/a` |
| `error_code` | taxonomy: `ADB_MISSING`, `ADB_UNAUTHORIZED`, `ADB_MULTI`, `PORT_IN_USE`, `RELAY_DEAD`, `APK_MISMATCH`, `VPN_DENIED`, `VPN_REVOKED`, `DEVICE_GONE`, `NETWORK_FAIL`, `MALFORMED`, `SHUTDOWN`, … |
| `msg` | human-readable, no secrets |

Events from `subscribe` must be correlatable via `session_id` + `timestamp`.

### Diagnostics pack (user-exportable)

Must include:

1. Application + orchestrator + relay + APK version strings  
2. OS name/version/arch  
3. `adb version` and resolved `ADB` path  
4. `adb devices -l` snapshot  
5. Port listen state for configured relay port (e.g. ss/netstat equivalent)  
6. Last N MB of app + relay logs (redact local IPs optionally; keep serials)  
7. Recent error_code histogram  
8. Whether VPN permission was reported granted (best-effort from events)

**SLA:** UI reflects relay process death within **3s**; device disconnect within **5s**; adb daemon loss within **5s** of next polled ensure (poll ≤2s when session active).

Full failure→log mapping: FAILURE_SCENARIOS.md. Cross-platform quirks: CROSS_PLATFORM.md.

---

## 9. Ownership

| Area | Primary | Secondary |
|------|---------|-----------|
| Unit / integration tests | Eng | QA review |
| E2E lab scripts | QA | Eng fixtures |
| Compat / OEM | QA | — |
| Packaging / signing | Release | Eng |
| Docs accuracy vs MVP_SPEC / ARCHITECTURE | Tech writer + QA | Architect |

---

## 10. CI vs lab

| CI (every PR) | Lab (release + nightly) |
|---------------|-------------------------|
| Unit + mocked adb integration | Real USB E2E P0 |
| Static analysis, license/attribution | Android version matrix |
| Artifact build smoke (start binary, `--help` / version) | Failure injection (kill relay, occupy port) |
| No requirement for physical device | Soak, OEM, Windows drivers, udev |

CI must **not** skip packaging smoke for Linux AppImage/deb (or project-chosen format) and Windows installer when those jobs exist.

---


## 11. Companion sections (coverage map)

Use these documents as the executable companions to this strategy:

| Section | Where | What |
|---------|-------|------|
| **Test matrix** | [TEST_MATRIX.md](./TEST_MATRIX.md) | Android × OS × adb × VPN × relay cells; packaging; P0/P1/P2 |
| **Failure scenarios** | [FAILURE_SCENARIOS.md](./FAILURE_SCENARIOS.md) | detect → symptom → recovery → logs → pass/fail |
| **Recovery expectations** | FAILURE_SCENARIOS.md §1 + per-row Recovery | SLAs (relay death ≤3s, device gone ≤5s) |
| **Logging requirements** | this doc §8; FAILURE_SCENARIOS.md §6 | `timestamp`, `serial`, `adb_exit_code`, `port`, `stage`, `error_code` |
| **Diagnostics requirements** | this doc §8; FAILURE_SCENARIOS.md §7 | logs, `adb devices -l`, versions, OS, port listen state |
| **Release checklist** | [RELEASE_PLAN.md](./RELEASE_PLAN.md) §3 | preflight, smoke, sign-off, soak |

**Android versions:** claim API 21+; P0 retest modern Android 10–16; OEM variants P2 (TEST_MATRIX M-01–M-05).

---

## 12. Acceptance themes (MVP) — must map to cases

1. **Happy path USB** — select device → install if needed → start → IPv4 tether works → stop clean  
2. **Port conflict** — :31416 busy → clear error + offer alternate port or fail with `PORT_IN_USE`  
3. **Relay killed externally** — UI ≤3s → session marked failed → recovery path (`start` or `reset_tunnel`)  
4. **Quit while running** — orderly stop relay + reverse teardown; no orphan listener on port  
5. **APK version mismatch** — detect vs expected; block or prompt reinstall; log `APK_MISMATCH`  
6. **Multi-device require selection** — ≥2 authorized devices → no auto-start; user must pick serial  

Detailed cells: TEST_MATRIX.md. Pass/fail: FAILURE_SCENARIOS.md. Ship gate: RELEASE_PLAN.md.

---

## 13. Open questions (need architect / UX)

Tracked for resolution before freezing P0 scripts; do not block drafting:

- Exact UX when port is busy: auto-increment vs modal “choose port”?  
- APK mismatch: hard fail vs one-click reinstall?  
- Session state after VPN revoked mid-tether: auto-stop vs “degraded”?  
- Official stance on shipping optional platform-tools vs PATH-only (packaging risk).  
- macOS: notarization timeline vs “best-effort unsigned lab builds.”  

