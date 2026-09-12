# CROSS_PLATFORM.md

| Field | Value |
|-------|-------|
| Status | Draft v0.1 |
| Date | 2026-09-12 |
| Project | gnirehtet desktop GUI |
| Related | [TEST_STRATEGY.md](./TEST_STRATEGY.md), [TEST_MATRIX.md](./TEST_MATRIX.md), [FAILURE_SCENARIOS.md](./FAILURE_SCENARIOS.md), [RELEASE_PLAN.md](./RELEASE_PLAN.md) |

MVP: **Linux + Windows primary**; **macOS best-effort**. Stack: Tauri 2 + Svelte/TS + Rust orchestrator; relay **sidecar**; APK bundled; **adb external**.

---

## 1. Platform support summary

| Topic | Linux | Windows | macOS |
|-------|-------|---------|-------|
| MVP support | Primary | Primary | Best-effort |
| USB tether E2E | P0 lab | P0 lab | P1/P2 |
| Code signing | Optional (repo trust) | Authenticode for GA | Notarization for “supported” |
| USB plumbing | **udev** rules | **OEM/Google USB drivers** | Xcode CLT / built-in; still flaky |
| Webview | WebKitGTK | WebView2 | WKWebView |
| Path separators | `/` | `\` (accept `/` in Rust) | `/` |
| Default adb | PATH / `ADB` | PATH / `ADB` (often `%LOCALAPPDATA%\Android\Sdk\platform-tools`) | PATH / `ADB` (Homebrew or SDK) |

---

## 2. ADB differences

| Concern | Linux | Windows | macOS |
|---------|-------|---------|-------|
| Install | distro pkg or SDK platform-tools | SDK / scoop / manual zip | brew `android-platform-tools` or SDK |
| Daemon | user session; `adb start-server` | same; may need USB driver first | same |
| Multiple devices | `-s serial` required | same | same |
| Unauthorized | RSA dialog on device | same | same |
| Missing binary | `ADB_MISSING` + apt/sdk hint | + driver vs binary distinction | + brew hint |
| Line endings / paths | POSIX | Use `std::path`; never concatenate `\` blindly | POSIX |
| Server crash | ensure_adb | ensure_adb; watch for leftover `adb.exe` | ensure_adb |

**Bundled adb:** not MVP default. If packaging option is enabled, isolate binary per-OS, still **do not** vendor full SDK; Windows still needs USB **drivers** separately. Test PATH vs configured path on all three (TEST_MATRIX P-05, P-06, P-08).

Env: `ADB`, `GNIREHTET_APK` honored identically.

---

## 3. Permissions

| | Linux | Windows | macOS |
|--|-------|---------|-------|
| App privileges | **No root** | **No Administrator** for normal tether | No root |
| USB access | udev (`plugdev` / `uaccess`) | WinUSB/Google driver; device manager | TCC may prompt for incoming net / USB accessories |
| VPN | On **device** only | On device | On device |
| Bind :31416 | unprivileged | unprivileged | unprivileged; may hit firewall prompt |
| File exec sidecar | `+x` on relay | not blocked by Mark-of-the-Web if signed | quarantine / Gatekeeper |

---

## 4. Packaging formats

| OS | Expected formats | Missing deps | Upgrade |
|----|------------------|--------------|---------|
| Linux | AppImage and/or `.deb` (optional `.rpm`) | WebKitGTK, libayatana-appindicator (distro-specific) — document | Replace package; preserve config dir (`$XDG_CONFIG_HOME`) |
| Windows | MSI/NSIS + portable zip | WebView2 Evergreen (Win10 may need bootstrap) | In-place upgrade; no dual tray icons |
| macOS | `.dmg` containing `.app` | none typical if bundled | Replace `.app`; Gatekeeper on first launch |

Installation tests: TEST_MATRIX P-01–P-04. Product does **not** auto-vendor full SDK.

---

## 5. Webview / Tauri 2 dependencies

| OS | Runtime | Test if missing |
|----|---------|-----------------|
| Linux | `webkit2gtk-4.1` (or Tauri 2 documented pkg) | App fails with install hint (F-P1) |
| Windows | Edge WebView2 | Bootstrapper or error; do not hang |
| macOS | system WKWebView | n/a on supported OS versions |

CI Linux image must match documented runtime.

---

## 6. Sidecar (relay) lifecycle

| Event | Linux | Windows | macOS |
|-------|-------|---------|-------|
| Spawn | `Command` + inherited null stdio or log files | same; job object recommended so children die with parent | same; note App Translocation if run from DMG |
| Port 31416 | `ss -ltnp` in diagnostics | `netstat`/`Get-NetTCPConnection` | `lsof -iTCP:31416` |
| External kill | SIGKILL → UI ≤3s | Task Manager kill → ≤3s | `kill -9` → ≤3s |
| Quit while running | SIGTERM tree; no orphan | must kill child; Windows orphans are common **P0 bug class** | kill child |
| Working dir | app resource / sidecar dir | same | `Contents/Resources` or Tauri sidecar path |
| Path spaces | quote | common on `Program Files` — **must test** | spaces in `/Applications` |

`start_relay` / `stop_relay` + quit hook are P0 on Linux **and** Windows (TEST_MATRIX M-18, M-20, M-23).

---

## 7. Code signing, quarantine, notarization

| OS | Mechanism | QA expectation |
|----|-----------|----------------|
| Linux | Optional GPG/repo signature | Checksums always; signing optional |
| Windows | Authenticode; SmartScreen reputation | GA: signed **or** known-limitation. Test first-run SmartScreen “More info” path |
| macOS | Hardened runtime, notarization, staple; **quarantine** (`com.apple.quarantine`) | Best-effort: document `xattr -cr` **only for lab unsigned** — not a user-facing primary workaround for GA. If unsigned, label **lab-only** |

Release gates: RELEASE_PLAN.md §7.

---

## 8. Path separators & config

- Orchestrator (Rust) uses `PathBuf` for `ADB` and `GNIREHTET_APK`.
- UI must not display broken `C:/foo\bar` mixtures; accept user paste of either separator on Windows.
- Logs: always `/` or OS-native consistently; include resolved canonical path.

---

## 9. USB: Windows drivers vs Linux udev vs macOS

### Windows (P0)

- Without Google USB Driver / OEM ADB interface, `adb devices` empty despite cable → F-P5.
- Test: clean VM + driver install doc; Win10 and Win11 (D-W1, D-W2).
- Driver ≠ adb binary. Missing driver vs `ADB_MISSING` must be distinguishable in UX.

### Linux (P0)

- udev: vendor IDs for common OEMs; `udevadm` reload documented.
- User in correct group (`plugdev`) or uaccess tags.
- Test: first-plug before and after udev rule.

### macOS (best-effort)

- Usually no extra driver; still test unauthorized + reconnect.
- USB hubs / dongles flaky — soak not a GA blocker.
- Notarization does not replace USB authorize dialog.

---

## 10. Desktop OS test expectations

| ID | Expectation |
|----|-------------|
| **Linux** | P0 happy path, udev, webview deps documented, sidecar teardown, PATH adb |
| **Windows** | P0 happy path, USB driver matrix, WebView2, no-admin, child process teardown, signing/SmartScreen |
| **macOS** | Launch + one USB tether if hardware; quarantine documented; notarization status explicit; failures don’t block Linux/Win GA |

---

## 11. Logging & diagnostics per OS

Pack always includes: versions, OS string, `adb devices -l`, adb path, port listen command output **native to OS**, logs with `timestamp`, `serial`, `stage`, `port`, `adb_exit_code`, `error_code`.

| OS | Port probe in pack |
|----|--------------------|
| Linux | `ss` or `lsof` |
| Windows | `netstat -ano` or PowerShell equivalent |
| macOS | `lsof -iTCP:<port>` |

---

## 12. Recovery SLAs (same numbers, OS-specific risk)

| Event | SLA | Highest risk OS |
|-------|-----|-----------------|
| Relay death UI | ≤3s | Windows (orphan child) |
| Device gone UI | ≤5s | All (USB) |
| Quit teardown | port free | Windows |
| Reconnect user path | ≤30s | Linux udev / Win driver |

---

## 13. Post-MVP

- macOS first-class: notarized GA + P0 USB lab  
- Simultaneous multi-tether: per-device sidecar or shared relay — **architecture decision**; expand CROSS_PLATFORM sidecar section  
- Wireless adb: firewall and mDNS differ by OS — new section  

---

## 14. QA checklist (copy into lab sheets)

### Linux
- [ ] udev applied; device authorized  
- [ ] webview deps installed **and** missing-deps negative test  
- [ ] AppImage/deb install + upgrade  
- [ ] Sidecar +x; quit leaves :31416 free  

### Windows
- [ ] Driver present vs missing  
- [ ] PATH adb vs `ADB` to `platform-tools\adb.exe`  
- [ ] WebView2 present vs missing  
- [ ] Kill `relay` in Task Manager → UI ≤3s  
- [ ] Quit while running — no orphan `relay.exe`  
- [ ] Signing / SmartScreen recorded  

### macOS
- [ ] Quarantine first launch behavior  
- [ ] Notarization staple (if claimed)  
- [ ] One USB happy path if device available  
- [ ] Label best-effort in notes  

