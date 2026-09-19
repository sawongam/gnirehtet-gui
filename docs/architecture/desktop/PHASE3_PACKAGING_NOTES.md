# Phase 3 — Packaging smoke notes

**Date:** 2026-09-19 NPT  
**Branch tip (packaging follow-up):** see `dev` HEAD  
**Lab host:** Linux x86_64 (box)

## Goal

Keep release packaging reachable on this box: deb/rpm smoke, documented sidecar enable path, AppImage deferred until linuxdeploy is fixed. **No MVP Done claim** — USB E2E still required (Sangam lab).

## Config snapshot

| Item | Value |
|------|--------|
| `bundle.targets` | **`["deb", "rpm"]`** — AppImage excluded (linuxdeploy fail on this host) |
| `bundle.externalBin` | **`[]` (empty)** — CI/`cargo check` safe without committed binary |
| Sidecar drop-in on box | `apps/desktop/src-tauri/binaries/gnirehtet-x86_64-unknown-linux-gnu` (present, **not** embedded while externalBin empty) |
| APK | `resources/gnirehtet.apk` via `bundle.resources` (pin **v2.5.1**) |
| Docs | `apps/desktop/docs/SIDECAR.md` — enable recipe + pins |

Runtime still resolves `GNIREHTET_BIN` → packaged sidecar / `binaries/gnirehtet-<triple>` → `PATH` `gnirehtet`. Compatible with controller defaults.

### Upstream binary pins (packaging)

| Host | Zip pin | Executable name in zip |
|------|---------|------------------------|
| Linux / Windows | **v2.5.1** | `gnirehtet` / `gnirehtet.exe` (**not** the zip folder name) |
| macOS | **v2.2.1** (upstream prebuilt exception) | `gnirehtet` — do not silently mix with v2.5.1 Linux/Win |

## Enable `externalBin` for real installers (reversible)

Default stays empty so cargo check does not require the sidecar.

1. Drop triple-suffixed binary (see SIDECAR.md).  
2. In `apps/desktop/src-tauri/tauri.conf.json`, set `"externalBin": ["binaries/gnirehtet"]`.  
3. Run `npm run tauri build` — deb/rpm should embed the sidecar.  
4. Revert to `"externalBin": []` for CI/dev-safe default.

## Smoke results (2026-09-19 NPT)

| Step | Result |
|------|--------|
| `npm run check` | **PASS** (0 errors) |
| `cargo check -p gnirehtet-desktop` | **PASS** |
| `vite build` / SvelteKit static (`beforeBuildCommand`) | **PASS** → `apps/desktop/build` |
| Rust release link (`gnirehtet-desktop`) | **PASS** → `target/release/gnirehtet-desktop` |
| Bundle `.deb` | **PASS** → `target/release/bundle/deb/gnirehtet-gui_0.1.0_amd64.deb` |
| Bundle `.rpm` | **PASS** → `target/release/bundle/rpm/gnirehtet-gui-0.1.0-1.x86_64.rpm` |
| Bundle AppImage | **DEFERRED** — previously **FAIL** (`failed to run linuxdeploy` under `targets: "all"`). Config now omits appimage. |

### Honest blockers / limits

1. **AppImage / linuxdeploy** — deferred. Deb/rpm are the supported Linux installer targets until linuxdeploy is fixed/available. Do not set `targets: "all"` on this host.
2. **`externalBin: []`** — release **binary** builds, but the installer **does not embed** the gnirehtet sidecar until the enable recipe above is applied with a triple-suffixed file present.
3. **No gnirehtet-relay dep** — confirmed; Desktop links controller/adb only, spawns external binary.
4. **USB E2E / Sharing** — not claimed here; see `PHASE3_ACCEPTANCE.md`. Never invent Sharing from handshake probe alone.

## Phase 3 product wiring (same land)

- **Quit-clean:** `prepare_quit` + `RunEvent::Exit` → epoch bump → best-effort device `stop` → `clear_owned_relay` (not `RELAY_CRASHED`). UI `onCloseRequested` clears session layers.
- **ERROR_UX recovery:** prominent banners/CTAs for `APK_MISSING`, unauthorized, `TUNNEL_LOST` (Repair), plus `INSTALL_FAILED` / relay crash / port-in-use.
- **Handshake probe:** owned-relay LogLine `Client #(\d+) (connected|disconnected)` → Tunnel chip “Relay accepted client”; Device VPN stays **Pending**; **never Sharing** from probe alone (`HANDSHAKE_LIVENESS_MVP.md`).

## Commands

```bash
cd apps/desktop
npm run check
cargo check -p gnirehtet-desktop
npm run tauri build   # deb + rpm only; AppImage not in targets
```
