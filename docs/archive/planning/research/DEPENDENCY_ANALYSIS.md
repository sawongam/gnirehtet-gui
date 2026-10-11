# DEPENDENCY_ANALYSIS.md

**Upstream pin:** `/workspace/gnirehtet-upstream` @ `1eb2e58` / v2.5.1  
**Sources:** `Cargo.toml`, `Cargo.lock`, root/`app`/`relay-*` `build.gradle`, `gradle-wrapper.properties`, `LICENSE`, scripts.  
**Date context:** research performed 2026-09-12; “outdated” is relative to that era vs lockfile pins from the v2.5.1 release line.

---

## 1. License (project)

| Item | Value | Evidence |
|------|-------|----------|
| Project license | **Apache License 2.0** | `LICENSE` (Apache 2.0 text); every inspected source file header “Licensed under the Apache License, Version 2.0” |
| Copyright | Genymobile (headers say Copyright (C) 2017 Genymobile; authors in Cargo.toml Romain Vimont) | `Cargo.toml` authors; file headers |
| Successor obligation | Retain license/notice; attribution | Apache-2.0 |

**Runtime dependency licenses:** not fully audited here (would need `cargo license` / Gradle license plugin on a build machine). Notable crates are common permissive (MIT/Apache) ecosystem crates; **unknown until generated SBOM** — mark as **unknown / needs SBOM**.

---

## 2. Android / Gradle toolchain

| Dependency | Version in tree | Role | Status (2026 lens) |
|------------|-----------------|------|---------------------|
| Gradle Wrapper | **5.4.1** | Build | **Outdated** (current Gradle is 8.x). Evidence: `gradle/wrapper/gradle-wrapper.properties` `distributionUrl=...gradle-5.4.1-all.zip` |
| Android Gradle Plugin | **3.5.0** | APK build | **Outdated** / incompatible with modern Studio. Evidence: root `build.gradle` `classpath 'com.android.tools.build:gradle:3.5.0'` |
| `compileSdkVersion` | **28** | Compile | **Outdated**. `build.gradle` `ext.compileSdkVersion = 28` |
| `buildToolsVersion` | **28.0.3** | Build tools | **Outdated**. same file |
| `minSdkVersion` | **21** | Runtime floor | Still a valid product choice. `app/build.gradle` |
| `targetSdkVersion` | **29** | Target | **Outdated** vs Play/policy (34+). `app/build.gradle` |
| `versionCode` / `versionName` | **9** / **2.5.1** | APK identity | Current release. `app/build.gradle` |
| Repositories | **jcenter()** + google() | Resolution | **jcenter unmaintained/shut down** — **P0 build risk**. `build.gradle` buildscript + allprojects |
| JUnit | **4.12** | Unit tests | Old but usable. `app/build.gradle` `testImplementation`; `relay-java` `testCompile` |
| Espresso | **2.2.2** | androidTest | **Outdated**; Support library era. `app/build.gradle` |
| Android Support Test Runner | `android.support.test.runner.AndroidJUnitRunner` | Instrumentation | **Outdated** (AndroidX migration not done) |
| Checkstyle | **6.19** | Java style | **Outdated**. `config/java-checkstyle.gradle` |
| Java language (relay) | **Java 8** (documented) | relay-java | README requires JRE 8; application plugin, no toolchain block |
| `relay-java` Gradle configs | `compile` / `testCompile` | Dep config | **Deprecated** configuration names |

### Android runtime permissions / platform deps (not Maven)

- `ACCESS_NETWORK_STATE`, `FOREGROUND_SERVICE`, `INTERNET` — `AndroidManifest.xml`
- VpnService / ConnectivityManager — Android SDK
- No third-party networking libraries in `app/`; `implementation fileTree(dir: 'libs')` only (empty-by-convention)

### Signing

- Optional `RELEASE_STORE_FILE` / passwords via `config/android-signing.gradle` reading `gradle.properties` — not vendored secrets in tree.

---

## 3. Rust (`relay-rust`) — direct dependencies

From `relay-rust/Cargo.toml` (semver reqs) and resolved versions in `Cargo.lock`:

