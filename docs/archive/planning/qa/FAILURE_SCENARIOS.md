# FAILURE_SCENARIOS.md

| Field | Value |
|-------|-------|
| Status | Draft v0.1 |
| Date | 2026-09-12 |
| Project | gnirehtet desktop GUI |
| Related | [TEST_STRATEGY.md](./TEST_STRATEGY.md), [TEST_MATRIX.md](./TEST_MATRIX.md), [RELEASE_PLAN.md](./RELEASE_PLAN.md), [CROSS_PLATFORM.md](./CROSS_PLATFORM.md) |

Each row: **detect → user-visible symptom → recovery expectation → must-log → pass/fail**. No fabricated run results.

Error taxonomy codes align with TEST_STRATEGY.md §8.

---

## 1. Recovery expectations (global)

| Class | User recovery | Time bound |
|-------|---------------|------------|
| Config / missing tool | Fix path / install adb; Retry | Immediate after fix |
| Transient device/adb | ensure_adb → list → Start | UI update ≤5s; restore ≤30s with action |
| Relay death / port | stop_relay / choose port → start_relay → reverse → START | UI ≤3s on death |
| VPN | Re-grant on device → Start | After grant |
| Quit / shutdown | Automatic teardown | Port free before process exit completes |
| APK mismatch | Install / reinstall bundled APK | Before Start allowed |

Orchestrator hooks: `ensure_adb`, `reset_tunnel`, `start`/`stop`, `start_relay`/`stop_relay`, `subscribe`.

---

## 2. Android failure modes

### F-A1 USB debugging disabled

| | |
|--|--|
| **Detect** | `adb devices` empty / offline; `list_devices` returns none while cable present (lab confirm) |
| **Symptom** | “No devices” / enable USB debugging guidance |
| **Recovery** | User enables debugging + authorize; Refresh |
| **Must-log** | `timestamp`, `serial=none`, `stage=list_devices`, `adb_exit_code`, `error_code=ADB_NO_DEVICE` (or equiv.), `msg` |
| **Pass** | No crash; actionable copy; retry works after enable |
| **Fail** | Hang; false “connected”; missing log fields |

### F-A2 Unauthorized device

| | |
|--|--|
| **Detect** | `adb devices` → `unauthorized` |
| **Symptom** | Device listed unauthorized; Start disabled; RSA prompt hint |
| **Recovery** | Accept RSA on phone; auto-refresh ≤5s or manual Refresh |
| **Must-log** | `serial`, `stage=list_devices|ensure_adb`, `error_code=ADB_UNAUTHORIZED`, `adb_exit_code` |
| **Pass** | No install/start intents sent |
| **Fail** | Proceeds to install; ignores unauthorized |

### F-A3 Authorized happy path (control — not failure)

Documented in TEST_MATRIX M-01; used as baseline for contrasts.

### F-A4 Multiple devices present

| | |
|--|--|
| **Detect** | `list_devices` length ≥ 2 authorized |
| **Symptom** | Selection required; Start blocked until serial chosen |
| **Recovery** | User selects one device (MVP: **one** active session only) |
| **Must-log** | `stage=list_devices`, device count, `error_code=ADB_MULTI` if Start attempted without selection |
| **Pass** | No auto-pick; simultaneous multi-tether **not** offered (post-MVP) |
| **Fail** | Starts on wrong/first device silently |

### F-A5 Device reconnects (cable / USB reset)

| | |
|--|--|
| **Detect** | serial vanishes then returns via subscribe/poll |
| **Symptom** | Brief disconnect banner; session ended or “reconnecting” per product policy (document chosen policy) |
| **Recovery** | If session ended: Start again; `reset_tunnel` as needed; VPN re-grant if Android requires |
| **Must-log** | `serial`, `stage` transitions, `error_code=DEVICE_GONE` then clear, `port` |
| **Pass** | No zombie reverse; user can restore ≤30s |
| **Fail** | UI stuck Connected; orphan reverse |

### F-A6 VPN permission denied

| | |
|--|--|
| **Detect** | START intent returns / events show VPN not active; traffic test fail + Android state |
| **Symptom** | `VPN_DENIED`; not Connected |
| **Recovery** | User grants VPN; Start again |
| **Must-log** | `serial`, `stage=start_intent`, `error_code=VPN_DENIED`, `port` |
| **Pass** | Honest state; no “Connected” without VPN |
| **Fail** | Green Connected while VPN off |

### F-A7 VPN permission revoked mid-session

| | |
|--|--|
| **Detect** | Traffic death + VpnService stop signals / poll; subscribe event |
| **Symptom** | Session failed; `VPN_REVOKED` |
| **Recovery** | stop/reset on desktop; user re-grants; Start |
| **Must-log** | `serial`, `stage=monitor|stop`, `error_code=VPN_REVOKED`, `timestamp` |
| **Pass** | UI ≤5s; relay stopped or clearly idle per policy |
| **Fail** | Silent blackhole with Connected badge |

