# ERROR_CODE_MAP.md

| Field | Value |
|-------|-------|
| Status | Draft v0.1 |
| Date | 2026-09-12 |
| Project | gnirehtet desktop GUI |
| Purpose | Align networking FAILURE_MODES, UX ERROR_UX codes, and prior QA FAILURE_SCENARIOS codes for tests going forward |
| Canonical rule | **Canonical test code = ERROR_UX code where it exists**; otherwise mark **gap / needs-signal** — no invented behavior |
| Related | [FAILURE_SCENARIOS.md](./FAILURE_SCENARIOS.md), [TEST_STRATEGY.md](./TEST_STRATEGY.md), [TEST_MATRIX.md](./TEST_MATRIX.md), [PHASE0_ACCEPTANCE.md](./PHASE0_ACCEPTANCE.md), [FAILURE_MODES.md](../networking/FAILURE_MODES.md), [ERROR_UX.md](../ux/ERROR_UX.md), [EVENT_STATUS_MAP.md](../ux/EVENT_STATUS_MAP.md) |

Sharing = Relay Listening + Tunnel OK + Device VPN Active (handshake). Never claim Sharing on intent-sent or relay-only (EVENT_STATUS_MAP).

**SLA sources (known):** TEST_STRATEGY §8 / FAILURE_SCENARIOS §1 / EVENT_STATUS_MAP P0 notes — relay death UI **≤3s**; device disconnect UI **≤5s**; adb daemon loss **≤5s** (poll ≤2s when active); quit frees port before exit completes; transient restore **≤30s** with user action where documented.

---

## 1. Legend

| Column | Meaning |
|--------|---------|
| Networking FAILURE_MODES | Name / recommended taxonomy from `docs/architecture/networking/FAILURE_MODES.md` |
| Prior QA code | Code used in FAILURE_SCENARIOS / TEST_STRATEGY §8 / TEST_MATRIX |
| ERROR_UX code | Stable UX code from `ERROR_UX.md` |
| **Canonical** | Code tests/logs should use going forward |
| Chip / layer | EVENT_STATUS_MAP session chip and/or three-layer impact |
| Recovery SLA | From QA/UX docs if stated; else — |
| Notes | gap / needs-signal / MVP Later / deprecation of prior QA alias |

---

## 2. Master map

### 2.A ADB / environment

| Networking FAILURE_MODES | Prior QA | ERROR_UX | Canonical | Chip / layer | Recovery SLA | Notes |
|--------------------------|----------|----------|-----------|--------------|--------------|-------|
| `AdbUnavailable` (recommended taxonomy §9); adb dies / replaced (§3) | `ADB_MISSING` | `ADB_MISSING` | **`ADB_MISSING`** | Error (setup); layer — | Immediate after fix path | Align |
| — (path probe) | — | `ADB_PATH_INVALID` | **`ADB_PATH_INVALID`** | Error (setup); layer — | Immediate after Choose adb… | No prior QA alias |
| — (server conflict) | `ADB_DAEMON` (partial) | `ADB_SERVER_MISMATCH` | **`ADB_SERVER_MISMATCH`** when detectable | Error / device flicker; layer — | Retry / same-adb; UI ≤5s if mid-session loss | Prior `ADB_DAEMON` broader than UX code — **needs-signal** for pure daemon crash vs mismatch; ERROR_UX: MVP if detectable else Later; do not kill shared adb on quit by default |
| DeviceMissing / empty devices | `ADB_NO_DEVICE` | `NO_DEVICES` | **`NO_DEVICES`** | Idle / empty; Start gated | After cable/debug fix + Refresh | Prior alias deprecated |
| Unauthorized (adb) | `ADB_UNAUTHORIZED` | `DEVICE_UNAUTHORIZED` | **`DEVICE_UNAUTHORIZED`** | Device row; Start gated | After RSA accept; refresh ≤5s or manual | Prior alias deprecated |
| — (offline transport) | (often folded into `DEVICE_GONE`) | `DEVICE_OFFLINE` | **`DEVICE_OFFLINE`** | Device row; Tunnel degraded possible | Wake/replug + Refresh | Prefer over vague gone when `offline` state seen |
| Multi-device no selection | `ADB_MULTI` | `MULTIPLE_DEVICES_NO_SELECTION` | **`MULTIPLE_DEVICES_NO_SELECTION`** | Inline under Start; Idle | Immediate on select | Prior alias deprecated |

### 2.B APK / helper

