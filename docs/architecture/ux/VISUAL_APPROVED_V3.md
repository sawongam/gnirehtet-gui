# Visual SoT v3 — Sangam approved dashboard mockup

**Status:** LOCKED (2026-09-20)  
**Mockup:** [mockups/approved-saas-dashboard.png](./mockups/approved-saas-dashboard.png)  
**Stack unchanged:** Tauri 2 + Svelte 5 + SvelteKit + Tailwind + shadcn-svelte + Lucide  

## Direction

Professional developer utility + modern SaaS dashboard.  
**Linear × Vercel × Raycast × networking utility** — not Bootstrap admin, not marketing SaaS.

Personality: clean, technical, trustworthy, lightweight, calm, precise, slightly premium.  
Whitespace + strong hierarchy. Blue for actions. Green **only** for healthy connectivity. Red **only** for destructive/error.

## Exact tokens (SoT)

### Brand (blue — not purple-primary)
| Token | Value | Use |
|-------|-------|-----|
| Primary 500 | `#2563EB` | Main buttons |
| Primary 600 | `#1D4ED8` | Hover |
| Primary 700 | `#1E40AF` | Pressed |
| Primary 100 | `#DBEAFE` | Soft backgrounds |
| Primary 50 | `#EFF6FF` | Selected surfaces |

### Accent indigo (sparingly)
`#4F46E5` — upload graph, secondary highlights, focus/selected nav details. **Do not** paint the whole UI indigo.

### Semantic
- Success: `#15803D` / `#16A34A` / `#22C55E` / `#DCFCE7` / `#F0FDF4` — Connected, Online, Relay running, ADB ok, healthy tunnel, download
- Warning: `#B45309` / `#D97706` / `#F59E0B` / `#FEF3C7` / `#FFFBEB` — attention, permission, degraded
- Error: `#B91C1C` / `#DC2626` / `#EF4444` / `#FEE2E2` / `#FEF2F2` — disconnect, failed, fatal (**not** ordinary idle)

### Neutrals (cool slate)
Background `#F8FAFC` · Surface `#FFFFFF` · Border `#E2E8F0` · Heading `#0F172A` · Body `#475569` · Muted `#64748B`  
Slate scale 50–950 as in Sangam brief.

### Type
Inter (sans). Mono for ports/serials/metrics tech values.  
Scale: display 32 / page 28 / section 18 / card title 15 / body 14 / small 13 / label 12 / metrics 24.

### Spacing / radius / shadow
8-point spacing (4–64). Radii: sm 6, md 8, lg 12, xl 16. Cards **12px**. Badges pill. Soft card shadow only.

### Buttons
Primary 40px / 8px radius / `#2563EB`. Secondary white+border. Destructive soft red fill (`#FEF2F2` / `#DC2626`) — not solid red blocks.

### Layout chrome (match mockup)
- Sidebar ~240px: Dashboard, Devices, Traffic, Logs, Settings (+ ADB footer)
- Top bar: search, theme, service status
- Dashboard: connection hero, traffic, devices, install client, how-it-works, quick actions

## MVP behavior guardrails (Lead)

Implement the **visual shell** to match the mockup. Do **not** invent product features:

| Mockup surface | MVP rule |
|----------------|----------|
| Connection “Connected / tunnel active” | Only when three-layer healthy (EVENT_STATUS_MAP). Otherwise Idle / Waiting / Error honesty |
| Traffic graphs / live MB | **Deferred** — placeholder or empty state until real counters exist |
| Quick Actions toggles (auto-start, etc.) | UI ok if **non-functional or settings stubs**; no silent behavior change |
| Multi-page nav | Routes/views OK; keep Run/Stop/Install/Repair/Relay wired on Dashboard/Devices |
| Sharing chip | Still never without three-layer |

ERROR_UX codes + existing SessionController IPC stay SoT.

## Ownership

- UX: keep THEME.md / SHELL_SCREENS aligned to this doc + mockup  
- Desktop: restyle AppShell to match mockup pixel-close; report tip + screenshots  
