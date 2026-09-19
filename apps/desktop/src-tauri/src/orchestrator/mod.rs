//! Host-orchestrator — thin Tauri glue over `gnirehtet-controller::SessionController`.
//!
//! Does **not** link `gnirehtet-relay` / relaylib. Relay child + ADB verbs live in the
//! controller; this module maps IPC ↔ controller and emits ERROR_UX events.
//!
//! Child stdout/stderr → `LogLine`: Desktop is the **sole** reader of pipes from
//! `start_relay_with_stdio` (take under mutex, pump outside). Crash watcher polls
//! `poll_owned_relay` every ≤500ms → `RelayState` exited + `RELAY_CRASHED` ≤3s.

mod paths;
mod types;

pub use paths::{controller_config_from_env, resolved_paths};
pub use types::*;

use gnirehtet_controller::{
    ControllerError, RelayStdio, SessionController, VpnOptions,
    DEFAULT_RELAY_PORT,
};
use serde::Serialize;
use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

/// ERROR_UX fallback when `ControllerError::ux_code()` is None.
const CODE_INTERNAL: &str = "INTERNAL";
const CODE_RELAY_START_FAILED: &str = "RELAY_START_FAILED";
const CODE_STOP_FAILED: &str = "STOP_FAILED";
const CODE_RELAY_CRASHED: &str = "RELAY_CRASHED";

#[derive(Debug, thiserror::Error)]
pub enum OrchError {
    #[error("{message}")]
    Message { code: String, message: String },
}

impl OrchError {
    fn coded(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Message {
            code: code.into(),
            message: message.into(),
        }
    }

    fn from_controller(e: ControllerError) -> Self {
        let (code, message) = map_controller_err(&e);
        Self::coded(code, message)
    }
}

impl Serialize for OrchError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            OrchError::Message { code, message } => {
                #[derive(Serialize)]
                #[serde(rename_all = "camelCase")]
                struct ErrBody<'a> {
                    code: &'a str,
                    message: &'a str,
                }
                ErrBody { code, message }.serialize(serializer)
            }
        }
    }
}

pub type OrchResult<T> = Result<T, OrchError>;

pub struct OrchestratorState {
    pub session: Mutex<SessionController>,
    /// Bumped on each successful start and on intentional stop/teardown so crash
    /// watchers ignore expected exits (P0-R5).
    relay_epoch: AtomicU64,
}

impl Default for OrchestratorState {
    fn default() -> Self {
        let cfg = controller_config_from_env();
        let paths = resolved_paths(&cfg);
        // Cold-start diagnostics (no GUI required): help lab / triage see drop-ins.
        eprintln!(
            "[orchestrator] paths: bin={} (present={}) apk={} (present={})",
            paths.gnirehtet_bin.display(),
            paths.bin_present,
            paths.apk_path.display(),
            paths.apk_present
        );
        Self {
            session: Mutex::new(SessionController::new(cfg)),
            relay_epoch: AtomicU64::new(0),
        }
    }
}

fn map_controller_err(e: &ControllerError) -> (String, String) {
    let code = e.ux_code().unwrap_or(CODE_INTERNAL).to_string();
    (code, e.to_string())
}

fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn emit_log(app: &AppHandle, level: &str, source: &str, message: impl Into<String>) {
    let _ = app.emit(
        "LogLine",
        LogLine {
            timestamp_ms: now_ms(),
            level: level.to_string(),
            source: source.to_string(),
            message: message.into(),
        },
    );
}

fn emit_relay_state(app: &AppHandle, payload: RelayStatePayload) {
    let _ = app.emit("RelayState", payload);
}

fn emit_error(app: &AppHandle, code: &str, message: impl Into<String>) {
    emit_error_serial(app, code, message, None);
}

fn emit_error_serial(
    app: &AppHandle,
    code: &str,
    message: impl Into<String>,
    serial: Option<String>,
) {
    let message = message.into();
    let _ = app.emit(
        "Error",
        AppError {
            code: code.to_string(),
            message: message.clone(),
            serial,
        },
    );
    emit_log(
        app,
        "error",
        "orchestrator",
        format!("error_code={code} {message}"),
    );
}