| Networking FAILURE_MODES | Prior QA | ERROR_UX | Canonical | Chip / layer | Recovery SLA | Notes |
|--------------------------|----------|----------|-----------|--------------|--------------|-------|
| — (bundle path) | `APK_MISSING` | `APK_MISSING` | **`APK_MISSING`** | Error; Device VPN layer | Before Start allowed | Align |
| — (install) | — | `INSTALL_FAILED` | **`INSTALL_FAILED`** | Error; Device VPN | Reinstall / OEM USB install | No distinct prior QA code |
| — (version compare) | `APK_MISMATCH` | `APK_VERSION_MISMATCH` | **`APK_VERSION_MISMATCH`** | Error; Device VPN | Reinstall before Start | Prior alias deprecated |

### 2.C Relay

| Networking FAILURE_MODES | Prior QA | ERROR_UX | Canonical | Chip / layer | Recovery SLA | Notes |
|--------------------------|----------|----------|-----------|--------------|--------------|-------|
| Relay process dies (§4); `RelayCrashed` / `RelayNotRunning` (§9) | `RELAY_DEAD` | `RELAY_CRASHED` | **`RELAY_CRASHED`** | Error; Relay Error | UI **≤3s**; then restart sharing path | Prior alias deprecated |
| Bind / port busy (GUI requirement; EVENT_STATUS_MAP) | `PORT_IN_USE` | `PORT_IN_USE` | **`PORT_IN_USE`** | Error; Relay Error — **before** claiming Listening | On start_relay | Align; never set Relay Listening on bind fail |
| Relay start fail (exited / never listened) | (often `RELAY_DEAD` at start) | `RELAY_START_FAILED` | **`RELAY_START_FAILED`** | Error; Relay | Retry Start; firewall tips | Distinguish from mid-session crash when detectable |
| Leftover after Stop/Quit | (F-R4 fail mode; `SHUTDOWN` success path) | `ORPHAN_RELAY` | **`ORPHAN_RELAY`** | Idle UI but port busy; Relay | Stop leftover relay; quit must free port | P0 quit = no orphan; code when leftover detected |
| Relay not listening but reverse exists (§1/§4); handshake read fails | — | `HANDSHAKE_FAILED` | **`HANDSHAKE_FAILED`** | Error; Relay+VPN | Repair tunnel / restart; **needs device validation** | ERROR_UX: MVP if signals exist; else `START_TIMEOUT` / tips |
| Firewall/AV blocking listen | `NETWORK_FAIL` (partial) | `FIREWALL_RELAY` | **`FIREWALL_RELAY`** | Relay | Allow app; retry | ERROR_UX **Later** unless cheap to detect — **gap** if only inferred |

### 2.D Tunnel / USB / device loss

| Networking FAILURE_MODES | Prior QA | ERROR_UX | Canonical | Chip / layer | Recovery SLA | Notes |
|--------------------------|----------|----------|-----------|--------------|--------------|-------|
| USB disconnect (§2); `TunnelDisconnected` (§9); reverse must be re-asserted | `DEVICE_GONE` (mid-session) | `TUNNEL_LOST` | **`TUNNEL_LOST`** when was Sharing / tunnel path broken | **Interrupted**; Tunnel Lost; primary Repair tunnel | UI ≤5s device loss; Repair / replug | Prior `DEVICE_GONE` still OK for absent serial pre-session; mid-share prefer `TUNNEL_LOST` |
| `ReverseFailed` (§9); tunnel setup fail | — | `TUNNEL_FAILED` | **`TUNNEL_FAILED`** | Error; Tunnel Failed | Repair tunnel | No prior QA alias |
| — (reconnect success taxonomy) | — | — | — | Tunnel OK when reverse restored | — | `TunnelConnected` is health signal, not an error code |

### 2.E Client / Device VPN / OEM

