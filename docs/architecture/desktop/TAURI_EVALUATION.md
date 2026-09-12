# Tauri 2 Evaluation — Gnirehtet Desktop Shell

**Status:** Draft v0.2  
**Owner:** Desktop Application Engineer  
**Date:** 2026-09-12  
**Decision:** **Tauri 2 + Svelte/TypeScript (provisional)** with **sidecar MVP** (`gnirehtet` CLI spawn)  
**Fallback:** Electron, only if named spikes fail (see §8)

**Binding inputs:**

| Document | Role |
|----------|------|
| [`research/lead-architect-baseline.md`](research/lead-architect-baseline.md) | **BINDING** process boundary: Rust relay is a **managed child/sidecar**; process kill/lifecycle owned by the shell |
| [`research/architect-decisions-2026-09-12.md`](research/architect-decisions-2026-09-12.md) | **BINDING** Lead Architect: spawn `gnirehtet` for CLI verbs; session-scoped stop; settings > env > default; IPC casing; VPN heuristic; hidden poll |
| [`research/framework-comparison.md`](research/framework-comparison.md) | Primary framework evidence (scores, plugins, packaging, official URLs) |
| [`research/gnirehtet-core-boundaries.md`](research/gnirehtet-core-boundaries.md) | Frozen networking + CLI surface; sidecar is the **MVP** integration (M2 lib extract keeps the UI API) |

**Related:** [`DESKTOP_ARCHITECTURE.md`](DESKTOP_ARCHITECTURE.md) (shell layout already assumes this decision).

**Non-goals of this document:** Redesign relay/TCP/packet protocol; rewrite VpnService; vendor platform-tools; treat in-process `relaylib` as MVP; reimplement CLI verbs with hand-rolled adb.

---

## 1. Decision summary

| Item | Choice | Status |
|------|--------|--------|
| Framework | **Tauri 2** | Recommended on evidence (§3, §4) |
| Frontend | **Svelte + TypeScript** (Vite) | **Provisional** pending spikes (§7) |
| Rust core integration (MVP) | **Managed sidecar** (`bundle.externalBin` + `tauri-plugin-shell`); spawn `gnirehtet` for `install\|reinstall\|start\|stop\|tunnel\|relay\|run` | **Binding** — Lead Architect |
| Orchestration extract (M2) | Shared Rust lib behind the **same** UI-facing API | **Planned** — no UI change; still no protocol rewrite |
| Rust core integration (later) | In-process `relaylib` link into `src-tauri` | **Deferred** until a graceful stop / async API exists |
| Fallback framework | **Electron** | Only if Tauri spikes fail (§8) |
| Networking | Unmodified upstream (VpnService → `adb reverse` → host relay) | Out of scope to change |

**Why Tauri 2 (framework research, adapted to this app):**

