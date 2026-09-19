# Design Tokens — gnirehtet-gui

**Audience:** Desktop Engineer  
**Status:** LOCKED with visual SoT v3 (2026-09-20)  
**SoT:** [`VISUAL_APPROVED_V3.md`](./VISUAL_APPROVED_V3.md) + [`mockups/approved-saas-dashboard.png`](./mockups/approved-saas-dashboard.png)  
**Stack:** Tauri 2 + Svelte 5 + SvelteKit + Tailwind + shadcn-svelte + Lucide  

Do **not** invent a competing palette. Indigo `#4F46E5` is accent only — UI must feel **blue, not purple**.

---

## 1. Brand primary (blue)

| Token | Hex | Use |
|-------|-----|-----|
| Primary 50 | `#EFF6FF` | Selected surfaces, soft nav wash |
| Primary 100 | `#DBEAFE` | Soft backgrounds, sidebar promo card |
| Primary 500 | `#2563EB` | Main buttons, active nav, primary actions |
| Primary 600 | `#1D4ED8` | Hover |
| Primary 700 | `#1E40AF` | Pressed / active |

Map shadcn `--primary` → Primary 500 `#2563EB`.

---

## 2. Accent indigo (sparingly)

| Token | Hex | Use |
|-------|-----|-----|
| Accent | `#4F46E5` | Upload graph series, secondary highlights, focus/selected nav details |

**Do not** paint the whole UI indigo. Primary brand is blue.

---

## 3. Success / Warning / Error

### Success (health only)

| Role | Hex |
|------|-----|
| Success 800 | `#15803D` |
| Success 600 | `#16A34A` |
| Success 500 | `#22C55E` |
| Success 100 | `#DCFCE7` |
| Success 50 | `#F0FDF4` |

Use for: Connected / Sharing (three-layer healthy), Online, Relay running, ADB ok, healthy tunnel, download series. **Not** for primary CTAs or switches.

### Warning

| Role | Hex |
|------|-----|
| Warning 800 | `#B45309` |
| Warning 600 | `#D97706` |
| Warning 500 | `#F59E0B` |
| Warning 100 | `#FEF3C7` |
| Warning 50 | `#FFFBEB` |

Use for: attention, VPN permission pending, degraded / Waiting for VPN.

### Error (destructive / fatal only)

| Role | Hex |
|------|-----|
| Error 800 | `#B91C1C` |
| Error 600 | `#DC2626` |
| Error 500 | `#EF4444` |
| Error 100 | `#FEE2E2` |
| Error 50 | `#FEF2F2` |

Use for: disconnect / Stop tethering, failed, fatal banners. **Not** ordinary idle.

---

## 4. Neutrals (cool slate) + surfaces

| Token | Hex | Use |
|-------|-----|-----|
| Slate 50 | `#F8FAFC` | App background / canvas |
| Slate 100 | `#F1F5F9` | Muted wells, empty states |
| Slate 200 | `#E2E8F0` | Borders |
| Slate 300 | `#CBD5E1` | Stronger dividers |
| Slate 400 | `#94A3B8` | Subtle icons / placeholders |
| Slate 500 | `#64748B` | Muted text |
| Slate 600 | `#475569` | Body text |
| Slate 700 | `#334155` | Emphasized body |
| Slate 800 | `#1E293B` | Strong labels |
| Slate 900 | `#0F172A` | Headings |
| Slate 950 | `#020617` | Rare max contrast |
| White | `#FFFFFF` | Surface / cards |

| Semantic surface | Hex |
|------------------|-----|
| Background | `#F8FAFC` |
| Surface | `#FFFFFF` |
| Border | `#E2E8F0` |
| Heading | `#0F172A` |
| Body | `#475569` |
| Muted | `#64748B` |

---

## 5. Typography

| Role | Family |
|------|--------|
| UI | Inter, "Segoe UI", system-ui, sans-serif |
| Mono | ui-monospace / JetBrains Mono / Cascadia Mono — ports, serials, ERROR codes, tech metrics |

