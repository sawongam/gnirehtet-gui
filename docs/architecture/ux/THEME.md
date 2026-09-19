# Theme — gnirehtet-gui

**Audience:** Desktop Engineer  
**Status:** Dark-first design tokens for ASAP polish  
**Pairs with:** `DESIGN_SYSTEM.md` (shadcn-svelte + Tailwind `class` strategy)

Cross-ref: `SHELL_SCREENS.md`, `EVENT_STATUS_MAP.md` (layer health semantics).

---

## 1. Principles

- **Dark-first.** Utilities live in dark; light theme optional Phase 1.1 only.  
- **Dense technical.** Linear / Raycast / scrcpy — not Bootstrap admin, not SaaS marketing.  
- **Restrained color.** Accent is teal/cyan-leaning technical — **not** generic Material blue (`#1a73e8` in current `+page.svelte` must go).  
- **Semantics over decoration.** Success / warning / danger / info map to Relay · Tunnel · Device VPN and ERROR_UX banners.  
- **Elevation = 1 border + soft shadow.** No heavy glass / neon.

---

## 2. Window chrome

| | Value |
|--|-------|
| Default size | ~**920 × 640** |
| Minimum | ~**720 × 520** |
| Title (`app.html`) | Product name (`gnirehtet-gui`) — **fix** current “Tauri + SvelteKit + Typescript App” |
| Direction chrome | Always visible: **Internet: This PC → Phone** |

Set via Tauri window config + CSS; do not rely on browser chrome.

---

## 3. Type

| Role | Family | Notes |
|------|--------|-------|
| UI | Inter **or** system-ui stack | `Inter, "Segoe UI", system-ui, -apple-system, sans-serif` |
| Mono | JetBrains Mono / ui-monospace | Serials, ports, logs, `[CODE]` |

**Type scale (dense):**

| Token | Size | Use |
|-------|------|-----|
| `text-2xs` | **12px** | Layer labels, meta, log lines |
| `text-xs` | **13px** | Secondary actions, badges |
| `text-sm` | **14px** | Body, device rows, banner copy |
| `text-base` | **16px** | Primary button label, section titles |
| `text-lg` | **20px** | App brand / window title only |

Line-height ~1.4–1.45. Avoid 18px+ body in the shell.

---

## 4. Spacing & shape

- **Base unit:** 4px (`space-1` = 4, `space-2` = 8, …).  
- **Radius:** **6–8px** for buttons, cards, banners. **Not** pill-soup (`border-radius: 999px` only for tiny status dots if needed).  
- **Elevation:** `1px` border (`--border`) + optional `box-shadow: 0 1px 2px rgba(0,0,0,0.35)`.  
- **Gaps:** shell sections `12–16px`; tight action clusters `8px`.

---

## 5. Color tokens

Semantic names Desktop should map into Tailwind / shadcn CSS variables.

### Surfaces

| Token | Intent |
|-------|--------|
| `--bg` | App background |
| `--bg-elevated` | Cards, panels, title bar strip |
| `--bg-muted` | Log pane, nested wells, hover row |
| `--border` | Default edges |
| `--border-subtle` | Dividers inside panels |

### Text

| Token | Intent |
|-------|--------|
| `--fg` | Primary text |
| `--fg-muted` | Secondary / hints |
| `--fg-subtle` | Labels, uppercase microcopy |

### Accent (primary CTA)

| Token | Example | Intent |
|-------|---------|--------|
| `--accent` | `#2DD4BF` | Run / primary recovery — teal technical |
| `--accent-fg` | `#042F2E` | Text on accent fill |
| `--accent-muted` | teal @ ~20% | Soft selected device / focus ring |

### Semantic (banners + chips)

| Token | Example | Use |
|-------|---------|-----|
| `--success` | `#34D399` | Sharing chip; healthy layer |
| `--warning` | `#FBBF24` | Waiting VPN, Unauthorized, Starting |
| `--danger` | `#F87171` | Error chip, TUNNEL_LOST, crashes |
| `--info` | `#67E8F9` | Neutral informational (relay port tip) |

