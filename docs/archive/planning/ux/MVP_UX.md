# MVP UX — gnirehtet Desktop GUI

Explicit cut of research + flows + screens + errors for the first shippable GUI.  
Authoritative functional scope: **MVP_SPEC**. Vision lives in other docs; anything not listed here is **deferred**.

---

## 1. MVP product promise

Launch → see adb devices **or** clear missing/unauthorized → install helper APK if needed → one primary **Start sharing** → live status (Relay · Tunnel · Device VPN) + logs → clean **Stop sharing**.

- **ONE** active share session  
- Direction chrome always: **Internet: This PC → Phone**  
- Honest about on-device steps (USB auth, VPN permission, OEM Permission Monitoring)  
- Primary recovery after unplug: **Repair tunnel**

---

## 2. MVP screen list only

| # | Screen | Must ship |
|---|--------|-----------|
| 1 | First-run / Setup checklist | Yes |
| 2 | Main (devices + three-layer status + Start/Stop + Repair + log strip) | Yes |
| 3 | Waiting-for-VPN panel | Yes |
| 4 | Interrupted banner (tunnel lost) | Yes |
| 5 | OEM Permission Monitoring card | Yes |
| 6 | Settings / Advanced (adb path, port, DNS/routes optional, APK path) | Yes |
| 7 | Diagnostics (doctor checklist + copy logs) | Yes |
| 8 | About / Licenses (Genymobile attribution) | Yes |
| 9 | Confirm quit if sharing | Yes |
| — | System tray agent | **No** |
| — | Multi-device simultaneous share UI | **No** |
| — | Wireless-adb happy path | **No** |
| — | Autorun / auto-share UI | **No** |
| — | Throughput charts / fancy metrics | **No** |
| — | Play Store client | **No** |

---

## 3. MVP navigation

```
[Setup once] ──► Main ◄──► Settings
                  │
                  ├── Diagnostics (sheet/panel)
                  ├── About (dialog)
                  ├── Waiting-VPN (panel/modal)
                  └── Error/OEM/Interrupted (inline on Main)
```

- Single primary window  
- Close: confirm if session active → Stop sharing then quit  
- Minimize to taskbar OK; **no tray requirement**

---

## 4. MVP state machine

User-facing session states:

`Idle` → `Starting` → `WaitingVpn` → `Sharing` → `Stopping` → `Idle`  
↘ `Error`  
`Sharing` → `Interrupted` → (Repair) → `Sharing` | Stop → `Stopping`

**Enter `Sharing` only when:**

1. Relay listening  
2. Tunnel present for selected serial  
3. Device VPN allowed **and** client handshake OK (**needs device validation** for handshake signal; if unavailable, document fallback and never claim Sharing on intent-sent alone)

**Three-layer strip** always visible on Main (see SCREEN_SPEC).

---

## 5. MVP actions (verbs)

| UI | CLI / engine |
|----|----------------|
| Start sharing | install-if-needed + tunnel + start + relay (`run`) |
| Stop sharing | `stop` + tear down reverse + stop relay |
| Repair tunnel | `tunnel` |
| Refresh devices | `adb devices` |
| Reinstall helper | `reinstall` / uninstall+install |
| Choose adb… | persist path (`ADB`) |

---

## 6. MVP copy deck (key strings)

