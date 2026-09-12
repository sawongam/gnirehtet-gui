# TECHNICAL_DEBT.md

**Upstream pin:** `/workspace/gnirehtet-upstream` @ `1eb2e58` / v2.5.1  
**Severity:** P0 = blocks successor or likely broken on current toolchains/devices; P1 = high risk / must plan; P2 = cleanup / docs / tests.  
**Risk** column is impact if left unaddressed.

---

## P0 — Build / platform blockers

### D-P0-1 — Android build stack is years out of date (jcenter + AGP 3.5 + Gradle 5.4.1)

1. **File/module:** `build.gradle`, `gradle/wrapper/gradle-wrapper.properties`, `app/build.gradle`
2. **What it does:** Builds the APK with `com.android.tools.build:gradle:3.5.0`, `compileSdkVersion = 28`, `targetSdkVersion 29`, repositories `jcenter()` + `google()`.
3. **Why it matters:** jcenter is shut down; AGP 3.5 / Gradle 5.4.1 will not build cleanly on modern JDKs/Android Studio. Successor cannot ship APK updates without a toolchain bump.
4. **Whether reusable:** Logic of app **yes**; build files **no** as-is.
5. **Refactor required:** Migrate to current AGP/Gradle, Maven Central, raise compile/target SDK carefully; retest VpnService/FGS.
6. **Risk:** **Critical** — cannot rebuild release APK from this tree on a modern host without work.
7. **Evidence:** `build.gradle:4-14` `compileSdkVersion = 28`, `classpath 'com.android.tools.build:gradle:3.5.0'`, `jcenter()`; wrapper `distributionUrl=...gradle-5.4.1-all.zip`; `app/build.gradle:10-11` min 21 / target 29.

### D-P0-2 — Rust async I/O pinned to mio **0.6** (2018-era API)

1. **File/module:** `relay-rust/Cargo.toml`, `relay/selector.rs`, all `Ready`/`PollOpt`/`Evented` usages
2. **What it does:** Entire relay is written against mio 0.6 (`Poll`, `Ready`, `PollOpt`, `Evented`).
3. **Why it matters:** mio 0.6 is obsolete vs mio 0.8/1.x; ecosystem crates moved on; future Rustc/std changes may break the 0.6 stack. Lockfile still pulls `iovec`, `net2`, `fuchsia-zircon`, old `winapi 0.2`.
4. **Whether reusable:** Binary as-is **yes** (sidecar); source modernization **required** before long-term embed.
5. **Refactor required:** Planned mio upgrade with packet conformance tests; or freeze binary and accept dep age.
6. **Risk:** **High** — security/compat debt accumulates; upgrade is a large mechanical rewrite of selector interest APIs.
7. **Evidence:** `Cargo.toml:12` `mio = "0.6"`; `Cargo.lock` `mio 0.6.23`; `selector.rs:18` `use mio::{Event, Evented, Events, Poll, PollOpt, Ready, Token}`.

### D-P0-3 — No cooperative relay shutdown API