### F-A8 App restart on device (force-stop / swipe away)

| | |
|--|--|
| **Detect** | VPN/app process gone; reverse may remain |
| **Symptom** | Tether down; desktop shows disconnected/degraded |
| **Recovery** | STOP cleanup + START; or user Start |
| **Must-log** | `serial`, `stage=monitor`, `error_code` (e.g. `APP_DEAD`), `port` |
| **Pass** | Desktop not stuck; Start works |
| **Fail** | Assumes still running indefinitely |

### F-A9 Device reboot

| | |
|--|--|
| **Detect** | Device offline then boot; unauthorized possible |
| **Symptom** | Session ended; device may need re-authorize |
| **Recovery** | Cold path: authorize → select → install if needed → VPN grant → Start |
| **Must-log** | `DEVICE_GONE`, later `list_devices` success; `adb_exit_code`s |
| **Pass** | Full cold start works; no stale session |
| **Fail** | Crash on reboot storm; stale Connected |

### F-A10 Android version / OEM variance

| | |
|--|--|
| **Detect** | Install or VPN dialog OEM-specific failure |
| **Symptom** | Error with OEM hint where known; diagnostics include `ro.build.version.release` if obtainable |
| **Recovery** | Doc link; manual VPN settings; still no-root |
| **Must-log** | device props if available, `stage=install|start`, `error_code` |
| **Pass** | Failure is explicit; API 21+ claim not contradicted by unhandled crash |
| **Fail** | Uncaught exception; blank UI |

---

## 3. ADB failure modes

### F-D1 adb missing

| | |
|--|--|
| **Detect** | `ensure_adb` cannot exec `ADB` or PATH binary |
| **Symptom** | Setup screen: install platform-tools / set path |
| **Recovery** | Install adb or set `ADB` / settings path; Retry |
| **Must-log** | `stage=ensure_adb`, `error_code=ADB_MISSING`, resolved path attempt, `adb_exit_code=n/a` |
| **Pass** | No half-started relay claiming success |
| **Fail** | Crash; proceeds without adb |

### F-D2 adb daemon stopped

| | |
|--|--|
| **Detect** | `adb devices` fails talking to server; ensure attempts `adb start-server` |
| **Symptom** | Transient error or auto-recover toast |
| **Recovery** | ensure_adb starts server; list refreshes |
| **Must-log** | `stage=ensure_adb`, start-server `adb_exit_code`, `error_code` if still down |
| **Pass** | Recovers without app restart |
| **Fail** | Infinite spinner; requires app reinstall |

### F-D3 adb daemon crashes mid-session

| | |
|--|--|
| **Detect** | Subsequent adb calls fail; poll ≤2s when active |
| **Symptom** | Session interrupted ≤5s UI |
| **Recovery** | ensure_adb → reset_tunnel → Start (user or guided) |
| **Must-log** | `serial`, `stage`, `adb_exit_code`, `error_code=ADB_DAEMON` |
| **Pass** | Recoverable without host reboot |
| **Fail** | Zombie UI Connected |

### F-D4 Device disappears

| | |
|--|--|
| **Detect** | serial absent from `adb devices -l` |
| **Symptom** | Disconnect ≤5s |
| **Recovery** | Replug; authorize; Start |
| **Must-log** | `error_code=DEVICE_GONE`, `serial`, `port`, `stage=monitor` |
| **Pass** | Teardown reverse/relay per policy; port not leaked if session ends |
| **Fail** | Relay left orphaned without user notice |

### F-D5 Device reconnects

| | |
|--|--|
| **Detect** | serial returns |
| **Symptom** | Device available in list |
| **Recovery** | User Start (MVP); optional future auto = post-MVP |
| **Must-log** | reconnect event, `serial`, `stage=list_devices` |
| **Pass** | Selectable; prior errors cleared |
| **Fail** | Duplicate ghost entries; can’t Start |

### F-D6 Multiple devices

| | |
|--|--|
| **Detect** | count ≥ 2 |
| **Symptom** | Picker mandatory |
| **Recovery** | Select serial |
| **Must-log** | `ADB_MULTI` on invalid Start; list snapshot in diagnostics |
| **Pass** | Intents/`adb -s` always targeted |
| **Fail** | Ambiguous adb without `-s` |

### F-D7 Unauthorized (adb-focused)

Same as F-A2; ensure no `adb reverse` / install until `device` state.

---

## 4. Relay failure modes

### F-R1 Relay process failure / crash

| | |
|--|--|
| **Detect** | Child exit; health check; subscribe |
| **Symptom** | `RELAY_DEAD` ≤**3s** |
| **Recovery** | stop cleanup → start_relay → reverse → START |
| **Must-log** | `stage=relay_monitor`, `port`, exit status, `error_code=RELAY_DEAD`, `serial` |
| **Pass** | SLA met; no Connected lie |
| **Fail** | >3s stale Connected |

### F-R2 Port already occupied

