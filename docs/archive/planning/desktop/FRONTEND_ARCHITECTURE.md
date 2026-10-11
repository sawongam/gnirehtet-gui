# Frontend Architecture — Gnirehtet Desktop GUI

**Status:** Draft v0.2  
**Owner:** Desktop Application Engineer  
**Date:** 2026-09-12  
**Binding inputs:**
- [`research/lead-architect-baseline.md`](research/lead-architect-baseline.md)
- [`research/gnirehtet-core-boundaries.md`](research/gnirehtet-core-boundaries.md)
- [`research/architect-decisions-2026-09-12.md`](research/architect-decisions-2026-09-12.md)

**Related docs:** [DESKTOP_ARCHITECTURE.md](DESKTOP_ARCHITECTURE.md) · [STATE_MODEL.md](STATE_MODEL.md) · [DESKTOP_LIFECYCLE.md](DESKTOP_LIFECYCLE.md)

---

## 1. Why Svelte + TypeScript in Tauri 2

| Criterion | Fit for this app |
|-----------|------------------|
| **App size** | Few views (devices, connection, settings, logs); not a large SPA — Svelte’s compile-time approach keeps the bundle small |
| **Reactivity** | Device list + relay status + log stream update frequently; Svelte stores map cleanly to orchestrator events |
| **TypeScript** | Shared conceptual types with Rust serde shapes ([STATE_MODEL.md](STATE_MODEL.md)); catch command payload mistakes early |
| **Tauri 2** | Native window/tray, `invoke` + events, `externalBin` sidecar packaging — matches MVP process model |
| **Team cost** | Minimal frontend surface; Svelte learning curve acceptable vs React/Vue for this scope |

**Provisional:** Stack remains contingent on spikes (sidecar spawn/stop + log stream; e2e on device via `gnirehtet` CLI sidecar; APK resource). Electron is fallback only if those fail — frontend structure below should still map (views/stores/commands) if a fallback is required.

**Hard rule:** The UI **never** calls `adb`, never builds shell commands, and never invents install/tunnel/start flows. It only invokes host-orchestrator commands and consumes events ([DESKTOP_ARCHITECTURE.md](DESKTOP_ARCHITECTURE.md) §3–5). The orchestrator, not the UI, spawns `gnirehtet` for those verbs.

---

## 2. App structure (routes / views)

```text
src/
├── routes/                    # or pages/ if not using a router
│   ├── DeviceListView         # serials, adb state, select target
│   ├── ConnectionView         # Run / Stop / Reset tunnel; lifecycle status
│   ├── SettingsView           # ADB path, APK path, port, DNS, routes, tray
│   └── LogsView               # ring buffer of LogLine; filter by level
├── lib/
│   ├── api/                   # typed wrappers around invoke()
│   ├── stores/                # devices, relay, settings, logs, errors
│   ├── components/            # presentational pieces
│   └── types/                 # mirrored orchestrator types (camelCase IPC)
├── App.svelte                 # shell layout: nav + content + status bar
└── main.ts
```

| View | Primary purpose | Key commands |
|------|-----------------|--------------|
| **Device list** | Show `list_devices` results; pick serial; show unauthorized/offline | `list_devices`, `ensure_adb` |
| **Connection** | One-click Run; session Stop; Reset tunnel; show device + relay state machine | `run`, session teardown, `reset_tunnel` |
| **Settings** | Persistable prefs; validate paths via backend | settings get/set, `ensure_adb` |
| **Logs** | Scrollable `LogLine` feed; optional export | subscribe only (no adb logcat from UI directly) |

**Assumption:** Simple client-side navigation (tabs or lightweight router) is enough; deep linking is not required for MVP.

---

## 3. Component hierarchy