### Type scale

| Token | Size | Use |
|-------|------|-----|
| Display | 32px | Greeting (“Good to see you!”) |
| Page | 28px | Rare large page titles |
| Section | 18px | Section headers |
| Card | 15px | Card titles |
| Body | 14px | Default body |
| Small | 13px | Secondary lines |
| Label | 12px | Badges, meta, uppercase labels |
| Metrics | 24px | Download / upload totals (when shown) |

Line-height ~1.5. Weights: regular / medium / semibold / bold as hierarchy needs.

---

## 6. Spacing (8-point)

Allowed: **4, 8, 12, 16, 20, 24, 32, 40, 48, 64** px.  
Card padding typically 16–20. Section gaps 16–24. Sidebar / topbar chrome uses 12–16 gutters.

---

## 7. Radii

| Token | Value | Component rule |
|-------|-------|----------------|
| sm | 6px | Small chips, tight controls |
| md | 8px | Buttons, inputs, switches track ends |
| lg | 12px | **Cards** (default) |
| xl | 16px | Large panels / hero |
| 2xl | 24px | Search pill, rare oversized shells |

Badges: pill (`9999px` / full). Prefer card radius **12px** to match mockup.

---

## 8. Borders & shadows

- Border default: `1px solid #E2E8F0`
- Card shadow (soft): `0 1px 2px rgba(15, 23, 42, 0.05), 0 4px 16px rgba(15, 23, 42, 0.04)`
- Elevated: slightly stronger blur for dropdowns / hovering cards
- Modal: `0 8px 32px rgba(15, 23, 42, 0.12)`

Avoid heavy drop shadows; keep calm SaaS elevation.

---

## 9. Buttons

| Variant | Height | Radius | Fill / border | Text |
|---------|--------|--------|---------------|------|
| Primary | 40px | 8px (md) | `#2563EB` → hover `#1D4ED8` → pressed `#1E40AF` | white |
| Secondary | 40px | 8px | white + `#E2E8F0` border | slate-900 |
| Destructive | 40px | 8px | soft `#FEF2F2` fill, `#DC2626` text/border | error-600 — **not** solid red blocks |
| Icon | 36–40px square | 8px | ghost / subtle hover slate-100 | slate-600 |

Padding: primary/secondary ~`0 16px` (px-4). Icon buttons centered.

---

## 10. Status badges & switches

- **Badges:** pill shape; label 12px; optional 6–8px status dot (never color-alone).
  - Healthy / Connected / Service running / ADB ok → success greens on success-50/100 wash
  - Waiting / Warning → warning wash
  - Error → error wash
- **Switches:** track **blue (`#2563EB`) when ON**, slate when OFF. **Not green when on.**

---

## 11. Cards & connection hero

- Default card: white surface, border `#E2E8F0`, radius 12px, soft card shadow.
- **Connection hero (healthy):** light success tint (`#F0FDF4` / `#DCFCE7` edge), “Connected” + “Internet tunnel active” **only** when EVENT_STATUS_MAP three-layer Sharing healthy.
- Idle / waiting / error heroes: neutral or warning/error wash — never fake Connected green.

---

## 12. Shell chrome

| Chrome | Spec |
|--------|------|
| Sidebar | **240px** — logo, Dashboard / Devices / Traffic / Logs / Settings, promo card, ADB footer |
| Top bar | **72px** — search pill, theme toggle, service status badge |
| Search pill | Large rounded (2xl); MVP: decorative / non-functional or **Later** |
| Active nav | Primary-50 wash + Primary-500 text/icon |

---

## 13. Chart colors (Traffic — Later)

| Series | Color |
|--------|-------|
| Download | Success green (`#22C55E` / `#16A34A`) |
| Upload | Accent indigo (`#4F46E5`) |

