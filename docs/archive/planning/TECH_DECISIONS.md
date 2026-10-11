# Technical Decisions

**Status:** Draft v0.1  
**Upstream pin:** `Genymobile/gnirehtet` @ `1eb2e58` / v2.5.1  
**Method:** Facts vs assumptions, risks, unknowns, alternatives, recommendation, rejected options.

---

## TD-01 — Preserve the Android VPN client

### Facts
- Client is a focused Java/Android codebase under `app/` (~15 production classes).
- It implements `VpnService`, intent-driven start/stop, DNS/routes configuration, and resilient reconnect via `PersistentRelayTunnel`.
- Host already drives it with documented `adb` + intent commands.
- License is Apache-2.0 (compatible with OSS successor with attribution).

### Assumptions
- VpnService behavior on modern Android (API 34+) remains compatible with this APK with at most minor manifest/foreground-service updates.
- Users accept a one-time on-device VPN permission dialog (inherent to the design).

### Risks
- Upstream unmaintained: future Android policy changes may force a fork.
- Manifest exports `GnirehtetActivity` with `WRITE_SECURE_SETTINGS` permission attribute — unusual; need to validate install/start behavior on current devices (unknown until device matrix testing).

### Unknowns
- Exact failure modes on Android 13–16 (foreground service types, VPN always-on interactions).
- Whether Google Play distribution is desired later (affects VPN app policies); sideload via adb is the historical path.

### Alternatives
| Option | Pros | Cons |
|--------|------|------|
| **A. Reuse APK as-is** | Lowest risk; proven path | May need small maintenance fork |
| B. Rewrite client in Kotlin | Modern code | High risk, no GUI benefit |
| C. Different capture approach (no VpnService) | — | Root or unsupported; conflicts with product goals |

### Recommendation
**A. Reuse / minimally fork the Android client.** Rewrite is unjustified for a desktop GUI project.

### Rejected
- **B, C** — unnecessary rewrite risk; C violates no-root goal.

---

## TD-02 — Rust relay as networking core

### Facts
- Upstream explicitly prefers Rust over Java for the relay.
- Rust code lives in `relay-rust/`; library API is `relay(port)`.
- Uses `mio = "0.6"`, edition 2018, deps last aligned to the v2.5.x era.
- Java relay remains a maintained fallback in upstream docs but is not the recommended path.

### Assumptions
- Behavioral parity between shipping Rust binary and our managed process is sufficient for MVP.
- We can vendor or submodule `relay-rust` under Apache-2.0 terms.

### Risks
- **Dependency age:** mio 0.6 is far behind current mio; security/compat risk over time.
- **Lifecycle:** `relay()` runs a blocking poll loop; clean stop from a GUI needs process kill or new API.
- **macOS artifacts:** README ships macOS zip at **v2.2.1** while Linux/Windows are **v2.5.1** — build-from-source likely required for macOS parity.
- Single-threaded design is fine for typical device counts but is a scalability ceiling.

### Unknowns
- Effort to bump mio / Rust edition without subtle packet bugs.
- Whether in-process linking beats sidecar for crash isolation (sidecar wins isolation; in-process wins control).

### Alternatives
| Option | Pros | Cons |
|--------|------|------|
| **A. Sidecar / child: stock Rust `gnirehtet`** | Max reuse; crash isolation; matches upstream CLI | Process mgmt; older deps frozen inside binary |
| B. In-process `relaylib` | Tighter integration | Needs shutdown API; shares fate with UI process |
| C. Keep Java relay | Cross-platform jar | JRE dependency; upstream says prefer Rust |
| D. Rewrite relay (tokio/smoltcp/etc.) | Modern stack | Highest risk; easy to break TCP semantics |

### Recommendation
**A for MVP.** Treat Rust relay as the core networking engine, delivered as a **managed binary**. Plan a later ADR for dependency modernization or in-process embed once stop/health APIs exist. **Do not rewrite the relay for MVP.**

### Rejected
- **C** for default path (heavier runtime; contrary to upstream guidance).
- **D** for MVP (reinvents the hardest part).
- **B** deferred until lifecycle story is designed.

---

## TD-03 — Desktop application stack (provisional)

### Hypothesis (from product brief)
Rust + **Tauri 2** + **Svelte/TypeScript**.