| Crate | Cargo.toml req | Locked version | Purpose | Notes |
|-------|----------------|----------------|---------|-------|
| **mio** | `0.6` | **0.6.23** | async I/O | **Unmaintained line** for new work; API replaced in 0.7+ |
| **slab** | `0.4` | **0.4.7** | Token→handler map | Still ok; used by `selector.rs` |
| **log** | `0.4` | **0.4.17** | logging facade | Ok |
| **chrono** | `0.4` | **0.4.23** | log timestamps | Pulls time/wasm bits via features default; used in `relay.rs` deadlines |
| **byteorder** | `1.3` | **1.4.3** | endian helpers | Ok |
| **rand** | `0.7` | **0.7.3** | random TCP ISN | **Outdated** (rand 0.8/0.9 exist); 0.7 still builds |
| **ctrlc** | `3.0` + feature `termination` | **3.2.4** | Ctrl+C / termination | Ok for CLI |

**Edition:** `2018` (`Cargo.toml`).  
**Package/lib names:** package `gnirehtet` `2.5.1`; lib **`relaylib`**.  
**Release profile:** `lto = true`.

### Notable transitive / platform crates in `Cargo.lock` (mio 0.6 stack)

| Crate | Locked | Concern |
|-------|--------|---------|
| iovec | 0.1.4 | Legacy mio dependency |
| net2 | 0.2.38 | Legacy; largely superseded by std |
| winapi | 0.2.8 **and** 0.3.9 | Dual major versions |
| kernel32-sys | 0.2.2 | Ancient Windows sys |
| fuchsia-zircon | 0.3.3 | Dead platform bits via mio |
| cfg-if | 0.1.10 **and** 1.0.0 | Duplication |
| time | 0.1.45 | Old `time` via chrono 0.4.23 tree |
| libc | 0.2.139 | Normal |

**Unmaintained / high-churn risk:** the **mio 0.6 + net2 + iovec** cluster is the primary Rust dependency debt (aligns with TECHNICAL_DEBT D-P0-2).

---

## 4. Runtime / operational dependencies (not in Cargo/Gradle)

| Dependency | Version constraint in tree | Role | Evidence |
|------------|---------------------------|------|----------|
| **adb** | “recent” with `adb reverse` (README cites 1.0.36+) | Device control + reverse | README; `exec_adb` |
| **JDK/JRE 8** | Java flavor only | Run `gnirehtet.jar` | README |
| **Rust toolchain** | Capable of edition 2018 + mio 0.6 | Build relay | DEVELOP.md |
| **mingw-w64** (optional) | for Windows cross | `releaseCrossToWindows` | `relay-rust/build.gradle`, DEVELOP.md |
| Host localhost TCP port | default **31416** | Relay listen | `cli_args.rs` / Java `DEFAULT_PORT` |
| adbd | port **5037** | track-devices | `AdbMonitor` |

Env overrides: `ADB`, `GNIREHTET_APK` (Rust/Java mains).

---

## 5. Test-only dependencies

| Stack | Dep | Version |
|-------|-----|---------|
| Android | junit | 4.12 |
| Android | espresso-core | 2.2.2 (androidTest) |
| relay-java | junit | 4.12 |
| relay-rust | (dev) none extra — tests use std + crates already linked | — |

No `mockito`, no fuzzers, no packet golden-file framework.

---

## 6. Packaging / distribution deps

| Artifact | How built | Evidence |
|----------|-----------|----------|
| `gnirehtet.apk` | `:app:assembleRelease` | `release` script copies `app/build/outputs/apk/release/gnirehtet-release.apk` |
| Linux rust zip | `cargo build --release` → strip | `release` script |
| Windows rust zip | `--target=x86_64-pc-windows-gnu` + `gnirehtet-run.cmd` | `release` script |
| Java zip | `gnirehtet.jar` + shell/cmd wrappers | `relay-java/scripts/*`; `java -jar gnirehtet.jar "$@"` |

**Unknown:** macOS build not in `release` script (only java, rust-linux64, rust-win64). README historically mentions macOS zips — **not evidenced in this `release` file**.

---

## 7. Outdated / unmaintained summary