```text
App
├── TitleBar / chrome (platform-dependent)
├── Nav (Devices | Connection | Settings | Logs)
├── StatusBar
│   ├── RelayBadge          ← RelayState store (running + owned-by-session)
│   ├── SelectedDeviceChip  ← devices store
│   └── AdbHealthHint       ← ensure_adb result / last check
├── DeviceListView
│   ├── AdbMissingBanner
│   ├── DeviceRow[]         (serial, adb state, apk hint, connection state)
│   └── RefreshButton       → list_devices
├── ConnectionView
│   ├── LifecycleStepper    (idle → … → connected | error)
│   ├── RunButton / StopButton / ResetTunnelButton
│   ├── OptionsPanel        (dns, routes, port — pass-through knobs; port sets session)
│   └── VpnPermissionHint   (awaiting_vpn_permission copy)
├── SettingsView
│   ├── PathField (adb)     # empty → env ADB → PATH
│   ├── PathField (apk)     # empty → env GNIREHTET_APK → bundled
│   ├── NumberField (port)  # default 31416; applied on next Run, not mid-session
│   ├── DnsRoutesFields
│   └── TrayPrefs
├── LogsView
│   ├── LogFilter
│   └── LogVirtualList
└── TrayMenu (native, configured from Rust + mirrored actions)
    ├── Show window
    ├── Stop tether           # session teardown (device + owned relay)
    └── Quit
```

Presentational components receive props from stores; they do not call `invoke` themselves except via thin `lib/api` helpers used in view actions.

**Lead Architect decision (stop policy):** default Run / Stop ≈ upstream `run` teardown. One-click Stop (Connection + tray) and Quit stop the device client **and** the relay **if this app started that relay**. Do not expose “stop device only” as the default control; that is optional advanced UI later.

---

## 4. Invoking commands and subscribing to events

**Lead Architect decision (IPC casing):** Rust core stays `snake_case`. Payloads cross IPC as camelCase via `serde(rename_all = "camelCase")` (or equivalent). TypeScript types in [STATE_MODEL.md](STATE_MODEL.md) are the camelCase IPC shapes. Command **names** remain snake_case (`ensure_adb`, `list_devices`, `reset_tunnel`, …).

### Commands (`invoke`)

```typescript
// lib/api/orchestrator.ts (conceptual)
import { invoke } from '@tauri-apps/api/core';

export const ensureAdb = () => invoke<AdbInfo>('ensure_adb');
export const listDevices = () => invoke<DeviceInfo[]>('list_devices');
export const run = (serial: string, opts?: RunOptions) =>
  invoke<void>('run', { serial, opts });
export const stop = (serial: string) => invoke<void>('stop', { serial });
export const resetTunnel = (serial: string, port?: number) =>
  invoke<void>('reset_tunnel', { serial, port });
export const startRelay = (port?: number) => invoke<void>('start_relay', { port });
export const stopRelay = () => invoke<void>('stop_relay');
// settings get/set similarly
```

- All payloads are typed; no free-form strings for shell.
- Long operations (`run`, `install`) return when orchestration has kicked off or completed a phase — exact sync/async contract is an orchestrator spike; UI always also listens for events for truth.
- **One-click Stop** should invoke **session teardown** (device `stop` + `stop_relay` when owned), not `stop(serial)` alone. Implementation may be a dedicated command or an orchestrator-owned sequence; the UI must not invent a third policy.
- `stop_relay` is a no-op / refused if the current relay is **not** owned by this session (foreign process).

### Events (`listen`)

```typescript
import { listen } from '@tauri-apps/api/event';

listen<DeviceSnapshot>('DeviceChanged', (e) => devicesStore.apply(e.payload));
listen<RelayStatePayload>('RelayState', (e) => relayStore.set(e.payload));
listen<LogLine>('LogLine', (e) => logsStore.append(e.payload));
listen<AppError>('Error', (e) => errorsStore.push(e.payload));
```

