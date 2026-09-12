//! Host-orchestrator — thin Tauri glue over `gnirehtet-controller::SessionController`.
//!
//! Does **not** link `gnirehtet-relay` / relaylib. Relay child + ADB verbs live in the
//! controller; this module maps IPC ↔ controller and emits ERROR_UX events.
//!
//! Child stdout/stderr → `LogLine`: deferred until controller exposes
//! `start_relay_with_stdio` (Desktop will own the single pipe reader). For now we emit
//! orchestrator stage `LogLine`s only.

mod types;

pub use types::*;

use gnirehtet_controller::{
    ControllerError, RunOptions, SessionController, VpnOptions, DEFAULT_RELAY_PORT,
};
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

/// ERROR_UX fallback when `ControllerError::ux_code()` is None.
const CODE_INTERNAL: &str = "INTERNAL";
const CODE_RELAY_START_FAILED: &str = "RELAY_START_FAILED";
const CODE_STOP_FAILED: &str = "STOP_FAILED";
#[allow(dead_code)]
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
}

impl Default for OrchestratorState {
    fn default() -> Self {
        Self {
            session: Mutex::new(SessionController::from_env()),
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
    let message = message.into();
    let _ = app.emit(
        "Error",
        AppError {
            code: code.to_string(),
            message: message.clone(),
            serial: None,
        },
    );
    emit_log(
        app,
        "error",
        "orchestrator",
        format!("error_code={code} {message}"),
    );
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
pub fn ensure_adb(state: State<'_, OrchestratorState>) -> OrchResult<AdbInfo> {
    let session = lock_session(&state)?;
    match session.ensure_adb() {
        Ok(status) => Ok(AdbInfo {
            path: status.path,
            version: status.version,
            available: true,
        }),
        Err(e) => Err(OrchError::from_controller(e)),
    }
}

#[tauri::command]
pub fn list_devices(state: State<'_, OrchestratorState>) -> OrchResult<Vec<DeviceInfo>> {
    let session = lock_session(&state)?;
    match session.list_devices() {
        Ok(devices) => Ok(devices
            .into_iter()
            .map(|d| DeviceInfo {
                serial: d.serial,
                adb_state: d.state,
                model: d.model,
                product: d.product,
            })
            .collect()),
        Err(e) => Err(OrchError::from_controller(e)),
    }
}

#[tauri::command]
pub fn start_relay(
    app: AppHandle,
    state: State<'_, OrchestratorState>,
    port: Option<u16>,
) -> OrchResult<RelayStatePayload> {
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

    match session.start_relay(port) {
        Ok(relay) => {
            let listen_port = relay.port();
            let pid = relay.pid();
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
            // Child stdio → LogLine: wait for start_relay_with_stdio (single Desktop reader).
            Ok(payload)
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
            Err(OrchError::coded(code, message))
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
            Err(OrchError::coded(code, message))
        }
    }
}

#[tauri::command]
pub fn stop_relay(
    app: AppHandle,
    state: State<'_, OrchestratorState>,
) -> OrchResult<RelayStatePayload> {
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

#[tauri::command]
pub fn run_session(
    app: AppHandle,
    state: State<'_, OrchestratorState>,
    serial: Option<String>,
    dns_servers: Option<String>,
    routes: Option<String>,
    port: Option<u16>,
) -> OrchResult<RelayStatePayload> {
    let mut session = lock_session(&state)?;
    let dns = dns_servers.as_deref();
    let routes_s = routes.as_deref();
    let opts = RunOptions {
        dns_servers: dns,
        routes: routes_s,
        port,
    };
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
    match session.run(serial.as_deref(), &opts) {
        Ok(()) => {
            let payload = snapshot_relay(&session);
            emit_relay_state(&app, payload.clone());
            Ok(payload)
        }
        Err(e) => {
            let (code, message) = map_controller_err(&e);
            emit_error(&app, &code, message.clone());
            Err(OrchError::coded(code, message))
        }
    }
}