| Networking FAILURE_MODES | Prior QA | ERROR_UX | Canonical | Chip / layer | Recovery SLA | Notes |
|--------------------------|----------|----------|-----------|--------------|--------------|-------|
| VPN permission / `VpnNotPrepared`; waiting consent | — | `VPN_PERMISSION_PENDING` | **`VPN_PERMISSION_PENDING`** | **Waiting for VPN**; Device VPN Pending | After user Allow + I’ve allowed it | **Not Sharing** |
| VPN prepare denied | `VPN_DENIED` | `VPN_PERMISSION_DENIED` | **`VPN_PERMISSION_DENIED`** | Error; Device VPN Error | Re-Start + Allow | Prior alias deprecated |
| VPN permission revoked (§5); `VpnRevoked` | `VPN_REVOKED` | — (no dedicated ERROR_UX code) | **gap → treat as hard stop**; map UX to stop + fresh START / `VPN_PERMISSION_*` or `CLIENT_START_FAILED` per signals | Error / leave Sharing; Device VPN | UI ≤5s revoke (FAILURE_SCENARIOS); re-grant + Start | **needs-signal** — do not invent `VPN_REVOKED` in ERROR_UX; document detection when available |
| Another VPN starts (§1); platform deactivates | — | `VPN_ALREADY_ACTIVE` | **`VPN_ALREADY_ACTIVE`** | Error; Device VPN | Disconnect other VPN | ERROR_UX: MVP if detectable; else tips under start failure |
| `establish()` returns null; `VpnEstablishFailed` | — | `CLIENT_START_FAILED` (and/or tips) | **`CLIENT_START_FAILED`** unless more specific signal | Error; Device VPN | Retry / Diagnostics | Prefer OEM code when Permission Monitoring matched |
| OEM Permission Monitoring / `am start` 255 | — | `VENDOR_PERMISSION_MONITORING` | **`VENDOR_PERMISSION_MONITORING`** | Error; Device VPN Error; OEM card | Disable monitoring / Install via USB; Retry | P0 UX priority |
| Helper hang no VPN prompt | — | `CLIENT_HANG_NO_PROMPT` | **`CLIENT_HANG_NO_PROMPT`** | Error / Waiting escalation; Device VPN | Open helper / reinstall; **needs device validation** | MVP if signals exist else `CLIENT_START_FAILED` / `START_TIMEOUT` |
| Process kill on device (§1); `START_NOT_STICKY` | `APP_DEAD` | — | **gap / needs-signal** → degraded session + user Start; prefer `CLIENT_START_FAILED` or monitor-driven Error when wired | Leave Sharing; Device VPN | User Start again | No ERROR_UX twin yet |
| Start pipeline timeout | — | `START_TIMEOUT` | **`START_TIMEOUT`** | Error; Any layer | Retry / I’ve allowed VPN / Repair | Align |
| Stop / teardown fail | `SHUTDOWN` (success path only) | `STOP_FAILED` | **`STOP_FAILED`** on failure | Error / Stopping incomplete; Any | Retry Stop; stop leftover relay | `SHUTDOWN` is not an ERROR_UX failure code |

### 2.F Path / host network / partial session internals

| Networking FAILURE_MODES | Prior QA | ERROR_UX | Canonical | Chip / layer | Recovery SLA | Notes |
|--------------------------|----------|----------|-----------|--------------|--------------|-------|
| Host uplink / iface down | `NETWORK_FAIL` | `PC_OFFLINE` | **`PC_OFFLINE`** when detectable | Tip / Error; Relay/path — **do not** fake Sharing change | Fix PC network | ERROR_UX **Later** preflight unless cheap — **gap** otherwise |
| Apps say offline while layers healthy | — | `APPS_REPORT_NO_INTERNET` | **`APPS_REPORT_NO_INTERNET`** | Diagnostics tip; **no** green Sharing change | Wi‑Fi on / DNS tips | Not a red session chip if layers green |
| Invalid / non-IPv4 packet; buffer full; UDP idle; partial drops (§1/§6) | `MALFORMED` (fuzz/parse) | — | **no user ERROR_UX** — keep relay/advanced logs | Sharing unchanged if layers still healthy | None if stable | Do not invent UX codes for packet drops |
| IPv4 fragments unsupported | — | — | — | — | — | Documented gap in FAILURE_MODES; not a GUI code |

---

## 3. Prior QA → Canonical (quick deprecation list)

| Prior QA code | Canonical going forward | Status |
|---------------|-------------------------|--------|
| `ADB_MISSING` | `ADB_MISSING` | Keep |
| `ADB_NO_DEVICE` | `NO_DEVICES` | Rename |
| `ADB_UNAUTHORIZED` | `DEVICE_UNAUTHORIZED` | Rename |
| `ADB_MULTI` | `MULTIPLE_DEVICES_NO_SELECTION` | Rename |
| `ADB_DAEMON` | `ADB_SERVER_MISMATCH` **or** needs-signal mid-session adb loss | Split / gap |
| `PORT_IN_USE` | `PORT_IN_USE` | Keep |
| `RELAY_DEAD` | `RELAY_CRASHED` (mid-session) / `RELAY_START_FAILED` (never listened) | Rename + split when detectable |
| `APK_MISSING` | `APK_MISSING` | Keep |
| `APK_MISMATCH` | `APK_VERSION_MISMATCH` | Rename |
| `VPN_DENIED` | `VPN_PERMISSION_DENIED` | Rename |
| `VPN_REVOKED` | **gap** — no ERROR_UX twin; hard-stop + re-START | needs-signal |
| `DEVICE_GONE` | `TUNNEL_LOST` if mid-share; else device absent / `DEVICE_OFFLINE` / `NO_DEVICES` | Context split |
| `NETWORK_FAIL` | `PC_OFFLINE` / `FIREWALL_RELAY` / tip codes when detectable | Later / gap |
| `MALFORMED` | (log only; no ERROR_UX) | Keep as advanced log tag if useful |
| `SHUTDOWN` | Success teardown marker — not an error; failures → `STOP_FAILED` / `ORPHAN_RELAY` | Not canonical error |
| `APP_DEAD` | **gap / needs-signal** | Map when monitor exists |

