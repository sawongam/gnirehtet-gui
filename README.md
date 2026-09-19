# gnirehtet-gui

Open-source **desktop GUI** successor / front-end for [Genymobile/gnirehtet](https://github.com/Genymobile/gnirehtet) — reverse tethering for Android over `adb`, without root.

This repository is the **product home**. Upstream gnirehtet remains a **pinned dependency** (vendored or submodule), not this repo’s conceptual `origin` for networking logic.

## Status

**Release candidate packaging** on branch `dev` (local-first; GitHub sync deferred). Baseline branch: `master`. Authored as `sawongam`.

- **Not** MVP Done — USB E2E on a real device still required before any Sharing / Done claim.
- Linux: `.deb` / `.rpm` with embedded Genymobile **v2.5.1** sidecar + APK (AppImage deferred).
- Windows: NSIS/MSI recipe documented (build on a Windows host).
- macOS packaging deferred this slice.

### First-run (Linux .deb)

See **[`docs/architecture/desktop/RC_PACKAGING.md`](docs/architecture/desktop/RC_PACKAGING.md)** for artifacts, pins, and install steps. Short path:

1. Install the RC `.deb` from the release/lab bundle path documented there.
2. Ensure **`adb` is on `PATH`** (platform-tools). Do not rely on a bundled adb for MVP.
3. Launch the app, authorize the device, then Install → Run. Sharing is only claimed when Relay + Tunnel + Device VPN are healthy.

Override paths only if needed: `ADB`, `GNIREHTET_BIN`, `GNIREHTET_APK`.

### Dev

```bash
cd apps/desktop && npm install && npm run tauri dev
# or: cargo check -p gnirehtet-desktop
```

See `apps/desktop/README.md`, `apps/desktop/docs/SIDECAR.md`, `resources/README.md`.

## Docs

| Path | Owner |
|------|--------|
| [`docs/architecture/`](docs/architecture/) | Lead architecture, MVP, roadmap, migration |
| [`docs/architecture/desktop/`](docs/architecture/desktop/) | Desktop shell, RC packaging, Phase 0–3 acceptance |
| [`docs/architecture/rust/`](docs/architecture/rust/) | Rust crate structure, concurrency, refactor plan |

## Upstream pin

Planning + Linux/Windows RC sidecars are pinned to Genymobile/gnirehtet **v2.5.1** / **`1eb2e58`** unless an ADR updates the pin. Do not silently mix older macOS prebuilt pins into Linux/Win packages.

## License

Apache License 2.0. Reused gnirehtet components remain under their Apache-2.0 terms with Genymobile copyright retained in those files / `NOTICE`.
