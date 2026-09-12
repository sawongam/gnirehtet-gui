# gnirehtet-gui desktop (Phase 0)

Tauri 2 + SvelteKit (static) + TypeScript shell with a host-orchestrator stub.

## Prerequisites

- Node 20+, Rust stable, platform Tauri deps (Linux: webkit2gtk-4.1, etc.)
- User-provided `adb` on PATH (or `ADB` env)
- Optional: stock upstream `gnirehtet` binary — see [docs/SIDECAR.md](docs/SIDECAR.md)
- Optional: `resources/gnirehtet.apk` — see [../../resources/README.md](../../resources/README.md)

## Run

```bash
cd apps/desktop
npm install
npm run tauri dev
```

## Build check (no GUI)

```bash
# from repo root
cargo check -p gnirehtet-desktop

cd apps/desktop && npm run check
```

`tauri build` requires the `externalBin` sidecar file under `src-tauri/binaries/` (target-triple suffix). Absent binary is OK for `cargo check` only if you temporarily clear `bundle.externalBin` or drop a placeholder — see SIDECAR.md.

## Commands (snake_case)

| Command | Purpose |
|---------|---------|
| `ensure_adb` | Validate adb |
| `list_devices` | `adb devices -l` |
| `start_relay` / `stop_relay` | Session-owned external `gnirehtet relay` |
| `get_relay_state` | Poll ownership / running |

Events: `LogLine`, `RelayState` (camelCase payloads).