### Layer accents (restrained)

Distinct enough to scan; not rainbow.

| Layer | Token | Hue lean |
|-------|-------|----------|
| Relay | `--layer-relay` | Cool blue-grey / cyan muted |
| Tunnel | `--layer-tunnel` | Violet-grey muted |
| Device VPN | `--layer-vpn` | Teal muted (aligns with accent when healthy) |

Healthy state still uses `--success` for the **status word**; layer tint is background/border only.

---

## 6. Pasteable CSS variables

Desktop can paste into global CSS (then wire Tailwind `@theme` / shadcn vars). Dark is default; light is stub for Phase 1.1.

```css
:root {
  /* Light optional — Phase 1.1; prefer .dark as default in app.html */
  --bg: #f4f4f5;
  --bg-elevated: #ffffff;
  --bg-muted: #e4e4e7;
  --border: #d4d4d8;
  --border-subtle: #e4e4e7;

  --fg: #18181b;
  --fg-muted: #52525b;
  --fg-subtle: #71717a;

  --accent: #0d9488;
  --accent-fg: #f0fdfa;
  --accent-muted: rgba(13, 148, 136, 0.15);

  --success: #059669;
  --warning: #d97706;
  --danger: #dc2626;
  --info: #0891b2;

  --layer-relay: #64748b;
  --layer-tunnel: #7c6f9a;
  --layer-vpn: #0f766e;

  --radius: 8px;
  --radius-sm: 6px;
  --shadow: 0 1px 2px rgba(0, 0, 0, 0.08);

  --font-ui: "Inter", "Segoe UI", system-ui, -apple-system, sans-serif;
  --font-mono: "JetBrains Mono", ui-monospace, "Cascadia Mono", Consolas, monospace;
}

.dark,
:root.dark,
html.dark {
  --bg: #0c0e12;
  --bg-elevated: #141820;
  --bg-muted: #1a1f29;
  --border: #2a3140;
  --border-subtle: #1e2430;

  --fg: #e8eaed;
  --fg-muted: #9aa3b2;
  --fg-subtle: #6b7385;

  --accent: #2dd4bf;
  --accent-fg: #042f2e;
  --accent-muted: rgba(45, 212, 191, 0.16);

  --success: #34d399;
  --warning: #fbbf24;
  --danger: #f87171;
  --info: #67e8f9;

  --layer-relay: #7dd3fc;
  --layer-tunnel: #c4b5fd;
  --layer-vpn: #5eead4;

  --shadow: 0 1px 2px rgba(0, 0, 0, 0.45);
}

html {
  color-scheme: dark;
  background: var(--bg);
  color: var(--fg);
  font-family: var(--font-ui);
  font-size: 14px;
  line-height: 1.45;
}

code,
.mono,
.log-pane {
  font-family: var(--font-mono);
}
```

**MVP boot:** put `class="dark"` on `<html>` in `app.html` and set `color-scheme: dark`.

---

## 7. Component color mapping (quick)

| UI | Tokens |
|----|--------|
| Run (primary) | `bg: accent`, `fg: accent-fg` |
| Stop | `destructive` / danger outline or solid when session active |
| Repair / Install / Refresh | `secondary` or `ghost` — never equal weight to Run |
| Session chip Sharing | success |
| Session chip Waiting / Starting / Stopping | warning |
| Session chip Interrupted / Error | danger |
| Alert ERROR_UX | border + bg tint from danger/warning; always show `[CODE]` |
| Log pane | `bg-muted`, mono 12px |

---

## 8. What not to do

- Keep Material `#1a73e8` primary.  
- Pill chips for every status row.  
- Large marketing gradients / hero headers.  
- Light-only default.  
- Color-alone status (always pair with text from `EVENT_STATUS_MAP` / `COPY_RULES`).

---

*Tokens only. Layout states → `SHELL_SCREENS.md`. Component map → `COMPONENT_INVENTORY.md`.*