### Facts
- Tauri 2 documents **sidecar** embedding of external binaries (`bundle.externalBin`, `shell` plugin spawn/kill) — fits gnirehtet relay/CLI.
- Tauri host is Rust — same language as relay, enabling future in-process option without Electron’s Node host.
- Svelte is a capable UI layer; not uniquely required by gnirehtet.

### Assumptions
- Team can accept webview UI constraints (native menus/tray via Tauri plugins).
- Code signing + sidecar signing on macOS is operationally solvable.
- “OSS friendly” includes acceptable webview dependency (platform WebView2 / WKWebView / WebKitGTK).

### Risks
- Sidecar lifecycle (restart, orphan prevention, port-in-use) needs custom code; upstream Tauri issues note missing batteries-included lifecycle plugin.
- Linux WebKitGTK variance across distros.
- Packaging adb: we will **not** fully solve “adb not installed” beyond detection + docs in MVP.

### Unknowns
- Real UX performance for log streaming and multi-device.
- Whether contributors prefer React; Svelte is a preference, not a protocol constraint.

### Alternatives
| Option | Pros | Cons |
|--------|------|------|
| **A. Tauri 2 + Svelte/TS** | Small runtime; Rust host; official sidecars | Webview quirks; younger ecosystem than Electron |
| B. Tauri 2 + React/Solid | Larger hiring pool (React) | Same host pros/cons; framework churn |
| C. Electron + TS | Mature tooling; many ADB GUI precedents | Larger footprint; weaker “reuse Rust relay in-process” story |
| D. Pure Rust UI (egui/iced) | Single language | Weaker desktop polish/OSS contributor pool for UI |
| E. Flutter desktop | Nice UI toolkit | Extra toolchain; awkward relay integration |
| F. Wails (Go) | Simple | Wrong language gravity vs Rust relay |

### Recommendation
**Provisionally A (Tauri 2 + Svelte/TS)**, contingent on validation gates:

1. Spike: spawn/stop upstream `gnirehtet relay` as sidecar; stream stdout to UI.
2. Spike: `adb devices` watch + install/start/stop one device end-to-end on Linux **and** Windows (macOS as soon as signing allows).
3. Packaging spike: ship APK as a resource + resolve `GNIREHTET_APK`.
4. Compare contributor docs friction vs Electron only if spikes fail.

Until gates pass, treat the stack as **hypothesis**, not law.

### Rejected (for now)
- **D, E, F** — worse fit for GUI+Rust-relay combination.
- **C** — fallback if Tauri packaging/webview blockers prove severe; not default.
- **B** — acceptable variant; decide only if team skill mix demands React. Not an architecture fork.

---

## TD-04 — Orchestration: wrap CLI semantics before inventing a new daemon API

### Facts
- `main.rs` already encodes the product operations GUI needs (`run`, `start`, `stop`, `tunnel`, `relay`, `install`, …).
- Behavior includes APK version check (`REQUIRED_APK_VERSION_CODE = "9"`), reverse setup, and autorun monitoring.

### Recommendation
MVP orchestrator **mirrors CLI semantics** (same adb argument shapes and ordering). Avoid a parallel “clever” API that drifts from upstream. Later, extract shared Rust modules from `main.rs` / `adb_monitor.rs` into a library used by both CLI and GUI.

### Rejected
- Greenfield orchestrator that re-implements adb flows from memory without tests against upstream behavior.

---

## TD-05 — Repository shape (successor vs overlay)

### Recommendation (provisional)
**New OSS repo** that:

- Vendors or submodules `relay-rust` + `app` (or consumes release artifacts),
- Adds `desktop/` (Tauri app) + docs,
- Keeps a thin compatibility CLI **or** shells out to patched gnirehtet.

Forking the entire Genymobile repo is viable but mixes abandoned Java relay + Gradle world with a new desktop app; a **successor repo with clear vendoring** is cleaner for contributors, provided Apache-2.0 attribution is maintained.

### Unknown
- Whether Genymobile trademark/name constraints affect branding (“gnirehtet” vs new name). Legal/community check required before final naming.

---

## Decision log summary

| ID | Decision | State |
|----|----------|-------|
| TD-01 | Reuse Android VPN client | **Accepted** |
| TD-02 | Rust relay via managed process for MVP | **Accepted** |
| TD-03 | Tauri 2 + Svelte/TS | **Provisional** (gates) |
| TD-04 | Orchestrator mirrors CLI semantics | **Accepted** |
| TD-05 | Successor repo + vendor upstream pieces | **Provisional** |
