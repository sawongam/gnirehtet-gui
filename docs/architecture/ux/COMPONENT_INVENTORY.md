# Component Inventory — gnirehtet-gui

**Audience:** Desktop Engineer  
**Status:** Map shadcn-svelte (recommended) → current MVP actions  
**Design system:** `DESIGN_SYSTEM.md` (option A)  
**Does not invent:** tray, multi-share, charts, wireless wizard, Settings redesign

Cross-ref: `SHELL_SCREENS.md`, `COPY_RULES.md`, `ERROR_UX.md`, `EVENT_STATUS_MAP.md`.

---

## 1. Principle

Replace hand-rolled flat panels in `apps/desktop/src/routes/+page.svelte` with a small owned set of shadcn-svelte components. **Orchestrator IPC and stores stay.** UI chrome only.

---

## 2. Inventory

### Button

| Variant | Actions |
|---------|---------|
| `default` (accent) | **Run**; code-specific Retry / Restart Run; **Repair tunnel** when Interrupted |
| `secondary` | Repair tunnel (Ready); Install helper; I’ve allowed it |
| `destructive` | **Stop** (when session can stop); Stop in error recovery when tearing down |
| `ghost` | Refresh devices; Clear logs; Advanced triggers; Install when demoted |
| `outline` (optional) | Start Relay / Stop Relay inside Advanced |

**Map from current shell:**

| Current control | Variant | Label (`COPY_RULES`) |
|-----------------|---------|----------------------|
| Run | default | Run |
| Stop | destructive | Stop |
| Repair tunnel | secondary → default if Interrupted | Repair tunnel |
| Install | ghost / secondary | Install helper |
| Start Relay | outline (Advanced) | Start Relay |
| Stop Relay | outline (Advanced) | Stop Relay |
| Refresh | ghost | Refresh devices |

Busy: disable + label “Starting…” / “Stopping…” (existing behavior).

**Ban:** one `actions wrap` row where all four look the same except `.primary`.

---

### Badge / StatusDot

| Use | Source |
|-----|--------|
| Session chip | `deriveSessionChip` / EVENT_STATUS_MAP |
| Device adb state | `deviceStateLabel` / `deviceStateClass` (`ready` / `unauthorized` / `offline`) |
| Layer health word | Relay Listening · Tunnel OK · Device VPN Active / Waiting |

Prefer **Badge** with text; optional 6px StatusDot beside text (never color-alone).

Chip labels: Idle · Starting · Waiting for VPN · Sharing · Interrupted · Stopping · Error.

---

### Card (or dense list row)

| Use | Notes |
|-----|-------|
| Device row | Click/select entire card; serial mono; model muted; state Badge |
| Selected device | Border `--accent-muted` or ring; not radio circles |

Replace `<input type="radio">` list. Keyboard: arrow/enter acceptable Later; click select is MVP.

Empty: Card-like empty state with `NO_DEVICES` copy — not a blank panel.

---

### Alert / Callout

| Use | Notes |
|-----|-------|
| ERROR_UX banners | Title includes **`[CODE]`**; explanation; recoveryHint; recovery Buttons |
| VPN pending | `warning` variant |
| Unauthorized | `warning` |
| TUNNEL_LOST / crashes | `destructive` / error |

Wire copy from `errorUxFor(code)` — do not invent strings that contradict `ERROR_UX.md`.

One banner slot (`SHELL_SCREENS.md` priority).

---

### Separator

| Use |
|-----|
| Between primary session actions and Advanced relay |
| Optional under title chrome |

---

### Collapsible

| Use | Default |
|-----|---------|
| **Advanced — Relay only** | Collapsed |
| **Logs** | Expanded while debugging / first-run; may default collapsed once Sharing stable (Desktop choice) |

Do not put Run/Stop inside Collapsible.

---

### ScrollArea

| Use |
|-----|
| Log pane (`LOG_CAP` 500 remains) |

Mono 12px; Clear as ghost Button in header. Filter = Later.

---

### Tooltip

| Target | Content (user voice) |
|--------|----------------------|
| Relay layer | “PC process that phones connect to.” |
| Tunnel layer | “USB reverse path from phone to this PC.” |
| Device VPN | “Phone VPN permission + helper handshake.” |
| Direction badge | “Phone uses this PC’s internet — not the other way.” |

**Never** put `HANDSHAKE_LIVENESS_MVP`, architecture filenames, or “not Sharing” essays in tooltips — one sentence max.

---

### Dialog

| Use | MVP? |
|-----|------|
| Confirm quit if session active | **Yes** — “Stop sharing and quit?” |
| Settings / About | Already in MVP_UX as separate surfaces; not part of this chrome pass unless already wired |

No tray. No multi-device share dialog.

---

## 3. Composition sketch (Svelte)

Illustrative only — Desktop owns file split:

```
Shell
├── TitleBar (Brand, DirectionBadge, SessionBadge)
├── LayerStrip (3× LayerCell + Tooltip)
├── Body
│   ├── DevicePanel (Refresh ghost, DeviceCard[])
│   └── SessionPanel
│       ├── Button Run|Stop
│       ├── Button Repair, Install
│       └── Collapsible Advanced → Start/Stop Relay
├── AlertSlot (single)
└── Collapsible Logs → ScrollArea
Dialog QuitConfirm
```

---

## 4. Out of inventory (do not add for this polish)

- System tray components  
- Multi-share device matrix  
- Throughput charts / sparklines  
- Toast spam for every log line  
- Theme picker UI (dark hard-default)  
- New orchestrator commands  

---

## 5. Acceptance for Desktop

- [ ] shadcn Button variants visibly hierarchy Run vs secondary  
- [ ] Device cards replace radio list  
- [ ] One Alert slot with `[CODE]`  
- [ ] Advanced relay + Logs use Collapsible  
- [ ] Quit Dialog when session active  
- [ ] No new MVP features beyond current IPC surface  

---

*Implementation: Desktop. Spec: UX. State/errors SoT unchanged.*