| Event | UI reaction |
|-------|-------------|
| `DeviceChanged` | Replace/patch device list; update Connection stepper |
| `RelayState` | Badge + disable/enable relay-dependent actions; show ownership (ours vs foreign) |
| `LogLine` | Append to ring buffer (cap size); optional auto-scroll |
| `Error` | Banner/toast; map to device `error` state display |

Channels (Tauri channels) are reserved if high-volume log streaming needs backpressure; MVP default is event-per-line.

---

## 5. Error display patterns

| Pattern | When | UX |
|---------|------|-----|
| **Inline field error** | Settings path invalid | Under the input; from command `Result` Err |
| **Banner** | adb missing, no devices, relay_error | Persistent until resolved or dismissed |
| **Toast** | Transient failures (tunnel reset fail) | Auto-dismiss + still in Logs |
| **Lifecycle stepper error** | Device state `error` | Show last error message + Retry → reconnecting path |
| **VPN consent copy** | `awaiting_vpn_permission` | **Lead Architect decision:** heuristic state. Prompt: *“Check your phone and accept the VPN / connection request.”* Do not fake success. On timeout → recoverable `error` (`vpn_permission_timeout` or equivalent) with Retry. No APK/protocol change. Real status channel later if heuristics are noisy. |

Errors from orchestrator include stable `code` strings where possible (`adb_not_found`, `unauthorized`, `port_in_use`, `install_failed`, `vpn_denied`, `vpn_permission_timeout`, `foreign_relay`, …) so UI can localize without parsing English log lines. Log lines remain secondary evidence.

Settings copy should state precedence: **app setting > `ADB` / `GNIREHTET_APK` env > PATH / bundled APK** (Lead Architect decision). Empty path fields mean “use env, then default.”

Port field copy: changing the default port applies on the **next** Run, not mid-session (stop + re-run). See [STATE_MODEL.md](STATE_MODEL.md) § port session.

---

## 6. Accessibility and theming (light touch)

- Prefer native focus order; buttons and device rows keyboard-activatable.
- Status colors not sole indicator (include text: “Connected”, “Unauthorized”, “Accept VPN on the phone”).
- Respect OS light/dark if cheap via CSS variables; Settings may offer override later (not MVP-critical).
- Log view: monospace, sufficient contrast; do not rely on color alone for log level (prefix `INFO`/`ERROR`).
- Tray icon states should have tooltip text matching relay/device summary.

---

## 7. Testing approach (frontend)

| Layer | Approach |
|-------|----------|
| **Unit** | Store reducers / state transitions with mocked events (Vitest or equivalent) |
| **Component** | Svelte Testing Library: Run button disabled when no device; error banner on `Error` event; VPN hint visible in `awaiting_vpn_permission` |
| **Contract** | Type fixtures matching Rust serde JSON (**camelCase** payloads); fail CI on drift |
| **E2E** | Deferred to app-level spikes with real/fake orchestrator; UI tests against a **mock orchestrator** that emits the four event types — never against live adb in unit CI |

Do **not** require a physical device for frontend unit/component tests.

---

## 8. Explicit boundary checklist

- [ ] No `Command` / shell APIs from the frontend
- [ ] No hardcoded `adb …` strings in Svelte/TS
- [ ] No direct filesystem writes except via settings commands
- [ ] No assumption of IPv6 or multi-device autorun in MVP UI copy
- [ ] Connection actions only: orchestrator `run` / session Stop / `reset_tunnel` (granular relay start|stop is not the default UX)
- [ ] TS types are camelCase IPC shapes; command names stay snake_case
- [ ] VPN permission copy is a prompt + timeout-to-error, not a fake connected state

---

## 9. Cross-references

- IPC list and security: [DESKTOP_ARCHITECTURE.md](DESKTOP_ARCHITECTURE.md) §5–7  
- Types and state machines: [STATE_MODEL.md](STATE_MODEL.md)  
- When views mount vs tray; hidden poll: [DESKTOP_LIFECYCLE.md](DESKTOP_LIFECYCLE.md)