| Area | Verdict | Successor action |
|------|---------|------------------|
| jcenter + AGP 3.5 + Gradle 5.4.1 | **Unusable long-term** | New Android build; don't inherit root Gradle for desktop |
| mio 0.6 stack | **Frozen tech** | Sidecar binary MVP; schedule upgrade ADR |
| rand 0.7 | Outdated | Bump when touching deps |
| JUnit 4 / Espresso 2 / Support test | Outdated | AndroidX if revitalizing app tests |
| Checkstyle 6.19 | Outdated | Replace or drop |
| Apache-2.0 project | **OK** | Keep |
| Third-party license SBOM | **Unknown** | Generate before shipping GUI redistributable |

---

## 8. Finding-format entries (deps-focused)

### F-DEP-1 — jcenter() still declared

1. **File/module:** root `build.gradle`
2. **What it does:** Resolves Android plugin/deps from jcenter + google.
3. **Why it matters:** Build reproducibility is broken on clean networks.
4. **Whether reusable:** No.
5. **Refactor required:** Maven Central / Google only; bump AGP.
6. **Risk:** P0 build break.
7. **Evidence:** `build.gradle:9-11,22-24` `jcenter()`.

### F-DEP-2 — mio 0.6.23 locked

1. **File/module:** `Cargo.toml` / `Cargo.lock`
2. **What it does:** Pins async runtime for entire relay.
3. **Why it matters:** Dictates selector API and transitive junk crates.
4. **Whether reusable:** Binary yes; source evolution blocked.
5. **Refactor required:** mio 1.x migration project + tests.
6. **Risk:** High long-term.
7. **Evidence:** lock `name = "mio"` `version = "0.6.23"`.

### F-DEP-3 — Zero third-party deps on Android client

1. **File/module:** `app/build.gradle` dependencies block
2. **What it does:** Only fileTree jars + test libs.
3. **Why it matters:** **Positive** — small APK surface, fewer CVEs.
4. **Whether reusable:** Yes.
5. **Refactor required:** Prefer keep dependency-free if forking.
6. **Risk:** Low.
7. **Evidence:** `app/build.gradle:24-30`.

### F-DEP-4 — Dual winapi majors in lockfile

1. **File/module:** `Cargo.lock`
2. **What it does:** winapi 0.2.8 and 0.3.9 both present.
3. **Why it matters:** Symptom of stale graph (mio/ctrlc/chrono trees).
4. **Whether reusable:** N/A.
5. **Refactor required:** Clears on modern mio/ctrlc bump.
6. **Risk:** Low–Medium (Windows builds).
7. **Evidence:** Cargo.lock entries at lines ~532 and ~538 (lockfile).

### F-DEP-5 — Java 8 + application plugin relay

1. **File/module:** `relay-java/build.gradle`, README
2. **What it does:** Fat-main jar `gnirehtet.jar`, JRE 8.
3. **Why it matters:** GUI default should avoid forcing JRE on users (upstream agrees).
4. **Whether reusable:** Fallback only.
5. **Refactor required:** None for Rust-first GUI.
6. **Risk:** Low if not default.
7. **Evidence:** README Java 8; `mainClassName = 'com.genymobile.gnirehtet.Main'`.

### F-DEP-6 — License clarity vs crate SBOM gap

1. **File/module:** `LICENSE` vs transitive crates
2. **What it does:** Project is Apache-2.0; transitive not enumerated in-repo.
3. **Why it matters:** Shipping a GUI+relay bundle needs attribution file.
4. **Whether reusable:** Project license yes.
5. **Refactor required:** `cargo deny` / `cargo license` in CI; document.
6. **Risk:** Medium compliance until done.
7. **Evidence:** `LICENSE` present; no `NOTICE` of third-party crates in tree listing; **unknown** full graph licenses.

---

## 9. Alignment with reuse strategy

- **MVP sidecar** freezes the Rust dependency graph inside a binary → avoids forcing GUI toolchain onto mio 0.6.
- **APK rebuild** is a separate dependency upgrade track (Gradle/AGP/SDK) from the desktop UI stack.
- Prefer naming/extracting **`gnirehtet-relay`** without dragging adb into that crate’s Cargo.toml (keeps relay deps = mio/slab/log/… only).

---

*End of DEPENDENCY_ANALYSIS.md*
