# UX docs — gnirehtet-gui

Index for Desktop / Lead. **Desktop owns implementation; UX owns these specs.**  
Stack: Tauri 2 + Svelte 5 + SvelteKit + Tailwind + shadcn-svelte + Lucide.

## Visual SoT (locked 2026-09-20)

| Doc / asset | Purpose |
|-------------|---------|
| [**VISUAL_APPROVED_V3.md**](./VISUAL_APPROVED_V3.md) | **LOCKED** Sangam dashboard SoT — Primary `#2563EB` blue |
| [mockups/approved-saas-dashboard.png](./mockups/approved-saas-dashboard.png) | Pixel goal for AppShell |
| [DESIGN_TOKENS.md](./DESIGN_TOKENS.md) | Exact tokens + pasteable `@theme` |
| [THEME.md](./THEME.md) | Theme summary → points at DESIGN_TOKENS / V3 |
| [SHELL_SCREENS.md](./SHELL_SCREENS.md) | AppShell IA + MVP vs Later |
| [COMPONENT_INVENTORY.md](./COMPONENT_INVENTORY.md) | AppShell components → Run/Stop/Repair/Install/Relay |
| [COPY_RULES.md](./COPY_RULES.md) | Connected synonym rules + ERROR_UX |

**Historical (not primary goal):** `mockups/approved-waiting-vpn.png`, `VISUAL_V2_SAAS.md`, older indigo-as-primary notes in git history.

## Design system

| Doc | Purpose |
|-----|---------|
| [DESIGN_SYSTEM.md](./DESIGN_SYSTEM.md) | Recommend shadcn-svelte + Tailwind |

## Existing behavior SoT (do not delete)

| Doc | Purpose |
|-----|---------|
| [UX_RESEARCH.md](./UX_RESEARCH.md) | Research backdrop |
| [USER_FLOWS.md](./USER_FLOWS.md) | End-to-end flows |
| [SCREEN_SPEC.md](./SCREEN_SPEC.md) | Earlier screen inventory |
| [ERROR_UX.md](./ERROR_UX.md) | **SoT** error codes, titles, recovery |
| [MVP_UX.md](./MVP_UX.md) | MVP cut + copy deck |
| [EVENT_STATUS_MAP.md](./EVENT_STATUS_MAP.md) | **SoT** session chip + three-layer ↔ events |

## Hard product rules

- Linux + Windows first-class; macOS best-effort  
- No networking rewrite — gnirehtet APK + Rust relay; UI ↔ existing orchestrator events  
- MVP: one device, ADB detect, install APK, start/stop, repair, clear state, errors, logs, graceful quit  
- **Connected / Sharing** only when Relay + Tunnel + Device VPN handshake healthy  
- Direction: **Internet: This PC → Phone**  
- Aesthetic: professional developer utility + modern SaaS dashboard (**blue** primary, not purple)  
- Traffic charts / functional search / multi-device = **Later**  

## Do not implement UI polish until

UX / Sangam greenlights handoff from chat — see `VISUAL_APPROVED_V3.md` ownership. Match `approved-saas-dashboard.png` when greenlit; do not invent a competing theme.
