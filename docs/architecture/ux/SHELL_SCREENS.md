# Shell Screens — gnirehtet-gui (AppShell)

**Audience:** Desktop Engineer  
**Status:** Locked to visual SoT v3 (2026-09-20)  
**Visual SoT:** [`VISUAL_APPROVED_V3.md`](./VISUAL_APPROVED_V3.md) + [`mockups/approved-saas-dashboard.png`](./mockups/approved-saas-dashboard.png)  
**Tokens:** [`DESIGN_TOKENS.md`](./DESIGN_TOKENS.md) · [`THEME.md`](./THEME.md)  
**Behavior SoT:** `EVENT_STATUS_MAP.md`, `ERROR_UX.md`, `COPY_RULES.md`, `MVP_UX.md`, `TEAM_BRIEF.md`

Visual language of the screenshot = **GOAL** for all shipped UI.  
MVP must still support: one device, ADB detect, install APK, start/stop reverse tether, repair tunnel, clear state, errors, logs, graceful quit.  
No networking rewrite — UI talks to existing orchestrator events (`DeviceChanged`, `RelayState`, `LogLine`, `Error`). Rust rewrite is **optional** alignment only, not required.

---

## 1. Information architecture (AppShell)

```
┌──────────────┬─────────────────────────────────────────────────────────┐
│ SIDEBAR 240  │ TOPBAR 72                                               │
│              │  [🔍 Search devices, logs, or settings…]  ☀  [Service…] │
│ Gnirehtet    ├─────────────────────────────────────────────────────────┤
│ Reverse Tet. │ MAIN — Dashboard                                        │
│              │  Greeting                                               │
│ ● Dashboard  │  ┌ Connection hero ─────────────┐ ┌ Devices ──────────┐ │
│   Devices (1)│  │ Connected / tunnel active*   │ │ 1 device (MVP)    │ │
│   Traffic    │  │ PC ── link ── Phone          │ │ actions           │ │
│   Logs       │  │ [Stop tethering]             │ ├───────────────────┤ │
│   Settings   │  └──────────────────────────────┘ │ Install client    │ │
│              │  ┌ Network Traffic — Later ─────┐ │ How it works      │ │
│ [promo card] │  │ placeholder / hide in MVP    │ │ Quick actions**   │ │
│ ADB · v…     │  └──────────────────────────────┘ └───────────────────┘ │
└──────────────┴─────────────────────────────────────────────────────────┘
```

\* Connected / “Internet tunnel active” **only** when EVENT_STATUS_MAP three-layer Sharing healthy.  
\*\* Quick actions only if they map to existing settings (Verbose logs OK; Auto-start relay / Keep ADB = Later unless already supported).

---

## 2. Sidebar (240px)

| Item | MVP | Notes |
|------|-----|-------|
| Dashboard | **Ship** | Primary surface |
| Devices | Ship or same panel | May duplicate dashboard Devices card; **one device only** |
| Traffic | **Later** | Stub page or hide |
| Logs | **Ship** | Log view / route |
| Settings | **Ship** | Existing settings |
| Devices badge “1” | Cosmetic OK | MVP = one device; no multi-device |
| Promo card | Visual OK | Copy aligned to reverse tether |
| ADB footer | Ship | ADB connected / missing + version |

Active nav: Primary-50 wash + Primary-500 icon/text (Lucide).

---

## 3. Topbar (72px)

| Element | MVP |
|---------|-----|
| Search pill | **Later** or decorative non-functional — say so in UI/tooltip if shown |
| Theme toggle | Chrome OK; dark = Later |
| Service status | Map to relay/service truth (e.g. Relay listening → “Service running”); never invent |

---

## 4. Dashboard — MVP cut vs Later

| Surface | MVP | Later |
|---------|-----|-------|
| Greeting | Yes | — |
| **Connection hero** | **Yes** — honest states | — |
| **Devices card** (one device) | **Yes** | Multi-device |
| **Install client CTA** | **Yes** → Install helper / install APK | — |
| **How it works** | **Yes** — static education | — |
| **Quick actions** | Only if mapped: Verbose logs OK | Auto-start relay, Keep ADB unless already supported |
| Network Traffic chart / live MB | — | **Later** (honest zeros/placeholder OK if labeled; no fake data) |
| “View all →” multi-device | — | Later |

---

## 5. Vocabulary map (screenshot → existing)

| Screenshot / SaaS label | Existing / honest meaning |
|-------------------------|---------------------------|
| **Connected** / “Internet tunnel active” | User-facing synonym for **Sharing** chip — **only** when Relay + Tunnel + Device VPN handshake healthy (`EVENT_STATUS_MAP`). Never on intent-sent. |
| **Stop tethering** / Disconnect | **Stop** sharing / stop session |
| **Restart tunnel** | **Repair tunnel** |
| **Connect device** / Run | Start session pipeline (`runSession` / Run) |
| Install client | **Install helper** (APK) |
| Service running | Relay / service listening when true |

Direction chrome remains valid: **Internet: This PC → Phone**.

---

## 6. Connection hero — state honesty

| Chip / truth | Hero treatment | Primary action |
|--------------|----------------|----------------|
| Idle + device ready | Neutral / empty-ready | **Run** / Connect device (primary blue) |
| Idle + no device / ADB missing | Empty “No device connected” | Connect guidance; Run disabled |
| Starting | Progress / disabled | Starting… |
| Waiting for VPN | Warning wash — **not** Connected | I’ve allowed it + Stop |
| **Sharing** (3-layer) | Success wash — **Connected** OK | **Stop tethering** (destructive soft) |
| Interrupted | Warning/error | **Restart tunnel** (= Repair) |
| Error | Error wash + `[CODE]` alert | ERROR_UX recovery |

---

## 7. Devices card (MVP = one)

- Model + serial (mono) + Online/ready/unauthorized badge  
- Actions: Disconnect/Stop (destructive soft), Restart tunnel (Repair), optional overflow  
- Refresh devices available (ghost / secondary)  
- Empty state → Connect device CTA (primary)

---

## 8. Install client · How it works · Quick actions

- **Install client** → existing Install helper IPC  
- **How it works** → static numbered steps; no new networking claims  
- **Quick actions:** switches ON = **blue** (not green). Wire only supported settings; stub/Later otherwise — no silent behavior change

---

## 9. Logs & Settings routes

- **Logs:** ScrollArea mono, Clear, ERROR-friendly; Collapsible or dedicated route  
- **Settings:** Existing surfaces only  
- One alert slot for ERROR_UX (`[CODE]` title) — prefer not stacking banners

---

## 10. Events (no rewrite)

UI consumes existing orchestrator:

- `DeviceChanged`  
- `RelayState`  
- `LogLine`  
- `Error`  

Optional alignment with Tauri SessionController — **not** a Rust networking rewrite.

---

## 11. Explicit non-goals (MVP)

- Live throughput charts with invented numbers  
- Multi-device sharing  
- Functional global search  
- Traffic page as a real analytics product  
- Claiming Connected without three-layer health  
- Indigo-as-primary theme  

---

*Components → `COMPONENT_INVENTORY.md`. Tokens → `DESIGN_TOKENS.md`.*
