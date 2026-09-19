# Shell Screens — gnirehtet-gui

**Audience:** Desktop Engineer  
**Status:** Single-window main shell layout + states for ASAP polish  
**SoT for chips / Sharing:** `EVENT_STATUS_MAP.md`  
**SoT for errors:** `ERROR_UX.md` + `apps/desktop/src/lib/errorUx.ts`  
**Copy:** `COPY_RULES.md`

Does **not** invent tray, multi-share, charts, or networking changes.

---

## 1. Single main shell (ASCII)

```
┌ titlebar / app chrome ──────────────────────────────────────────┐
│ Brand · direction badge · session chip                          │
├─────────────────────────────────────────────────────────────────┤
│ Three-layer strip (Relay | Tunnel | Device VPN)                 │
├──────────────────────┬──────────────────────────────────────────┤
│ Device list          │ Session column                           │
│ (selected card)      │ Primary: Run / Stop                      │
│                      │ Secondary: Repair · Install              │
│                      │ Advanced disclosure: Relay only          │
├──────────────────────┴──────────────────────────────────────────┤
│ Contextual banner (error / VPN / unauthorized) — ONE slot       │
├─────────────────────────────────────────────────────────────────┤
│ Logs (collapsible, monospace, filter later)                     │
└─────────────────────────────────────────────────────────────────┘
```

### Zone rules

| Zone | Content | Notes |
|------|---------|-------|
| **Title / chrome** | Brand (`gnirehtet-gui`), **Internet: This PC → Phone**, session chip | Fix `app.html` title. Drop “Phase 3 — …” subtitle jargon. |
| **Three-layer strip** | Relay · Tunnel · Device VPN | Always visible. Tooltips for help — no HANDSHAKE_LIVENESS dump. |
| **Device list** | Selectable cards/rows; Refresh ghost button in header | Replace radio-list. Show serial (mono) + state badge + model. |
| **Session column** | Hierarchy: **one** primary CTA, secondary actions quieter, Advanced collapsed | Ban equal-weight Install/Repair/Run/Stop grid. |
| **Banner** | Single contextual Alert from ERROR_UX | Stacking multiple banners = noise; prioritize (ADB > auth > recovery > VPN wait > action). |
| **Logs** | Collapsed by default after first success optional; always available | Monospace; Clear; filter = Later. |

Window: ~920×640 default, min ~720×520 (`THEME.md`).

---

## 2. Session column hierarchy

```
[  Run  ]                    ← accent primary (or Stop when session active)
Repair tunnel · Install      ← secondary / ghost text buttons
Refresh devices              ← may live on device panel header instead

▸ Advanced
    Start Relay · Stop Relay ← only here; not on primary row
```

**Decision (buttons):** Keep **Run** / **Stop** as shell labels matching current IPC (`runSession` / `stopClient`). Subtitle under primary: **“Share this PC’s network”**. The healthy chip says **Sharing** — never rename the chip to “Connected”.

| Action | Visual weight |
|--------|----------------|
| Run | Primary (accent) when Idle / Ready / Error-retry |
| Stop | Destructive or strong secondary when Starting / Waiting VPN / Sharing / Interrupted / Stopping |
| Repair tunnel | Secondary; **primary only** when chip = Interrupted (`TUNNEL_LOST`) |
| Install helper | Ghost / secondary |
| Refresh devices | Ghost on device panel |
| Start/Stop Relay | Inside Advanced collapsible only |

---

## 3. First-run / empty

**Not** a SaaS wizard page.

Use either:

1. **Inline checklist banner** on Main (ADB path · USB debugging · cable · helper APK), or  
2. **Compact empty state** in the device column when `NO_DEVICES` / no ADB.

Checklist items collapse once green. Do not navigate away from Main for first-run.

---

## 4. States

Chip vocabulary SoT: `EVENT_STATUS_MAP.md` — Idle, Starting, Waiting for VPN, Sharing, Interrupted, Stopping, Error.

**Sharing** only when **Relay Listening + Tunnel OK + Device VPN Active (handshake)**. Never on intent-sent, relay-only, or client-id absent.

### Idle empty

| | |
|--|--|
| **When** | No devices / ADB missing / nothing selected |
| **Chip** | Idle |
| **Primary CTA** | Disabled Run (or hidden); emphasize Refresh / setup tips |
| **Show** | Empty state or `NO_DEVICES` / `ADB_MISSING` banner; direction badge |
| **Do NOT** | Claim Sharing; show equal action grid; dump logs as the hero |

### Unauthorized

