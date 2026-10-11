# UX Research — gnirehtet Desktop GUI

**Product:** Open-source desktop front-end for [Genymobile gnirehtet](https://github.com/Genymobile/gnirehtet) (Android reverse tethering over adb, no root).  
**Stack (provisional):** Tauri + Svelte, Rust relay sidecar, reuse upstream Android VpnService APK.  
**Scope note:** Expand UX vision freely; label **MVP** vs **Later** everywhere it matters. Authoritative functional scope: MVP_SPEC.

---

## 1. Goals & non-goals

### Goals

| Goal | MVP? | Notes |
|------|------|-------|
| One-click reverse tether for a single authorized USB device | **MVP** | `Run` = install-if-needed + tunnel + start + relay |
| Make adb/device/APK/relay state visible without CLI | **MVP** | Device row + status chip + logs |
| Honest about on-device steps the PC cannot perform | **MVP** | VPN permission; vendor permission monitoring |
| Progressive disclosure of DNS/routes/port/paths | **MVP** | Advanced drawer / Settings |
| Recover from USB unplug (tunnel lost) without rediscovering the product | **MVP** | Reset tunnel + reconnect guidance |
| Multi-device simultaneous sessions | Later | Select-which + autorun concept only in docs |
| System tray always-on agent | Later | MVP: close confirms if tethering; taskbar minimize OK |
| Wireless-adb as primary path | Later | Document as secondary; not happy path |

### Non-goals

- Marketing this as a consumer “VPN app” (wrong mental model; opposite direction of typical VPN).
- Auto-downloading Android platform-tools (**MVP** forbids).
- Play Store client distribution (**out of MVP** / product decision).
- Pretty bandwidth charts, onboarding gamification, account walls, telemetry dashboards.
- Replacing gnirehtet protocol/APK — wrap and orchestrate; attribute Genymobile.

---

## 2. Personas

### P1 — Developer / power user (primary)

| Attribute | Detail |
|-----------|--------|
| Context | Local Android debugging, CI-adjacent device work, captive portals, hotel Wi‑Fi, lab networks where phone has no usable net |
| Skills | Comfortable with USB debugging, adb, logs; may already use scrcpy / Android Studio |
| Needs | Fast Start/Stop, serial-visible device list, copyable logs, reset tunnel, optional DNS/routes, predictable state machine |
| Friction today | Memorizing `gnirehtet run`, juggling relay + reverse + APK versions, Xiaomi permission monitoring |
| Success | Launch → device authorized → Run → key icon on phone → apps use PC network → Stop cleans up |

### P2 — Occasional reverse-tether user (secondary)

| Attribute | Detail |
|-----------|--------|
| Context | Needs phone internet via laptop occasionally (travel, broken mobile data, shared PC Wi‑Fi) |
| Skills | May have enabled USB debugging once; does not live in terminals |
| Needs | Checklist first-run, plain language (“Share PC network”), clear “do this on the phone” steps, calm errors |
| Friction today | CLI-only upstream; RSA authorize; VPN popup misunderstood as “installing a VPN” |
| Success | Guided setup once; thereafter one primary button and readable status |

**Design implication:** Default UI serves P2 copy and P1 density (device serial, log strip, advanced settings) without separate “modes.”

---

## 3. Competitive / inspirational analysis

| Product | Role | What to learn | What not to copy |
|---------|------|---------------|------------------|
| **gnirehtet CLI** | Upstream | Commands (`run`, `autorun`, `install`, `start`, `stop`, `tunnel`, `relay`, `reinstall`); VPN permission; key icon; API 21+; computer-controlled | No UI; multi-device requires serial; users must discover `tunnel` after unplug |
| **scrcpy** | Sibling Genymobile tool | Device by serial; unauthorized/offline; USB debugging first-run; RSA “Always allow” | Mirroring UX; not a network-share product |
| **QtScrcpy** | GUI wrapper pattern | Auto-refresh devices, nicknames, double-click connect, multi-window, tray Show/Quit, first tray notify, log pane, start/stop/stop-all | Multi-device windows (**Later**); tray (**Later**) |
| **SideQuest** | First-run education | Guided checklist: developer options → USB debugging → RSA → data cable; green when authorized | Store/sideload product chrome; VR-specific copy |
| **Tailscale / Mullvad-style clients** | Status-first VPN UX | Connected / Connecting / Disconnected chips; one-click connect; diagnostics escape hatch | Typical VPN = device→internet via provider; **ours is PC→phone** — map labels carefully |
| **Tuxscale** | CLI wrapper + tray | Wrap orchestrator; auto-refresh status; tray as secondary surface | Full mesh product identity |
| **Wireshark / Proxyman** | Technical utility chrome | Logs/packets first-class; dense but scannable; no SaaS onboarding | Packet-level UI overkill for MVP |
| **Android Studio Device Manager** | Device list semantics | Serial, state, refresh; clear offline/unauthorized | IDE weight; multi-pane complexity |

Sources: [gnirehtet README](https://github.com/Genymobile/gnirehtet), Medium reverse-tether intros, QtScrcpy UI patterns, SideQuest setup guides, Tailscale/Mullvad client UX, Android Studio Device Manager.

---

## 4. Transferable patterns (why they fit reverse tether)

| Pattern | Source | Why it fits |
|---------|--------|-------------|
| Device list with serial + state chips | scrcpy / AS Device Manager | adb is the source of truth; unauthorized ≠ missing |
| Auto-refresh device list | QtScrcpy | USB plug events are the common path |
| First-run checklist (green when OK) | SideQuest | Three external gates: adb binary, authorize, later VPN on phone |
| One primary Connect / Disconnect | Tailscale-class | Maps to `Run` / `Stop`; status is the product |
| Collapsible log pane + copy | QtScrcpy / Wireshark-light | Relay/tunnel failures are opaque without logs |
| Progressive disclosure (Advanced) | Technical utilities | DNS/routes/port/adb path are power features |
| Explicit “do this on device” interstitial | gnirehtet reality | Desktop cannot grant VPN or disable MIUI permission monitoring |
| Reset tunnel as named action | gnirehtet `#47` / `tunnel` | Unplug kills `adb reverse`; users need a noun in the UI |
| Tray Connected/Disconnected | Tuxscale (**Later**) | Good for long sessions; not required for MVP |

---

## 5. Anti-patterns to avoid

| Anti-pattern | Why harmful here |
|--------------|------------------|
| SaaS-style multi-step marketing onboarding | Users need adb + cable, not a tour of value props |
| Account / cloud walls | Offline local tool; trust via open source + Genymobile attribution |
| Calling the product a “VPN” in chrome | Confuses with outbound VPN; use “Reverse tether” / “Share PC network” |
| Hiding logs behind “support only” | Failures are environmental (adb, OEM, USB); logs are primary recovery |
| Feature dump on first paint | DNS, routes, IPv6, multi-device overwhelm P2 |
| Silent auto-download of platform-tools | **MVP** forbids; consent + path picker instead |
| Fake “Connected” before VPN grant | Must show **Waiting for VPN permission** |
| Blaming the user (“You failed to…”) | OEM permission monitoring and charge-only cables are common |

---

## 6. Mental model (text diagram)

```
┌─────────────┐     ┌──────────────┐     ┌─────────────────┐     ┌──────────────────┐     ┌─────────────┐
│  PC network │ ──► │ Relay server │ ──► │ adb reverse     │ ──► │ Device VpnService│ ──► │ Phone apps  │
│  (Wi‑Fi/eth)│     │ (port 31416) │     │ (host←device)   │     │ (captures traffic)│     │ (IPv4 TCP/UDP)│
└─────────────┘     └──────────────┘     └─────────────────┘     └──────────────────┘     └─────────────┘
                           ▲                      ▲                         ▲
                           │                      │                         │
                     Desktop GUI            Unplug kills            Key icon in status bar
                     starts/stops           reverse; need           when tether active
                     relay + intents        `tunnel` reset          First start: VPN popup
```

**User-facing one-liner:** “This app shares your computer’s network with your phone over USB. Android asks for VPN permission because that is how the phone captures traffic — you are not joining a commercial VPN.”

**Direction reminder for copywriters:** Typical VPN client = phone traffic exits via remote. This product = phone traffic exits via **this PC**.

---

## 7. Terminology dictionary

| User-facing (UI) | Internal / CLI / docs | Notes |
|------------------|----------------------|-------|
| Reverse tether / Share PC network | gnirehtet session | Prefer over “VPN connection” |
| Start sharing | `gnirehtet run` (orchestrated) | Primary button; install-if-needed + tunnel + start + relay |
| Stop sharing | `stop` + stop relay | Clean teardown |
| Repair tunnel | `tunnel` | Restores `adb reverse` after unplug; primary recovery |
| Relay · Tunnel · Device VPN | relay / reverse / VpnService | Three-layer status; claim Sharing only after handshake |
| Install helper / Reinstall helper | `install` / `reinstall` | APK on device |
| Waiting for VPN permission | First `start` → system VPN dialog | On-device only; not Sharing yet |
| Sharing / Interrupted | Active session / tunnel lost | Prefer over “Connected” / kill-switch language |
| Device unauthorized | adb `unauthorized` | RSA fingerprint prompt on phone |
| Permission monitoring (OEM) | `WRITE_SECURE_SETTINGS` / MIUI etc. | Issues #5, #302, #566 — guide, don’t auto-fix |
| Autorun | `autorun` | **Later** multi-device / reconnect stories |
| DNS / Routes | gnirehtet DNS & route options | Advanced; optional |

Avoid in marketing chrome: “Secure your privacy,” “Browse anonymously,” “VPN subscription.”

---

## 8. Information hierarchy

**Primary (always visible on Main):**

1. Selected device identity (name/model if known + serial + adb state)
2. Session state chip (Idle / Starting / Waiting for VPN… / Sharing / Interrupted / Stopping / Error)
3. Primary action: **Start sharing** or **Stop sharing** (context-sensitive)
4. One-line status explanation (“Waiting for VPN permission on the phone”)

**Secondary (same window):**

5. Device list (MVP: list all; only one session active)
6. Log strip (collapsible, copyable) — last N lines + “Open diagnostics”
7. Quick recovery: **Reset tunnel**, **Refresh devices**

**Tertiary (Settings / Advanced):**

8. adb path, relay port, DNS, routes, APK path, persist preferences

**Quaternary (About):**

9. Genymobile gnirehtet attribution, licenses, version (GUI + APK + relay)

---

## 9. Accessibility & platform notes

| Topic | Guidance |
|-------|----------|
| Platforms | **Win / Linux primary**; **macOS best-effort** (adb/USB quirks; needs device validation) |
| Contrast | Semantic status colors + text label (not color alone); WCAG AA for chips on dark/light |
| Keyboard | Refresh devices, Run/Stop, focus log, copy selected log; Settings reachable without tray |
| Screen readers | State chip `aria-live` polite on transitions; VPN-waiting announcement important |
| Motion | No decorative animation; indeterminate progress only while Starting / Stopping |
| High DPI | Tauri/Svelte: scalable type; target ~900×640 usable, ~1280×800 comfortable |
| Notifications | OS notify sparingly (**MVP**: VPN granted→Tethering; unexpected disconnect). Tray notify first-time = **Later** |

---

## 10. Citations

| Topic | URL / reference |
|-------|-----------------|
| gnirehtet upstream (no UI, run/autorun, VPN permission, key icon, API 21+) | https://github.com/Genymobile/gnirehtet |
| Architecture (VPN client + relay; NAT-like; TCP/UDP IPv4) | Medium intros to gnirehtet; README architecture notes |
| OEM permission monitoring / WRITE_SECURE_SETTINGS | GitHub issues [#5](https://github.com/Genymobile/gnirehtet/issues/5), [#302](https://github.com/Genymobile/gnirehtet/issues/302), [#566](https://github.com/Genymobile/gnirehtet/issues/566) |
| USB unplug kills reverse; tunnel reset | GitHub issue [#47](https://github.com/Genymobile/gnirehtet/issues/47); CLI `tunnel` |
| APK version mismatch | Community reports / install-reinstall practice — treat as first-class error |
| scrcpy device auth UX | https://github.com/Genymobile/scrcpy |
| QtScrcpy tray, refresh, logs, multi-device | QtScrcpy project UI conventions |
| SideQuest USB checklist | SideQuest setup documentation (developer options, RSA, data cable) |
| Status-first VPN clients | Tailscale, Mullvad, Tuxscale client UX (labels remapped for reverse tether) |

When upstream behavior is ambiguous (exact reconnect timing, OEM dialogs), UI copy must say **needs device validation** rather than inventing CLI semantics.

---

## 11. Research → product principles (summary)

1. Technical desktop utility, not generic SaaS.  
2. Expose power without requiring typed adb commands.  
3. Progressive disclosure: one big **Run**, advanced for DNS/routes/port/paths.  
4. Honest about on-device steps.  
5. Visible state machine: Idle | Starting | Waiting for VPN permission | Tethering | Degraded/Reconnect | Stopping | Error.  
6. Device-centric layout; wizard only for first-run.  
7. Logs first-class.  
8. Terminology: Reverse tether / Share PC network / Relay — VPN permission explained as Android’s capture mechanism.

---

## Addendum — research fold-in (2026-09-12)

Sources: `/workspace/gnirehtet-ux/research/{tools-comparison,pain-points,pattern-library}.md`

### Three-layer status (Relay · Tunnel · Device VPN)

UI must never collapse health into a single vague “Connected.” Always expose:

| Layer | Meaning | Healthy signal | Broken signal |
|-------|---------|----------------|---------------|
| **Relay** | PC process listening (default port **31416**) | `Relay listening` | Not listening / crashed / port in use |
| **Tunnel** | `adb reverse` (localabstract→tcp) | `Tunnel OK` | Dead after unplug; missing from `adb reverse --list` |
| **Device VPN** | Android VpnService client + handshake | `VPN active` / client id received | Waiting consent / denied / OEM block / fake connect |

**Rule:** TCP reverse can look “up” while relay is down — wait for **client id** handshake before claiming Sharing (DEVELOP.md; Scrcpy GUI “don’t report success early”).

**Status line pattern:** `Relay listening on 31416 · Tunnel OK · VPN active on {device}`  
**Idle-but-relay-up:** `Relay: listening · Devices sharing: 0` (do not call this Sharing).

### Direction copy (mandatory chrome)

Always-visible subtitle / badge: **Internet: This PC → Phone**  
Prevents collision with typical VPN (phone→remote) and classic tether (phone→PC: PdaNet+/EasyTether).

### Primary verbs (aligned with research)

| Prefer in UI | Maps to | Avoid alone |
|--------------|---------|-------------|
| **Start sharing** | orchestrated `run` | “Connect VPN”, “Connect” |
| **Stop sharing** | `stop` + relay teardown | Ambiguous “Disconnect” |
| **Repair tunnel** | `tunnel` | Kill-switch / “Reconnect VPN” |
| **Reinstall helper** | `reinstall` / uninstall+install | Asking users for `adb uninstall` |
| **Refresh devices** | `adb devices` poll | — |

“Reset tunnel” in older notes = **Repair tunnel** (same action).

### Session state labels (user-facing)

`Idle` · `Starting` · `Waiting for VPN permission` · **`Sharing`** · **`Interrupted`** · `Stopping` · `Error`  
(`Interrupted` replaces vague “Degraded”; means was sharing, tunnel/USB path broke.)

### OEM / MIUI Permission Monitoring (P0)

When `am start` returns 255 / `WRITE_SECURE_SETTINGS` / SecurityException ([#566](https://github.com/Genymobile/gnirehtet/issues/566)):

- Dedicated error card — not a generic “client start failed”
- Steps: **Disable Permission Monitoring**; enable **Install via USB** / **USB debugging (Security settings)** (MIUI/Xiaomi/Redmi/POCO first; OnePlus/OPPO similar reports)
- Offer **Launch helper manually** + Retry

### Kill-switch language — do not import

Mullvad’s **BLOCKING INTERNET** / lockdown = blocks **PC** traffic. Wrong here.  
Use **Sharing interrupted** / **Phone offline (path broken)** — never imply this app blocks PC internet.

### Competitive additions

| Tool | Takeaway |
|------|----------|
| **Wirebound** | Closest GUI competitor; Dashboard+Logs+DNS; wants better first-run/diagnostics |
| **Phone-Net** | Peer Start/Stop; clean-shutdown trust copy |
| **re-Link** | No-ADB commercial reverse tether; we stay honest about ADB requirement; watch driver conflicts |
| **Scrcpy GUI** | Unauthorized cards; never success-before-ready; bundled ADB; competing-ADB caution |

### Extra pain modes to plan for (see ERROR_UX)

- Competing ADB server version mismatch  
- VPN dialog hang / never appears ([#515](https://github.com/Genymobile/gnirehtet/issues/515))  
- Another VPN already active on phone  
- “Sharing” but apps report no internet ([#491](https://github.com/Genymobile/gnirehtet/issues/491), [#567](https://github.com/Genymobile/gnirehtet/issues/567)) — Wi‑Fi radio tip  
- Quest/headset reinstall loop ([#577](https://github.com/Genymobile/gnirehtet/issues/577), [#578](https://github.com/Genymobile/gnirehtet/issues/578)) — **Later**/niche unless validated  
- Firewall blocking relay port; PC offline preflight  
