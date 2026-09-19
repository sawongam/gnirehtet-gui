# Theme — gnirehtet-gui

**Audience:** Desktop Engineer  
**Status:** **Light-first** design tokens (Sangam request 2026-09-20)  
**Pairs with:** `DESIGN_SYSTEM.md` (shadcn-svelte + Tailwind `class` strategy)

Cross-ref: `SHELL_SCREENS.md`, `EVENT_STATUS_MAP.md` (layer health semantics).

---

## 1. Principles

- **Light-first.** Default shell is a bright, clean technical utility — not a dark IDE clone. Dark remains a supported alternate via `class="dark"`, not the MVP boot default.  
- **Nice, restrained color.** Soft cool-gray canvas + white cards + teal accent. Feels closer to Linear light / Apple Settings than Bootstrap or Material.  
- **Dense technical.** scrcpy / Raycast polish — no marketing heroes, no rainbow chrome.  
- **Semantics over decoration.** Success / warning / danger / info map to Relay · Tunnel · Device VPN and ERROR_UX banners.  
- **Elevation = soft shadow + hairline border.** Airy but still tool-like.

---

## 2. Window chrome

| | Value |
|--|-------|
| Default size | ~**920 × 640** |
| Minimum | ~**720 × 520** |
| Title (`app.html`) | Product name (`gnirehtet-gui`) — **fix** “Tauri + SvelteKit + Typescript App” |
| Direction chrome | Always visible: **Internet: This PC → Phone** |
| Default color-scheme | **`light`** |

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

- **Base unit:** 4px.  
- **Radius:** **8px** cards/buttons; **6px** chips/inputs. Avoid pill-soup.  
- **Elevation (light):** `0 1px 2px rgba(15, 23, 42, 0.06), 0 4px 12px rgba(15, 23, 42, 0.04)`.  
- **Gaps:** shell sections `12–16px`; tight action clusters `8px`.

---

## 5. Color story (light)

| Role | Hex | Feel |
|------|-----|------|
| Canvas | `#F4F7FB` | Soft cool paper (not stark #fff wall) |
| Card / elevated | `#FFFFFF` | Clean panels |
| Muted well | `#EEF2F7` | Logs, nested rows |
| Border | `#D8E0EA` | Soft cool edge |
| Text | `#0F172A` | Near-slate, readable |
| Muted text | `#64748B` | Secondary |
| Accent | `#0D9488` | Teal — CTA / focus (works on white) |
| Accent soft | `#CCFBF1` | Selected device / chip fill |
| Success | `#059669` | Sharing / healthy layer |
| Warning | `#D97706` | Waiting VPN / unauthorized |
| Danger | `#DC2626` | Errors / Interrupted |
| Info | `#0284C7` | Tips / relay port |

Accent on light is **deeper teal** (`#0D9488`) so contrast holds; dark mode may use brighter `#2DD4BF`.

---

## 6. Token tables

### Surfaces

| Token | Light intent |
|-------|--------------|
| `--bg` | App canvas |
| `--bg-elevated` | Cards, panels, title strip |
| `--bg-muted` | Log pane, nested wells, hover |
| `--border` | Default edges |
| `--border-subtle` | Inner dividers |

### Text

| Token | Intent |
|-------|--------|
| `--fg` | Primary |
| `--fg-muted` | Secondary / hints |
| `--fg-subtle` | Micro labels |

### Accent & semantic

| Token | Light example | Use |
|-------|---------------|-----|
| `--accent` | `#0D9488` | Run / primary |
| `--accent-fg` | `#F0FDFA` | Text on accent |
| `--accent-muted` | teal wash | Selection / focus ring |
| `--success` | `#059669` | Sharing chip; healthy |
| `--warning` | `#D97706` | Waiting / Starting |
| `--danger` | `#DC2626` | Error / TUNNEL_LOST |
| `--info` | `#0284C7` | Informational |

### Layer accents (restrained tints)

| Layer | Token | Light lean |
|-------|-------|------------|
| Relay | `--layer-relay` | Sky / slate-blue |
| Tunnel | `--layer-tunnel` | Soft violet |
| Device VPN | `--layer-vpn` | Teal (aligns with accent when healthy) |

Healthy **status word** still uses `--success`; layer tint is border/bg only.

---

## 7. Pasteable CSS variables

**MVP boot: light default** — do **not** put `class="dark"` on `<html>` unless user toggles it.

```css
:root {
  color-scheme: light;

  --bg: #f4f7fb;
  --bg-elevated: #ffffff;
  --bg-muted: #eef2f7;
  --border: #d8e0ea;
  --border-subtle: #e8eef5;

  --fg: #0f172a;
  --fg-muted: #64748b;
  --fg-subtle: #94a3b8;

  --accent: #0d9488;
  --accent-fg: #f0fdfa;
  --accent-muted: rgba(13, 148, 136, 0.12);

  --success: #059669;
  --warning: #d97706;
  --danger: #dc2626;
  --info: #0284c7;

  --layer-relay: #0284c7;
  --layer-tunnel: #7c3aed;
  --layer-vpn: #0d9488;

  --radius: 8px;
  --radius-sm: 6px;
  --shadow: 0 1px 2px rgba(15, 23, 42, 0.06), 0 4px 12px rgba(15, 23, 42, 0.04);

  --font-ui: "Inter", "Segoe UI", system-ui, -apple-system, sans-serif;
  --font-mono: "JetBrains Mono", ui-monospace, "Cascadia Mono", Consolas, monospace;
}

.dark,
:root.dark,
html.dark {
  color-scheme: dark;

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

shadcn-svelte init: prefer **zinc/slate base + custom accent override** to these CSS vars (map `--primary` → `--accent`).

---

## 8. Component color mapping (quick)

| UI | Tokens |
|----|--------|
| Shell background | `--bg` |
| Cards / device list / session column | `--bg-elevated` + `--shadow` + `--border` |
| Run (primary) | fill `--accent`, text `--accent-fg` |
| Stop | danger outline or solid when session active |
| Repair / Install / Refresh | secondary / ghost — never equal weight to Run |
| Selected device row | `--accent-muted` border/bg |
| Session chip Sharing | success |
| Waiting / Starting / Stopping | warning |
| Interrupted / Error | danger |
| ERROR_UX alert | tinted bg from danger/warning; always show `[CODE]` |
| Log pane | `--bg-muted`, mono 12px |

---

## 9. What not to do

- Default to dark after this revision.  
- Keep Material `#1a73e8` primary.  
- Pure `#FFFFFF` full-window with no canvas tint (looks unfinished).  
- Neon accents / heavy gradients.  
- Pill chips for every status.  
- Color-alone status (always pair with text from `EVENT_STATUS_MAP` / `COPY_RULES`).

---

*Tokens only. Layout → `SHELL_SCREENS.md`. Components → `COMPONENT_INVENTORY.md`.*
