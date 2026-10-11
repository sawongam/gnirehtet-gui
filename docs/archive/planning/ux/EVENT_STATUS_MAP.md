# Event → Status Map (one-pager)

**Audience:** Desktop Engineer, QA, Rust Architect  
**Source of truth for UX verbs/states:** `MVP_UX.md`, `ERROR_UX.md`, `SCREEN_SPEC.md`  
**Orchestrator events (ARCHITECTURE):** `DeviceChanged` | `RelayState` | `LogLine` | `Error`

---

## 1. Session chip (user-facing)

| Chip | When |
|------|------|
| Idle | No owned session; relay not ours (or stopped cleanly) |
| Starting | Run pipeline in progress (install/tunnel/start/relay) |
| Waiting for VPN | Client started; Device VPN not Active yet (`VPN_PERMISSION_PENDING`) |
| Sharing | **Relay Listening + Tunnel OK + Device VPN Active** (handshake done) |
| Interrupted | Was Sharing; tunnel lost (`TUNNEL_LOST`) — primary CTA: **Repair tunnel** |
| Stopping | Stop pipeline in progress |
| Error | Blocking `Error` with recovery; chip title from ERROR_UX |

**Never** show Sharing / Connected on intent-sent or relay-only.

---

## 2. Three-layer strip ← events

| Layer | Healthy | From |
|-------|---------|------|
| Relay | Listening + port | `RelayState` = listening / ready |
| Tunnel | OK (for selected serial) | tunnel success after `adb reverse`; clear on unplug |
| Device VPN | Active | handshake / client-id read (not START intent alone) |

| Layer | Degraded / error | From |
|-------|------------------|------|
| Relay Off | idle / stopped | `RelayState` stopped |
| Relay Error | crashed / exited | `RelayState` exited + typed crash → UI `RELAY_CRASHED` |
| Relay Error | bind failed | `Error` `PORT_IN_USE` **before** claiming ownership |
| Tunnel Lost / Failed | unplug or reverse fail | `TUNNEL_LOST` / `TUNNEL_FAILED` |
| Device VPN Pending | waiting consent | `VPN_PERMISSION_PENDING` |
| Device VPN Error | denied / OEM / start fail | `VPN_PERMISSION_DENIED`, `VENDOR_PERMISSION_MONITORING`, `CLIENT_START_FAILED` |

---

## 3. Commands ↔ UX verbs

| UI | Orchestrator / CLI mirror |
|----|---------------------------|
| Start sharing | install-if-needed → tunnel → start → relay (`run` semantics) |
| Stop sharing | stop client + stop owned relay |
| Repair tunnel | `tunnel` (restore `adb reverse`); stay Interrupted until layers healthy |
| Refresh devices | `list_devices` / `DeviceChanged` |
| Reinstall helper | `reinstall` |
| Choose adb… | settings; `ensure_adb` |

Shell must not invent ad-hoc adb outside orchestrator.

---

## 4. Event routing rules

| Signal | Route |
|--------|-------|
| Child exit / crash after we own relay | `RelayState` (exited) + typed process error → chip Error / layer Relay; code `RELAY_CRASHED` |
| Port bind failure before ownership | `Error` `PORT_IN_USE` only — do **not** set Relay Listening |
| Device list / unauthorized / offline | `DeviceChanged` → device row states; may gate Start |
| Log stdout/stderr | `LogLine` → log strip (always append; never alone sets Sharing) |
| Quit while owning relay | teardown; port `:31416` (or configured) free — QA P0 |
| Apps say offline while layers healthy | tip / Diagnostics `APPS_REPORT_NO_INTERNET` — **no** green Sharing change |

---

## 5. Primary CTAs by chip

| Chip | Primary | Secondary |
|------|---------|-----------|
| Idle (device ready) | Start sharing | Install helper, Settings |
| Waiting for VPN | I’ve allowed it (recheck) | Stop sharing |
| Sharing | Stop sharing | Repair tunnel (rare) |
| Interrupted | Repair tunnel | Stop sharing, Replug tips |
| Error | code-specific recovery from ERROR_UX | Copy error, Open Diagnostics |

---

*MVP: one selected device / one session. Multi-share & tray = Later.*
