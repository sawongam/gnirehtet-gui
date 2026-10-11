# Screen Spec — gnirehtet Desktop GUI

Concrete layout and component spec for implementers (Tauri + Svelte provisional).  
**MVP** vs **Later** labeled. Cross-ref: `UX_RESEARCH.md`, `USER_FLOWS.md`, `ERROR_UX.md`, `MVP_UX.md`.

---

## 1. Navigation model

| Surface | MVP? | Notes |
|---------|------|-------|
| **Main** (single primary window) | **MVP** | Devices + session + actions + log strip |
| **Setup / First-run** | **MVP** | Modal or full-panel checklist; skippable after complete with warnings |
| **Settings / Advanced** | **MVP** | Separate window or drawer/panel |
| **Diagnostics** | **MVP** | Panel/sheet from Main or Settings |
| **About / Licenses** | **MVP** | Dialog |
| **System tray** | **Later** | Spec included; not required to ship |

**MVP window chrome:** title + always-visible direction badge **`Internet: This PC → Phone`**.  
**MVP close behavior:** If session active (`Starting`/`WaitingVpn`/`Sharing`/`Interrupted`/`Stopping`) → confirm “Stop sharing and quit?”; else quit. Minimize to taskbar OK. **No tray required.**

---

## 2. Global status chip taxonomy

Semantic colors + **text** (not color alone). Map carefully — this is reverse share, not PC VPN.

| Chip ID | Label | Color intent | When |
|---------|-------|--------------|------|
| `idle` | Idle | Neutral/grey | No active share intent |
| `starting` | Starting | Amber | Install / relay / tunnel / start in progress |
| `waiting_vpn` | Waiting for VPN permission | Amber | On-device consent pending |
| `sharing` | Sharing | Green | Relay + Tunnel + Device VPN healthy (+ handshake) |
| `interrupted` | Interrupted | Red/alert | Was sharing; USB/tunnel lost — **not** “PC blocked” |
| `stopping` | Stopping | Amber | Teardown |
| `error` | Error | Red | Blocking failure (see ERROR_UX codes) |
| `device_ready` | Ready | Green-muted | adb `device` |
| `device_unauth` | Unauthorized | Amber | adb `unauthorized` |
| `device_offline` | Offline | Grey/red-muted | adb `offline` |
| `relay_on` | Relay listening | Info/blue-grey | Layer chip |
| `tunnel_ok` | Tunnel OK | Info | Layer chip |
| `tunnel_bad` | Tunnel lost | Alert | Layer chip |
| `vpn_on` | Device VPN on | Info | Layer chip |
| `vpn_wait` | VPN pending | Amber | Layer chip |

**Never use:** “Connected” alone, “Protected”, “BLOCKING INTERNET”, “Kill switch”.

---

## 3. Three-layer status strip (required on Main)

Compact strip under title / above actions:

```
Internet: This PC → Phone
┌─────────────┬─────────────┬──────────────────┐
│ Relay       │ Tunnel      │ Device VPN       │
│ Listening   │ OK / Lost   │ Active / Waiting │
│ :31416      │             │ / Off            │
└─────────────┴─────────────┴──────────────────┘
```

| Layer | Populated | Empty / off | Error |
|-------|-----------|-------------|-------|
| Relay | Listening + port | Off | Port in use / crashed |
| Tunnel | OK (serial) | — | Lost / failed |
| Device VPN | Active | Off | Pending / denied / OEM |

Partial: `Relay listening · Devices sharing: 0` is **not** Sharing.

---

## 4. Screen: First-run / Setup checklist **[MVP]**

**Purpose:** Clear environmental gates before first Start sharing.

### Layout zones

1. Header: product name + direction line  
2. Checklist body (pass/fail/pending icons)  
3. Footer: Continue / Open guide / Choose adb…

### Checklist items

| # | Item | Pass criteria |
|---|------|---------------|
| 1 | ADB available | Bundled or custom path works (`adb version`) — **MVP:** detect only; **no auto-download** platform-tools |
| 2 | USB debugging enabled | User attestation + device appears |
| 3 | Computer authorized | adb state `device` (not `unauthorized`) |
| 4 | Data cable | Guidance only |
| 5 | Heads-up: VPN permission | Informational — “Phone will ask to allow a VPN connection; that is how Android captures traffic for reverse tether.” |

OEM note link (Xiaomi/MIUI): Permission Monitoring / Install via USB — not blocking first-run unless start already failed.

### Variants

| Variant | Content |
|---------|---------|
| Empty/loading | “Checking ADB…” |
| ADB missing | `ADB_MISSING` actions: Choose adb… · Guide |
| No devices | Checklist item 2–3 amber + empty-state tips |
| Ready | All critical green → Continue enabled |

### Actions

- **Primary:** Continue to Main  
- **Secondary:** Choose adb… · Copy diagnostics · Skip (with banner on Main)

### Shortcuts

`Enter` Continue when enabled; `Esc` skip/dismiss if allowed.

---

## 5. Screen: Main **[MVP]**