Live throughput charts = **Deferred** unless Desktop can show honest zeros/placeholder without fake data. Label **Later** in UI if stubbed.

---

## 14. Icons & motion

- **Icons:** Lucide (thin stroke), consistent size 16–20 in nav/actions.
- **Focus outline:** `#93C5FD` (primary-300-ish ring) — visible keyboard focus.
- **Motion:** short, calm (150–200ms) opacity/transform; no bounce marketing motion.

---

## 15. Visual hierarchy ratio

Target paint distribution:

| Share | Role |
|-------|------|
| **~70%** | Neutral (slate canvas, white cards, muted text) |
| **~20%** | Blue primary (actions, active nav, links) |
| **~7%** | Green (health only) |
| **~3%** | Warn / error |

---

## 16. Pasteable `@theme` (Tailwind v4)

```css
@theme {
  --color-background: #f8fafc;
  --color-surface: #ffffff;
  --color-border: #e2e8f0;

  --color-primary-50: #eff6ff;
  --color-primary-100: #dbeafe;
  --color-primary-500: #2563eb;
  --color-primary-600: #1d4ed8;
  --color-primary-700: #1e40af;

  --color-accent: #4f46e5;

  --color-success-50: #f0fdf4;
  --color-success-100: #dcfce7;
  --color-success-500: #22c55e;
  --color-success-600: #16a34a;
  --color-success-800: #15803d;

  --color-warning-50: #fffbeb;
  --color-warning-100: #fef3c7;
  --color-warning-500: #f59e0b;
  --color-warning-600: #d97706;
  --color-warning-800: #b45309;

  --color-error-50: #fef2f2;
  --color-error-100: #fee2e2;
  --color-error-500: #ef4444;
  --color-error-600: #dc2626;
  --color-error-800: #b91c1c;

  --color-slate-50: #f8fafc;
  --color-slate-100: #f1f5f9;
  --color-slate-200: #e2e8f0;
  --color-slate-300: #cbd5e1;
  --color-slate-400: #94a3b8;
  --color-slate-500: #64748b;
  --color-slate-600: #475569;
  --color-slate-700: #334155;
  --color-slate-800: #1e293b;
  --color-slate-900: #0f172a;
  --color-slate-950: #020617;

  --color-focus-ring: #93c5fd;

  --font-sans: "Inter", "Segoe UI", system-ui, -apple-system, sans-serif;
  --font-mono: "JetBrains Mono", ui-monospace, "Cascadia Mono", Consolas, monospace;

  --text-display: 32px;
  --text-page: 28px;
  --text-section: 18px;
  --text-card: 15px;
  --text-body: 14px;
  --text-small: 13px;
  --text-label: 12px;
  --text-metrics: 24px;

  --radius-sm: 6px;
  --radius-md: 8px;
  --radius-lg: 12px;
  --radius-xl: 16px;
  --radius-2xl: 24px;

  --shadow-card: 0 1px 2px rgba(15, 23, 42, 0.05), 0 4px 16px rgba(15, 23, 42, 0.04);
  --shadow-elevated: 0 4px 16px rgba(15, 23, 42, 0.08);
  --shadow-modal: 0 8px 32px rgba(15, 23, 42, 0.12);

  --sidebar-width: 240px;
  --topbar-height: 72px;
}
```

Map shadcn `--primary` → `--color-primary-500`. Optional dark theme = Later; **boot light**.

---

## 17. Anti-patterns

- Indigo / purple as the primary brand button color  
- Green switches or green primary CTAs  
- Fake Connected / live traffic numbers without real data  
- Solid red primary-style Stop blocks (use soft destructive)  
- Competing token sheets that diverge from `VISUAL_APPROVED_V3.md`

---

*Behavior SoT unchanged: `EVENT_STATUS_MAP.md`, `ERROR_UX.md`, `TEAM_BRIEF.md`. Layout → `SHELL_SCREENS.md`. Theme summary → `THEME.md`.*
