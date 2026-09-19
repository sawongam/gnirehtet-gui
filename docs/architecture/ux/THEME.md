# Theme — gnirehtet-gui

**Audience:** Desktop Engineer  
**Status:** **Light SaaS UI** (Sangam 2026-09-20: “standard saas type of ui”)  
**Pairs with:** `DESIGN_SYSTEM.md` (shadcn-svelte + Tailwind), `SHELL_SCREENS.md`

Cross-ref: `EVENT_STATUS_MAP.md`, `ERROR_UX.md`, `COPY_RULES.md` (behavior unchanged).

---

## 1. Direction change

| Was (v1 utility) | Now (v2 SaaS) |
|------------------|---------------|
| Dense / scrcpy-tool chrome | Spacious product UI (Stripe / Vercel / Linear dashboard feel) |
| Compact 4px rhythm everywhere | Comfortable 8–16px padding, clearer sections |
| Tech-strip status chips | Soft SaaS badges + page header |
| Dark-or-dense optional | **Light SaaS default** |

**Unchanged:** MVP actions, ERROR_UX codes, Sharing = three-layer healthy only, direction copy **Internet: This PC → Phone**.

---

## 2. Principles

- **Standard SaaS.** Clean white surfaces, soft gray canvas, indigo/blue primary, generous whitespace, clear hierarchy.  
- **Product, not terminal.** Serials/logs stay monospace; everything else reads like a modern web app.  
- **One primary CTA per state.** Secondary actions quieter (outline / ghost).  
- **Semantics still honest.** Never claim Sharing early; banners keep `[CODE]`.

---

## 3. Window chrome

| | Value |
|--|-------|
| Default size | ~**1000 × 700** (SaaS needs a bit more air) |
| Minimum | ~**800 × 560** |
| Title | `gnirehtet-gui` |
| Boot | **Light** (no `html.dark` by default) |
| Dark | Optional later toggle |

---

## 4. Type

| Role | Family |
|------|--------|
| UI | Inter, "Segoe UI", system-ui, sans-serif |
| Mono | ui-monospace / JetBrains Mono (serials, logs, codes only) |

| Token | Size | Use |
|-------|------|-----|
| `text-xs` | 12px | Badges, meta |
| `text-sm` | 14px | Body, rows |
| `text-base` | 16px | Section titles, primary buttons |
| `text-lg` | 20px | Page title |
| `text-xl` | 24px | Rare — empty-state headline only |

Line-height ~1.5. **More air than v1 utility.**

---

## 5. Spacing & shape

- Base **8px** (SaaS comfort). Section gaps **16–24px**. Card padding **16–20px**.  
- Radius **10–12px** cards; **8px** buttons.  
- Shadow: `0 1px 2px rgba(15,23,42,0.05), 0 8px 24px rgba(15,23,42,0.06)`.  
- Borders hairline `#E2E8F0`.

---

## 6. Color story (SaaS light)

| Role | Hex | Notes |
|------|-----|-------|
| Canvas | `#F8FAFC` | Soft slate paper |
| Surface / card | `#FFFFFF` | |
| Muted / well | `#F1F5F9` | Empty states, logs |
| Border | `#E2E8F0` | |
| Text | `#0F172A` | |
| Muted text | `#64748B` | |
| **Primary** | `#4F46E5` | Indigo — standard SaaS CTA (replaces teal-as-primary) |
| Primary soft | `#EEF2FF` | Selected rows / focus wash |
| Success | `#10B981` | Sharing / healthy |
| Warning | `#F59E0B` | Waiting VPN |
| Danger | `#EF4444` | Stop / errors |
| Info | `#3B82F6` | Tips |

Teal may remain as a **Device VPN layer accent only**, not the primary brand button.

---

## 7. Pasteable CSS variables

```css
:root {
  color-scheme: light;

  --bg: #f8fafc;
  --bg-elevated: #ffffff;
  --bg-muted: #f1f5f9;
  --border: #e2e8f0;
  --border-subtle: #f1f5f9;

  --fg: #0f172a;
  --fg-muted: #64748b;
  --fg-subtle: #94a3b8;

  --accent: #4f46e5;          /* primary CTA — indigo SaaS */
  --accent-fg: #ffffff;
  --accent-muted: rgba(79, 70, 229, 0.10);

  --success: #10b981;
  --warning: #f59e0b;
  --danger: #ef4444;
  --info: #3b82f6;

  --layer-relay: #3b82f6;
  --layer-tunnel: #8b5cf6;
  --layer-vpn: #14b8a6;

  --radius: 12px;
  --radius-sm: 8px;
  --shadow: 0 1px 2px rgba(15, 23, 42, 0.05), 0 8px 24px rgba(15, 23, 42, 0.06);

  --font-ui: "Inter", "Segoe UI", system-ui, -apple-system, sans-serif;
  --font-mono: "JetBrains Mono", ui-monospace, "Cascadia Mono", Consolas, monospace;
}

/* Optional dark — not boot default */
html.dark {
  color-scheme: dark;
  --bg: #0b1220;
  --bg-elevated: #111827;
  --bg-muted: #1f2937;
  --border: #374151;
  --border-subtle: #1f2937;
  --fg: #f9fafb;
  --fg-muted: #9ca3af;
  --fg-subtle: #6b7280;
  --accent: #818cf8;
  --accent-fg: #0f172a;
  --accent-muted: rgba(129, 140, 248, 0.16);
  --success: #34d399;
  --warning: #fbbf24;
  --danger: #f87171;
  --info: #60a5fa;
  --shadow: 0 1px 2px rgba(0, 0, 0, 0.4);
}

html {
  background: var(--bg);
  color: var(--fg);
  font-family: var(--font-ui);
  font-size: 14px;
  line-height: 1.5;
}
```

Map shadcn `--primary` → `--accent` (indigo).

---

## 8. Component mapping

| UI | Treatment |
|----|-----------|
| Page header | Title + direction badge + session/ADB badges (right) |
| Run | Solid indigo primary, full-width in session card |
| Stop | Destructive solid/outline |
| Repair / Install / Refresh | Outline secondary |
| Layer strip | Three soft cards with left accent bar + label + value |
| Device row | Selectable card/row with indigo wash when selected |
| ERROR_UX | Soft tint Alert with `[CODE]` title |
| Logs | Muted well, more padding, collapsible |

---

## 9. Anti-patterns (this pass)

- Dense “sysadmin” chrome / terminal aesthetic as the default look.  
- Teal as the only brand color for primary buttons.  
- Equal-weight button grids.  
- Claiming Sharing without three-layer health.

---

*Layout wireframe → `SHELL_SCREENS.md`. Behavior SoT unchanged.*
