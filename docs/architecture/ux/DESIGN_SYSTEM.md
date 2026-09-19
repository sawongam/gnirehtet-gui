# Design System — gnirehtet-gui

**Audience:** Desktop Engineer, UX  
**Status:** Spec for ASAP polish (Sangam: live UI = “90s form / not shippable”)  
**Stack tip:** `d8c8392` on `dev` — Tauri 2 + Svelte 5 + SvelteKit, **no Tailwind yet**  
**Owns implementation:** Desktop. **Owns this spec:** UX.

Cross-ref: `THEME.md`, `SHELL_SCREENS.md`, `COMPONENT_INVENTORY.md`, `COPY_RULES.md`, `EVENT_STATUS_MAP.md`, `ERROR_UX.md`.

---

## 1. Goal

Ship a dense technical-utility shell (Linear / Raycast / scrcpy polish) without rewriting the orchestrator, networking, or ERROR_UX codes.

Hard constraints:

- Linux + Windows first-class; macOS best-effort
- Reuse gnirehtet APK + Rust relay (no networking rewrite)
- MVP actions only: Install, Repair tunnel, Run, Stop, Start/Stop Relay (advanced), Refresh devices
- Never claim **Sharing** without three-layer healthy — `EVENT_STATUS_MAP.md` is SoT
- Keep ERROR_UX codes from `apps/desktop/src/lib/errorUx.ts` and `ERROR_UX.md`

---

## 2. Options compared (exactly three)

### A. shadcn-svelte (Bits UI + Tailwind; copy-owned components) — **RECOMMENDED**