fn emit_device_changed(app: &AppHandle, devices: Vec<DeviceInfo>) {
    let _ = app.emit("DeviceChanged", DeviceChangedPayload { devices });
}

fn map_devices(devices: Vec<gnirehtet_controller::AdbDevice>) -> Vec<DeviceInfo> {
    devices
        .into_iter()
        .map(|d| DeviceInfo {
            serial: d.serial,
            adb_state: d.state,
            model: d.model,
            product: d.product,
        })
        .collect()
}

/// Invalidate in-flight crash watchers before killing the owned child.
fn bump_relay_epoch(state: &OrchestratorState) {
    state.relay_epoch.fetch_add(1, Ordering::SeqCst);
}

/// Sole Desktop LogLine pump + crash watcher for a session-owned piped relay.
///
/// Readers run **outside** `Mutex<SessionController>`. No second spawn / second reader.
fn spawn_relay_stdio_pump(
    app: AppHandle,
    stdio: RelayStdio,
    port: u16,
    pid: u32,
    epoch: u64,
) {
    let RelayStdio { stdout, stderr } = stdio;

    let app_out = app.clone();
    let t_out = thread::spawn(move || {
        for line in BufReader::new(stdout).lines().flatten() {
            emit_log(&app_out, "info", "relay", line);
        }
    });

    let app_err = app.clone();
    let t_err = thread::spawn(move || {
        for line in BufReader::new(stderr).lines().flatten() {
            emit_log(&app_err, "warn", "relay", line);
        }
    });

    thread::spawn(move || {
        // P0-R5: Desktop sole poller — poll_owned_relay every 500ms (sleep outside lock).
        // Pipe EOF join is a backup signal if poll races with teardown.
        loop {
            if app
                .try_state::<OrchestratorState>()
                .map(|s| s.relay_epoch.load(Ordering::SeqCst) != epoch)
                .unwrap_or(true)
            {
                let _ = t_out.join();
                let _ = t_err.join();
                return;
            }

            let exited_via_poll = poll_owned_relay_exit(&app, pid);
            let pipes_done = t_out.is_finished() && t_err.is_finished();
            if exited_via_poll || pipes_done {
                break;
            }
            thread::sleep(Duration::from_millis(500));
        }

        let _ = t_out.join();
        let _ = t_err.join();

        let Some(state) = app.try_state::<OrchestratorState>() else {
            return;
        };
        if state.relay_epoch.load(Ordering::SeqCst) != epoch {
            return;
        }

        // Confirm unexpected death for this epoch (intentional stop bumps epoch first).
        let listen_port = {
            let Ok(mut session) = state.session.lock() else {
                return;
            };
            let listen = session.session_port().unwrap_or(port);
            if session.owned_relay_pid() == Some(pid) {
                // Clear zombie ownership (poll_owned_relay would have done this).
                session.clear_owned_relay();
            } else if session.owns_relay() {
                // Superseded by a different owned child.
                return;
            }
            // else: already cleared by poll_owned_relay — still emit crash for this epoch.
            listen
        };

        if state.relay_epoch.load(Ordering::SeqCst) != epoch {
            return;
        }

        let msg = format!("Relay process exited unexpectedly (pid={pid})");
        emit_error(&app, CODE_RELAY_CRASHED, msg.clone());
        emit_relay_state(
            &app,
            RelayStatePayload {
                state: "relay_exited".into(),
                port: Some(listen_port),
                pid: None,
                owned_by_session: false,
                message: Some("RELAY_CRASHED".into()),
            },
        );
        emit_log(
            &app,
            "error",
            "orchestrator",
            format!("stage=relay_crash port={listen_port} pid={pid} error_code=RELAY_CRASHED"),
        );
    });
}

