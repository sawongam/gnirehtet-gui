# Error UX — gnirehtet Desktop GUI

Calm, specific, no blame. In-app actions over “contact support.”  
Codes are **stable** for logs, diagnostics export, and tests.  
Cross-ref: `USER_FLOWS.md`, `SCREEN_SPEC.md`, `MVP_UX.md`.  
Evidence: `/workspace/gnirehtet-ux/research/pain-points.md`.

---

## 1. Tone & presentation rules

| Do | Don’t |
|----|-------|
| Name the broken layer: Relay · Tunnel · Device VPN | Say “Connected” / “VPN failed” vaguely |
| Direction: Internet is PC → Phone | Imply PC kill-switch / “BLOCKING INTERNET” |
| Action links: Choose adb…, Repair tunnel, Open OEM guide, Copy error | Blame (“You forgot…”) |
| Show last log snippet when command failed | Dump full log in the title |
| Keep session chip = Error or Interrupted | Fake green Sharing |

**Where to show:** inline card on Main (primary); Diagnostics for detail; OS notification only per policy below.

**Log snippet:** show when a subprocess/adb command failed or handshake timed out. Hide for purely instructional states (`VPN_PERMISSION_PENDING` can offer “Show logs” collapsed).

---

## 2. Notification policy

| Event | OS notification **[MVP]** | In-app |
|-------|---------------------------|--------|
| Waiting for VPN | No (user should be looking at phone + window) | Blocking panel |
| Sharing started (VPN granted) | Optional quiet once | Chip change |
| Sharing interrupted (unplug) | **Yes** if window unfocused | Banner + Repair tunnel |
| Relay crashed | **Yes** | Error card |
| Unauthorized / no devices | No | Empty/row state |
| OEM / install errors | No | Card |
| Tray first-run balloon | **Later** | — |

Never notify “PC internet blocked.”

---

## 3. Taxonomy

Columns: **Code** · **Title** · **Explanation** · **Recovery** · **Logs?** · **Layer** · **MVP**

### ADB / environment

| Code | Title | Explanation | Recovery actions | Logs? | Layer |
|------|-------|-------------|------------------|-------|-------|
| `ADB_MISSING` | ADB not found | Reverse tether needs the Android Debug Bridge (`adb`). It isn’t on the configured path. This app does not download platform-tools automatically. | **Choose adb…** · Open setup guide · Copy error | Short | — |
| `ADB_PATH_INVALID` | ADB path invalid | The saved `adb` path isn’t a working binary (`adb version` failed). | **Choose adb…** · Clear path / use default · Copy error | Yes | — |
| `ADB_SERVER_MISMATCH` | ADB server conflict | Another tool (Android Studio, Vysor, SideQuest) may be running a different `adb` server. Devices can flicker or disappear. | Switch to the same adb those tools use · **Retry** · Advanced: restart adb server (warn: other tools) | Yes | — |

`ADB_SERVER_MISMATCH` **[MVP if detectable; else Later]** — do **not** kill shared adb on quit by default.

---

### Devices

| Code | Title | Explanation | Recovery actions | Logs? | Layer |
|------|-------|-------------|------------------|-------|-------|
| `NO_DEVICES` | No Android devices detected | ADB is running but no phones/tablets appear. Often a charge-only cable, USB debugging off, or a bad port. | Refresh · Open setup checklist · Copy `adb devices` | Optional | — |
| `DEVICE_UNAUTHORIZED` | Waiting for USB authorization | The phone is connected but hasn’t allowed this computer. Unlock it and accept **Allow USB debugging?** | I’ve allowed it (refresh) · Setup guide · Advanced: revoke authorizations on phone, replug | No | — |
| `DEVICE_OFFLINE` | Device offline | ADB sees the serial but the transport is offline (sleep, cable glitch, driver). | Wake/replug · Refresh · Try another cable/port | Optional | Tunnel |
| `MULTIPLE_DEVICES_NO_SELECTION` | Select a device | More than one device is connected. This version shares with **one** device at a time. | Select a row · Refresh | No | — |

