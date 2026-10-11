# Desktop Lifecycle — Gnirehtet GUI Shell

**Status:** Draft v0.2  
**Owner:** Desktop Application Engineer  
**Date:** 2026-09-12  
**Binding inputs:**
- [`research/lead-architect-baseline.md`](research/lead-architect-baseline.md)
- [`research/gnirehtet-core-boundaries.md`](research/gnirehtet-core-boundaries.md) §3, §8–9
- [`research/architect-decisions-2026-09-12.md`](research/architect-decisions-2026-09-12.md)

**Related docs:** [DESKTOP_ARCHITECTURE.md](DESKTOP_ARCHITECTURE.md) · [FRONTEND_ARCHITECTURE.md](FRONTEND_ARCHITECTURE.md) · [STATE_MODEL.md](STATE_MODEL.md)

---

## 1. Startup sequence

Ordered boot of the host-orchestrator (Tauri backend), then UI hydration:

```mermaid
sequenceDiagram
  participant App as Tauri app
  participant Orch as host-orchestrator
  participant UI as Svelte UI
  participant Adb as adb

  App->>Orch: init
  Orch->>Orch: load settings from disk
  Orch->>Orch: resolve APK path (setting > GNIREHTET_APK > bundle)
  Orch->>Orch: orphan sidecar scan (kill stale PID if ours)
  Orch->>Orch: ensure_adb (setting > ADB env > PATH)
  Orch-->>UI: settings + AdbInfo / Error
  Orch->>Adb: list_devices (if adb OK)
  Orch-->>UI: DeviceChanged
  Orch-->>UI: RelayState(relay_stopped)
  Note over Orch,UI: MVP: no multi-device autorun; optional OS autostart of the app shell only (later)
  UI->>UI: restore window or tray per settings
```

| Step | Action | Failure handling |
|------|--------|------------------|
| 1 | Load settings | Defaults if missing/corrupt; log warning |
| 2 | Resolve sidecar + APK paths | Fatal banner if sidecar binary missing |
| 3 | Orphan recovery | See §8 — only **our** recorded sidecar |
| 4 | `ensure_adb()` | UI banner `adb_not_found`; Settings focus |
| 5 | `list_devices()` | Empty list OK; show unauthorized distinctly (**direct adb**, discovery only) |
| 6 | Emit initial `RelayState` | Always `relay_stopped` unless we adopted **our** orphan (unusual). Never mark a foreign process `ownedBySession` |
| 7 | Show window / start minimized | Per settings |

**Out of MVP:** automatic `autorun` / multi-device start-on-plug. Optional later “start relay on launch” must be an explicit setting, not silent.

---

## 2. Window vs tray vs quit

| Action | Behavior |
|--------|----------|
| **Close window** (X) | If `closeToTray`: hide window, keep orchestrator + any **owned** relay running; else treat as **Quit** |
| **Minimize** | Normal minimize; optional “minimize to tray” if platform supports |
| **Tray → Show** | Restore/focus main window |
| **Tray → Stop** | **Session teardown** (Lead Architect decision): `stop(selected)` **and** `stop_relay` if we started that relay |
| **Tray → Quit** / **File → Quit** | Full teardown (§5) then exit |

Tray status icon should reflect aggregate: relay running + any device `connected` vs idle/error (tooltips with text).

---

## 3. Close-to-tray / background operation

- Background mode = window hidden, event loop alive, sidecar may still be running.
- Continue emitting `LogLine` / `DeviceChanged`. Device **poll interval** follows §3.1 — do **not** go blind during an active session.
- Notifications (§4) remain available while backgrounded.
- User must be able to Quit from tray so a hidden relay is not left unexplained.

### 3.1 Device poll when hidden (Lead Architect decision)

Back off when **idle + hidden**. Do **not** go blind while a tether session is active.

Spike starting point (tune if noisy / battery / USB flakiness):

| Window / session | Poll interval | Notes |
|------------------|---------------|-------|
| **Visible** | ~**2s** | Full discovery (`adb devices` via `list_devices`) |
| **Hidden + idle** | ~**10s** (or pause until focus) | No active tether; cheap refresh is enough |
| **Hidden + active tether** | ~**5s** | Keep watching unauthorized / unplug / reverse loss |

“Active tether” = any device in `connecting` | `awaiting_vpn_permission` | `connected` | `reconnecting_tunnel` | `disconnecting`, or an owned `relay_running`.

