# Architecture

## How gnirehtet works

gnirehtet has two parts:

- The **relay** runs on the computer and listens on a local TCP port (31416 by
  default). It opens the real network connections on the phone's behalf.
- The **client** is an Android app. It creates a local VPN that captures the
  phone's IPv4 traffic and sends it to the relay.

The two are linked by an adb reverse tunnel: `adb reverse localabstract:gnirehtet
tcp:31416` makes a connection on the phone reach the relay on the computer
through the USB cable.

Connecting a phone takes four steps:

1. Start the relay.
2. Install the client if it is missing.
3. Create the reverse tunnel.
4. Start the client with an `am start` intent.

The phone then asks the user to accept the VPN. Once they do, the client
connects to the relay, and the relay logs `Client #N connected`.

This project does not change any of that. It ships the upstream relay and APK
(v2.5.1) unchanged and automates the steps.

## Components

```
┌──────────────────────── desktop app (Tauri) ────────────────────────┐
│                                                                     │
│  UI: SvelteKit + TypeScript (apps/desktop/src)                      │
│    app.svelte.ts      state, device polling, phase transitions      │
│    components/        views                                         │
│         │  invoke(command)             ▲  events: log,              │
│         ▼                              │  relay-status, relay-client│
│  Backend: Rust (apps/desktop/src-tauri)                             │
│    commands.rs   thin async command layer                           │
│    session.rs    one reverse-tethering session                      │
│    paths.rs      locate adb, relay and APK                          │
│         │                        │                                  │
│         ▼                        ▼                                  │
│  gnirehtet-adb             gnirehtet-controller                     │
│  (runs adb commands)       (supervises the relay process)           │
└─────────┼────────────────────────┼──────────────────────────────────┘
          ▼                        ▼
         adb  ──── USB ────►  phone: gnirehtet client (APK)
                                   │  VPN traffic via adb reverse tunnel
                                   ▼
                          gnirehtet-relay process (sidecar) ──► internet
```

### Rust crates

| Crate | Path | Role |
|-------|------|------|
| `gnirehtet-adb` | `crates/gnirehtet-adb` | adb client extracted from upstream: device listing, install, reverse tunnel, start and stop of the Android client. Every command runs with a timeout, and user-supplied DNS and route values are validated before they reach `adb shell`. Errors carry stable codes such as `DEVICE_UNAUTHORIZED` and `ADB_TIMEOUT`. |
| `gnirehtet-controller` | `crates/gnirehtet-controller` | `RelaySupervisor`: starts the relay as a child process, waits until it listens, reports early exits with the relay's own error output, and only ever stops the process it started. It refuses to start on a busy port instead of adopting a foreign relay. |
| `gnirehtet-relay` | `crates/gnirehtet-relay` | Upstream relay library (`relaylib`) at v2.5.1, vendored unchanged. |
| `gnirehtet-cli` | `crates/gnirehtet-cli` | The upstream `gnirehtet` command-line tool built from the vendored sources. Used to build the relay on platforms without an upstream binary (macOS, ARM). |
| `gnirehtet-desktop` | `apps/desktop/src-tauri` | The Tauri application. |

The desktop app does **not** link the relay library. It runs the relay binary
as a separate process (a Tauri *sidecar*, installed as `gnirehtet-relay`), for
three reasons:

- A crash in the networking code cannot take down the UI.
- The shipped relay is byte-for-byte the upstream release.
- The same code path works with any relay binary set with `GNIREHTET_BIN`.

### Backend: the session

`session.rs` owns everything that must be cleaned up: the relay process and the
serial of the device whose client was started.

- **Serialized actions.** Connect, disconnect, install and repair take an
  operation lock, so two clicks cannot interleave. Device polling does not take
  this lock, so the device list keeps updating during a slow install.
- **Connect order.** Validate the options, start the relay (or reuse it on the
  same port), then install, tunnel and start the client. If starting the client
  fails, the relay started for this attempt is stopped again.
- **Relay monitoring.** Threads read the relay's stdout and stderr, forward
  each line to the log view, and count `Client #N connected` / `disconnected`
  lines to produce `relay-client` events. A watcher reports an unexpected exit
  as a `relay-status` event with state `crashed`. A generation counter keeps
  threads of an old relay from reporting on a new one.
