# Phase 3 — Packaging smoke notes

**Date:** 2026-09-19 NPT  
**Branch tip (pre-polish):** `d68435a`  
**Lab host:** Linux x86_64 (box)

## Goal

Try `npm run tauri build` (or document honest blockers) so Lead knows whether a release bundle is reachable on this box.

## Config snapshot

| Item | Value |
|------|--------|
| `bundle.externalBin` | **`[]` (empty)** — intentional for CI/`cargo check` without committed binary |
| Sidecar drop-in on box | `apps/desktop/src-tauri/binaries/gnirehtet-x86_64-unknown-linux-gnu` (present, **not** embedded while externalBin empty) |
| APK | `resources/gnirehtet.apk` via `bundle.resources` |
| Docs | `apps/desktop/docs/SIDECAR.md` — set `"externalBin": ["binaries/gnirehtet"]` when shipping |

Runtime still resolves `GNIREHTET_BIN` → packaged sidecar → `binaries/gnirehtet-<triple>` → `PATH`.

## Smoke results (2026-09-19 NPT)

| Step | Result |
|------|--------|
| `npm run check` | **PASS** (0 errors) |
| `cargo check -p gnirehtet-desktop` | **PASS** |
| `vite build` / SvelteKit static (`beforeBuildCommand`) | **PASS** → `apps/desktop/build` |
| Rust release link (`gnirehtet-desktop`) | **PASS** → `target/release/gnirehtet-desktop` |
| Bundle `.deb` | **PASS** → `target/release/bundle/deb/gnirehtet-gui_0.1.0_amd64.deb` |
| Bundle `.rpm` | **PASS** → `target/release/bundle/rpm/gnirehtet-gui-0.1.0-1.x86_64.rpm` |
| Bundle AppImage | **FAIL** — `failed to run linuxdeploy` (Tauri still exits non-zero when `targets: "all"`) |

### Honest blockers / limits

1. **AppImage / linuxdeploy** — WebView compile/link OK on this host; AppImage packaging failed at `linuxdeploy`. Deb/rpm succeeded. For CI, prefer `bundle.targets: ["deb"]` (or exclude appimage) until linuxdeploy is fixed/available.
2. **`externalBin: []`** — release **binary** builds, but the installer **does not embed** the gnirehtet sidecar. Shipping a usable tether bundle still requires enabling externalBin + triple-suffixed binary (SIDECAR.md).
3. **No gnirehtet-relay dep** — confirmed; Desktop links controller/adb only, spawns external binary.

## Phase 3 product wiring (same land)

- **Quit-clean:** `prepare_quit` + `RunEvent::Exit` → epoch bump → best-effort device `stop` → `clear_owned_relay` (not `RELAY_CRASHED`). UI `onCloseRequested` clears session layers.
- **ERROR_UX recovery:** prominent banners/CTAs for `APK_MISSING`, unauthorized, `TUNNEL_LOST` (Repair), plus `INSTALL_FAILED` / relay crash / port-in-use.
- **Handshake probe:** owned-relay LogLine `Client #(\d+) (connected|disconnected)` → Tunnel chip “Relay accepted client”; Device VPN stays **Pending**; **never Sharing** from probe alone (`HANDSHAKE_LIVENESS_MVP.md`).

## Commands

```bash
cd apps/desktop
npm run check
cargo check -p gnirehtet-desktop
npm run tauri build   # expect deb/rpm OK; AppImage may fail on linuxdeploy
```
