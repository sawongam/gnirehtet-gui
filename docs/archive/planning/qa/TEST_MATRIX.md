# TEST_MATRIX.md

| Field | Value |
|-------|-------|
| Status | Draft v0.1 |
| Date | 2026-09-12 |
| Project | gnirehtet desktop GUI |
| Related | [TEST_STRATEGY.md](./TEST_STRATEGY.md), [FAILURE_SCENARIOS.md](./FAILURE_SCENARIOS.md), [RELEASE_PLAN.md](./RELEASE_PLAN.md), [CROSS_PLATFORM.md](./CROSS_PLATFORM.md) |

Concrete expectations only — **no executed results**. Priority: **P0** / **P1** / **P2**. Post-MVP cells marked explicitly.

Default relay port: **31416**. Stack: Tauri 2 + Svelte/TS + Rust orchestrator; relay sidecar; APK bundled; adb external.

---

## 1. Test matrix — Android × Host OS × adb × VPN × relay

Legend: **Exp** = expected MVP behavior. Cells are Pass criteria summaries.

| ID | Android | Host OS | adb state | VPN | Relay | Pri | Expected (MVP) |
|----|---------|---------|-----------|-----|-------|-----|----------------|
| M-01 | 10–16 (modern retest) | Linux | OK, 1 auth device | Grant on prompt | Healthy :31416 | P0 | Full tether; STOP tears down reverse + relay |
| M-02 | 10–16 | Windows | OK, 1 auth | Grant | Healthy | P0 | Same as M-01; USB driver present |
| M-03 | 10–16 | macOS | OK, 1 auth | Grant | Healthy | P1 | Best-effort Pass; quarantine/signing per CROSS_PLATFORM |
| M-04 | API 21+ claim smoke | Linux | OK | Grant | Healthy | P2 | Install+start+brief traffic; document OEM gaps |
| M-05 | OEM variant (Samsung/Xiaomi/…) | Linux or Win | OK | Grant | Healthy | P2 | VPN dialog survivable; no silent fail |
| M-06 | Any supported | Linux | **Missing adb** | n/a | n/a | P0 | `ADB_MISSING`; point to PATH/`ADB` config; no crash |
| M-07 | Any | Win | Missing adb | n/a | n/a | P0 | Same |
| M-08 | Any | Linux | Daemon **stopped** | — | — | P1 | `ensure_adb` restarts or instructs; retry list_devices |
| M-09 | Any | Any | Daemon **crashes** mid-session | Granted | Running | P1 | Detect ≤5s; session failed; recovery via ensure+start |
| M-10 | Any | Any | Device **unauthorized** | n/a | n/a | P0 | `ADB_UNAUTHORIZED`; wait/retry UI; no install attempt |
| M-11 | Any | Any | **Multiple** authorized | — | — | P0 | Must select serial; no auto-start (MVP) |
| M-12 | Any | Any | Multiple present, one unauthorized | — | — | P1 | List both; block unauthorized selection with clear reason |
| M-13 | Any | Any | Device **disappears** mid-session | Was granted | Running | P1 | UI ≤5s; `DEVICE_GONE`; stop relay or idle cleanly |
| M-14 | Any | Any | Device **reconnects** same serial | Re-grant if needed | Restartable | P1 | User can Start again; reverse re-established |
| M-15 | Any | Any | OK | **Denied** / dismiss | Ready | P0 | `VPN_DENIED`; session not “connected” |
| M-16 | Any | Any | OK | **Revoked** mid-session | Running | P0 | Detect; `VPN_REVOKED`; stop/offer reset |
| M-17 | Any | Any | OK | Granted | **Port occupied** | P0 | `PORT_IN_USE`; no false “connected” |
| M-18 | Any | Any | OK | Granted | **Killed externally** | P0 | UI ≤3s; `RELAY_DEAD`; recovery path |
| M-19 | Any | Any | OK | Granted | **Network failure** (host iface down) | P1 | Traffic fails visibly; optional auto-reconnect policy documented |
| M-20 | Any | Any | OK | Granted | Orderly **shutdown** | P0 | reverse removed; port free; no orphan |
| M-21 | Any | Any | OK | Granted | **Reconnect** after transient fail | P1 | reset_tunnel + start restores ≤30s user action |
| M-22 | Any | Any | OK | Granted | **Malformed** client traffic | P2 | Relay stable; log `MALFORMED`; session may continue |
| M-23 | Any | Any | OK | Granted | Healthy | P0 | **Quit app while running** → clean teardown |
| M-24 | Any | Any | OK | Granted | Healthy | P0 | **APK version mismatch** → `APK_MISMATCH`; block or forced reinstall |
| M-25 | App **restart** on device | Any | OK | Re-prompt if needed | Running/idle | P1 | Desktop detects; user can STOP/START |
| M-26 | Device **reboot** | Any | Re-auth USB if needed | Re-grant VPN | Stopped | P1 | Session ended; cold start works after authorize |