| | |
|--|--|
| **When** | Selected (or only) device `unauthorized` |
| **Chip** | Idle (session) + device badge Unauthorized |
| **Primary CTA** | I’ve allowed it (refresh) |
| **Show** | `[DEVICE_UNAUTHORIZED]` banner; unlock phone tip |
| **Do NOT** | Enable Run; say Connected |

### Ready

| | |
|--|--|
| **When** | adb OK, device `device`, Idle, layers not in a live session |
| **Chip** | Idle |
| **Primary CTA** | **Run** |
| **Show** | Secondary Repair / Install; layers Off / unknown muted |
| **Do NOT** | Green Sharing; verbose “Run ≈ install-if-needed…” paragraph in UI |

### Starting

| | |
|--|--|
| **When** | Run pipeline in progress |
| **Chip** | Starting |
| **Primary CTA** | Busy Run (disabled / “Starting…”) + Stop available |
| **Show** | Layer strip updating; optional quiet log append |
| **Do NOT** | Sharing; unlock Install as primary |

### Waiting VPN

| | |
|--|--|
| **When** | Client started; Device VPN not Active (`VPN_PERMISSION_PENDING`) |
| **Chip** | Waiting for VPN |
| **Primary CTA** | I’ve allowed it (recheck); Stop secondary |
| **Show** | Banner: allow Connection request on phone; key-icon tip |
| **Do NOT** | Sharing; show `HANDSHAKE_LIVENESS_MVP` or “Pending until handshake…” |

User-facing VPN pending line: **“Waiting for phone to finish connecting”** (`COPY_RULES.md`).

### Sharing

| | |
|--|--|
| **When** | All three layers healthy + handshake (`EVENT_STATUS_MAP`) |
| **Chip** | Sharing (success) |
| **Primary CTA** | **Stop** |
| **Show** | Healthy layer strip; Repair available but secondary/rare |
| **Do NOT** | Say Connected / Online / VPN Connected; hide Stop |

### Interrupted

| | |
|--|--|
| **When** | Was Sharing; tunnel lost (`TUNNEL_LOST`) |
| **Chip** | Interrupted |
| **Primary CTA** | **Repair tunnel** |
| **Show** | `[TUNNEL_LOST]` banner; Stop secondary; replug tip |
| **Do NOT** | Keep Sharing green; kill-switch / “PC blocked” language |

### Error

| | |
|--|--|
| **When** | Blocking `Error` with recovery |
| **Chip** | Error (title from ERROR_UX) |
| **Primary CTA** | Code-specific recovery (`ERROR_UX.md`) |
| **Show** | `[CODE]` in banner; recovery hint; Copy error if useful |
| **Do NOT** | Invent new codes; claim Sharing |

### Stopping

| | |
|--|--|
| **When** | Stop pipeline in progress |
| **Chip** | Stopping |
| **Primary CTA** | Stop busy (“Stopping…”) |
| **Show** | Layers clearing |
| **Do NOT** | Flash Sharing |

---

## 5. Layer strip behavior

| Layer | Healthy label | Off / pending | Error |
|-------|---------------|---------------|-------|
| Relay | Listening (+ port) | Off | Error / crashed / port in use |
| Tunnel | OK | — / unknown | Lost / Failed |
| Device VPN | Active / On | Off / Waiting | Error / denied |

- Tooltips: one short sentence each — no architecture doc names.  
- Hide developer “owned” unless Diagnostics (Later); MVP may keep muted “owned” only in Advanced.  
- Partial relay listening with 0 clients ≠ Sharing (`EVENT_STATUS_MAP`).

---

## 6. Banner priority (one slot)

1. ADB missing / invalid  
2. Device unauthorized / offline (selected)  
3. Recovery Error (INSTALL_FAILED, RELAY_CRASHED, …)  
4. Interrupted / TUNNEL_LOST  
5. VPN pending  
6. Empty NO_DEVICES  
7. Transient action error  

Never stack all of them. Logs stay in the log zone.

---

## 7. Quitting

If session active (Starting / Waiting for VPN / Sharing / Interrupted / Stopping): **Dialog** — “Stop sharing and quit?” → teardown then quit (`MVP_UX` / SCREEN_SPEC). MVP only — no tray.

---

## 8. Explicit bans

- Equal-weight button grids  
- Dumping HANDSHAKE / architecture identifiers into UI chrome  
- Claiming Sharing early  
- Wizard SaaS first-run page  
- “Connected”, “Online”, “VPN Connected”, kill-switch language (`COPY_RULES.md`)

---

*Layout only. Tokens → `THEME.md`. Components → `COMPONENT_INVENTORY.md`.*