Relay supervision (child exit) is event-driven and is **not** paused when hidden.

---

## 4. Notifications

Use OS notifications sparingly for MVP:

| Event | Notify? |
|-------|---------|
| Entered `connected` | Optional success toast (once per Run) |
| Fell to `error` while window hidden | Yes — short message + code |
| `awaiting_vpn_permission` | Yes — “Check your phone and accept the VPN request” (heuristic; see [STATE_MODEL.md](STATE_MODEL.md) §2.1) |
| `awaiting_vpn_permission` timeout | Yes — recoverable error; Retry from the app |
| Relay unexpected exit | Yes |
| Routine LogLine | No |

Respect OS permission prompts (esp. macOS/Windows). Do not notify on every device poll.

---

## 5. One-click Run ≈ upstream `run`; Stop / Quit teardown

### Run

Orchestrator implements upstream `run` semantics by **spawning `gnirehtet run`** (or the equivalent CLI verb sequence on the sidecar — **not** hand-rolled adb):

1. Ensure relay up (`start_relay` if not `relay_running`) on the **session port** (explicit `run`/`start` port sets it; else reuse session port; else 31416).
2. Install-if-needed (`gnirehtet install` when APK missing/wrong versionCode).
3. Tunnel (`gnirehtet tunnel`) for serial.
4. `start` intent via `gnirehtet start` (DNS/routes/port extras as configured).
5. Drive device lifecycle toward `connected` / `awaiting_vpn_permission` (heuristic + phone prompt).

UI exposes a single **Run** control that calls `run(serial, opts)` — not a manual checklist of adb steps.

If a **foreign** relay is already bound to the session port, do not kill it; surface `port_in_use` / `foreign_relay` and keep `ownedBySession: false`.

### Stop — session-scoped ownership (Lead Architect decision)

Default one-click Stop ≈ upstream `run` teardown (**no orphans** from a session we started):

1. Stop the device client (`gnirehtet stop` / STOP intent) → `disconnecting` → `device_detected`/`idle`.
2. Stop the relay **if and only if this app started that relay** (`ownedBySession`).
3. Do **not** kill a relay we did not start.

Optional advanced “stop device only” is **later** UI; MVP default is the composite above. Primitive `stop(serial)` remains on the orchestrator API.

### Quit

1. For each known active serial: best-effort device `stop`.
2. `stop_relay()` **only if we started** the relay.
3. Flush log file if enabled.
4. Persist settings.
5. Exit process.

Close-to-tray is **not** Quit: owned relay may keep running until tray Stop or Quit.

---

## 6. Sidecar spawn / stop

### Spawn

- Resolve bundled `gnirehtet` / `gnirehtet.exe` (Tauri `externalBin` / target-triple suffix).
- Spawn with piped stdout/stderr; set env from resolved paths: `ADB` and `GNIREHTET_APK` after applying **setting > env > PATH/bundle** (so the child sees the same resolution).
- **Lead Architect decision (orchestration, MVP):** spawn the sidecar for `install | reinstall | start | stop | tunnel | relay | run`. Direct `adb` only for discovery/health (`devices`, unauthorized, adb missing). Do **not** reimplement those verbs with hand-rolled adb in the Tauri backend.
- **M2:** extract a shared Rust lib without changing the UI-facing API. Until then, CLI spawn is the only path. The v0.1 “docs allow both” fork is closed.
- UI never spawns.

### Stop — no graceful API

Upstream `relaylib::relay(port)` is a **blocking mio loop with no graceful stop**. Therefore, for a relay **we own**:

| Method | MVP stance |
|--------|------------|
| SIGTERM / TerminateProcess | Attempt first on Unix/Windows |
| SIGKILL / force kill | **OK and expected** if still alive after short grace (e.g. 1–2s) |
| stdin “quit” command | **Does not exist** — do not invent |
| Foreign relay | **Do not kill** |

**Implications (document in UX/help):**

- In-flight TCP/UDP relay connections drop immediately.
- Android client may show tunnel failure until Stop/Run again.
- Always clear orchestrator’s PID bookkeeping after kill; emit `RelayState(relay_stopped)`; clear `ownedBySession`.
- Port may remain in `TIME_WAIT` briefly — surface `port_in_use` on rapid restart.
- Session port is **not** hot-swapped; change requires Stop + re-run ([STATE_MODEL.md](STATE_MODEL.md) §3.2).

