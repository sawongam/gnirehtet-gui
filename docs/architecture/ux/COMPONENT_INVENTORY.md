# Component Inventory — gnirehtet-gui (AppShell)

**Audience:** Desktop Engineer  
**Status:** Locked to visual SoT v3 (2026-09-20)  
**Visual SoT:** [`VISUAL_APPROVED_V3.md`](./VISUAL_APPROVED_V3.md) + [`mockups/approved-saas-dashboard.png`](./mockups/approved-saas-dashboard.png)  
**Tokens:** [`DESIGN_TOKENS.md`](./DESIGN_TOKENS.md) · Layout: [`SHELL_SCREENS.md`](./SHELL_SCREENS.md)  
**Design system:** shadcn-svelte + Tailwind + **Lucide** (`DESIGN_SYSTEM.md`)

Does **not** invent: tray, multi-share, live fake charts, networking rewrite.  
Orchestrator IPC and stores stay; chrome restyle only.

---

## 1. Principle

Compose an **AppShell** that matches the approved dashboard. Map every interactive control to existing Run / Stop / Repair / Install / Refresh / Relay commands. Primary brand = **blue `#2563EB`**.

---

## 2. Shell primitives

| Component | shadcn / build | Spec |
|-----------|----------------|------|
| **AppShell** | layout | Sidebar 240 + Topbar 72 + main scroll |
| **Sidebar** | nav + Badge | Dashboard, Devices, Traffic*, Logs, Settings; ADB footer; Lucide icons |
| **Topbar** | Input + Button + Badge | Search pill*, theme icon, service status |
| **ConnectionCard** (hero) | Card + Badge + Button | Honest Connected / Waiting / Idle / Error |
| **DeviceCard** | Card + Badge + Button | One device MVP; select/actions |
| **TrafficCard** | Card (+ chart Later) | **Later** — stub/hide/placeholder labeled |
| **InstallClientCard** | Card + Button | → Install helper |
| **HowItWorks** | Card | Static steps |
| **QuickActions** | Card + Switch | Blue ON; wire only supported settings |
| **AlertSlot** | Alert | Single ERROR_UX `[CODE]` slot |
| **LogsPanel** | Collapsible + ScrollArea | Mono log lines, Clear |

\* Traffic route / Search: Later or non-functional — see `SHELL_SCREENS.md`.

---

## 3. Buttons → existing actions

| Variant (tokens) | Actions |
|------------------|---------|
| Primary (`#2563EB`, h-40, radius 8) | **Run** / Connect device; Repair when Interrupted; code Retry |
| Secondary (white + border) | Repair tunnel (Ready); Install helper / Install client; I’ve allowed it; Refresh |
| Destructive soft (`#FEF2F2` / `#DC2626`) | **Stop** / Stop tethering / Disconnect |
| Ghost / icon | Refresh devices; Clear logs; theme; overflow |

| Screenshot label | Shell / IPC | Variant |
|------------------|-------------|---------|
| Connect device / Run | `runSession` / **Run** | primary |
| Stop tethering / Disconnect | `stopClient` / **Stop** | destructive soft |
| Restart tunnel | **Repair tunnel** | secondary → primary if Interrupted |
| Install client | **Install helper** | secondary |
| Refresh | **Refresh devices** | ghost |
| Start/Stop Relay | Advanced / Settings | outline |
| I’ve allowed it | VPN recheck | secondary |

Busy: disable + “Starting…” / “Stopping…”.

---

## 4. Badges & switches

| Use | Source | Visual |
|-----|--------|--------|
| Connected / Sharing | EVENT_STATUS_MAP three-layer healthy | Success pill — **only then** |
| Session chips (Idle, Starting, Waiting for VPN, Interrupted, Error) | deriveSessionChip | Neutral / warning / error |
| Device Online / ready / unauthorized | deviceState* | Success / warning |
| Service running | Relay listening / service truth | Success pill |
| ADB footer | ADB ok / missing | Dot + label |
| Switches | Settings | **Blue when ON**, slate OFF — never green ON |

Prefer Badge + optional StatusDot; never color-alone.

---

## 5. Cards

| Card | Notes |
|------|-------|
| Connection hero | Success tint **only** if Sharing healthy; PC↔Phone diagram OK as illustration |
| Device | Serial mono; model; USB icon; actions row |
| Install client | Download Lucide + CTA |
| How it works | Numbered 1–3 + optional Learn more (docs link OK) |
| Quick actions | Switch rows; stub unsupported |
| Traffic | Later |

Empty device: dashed/muted well + primary Connect device.

---

## 6. Alert / Dialog / Tooltip

| Component | MVP use |
|-----------|---------|
| Alert | ERROR_UX banners — title `[CODE] …`; recovery Buttons from ERROR_UX |
| Dialog | Quit confirm if session active: “Stop sharing and quit?” |
| Tooltip | One-sentence layer / direction hints; no HANDSHAKE_* ids |
| Separator | Between action groups / Advanced |
| Collapsible | Advanced relay; Logs |
| ScrollArea | Logs (`LOG_CAP` 500) |

---

## 7. Composition sketch

```
AppShell
├── Sidebar
│   ├── Brand (Lucide signal / logo)
│   ├── Nav: Dashboard, Devices, Traffic?, Logs, Settings
│   ├── PromoCard (optional visual)
│   └── AdbFooter
├── Topbar
│   ├── SearchPill (Later / decorative)
│   ├── ThemeIconButton
│   └── ServiceStatusBadge
└── Main
    ├── DashboardView
    │   ├── Greeting
    │   ├── ConnectionCard
    │   ├── DeviceCard(s)          // one device MVP
    │   ├── InstallClientCard
    │   ├── HowItWorks
    │   ├── QuickActions           // supported only
    │   ├── TrafficCard?           // Later
    │   └── AlertSlot
    ├── LogsView
    └── SettingsView
Dialog QuitConfirm
```

---

## 8. Out of inventory (MVP)

- System tray  
- Multi-device matrix / badge-as-product  
- Live throughput charts with fake MB/s  
- Functional global search  
- New orchestrator commands  
- Indigo-primary button system  

---

## 9. Acceptance (Desktop)

- [ ] AppShell matches approved-saas-dashboard visual language (blue primary)  
- [ ] Connected only when three-layer healthy  
- [ ] Run / Stop / Repair / Install / Refresh / Relay wired  
- [ ] Traffic chart not shipping fake data  
- [ ] Search non-functional or hidden  
- [ ] Switches blue when on  
- [ ] One Alert slot with `[CODE]`  
- [ ] Quit Dialog when session active  
- [ ] Tokens from `DESIGN_TOKENS.md` — no competing theme  

---

*Implementation: Desktop. Spec: UX. State/errors SoT unchanged.*
