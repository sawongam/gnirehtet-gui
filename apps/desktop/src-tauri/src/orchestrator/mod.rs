//! Host-orchestrator stubs (Phase 0).
//!
//! Spawns the stock external `gnirehtet` binary for relay — never links relaylib.
//! Direct `adb` is used only for discovery/health (`ensure_adb`, `list_devices`).
//! Error codes follow docs/architecture/qa/ERROR_CODE_MAP.md (ERROR_UX canonical).

mod types;

pub use types::*;

use serde::Serialize;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

const DEFAULT_RELAY_PORT: u16 = 31416;

/// ERROR_UX / ERROR_CODE_MAP canonical codes used by Phase 0 stubs.
mod codes {
    pub const ADB_MISSING: &str = "ADB_MISSING";
    pub const ADB_PATH_INVALID: &str = "ADB_PATH_INVALID";
    pub const PORT_IN_USE: &str = "PORT_IN_USE";
    pub const RELAY_START_FAILED: &str = "RELAY_START_FAILED";
    pub const RELAY_CRASHED: &str = "RELAY_CRASHED";
    pub const STOP_FAILED: &str = "STOP_FAILED";
    pub const APK_MISSING: &str = "APK_MISSING"; // reserved for install path; logged when unresolved
}

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

pub struct RelaySession {
    child: Option<Child>,
    port: u16,
    owned_by_session: bool,
}

impl Default for RelaySession {
    fn default() -> Self {
        Self {
            child: None,
            port: DEFAULT_RELAY_PORT,
            owned_by_session: false,
        }
    }
}

pub struct OrchestratorState {
    pub relay: Mutex<RelaySession>,
}

impl Default for OrchestratorState {
    fn default() -> Self {
        Self {
            relay: Mutex::new(RelaySession::default()),
        }
    }
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

fn resolve_adb_path() -> PathBuf {
    if let Ok(p) = std::env::var("ADB") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    PathBuf::from("adb")
}

fn which_exists(cmd: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {cmd} >/dev/null 2>&1"))
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn which_path(cmd: &Path) -> Option<PathBuf> {
    if cmd.is_absolute() && cmd.is_file() {
        return Some(cmd.to_path_buf());
    }
    let name = cmd.to_string_lossy();
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {name}"))
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let p = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if p.is_empty() {
        None
    } else {
        Some(PathBuf::from(p))
    }
}

fn current_host_triple_guess() -> String {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        return "x86_64-unknown-linux-gnu".into();
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        return "aarch64-apple-darwin".into();
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        return "x86_64-apple-darwin".into();
    }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        return "x86_64-pc-windows-msvc".into();
    }
    #[allow(unreachable_code)]
    "unknown".into()
}

/// Resolve stock `gnirehtet` binary: GNIREHTET_BIN → sidecar candidates → PATH.
fn resolve_gnirehtet_bin(app: &AppHandle) -> OrchResult<PathBuf> {
    if let Ok(p) = std::env::var("GNIREHTET_BIN") {
        let pb = PathBuf::from(&p);
        if pb.is_file() {
            return Ok(pb);
        }
        return Err(OrchError::coded(
            codes::RELAY_START_FAILED,
            format!("GNIREHTET_BIN set but not a file: {p}"),
        ));
    }

    if let Ok(res_dir) = app.path().resource_dir() {
        for name in ["gnirehtet", "gnirehtet.exe"] {
            let candidate = res_dir.join(name);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }

    let triple = std::env::var("TAURI_ENV_TARGET_TRIPLE")
        .or_else(|_| std::env::var("TARGET"))
        .unwrap_or_else(|_| current_host_triple_guess());
    let exe = if cfg!(windows) { ".exe" } else { "" };
    let candidates = [
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("binaries")
            .join(format!("gnirehtet-{triple}{exe}")),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("binaries")
            .join(format!("gnirehtet{exe}")),
    ];
    for c in candidates {
        if c.is_file() {
            return Ok(c);
        }
    }

    if which_exists("gnirehtet") {
        return Ok(PathBuf::from("gnirehtet"));
    }

    Err(OrchError::coded(
        codes::RELAY_START_FAILED,
        "gnirehtet binary not found. Drop a stock upstream binary into \
         apps/desktop/src-tauri/binaries/ (see docs/SIDECAR.md), set GNIREHTET_BIN, \
         or install gnirehtet on PATH.",
    ))
}

/// Setting (none yet) > GNIREHTET_APK > bundled resources/.
pub fn resolve_apk_path(app: &AppHandle) -> Option<PathBuf> {
    if let Ok(p) = std::env::var("GNIREHTET_APK") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }
    if let Ok(res_dir) = app.path().resource_dir() {
        let bundled = res_dir.join("gnirehtet.apk");
        if bundled.is_file() {
            return Some(bundled);
        }
    }
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../resources/gnirehtet.apk");
    if repo.is_file() {
        return Some(repo);
    }
    None
}

