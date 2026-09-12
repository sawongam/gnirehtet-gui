# MVP Specification

**Status:** Draft v0.1  
**Goal:** A usable cross-platform desktop GUI that can reverse-tether **one** Android device using the existing gnirehtet technical foundation — without rewriting the relay or VPN client.

---

## 1. Success definition

A user on Linux or Windows (macOS best-effort in MVP) can:

1. Launch the desktop app.
2. See connected adb devices (or a clear “adb missing / unauthorized” state).
3. Install the gnirehtet APK on a selected device (if needed).
4. Start reverse tethering with one primary action.
5. See live status (relay running, device connected/tethering, recent logs).
6. Stop cleanly (relay + device client), leaving no orphan relay process under normal exit.

**Out of MVP success:** pretty charts, IPv6, Play Store client, custom VPN protocols, wireless-adb-only workflows as the happy path.

---

## 2. Personas

- **Developer / power user** who already has `adb`, USB debugging, and occasional reverse-tether needs.
- **Contributor** evaluating whether the project is a trustworthy reuse of gnirehtet vs a reckless rewrite.

---

## 3. In scope

### 3.1 Functional

| ID | Requirement | Notes / upstream mapping |
|----|-------------|--------------------------|
| F1 | Detect `adb` on `PATH` or user-configured path | Mirrors `ADB` env behavior |
| F2 | List devices with serial, state (`device`/`unauthorized`/`offline`) | `adb devices` |
| F3 | Select a single target device | MVP = one active tether session |
| F4 | Install / reinstall APK from bundled or configured path | `install` / `reinstall`; `GNIREHTET_APK` |
| F5 | Start relay on configurable port (default `31416`) | `relay` / `run` |
| F6 | Establish `adb reverse localabstract:gnirehtet tcp:<port>` | `tunnel` |
| F7 | Start client via START intent | `start` |
| F8 | Stop client via STOP intent | `stop` |
| F9 | One-click **Run** = install-if-needed + tunnel + start + relay, with teardown on Stop/Quit | Align with `run` semantics |
| F10 | Show relay/orchestrator logs in UI | stdout/stderr capture |
| F11 | Reset tunnel action | `tunnel` |
| F12 | Optional DNS servers and routes fields | `-d`, `-r` passthrough |
| F13 | Persist settings: adb path, apk path, default port, last serial | App config |

### 3.2 Non-functional

| ID | Requirement |
|----|-------------|
| N1 | No root required on device or host (same as upstream) |
| N2 | Apache-2.0 compatible packaging; preserve Genymobile copyright/attribution for reused code |
| N3 | Graceful Stop/Quit does not leave relay listening (best effort; document force-kill fallback) |
| N4 | Clear errors when USB unauthorized, APK version mismatch, port in use, or adb missing |
| N5 | Works offline aside from device network path under test |

---

## 4. Out of scope (MVP)

- Multi-device simultaneous tether UI (`autorun` / `autostart` deferred)
- IPv6
- Rewriting relay or Android client
- In-process `relaylib` embed
- Auto-download of Android platform-tools (detect + link to docs only)
- Wireless debugging as primary UX (may work if adb sees the device; not designed/tested as MVP pillar)
- Tray-only agent mode, CLI feature parity beyond what GUI exposes
- Telemetry

---

## 5. UX outline

**Primary screen**

- Device list + refresh
- Status chip: Idle | Starting | Tethering | Error
- Primary buttons: Run / Stop
- Secondary: Install APK, Reset tunnel, Open logs
- Advanced drawer: port, DNS, routes, adb path, apk path

**First-run**

- Checklist: adb found? device authorized? VPN permission will appear on device.

**Device VPN permission**

- Explicit copy: the phone will prompt once; the desktop app cannot click it for you.

---

## 6. Technical acceptance tests

1. **Happy path (USB):** unauthorized → authorize → install → run → device browser reaches host network resource → stop.
2. **Port conflict:** relay fails with actionable message if `31416` busy.
3. **Relay killed externally:** UI transitions to Error within a bounded time; Stop is safe to click.
4. **Quit while running:** no listening process on relay port after quit (platform-specific verification).
5. **APK version mismatch:** surface upstream-style version check failure (`REQUIRED_APK_VERSION_CODE`).
6. **Multi-device present:** require serial selection; do not pick silently.

---

## 7. Packaging MVP

Ship:

- Desktop app binary (Tauri provisional)
- `gnirehtet` relay/CLI sidecar (or equivalent rust binary)
- `gnirehtet.apk` resource
- License / NOTICE for Genymobile + project

Do not require a system-wide gnirehtet install.

---

## 8. Open MVP questions (need evidence, not guesses)

- Minimum Android API to claim support after retest (upstream states API 21).
- Whether macOS is a hard MVP platform or “best effort” given historical release lag.
- Final product name / trademark posture vs “gnirehtet”.
