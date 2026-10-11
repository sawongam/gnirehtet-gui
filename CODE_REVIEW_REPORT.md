# gnirehtet-gui: Code Review Report

- **Date:** 2026-09-27
- **Reviewed revision:** `dev` @ `f4885e1` (44 commits, 2026-09-12 → 2026-09-20)
- **Scope:** the entire repository: 5 Rust crates, the Tauri backend, the SvelteKit frontend, build and packaging, and docs.

Part 1 describes the code as it was when I started. Part 2 lists what was changed in this pass and how it was verified.

---

## Part 1: Assessment of `f4885e1`

### 1.1 What the project is

This is a desktop GUI (Tauri 2 + SvelteKit + Rust) for [Genymobile/gnirehtet](https://github.com/Genymobile/gnirehtet). gnirehtet does reverse tethering: an Android phone uses the computer's internet over USB/adb, without root. The app does not run the relay in-process. It supervises the stock upstream `gnirehtet relay` binary as a child process (a "sidecar") and drives `adb` for install, tunnel, start and stop.

| Component | Path | LOC | Origin |
|---|---|---:|---|
| Vendored relay (`relaylib`) | `crates/gnirehtet-relay` | ~4,900 | Upstream v2.5.1, unmodified |
| ADB client | `crates/gnirehtet-adb` | ~1,360 | Upstream extract + additions |
| CLI | `crates/gnirehtet-cli` | ~690 | Upstream port |
| Relay supervisor | `crates/gnirehtet-controller` | ~950 (incl. 466 lab example) | New |
| Tauri backend | `apps/desktop/src-tauri` | ~1,130 | New |
| Frontend | `apps/desktop/src` | ~2,800 (1,565 in one file) | New |
| Docs | `docs/` | 63 files / 11,593 lines | New |

### 1.2 Overall level

**An advanced prototype, labelled "release candidate". It was not publishable.**

| Area | Grade | Summary |
|---|:--:|---|
| Rust core (adb/controller) | **B** | Correct ownership model (it only kills the relay it started, and probes the port before claiming it), well unit-tested, but carries dead API and ticket-jargon comments |
| Tauri backend | **C** | Works in the happy path, but blocking commands freeze the UI, relay-start logic is duplicated, and there is one release-blocking packaging bug |
| Frontend | **D+** | Compiles clean, but it is a 1,565-line monolith and roughly 40% of the visible UI is non-functional mock-up. A **working** session is reported as an **error** after 60 s |
| Security | **C-** | Unscoped shell-spawn capability, CSP disabled, and shell injection into the phone via DNS/route strings |
| Build / DX | **D** | A fresh clone cannot build (a missing sidecar breaks `cargo check`), and there is no CI and no fetch script |
| Tests | **C** | 67 Rust unit tests pass. Frontend: 0 tests |
| Docs | **D** | 11.6k lines of internal planning notes and no end-user install guide |

**Progress estimate: about 55% of the way to a publishable v0.1.** The hard parts are done and correct: relay supervision, ownership, port probing and crash detection. Most of the remaining work is fixing what the UI tells the user, packaging, and documentation.

### 1.3 Critical defects (would break a real user)

1. **The packaged app cannot find its own APK (release blocker).** Tauri bundles `../../../resources/*` into `/usr/lib/gnirehtet-gui/_up_/_up_/_up_/resources/gnirehtet.apk`. The runtime search (`gnirehtet-adb::apk_search_candidates`) only probes paths next to `/usr/bin/gnirehtet-desktop` and the CWD. Every install or first run on a phone without the client therefore fails with `APK_MISSING`. `RC_PACKAGING.md` checked that the file was *listed* in the `.deb`, never that it was *found*.
2. **A working connection is shown as an error.** The UI never leaves "Waiting for VPN", even after the phone connects. After 60 s it flips to `START_TIMEOUT` / "Error", even though traffic is flowing. The relay's `Client #N connected` log line was parsed, but deliberately never used as a connection signal. On the Android side, gnirehtet opens the relay socket *only after* the VPN is established, so that line is a reliable "connected" signal.
3. **The UI freezes during adb operations.** All Tauri commands were synchronous `fn`s, and Tauri 2 runs those **on the main thread**. `adb install` (seconds) or a hung `adb` (unbounded) froze the whole window.
4. **Shell injection into the device.** `dns_servers` and `routes` were passed unvalidated into `adb shell am start --esa dnsServers <value>`. `adb shell` joins its arguments into a device-side shell command line, so a value like `8.8.8.8;reboot` executes on the phone.
5. **Stop on exit hit the wrong device.** Teardown ran on both `ExitRequested` and `Exit`. The second pass had no remembered serial, so it called `adb shell am start …STOP` **without `-s`**. That stopped gnirehtet on whatever single device was attached, even one this app never started.
6. **adb calls had no timeouts.** A wedged `adb` (offline transport, a stuck install) blocked the session mutex forever, and with it every later command, including Quit.

### 1.4 Major issues

- **Over-broad capabilities:** `shell:allow-spawn`, `shell:allow-execute`, `shell:allow-stdin-write` and `shell:allow-kill` had no scope, so any script running in the webview could execute arbitrary programs. The frontend never used the shell or opener plugins. `csp: null` disabled Content Security Policy.
- **Remote font load:** `app.html` pulled Inter from Google Fonts. That is an offline-hostile network call and a privacy leak for a local utility.
- **Fresh clone does not build:** `externalBin` requires `binaries/gnirehtet-<triple>`, which is gitignored, and the "fix" was to hand-edit `tauri.conf.json`. There was no script to fetch the pinned upstream artifacts and no checksum verification.
- **Duplicated relay-start logic:** `start_relay` and `run_session` each re-implemented spawn, epoch and pump with different error mapping (`run_session` could leak the `INTERNAL` code).
- **Poor error messages:** adb stdout/stderr was inherited (invisible in a GUI), so failures read `Command adb [...] returned with value 1`. `am start` exits 0 even when it prints `Error: Activity not started`, so failures were silently treated as success.
- **Panics on edge input:** `.expect("invalid ADB value")` on a non-UTF-8 `ADB` env var, `.expect("apk path is not valid UTF-8")`, and `status.signal().unwrap()`.
- **`adb reverse` left behind** after Stop.

### 1.5 "AI slop" / non-functional code

The frontend shipped a lot of mock-up chrome with no behaviour behind it:

- A header **search box** labelled `aria-label="Search (visual)"` that searched nothing.
- A **Network Traffic** card with `—` values, a "Last 10 minutes" dropdown that did nothing, and "Coming soon". Also a whole **Traffic** page saying it is unavailable.
- **Quick actions** and **Settings**: three switches ("Auto-start relay", "Keep ADB connected", "Verbose logs"), all labelled *stub*, persisting nothing.
- A "**Good morning!**" greeting, a `MoreHorizontal` icon with no menu, "View all →", "Learn more →" pointing at the stub settings, and a fake phone illustration in the sidebar.
- A `?preview=` **dev-mock system** (about 100 lines) that injected fake devices and a fake "Connected" hero into the production component.
- Placeholder-only fields (`androidVersion` is never populated by the backend).
- Code comments full of internal ticket references (`P0-R5`, `P0-Q1/Q3`, `V3 §20`, `ERROR_UX`, `EVENT_STATUS_MAP`), 114 occurrences, meaningless to an outside contributor.
- `void relayHealthy; void layers.clientAccepted;` to silence unused-variable warnings in `deriveSessionChip`, a function that could never return its most important state.
- Docs: 63 planning files ("lock Sangam approved SaaS dashboard visual SoT", "Phase 0 R3 LogLine strip PASS", three superseded visual specs), but no end-user installation guide.

### 1.6 What was good

- The **ownership model** is right: the port is probed before spawn (`PORT_IN_USE` without claiming ownership), only a relay this session started is ever killed, and the shared adb server is never touched.
- **Crash detection** uses an epoch counter, so an intentional stop is not reported as a crash, and the lock is not held while reading pipes.
- Vendored upstream code is kept pristine, with Apache-2.0 headers preserved.
- Rust unit tests use stub relay scripts to test ownership and crash handling without a device.
- Frontend TypeScript is strict, `svelte-check` is clean, and `npm audit` is clean.

---

## Part 2: Changes made in this pass

All changes are uncommitted on `dev`, so they can be reviewed as one diff. Old
files that were replaced or moved are staged as deletions or renames; new files
are untracked. Nothing was committed or pushed.

### 2.1 Critical defects: all fixed

| # | Defect | Fix |
|---|--------|-----|
| 1 | Packaged app cannot find its APK | The resource is mapped to `gnirehtet.apk` at the bundle root (`/usr/lib/gnirehtet-gui/gnirehtet.apk`). `paths.rs` resolves it through Tauri's `resource_dir()`. Verified in the built `.deb`, `.rpm` and AppImage. |
| 2 | Working connection shown as an error | The relay's `Client #N connected/disconnected` lines are parsed (`relay_output.rs`) into `relay-client` events. The UI moves from *Waiting* to *Connected* on them. The 60 s timeout is gone; after 30 s the waiting text suggests checking the phone instead. |
| 3 | UI freezes during adb calls | Every command that touches adb is `async` and runs on `spawn_blocking`. |
| 4 | Shell injection via DNS/routes | `validate_dns_servers` and `validate_routes` in `gnirehtet-adb` accept only IPv4 addresses and CIDR ranges and re-serialize them. The backend validates again before any adb call. The device serial is validated as well. Covered by tests. |
| 5 | Exit stopped the client on an unrelated device | One shutdown path, on `RunEvent::Exit`. It stops the relay, then stops the client only on the device whose session this app started, with a time limit. `stop` without a serial no longer exists in the app. |
| 6 | No adb timeouts | Every adb command has a timeout (15 s for queries, 30 s for shell commands, 180 s for `install`), after which it is killed. Output is captured with a bounded grace period, so a forked adb server holding the pipe cannot hang the call. |

### 2.2 Other issues fixed

- **Capabilities and CSP.** The shell and opener plugins were removed.
  Capabilities are `core:default` plus `core:window:allow-destroy`. A strict CSP
  is enabled, and the Google Fonts request was removed (system font stack).
- **Error reporting.** adb output is captured and included in errors.
  `am start` output containing `Error` is treated as a failure. Every backend
  error carries a stable code (`DEVICE_UNAUTHORIZED`, `PORT_IN_USE`,
  `ADB_TIMEOUT`, …), and `errors.ts` maps each code to a title and an
  actionable hint.
- **Panics.** Non-UTF-8 paths, signal-terminated processes and poisoned locks
  are handled without panicking.
- **Cleanup.** The reverse tunnel is removed on disconnect. If a connection
  attempt fails after the relay was started, that relay is stopped again.
- **Fresh clone builds.** `scripts/fetch-upstream.mjs` (`npm run setup`)
  downloads the pinned v2.5.1 artifacts, verifies their SHA-256 checksums, and
  builds the relay from the vendored sources for targets upstream does not
  ship (macOS, ARM).
- **Unplug and replug.** The UI detects that the session's device
  disappeared (*Connection interrupted*) and restores the tunnel automatically
  when it returns.
- **"No permissions" (Linux udev).** This device state is parsed and explained.
- **Sidecar name.** The bundled relay is now installed as `gnirehtet-relay`
  instead of `gnirehtet`. The old name clashed with distribution `gnirehtet`
  packages at `/usr/bin/gnirehtet`, which would have made installing both
  impossible.
- **Linux menu entry.** A desktop template makes the menu show "Gnirehtet"
  instead of `gnirehtet-gui`. The package metadata now includes a publisher and
  homepage.

### 2.3 Architecture after the pass

The Rust side has a clear split:

- `gnirehtet-adb` runs adb commands.
- `gnirehtet-controller` supervises the relay process only
  (`RelaySupervisor`); the unused `SessionController` layer and the lab
  example were deleted.
- The desktop backend's `session.rs` owns the session lifecycle.
  `commands.rs` is a thin async layer over it.

The frontend's 1,565-line `+page.svelte` became a small set of modules, about
1,700 lines in total:

- `app.svelte.ts`: state and phases
- `session-logic.ts`: pure transitions, unit-tested
- `api.ts`, `errors.ts`, `validation.ts`, `settings.svelte.ts`
- seven view components

`docs/ARCHITECTURE.md` describes the result.

### 2.4 Removed

- All mock-up UI: the search box, the traffic card and page, stub switches, the
  greeting, dead links, and the `?preview=` mock system.
- Unused UI primitives (collapsible, scroll-area, separator, switch, tooltip),
  the old stores, the AppShell, and the template's leftover SVGs.
- Ticket-jargon comments throughout.
- The shell and opener plugins.
- The duplicated relay-start path.

Every remaining control does what it says:

- **Connect**, **Disconnect**, **Reconnect** and **Restore tunnel**.
- **Reinstall client**.
- The Settings fields (port, DNS, routes, theme), which are persisted and
  validated.
- The log view with **Copy** and **Clear**.
- A diagnostics panel showing the resolved adb, relay and APK.
- A quit confirmation while connected.

The 63 planning documents moved to `docs/archive/planning/` with a README that
marks them as historical.

### 2.5 Tests and quality gates

| Check | Result |
|-------|--------|
| `cargo fmt --all --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean. The vendored relay is exempted through `[lints]` in its manifest; its source is unchanged. |
| `cargo test --workspace` | all pass: adb 26, controller 8, desktop 17, CLI 10, plus the relay's own tests |
| Controller test with the real v2.5.1 relay (`GNIREHTET_TEST_RELAY`) | pass |
| `npm run check` (svelte-check) | 0 errors, 0 warnings |
| `npm test` (vitest) | 18 of 18 pass |
| `npm run tauri build` (Linux) | `.deb`, `.rpm` and `.AppImage` built. Package contents checked: `/usr/bin/gnirehtet-gui`, `/usr/bin/gnirehtet-relay`, `/usr/lib/gnirehtet-gui/gnirehtet.apk`, desktop entry. |

Notes:

- **New tests.** Ten desktop session tests use fake adb and relay scripts.
  They cover connect and disconnect, installing the client first, rejecting
  invalid options, cleanup after a failed client start, unauthorized devices, a
  busy port, a relay crash, an unreachable device on disconnect, shutdown
  without a session, and a missing APK. There are also path-resolution tests
  and frontend tests for validation, phase transitions and error mapping.
- **Flaky tests fixed.** The process-spawning tests were flaky when run in
  parallel: ephemeral-port reuse, and `ETXTBSY` when a freshly written script
  was executed while another thread forked. They are now serialized. The
  controller suite passed ten consecutive runs.
- **UI checked visually.** Screenshots of the UI with a mocked backend were
  reviewed for the main states, including the dark theme and the quit dialog.
- **npm audit.** Three low-severity advisories remain in the `cookie`
  dependency of SvelteKit's dev server. That code is not part of the shipped
  app.

### 2.6 Publishing

Added:

- **Workflows.** `.github/workflows/ci.yml` runs fmt, clippy and tests on
  Linux and Windows, plus the frontend checks.
  `.github/workflows/release.yml` builds Linux, Windows, macOS arm64 and macOS
  x64 installers on a `v*` tag into a draft release.
- **Docs.** New `README.md`, `docs/INSTALL.md`, `docs/TROUBLESHOOTING.md`,
  `docs/ARCHITECTURE.md`, `docs/DEVELOPMENT.md` (including a manual release
  checklist), `CONTRIBUTING.md` and `CHANGELOG.md`. The `NOTICE` now mentions
  the bundled upstream binaries.

### 2.7 Not verified here, and remaining work

- **No real phone was connected** during this pass. Everything up to adb is
  covered by the fake-adb tests and the real-relay test, but the end-to-end flow
  has not been exercised. Run the manual checklist in `docs/DEVELOPMENT.md`
  before tagging v0.1.0. This is the most important remaining step.
- **Workflows not run yet.** The GitHub workflows are syntactically valid but
  have not run, because nothing was pushed. Expect a small fix-up on their
  first run; the Windows and macOS jobs are the most likely to need one.
- **Windows and macOS builds** were not produced locally. The code paths are
  written for them (`CREATE_NO_WINDOW`, `.exe` suffixes, SDK locations) and are
  exercised by CI.
- **AppImage on Arch.** Building the AppImage on Arch requires
  `NO_STRIP=true`, because linuxdeploy's `strip` does not understand
  `.relr.dyn` sections. This is documented, and it does not affect CI on
  Ubuntu 22.04.
- **Unsigned installers.** The Windows installer and the macOS app are not
  signed or notarized. `INSTALL.md` explains the first-run prompts. Signing
  needs certificates the project does not have.
- **Possible later features:** Wi-Fi adb, a system tray, autostart, and
  connecting several phones at once.