/// Probe whether we can bind 127.0.0.1:port before claiming relay ownership.
fn port_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

fn kill_owned_child(child: &mut Child) -> Result<(), String> {
    if let Err(e) = child.kill() {
        // Already exited is fine
        if child.try_wait().ok().flatten().is_some() {
            return Ok(());
        }
        return Err(format!("kill failed: {e}"));
    }
    let _ = child.wait();
    Ok(())
}

/// Tear down session-owned relay only. Never kills foreign processes or adb server.
pub fn teardown_owned_relay(app: &AppHandle, state: &OrchestratorState, stage: &str) {
    let mut relay = match state.relay.lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    let port = relay.port;
    if !relay.owned_by_session {
        emit_log(
            app,
            "info",
            "orchestrator",
            format!("stage={stage} port={port} owned=false skip_kill"),
        );
        return;
    }
    if let Some(mut child) = relay.child.take() {
        let pid = child.id();
        emit_log(
            app,
            "info",
            "orchestrator",
            format!("stage={stage} port={port} pid={pid} stopping_owned_relay"),
        );
        match kill_owned_child(&mut child) {
            Ok(()) => emit_log(
                app,
                "info",
                "orchestrator",
                format!("stage={stage} port={port} success=true"),
            ),
            Err(e) => {
                emit_error(
                    app,
                    codes::STOP_FAILED,
                    format!("stage={stage} port={port} success=false {e}"),
                );
            }
        }
    }
    relay.owned_by_session = false;
    emit_relay_state(
        app,
        RelayStatePayload {
            state: "relay_stopped".into(),
            port: Some(port),
            pid: None,
            owned_by_session: false,
            message: Some(format!("Teardown complete ({stage})").into()),
        },
    );
}