**Purpose:** Device-centric reverse-tether control — primary daily surface.

### Layout zones (~900×640 priority)

```
+------------------------------------------------------------------+
| App · Internet: This PC → Phone                    [⚙] [Diagnostics]|
+------------------------------------------------------------------+
| Relay: … | Tunnel: … | Device VPN: …     SESSION CHIP: Sharing     |
+------------------------------------------------------------------+
| DEVICES                         | SESSION / SELECTED DEVICE        |
| [Refresh]                       | Model · Serial · ADB state       |
| ○ Pixel 7  Ready     serial…    | Phase: USB→Auth→Helper→VPN→Share |
| ○ … Unauthorized                |                                  |
|                                 | [ Start sharing ] or [ Stop ]    |
|                                 | [ Repair tunnel ] [ Reinstall ]  |
|                                 | One-line status explanation      |
+------------------------------------------------------------------+
| LOGS ▾  (collapsible)                              [Copy] [Clear]  |
| … last lines …                                                   |
+------------------------------------------------------------------+
```

**~1280×800:** Wider device list; log pane taller or side tab; three-layer strip with more detail (bytes **Later**).

### Components

| Component | Behavior |
|-----------|----------|
| Device list | Auto-refresh with hysteresis (avoid flicker — SideQuest lesson); manual Refresh |
| Device row | Model/product if known, serial, state chip; click selects |
| Primary CTA | **Start sharing** when Idle/Error/Interrupted(recoverable); **Stop sharing** when Starting/WaitingVpn/Sharing/Interrupted |
| Repair tunnel | Enabled when device authorized and (Interrupted OR Sharing OR Idle-with-stale-tunnel); maps to `tunnel` |
| Reinstall helper | Secondary/overflow — APK reinstall |
| Log strip | Live, collapsible, monospace, copyable; first-class |
| Settings gear | Opens Settings |
| Direction badge | Always visible |

### Main variants

| Variant | UI |
|---------|----|
| Empty devices | See empty-state copy below; Start disabled |
| Loading devices | Skeleton/spinner on list; “Looking for devices…” |
| Populated | Rows + selection |
| Waiting VPN | Session chip + coach panel (do not claim Sharing) |
| Sharing | Green chip; three layers green; Stop primary |
| Interrupted | Alert banner: “Sharing interrupted — USB reverse tunnel lost.” Primary **Repair tunnel** |
| Error | Inline error card with ERROR_UX code + recovery actions |

### Empty states (concrete)

| Condition | Title | Body | Actions |
|-----------|-------|------|---------|
| No devices | No Android devices detected | 1) Data-capable USB cable 2) Developer options → USB debugging 3) Unlock & accept Allow USB debugging 4) Try another port (avoid unpowered hubs) | Refresh · Open setup · Copy `adb devices` |
| Unauthorized only | Waiting for USB authorization | Unlock the phone and tap **Allow**. Check **Always allow from this computer**. | I’ve allowed it (refresh) · Setup guide |
| Multiple, none selected | Select a device | MVP supports one active share session. Select which phone to use. | — |
| ADB missing | ADB not found | This app needs Android `adb`. Choose your platform-tools `adb` binary. We do not download it automatically. | Choose adb… · Setup |

### Primary / secondary actions

| State | Primary | Secondary |
|-------|---------|-------------|
| Ready + Idle | Start sharing | Refresh, Settings |
| WaitingVpn | Cancel / Stop sharing | I’ve allowed VPN |
| Sharing | Stop sharing | Repair tunnel, Logs |
| Interrupted | Repair tunnel | Restart sharing, Stop |
| Error | Retry / code-specific | Copy error, Diagnostics |

### Keyboard **[MVP]**

| Key | Action |
|-----|--------|
| `F5` / `Ctrl+R` | Refresh devices |
| `Ctrl+Enter` | Start / Stop sharing (context) |
| `Ctrl+Shift+T` | Repair tunnel |
| `Ctrl+L` | Focus / expand logs |
| `Ctrl+,` | Settings |

### Data fields shown

Selected: model, serial, adb state, helper APK version (if known), DNS summary (if non-default), relay port.

---

## 6. Device detail / row states **[MVP]**

Rows are the detail surface (no separate multi-page wizard). Optional expand row **Later**.

| adb / session | Row badge | Row helper text |
|---------------|-----------|-----------------|
| (none) | — | Empty list |
| `unauthorized` | Unauthorized | “Accept the prompt on the phone” |
| `offline` | Offline | “Replug or wake device” |
| `device` + Idle | Ready | “Ready to share PC network” |
| Installing | Installing helper | — |
| WaitingVpn | Waiting for VPN | “Check the phone” |
| Sharing | Sharing | Key icon reminder |
| Interrupted | Interrupted | “Repair tunnel” |
| Error | Error | Short code title |

**MVP:** single selection highlight. **Later:** multi-share indicators, nicknames, double-click start (QtScrcpy).

---

## 7. Screen: Advanced / Settings **[MVP]**

**Purpose:** Progressive disclosure — paths, port, DNS/routes, behavior toggles.

### Sections

