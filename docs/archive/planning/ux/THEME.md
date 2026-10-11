# Theme — gnirehtet-gui

**Audience:** Desktop Engineer  
**Status:** Locked to visual SoT v3 (2026-09-20)  
**Token SoT:** [`DESIGN_TOKENS.md`](./DESIGN_TOKENS.md)  
**Visual SoT:** [`VISUAL_APPROVED_V3.md`](./VISUAL_APPROVED_V3.md) + [`mockups/approved-saas-dashboard.png`](./mockups/approved-saas-dashboard.png)  
**Pairs with:** `SHELL_SCREENS.md`, `DESIGN_SYSTEM.md` (shadcn-svelte + Tailwind)

Behavior unchanged: `EVENT_STATUS_MAP.md`, `ERROR_UX.md`, `COPY_RULES.md`, `TEAM_BRIEF.md`.

---

## 1. Direction

Professional developer utility + modern SaaS dashboard  
(**Linear × Vercel × Raycast × networking utility**).

| Was (v2 indigo-primary) | Now (v3 blue-primary) |
|-------------------------|------------------------|
| Primary `#4F46E5` indigo as brand CTA | **Primary `#2563EB` blue** — UI feels blue, not purple |
| Single-page product strip | **AppShell:** 240px sidebar + 72px topbar + dashboard grid |
| Waiting-VPN frame as visual goal | **Dashboard mockup** is visual goal; Waiting-VPN is historical only |

**Unchanged:** MVP actions, ERROR_UX codes, Sharing / Connected only when three-layer healthy, direction **Internet: This PC → Phone**.

---

## 2. Principles

- **Tokens live in `DESIGN_TOKENS.md`.** Do not fork a second palette here.
- **Light boot.** No `html.dark` by default; theme toggle chrome may exist (Later functional).
- **Blue = actions.** Green = health only. Red = destructive / error only.
- **Honest states.** Never claim Connected / Sharing on intent-sent or relay-only.
- **Optional alignment** with existing Tauri orchestrator events — no Rust rewrite required.

---

## 3. Window chrome

| | Value |
|--|-------|
| Default size | ~**1100 × 720** (sidebar shell) |
| Minimum | ~**900 × 600** |
| Title | `gnirehtet-gui` / product name per mockup |
| Boot | **Light** |
| Dark | Optional Later |

---

## 4. Brand at a glance (see DESIGN_TOKENS for full scales)

| Role | Hex |
|------|-----|
| **Primary** | `#2563EB` |
| Primary hover / pressed | `#1D4ED8` / `#1E40AF` |
| Primary soft | `#DBEAFE` / `#EFF6FF` |
| Accent indigo (sparing) | `#4F46E5` |
| Background | `#F8FAFC` |
| Surface | `#FFFFFF` |
| Border | `#E2E8F0` |
| Success / Warning / Error | Per `DESIGN_TOKENS.md` |

Focus ring: `#93C5FD`. Type: Inter + mono for tech values. Scale: Display 32 → Label 12 / Metrics 24.

---

## 5. Shell mapping

| Chrome | Spec |
|--------|------|
| Sidebar | 240px — Dashboard, Devices, Traffic, Logs, Settings |
| Topbar | 72px — search pill, theme, service status |
| Dashboard | Connection hero, Devices, Install client, How it works, Quick actions; Traffic card = **Later** |

Full IA / wireframe → `SHELL_SCREENS.md`.

---

## 6. Component color rules

| UI | Treatment |
|----|-----------|
| Primary CTA (Connect device / Run) | Solid `#2563EB` |
| Stop tethering / Disconnect | Soft destructive (`#FEF2F2` / `#DC2626`) |
| Repair / Restart tunnel / Install | Secondary outline or primary when Interrupted |
| Switches ON | **Blue**, not green |
| Connected hero | Success wash **only** if three-layer healthy |
| Active nav | Primary-50 wash + Primary-500 |

---

## 7. Historical notes

- `THEME.md` v2 indigo-primary and `VISUAL_V2_SAAS.md` / `approved-waiting-vpn.png` are **superseded** as the primary visual goal.
- Keep Waiting-VPN mockup as historical reference only.
- Implement against `mockups/approved-saas-dashboard.png` + `DESIGN_TOKENS.md`.

---

## 8. Anti-patterns

- Treating indigo as primary brand  
- Competing theme docs that diverge from V3  
- Fake live traffic / Connected without orchestrator truth  
- Dense terminal aesthetic as default  

---

*Pasteable `@theme` block → `DESIGN_TOKENS.md` §16. Layout → `SHELL_SCREENS.md`.*