---

## 4. Gaps / needs-signal register

| ID | Topic | Why open | Interim test guidance |
|----|-------|----------|------------------------|
| G1 | `VPN_REVOKED` | FAILURE_MODES + prior QA name it; ERROR_UX has no dedicated code | Assert leave Sharing ≤5s; require fresh START + prepare; log stage + evidence; do not invent UX code in product strings until UX adds it |
| G2 | `APP_DEAD` / device process kill | FAILURE_MODES observed `START_NOT_STICKY`; weak host signal | Desktop must not stay Sharing indefinitely; user Start; prefer explicit code when monitor ships |
| G3 | `ADB_DAEMON` vs `ADB_SERVER_MISMATCH` | Prior QA conflates crash and multi-server conflict | Use `ADB_SERVER_MISMATCH` only when detectable; else generic ensure_adb recovery + ≤5s UI |
| G4 | `HANDSHAKE_FAILED` / client-id | ERROR_UX + FAILURE_MODES: connect≠relay serving | **Needs device validation**; until then do not claim Sharing on reverse-only |
| G5 | `VPN_ALREADY_ACTIVE`, `CLIENT_HANG_NO_PROMPT` | ERROR_UX conditional MVP | Fall back to `CLIENT_START_FAILED` / `START_TIMEOUT` + tips |
| G6 | `PC_OFFLINE`, `FIREWALL_RELAY` | Later unless cheap | Do not block Phase 0; avoid false positives |
| G7 | `MALFORMED` / buffer / UDP idle | Networking internal | No session chip change required if relay stays up |
| G8 | Orchestrator taxonomy vs ERROR_UX names | FAILURE_MODES §9 uses `AdbUnavailable`, `ReverseFailed`, … | Map at orchestrator→UI boundary to Canonical column; do not expose dual taxonomies in UI |

---

## 5. Chip impact cheat-sheet (canonical codes)

| Canonical code | Typical chip | Layer strip |
|----------------|--------------|-------------|
| `PORT_IN_USE` | Error | Relay Error (never Listening) |
| `RELAY_CRASHED` / `RELAY_START_FAILED` / `ORPHAN_RELAY` | Error (Idle if orphan after failed cleanup) | Relay Error / Off |
| `TUNNEL_LOST` | **Interrupted** | Tunnel Lost |
| `TUNNEL_FAILED` | Error | Tunnel Failed |
| `VPN_PERMISSION_PENDING` | **Waiting for VPN** | Device VPN Pending |
| `VPN_PERMISSION_DENIED` / `VENDOR_PERMISSION_MONITORING` / `CLIENT_START_FAILED` | Error | Device VPN Error |
| `APPS_REPORT_NO_INTERNET` | (unchanged if layers green) | Tip only |
| `START_TIMEOUT` / `STOP_FAILED` / `HANDSHAKE_FAILED` | Error | Per failed layer |

---

## 6. Recovery SLA index (known only)

| Event | SLA | Source |
|-------|-----|--------|
| Owned relay death → UI | ≤**3s** | TEST_STRATEGY, FAILURE_SCENARIOS, P0 |
| Device disconnect → UI | ≤**5s** | TEST_STRATEGY |
| adb daemon loss → UI | ≤**5s** (poll ≤2s when active) | TEST_STRATEGY |
| Quit while owning relay | Port free; no orphan before exit completes | EVENT_STATUS_MAP, FAILURE_SCENARIOS F-R4 |
| Unauthorized → refresh after grant | ≤5s auto or manual Refresh | FAILURE_SCENARIOS F-A2 |
| User restore after break | ≤**30s** with action | FAILURE_SCENARIOS §1 |
| VPN revoke mid-session | UI ≤5s | FAILURE_SCENARIOS F-A7 |

Rows without a cited SLA: **do not invent** timing — mark “—” in lab scripts until Architect/UX sets one.

---

## 7. Row counts (this draft)

| Section | Data rows (approx.) |
|---------|---------------------|
| §2.A ADB / environment | 7 |
| §2.B APK / helper | 3 |
| §2.C Relay | 6 |
| §2.D Tunnel / USB | 3 |
| §2.E Client / VPN / OEM | 10 |
| §2.F Path / internals | 4 |
| **§2 master map total** | **33** |
| §3 prior→canonical | 16 |
| §4 gaps register | 8 |

Update this doc when ERROR_UX or FAILURE_MODES gain real signals — do not fill gaps with guessed product behavior.