1. **File/module:** `lib.rs` `relay()`, `relay.rs` `poll_loop`
2. **What it does:** Infinite loop; returns only on poll/register I/O errors (rare).
3. **Why it matters:** GUI cannot “Stop relay” cleanly in-process; must kill process. Even sidecar needs documented signal/`kill` behavior (Ctrl+C in CLI exits the whole process after `cmd_stop`).
4. **Whether reusable:** Sidecar with OS kill **yes**; library embed **no**.
5. **Refactor required:** Add shutdown (waker/token, or process protocol). Until then mandate sidecar (REUSE_ANALYSIS Architect #2).
6. **Risk:** **High** for in-process designs; **Medium** for sidecar (zombie ports if kill is unclean).
7. **Evidence:** `relay.rs:57-77` `loop`; `lib.rs:23-25`; CLI `ctrlc` → `exit(0)` (`main.rs:370-378`).

---

## P1 — Correctness / product risks

### D-P1-1 — Exported Activity requires `WRITE_SECURE_SETTINGS`

1. **File/module:** `app/src/main/AndroidManifest.xml` `GnirehtetActivity`
2. **What it does:** `android:exported="true"` with `android:permission="android.permission.WRITE_SECURE_SETTINGS"`.
3. **Why it matters:** On normal devices that permission is not granted to third-party apps. Unclear whether `adb shell am start` (same UID as shell) still works on all API levels; GUI docs already flag validation need.
4. **Whether reusable:** Pattern needs **device-matrix verification**.
5. **Refactor required:** Confirm start path; possibly remove permission gate or use a shell-only component; document supported invoke method.
6. **Risk:** **High** — start/stop from host could silently fail on some devices/OEMs.
7. **Evidence:** `AndroidManifest.xml:17-25`; TECH_DECISIONS TD-01 Risks.

### D-P1-2 — Missing `VpnService.onRevoke()` handling

1. **File/module:** `GnirehtetService.java` (no override); platform default
2. **What it does:** Relies on default `stopSelf` / TUN teardown when user revokes VPN or another VPN takes over.
3. **Why it matters:** Forwarder threads / notification / failure UI may not run the same `close()` path as intentional STOP (`FAILURE_MODES.md`).
4. **Whether reusable:** Base behavior ok; lifecycle polish needed.
5. **Refactor required:** Override `onRevoke()` to call the same `close()` / notifier stop as `ACTION_CLOSE_VPN`.
6. **Risk:** **Medium–High** — leaked threads or stale foreground notification (platform-dependent).
7. **Evidence:** `rg onRevoke` empty under `app/`; `FAILURE_MODES.md` revoke row; `close()` only from `ACTION_CLOSE_VPN` (`GnirehtetService:95-97,187-203`).

### D-P1-3 — Foreground service / notification policy debt (targetSdk 29 era)

1. **File/module:** `AndroidManifest.xml` (`FOREGROUND_SERVICE` only), `Notifier.java`, `GnirehtetService`
2. **What it does:** Uses generic FGS + notification channel; no `foregroundServiceType`, no `POST_NOTIFICATIONS`.
3. **Why it matters:** Android 10–14 tightened FGS types and notification permission. Raising targetSdk without updates can break start.
4. **Whether reusable:** Code is a starting point.
5. **Refactor required:** When bumping targetSdk: declare FGS type (e.g. connectedDevice/specialUse per policy), request post-notifications if needed, retest O–16.
6. **Risk:** **High** on modern targets; **Low** if staying on old sideloaded APK forever.
7. **Evidence:** Manifest permissions L6–8; `Notifier` channel `IMPORTANCE_DEFAULT`; no `foregroundServiceType` attribute in manifest.

### D-P1-4 — `AdbMonitor` ignores `ADB` env and hardcodes `"adb"`

1. **File/module:** `adb_monitor.rs` `start_adb_daemon`; Java `AdbMonitor` similarly uses `"adb"` for repair (verify: Rust confirmed)
2. **What it does:** Repair path `Command::new("adb").args(["start-server"])` while CLI uses `get_adb_path()` / `ADB`.
3. **Why it matters:** Custom adb installs (Windows platform-tools path, flatpak, etc.) break autorun repair.
4. **Whether reusable:** After fix.
5. **Refactor required:** Pass adb path into `AdbMonitor`; use same env as CLI.
6. **Risk:** **Medium**.
7. **Evidence:** `adb_monitor.rs:208-212`; contrast `main.rs:40-45`.

### D-P1-5 — `adb_monitor` ↔ `relaylib::byte_buffer` packaging debt

1. **File/module:** `adb_monitor.rs`, `lib.rs`
2. **What it does:** Binary module depends on relay library for a generic buffer.
3. **Why it matters:** Blocks clean crate split (see REUSE_ANALYSIS Architect #1).
4. **Whether reusable:** After decoupling.
5. **Refactor required:** Move buffer or stop re-exporting.
6. **Risk:** **Medium** for architecture; low for runtime.
7. **Evidence:** `adb_monitor.rs:18`; `lib.rs:18`.

### D-P1-6 — Panic paths in relay event handlers

1. **File/module:** `client.rs`, `udp_connection.rs`, `tcp_connection.rs` (pattern)
2. **What it does:** Non-`WouldBlock` errors in `on_ready` → `panic!("Unexpected unhandled error")`; pending source failures can panic.
3. **Why it matters:** One unexpected `io::ErrorKind` kills the entire relay process (all devices).
4. **Whether reusable:** Acceptable for CLI; harsh for always-on GUI sidecar (still better than killing GUI if sidecar).
5. **Refactor required:** Convert panics to connection/client close + log.
6. **Risk:** **Medium–High** availability.
7. **Evidence:** `client.rs:185` `Err(_) => panic!(...)`; `:342` `panic!("Cannot send packet to client...")`; `udp_connection.rs:112` same pattern.

### D-P1-7 — CLI port parse uses `unwrap()`

1. **File/module:** `cli_args.rs`
2. **What it does:** `port = value.into().parse().unwrap();`
3. **Why it matters:** Invalid `-p` aborts with panic instead of usage error (exit 2 path).
4. **Whether reusable:** After hardening.
5. **Refactor required:** `parse().map_err(...)` like other arg errors.
6. **Risk:** **Medium** UX / supervisor noise.
7. **Evidence:** `cli_args.rs:66`.

### D-P1-8 — IPv4-only; IPv6 silently dropped on device

1. **File/module:** `Forwarder.java`, README, IPV6_ANALYSIS.md
2. **What it does:** Non-IPv4 TUN packets logged and dropped; no IPv6 VPN addresses/routes.
3. **Why it matters:** Dual-stack networks / IPv6-only destinations fail; product limitation, not a bug — but a debt vs modern networks.
4. **Whether reusable:** Document clearly in GUI.
5. **Refactor required:** Out of MVP; see IPV6_ANALYSIS.md if ever tackled (large).
6. **Risk:** **Medium** user-visible “no connectivity” for IPv6-only resources.
7. **Evidence:** `Forwarder.java:103-110`; README “does not support IPv6”; `IPV6_ANALYSIS.md`.

### D-P1-9 — Host reverse not restored by the APK

1. **File/module:** control plane in `main.rs` vs device `PersistentRelayTunnel`
2. **What it does:** Device reconnects socket; cannot recreate `adb reverse`.
3. **Why it matters:** After USB replug, GUI/CLI must run `tunnel` again or autorun path.
4. **Whether reusable:** Design is fine if orchestrator owns it.
5. **Refactor required:** GUI state machine (FAILURE_MODES / RECONNECT_DESIGN).
6. **Risk:** **High** if GUI omits it.
7. **Evidence:** `cmd_tunnel`; FAILURE_MODES USB row; `RelayTunnel` cannot call adb.

### D-P1-10 — `START_NOT_STICKY` + process death

1. **File/module:** `GnirehtetService.onStartCommand`
2. **What it does:** Service not restarted by system after kill.
3. **Why it matters:** OEM battery killers drop tether until host `start` again.
4. **Whether reusable:** Document; optional sticky is a product choice (may be undesirable).
5. **Refactor required:** Product decision + maybe restart hint in notification.
6. **Risk:** **Medium**.
7. **Evidence:** `GnirehtetService.java:98` `return START_NOT_STICKY`.

---

## P2 — Cleanup, docs, tests, hygiene

### D-P2-1 — DEVELOP.md references missing `AuthorizationActivity`

1. **File/module:** `DEVELOP.md` vs `app/` tree
2. **What it does:** Docs still point to `AuthorizationActivity.java`; permission UX lives in `GnirehtetActivity`.
3. **Why it matters:** Misleads successor authors.
4. **Refactor required:** Doc fix.
5. **Risk:** Low (confusion).
6. **Evidence:** `DEVELOP.md:145,163`; no such file under `app/` (tree listing).

### D-P2-2 — Typo: `setAsUndernlyingNetwork`

1. **File/module:** `GnirehtetService.java`
2. **Evidence:** L150 call, L155 method name `setAsUndernlyingNetwork` (missing “e” in Underlying).
3. **Risk:** Low (private method).

### D-P2-3 — Test class name typo `TestIPPacketOutputSteam`

1. **File/module:** `app/src/test/.../TestIPPacketOutputSteam.java`
2. **Evidence:** class name “Steam” vs Stream; SOURCE_INDEX notes it.
3. **Risk:** Low.

### D-P2-4 — Thin / missing tests for TCP state machine & orchestration

1. **File/module:** no tests in `tcp_connection.rs` / `TCPConnection.java`; no integration tests for `cmd_start`
2. **What it does:** Unit tests focus on headers/buffers/adb framing/CLI argv.
3. **Why it matters:** mio bumps and refactors are unsafe without a conformance suite.
4. **Refactor required:** Add golden PCAP-style or scripted TCP open/close tests before P0-2 work.
5. **Risk:** Medium (enables other debt to bite).
6. **Evidence:** test file inventory in CODEBASE_MAP / SOURCE_INDEX.

### D-P2-5 — `relay-java` uses deprecated Gradle `compile` / `testCompile`

1. **File/module:** `relay-java/build.gradle`
2. **Evidence:** `compile fileTree...`, `testCompile 'junit:junit:4.12'`
3. **Risk:** Low–Medium when touching Java module on new Gradle.

### D-P2-6 — Checkstyle 6.19 / ancient static analysis

1. **File/module:** `config/java-checkstyle.gradle` `toolVersion = '6.19'`
2. **Risk:** Low.

### D-P2-7 — `cli_args` tests `ACCEPT_ALL` omits `PARAM_PORT`

1. **File/module:** `cli_args.rs` tests
2. **Evidence:** `ACCEPT_ALL = PARAM_SERIAL | PARAM_DNS_SERVERS | PARAM_ROUTES` (no PORT) — port parsing less covered.
3. **Risk:** Low.

### D-P2-8 — Project explicitly unmaintained

1. **File/module:** `README.md`
2. **Evidence:** “not actively maintained anymore, only major blockers…”
3. **Risk:** Process — successor must assume ownership.

### D-P2-9 — Java `TCPConnection` TODO

1. **File/module:** `relay-java/.../TCPConnection.java:466`
2. **Evidence:** `// TODO update only when necessary`
3. **Risk:** Low unknown micro-optimization / interest update.

### D-P2-10 — Dual maintenance of AdbMonitor / CLI in Java and Rust

1. **File/module:** `adb_monitor.rs` vs `AdbMonitor.java`; `main.rs` vs `Main.java`
2. **Risk:** Medium drift (already separate bugfix surfaces).
3. **Refactor required:** GUI standardizes on Rust path only.

### D-P2-11 — `versionCode` parsing is brittle (string find)

1. **File/module:** `must_install_client` in `main.rs`
2. **Evidence:** finds `"    versionCode="` then space; Java uses regex `^    versionCode=(\\p{Digit}+).*`
3. **Risk:** Low–Medium across Android versions / locales? (dumpsys English keys assumed).

### D-P2-12 — Shared global thread pool in `Forwarder`

1. **File/module:** `Forwarder.EXECUTOR_SERVICE` static pool size 3
2. **Evidence:** `Forwarder.java:36`
3. **Risk:** Low (single VPN); slight leak if multiple service instances imagined.

---

## Priority summary for GUI successor

| Priority | Action |
|----------|--------|
| P0 | Treat Rust relay as **frozen sidecar binary**; plan AGP/SDK migration before APK changes; do not embed `relay()` without shutdown. |
| P1 | Validate Activity permission + FGS on device matrix; own adb reverse in orchestrator; fix AdbMonitor `ADB` path when extracting; demote panics when touching relay. |
| P2 | Doc/test hygiene; ignore Java path for product; fix typos when touching files. |

---

*End of TECHNICAL_DEBT.md*
