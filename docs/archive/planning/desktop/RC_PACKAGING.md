# Release candidate packaging (Linux + Windows)

**Label:** **release candidate packaging** — **not** MVP Done.  
**Date:** 2026-09-20 ~01:42 NPT  
**Branch tip (pre-commit):** `dev` @ `879b07d` + this packaging land  
**Lab host:** Linux x86_64 (box) — Linux installers smoked here; Windows path documented (no Windows smoke / no USB E2E on this box).

## Explicit non-claims

- No **MVP Done**.
- No invented **Sharing** state (handshake probe ≠ Sharing).
- No AppImage (linuxdeploy deferred).
- **macOS packaging deferred** this slice (not first-class here).
- No signed Windows MSI/NSIS claimed; no USB E2E claimed.

## Pins (do not mix)

| Host | Upstream zip | Binary name inside zip | APK |
|------|--------------|------------------------|-----|
| **Linux x86_64** | Genymobile **v2.5.1** `gnirehtet-rust-linux64-v2.5.1.zip` | `gnirehtet` | **v2.5.1** `gnirehtet.apk` |
| **Windows x86_64** | Genymobile **v2.5.1** `gnirehtet-rust-win64-v2.5.1.zip` | `gnirehtet.exe` | same **v2.5.1** APK |
| macOS | deferred this slice | — | — |

Checksums verified on box against upstream v2.5.1 zip (sha256):

| Artifact | sha256 |
|----------|--------|
| `gnirehtet` (linux64) | `8acee2e88dc5653db08a61e1a14e0ab86247524e366a482a883940220a2f4ea9` |
| `gnirehtet.apk` | `c1ac2b869a48e3c836046aac5a168f3ade510288f3304a87ddb671315c564b9a` |

## Config (release packaging)

`apps/desktop/src-tauri/tauri.conf.json`:

| Key | RC value |
|-----|----------|
| `bundle.targets` | `["deb", "rpm", "nsis", "msi"]` — **no** `appimage`; macOS targets not enabled this slice |
| `bundle.externalBin` | `["binaries/gnirehtet"]` — **enabled** for RC (embeds sidecar) |
| `bundle.resources` | `../../../resources/*` (includes `gnirehtet.apk` when present) |
| `bundle.windows` | digest `sha256`; signing thumbprint unset |

### Sidecar drop-in (gitignored)

```text
apps/desktop/src-tauri/binaries/gnirehtet-x86_64-unknown-linux-gnu   # Linux
apps/desktop/src-tauri/binaries/gnirehtet-x86_64-pc-windows-msvc.exe # Windows
resources/gnirehtet.apk
```

### CI / `cargo check` without sidecar

Release config keeps `externalBin` **enabled** when the triple binary is present (RC path).

If CI/dev has **no** binary and must stay green:

1. Temporarily set `"externalBin": []` in `tauri.conf.json`.
2. `cargo check -p gnirehtet-desktop` (and frontend `npm run check`) without embedding.
3. Re-enable `"externalBin": ["binaries/gnirehtet"]` before any installer build that should ship the sidecar.

Runtime still resolves `GNIREHTET_BIN` → drop-in / PATH even when `externalBin` is empty (dev). See `apps/desktop/docs/SIDECAR.md`.

## Linux RC artifacts (smoked on this box)

Built with `externalBin` enabled + v2.5.1 sidecar + APK present:

```bash
cd apps/desktop
npm run tauri build
```

| Artifact | Path | Size (bytes) |
|----------|------|--------------|
| **deb** | `target/release/bundle/deb/gnirehtet-gui_0.1.0_amd64.deb` | **3665132** |
| **rpm** | `target/release/bundle/rpm/gnirehtet-gui-0.1.0-1.x86_64.rpm` | **3667131** |

### Embedded contents (deb listing)

| Path in package | Role | Size |
|-----------------|------|------|
| `usr/bin/gnirehtet-desktop` | Tauri app | 8254584 |
| `usr/bin/gnirehtet` | Upstream sidecar **v2.5.1** | 678368 |
| `usr/lib/gnirehtet-gui/_up_/_up_/_up_/resources/gnirehtet.apk` | Client APK **v2.5.1** | 23904 |

