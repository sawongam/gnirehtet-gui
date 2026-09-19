# Phase 3 — Acceptance summary (not MVP Done)

**Date:** 2026-09-19 NPT  
**Scope:** Desktop packaging + product wiring on `dev`.  
**Status:** Code/packaging path **pass** on lab box; **USB E2E hold** before any MVP Done claim.

## Code / packaging pass (box)

| Item | Status |
|------|--------|
| Install / tunnel / start / stop / run wired into shell | Pass (prior commits) |
| Quit-clean owned relay + best-effort device stop | Pass |
| ERROR_UX recovery surfaces | Pass |
| Handshake liveness probe (LogLine client connect) | Pass — Tunnel chip only; **never Sharing** |
| `cargo check -p gnirehtet-desktop` without sidecar in git | Pass (`externalBin: []`) |
| `npm run check` | Pass |
| Linux installers | **deb + rpm** preferred; AppImage **deferred** (linuxdeploy) |
| Sidecar embed path | Documented reversible enable (`SIDECAR.md`); default off |

## Packaging pins (do not mix)

- Linux / Windows rust binary + APK: Genymobile **v2.5.1** — zip contains `gnirehtet` / `gnirehtet.exe` (not the folder name).
- macOS prebuilt rust zip: still **v2.2.1** — packaging exception; document explicitly.
- Controller: `GNIREHTET_BIN` → drop-in / sidecar → `PATH` `gnirehtet`.

## Hold — USB E2E (Sangam lab)

MVP Done requires physical USB device end-to-end: authorize → install APK → tunnel → start → VPN / Sharing truth on device. Handshake probe alone must **not** be treated as Sharing or MVP Done.

## Explicit non-claims

- No **MVP Done**.
- No invented **Sharing** state.
- No AppImage as a supported target until linuxdeploy is fixed.