/// Briefly lock and call `SessionController::poll_owned_relay` (no sleep under lock).
///
/// Returns true if the watched pid exited; ownership is already cleared by the controller.
fn poll_owned_relay_exit(app: &AppHandle, watched_pid: u32) -> bool {
    let Some(state) = app.try_state::<OrchestratorState>() else {
        return false;
    };
    let Ok(mut session) = state.session.lock() else {
        return false;
    };
    // Match watched pid before poll; stop/teardown may have cleared ownership.
    match session.owned_relay_pid() {
        Some(pid) if pid == watched_pid => {}
        _ => return false,
    }
    // Brief lock only — no sleep under mutex. Some(status) => exited; ownership cleared.
    matches!(session.poll_owned_relay(), Ok(Some(_)))
}

fn lock_session<'a>(
    state: &'a OrchestratorState,
) -> OrchResult<std::sync::MutexGuard<'a, SessionController>> {
    state
        .session
        .lock()
        .map_err(|_| OrchError::coded(CODE_INTERNAL, "session lock poisoned"))
}

fn snapshot_relay(session: &SessionController) -> RelayStatePayload {
    if session.owns_relay() {
        RelayStatePayload {
            state: "relay_running".into(),
            port: session.session_port().or(Some(DEFAULT_RELAY_PORT)),
            pid: session.owned_relay_pid(),
            owned_by_session: true,
            message: None,
        }
    } else {
        RelayStatePayload {
            state: "relay_stopped".into(),
            port: session.session_port().or(Some(DEFAULT_RELAY_PORT)),
            pid: None,
            owned_by_session: false,
            message: None,
        }
    }
}

/// P0-Q1: tear down session-owned relay only. Never kills foreign processes or adb server.
pub fn teardown_owned_relay(app: &AppHandle, state: &OrchestratorState, stage: &str) {
    bump_relay_epoch(state);
    let Ok(mut session) = state.session.lock() else {
        return;
    };
    let port = session.session_port().unwrap_or(DEFAULT_RELAY_PORT);
    let owned = session.owns_relay();
    let pid = session.owned_relay_pid();
    if !owned {
        emit_log(
            app,
            "info",
            "orchestrator",
            format!("stage={stage} port={port} owned=false skip_kill"),
        );
        return;
    }
    emit_log(
        app,
        "info",
        "orchestrator",
        format!(
            "stage={stage} port={port} pid={} stopping_owned_relay",
            pid.map(|p| p.to_string()).unwrap_or_else(|| "?".into())
        ),
    );
    session.clear_owned_relay();
    emit_log(
        app,
        "info",
        "orchestrator",
        format!("stage={stage} port={port} success=true"),
    );
    emit_relay_state(
        app,
        RelayStatePayload {
            state: "relay_stopped".into(),
            port: Some(port),
            pid: None,
            owned_by_session: false,
            message: Some(format!("Teardown complete ({stage})")),
        },
    );
}

#[tauri::command]
pub fn ensure_adb(
    app: AppHandle,
    state: State<'_, OrchestratorState>,
) -> OrchResult<AdbInfo> {
    let session = lock_session(&state)?;
    match session.ensure_adb() {
        Ok(status) => {
            emit_log(
                &app,
                "info",
                "orchestrator",
                format!("stage=ensure_adb path={} version={}", status.path, status.version),
            );
            Ok(AdbInfo {
                path: status.path,
                version: status.version,
                available: true,
            })
        }
        Err(e) => {
            let (code, message) = map_controller_err(&e);
            // Surface ADB_MISSING / ADB_PATH_INVALID (and any other mapped code) via Error.
            emit_error(&app, &code, message.clone());
            Err(OrchError::coded(code, message))
        }
    }
}

#[tauri::command]
pub fn list_devices(
    app: AppHandle,
    state: State<'_, OrchestratorState>,
) -> OrchResult<Vec<DeviceInfo>> {
    let session = lock_session(&state)?;
    match session.list_devices() {
        Ok(devices) => {
            let mapped = map_devices(devices);
            emit_device_changed(&app, mapped.clone());
            Ok(mapped)
        }
        Err(e) => {
            let (code, message) = map_controller_err(&e);
            emit_error(&app, &code, message.clone());
            // Clear the UI list on discovery failure (e.g. ADB_MISSING mid-session).
            emit_device_changed(&app, Vec::new());
            Err(OrchError::coded(code, message))
        }
    }
}