| Section | Fields | MVP? |
|---------|--------|------|
| ADB | Bundled vs Custom path; Browse; “Test adb” | **MVP** (custom path + detect; bundled if product ships it — path picker mandatory per MVP_SPEC no auto-download) |
| Relay | Port (default 31416) | **MVP** |
| DNS | Presets: Google / Cloudflare / Custom; optional routes | **MVP** optional DNS/routes |
| Helper APK | Path / bundled; Reinstall action | **MVP** |
| Behavior | Auto-refresh interval; “Stop ADB on quit” default **off** | **MVP** refresh; stop-adb **Later**/advanced |
| Auto-share new devices | = autorun | **Later** |
| Tray / launch on login / start minimized | — | **Later** |
| Theme / language | — | **Later** |

### Idle vs Sharing

While `Sharing`/`Starting`/`Stopping`: disable port, adb path, DNS edits (or mark “applies on next Start”). See USER_FLOWS §12.

### Actions

Save (persist) · Restore defaults · Open Diagnostics

---

## 8. Screen: Diagnostics **[MVP]**

**Purpose:** Power-user escape hatch — tool-up vs path-broken (Proxyman lesson).

### Doctor steps (pass/fail)

1. ADB binary OK  
2. Target device state = `device`  
3. Relay port listening  
4. `adb reverse` contains gnirehtet / 31416  
5. Helper package installed (+ version)  
6. Last error code + log snippet  
7. Tips: Wi‑Fi radio on without joining; other VPN disconnected; OEM Permission Monitoring  

**Later:** HTTP connectivity test through path; PC-offline preflight.

### Actions

Copy diagnostics report · Export logs · Repair tunnel · Refresh

---

## 9. Screen: About / Licenses **[MVP]**

- App name + version (GUI)  
- Relay / engine version; APK version  
- **Attribution:** Genymobile gnirehtet — link https://github.com/Genymobile/gnirehtet  
- License texts (app + upstream + dependencies)  
- “Not affiliated with Genymobile” if required by project policy (**needs legal validation**)

---

## 10. System tray **[Later]** (specify now)

| Item | Spec |
|------|------|
| Icon colors | Grey idle · Amber starting/waiting · Green sharing · Red interrupted/error |
| Menu | Open dashboard · Start/Stop sharing · Repair tunnel · Show logs · Preferences · Quit |
| Notifications | Critical only: tunnel lost, VPN denied; mute non-critical |
| Close-to-tray | Optional; Quit stops sharing cleanly with confirm |
| Kill-switch | **Do not implement / do not label** |

**MVP substitute:** taskbar minimize; close quits (with confirm if sharing).

---

## 11. Waiting-for-VPN panel **[MVP]**

Modal or anchored panel on Main:

- Title: **Allow VPN permission on your phone**  
- Body: Explains Android VpnService is the capture mechanism; **Internet still: This PC → Phone**; not a commercial VPN.  
- Visual: simple mock of system “Connection request”  
- Reminder: key icon appears in status bar when active (README)  
- Actions: **I’ve allowed it** (re-check) · **Stop sharing** · Open recovery (revoke VPN / battery tip)  
- Timeout → `VPN_PERMISSION_PENDING` / `START_TIMEOUT`

---

## 12. OEM Permission Monitoring card **[MVP]**

Triggered by `VENDOR_PERMISSION_MONITORING`:

- Title: **Phone blocked starting the helper**  
- Body: OEM Permission Monitoring / restricted shell (common on Xiaomi/MIUI).  
- Steps: Developer options → **Disable Permission Monitoring**; enable **Install via USB** and **USB debugging (Security settings)** if present.  
- Actions: Retry · Launch helper manually (instructions) · Copy log excerpt · Open issue ref (#566)

---

## 13. Interrupted banner **[MVP]**

- Title: **Sharing interrupted**  
- Body: Unplugging USB kills the `adb reverse` tunnel. Replug the phone, then repair.  
- Primary: **Repair tunnel**  
- Secondary: Restart sharing · Stop sharing  
- Three-layer strip: Tunnel = Lost (even if Relay still listening)

---

## 14. Content priority by size

### ~900×640

1. Direction badge + session chip  
2. Three-layer strip (compact)  
3. Selected device + Start/Stop  
4. Device list (scroll)  
5. Collapsed log (1–3 lines) + expand  

### ~1280×800

+ Full log pane  
+ DNS/port summary  
+ Phase strip  
+ Side-by-side device list + detail  

---

## 15. Accessibility notes

- Chips: icon + text  
- `aria-live` on session chip and Waiting VPN  
- Focus order: devices → primary CTA → repair → logs → settings  
- No information by color alone  

---

## 16. Copy snippets (Main)

| Slot | Copy |
|------|------|
| Subtitle | Internet: This PC → Phone |
| Start | Start sharing |
| Stop | Stop sharing |
| Repair | Repair tunnel |
| Sharing help | Phone apps use this PC’s network. Look for the key icon in the phone status bar. |
| Relay-only | Relay is listening, but no phone is sharing yet. |