| ID | String |
|----|--------|
| `dir.badge` | Internet: This PC → Phone |
| `cta.start` | Start sharing |
| `cta.stop` | Stop sharing |
| `cta.repair` | Repair tunnel |
| `cta.refresh` | Refresh devices |
| `cta.reinstall` | Reinstall helper |
| `chip.idle` | Idle |
| `chip.starting` | Starting |
| `chip.waiting_vpn` | Waiting for VPN permission |
| `chip.sharing` | Sharing |
| `chip.interrupted` | Interrupted |
| `chip.stopping` | Stopping |
| `chip.error` | Error |
| `layer.relay.off` | Relay off |
| `layer.relay.on` | Relay listening |
| `layer.tunnel.ok` | Tunnel OK |
| `layer.tunnel.lost` | Tunnel lost |
| `layer.vpn.off` | Device VPN off |
| `layer.vpn.wait` | Waiting for VPN |
| `layer.vpn.on` | Device VPN on |
| `empty.no_devices.title` | No Android devices detected |
| `empty.no_devices.body` | Use a data-capable USB cable. Enable Developer options → USB debugging. Unlock the phone and accept Allow USB debugging. Try another port (avoid unpowered hubs). |
| `empty.unauth` | Unlock the phone and tap Allow. You can check Always allow from this computer. |
| `empty.adb_missing` | ADB not found. Choose your platform-tools adb binary. This app does not download it automatically. |
| `vpn.title` | Allow VPN permission on your phone |
| `vpn.body` | Android asks for a VPN connection so the phone can send traffic through this PC — not a commercial VPN. Look for the key icon in the status bar when sharing is active. |
| `vpn.ive_allowed` | I’ve allowed it |
| `interrupted.title` | Sharing interrupted |
| `interrupted.body` | Unplugging USB kills the adb reverse tunnel. Replug, then Repair tunnel. |
| `oem.title` | Phone blocked the helper start |
| `oem.body` | OEM Permission Monitoring likely blocked starting the helper (common on Xiaomi/MIUI). In Developer options, disable Permission Monitoring. Enable Install via USB and USB debugging (Security settings) if shown. Then retry. |
| `quit.confirm` | Stop sharing and quit? |
| `settings.dns` | DNS (optional) |
| `settings.port` | Relay port |
| `about.attr` | Based on Genymobile gnirehtet |
| `log.copy` | Copy logs |
| `diag.copy` | Copy diagnostics |
| `relay.zero_clients` | Relay is listening, but no phone is sharing yet. |
| `tip.apps_offline` | Some apps report offline on a VPN path. Try Wi‑Fi on (no need to join) or a DNS preset. |

**Avoid in MVP chrome:** Connect VPN, Kill switch, BLOCKING INTERNET, Protected/Secured, Tether (without “reverse”), delight/SaaS fluff.

---

## 7. MVP error codes (must handle)

| Priority | Codes |
|----------|-------|
| P0 | `ADB_MISSING`, `ADB_PATH_INVALID`, `NO_DEVICES`, `DEVICE_UNAUTHORIZED`, `DEVICE_OFFLINE`, `MULTIPLE_DEVICES_NO_SELECTION`, `APK_MISSING`, `INSTALL_FAILED`, `APK_VERSION_MISMATCH`, `PORT_IN_USE`, `RELAY_START_FAILED`, `RELAY_CRASHED`, `TUNNEL_FAILED`, `TUNNEL_LOST`, `CLIENT_START_FAILED`, `VENDOR_PERMISSION_MONITORING`, `VPN_PERMISSION_DENIED`, `VPN_PERMISSION_PENDING`, `START_TIMEOUT`, `STOP_FAILED`, `ORPHAN_RELAY` |
| Nice | `ADB_SERVER_MISMATCH`, `VPN_ALREADY_ACTIVE`, `CLIENT_HANG_NO_PROMPT`, `HANDSHAKE_FAILED`, `APPS_REPORT_NO_INTERNET` tip |

Full tables: `ERROR_UX.md`.

---

## 8. Acceptance ↔ MVP_SPEC

| MVP_SPEC intent | UX acceptance |
|-----------------|---------------|
| Detect adb | Setup item + `ADB_MISSING` / Choose adb… |
| List devices serial/state | Main device rows; unauthorized/offline chips |
| Select one | Selection required; `MULTIPLE_DEVICES_NO_SELECTION` |
| Install APK if needed | Part of Start sharing; Reinstall helper |
| Relay port 31416 | Settings default; layer shows port |
| adb reverse | Tunnel layer; Repair tunnel |
| Start/stop intents | Start/Stop sharing |
| Run orchestration | Start sharing pipeline |
| Logs | Collapsible copyable strip + Diagnostics |
| Reset tunnel | **Repair tunnel** control + Interrupted path |
| Optional DNS/routes | Settings Advanced |
| Persist settings | Settings Save |
| First-run checklist | Setup screen |
| VPN permission on phone | Waiting-VPN panel; desktop cannot click |
| One session only | Single Start; no multi-share UI |
| No auto-download platform-tools | Copy + no silent download |
| No tray agent | Close confirms; no tray menu |
| No wireless-first | USB happy path only |
| Clean Stop | Stop sharing + quit confirm |

