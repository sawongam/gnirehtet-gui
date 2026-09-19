# UX docs — gnirehtet-gui

Index for Desktop / Lead. **Desktop owns implementation; UX owns these specs.**  
Tip context: `d8c8392` on `dev`. Stack: Tauri 2 + Svelte 5 + SvelteKit.

## Redesign pack (ASAP polish)

| Doc | Purpose |
|-----|---------|
| [DESIGN_SYSTEM.md](./DESIGN_SYSTEM.md) | Compare shadcn-svelte / Skeleton / Melt+Bits; **recommend shadcn-svelte + Tailwind** |
| [THEME.md](./THEME.md) | Dark-first tokens, type, window size, pasteable CSS vars |
| [SHELL_SCREENS.md](./SHELL_SCREENS.md) | Single-window wireframe + Idle→Sharing states |
| [COMPONENT_INVENTORY.md](./COMPONENT_INVENTORY.md) | shadcn primitives → Run/Stop/Repair/Install/Relay/Refresh |
| [COPY_RULES.md](./COPY_RULES.md) | Chip verbs, Run/Stop decision, ban Connected / HANDSHAKE jargon |

## Existing SoT (do not delete)

| Doc | Purpose |
|-----|---------|
| [UX_RESEARCH.md](./UX_RESEARCH.md) | Research backdrop |
| [USER_FLOWS.md](./USER_FLOWS.md) | End-to-end flows |
| [SCREEN_SPEC.md](./SCREEN_SPEC.md) | Earlier screen inventory (MVP surfaces) |
| [ERROR_UX.md](./ERROR_UX.md) | **SoT** error codes, titles, recovery |
| [MVP_UX.md](./MVP_UX.md) | MVP cut + copy deck |
| [EVENT_STATUS_MAP.md](./EVENT_STATUS_MAP.md) | **SoT** session chip + three-layer ↔ events |

## Hard product rules (reminder)

- Linux + Windows first-class; macOS best-effort  
- No networking rewrite — gnirehtet APK + Rust relay  
- MVP actions only; no tray / multi-share / charts in this polish  
- **Never** claim Sharing without Relay + Tunnel + Device VPN handshake (`EVENT_STATUS_MAP`)  
- Direction: **Internet: This PC → Phone**  
- Aesthetic: dense utility (Linear / Raycast / scrcpy) — not Bootstrap admin / SaaS marketing  

## Known live UI debt (fix in polish)

- Flat dark panels, radio device list, equal-weight action row  
- Jargon: `HANDSHAKE_LIVENESS_MVP` tip in Device VPN layer  
- Log dump + stacked banners without hierarchy  
- `app.html` title still “Tauri + SvelteKit + Typescript App”