| | |
|--|--|
| **Detect** | bind fail on 31416 (or configured) |
| **Symptom** | `PORT_IN_USE`; suggest free port / kill conflict / settings |
| **Recovery** | Change port or free port; start_relay |
| **Must-log** | `stage=start_relay`, `port`, `error_code=PORT_IN_USE` |
| **Pass** | No pretend success; diagnostics show listen state |
| **Fail** | Attaches to wrong process; silent fail |

### F-R3 Network failure (host routing / iface down)

| | |
|--|--|
| **Detect** | Relay up but forwarding fails; OS net events if available |
| **Symptom** | Device online to VPN but no upstream; warning |
| **Recovery** | Restore host network; optional reset_tunnel |
| **Must-log** | `error_code=NETWORK_FAIL`, `port`, `serial`, `stage=monitor` |
| **Pass** | User sees degraded ≠ Connected-healthy |
| **Fail** | No indication while apps time out only on phone |

### F-R4 Shutdown (orderly stop / app quit)

| | |
|--|--|
| **Detect** | User Stop or app quit hook |
| **Symptom** | STOP intent; reverse removed; relay exited |
| **Recovery** | n/a — clean idle |
| **Must-log** | `stage=stop|shutdown`, `port`, `serial`, success flags |
| **Pass** | Port free; no orphan relay; quit-while-running safe |
| **Fail** | Listener remains; zombie child |

### F-R5 Reconnect after relay/tunnel break

| | |
|--|--|
| **Detect** | Prior RELAY_DEAD / DEVICE_GONE cleared |
| **Symptom** | Guided “Resume” / Start enabled |
| **Recovery** | reset_tunnel + start_relay + reverse + START ≤30s user path |
| **Must-log** | full stage sequence, `port`, `serial` |
| **Pass** | Traffic restored |
| **Fail** | Double reverse; port conflict on second start |

### F-R6 Malformed traffic

| | |
|--|--|
| **Detect** | Relay logs parse errors; process remains up |
| **Symptom** | Optional advanced log; session usually continues |
| **Recovery** | None required if stable; Stop/Start if wedged |
| **Must-log** | `error_code=MALFORMED`, `port`, count/rate, `serial` if known |
| **Pass** | No host crash; relay survives fuzz sample |
| **Fail** | Relay panic takes down GUI process |

---

## 5. Desktop / packaging-related failures (brief)

| ID | Mode | Detect | Symptom | Recovery | Must-log | Pass |
|----|------|--------|---------|----------|----------|------|
| F-P1 | Missing webview deps (Linux) | Launch fail | Dep error message | Install pkgs per CROSS_PLATFORM | launch `error_code` | Documented pkgs |
| F-P2 | Wrong/missing bundled APK | install stage | `APK_MISSING` / mismatch | Fix bundle / `GNIREHTET_APK` | path, version | Hard fail before Start |
| F-P3 | APK version mismatch | version compare | `APK_MISMATCH` | Reinstall bundled | expected vs found | Block or forced repair |
| F-P4 | Unsigned / quarantine (macOS) | Gatekeeper | OS block | xattr / notarize path | OS error text | Documented best-effort |
| F-P5 | Windows USB driver missing | no device | No serials | Install Google/OEM USB driver | `ADB_NO_DEVICE` | Doc’d in UI |

---

## 6. Logging requirements (summary)

Every failure path above **must** emit:

`timestamp`, `serial` (or `none`), `stage`, `port` (or `n/a`), `adb_exit_code` (or `n/a`), `error_code`, `msg`.

Prefer also: `session_id`, relay PID, APK version, adb path.

---

## 7. Diagnostics requirements

On failure (and on demand), diagnostics pack includes:

1. App / orchestrator / relay / APK version strings  
2. Host OS / arch  
3. Resolved adb path + `adb version`  
4. `adb devices -l`  
5. Relay port listen state  
6. Recent logs (required fields present)  
7. Last error_code list  
8. Active session_id / selected serial  

**Pass:** Pack sufficient to file upstream-quality bug without repro guessing.  
**Fail:** Missing versions, devices list, or port state for relay/port bugs.

---

## 8. Pass/fail quick index (P0)

| Scenario | error_code | UI SLA | P0? |
|----------|------------|--------|-----|
| adb missing | ADB_MISSING | immediate | Yes |
| unauthorized | ADB_UNAUTHORIZED | on list | Yes |
| multi device no selection | ADB_MULTI | on Start | Yes |
| port occupied | PORT_IN_USE | on start_relay | Yes |
| relay killed | RELAY_DEAD | ≤3s | Yes |
| quit while running | (clean shutdown) | before exit | Yes |
| APK mismatch | APK_MISMATCH | before Start | Yes |
| VPN denied/revoked | VPN_DENIED / VPN_REVOKED | ≤5s revoke | Yes |

Cross-check execution cells: TEST_MATRIX.md. Release blockers: RELEASE_PLAN.md.