Record sidecar PID (and optional start time) in memory; optionally a small pidfile under app data for orphan detection (§8).

---

## 7. Logging

```text
sidecar stdout/stderr ──► line buffer ──► parse ──► LogLine event ──► UI ring buffer
                                              └──► optional rotating file (shell-owned)
orchestrator messages ────────────────────────────► LogLine(source=orchestrator)
adb child pipes (if any) ─────────────────────────► LogLine(source=adb)
```

| Topic | Policy |
|-------|--------|
| Upstream format | `YYYY-MM-DD HH:MM:SS.mmm LEVEL target: message` (Rust SimpleLogger); non-error→stdout, error→stderr |
| Verbosity | Stock builds hardcode Info; **do not** require patching relay for GUI viewing |
| UI filter | Client-side level filter only |
| File log | Optional setting; path under app data; retention simple (size/day cap) |
| Android logcat | Optional later diagnostic; not required for MVP Run path |

Known lines may inform state heuristics inside orchestrator (`Starting relay server on port…`, `Execution error:…`) — still emit raw `LogLine` to UI. VPN-permission inference is heuristic only (Lead Architect decision); a real status channel is later if noisy.

---

## 8. Crash / orphan process recovery

| Scenario | Recovery |
|----------|----------|
| App crash while **our** relay running | On next start: detect PID file / process named our sidecar path listening on configured port; **kill** if identified as ours; warn user in logs |
| Relay crash while app alive | Supervisor gets exit status → `RelayState(relay_error)` + `Error`; devices marked `error`; offer Restart |
| Stale reverse / client | User **Reset tunnel** or Run again; do not auto-autorun in MVP |
| Orphan adb servers | Do not kill user `adb` server aggressively; only manage processes we spawned |
| Foreign `gnirehtet` on 31416 | Do **not** kill; do not set `ownedBySession`; surface `foreign_relay` / `port_in_use` if we need that port |

**Assumption:** PID file stores `{pid, port, exe_path, started_at}`; only kill if `exe_path` matches our bundled binary.

---

## 9. Settings load / save

| Moment | Behavior |
|--------|----------|
| Startup | Load JSON/TOML from app config dir; merge defaults |
| Settings view Save | Validate paths via orchestrator; write disk; apply ADB/APK path for subsequent sidecar spawns |
| Quit | Persist any dirty settings |
| Path resolution | **Lead Architect decision (not a spike):** **explicit app setting > `ADB` / `GNIREHTET_APK` env > PATH / bundled APK default** |

Never persist device lifecycle, relay PID, ownership, or session port as “truth” across runs.

---

## 10. Platform notes

| Platform | Notes |
|----------|-------|
| **Linux** | SIGTERM then SIGKILL; tray via StatusNotifier/AppIndicator variance — degrade to window-only if tray missing; udev/user in `plugdev` may affect adb — document, don’t elevate |
| **Windows** | `gnirehtet.exe` sidecar; TerminateProcess after graceful attempt; path quoting for adb; USB driver / “no devices” common — link help; do not require Admin for relay |
| **macOS (best-effort)** | Signing/notarization/quarantine for sidecar TBD; tray menu OK; notifications need permission; treat full parity as post-MVP until spikes prove spawn + adb + APK install |

All platforms: PC creates **no TUN**; VPN permission is Android-only (core-boundaries §9).

---

## 11. Lifecycle checklist (MVP)

- [ ] Boot: settings → orphan cleanup (**ours** only) → ensure_adb → list_devices → UI
- [ ] Run = upstream `run` via **`gnirehtet` sidecar spawn** (not hand-rolled adb)
- [ ] Stop / Quit = session teardown: device client + relay **if we started it**
- [ ] Hidden poll: visible ~2s; hidden+idle ~10s (or pause); hidden+active ~5s
- [ ] Close-to-tray does not abandon Quit path
- [ ] Logs: pipes → events → UI (+ optional file)
- [ ] No multi-device autorun required
- [ ] macOS labeled best-effort

---

## 12. Cross-references

- System context & packaging: [DESKTOP_ARCHITECTURE.md](DESKTOP_ARCHITECTURE.md)  
- UI Run/Stop controls + VPN copy: [FRONTEND_ARCHITECTURE.md](FRONTEND_ARCHITECTURE.md)  
- State transitions, ownership, port session: [STATE_MODEL.md](STATE_MODEL.md)