`usr/bin/gnirehtet` sha256 matches upstream linux64 v2.5.1.

## Install + first-run (Linux `.deb`)

### Install

```bash
sudo apt install ./target/release/bundle/deb/gnirehtet-gui_0.1.0_amd64.deb
# or: sudo dpkg -i ./gnirehtet-gui_0.1.0_amd64.deb && sudo apt-get install -f
```

RPM (Fedora/RHEL-class):

```bash
sudo rpm -Uvh ./target/release/bundle/rpm/gnirehtet-gui-0.1.0-1.x86_64.rpm
# or: sudo dnf install ./gnirehtet-gui-0.1.0-1.x86_64.rpm
```

### First-run requirements

1. **`adb` on `PATH`** (Android platform-tools). The GUI/controller shells out to `adb`; without it, device list / install / tunnel fail clearly.
2. USB debugging authorized on the Android device (physical lab — not claimed here).
3. **Do not** set `GNIREHTET_*` unless overriding defaults:
   - Packaged install already embeds sidecar + APK.
   - Optional overrides only when needed: `GNIREHTET_BIN`, `GNIREHTET_APK`.
4. Launch `gnirehtet-desktop` from the menu or `/usr/bin/gnirehtet-desktop`.

This document does **not** claim USB E2E, VPN on device, or Sharing.

## Windows packaging (first-class — documented path)

**Not smoked on this Linux box** (no Windows host / no cross NSIS-MSI smoke). Exact path for a Windows x86_64 builder:

### 1. Drop v2.5.1 sidecar + APK

```powershell
# From repo root on Windows
curl.exe -fsSL -o $env:TEMP\gnirehtet-rust-win64-v2.5.1.zip `
  https://github.com/Genymobile/gnirehtet/releases/download/v2.5.1/gnirehtet-rust-win64-v2.5.1.zip
# Extract gnirehtet.exe (name inside zip — not the folder name) to:
#   apps\desktop\src-tauri\binaries\gnirehtet-x86_64-pc-windows-msvc.exe
# Extract gnirehtet.apk to:
#   resources\gnirehtet.apk
```

Pin: **v2.5.1**, binary **`gnirehtet.exe`**.

### 2. Tooling

- Rust stable (MSVC), Node.js, WebView2 runtime
- Visual Studio Build Tools (C++), WiX (for MSI) as required by Tauri 2 bundler
- `adb` on PATH for first-run (same as Linux)

### 3. Build

```powershell
cd apps\desktop
npm install
npm run tauri build
```

With `bundle.targets` including `nsis` / `msi` and `externalBin: ["binaries/gnirehtet"]`, expect:

- `target/release/bundle/nsis/…setup.exe` (or similar NSIS artifact)
- `target/release/bundle/msi/….msi`

Record actual paths/sizes on the Windows builder when smoke succeeds. **Do not** claim a Windows build or USB E2E from this Linux RC slice.

### Windows first-run

1. Install NSIS or MSI artifact from the Windows builder.
2. Ensure **`adb` on PATH**.
3. Set `GNIREHTET_BIN` / `GNIREHTET_APK` **only if** overriding the embedded sidecar/APK.
4. No Sharing / no MVP Done claim until USB E2E on lab hardware.

## Commands (Linux RC)

```bash
# Sidecar + APK already present on this box (gitignored)
cd apps/desktop
npm run check
cargo check -p gnirehtet-desktop   # works with sidecar present + externalBin enabled
npm run tauri build                # deb + rpm (+ nsis/msi skipped on Linux)
```

## Related

- `apps/desktop/docs/SIDECAR.md` — enable/disable recipe + pins
- `PHASE3_ACCEPTANCE.md` — RC ≠ MVP Done; USB E2E hold
- `PHASE3_PACKAGING_NOTES.md` — earlier smoke; superseded for embed status by this RC note
