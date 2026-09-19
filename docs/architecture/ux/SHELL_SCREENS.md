# Shell Screens — gnirehtet-gui (SaaS layout)

**Audience:** Desktop Engineer  
**Status:** v2 **standard SaaS** layout (Sangam 2026-09-20)  
**Behavior SoT:** `EVENT_STATUS_MAP.md`, `ERROR_UX.md`, `COPY_RULES.md`, `MVP_UX.md`  
**Visual SoT:** `THEME.md` (indigo SaaS light)

MVP actions only — no new features.

---

## 1. Information architecture (SaaS)

Single page app shell (no marketing landing):

```
┌────────────────────────────────────────────────────────────┐
│ Page header                                                │
│  gnirehtet-gui                                             │
│  [Internet: This PC → Phone]     Session·…   ADB·…   v…   │
│  Optional one-line subtitle: Share this PC’s network…      │
├────────────────────────────────────────────────────────────┤
│ Connection status (3 cards)                                │
│  ┌ Relay ┐  ┌ Tunnel ┐  ┌ Device VPN ┐                    │
├──────────────────────┬─────────────────────────────────────┤
│ Devices              │ Session                             │
│  list / empty state  │  primary CTA(s)                     │
│  Refresh (link/btn)  │  secondary: Repair · Install        │
│                      │  ▸ Advanced (relay-only)            │
├──────────────────────┴─────────────────────────────────────┤
│ Alert slot (one) — ERROR_UX / VPN pending / unauthorized   │
├────────────────────────────────────────────────────────────┤
│ ▸ Logs                                                     │
└────────────────────────────────────────────────────────────┘
```

Feel: **Stripe-like product page** — header, status cards, two-column content, alert, logs — not a form dump.

---

## 2. Hierarchy rules

1. **Header** establishes product + direction + session truth.  
2. **Three status cards** are scannable health (Relay · Tunnel · Device VPN).  
3. **Session column owns primary actions** for the current chip.  
4. **One alert slot** below columns — never stack 4 banners.  
5. **Logs** secondary, collapsed by default once stable (MVP: may start expanded if debugging).

---

## 3. Header

| Element | Spec |
|---------|------|
| Title | `gnirehtet-gui` 20px semibold |
| Direction | Soft pill: **Internet: This PC → Phone** (unicode →) |
| Session badge | `Session · {Idle\|Starting\|Waiting for VPN\|Sharing\|Interrupted\|Stopping\|Error}` |
| ADB badge | `ADB · ok` / `ADB · missing` — **separate** from session |
| Version | muted meta |

Do **not** put ERROR_UX codes in the session badge.

---

## 4. Status cards (layer strip)

Each card: white surface, 12px radius, left accent bar (relay blue / tunnel violet / vpn teal), label uppercase 12px muted, value semibold.

| Layer | Healthy value | Idle / off |
|-------|---------------|------------|
| Relay | Listening + `:port` | stopped |
| Tunnel | OK | — |
| Device VPN | Active | Off / Waiting |

**Sharing chip** only when all three healthy + handshake (`EVENT_STATUS_MAP`).

---

## 5. Devices column

- Card titled **Devices** + text button **Refresh devices**.  
- Empty: dashed muted well + short setup copy (USB debug / authorize).  
- Rows: model + serial mono + ready/unauthorized badge; selected = indigo wash + border.  
- Radio semantics without looking like a 1990s form — use selectable rows/cards.

---

## 6. Session column (actions)

### Idle + device ready
- Primary: **Run** (indigo, large)  
- Helper line: Share this PC’s network  
- Secondary outline: **Repair tunnel** · **Install helper**  
- Advanced disclosure: Start/Stop Relay only

### Idle + ADB missing / no device
- Run **disabled**  
- Repair / Install **disabled**  
- Alert carries ADB_MISSING / NO_DEVICES

### Waiting for VPN
- Primary outline: **I’ve allowed it**  
- Destructive: **Stop**  
- Alert: `[VPN_PERMISSION_PENDING]…` guidance only (no duplicate buttons in alert)

### Sharing
- Primary destructive: **Stop**  
- Secondary: Repair tunnel  
- Session badge = Sharing (green)

### Interrupted
- Primary: **Repair tunnel**  
- Secondary: Stop  
- Alert: `[TUNNEL_LOST]`

---

## 7. Alert slot

Single Alert component:

- Title: `[CODE] Title` from ERROR_UX  
- Body: explanation  
- Optional muted recovery hint  
- Actions only if Session column cannot own them (prefer Session)

---

## 8. Logs

Collapsible. Muted well, 12–13px mono, Clear link. No jargon filenames in user chrome.

---

## 9. State matrix (visual)

| Chip | Status cards | Session primary |
|------|--------------|-----------------|
| Idle | mostly off | Run (if ready) |
| Starting | updating | disabled / Starting… |
| Waiting for VPN | Relay+Tunnel OK; VPN Waiting | I’ve allowed it + Stop |
| Sharing | all healthy | Stop |
| Interrupted | Tunnel lost | Repair tunnel |
| Error | depends | code recovery |

---

## 10. Explicit non-goals

- Sidebar nav / multi-page SaaS IA for MVP (single page is enough)  
- Marketing hero / pricing chrome  
- New MVP features (tray, multi-device, charts)  
- Dense terminal aesthetic as default

---

*Desktop implements with shadcn Card, Badge, Button, Alert, Collapsible.*