#[tauri::command]
pub fn start_relay(
    app: AppHandle,
    state: State<'_, OrchestratorState>,
    port: Option<u16>,
) -> OrchResult<RelayStatePayload> {
    // Take RelayStdio under the mutex, then pump **outside** the lock (sole reader).
    let (stdio, listen_port, pid, epoch) = {
        let mut session = lock_session(&state)?;
        let listen_hint = port
            .or_else(|| session.session_port())
            .unwrap_or(DEFAULT_RELAY_PORT);

        emit_log(
            &app,
            "info",
            "orchestrator",
            format!("stage=start_relay port={listen_hint} spawning"),
        );
        emit_relay_state(
            &app,
            RelayStatePayload {
                state: "relay_starting".into(),
                port: Some(listen_hint),
                pid: None,
                owned_by_session: false,
                message: Some("Spawning gnirehtet relay".into()),
            },
        );

        match session.start_relay_with_stdio(port) {
            Ok(stdio) => {
                let listen_port = session.session_port().unwrap_or(listen_hint);
                let pid = session.owned_relay_pid().ok_or_else(|| {
                    OrchError::coded(CODE_RELAY_START_FAILED, "relay started without pid")
                })?;
                let epoch = state.relay_epoch.fetch_add(1, Ordering::SeqCst) + 1;
                (stdio, listen_port, pid, epoch)
            }
            Err(e @ ControllerError::PortInUse { .. }) => {
                let (code, message) = map_controller_err(&e);
                debug_assert!(!session.owns_relay());
                emit_log(
                    &app,
                    "error",
                    "orchestrator",
                    format!("stage=start_relay port={listen_hint} error_code={code} {message}"),
                );
                emit_error(&app, &code, message.clone());
                emit_relay_state(
                    &app,
                    RelayStatePayload {
                        state: "relay_error".into(),
                        port: Some(listen_hint),
                        pid: None,
                        owned_by_session: false,
                        message: Some(code.clone()),
                    },
                );
                return Err(OrchError::coded(code, message));
            }
            Err(e) => {
                let (code, message) = map_controller_err(&e);
                let code = if code == CODE_INTERNAL {
                    CODE_RELAY_START_FAILED.to_string()
                } else {
                    code
                };
                emit_error(&app, &code, message.clone());
                emit_relay_state(
                    &app,
                    RelayStatePayload {
                        state: "relay_error".into(),
                        port: Some(listen_hint),
                        pid: None,
                        owned_by_session: false,
                        message: Some(message.clone()),
                    },
                );
                return Err(OrchError::coded(code, message));
            }
        }
    }; // MutexGuard dropped — do not read pipes while holding the lock.

    let payload = RelayStatePayload {
        state: "relay_running".into(),
        port: Some(listen_port),
        pid: Some(pid),
        owned_by_session: true,
        message: Some("Relay process started (session-owned)".into()),
    };
    emit_log(
        &app,
        "info",
        "orchestrator",
        format!("stage=start_relay port={listen_port} pid={pid} owned=true"),
    );
    emit_relay_state(&app, payload.clone());

    spawn_relay_stdio_pump(app, stdio, listen_port, pid, epoch);
    Ok(payload)
}