- **Disconnect** stops the client on the device, removes the tunnel, and stops
  the relay. The computer side is always stopped, even when the phone can no
  longer be reached.
- **Exit.** On `RunEvent::Exit`, the relay is stopped immediately and the
  phone gets a bounded amount of time to stop its client. The app never touches
  a device it did not start a client on.

Every command returns either a value or a `CommandError { code, message }`. The
UI maps codes to explanations in `src/lib/errors.ts`.

### Frontend: phases

`app.svelte.ts` holds the application state (a single Svelte 5 runes object).
It polls `list_devices` every 2 seconds while the window is visible, and drives
the connection through these phases:

| Phase | Meaning |
|-------|---------|
| `idle` | Nothing running. |
| `starting` | `connect` in progress. |
| `waiting` | Relay and client started; waiting for the user to accept the VPN. |
| `connected` | The relay reported a client connection. |
| `interrupted` | The session's device disappeared from adb (cable unplugged). When it returns, the tunnel is restored automatically. |
| `failed` | The relay exited unexpectedly. |
| `stopping` | `disconnect` in progress. |

The pure transition helpers live in `session-logic.ts` and are unit-tested.
Settings (port, DNS, routes, theme) are validated and stored in `localStorage`
(`settings.svelte.ts`).

### Tauri commands

| Command | Description |
|---------|-------------|
| `check_adb` | Run `adb version`; errors with `ADB_MISSING` or `ADB_PATH_INVALID`. |
| `list_devices` | `adb devices -l`, parsed. |
| `get_session` | Snapshot of the running session (used after a UI reload). |
| `connect(serial, options)` | Start a session. |
| `disconnect` | Stop the session. |
| `install_client(serial)` | Reinstall the client APK. |
| `repair_tunnel(serial)` | Recreate the reverse tunnel. |
| `get_environment` | Resolved tool paths and app version, for diagnostics. |

Events: `log` (a log entry), `relay-status` (relay state changes) and
`relay-client` (client connected or disconnected, with the current count).

### Locating the tools

| Tool | Resolution order |
|------|------------------|
| adb | `ADB` → `adb` on `PATH` → `platform-tools` in the Android SDK (`ANDROID_HOME`, `ANDROID_SDK_ROOT`, platform default) |
| relay | `GNIREHTET_BIN` → bundled `gnirehtet-relay` next to the app executable → `gnirehtet` on `PATH` |
| APK | `GNIREHTET_APK` → bundled resource → `resources/gnirehtet.apk` (debug builds only) |

adb is not bundled. It is widely packaged, users often have it already, and a
second adb server with a different version would fight the user's own adb over
the USB device.

## Packaging

`scripts/fetch-upstream.mjs` (`npm run setup`) prepares the two bundled files:

- `resources/gnirehtet.apk`
- `apps/desktop/src-tauri/binaries/gnirehtet-relay-<target-triple>[.exe]`

It downloads the pinned upstream v2.5.1 release archive and verifies its SHA-256
checksum. For targets without an upstream binary, it builds the relay from the
vendored sources. Neither file is committed.

`tauri build` then produces:

- Linux: `.deb`, `.rpm` and `.AppImage`
- Windows: NSIS `-setup.exe`
- macOS: `.app` and `.dmg`

The release workflow (`.github/workflows/release.yml`) builds all of them on a
push to `prod` with a new version, renames them after the system they are for
(for example `gnirehtet-gui-1.0.0-macos-intel.dmg`) and publishes everything but
the bare `.app`.

Installed layout on Linux:

```
/usr/bin/gnirehtet-gui
/usr/bin/gnirehtet-relay
/usr/lib/gnirehtet-gui/gnirehtet.apk
/usr/share/applications/gnirehtet-gui.desktop
```

## Security

- The content security policy allows only the app's own scripts and styles and
  the Tauri IPC channel; the app loads nothing remote.
- Capabilities are reduced to `core:default` plus window destruction (used by
  the quit confirmation). No shell, filesystem or opener plugins are included.
- Values from the UI never reach a shell unvalidated. The device serial, DNS
  servers and routes are checked in the backend: IPv4 addresses and CIDR
  ranges only.