#[tauri::command]
pub fn ensure_adb() -> OrchResult<AdbInfo> {
    let path = resolve_adb_path();
    if path.is_absolute() && !path.is_file() {
        return Err(OrchError::coded(
            codes::ADB_PATH_INVALID,
            format!("Configured ADB path is not a file: {}", path.display()),
        ));
    }
    let output = Command::new(&path)
        .arg("version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| {
            OrchError::coded(
                codes::ADB_MISSING,
                format!(
                    "Failed to run adb at '{}': {e}. Install platform-tools or set ADB / Settings.",
                    path.display()
                ),
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(OrchError::coded(
            codes::ADB_MISSING,
            format!("adb version failed: {stderr}"),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let version_line = stdout.lines().next().unwrap_or("adb").to_string();
    let resolved = which_path(&path).unwrap_or(path);

    Ok(AdbInfo {
        path: resolved.display().to_string(),
        version: version_line,
        available: true,
    })
}

#[tauri::command]
pub fn list_devices() -> OrchResult<Vec<DeviceInfo>> {
    let path = resolve_adb_path();
    let output = Command::new(&path)
        .args(["devices", "-l"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| {
            OrchError::coded(codes::ADB_MISSING, format!("Failed to run adb devices: {e}"))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(OrchError::coded(
            codes::ADB_MISSING,
            format!("adb devices failed: {stderr}"),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();
    for line in stdout.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(serial) = parts.next() else {
            continue;
        };
        let state = parts.next().unwrap_or("unknown");
        let adb_state = match state {
            "device" => "device",
            "unauthorized" => "unauthorized",
            "offline" => "offline",
            other => other,
        };
        devices.push(DeviceInfo {
            serial: serial.to_string(),
            adb_state: adb_state.to_string(),
            model: extract_prop(line, "model:"),
            product: extract_prop(line, "product:"),
        });
    }
    Ok(devices)
}

fn extract_prop(line: &str, key: &str) -> Option<String> {
    line.split_whitespace()
        .find_map(|tok| tok.strip_prefix(key).map(|v| v.to_string()))
}

#[tauri::command]
pub fn start_relay(
    app: AppHandle,
    state: State<'_, OrchestratorState>,
    port: Option<u16>,
) -> OrchResult<RelayStatePayload> {
    let mut relay = state
        .relay
        .lock()
        .map_err(|_| OrchError::coded(codes::RELAY_START_FAILED, "relay lock poisoned"))?;

    if let Some(child) = relay.child.as_mut() {
        match child.try_wait() {
            Ok(None) => {
                return Err(OrchError::coded(
                    codes::RELAY_START_FAILED,
                    "A session-owned relay is already running. Stop it first.",
                ));
            }
            Ok(Some(_)) => {
                relay.child = None;
                relay.owned_by_session = false;
            }
            Err(e) => {
                return Err(OrchError::coded(
                    codes::RELAY_START_FAILED,
                    format!("Failed to poll relay child: {e}"),
                ));
            }
        }
    }

    let listen_port = port.unwrap_or(relay.port);
    relay.port = listen_port;

    // P0-P2: bind probe BEFORE ownership / Relay Listening
    if !port_available(listen_port) {
        let msg = format!(
            "stage=start_relay port={listen_port} error_code={} \
             Port {listen_port} is already in use. Will not kill a foreign process.",
            codes::PORT_IN_USE
        );
        emit_log(&app, "error", "orchestrator", &msg);
        emit_error(
            &app,
            codes::PORT_IN_USE,
            format!("Port {listen_port} already in use (foreign relay or leftover)."),
        );
        // Do NOT set ownedBySession or relay_running
        emit_relay_state(
            &app,
            RelayStatePayload {
                state: "relay_error".into(),
                port: Some(listen_port),
                pid: None,
                owned_by_session: false,
                message: Some(codes::PORT_IN_USE.into()),
            },
        );
        return Err(OrchError::coded(
            codes::PORT_IN_USE,
            format!("Port {listen_port} already in use"),
        ));
    }

    let bin = match resolve_gnirehtet_bin(&app) {
        Ok(b) => b,
        Err(e) => {
            let (code, message) = match &e {
                OrchError::Message { code, message } => (code.as_str(), message.clone()),
            };
            emit_error(&app, code, message.clone());
            emit_relay_state(
                &app,
                RelayStatePayload {
                    state: "relay_error".into(),
                    port: Some(listen_port),
                    pid: None,
                    owned_by_session: false,
                    message: Some(message),
                },
            );
            return Err(e);
        }
    };

    if resolve_apk_path(&app).is_none() {
        // Not fatal for relay-only; document for install path later
        emit_log(
            &app,
            "warn",
            "orchestrator",
            format!(
                "stage=start_relay port={listen_port} note={} APK not resolved (OK for relay-only)",
                codes::APK_MISSING
            ),
        );
    }

    emit_log(
        &app,
        "info",
        "orchestrator",
        format!(
            "stage=start_relay port={listen_port} bin={} spawning",
            bin.display()
        ),
    );

    emit_relay_state(
        &app,
        RelayStatePayload {
            state: "relay_starting".into(),
            port: Some(listen_port),
            pid: None,
            owned_by_session: false, // not owned until spawn succeeds
            message: Some("Spawning gnirehtet relay".into()),
        },
    );

    let mut cmd = Command::new(&bin);
    cmd.args(["relay", "-p", &listen_port.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(apk) = resolve_apk_path(&app) {
        cmd.env("GNIREHTET_APK", &apk);
    }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let message = format!("Failed to spawn '{}': {e}", bin.display());
            emit_error(&app, codes::RELAY_START_FAILED, &message);
            emit_relay_state(
                &app,
                RelayStatePayload {
                    state: "relay_error".into(),
                    port: Some(listen_port),
                    pid: None,
                    owned_by_session: false,
                    message: Some(message.clone()),
                },
            );
            return Err(OrchError::coded(codes::RELAY_START_FAILED, message));
        }
    };

    let pid = child.id();
    pipe_child_output(app.clone(), child.stdout.take(), "gnirehtet.stdout");
    pipe_child_output(app.clone(), child.stderr.take(), "gnirehtet.stderr");

    // Brief settle: if child dies immediately, treat as RELAY_START_FAILED (never owned listening)
    thread::sleep(Duration::from_millis(150));
    match child.try_wait() {
        Ok(Some(status)) => {
            let message = format!(
                "stage=start_relay port={listen_port} error_code={} exited immediately: {status}",
                codes::RELAY_START_FAILED
            );
            emit_error(&app, codes::RELAY_START_FAILED, &message);
            emit_relay_state(
                &app,
                RelayStatePayload {
                    state: "relay_error".into(),
                    port: Some(listen_port),
                    pid: None,
                    owned_by_session: false,
                    message: Some(message.clone()),
                },
            );
            return Err(OrchError::coded(codes::RELAY_START_FAILED, message));
        }
        Ok(None) => {}
        Err(e) => {
            let message = format!("Failed to poll new relay child: {e}");
            emit_error(&app, codes::RELAY_START_FAILED, &message);
            let _ = child.kill();
            return Err(OrchError::coded(codes::RELAY_START_FAILED, message));
        }
    }

    relay.child = Some(child);
    relay.owned_by_session = true;
    drop(relay);

    let app_watch = app.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(400));
            let state = app_watch.state::<OrchestratorState>();
            let mut relay = match state.relay.lock() {
                Ok(g) => g,
                Err(_) => break,
            };
            let Some(child) = relay.child.as_mut() else {
                break;
            };
            match child.try_wait() {
                Ok(Some(status)) => {
                    relay.child = None;
                    relay.owned_by_session = false;
                    let port = relay.port;
                    drop(relay);
                    let code = codes::RELAY_CRASHED;
                    let msg = format!(
                        "stage=relay_exit port={port} error_code={code} status={status}"
                    );
                    // P0-R5: UI via Error + RelayState promptly (poll ≤400ms << 3s SLA)
                    emit_error(&app_watch, code, &msg);
                    emit_relay_state(
                        &app_watch,
                        RelayStatePayload {
                            state: "relay_error".into(),
                            port: Some(port),
                            pid: None,
                            owned_by_session: false,
                            message: Some(msg),
                        },
                    );
                    break;
                }
                Ok(None) => continue,
                Err(e) => {
                    emit_log(
                        &app_watch,
                        "error",
                        "orchestrator",
                        format!("Relay wait error: {e}"),
                    );
                    break;
                }
            }
        }
    });

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
    Ok(payload)
}

fn pipe_child_output<T>(app: AppHandle, stream: Option<T>, source: &'static str)
where
    T: std::io::Read + Send + 'static,
{
    let Some(stream) = stream else {
        return;
    };
    thread::spawn(move || {
        let reader = BufReader::new(stream);
        for line in reader.lines() {
            match line {
                Ok(text) => emit_log(&app, "info", source, text),
                Err(_) => break,
            }
        }
    });
}

#[tauri::command]
pub fn stop_relay(
    app: AppHandle,
    state: State<'_, OrchestratorState>,
) -> OrchResult<RelayStatePayload> {
    let mut relay = state
        .relay
        .lock()
        .map_err(|_| OrchError::coded(codes::STOP_FAILED, "relay lock poisoned"))?;

    if !relay.owned_by_session {
        // P0-R4: do not kill foreign process; no invented FOREIGN_* ERROR_UX code.
        let port = relay.port;
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

    let port = relay.port;
    let Some(mut child) = relay.child.take() else {
        relay.owned_by_session = false;
        let payload = RelayStatePayload {
            state: "relay_stopped".into(),
            port: Some(port),
            pid: None,
            owned_by_session: false,
            message: Some("Relay already stopped".into()),
        };
        emit_relay_state(&app, payload.clone());
        return Ok(payload);
    };

    let pid = child.id();
    emit_log(
        &app,
        "info",
        "orchestrator",
        format!("stage=stop port={port} pid={pid} stopping_owned_relay"),
    );

    if let Err(e) = kill_owned_child(&mut child) {
        relay.owned_by_session = false;
        emit_error(&app, codes::STOP_FAILED, &e);
        return Err(OrchError::coded(codes::STOP_FAILED, e));
    }
    relay.owned_by_session = false;

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

#[tauri::command]
pub fn get_relay_state(state: State<'_, OrchestratorState>) -> OrchResult<RelayStatePayload> {
    let mut relay = state
        .relay
        .lock()
        .map_err(|_| OrchError::coded(codes::RELAY_START_FAILED, "relay lock poisoned"))?;

    if let Some(child) = relay.child.as_mut() {
        match child.try_wait() {
            Ok(None) => {
                let pid = child.id();
                let port = relay.port;
                let owned = relay.owned_by_session;
                return Ok(RelayStatePayload {
                    state: "relay_running".into(),
                    port: Some(port),
                    pid: Some(pid),
                    owned_by_session: owned,
                    message: None,
                });
            }
            Ok(Some(_)) => {
                relay.child = None;
                relay.owned_by_session = false;
            }
            Err(_) => {}
        }
    }

    Ok(RelayStatePayload {
        state: "relay_stopped".into(),
        port: Some(relay.port),
        pid: None,
        owned_by_session: false,
        message: None,
    })
}