#[tauri::command]
pub fn stop_relay(
    app: AppHandle,
    state: State<'_, OrchestratorState>,
) -> OrchResult<RelayStatePayload> {
    // Invalidate crash watcher before killing so pipe EOF is not RELAY_CRASHED.
    bump_relay_epoch(&state);
    let mut session = lock_session(&state)?;
    let port = session.session_port().unwrap_or(DEFAULT_RELAY_PORT);

    if !session.owns_relay() {
        // P0-R4: no foreign kill; NoOwnedRelay is success for stop.
        emit_log(
            &app,
            "info",
            "orchestrator",
            format!("stage=stop port={port} owned=false skip_kill"),
        );
        return Ok(RelayStatePayload {
            state: "relay_stopped".into(),
            port: Some(port),
            pid: None,
            owned_by_session: false,
            message: Some(
                "No session-owned relay; left any foreign listener untouched".into(),
            ),
        });
    }

    let pid = session.owned_relay_pid();
    emit_log(
        &app,
        "info",
        "orchestrator",
        format!(
            "stage=stop port={port} pid={} stopping_owned_relay",
            pid.map(|p| p.to_string()).unwrap_or_else(|| "?".into())
        ),
    );

    match session.stop_relay() {
        Ok(()) => {
            emit_log(
                &app,
                "info",
                "orchestrator",
                format!("stage=stop port={port} success=true"),
            );
            let payload = RelayStatePayload {
                state: "relay_stopped".into(),
                port: Some(port),
                pid: None,
                owned_by_session: false,
                message: Some("Relay stopped".into()),
            };
            emit_relay_state(&app, payload.clone());
            Ok(payload)
        }
        Err(ControllerError::NoOwnedRelay) => {
            let payload = RelayStatePayload {
                state: "relay_stopped".into(),
                port: Some(port),
                pid: None,
                owned_by_session: false,
                message: Some("Relay already stopped".into()),
            };
            emit_relay_state(&app, payload.clone());
            Ok(payload)
        }
        Err(e) => {
            let (code, message) = map_controller_err(&e);
            let code = if code == CODE_INTERNAL {
                CODE_STOP_FAILED.to_string()
            } else {
                code
            };
            emit_error(&app, &code, &message);
            Err(OrchError::coded(code, message))
        }
    }
}

#[tauri::command]
pub fn get_relay_state(state: State<'_, OrchestratorState>) -> OrchResult<RelayStatePayload> {
    let session = lock_session(&state)?;
    Ok(snapshot_relay(&session))
}

// --- Optional thin ADB / tunnel verbs (easy IPC surface) ---

#[tauri::command]
pub fn install(
    app: AppHandle,
    state: State<'_, OrchestratorState>,
    serial: Option<String>,
) -> OrchResult<()> {
    let session = lock_session(&state)?;
    emit_log(
        &app,
        "info",
        "orchestrator",
        format!(
            "stage=install serial={}",
            serial.as_deref().unwrap_or("(default)")
        ),
    );
    session
        .install(serial.as_deref())
        .map_err(OrchError::from_controller)
}

#[tauri::command]
pub fn start_client(
    app: AppHandle,
    state: State<'_, OrchestratorState>,
    serial: Option<String>,
    dns_servers: Option<String>,
    routes: Option<String>,
    port: Option<u16>,
) -> OrchResult<()> {
    let mut session = lock_session(&state)?;
    let listen = port
        .or_else(|| session.session_port())
        .unwrap_or(DEFAULT_RELAY_PORT);
    let dns = dns_servers.as_deref();
    let routes_s = routes.as_deref();
    let opts = VpnOptions {
        dns_servers: dns,
        routes: routes_s,
        port: listen,
    };
    emit_log(
        &app,
        "info",
        "orchestrator",
        format!(
            "stage=start serial={} port={listen}",
            serial.as_deref().unwrap_or("(default)")
        ),
    );
    session
        .start(serial.as_deref(), &opts)
        .map_err(OrchError::from_controller)
}

#[tauri::command]
pub fn stop_client(
    app: AppHandle,
    state: State<'_, OrchestratorState>,
    serial: Option<String>,
) -> OrchResult<()> {
    let session = lock_session(&state)?;
    emit_log(
        &app,
        "info",
        "orchestrator",
        format!(
            "stage=stop_client serial={}",
            serial.as_deref().unwrap_or("(default)")
        ),
    );
    session
        .stop(serial.as_deref())
        .map_err(OrchError::from_controller)
}

#[tauri::command]
pub fn reset_tunnel(
    app: AppHandle,
    state: State<'_, OrchestratorState>,
    serial: Option<String>,
    port: Option<u16>,
) -> OrchResult<()> {
    let mut session = lock_session(&state)?;
    emit_log(
        &app,
        "info",
        "orchestrator",
        format!(
            "stage=reset_tunnel serial={} port={:?}",
            serial.as_deref().unwrap_or("(default)"),
            port
        ),
    );
    session
        .reset_tunnel(serial.as_deref(), port)
        .map_err(OrchError::from_controller)
}

