# PHASE0_LAB_NOTES.md

| Field | Value |
|-------|-------|
| Status | Lab evidence (headless) |
| Date | 2026-09-12 |
| Branch | `dev` |
| Host | Linux x86_64 (lab box) |
| Upstream pin | Genymobile/gnirehtet **v2.5.1** / `gnirehtet-rust-linux64-v2.5.1.zip` |
| Related | [PHASE0_ACCEPTANCE.md](../qa/PHASE0_ACCEPTANCE.md), [SIDECAR.md](../../../apps/desktop/docs/SIDECAR.md), [resources/README.md](../../../resources/README.md) |

Headless ownership lab for Desktop Phase 0 sidecar / APK drop-ins. **No full GUI / display required.** Binaries are **gitignored** — drop in locally for lab/CI only.

## Sidecar + APK drop-in (lab only)

```bash
# From repo root
curl -fsSL -o /tmp/gnirehtet-rust-linux64-v2.5.1.zip \
  https://github.com/Genymobile/gnirehtet/releases/download/v2.5.1/gnirehtet-rust-linux64-v2.5.1.zip
unzip -o /tmp/gnirehtet-rust-linux64-v2.5.1.zip -d /tmp/gnirehtet-dropin
HOST=$(rustc -vV | sed -n 's/^host: //p')
cp /tmp/gnirehtet-dropin/gnirehtet-rust-linux64/gnirehtet.apk resources/gnirehtet.apk
cp /tmp/gnirehtet-dropin/gnirehtet-rust-linux64/gnirehtet \
  "apps/desktop/src-tauri/binaries/gnirehtet-${HOST}"
chmod +x "apps/desktop/src-tauri/binaries/gnirehtet-${HOST}"
```

Resolution (product):

| Asset | Order |
|-------|--------|
| APK | app setting (later) → `GNIREHTET_APK` → first existing `resources/gnirehtet.apk` candidate → `gnirehtet.apk` |
| Binary | `GNIREHTET_BIN` → `apps/desktop/src-tauri/binaries/gnirehtet-<triple>` (and nearby) → `PATH` `gnirehtet` |

Missing APK on `install` → ERROR_UX **`APK_MISSING`**. Missing binary on spawn → **`RELAY_START_FAILED`**.

## How to re-run

```bash
./scripts/phase0_lab_relay.sh
# or:
cargo run -p gnirehtet-controller --example phase0_lab_relay
```

Optional: `PHASE0_SKIP_CRASH=1` skips the mid-run kill / `poll_owned_relay` leg.

## Lab run (2026-09-12)

Command: `./scripts/phase0_lab_relay.sh`

```text
Using GNIREHTET_BIN=.../binaries/gnirehtet-x86_64-unknown-linux-gnu
Using GNIREHTET_APK=.../resources/gnirehtet.apk
date_utc_unix=1789250202
gnirehtet_bin=.../gnirehtet-x86_64-unknown-linux-gnu
bin_is_file=true
apk_path=.../resources/gnirehtet.apk
apk_is_file=true
listen_port=31416
P0-R1_owns_relay=true
P0-R1_pid=1302744
P0-R1_session_port=Some(31416)
log_line=2026-09-12 21:56:42.705 INFO Main: Starting relay server on port 31416...
P0-R1_still_owned=true
P0-R5_poll_exited=true status=signal: 9 (SIGKILL)
P0-R5_owns_after_poll=false
P0-R5_detected_within_3s=true
P0-R2_restart_port=31416
P0-R2_stop_relay=ok
P0-R2_owns_after_stop=false
P0-R2_port_free_after_stop=true
APK_MISSING_err=gnirehtet APK not found at '/no/such/gnirehtet-lab.apk' (APK_MISSING); ...
APK_MISSING_ux=Some("APK_MISSING")
gate_R1=true
gate_R2=true
gate_R5=true
gate_APK_MISSING=true
RESULT=PASS
```

### Gate mapping

| ID | Result | Evidence |
|----|--------|----------|
| **P0-R1** Spawn owned sidecar | **PASS** | `owns_relay=true`, PID recorded, listen on 31416, stock log line |
| **P0-R2** Stop owned sidecar | **PASS** | `stop_relay=ok`, `owns_after_stop=false`, `port_free_after_stop=true` |
| **P0-R5** Crash poll ≤3s (lab) | **PASS** | `kill -9` mid-run → `poll_owned_relay` → exited, ownership cleared within 3s |
| **APK path / APK_MISSING** | **PASS** | Bundled APK resolved; `install` with missing path → `APK_MISSING` |
| P0-R3 LogLine UI | **Partial** | Child stdout drained in lab; Desktop sole pump wired in orchestrator (`start_relay` / `run_session`) — full WebView strip not exercised headless |
| P0-S1 GUI launch | **Not run** | No display in this lab |
| Sharing claims | **N/A** | Not claimed (relay-only) |

## Code hooks exercised

- `SessionController::start_relay_with_stdio` / `stop_relay` / `clear_owned_relay` / `poll_owned_relay`
- `ControllerConfig` / `GNIREHTET_BIN` + sidecar drop-in search
- `AdbConfig` / `GNIREHTET_APK` + `resources/gnirehtet.apk` search; `CommandExecutionError::ApkMissing`
- Desktop: `orchestrator::paths` + `run_session` uses piped stdio + same LogLine pump as `start_relay`

## Limits

- USB / ADB install / VPN / three-layer Sharing **not** in this lab.
- Binaries remain untracked (`.gitignore`); clones must drop in or set env.
- Settings-store APK override not implemented yet (slot reserved: setting > env > bundle).
- `tauri.conf.json` `externalBin` still `[]` so `cargo check` works without the sidecar; enable when packaging.