### Android coverage checklist (must appear in plans)

| Topic | Matrix IDs | Concrete expectation |
|-------|------------|----------------------|
| Android versions (API 21+ claim / retest 10–16 / OEM) | M-01–M-05 | Modern P0 on 10–16; API 21 P2 smoke; ≥1 OEM P2 |
| USB debugging | M-01–M-03 | Debugging enabled required; clear error if not |
| Authorized devices | M-01 | `adb devices` = device |
| Unauthorized devices | M-10 | No silent proceed |
| Multiple devices | M-11, M-12 | Selection required (simultaneous tether = **post-MVP**) |
| Reconnects | M-14, M-21 | Explicit user re-start unless product adds auto (document) |
| VPN permission | M-15 | On-device grant |
| VPN permission revoked | M-16 | Fail session cleanly |
| App restart | M-25 | Desktop state sync |
| Device reboot | M-26 | Cold path works |

### ADB coverage checklist

| Topic | IDs | Expectation |
|-------|-----|-------------|
| adb missing | M-06, M-07 | Hard fail with fix hint |
| adb daemon stopped | M-08 | ensure_adb recovers or instructs |
| adb daemon crashes | M-09 | Detect + recover |
| device disappears | M-13 | SLA ≤5s |
| device reconnects | M-14 | Startable again |
| multiple devices | M-11 | Selection |
| unauthorized device | M-10 | Block |

### Relay coverage checklist

| Topic | IDs | Expectation |
|-------|-----|-------------|
| relay failure | M-18 | ≤3s UI |
| port already occupied | M-17 | `PORT_IN_USE` |
| network failure | M-19 | Visible degradation |
| shutdown | M-20 | Clean |
| reconnect | M-21 | ≤30s with user action |
| malformed traffic | M-22 | No crash |

---

## 2. Desktop OS matrix

| ID | Platform | Pri | Install / run expectation | Notes |
|----|----------|-----|---------------------------|-------|
| D-L1 | Linux (Ubuntu LTS) | P0 | App launches; webview deps present or documented | udev for USB |
| D-L2 | Linux (Fedora or Arch sample) | P2 | Document webkit/GTK deps | Missing deps → clear error |
| D-W1 | Windows 10 | P0 | Installer + first tether | USB driver prerequisite |
| D-W2 | Windows 11 | P0 | Same | |
| D-M1 | macOS (current −1) | P1 | Best-effort launch + tether | Notarization/quarantine |
| D-M2 | macOS (current) | P2 | Same | |

Details: CROSS_PLATFORM.md.

---

## 3. Packaging matrix