/// One-click ≈ upstream `run`: owned relay with stdio LogLine pump, then ADB start.
///
/// Uses `start_relay_with_stdio` (same pump as `start_relay`) so GUI run is not
/// inherit-only. ADB install path surfaces `APK_MISSING` when the APK file is absent.
#[tauri::command]
pub fn run_session(
    app: AppHandle,
    state: State<'_, OrchestratorState>,
    serial: Option<String>,
    dns_servers: Option<String>,
    routes: Option<String>,
    port: Option<u16>,
) -> OrchResult<RelayStatePayload> {
    let dns = dns_servers.as_deref();
    let routes_s = routes.as_deref();

    emit_log(
        &app,
        "info",
        "orchestrator",
        format!(
            "stage=run serial={} port={:?}",
            serial.as_deref().unwrap_or("(default)"),
            port
        ),
    );

    // Start (or reuse) session-owned relay with piped stdio → sole LogLine pump.
    let maybe_pump: Option<(RelayStdio, u16, u32, u64)> = {
        let mut session = lock_session(&state)?;
        if session.owns_relay() {
            let listen = session.session_port().unwrap_or(DEFAULT_RELAY_PORT);
            if let Some(p) = port {
                if session.session_port() != Some(p) {
                    return Err(OrchError::from_controller(
                        ControllerError::RelayAlreadyRunning { port: listen },
                    ));
                }
            }
            None
        } else {
            let listen_hint = port
                .or_else(|| session.session_port())
                .unwrap_or(DEFAULT_RELAY_PORT);
            emit_relay_state(
                &app,
                RelayStatePayload {
                    state: "relay_starting".into(),
                    port: Some(listen_hint),
                    pid: None,
                    owned_by_session: false,
                    message: Some("Spawning gnirehtet relay (run)".into()),
                },
            );
            match session.start_relay_with_stdio(port) {
                Ok(stdio) => {
                    let listen_port = session.session_port().unwrap_or(listen_hint);
                    let pid = session.owned_relay_pid().ok_or_else(|| {
                        OrchError::coded(CODE_RELAY_START_FAILED, "relay started without pid")
                    })?;
                    let epoch = state.relay_epoch.fetch_add(1, Ordering::SeqCst) + 1;
                    Some((stdio, listen_port, pid, epoch))
                }
                Err(e) => {
                    let (code, message) = map_controller_err(&e);
                    emit_error(&app, &code, message.clone());
                    return Err(OrchError::coded(code, message));
                }
            }
        }
    };

    if let Some((stdio, listen_port, pid, epoch)) = maybe_pump {
        emit_log(
            &app,
            "info",
            "orchestrator",
            format!("stage=run port={listen_port} pid={pid} owned=true"),
        );
        emit_relay_state(
            &app,
            RelayStatePayload {
                state: "relay_running".into(),
                port: Some(listen_port),
                pid: Some(pid),
                owned_by_session: true,
                message: Some("Relay process started (session-owned)".into()),
            },
        );
        spawn_relay_stdio_pump(app.clone(), stdio, listen_port, pid, epoch);
    }

    // ADB install / tunnel / START (APK_MISSING if helper file absent when install needed).
    {
        let mut session = lock_session(&state)?;
        let listen = port
            .or_else(|| session.session_port())
            .unwrap_or(DEFAULT_RELAY_PORT);
        let vpn = VpnOptions {
            dns_servers: dns,
            routes: routes_s,
            port: listen,
        };
        if let Err(e) = session.start(serial.as_deref(), &vpn) {
            let (code, message) = map_controller_err(&e);
            emit_error(&app, &code, message.clone());
            return Err(OrchError::coded(code, message));
        }
        let payload = snapshot_relay(&session);
        emit_relay_state(&app, payload.clone());
        Ok(payload)
    }
}

