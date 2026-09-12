# Controller API draft (Desktop ↔ Rust)

**Status:** informal freeze candidate from Desktop Engineer + Rust Architect alignment  
**Upstream pin:** Genymobile/gnirehtet `@1eb2e58` / v2.5.1  
**MVP path:** Desktop may spawn stock `gnirehtet` CLI first; this API is the M2 swap target (`gnirehtet-controller`).

## Commands (Rust snake_case)

| Command | Notes / CLI mirror |
|---------|-------------------|
| `list_devices` | adb discovery |
| `ensure_adb` | adb health / availability |
| `install(serial)` | `gnirehtet install` |
| `start(serial, { dns?, routes?, port? })` | `gnirehtet start` |
| `stop(serial)` | `gnirehtet stop` |
| `reset_tunnel(serial, port?)` | `gnirehtet tunnel` (`adb reverse localabstract:gnirehtet`) |
| `start_relay(port?)` | spawn session-owned `gnirehtet relay` child (default port **31416**) |
| `stop_relay()` | kill **only** a relay this session started |
| `run(serial, opts?)` | one-click ≈ upstream `run` |

## Events

- `DeviceChanged` — device set / monitor updates
- `RelayState` — relay spawned / exited / status
- `LogLine` — child stdout/stderr + log bridge
- `Error` — Adb / process / relay-process failures

## IPC naming

- Rust: `snake_case`
- TypeScript boundary: `camelCase` via serde rename

## Ownership

Session-scoped: Stop/Quit stops the device client and only kills a relay this session started.

## Port policy (frozen with Desktop)

- Default relay port: **31416** (upstream `cli_args` default).
- `start_relay()` with no port → use **session port** if already chosen this session; else `31416`.
- `run` / `start` with explicit `port` sets the session port for that tether (tunnel + client + relay must agree).
- Changing port mid-session requires stop + re-run (do not hot-swap an owned relay’s listen port).