| | |
|--|--|
| **What** | CLI copies Svelte components into `src/lib/components/ui`. Primitives from **Bits UI** (a11y, keyboard, focus). Styling via **Tailwind** + CSS variables. |
| **Ownership** | Components live in-repo — Desktop can restyle density, radius, and accents without fighting `node_modules`. |
| **Tauri evidence** | Public Tauri 2 + Svelte 5 + shadcn-svelte templates exist (e.g. [alysonhower/tauri2-svelte5-shadcn](https://github.com/alysonhower/tauri2-svelte5-shadcn), [NoCrypt/ncpt-template](https://github.com/NoCrypt/ncpt-template)). Proven path for WRY webviews on Win/Linux. |
| **Fit** | Zinc/neutral + teal accent ≈ Linear/Raycast; incremental add (`button`, `badge`, `card`…) without touching Rust IPC. |

### B. Skeleton UI (npm toolkit + theming engine) — **REJECTED**

| | |
|--|--|
| **What** | npm-installed component + theme package; CSS-variable themes (`data-theme`), app-kit patterns (shells, drawers, nav). |
| **Why reject** | Too opinionated / app-kit for a **single-window dense utility**. Theme engine + preset look drifts toward SaaS dashboards. Customizing away from Skeleton chrome costs more than owning a small shadcn set. Runtime theme switching is not an MVP need (light-first; dark = optional toggle). |

### C. Melt UI / Bits UI + custom tokens (headless + bespoke CSS) — **REJECTED for ASAP**

| | |
|--|--|
| **What** | Headless builders (Melt) or headless primitives (Bits) + hand-rolled CSS/Tailwind wrappers. |
| **Why reject for ASAP** | Correct long-term foundation (Bits is already under shadcn-svelte), but pure headless means **writing every visual layer** — buttons, alerts, dialogs, scroll areas — from scratch. Too slow for “not shippable → polish” turnaround. Prefer A: Bits accessibility **plus** pre-styled, copy-owned shells. |

---

## 2.1 Theme default

**Light-first** (Sangam, 2026-09-20). Boot without `html.dark`. Tokens and pasteable CSS live in `THEME.md` (canvas `#F4F7FB`, cards white, accent `#0D9488`). Dark is an optional `class="dark"` alternate — do not ship dark as the default.

---

## 3. Recommendation: A (shadcn-svelte)

1. **Owns components** — restyle for density without library forks.  
2. **Bits accessibility** — Dialog (quit confirm), Collapsible (advanced relay / logs), Tooltip (layer help) get correct focus/keyboard for free.  
3. **Tailwind density control** — `h-8`, `text-xs`, `gap-1` match utility aesthetic; no Bootstrap-admin defaults.  
4. **Proven Tauri templates** — reduces webview + Vite + Svelte 5 friction.  
5. **Incremental** — wrap existing `+page.svelte` actions; do **not** rewrite orchestrator stores (`sessionLayers`, `errorUx`, device poll).  
6. **Look target** — light cool-gray canvas + white cards + teal accent (`THEME.md`) — Linear-light / scrcpy polish, not Material blue admin.

Desktop may later extract Bits-only if needed; start with shadcn-svelte wrappers.

---

## 4. Install sketch (Desktop)

From `apps/desktop/` (after adding Tailwind — required for design system):

```bash
# 1. Tailwind (v4 Vite plugin preferred; follow current shadcn-svelte docs)
npm install -D tailwindcss @tailwindcss/vite

# 2. shadcn-svelte init (creates components.json, CSS vars, utils)
npx shadcn-svelte@latest init
# When prompted: base color zinc/slate · style default · CSS variables yes · aliases $lib/components
# Override primary/accent CSS vars to THEME.md teal; boot **light** (no html.dark by default)

# 3. Add MVP primitives only
npx shadcn-svelte@latest add button badge card alert separator collapsible scroll-area tooltip dialog
```

**Needed primitives (MVP):**

| Primitive | Use |
|-----------|-----|
| `button` | Run, Stop, Repair, Install, Refresh, Relay advanced |
| `badge` | Session chip, adb/device state |
| `card` | Device row / selected device |
| `alert` | ERROR_UX banners (`[CODE]` + title + recovery) |
| `separator` | Session vs Advanced relay |
| `collapsible` | Advanced: Start/Stop Relay; Logs |
| `scroll-area` | Log pane |
| `tooltip` | Layer strip help (no jargon walls) |
| `dialog` | Quit confirm if session active |

Optional later (not MVP): `dropdown-menu`, `switch`, `input` (Settings).

Do **not** add charts, tray menus, or multi-share UI.

---

## 5. Tailwind + dark mode

- **Strategy:** `class` on `<html>` (or root) — e.g. `<html class="dark">`. Dark is default for utilities; light optional Phase 1.1.  
- **Tokens:** CSS variables in `app.css` / `app.html` styles — see pasteable block in `THEME.md`. Map Tailwind theme (or `@theme`) to `--bg`, `--accent`, etc.  
- **shadcn defaults:** Keep HSL/OKLCH variable pattern from init; override to zinc + teal accent — do not keep generic Material blue.  
- **Current app:** hand CSS in `+page.svelte` `<style>` — migrate gradually; avoid mixed competing token systems.

---

## 6. Windows / Linux webview notes

| Platform | Webview | Font stack |
|----------|---------|------------|
| Windows | WebView2 | `"Segoe UI", system-ui, sans-serif` |
| Linux | WebKitGTK (WRY) | `system-ui, "Inter", sans-serif` (bundle Inter or use system) |
| macOS | WKWebView (best-effort) | `-apple-system, system-ui, sans-serif` |

Recommended UI stack:

```css
font-family: "Inter", "Segoe UI", system-ui, -apple-system, sans-serif;
```

Mono (serials, logs):

```css
font-family: "JetBrains Mono", ui-monospace, "Cascadia Mono", "SF Mono", Consolas, monospace;
```

- Prefer `system-ui` fallbacks; Inter/JetBrains Mono via Fontsource or local static assets if bundled.  
- Avoid `-webkit` only tricks that break WebView2.  
- Default window ~**920×640**, min ~**720×520** (`THEME.md`).  
- Fix `apps/desktop/src/app.html` title: still **“Tauri + SvelteKit + Typescript App”** — change to product name (e.g. `gnirehtet-gui`).

---

## 7. Migration posture (Desktop)

1. Add Tailwind + shadcn-svelte init (no orchestrator changes).  
2. Replace equal-weight button row with Button variants (`COMPONENT_INVENTORY.md`).  
3. Restyle three-layer strip + session chip; strip HANDSHAKE jargon (`COPY_RULES.md`).  
4. Device radio-list → selectable Cards.  
5. Logs → Collapsible + ScrollArea.  
6. One contextual Alert slot (not stacked banner soup).  
7. Dialog for quit-if-active.

**Out of scope for this polish:** Settings redesign, Diagnostics charts, tray, multi-device share, networking changes.

---

## 8. Aesthetic bans

- Bootstrap / admin template look  
- SaaS marketing hero / wizard onboarding page  
- Pill-soup chips everywhere  
- Equal-weight Install / Repair / Run / Stop grid  
- Generic Material blue primary CTA  
- Claiming Sharing on intent-sent or relay-only

---

*UX owns this doc. Desktop implements. State/errors SoT remains `EVENT_STATUS_MAP.md` + `ERROR_UX.md`.*
