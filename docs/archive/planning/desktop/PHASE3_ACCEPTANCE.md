# Phase 3 — Acceptance summary (not MVP Done)

**Date:** 2026-09-20 NPT  
**Scope:** Desktop packaging + product wiring on `dev`.  
**Status:** **Release candidate packaging** pass on Linux lab box (deb/rpm + embedded sidecar); Windows path documented. **USB E2E hold** before any MVP Done claim.

## Code / packaging pass (box)

| Item | Status |
|------|--------|
| Install / tunnel / start / stop / run wired into shell | Pass (prior commits) |
| Quit-clean owned relay + best-effort device stop | Pass |
| ERROR_UX recovery surfaces | Pass |
| Handshake liveness probe (LogLine client connect) | Pass — Tunnel chip only; **never Sharing** |
| `cargo check -p gnirehtet-desktop` with sidecar present | Pass (`externalBin` enabled for RC) |
| No-sidecar check path | Documented: temporarily empty `externalBin` — see SIDECAR.md / RC_PACKAGING.md |
| `npm run check` | Pass (prior) |
| Linux installers | **deb + rpm** with **embedded** gnirehtet v2.5.1 + APK — see `RC_PACKAGING.md` |
| Windows installers | First-class targets `nsis`/`msi`; **build path documented**; not smoked on Linux box |
| AppImage | Deferred (linuxdeploy) |
| macOS | Deferred this slice |

## Packaging pins (do not mix)

- Linux / Windows rust binary + APK: Genymobile **v2.5.1** — zip contains `gnirehtet` / `gnirehtet.exe` (not the folder name).
- macOS: deferred this slice — do not invent a pin here.
- Controller: `GNIREHTET_BIN` → drop-in / sidecar → `PATH` `gnirehtet`.

## Hold — USB E2E (Sangam lab)

MVP Done requires physical USB device end-to-end: authorize → install APK → tunnel → start → VPN / Sharing truth on device. Handshake probe alone must **not** be treated as Sharing or MVP Done.

## Explicit non-claims

- No **MVP Done** — this is **release candidate packaging** only.
- No invented **Sharing** state.
- No AppImage as a supported target until linuxdeploy is fixed.
- No Windows USB E2E / no claimed Windows artifact from the Linux box.