---

### APK / helper

| Code | Title | Explanation | Recovery actions | Logs? | Layer |
|------|-------|-------------|------------------|-------|-------|
| `APK_MISSING` | Helper APK not found | The Android helper (gnirehtet client) isn’t bundled or the configured path is empty. | Choose APK… · Reinstall from Settings · Copy error | Yes | Device VPN |
| `INSTALL_FAILED` | Couldn’t install helper | `adb install` failed (storage, signature, OEM install-via-USB, etc.). | **Reinstall helper** · Enable Install via USB (MIUI) · Copy log | Yes | Device VPN |
| `APK_VERSION_MISMATCH` | Helper version doesn’t match | Installed APK version doesn’t match this desktop engine. A known failure mode. | **Reinstall helper** · Stop other old clients · Copy versions | Yes | Device VPN |

---

### Relay

| Code | Title | Explanation | Recovery actions | Logs? | Layer |
|------|-------|-------------|------------------|-------|-------|
| `PORT_IN_USE` | Relay port already in use | Nothing new can listen on the configured port (default **31416**). Another relay or leftover process may be bound. | Change port in Settings (next start) · **Stop leftover relay** · Copy error | Yes | Relay |
| `RELAY_START_FAILED` | Relay didn’t start | The PC relay process exited or never listened. Firewall/AV may block it. | Retry Start sharing · Allow app through firewall · Check Diagnostics | Yes | Relay |
| `RELAY_CRASHED` | Relay stopped unexpectedly | Sharing cannot continue without the relay. The phone may still show an old VPN state. | Restart sharing · Stop sharing (clean phone side) · Copy logs | Yes | Relay |
| `ORPHAN_RELAY` | Leftover relay process | A relay is still listening after Stop/Quit failed to clean up. The UI is Idle but port may be busy. | **Stop leftover relay** · Restart app · Copy PID/port | Yes | Relay |

---

### Tunnel

| Code | Title | Explanation | Recovery actions | Logs? | Layer |
|------|-------|-------------|------------------|-------|-------|
| `TUNNEL_FAILED` | Couldn’t create tunnel | `adb reverse` (tunnel) did not set up. Sharing can’t reach the relay. | **Repair tunnel** · Replug USB · Refresh device · Copy log | Yes | Tunnel |
| `TUNNEL_LOST` | Sharing interrupted | Unplugging USB **kills the adb reverse tunnel**. The relay may still be listening; the phone path is broken. | Replug · **Repair tunnel** · Restart sharing · Stop sharing | Yes if unexpected | Tunnel |

`TUNNEL_LOST` uses session state **Interrupted**, not a kill-switch metaphor.

---

### Client / Device VPN / OEM