| ID | Scenario | Pri | Expected |
|----|----------|-----|----------|
| P-01 | Fresh **installation** Linux | P0 | Binary runs; APK+relay present relative to app |
| P-02 | Fresh installation Windows | P0 | Same; Start Menu/entry works |
| P-03 | Fresh installation macOS | P1 | Gatekeeper/quarantine documented |
| P-04 | **Upgrade** N → N+1 | P1 | Settings preserved; APK replaced if newer; no dual relays |
| P-05 | **System PATH adb** | P0 | Discovered via `ensure_adb`; version logged |
| P-06 | **Configured adb path** (`ADB` env or settings) | P0 | Overrides PATH; invalid path → `ADB_MISSING` |
| P-07 | **Bundled relay + APK** (MVP default) | P0 | Sidecar spawn works; `GNIREHTET_APK` override honored |
| P-08 | **Bundled adb / platform-tools** (optional packaging policy) | P2 | If shipped: isolated from system adb; version pinned; **risk**: license size, driver still needed on Windows. If **not** shipped (MVP prefer system adb): document and skip binary assert — still validate “user pointed at SDK platform-tools” |
| P-09 | **Missing dependencies** (Linux webview libs) | P1 | Installer/docs list deps; app error ≠ segfault silence |
| P-10 | **Permissions** (Linux udev, Win admin-not-required for adb) | P0/P1 | Non-root app; udev rule doc; Win: no admin for normal tether |
| P-11 | **Code signing** Windows | P0 for release | Signed installer or documented SmartScreen caveat |
| P-12 | **Code signing / notarization** macOS | P1 | Best-effort; unsigned = lab-only label |
| P-13 | Missing `GNIREHTET_APK` override file | P1 | Fall back to bundled; if bundled missing → hard fail |

**Clarify:** Product does **not** auto-vendor full Android SDK. Test **bundled relay/APK** as required; treat **bundled adb** as optional packaging option/risk to validate if Release elects it; MVP prefers **system PATH / configured** adb.

---

## 4. MVP acceptance theme → case map

| Theme | Primary IDs | Pri |
|-------|-------------|-----|
| Happy path USB | M-01, M-02, D-L1, D-W1 | P0 |
| Port conflict | M-17 | P0 |
| Relay killed externally | M-18 | P0 |
| Quit while running | M-23 | P0 |
| APK version mismatch | M-24 | P0 |
| Multi-device require selection | M-11 | P0 |

---

## 5. Post-MVP expansion matrix (do not block MVP)

| ID | Scenario | Note |
|----|----------|------|
| X-01 | Two devices **simultaneous** tether | OUT of MVP UI; add when ARCHITECTURE supports |
| X-02 | Wireless adb primary | Expand adb state column; flaky link soaks |
| X-03 | IPv6 | Out of product claim |
| X-04 | Full macOS P0 parity | After notarization + USB stability |

---

## 6. Soak & recovery SLAs (measurable)

| Check | SLA / metric | Pri |
|-------|--------------|-----|
| Relay death → UI | ≤ **3s** | P0 |
| Device gone → UI | ≤ **5s** | P1 |
| adb daemon loss → UI | ≤ **5s** after poll (poll ≤2s active) | P1 |
| User reconnect after transient | ≤ **30s** guided | P1 |
| Soak duration | ≥ **4h** one host OS P0 | P0 |
| Orphan port after quit | **0** listeners on relay port | P0 |

---

## 7. Logging / diagnostics requirements (matrix gate)

Every P0 E2E must assert diagnostics pack contains: version strings, OS, `adb devices -l`, adb path, port listen state, logs with `timestamp`, `serial`, `stage`, `port`, `adb_exit_code`, `error_code`.

See TEST_STRATEGY.md §8 and FAILURE_SCENARIOS.md must-log columns.

---

## 8. Traceability

| Doc | Use with this matrix |
|-----|----------------------|
| FAILURE_SCENARIOS.md | Pass/fail detail per failure mode |
| RELEASE_PLAN.md | Which rows are release blockers |
| CROSS_PLATFORM.md | How D-* and P-* differ by OS |