---

## 9. Explicitly deferred (**Later**)

| Item | Why deferred |
|------|--------------|
| Multi-device simultaneous tether UI | MVP_SPEC; serial select only |
| Autorun / auto-share new devices | CLI exists; UI later |
| Tray agent + color glyph menu | Nice for long sessions |
| Wireless-adb pairing wizard | Secondary; USB first |
| Throughput charts / traffic monitor | Wirebound-like; not needed for trust |
| Nicknames / device groups | QtScrcpy-like |
| Bundled auto-download of platform-tools | Spec forbids auto-download |
| IPv6 | Upstream IPv4 TCP/UDP |
| Play Store client | Out of MVP |
| HTTP connectivity doctor step | Nice; tips first |
| Quest/headset special profile | Niche (#577/#578) |
| Localization / themes | Polish |
| Kill-switch / lockdown analogues | **Wrong metaphor — never** |

---

## 10. Open UX questions (need device validation)

1. Exact reliable signal for “client handshake / id received” before Sharing.  
2. How quickly key icon appears/disappears vs our Stop.  
3. Whether DNS/routes can hot-apply (assume **no** → edit only when Idle).  
4. Detection strings for MIUI Permission Monitoring across OEMs (OnePlus/OPPO).  
5. Behavior when another VPN is active (`prepare` tears down vs fails).  
6. Best default: bundled adb vs system-only when both exist (Scrcpy prefers bundled; MVP_SPEC requires path detect + picker).  
7. macOS USB/adb quirks — best-effort platform.  
8. Whether `I’ve allowed VPN` should poll notification presence or only relay accept.  
9. Competing adb: expose “restart adb server” or only document.  
10. APK package/version query method for mismatch detection.

---

## 11. Handoff notes

### Desktop Engineer

- Implement session state machine + three-layer model as source of truth for chips.  
- Map every failure to ERROR_UX **codes** in logs.  
- Orchestrate Start sharing like `run`; expose Repair = `tunnel`.  
- Never mark Sharing on start-intent alone.  
- Persist: adb path, port, DNS/routes, last serial, setup-complete flag.  
- Default: do **not** kill adb server on quit.  
- Windows: document firewall allow for relay on first listen failure.  
- Attribution string + licenses in About.

### Design

- Technical utility density; SideQuest-like checklist, not SaaS onboarding.  
- Progressive disclosure: one big Start sharing; Advanced for DNS/port/paths.  
- Semantic chips + text; direction badge always.  
- Interrupted + Repair are first-class, not buried.  
- OEM card is designed, not a generic alert().  
- Logs first-class (contrast Proxyman/Wireshark-light, not hidden “support”).  
- Tray mockups OK for **Later**; don’t block MVP on tray.

### QA scenarios (minimum)

1. Fresh PC, no adb → Choose adb…  
2. Unauthorized → allow RSA → Ready  
3. First Start → Waiting VPN → Allow → Sharing (three layers green)  
4. Deny VPN → `VPN_PERMISSION_DENIED`  
5. Unplug mid-share → Interrupted → replug → Repair tunnel → Sharing  
6. MIUI Permission Monitoring → OEM card (if device available)  
7. Port conflict → `PORT_IN_USE`  
8. Stop / quit confirm while sharing  
9. Two devices → must select one  
10. Copy diagnostics produces serial + layers + last code  

---

## 12. Consistency checklist (docs)

| Concept | Canonical term |
|---------|----------------|
| Primary start | Start sharing |
| Primary stop | Stop sharing |
| Tunnel recovery | Repair tunnel |
| Active success | Sharing |
| USB/tunnel break | Interrupted |
| Direction | Internet: This PC → Phone |
| Layers | Relay · Tunnel · Device VPN |
| Android consent | VPN permission (system) |
| Helper | Helper app / gnirehtet client APK |
| Not used | Kill switch, BLOCKING INTERNET, Connected (alone) |