| Code | Title | Explanation | Recovery actions | Logs? | Layer |
|------|-------|-------------|------------------|-------|-------|
| `CLIENT_START_FAILED` | Couldn’t start helper on phone | The start intent failed. If logs mention Permission Monitoring or `WRITE_SECURE_SETTINGS`, use the OEM card instead. | Retry · Open helper manually · Diagnostics | Yes | Device VPN |
| `VENDOR_PERMISSION_MONITORING` | Phone blocked the helper start | OEM **Permission Monitoring** (common on Xiaomi/MIUI/Redmi/POCO; also reported on OnePlus/OPPO) blocked `adb shell am start`. Often `SecurityException` / exit **255**. | Disable **Permission Monitoring** in developer options · Enable **Install via USB** / **USB debugging (Security settings)** · Retry · Launch helper manually · Copy log (#566) | Yes | Device VPN |
| `VPN_PERMISSION_PENDING` | Waiting for VPN permission | Android must show a **Connection request**. The desktop cannot tap it. Do not treat this as Sharing yet. | Allow on phone · **I’ve allowed it** · Recovery: forget VPN in Android settings, disable battery optimization, open helper · Stop sharing | Optional | Device VPN |
| `VPN_PERMISSION_DENIED` | VPN permission denied | The phone dismissed or denied the VPN request. Reverse tether uses VpnService only to send traffic through this PC. | Start sharing again and tap Allow · Disconnect other VPN apps · Open helper | Optional | Device VPN |
| `VPN_ALREADY_ACTIVE` | Another VPN is active | Android typically allows one user VPN. Another app may be holding the VPN slot. | Disconnect the other VPN on the phone · Retry | No | Device VPN |
| `CLIENT_HANG_NO_PROMPT` | Helper started but VPN never appeared | Some devices hang after “Starting: Intent…” with no dialog, or show a notification with no internet. | Open helper on phone · Forget/revoke gnirehtet VPN · Disable battery optimization · **Reinstall helper** · Retry | Yes | Device VPN |
| `START_TIMEOUT` | Start timed out | Start sharing didn’t reach a healthy Relay + Tunnel + Device VPN handshake in time. | Retry · I’ve allowed VPN · Diagnostics doctor · Repair tunnel | Yes | Any |
| `STOP_FAILED` | Couldn’t stop cleanly | Stop intent or teardown failed. The phone key icon or relay may linger. | Retry Stop · Stop leftover relay · Reinstall/stop helper on phone | Yes | Any |
| `HANDSHAKE_FAILED` | Phone didn’t complete handshake | A tunnel/TCP connect can succeed even when the relay isn’t really serving. Sharing is only confirmed after the client id/handshake. | Repair tunnel · Restart sharing · Check relay layer | Yes | Relay+VPN |

`VPN_ALREADY_ACTIVE`, `CLIENT_HANG_NO_PROMPT`, `HANDSHAKE_FAILED` — **MVP** if signals exist; else show as `CLIENT_START_FAILED` / `START_TIMEOUT` + tips. **Needs device validation.**

---

### Path looks up, apps say offline **[MVP tip / Later doctor]**

| Code | Title | Explanation | Recovery actions | Logs? | Layer |
|------|-------|-------------|------------------|-------|-------|
| `APPS_REPORT_NO_INTERNET` | Phone apps say they’re offline | Layers may be healthy. Some apps don’t treat a VPN path as “internet,” or DNS is wrong. | Turn **Wi‑Fi on** (no need to join a network) or enable mobile data — traffic can still use the PC · Try DNS preset (Cloudflare/Google) · Browser test · Diagnostics | Optional | Path |
| `PC_OFFLINE` | This PC appears offline | Sharing this computer’s network won’t help if the PC has no uplink. | Fix PC network · Retry | No | Relay/path |
| `FIREWALL_RELAY` | Firewall may be blocking the relay | Local listen failed or traffic never flows; Windows Firewall/AV sometimes prompts once. | Allow the app / adb on private networks · Retry relay | Yes | Relay |

`PC_OFFLINE` / `FIREWALL_RELAY` — **Later** preflight unless cheap to detect.

---

## 4. Code → UI mapping

| Code | Screen treatment |
|------|------------------|
| `ADB_*`, `NO_DEVICES` | Setup + Main empty |
| `DEVICE_UNAUTHORIZED` | Device row + helper |
| `MULTIPLE_DEVICES_NO_SELECTION` | Toast/inline under Start |
| `APK_*` | Error card + Reinstall |
| `PORT_*`, `RELAY_*`, `ORPHAN_RELAY` | Relay layer red + card |
| `TUNNEL_LOST` | **Interrupted** banner; primary **Repair tunnel** |
| `TUNNEL_FAILED` | Error card; primary Repair |
| `VENDOR_PERMISSION_MONITORING` | OEM card (SCREEN_SPEC §12) |
| `VPN_PERMISSION_*` | Waiting-VPN panel |
| `APPS_REPORT_NO_INTERNET` | Diagnostics tip (not a red session chip if layers green) |

---

## 5. Recovery action catalog (in-app)

| Action ID | Label | Effect |
|-----------|-------|--------|
| `choose_adb` | Choose adb… | File picker; persist; re-probe |
| `refresh_devices` | Refresh devices | Poll adb |
| `open_setup` | Open setup checklist | Setup screen |
| `repair_tunnel` | Repair tunnel | CLI `tunnel` |
| `restart_sharing` | Restart sharing | Stop + Start pipeline |
| `stop_sharing` | Stop sharing | Clean teardown |
| `reinstall_helper` | Reinstall helper | `reinstall` / uninstall+install |
| `copy_error` | Copy error | Code + snippet + versions |
| `copy_diagnostics` | Copy diagnostics | Doctor dump |
| `ive_allowed_vpn` | I’ve allowed VPN | Re-check handshake |
| `ive_allowed_usb` | I’ve allowed it | Refresh devices |
| `open_oem_guide` | Open developer options guide | In-app OEM steps |
| `launch_helper_manual` | Launch helper manually | Instructions (cannot remote-click) |
| `stop_orphan_relay` | Stop leftover relay | Kill known sidecar PID |
| `change_port` | Change relay port | Settings focus |

---

## 6. Detection hints (for Desktop Engineer)

Do not invent upstream behavior. Prefer matching **stderr / exit codes**:

| Signal | Prefer code |
|--------|-------------|
| `adb` not executable / not found | `ADB_MISSING` / `ADB_PATH_INVALID` |
| `unauthorized` | `DEVICE_UNAUTHORIZED` |
| `offline` | `DEVICE_OFFLINE` |
| empty `adb devices` | `NO_DEVICES` |
| install fail | `INSTALL_FAILED` |
| version compare fail | `APK_VERSION_MISMATCH` |
| EADDRINUSE / port bind | `PORT_IN_USE` |
| reverse fail | `TUNNEL_FAILED` |
| device drop while sharing | `TUNNEL_LOST` |
| `WRITE_SECURE_SETTINGS` / Permission Monitoring / `am start` **255** + SecurityException | `VENDOR_PERMISSION_MONITORING` |
| VPN prepare denied | `VPN_PERMISSION_DENIED` |
| stuck in WaitingVpn | `VPN_PERMISSION_PENDING` → `START_TIMEOUT` |
| relay process death | `RELAY_CRASHED` |
| relay still up after stop | `ORPHAN_RELAY` |

Handshake-before-success: **needs device validation** (DEVELOP.md client id).

---

## 7. Copy examples (full sentences)

**TUNNEL_LOST**  
Sharing interrupted. Unplugging the USB cable removes the adb reverse tunnel. Replug the phone, then Repair tunnel. The relay may still be running; that does not mean the phone is online.

**VENDOR_PERMISSION_MONITORING**  
The phone blocked starting the helper (OEM Permission Monitoring). On Xiaomi/MIUI this is often Developer options → disable Permission Monitoring, and enable Install via USB / USB debugging (Security settings). Then Retry.

**VPN_PERMISSION_PENDING**  
Allow the Connection request on the phone. Android uses a VPN interface to send traffic through this PC — you are not joining a commercial VPN. Internet direction stays: This PC → Phone.

**APPS_REPORT_NO_INTERNET**  
Some apps ignore a VPN path. Try turning Wi‑Fi on without joining a network, or set DNS to Cloudflare/Google in Settings. This is not a PC kill-switch.

---

## 8. Priority (from research)

| P | Codes |
|---|--------|
| P0 | `DEVICE_UNAUTHORIZED`, `VPN_PERMISSION_*`, `TUNNEL_LOST`, `VENDOR_PERMISSION_MONITORING` |
| P1 | `NO_DEVICES`, `ADB_*`, `APK_*`, `TUNNEL_FAILED`, `CLIENT_START_FAILED`, `APPS_REPORT_NO_INTERNET` |
| P2 | `ORPHAN_RELAY`, `FIREWALL_RELAY`, `PC_OFFLINE`, Quest reinstall loop (`INSTALL`/`CLIENT_HANG`) |