1. Official sidecar packaging and spawn/stop via capability-scoped shell plugin ([Embedding External Binaries](https://v2.tauri.app/develop/sidecar/)) — matches the binding MVP.
2. Host orchestrator is Rust anyway (`src-tauri`); Tauri is the lowest-friction place to own sidecar spawn of `gnirehtet` CLI verbs, discovery `adb`, and events ([Calling Rust from the Frontend](https://v2.tauri.app/develop/calling-rust/)).
3. Tray, notification, autostart, single-instance, log, and store plugins map 1:1 to a long-running utility ([plugin catalog](https://v2.tauri.app/plugin/)).
4. First-class `svelte-ts` template ([create-tauri-app](https://github.com/tauri-apps/create-tauri-app)).
5. Small installers (no bundled Chromium); MIT OR Apache-2.0.
6. If a future Rust Architect adds graceful stop / async API, **in-process linking is a Cargo dependency change**, not a framework rewrite. That option is unique to Tauri (and Slint) among the compared stacks.

**Why sidecar now (Lead Architect, binding — do not contradict):**

Upstream `relaylib::relay(port)` is a **blocking mio loop with no graceful stop API**. Kill/lifecycle is the shell’s problem. In-process linking would pin that loop inside the UI process (or force unsafe thread-kill). MVP therefore ships a **pinned upstream `gnirehtet` binary** as a child and treats process death as stop. Framework research’s preference for in-process library linking is **acknowledged and deferred**, not adopted for MVP.

**Assumption:** “Rust core” means Genymobile `relay-rust` / `relaylib` @ v2.5.1 lineage (Lead Architect: `@ 1eb2e58 / v2.5.1`). The shell does not fork the protocol.

---

## 2. Evaluation criteria and method

**Research date:** 2026-09-12. Method inherited from [`research/framework-comparison.md`](research/framework-comparison.md): official docs, plugin catalogs, release notes, and cited third-party measurements. Metrics without a source are marked **unknown**. Scores are **evidence-informed judgments for this project**, not universal rankings.

### Scoring legend

| Score | Meaning |
|-------|---------|
| **5** | Excellent fit for this use case |
| **4** | Strong; minor gaps or friction |
| **3** | Workable; notable cost or risk |
| **2** | Possible but poor fit |
| **1** | Poor / high friction |

### Criteria (this product)

| # | Criterion | Why it matters here |
|---|-----------|---------------------|
| 1 | Rust / native integration | Host orchestrator is Rust; sidecar spawn/stop + future library path |
| 2 | Bundle size & memory | Utility left running in the tray |
| 3 | Tray / notifications / autostart / single-instance | Required product surface |
| 4 | Background / close-to-tray | Primary UX pattern |
| 5 | Packaging maturity | Linux + Windows primary; macOS best-effort |
| 6 | Security model | UI must not get a raw shell; capability allowlists for sidecar |
| 7 | Frontend (Svelte + TS) | Stated team preference |
| 8 | Maturity 2025–2026 | GA enough to ship a tray utility |
| 9 | Licensing | Apache-2.0 attribution already required for gnirehtet; avoid extra tax |
| 10 | Dev velocity / hiring | Small team; Rust already exists in-house |

### Method notes

- **Lead Architect overrides framework research on process topology.** Framework comparison §1 preferred “link the crate; do not sidecar Rust you already own.” That advice assumes a cooperative library API. Upstream `relay(port)` is not cooperative. Sidecar is therefore the correct *integration mode* inside the winning *framework*.
- **TUN/elevation caveat:** Framework comparison discusses privileged TUN helpers. Core-boundaries §2 and §9 are binding: **the PC does not create a TUN**; VpnService lives on Android; **no root/admin is required on the host** for the core path. This evaluation does **not** propose an elevated helper. The sidecar exists because of the blocking mio loop, not because of privilege isolation.
- Third-party size/RAM numbers are treated as **one data point**, not gospel (framework comparison §1 criterion 2).

---

## 3. Comparison table

Scores from [`research/framework-comparison.md`](research/framework-comparison.md) summary scorecard, unchanged except where this app’s **sidecar-first** MVP changes the *interpretation* of criterion 1 (noted below).

| Criterion | Tauri 2 | Electron | Flutter desktop | Qt (Quick/Widgets) | Slint |
|-----------|---------|----------|-----------------|--------------------|-------|
| 1. Rust / native FFI | **5** | 3 | 3 | 3–4 | **5** |
| 2. Bundle size & memory | **4–5** | 2 | 3–4 | 4 | **5** |
| 3. Tray / notif / autostart / single-instance | **5** | **5** | 3–4 | **5** | 4 |
| 4. Background / close-to-tray | **5** | **5** | 4 | **5** | 4–5 |
| 5. Packaging maturity | **4** | **5** | 3–4 | **5** | 3 |
| 6. Security model | **5** | 3 | 3–4 | 4 | 4 |
| 7. Frontend (Svelte + TS) | **5** | **5** | 1 | 2 | 2 |
| 8. Maturity 2025–2026 | **4** | **5** | 4 | **5** | 3–4 |
| 9. Licensing | **5** | **5** | **5** | 3–4 | 3–4 |
| 10. Dev velocity / hiring | 4 | **5** | 4 | 4 | 2–3 |
| **Weighted fit (this project)** | **Strongest** | Strong shell; fallback if spikes fail | Weak UI-stack fit | Strong native; higher cost | Strong Rust UI; new desktop surface |

**Criterion 1, this project:** Tauri’s 5 is **not** “we will link `relaylib` in MVP.” It is: (a) first-class sidecar (`externalBin` + scoped `shell:allow-spawn`); (b) Rust host-orchestrator with typed commands/events/channels; (c) cheapest future path to in-process if a stop API appears. Electron’s 3 is napi-rs/Neon or child process — sidecar *works* (Wirebound precedent) but you pay Chromium + Node ABI forever. Flutter needs `flutter_rust_bridge`. Qt needs cxx-qt or a C ABI. Slint can call Rust in-process but fights Svelte+TS and has a newer tray surface (1.17, Jun 2026).

### Also-rans (brief)

| Option | Evidence | Why not |
|--------|----------|---------|
| **Wails** (Go + WebView) | Similar size story; v3 beta systray/packaging ([v3 systray](https://v3.wails.io/features/menus/systray/), [v3 beta](https://v3.wails.io/blog/wails-v3-beta/)) | Backend is Go. Rust core fit is FFI or a second sidecar. Only for Go teams. |
| **Neutralinojs** | Tiny WebView + JS `os.setTray` | Limited native depth; Rust is an external process or custom extension. Weak for a productized networking shell. |
| **Native trio** (SwiftUI / WinUI / GTK) | Best per-OS tray UX | 3× UI codebases. Highest cost. Lead Architect already chose a cross-platform shell. |
| **Iced / egui / Dioxus** | Pure-Rust UI | Tray/notifications/packaging thinner than Tauri/Qt/Slint for settings + logs + notifications polish. |

### When each main alternative would win

| Choose… | If… |
|---------|-----|
| **Electron** | Tauri spikes fail (sidecar spawn/stop + log stream; e2e install/start/stop; APK as resource) **or** you must have pixel-identical Chromium and the npm ecosystem. Accept ~80–200+ MB installers. This is the **named fallback**, not a co-equal plan. |
| **Qt** | Team drops Svelte, staffs C++/QML (or cxx-qt), and accepts LGPL/commercial review. Strongest decades-long tray-utility track record. |
| **Slint** | Team drops Svelte for `.slint`, accepts royalty-free attribution or commercial license, and accepts newer desktop tray + DIY packaging. |
| **Flutter** | Org standard is Dart and Svelte is dropped. |

**Runner-up if Tauri is abandoned after spikes:** Electron (same Svelte+TS UI, proven sidecar wrapping — [Wirebound](https://github.com/man612/wirebound)). **Runner-up if the team drops web UI:** Slint (pure Rust) or Qt (native + packaging).

---

## 4. Tauri 2 deep dive for this app

Official site: https://v2.tauri.app/  
GA: **2 October 2024** ([Tauri 2.0 Stable Release](https://v2.tauri.app/blog/tauri-20/)). Architecture: Rust core + OS WebView (WebView2 / WKWebView / WebKitGTK), not bundled Chromium ([Security: Not Bundling WebViews](https://v2.tauri.app/security/)).

### 4.1 IPC: commands, events, channels

Docs: [Calling Rust from the Frontend](https://v2.tauri.app/develop/calling-rust/), [IPC concept](https://v2.tauri.app/concept/inter-process-communication/).

| Mechanism | Shape | Use in this app |
|-----------|-------|-----------------|
| **Commands** (`invoke`) | Typed request/response; JSON-serializable; `async` + `Result` | Host-orchestrator API: `list_devices`, `ensure_adb`, `install`, `start`, `stop`, `reset_tunnel`, `start_relay` / `stop_relay`, one-click `run`. UI never talks to adb or the sidecar directly. Payloads: Rust `snake_case` + `serde(rename_all = "camelCase")` at IPC (Lead Architect). |
| **Events** | Fire-and-forget, bidirectional, JSON-only | `DeviceChanged`, `RelayState`, `Error`, and low-volume `LogLine`. Matches Lead Architect event set. |
| **Channels** (`tauri::ipc::Channel`) | Ordered, high-throughput streaming | **Optional** for continuous sidecar stdout/stderr if event-per-line is too chatty. Architecture draft assumes event-per-line is enough for MVP ([DESKTOP_ARCHITECTURE.md](DESKTOP_ARCHITECTURE.md) §5). **Assumption:** start with events; promote logs to a Channel only if the UI janks. |

Capability model: IPC rejects unauthorized calls; filesystem/shell/plugin access is gated by [capabilities](https://v2.tauri.app/security/capabilities/) and [permissions](https://v2.tauri.app/security/permissions/). Sidecar spawn must be allowlisted (`shell:allow-spawn` / `shell:allow-execute` with **named** binaries). The UI cannot construct arbitrary shell strings.

**Trust boundary (official):** capabilities protect the **frontend** attack surface. A compromised Rust process still has full OS privileges — same as any native app. Do not claim the capability DSL sandboxes the relay.

### 4.2 Plugins this app actually needs

Catalog (updated Jul 2026): https://v2.tauri.app/plugin/

| Need | Mechanism | Docs | Notes for this app |
|------|-----------|------|-------------------|
| System tray | Built-in `tray-icon`; `TrayIcon` / `TrayIconBuilder` | [System Tray](https://v2.tauri.app/learn/system-tray/) | Close-to-tray; status icon; Quit. **Linux:** click/hover events unsupported; context menu on right-click still works (docs, Apr 2026). Deb may need `libappindicator3-1` ([Debian guide](https://v2.tauri.app/distribute/debian/)). |
| Notifications | `tauri-plugin-notification` | [Notifications](https://v2.tauri.app/plugin/notification/) | Device unauthorized, VPN consent needed, tunnel lost, relay error. |
| Autostart | `tauri-plugin-autostart` (desktop only) | [Autostart](https://v2.tauri.app/plugin/autostart/) | Optional OS login start; can pass `--minimized`. |
| Single instance | `tauri-plugin-single-instance` (register **first**) | [Single Instance](https://v2.tauri.app/plugin/single-instance/) | One tray app. **Linux:** D-Bus; Snap/Flatpak need `--own-name` / `--talk-name` for `org.{id}.SingleInstance`. |
| Settings KV | `tauri-plugin-store` | [Store](https://v2.tauri.app/plugin/store/) | ADB path, APK override, port/DNS/routes, tray prefs. **Lead Architect:** explicit app setting > `ADB` / `GNIREHTET_APK` env > PATH / bundled APK. |
| Logs | `tauri-plugin-log` | [Logging](https://v2.tauri.app/plugin/logging/) | Shell-owned log file / levels. Sidecar lines are a **separate** pipe (upstream logger is hardcoded Info → stdout/stderr; no `RUST_LOG` — core-boundaries §8, [issue #118](https://github.com/Genymobile/gnirehtet/issues/118)). Do not patch `relaylib` for GUI logs. |
| Sidecar / spawn | `tauri-plugin-shell` + `bundle.externalBin` | [Embedding External Binaries](https://v2.tauri.app/develop/sidecar/) (updated Jun 2026) | **MVP path.** Spawn bundled `gnirehtet` for `install\|reinstall\|start\|stop\|tunnel\|relay\|run`. Direct `adb` only for discovery/health. Pipe stdout/stderr. |

`create-tauri-app` includes a **svelte-ts** template ([Create a Project](https://v2.tauri.app/start/create-project/)). Community `tauri-plugin-svelte` exists for persisted stores (listed on the plugin page) — optional, not required.

### 4.3 Lifecycle (window + tray + sidecar)

Supported close-to-tray pattern (framework comparison §1 criterion 4; official tray example):

1. On `CloseRequested`: `window.hide()` + `api.prevent_close()`.
2. Tray “Show” / (where supported) left-click: `unminimize` / `show` / `set_focus`.
3. Tray “Quit”: **session teardown** — stop device client, **kill sidecar only if this app started it**, then `app.exit(0)`.

Relay stop is **process kill**, not a graceful API. Document that in-flight sockets drop. Orphan recovery on next startup is a shell responsibility (architecture §4). Autostart may pass `--minimized`.

**Headless-with-tray is a first-class UX** for this app class. Do not quit when the last window closes.

### 4.4 Packaging

Official hub: https://v2.tauri.app/distribute/

| Platform | Built-in targets | This product |
|----------|------------------|--------------|
| Linux | `deb`, `rpm`, `appimage`; docs also discuss Snap, Flatpak, AUR | Primary. Build on the **oldest** glibc/WebKitGTK 4.1 baseline you need (e.g. Ubuntu 22.04 / Debian 12) — newer hosts raise glibc ([Debian](https://v2.tauri.app/distribute/debian/)). |
| Windows | WiX `.msi`, NSIS `-setup.exe`; WebView2 bootstrapper modes | Primary. MSI needs a Windows host (WiX); NSIS can cross-compile ([Windows Installer](https://v2.tauri.app/distribute/windows-installer/)). |
| macOS | `.app`, `.dmg`; App Store path; signing + notarization | **Best-effort** until signing/sidecar story is proven (Lead Architect). |

**Ship (Lead Architect packaging):**

```text
Tauri app + gnirehtet rust sidecar + gnirehtet.apk + NOTICE/attribution (Apache-2.0)
```

- Sidecar: `bundle.externalBin` with target-triple suffix (`gnirehtet-x86_64-unknown-linux-gnu`, etc.).
- APK: bundled **resource** (not an externalBin); resolved **setting > `GNIREHTET_APK` env > bundle**.
- `adb`: **user-provided** (setting > `ADB` env > PATH). Do not vendor full platform-tools in MVP.
- Attribution: NOTICE for Genymobile gnirehtet (Apache-2.0) plus Tauri/deps.

**Precedent:** [Wirebound](https://github.com/man612/wirebound) (Electron) already wraps the released Rust binary + APK this way. Tauri’s `externalBin` is the same topology with a smaller runtime.

### 4.5 Suggested layout (does not change networking)

```text
workspace/
  src-tauri/                 # host-orchestrator; plugins; sidecar supervisor
  ui/                        # Svelte + TS (Vite)
  sidecars/gnirehtet*        # pinned upstream binary (per-target)
  resources/gnirehtet.apk    # stock APK
  NOTICE
```

UI talks **only** to orchestrator commands/events. Orchestrator **spawns the `gnirehtet` sidecar** for `install`, `reinstall`, `run`, `start`, `stop`, `tunnel`, `relay` (optional `autorun` later). Direct `adb` is discovery/health only (`devices`, unauthorized, missing). No ad-hoc adb sequences in the frontend **or** as hand-rolled backend reimplementations of those verbs (Lead Architect decision).

---

## 5. Integration modes: sidecar CLI spawn (MVP), M2 lib extract, in-process later

Lead Architect **binds MVP to sidecar spawn of the `gnirehtet` CLI**. Core-boundaries §4 still allows a library extract later; that is **M2** (same UI-facing API) and/or in-process `relaylib` **after** a stop API exists. Framework research’s preference for linking applies to those later steps, not MVP. The v0.1 “docs allow both” orchestration fork is **closed**.

### 5.1 Why the library path is unsafe for MVP

From Lead Architect and core-boundaries §4:

- Canonical API: `relaylib::relay(port: u16) -> io::Result<()>` — the mio selector, clients, router, TCP/UDP, packetizer.
- It is a **blocking** event loop.
- There is **no graceful stop** (no token, no `stop()`, no async cancel).
- Putting that loop on a worker thread inside the Tauri process means:
  - `stop_relay()` cannot return the thread to a clean state without killing the thread (undefined behavior / leaked mio/fds) or exiting the whole app.
  - A relay panic or mio error can take down the UI process.
  - Close-to-tray + Quit semantics become entangled with an uninterruptible loop.

**Assumption:** We will not wrap `relay()` in `std::thread::spawn` and `std::mem::forget` / forced unwind as an MVP “stop.” That would be a networking-adjacent hack, not a shell feature.

### 5.2 Sidecar MVP (chosen)

```text
Svelte UI  --invoke/listen-->  host-orchestrator (src-tauri)
                                    |
                                    | spawn / pipe / kill
                                    v
                              gnirehtet sidecar
                              (CLI + relaylib in its own process)
                                    |
                              TCP :31416 via adb reverse
                                    v
                              Android VpnService
```

| Topic | Sidecar behavior |
|-------|------------------|
| Start | `tauri-plugin-shell` spawn of allowlisted `externalBin`; args are CLI verbs (`install`, `reinstall`, `start`, `stop`, `tunnel`, `relay [-p PORT]`, `run`) |
| Logs | Piped stdout (info) + stderr (error); line-parse `YYYY-MM-DD HH:MM:SS.mmm LEVEL target: message` (core-boundaries §8) |
| Stop | Kill the child (SIGTERM then SIGKILL / Windows TerminateProcess) **if this app started it**. **This is the stop API** for an owned relay. Do not kill a foreign relay. |
| Crash | Child exit → `RelayState = relay_error`; UI process stays up |
| Version pin | Ship a known upstream build; Apache-2.0 NOTICE |
| Orchestration | **Lead Architect decision:** spawn `gnirehtet` for CLI verbs. Direct `adb` only for `devices` / unauthorized / missing. Do **not** hand-roll those verbs in `src-tauri`. |

**Do not** use sidecar merely to “wrap Rust we already own as a library” *in the abstract* (framework comparison’s warning). We use it because **this** library has no stop API and because packaging parity with the released binary is the lowest protocol risk (core-boundaries §10).

### 5.3 M2 — shared Rust lib extract (planned)

**Lead Architect decision:** after MVP, extract a shared Rust orchestration lib **without changing the UI-facing API** (`list_devices`, `ensure_adb`, `install`, `start`, `stop`, `reset_tunnel`, `start_relay` / `stop_relay`, `run`, same events).

- Purpose: stop paying a process hop for every `install`/`start`/`stop`/`tunnel` once the spawn path has pinned behavior to upstream.
- Does **not** authorize hand-rolled adb in MVP, a protocol change, or an in-process blocking `relay()` loop.
- Relay may still be a child process until a graceful stop / async API exists (§5.4).

### 5.4 In-process library (deferred — future option)

Revisit **only if** a Rust Architect (or upstream) adds one of:

- a graceful stop (cancellation token, `stop()` on a handle, or mio wakeup that exits the loop);
- an async `relay` that runs on Tokio / Tauri’s runtime and is abortable;
- a structured status channel **outside** the packet protocol (optional; not required to switch modes).

Then the framework-research path becomes viable: add `relay-rust` as a Cargo dependency of `src-tauri`, run networking on a managed task, expose `start_relay` / `stop_relay` as real in-process commands, stream logs via Channels. **No UI rewrite. No protocol change.** Sidecar supervisor code is deleted or kept as a debug escape hatch.

| | Sidecar (MVP) | In-process (later) |
|--|---------------|--------------------|
| Stop | Process kill | Cooperative cancel |
| Isolation | Relay crash ≠ UI crash | Shared process — need panic boundaries |
| Packaging | `externalBin` + APK resource | Smaller (no second bin); still ship APK + NOTICE |
| Protocol risk | Zero (released binary) | Build against `relay-rust` source; still must not change internals |
| Elevation | Not why we sidecar (PC has no TUN) | Same |
| Framework fit | Tauri shell plugin | Tauri Cargo link (framework research “preferred”) |

**Out of scope for the switch:** IPv6, VpnService rewrite, host TUN, multi-device autorun as a requirement.

---

## 6. Platform risks

### 6.1 WebView differences

Tauri uses **system** WebViews: WebView2 (Windows, Chromium-based), WKWebView (macOS), WebKitGTK (Linux). CSS/JS feature gaps are real. Test the settings + log pane on Linux GTK/WebKit carefully (framework comparison, Tauri risk 1).

Memory: bundle-size advantage vs Electron is robust; **RAM advantage is workload- and metric-dependent**. GitHub [tauri#5889](https://github.com/tauri-apps/tauri/issues/5889) shows PSS/USS accounting can make WebKit-based Tauri look *worse* than Electron for heavy UIs. WebView2 is often similar to Electron at runtime. **Measure this UI**; do not cite marketing tables. A markdown-editor micro-benchmark (~5 MB vs ~150 MB installers; ~42 MB vs ~187 MB idle RAM, [johal.in](https://johal.in/benchmarks-performance-tauri-20-vs-electron-30-markdown)) is **one data point**.

### 6.2 Linux packaging

- Build on oldest required glibc + **WebKitGTK 4.1** (Ubuntu 22.04 / Debian 12 class).
- Tray: `libappindicator3-1` / ayatana / StatusNotifier — DE variance (GNOME vs KDE). Official: tray **clicks unsupported** on Linux; menu works.
- Single-instance: D-Bus name `org.{id}.SingleInstance`; Snap/Flatpak plugs/slots.
- Cross-compilation and AppImage quirks are real CI work (framework comparison criterion 5).
- Test GNOME and KDE for tray + notifications (freedesktop) + autostart (`XDG`).

### 6.3 Windows WebView2

- Evergreen WebView2 is present on current Windows 10/11; bootstrapper modes exist for older images ([Windows Installer](https://v2.tauri.app/distribute/windows-installer/)).
- Sidecar is `gnirehtet.exe`; quoting/paths for user `adb` are a common failure mode (architecture platform matrix).
- USB drivers / “no devices” is an ADB problem, not a Tauri problem — surface it in UX.
- Authenticode strongly recommended (SmartScreen). MSI needs a Windows host if using WiX.

### 6.4 macOS signing / sidecar (best-effort)

Lead Architect: **macOS is best-effort until signing/sidecar story is proven.**

Risks to spike, not to promise:

- Hardened Runtime + notarization for the `.app` **and** the nested sidecar binary.
- Quarantine / `com.apple.quarantine` on downloaded sidecars.
- `bundle.externalBin` path resolution inside the `.app` bundle.
- App Store vs Developer ID (out of MVP unless requested).

Do not block Linux/Windows MVP on this.

### 6.5 Cross-cutting (not framework-specific)

| Risk | Implication |
|------|-------------|
| ADB dependency | Discover PATH / settings; udev (Linux), USB drivers (Windows). Independent of UI toolkit. |
| Code signing | User trust > framework choice. |
| No structured core IPC | Infer state from adb + process + log lines (core-boundaries §5). |
| Upstream logging | Hardcoded Info; no `RUST_LOG`. GUI tees pipes; does not patch core. |

---

## 7. Spike plan / go–no-go criteria

Lead Architect: *“Tauri 2 + Svelte/TS is **provisional** pending spikes: sidecar spawn/stop + log stream; e2e install/start/stop on a real device; APK as resource. Electron is fallback only if those fail.”*

These spikes are **go/no-go for the stack**, not for networking design.

### Spike A — Sidecar spawn / stop + log stream

| | |
|--|--|
| **Goal** | Prove `externalBin` + `tauri-plugin-shell` can start `gnirehtet` (`relay` and other CLI verbs), pipe stdout/stderr into the UI, and **stop by process kill** without zombie/orphan on Linux and Windows. |
| **Pass** | Start → `RelayState=running`; parsed `LogLine`s appear; Stop → process gone within a bounded timeout; second Start works; Quit kills child then exits; capability allowlist rejects a non-named binary. |
| **Fail** | Cannot reliably kill on a primary OS; pipes deadlock; `externalBin` triple suffix unusable in CI; capability model cannot name the sidecar. |
| **On fail** | Trigger Electron fallback for the same topology (child process + pipes). Do not “fix” by linking `relaylib` in-process. |

### Spike B — E2E install / start / stop on a real device

| | |
|--|--|
| **Goal** | Host-orchestrator commands against a real Android device by **spawning `gnirehtet`** for `install` / `tunnel` / `start` / `stop` / `run`. Direct `adb` only for discovery/health. One-click `run` ≈ upstream `run`. |
| **Pass** | Device listed; APK installed via sidecar; reverse `localabstract:gnirehtet tcp:<port>` (default 31416); START/STOP via CLI; VPN-consent heuristic representable; session Stop tears down client **and** owned relay; Reset tunnel recovers unplug/replug without reinstall. |
| **Fail** | Tauri spawn/quoting cannot drive the `gnirehtet` sidecar reliably on a primary OS; cannot pass APK path / env parity (setting > `ADB` / `GNIREHTET_APK` > default). |
| **On fail** | Electron fallback **if** the failure is Tauri-specific. If the failure is ADB/device, it is not a framework fail. |

### Spike C — APK as resource

| | |
|--|--|
| **Goal** | Bundle stock `gnirehtet.apk` as a resource; orchestrator resolves it; `install` uses that path; `GNIREHTET_APK` override still works. |
| **Pass** | Fresh install (no setting, no env) finds bundled APK; app setting beats env; env beats bundle; path validation rejects traversal / non-files. |
| **Fail** | Resource path unresolved after packaging on a primary OS (especially NSIS/deb working-directory issues). |
| **On fail** | First try packager-path fixes. If Tauri resource resolution is the blocker, Electron fallback (Wirebound already ships APK this way). |

### Spike hygiene (recommended, not Lead Architect gates)

- Linux tray: icon + **right-click menu** works on GNOME and KDE (clicks optional).
- WebView2 bootstrap on a clean Windows VM.
- macOS: one Developer ID + sidecar notarization probe — **informational**, not a go/no-go for MVP.

### What spikes must not do

- Redesign relay/TCP or call `relaylib::relay` in-process as a “shortcut.”
- Reimplement `install\|start\|stop\|tunnel\|run` with hand-rolled adb (Lead Architect: CLI spawn only).
- Vendor full platform-tools.
- Treat `autorun`, IPv6, or VpnService rewrite as spike scope.

---

## 8. Recommendation and fallback triggers

### Recommendation

**Adopt Tauri 2 + Svelte/TypeScript as the provisional desktop stack. Integrate by spawning the pinned `gnirehtet` sidecar for CLI verbs + relay in MVP. Plan an M2 shared-lib extract that does not change the UI-facing API. Keep in-process `relaylib` as a later option gated on a graceful stop / async API. Keep Electron as the only named fallback.**

This reconciles the two research inputs:

| Source | Claim | How this doc treats it |
|--------|-------|------------------------|
| Framework comparison | Choose Tauri 2; prefer in-process crate link | **Framework: accepted.** Integration mode: **not** accepted for MVP. |
| Lead Architect (baseline) | Sidecar MVP; kill = stop; Electron if spikes fail | **Binding.** Entire §5 and §7. |
| Lead Architect (2026-09-12) | Spawn `gnirehtet` for CLI verbs; session-scoped stop; settings > env; M2 lib extract | **Binding.** Closes the “docs allow both” fork. |
| Core boundaries | Do not change networking; sidecar *or* library OK | Sidecar now; M2 lib extract later (same UI API); protocol frozen. |

### Fallback triggers (Electron)

Switch **only if** one or more Lead Architect spikes **fail for a Tauri-specific reason** on a **primary** OS (Linux or Windows):

1. Sidecar spawn/stop + log stream cannot be made reliable.
2. E2E install/start/stop cannot be driven through Tauri’s process/ADB spawning (and the same sequence works from a plain shell).
3. APK cannot be shipped/resolved as a packaged resource.

**Non-triggers:** macOS signing friction; Linux tray click events missing (menu is enough); WebView CSS polish; desire to link `relaylib` before a stop API exists; installer size vs Electron (Electron is larger); hiring familiarity with Node.

If fallback trips: reuse the Svelte+TS UI and the same orchestrator *semantics*; replace `src-tauri` with an Electron main process that **spawns the same `gnirehtet` sidecar** (CLI verbs + relay) + APK resource. Wirebound is the existence proof, not a dependency.

### If Electron also fails

That is an ADB/packaging/product problem, not a reason to in-process-link a blocking mio loop or to redesign networking. Escalate to Lead Architect; do not silently change the process boundary.

---

## 9. Sources

Primary research files (cite these first):

- [`research/framework-comparison.md`](research/framework-comparison.md)
- [`research/lead-architect-baseline.md`](research/lead-architect-baseline.md)
- [`research/architect-decisions-2026-09-12.md`](research/architect-decisions-2026-09-12.md)
- [`research/gnirehtet-core-boundaries.md`](research/gnirehtet-core-boundaries.md)

### Tauri (from framework comparison)

- https://v2.tauri.app/
- https://v2.tauri.app/blog/tauri-20/
- https://v2.tauri.app/learn/system-tray/
- https://v2.tauri.app/plugin/
- https://v2.tauri.app/plugin/notification/
- https://v2.tauri.app/plugin/autostart/
- https://v2.tauri.app/plugin/single-instance/
- https://v2.tauri.app/plugin/store/
- https://v2.tauri.app/plugin/logging/
- https://v2.tauri.app/develop/calling-rust/
- https://v2.tauri.app/concept/inter-process-communication/
- https://v2.tauri.app/develop/sidecar/
- https://v2.tauri.app/security/
- https://v2.tauri.app/security/capabilities/
- https://v2.tauri.app/security/permissions/
- https://v2.tauri.app/distribute/
- https://v2.tauri.app/distribute/debian/
- https://v2.tauri.app/distribute/windows-installer/
- https://github.com/tauri-apps/create-tauri-app

### Electron

- https://www.electronjs.org/docs/latest/tutorial/tray
- https://www.electronjs.org/docs/latest/api/app
- https://github.com/electron-userland/electron-builder
- https://blog.openreplay.com/comparing-electron-tauri-desktop-applications/

### Flutter

- https://docs.flutter.dev/platform-integration/desktop
- https://docs.flutter.dev/reference/supported-platforms
- https://github.com/fzyzcjy/flutter_rust_bridge
- https://pub.dev/packages/desktop_tray
- https://pub.dev/packages/flutter_desktop_notifications

### Qt

- https://doc.qt.io/qt-6/licensing.html
- https://doc.qt.io/qt-6/qml-qt-labs-platform-systemtrayicon.html
- https://www.qt.io/development/open-source-lgpl-obligations
- https://www.kdab.com/cxx-qt-0-7/

### Slint

- https://slint.dev/blog/slint-1.17-released
- https://docs.slint.dev/latest/docs/slint/reference/window/systemtrayicon/
- https://slint.dev/pricing
- https://slint.dev/agreements/slint-royalty-free-license.pdf

### Other / benchmarks (use cautiously)

- https://github.com/tauri-apps/tauri/issues/5889 (memory methodology caveat)
- https://johal.in/benchmarks-performance-tauri-20-vs-electron-30-markdown
- https://v3.wails.io/blog/wails-v3-beta/
- https://v3.wails.io/features/menus/systray/

### Gnirehtet / process boundary

- https://github.com/Genymobile/gnirehtet — README, DEVELOP.md, v2.5.1
- https://raw.githubusercontent.com/Genymobile/gnirehtet/master/relay-rust/src/lib.rs — `relaylib::relay`
- https://raw.githubusercontent.com/Genymobile/gnirehtet/master/relay-rust/src/logger.rs
- https://github.com/Genymobile/gnirehtet/issues/118 — logging threshold
- https://github.com/man612/wirebound — Electron GUI wrapping official Rust binary + ADB (fallback existence proof)

---

*Re-verify plugin support tables and Linux WebKitGTK package names before locking CI images. Lead Architect process boundary (sidecar CLI spawn in MVP; M2 lib extract without UI API change) outranks framework-research integration preference until a graceful stop / async API exists.*
