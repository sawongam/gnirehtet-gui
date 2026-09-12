//! Host-orchestrator stubs (Phase 0).
//!
//! Spawns the stock external `gnirehtet` binary for relay — never links relaylib.
//! Direct `adb` is used only for discovery/health (`ensure_adb`, `list_devices`).

mod types;

pub use types::*;

use serde::Serialize;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use tauri::{AppHandle, Emitter, Manager, State};

const DEFAULT_RELAY_PORT: u16 = 31416;

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

/// Session-scoped relay child owned by this process (if we started it).
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

fn emit_log(app: &AppHandle, level: &str, source: &str, message: impl Into<String>) {
    let line = LogLine {
        timestamp_ms: now_ms(),
        level: level.to_string(),
        source: source.to_string(),
        message: message.into(),
    };
    let _ = app.emit("LogLine", line);
}

fn emit_relay_state(app: &AppHandle, payload: RelayStatePayload) {
    let _ = app.emit("RelayState", payload);
}

fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn resolve_adb_path() -> PathBuf {
    if let Ok(p) = std::env::var("ADB") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    PathBuf::from("adb")
}

/// Resolve stock `gnirehtet` binary: GNIREHTET_BIN → sidecar candidates → PATH.
fn resolve_gnirehtet_bin(app: &AppHandle) -> OrchResult<PathBuf> {
    if let Ok(p) = std::env::var("GNIREHTET_BIN") {
        let pb = PathBuf::from(&p);
        if pb.is_file() {
            return Ok(pb);
        }
        return Err(OrchError::coded(
            "gnirehtet_not_found",
            format!("GNIREHTET_BIN set but not a file: {p}"),
        ));
    }

    // Packaged resource / externalBin layout (when present)
    if let Ok(res_dir) = app.path().resource_dir() {
        for name in ["gnirehtet", "gnirehtet.exe"] {
            let candidate = res_dir.join(name);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }

    // Dev drop-in: src-tauri/binaries/gnirehtet-<triple>
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

    // PATH
    if which_exists("gnirehtet") {
        return Ok(PathBuf::from("gnirehtet"));
    }

    Err(OrchError::coded(
        "gnirehtet_not_found",
        "gnirehtet binary not found. Drop a stock upstream binary into \
         apps/desktop/src-tauri/binaries/ (see docs/SIDECAR.md), set GNIREHTET_BIN, \
         or install gnirehtet on PATH.",
    ))
}

fn current_host_triple_guess() -> String {
    // Best-effort when env unset; Linux CI box is typically this.
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "x86_64-unknown-linux-gnu".into()
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "aarch64-apple-darwin".into()
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "x86_64-apple-darwin".into()
    }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "x86_64-pc-windows-msvc".into()
    }
    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "x86_64"),
    )))]
    {
        "unknown".into()
    }
}

fn which_exists(cmd: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {cmd} >/dev/null 2>&1"))
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Resolve APK path stub: setting (none yet) > GNIREHTET_APK > bundled resources/.
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
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../resources/gnirehtet.apk");
    if repo.is_file() {
        return Some(repo);
    }
    None
}

#[tauri::command]
pub fn ensure_adb() -> OrchResult<AdbInfo> {
    let path = resolve_adb_path();
    let output = Command::new(&path)
        .arg("version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| {
            OrchError::coded(
                "adb_not_found",
                format!(
                    "Failed to run adb at '{}': {e}. Install platform-tools or set ADB / Settings.",
                    path.display()
                ),
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(OrchError::coded(
            "adb_not_found",
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

#[tauri::command]
pub fn list_devices() -> OrchResult<Vec<DeviceInfo>> {
    let path = resolve_adb_path();
    let output = Command::new(&path)
        .args(["devices", "-l"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| {
            OrchError::coded(
                "adb_not_found",
                format!("Failed to run adb devices: {e}"),
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(OrchError::coded(
            "adb_error",
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
        let Some(serial) = parts.next() else { continue };
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
        .map_err(|_| OrchError::coded("internal", "relay lock poisoned"))?;

    if let Some(child) = relay.child.as_mut() {
        match child.try_wait() {
            Ok(None) => {
                return Err(OrchError::coded(
                    "relay_already_running",
                    "A session-owned relay is already running. Stop it first.",
                ));
            }
            Ok(Some(_)) => {
                relay.child = None;
                relay.owned_by_session = false;
            }
            Err(e) => {
                return Err(OrchError::coded(
                    "relay_error",
                    format!("Failed to poll relay child: {e}"),
                ));
            }
        }
    }

    let listen_port = port.unwrap_or(relay.port);
    relay.port = listen_port;

    let bin = resolve_gnirehtet_bin(&app)?;
    emit_log(
        &app,
        "info",
        "orchestrator",
        format!(
            "Starting gnirehtet relay on port {listen_port} via {}",
            bin.display()
        ),
    );

    emit_relay_state(
        &app,
        RelayStatePayload {
            state: "relay_starting".into(),
            port: Some(listen_port),
            pid: None,
            owned_by_session: true,
            message: Some("Spawning gnirehtet relay".into()),
        },
    );

    let mut cmd = Command::new(&bin);
    cmd.args(["relay", "-p", &listen_port.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Pass APK path through for verbs that need it later; harmless for relay.
    if let Some(apk) = resolve_apk_path(&app) {
        cmd.env("GNIREHTET_APK", &apk);
    }

    let mut child = cmd.spawn().map_err(|e| {
        let payload = RelayStatePayload {
            state: "relay_error".into(),
            port: Some(listen_port),
            pid: None,
            owned_by_session: false,
            message: Some(format!("Failed to spawn gnirehtet: {e}")),
        };
        emit_relay_state(&app, payload);
        OrchError::coded(
            "relay_spawn_failed",
            format!("Failed to spawn '{}': {e}", bin.display()),
        )
    })?;

    let pid = child.id();
    pipe_child_output(app.clone(), child.stdout.take(), "gnirehtet.stdout");
    pipe_child_output(app.clone(), child.stderr.take(), "gnirehtet.stderr");

    relay.child = Some(child);
    relay.owned_by_session = true;
    drop(relay); // release lock before spawning poller

    // Poll session-owned child exit without holding the mutex across sleep.
    let app_watch = app.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(std::time::Duration::from_millis(400));
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
                    let msg = format!("Relay exited: {status}");
                    drop(relay);
                    emit_log(&app_watch, "warn", "orchestrator", &msg);
                    emit_relay_state(
                        &app_watch,
                        RelayStatePayload {
                            state: if status.success() {
                                "relay_stopped".into()
                            } else {
                                "relay_error".into()
                            },
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
    emit_relay_state(&app, payload.clone());
    Ok(payload)
}

fn pipe_child_output<T>(app: AppHandle, stream: Option<T>, source: &'static str)
where
    T: std::io::Read + Send + 'static,
{
    let Some(stream) = stream else { return };
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
        .map_err(|_| OrchError::coded("internal", "relay lock poisoned"))?;

    if !relay.owned_by_session {
        return Err(OrchError::coded(
            "foreign_relay",
            "No session-owned relay to stop. Will not kill a foreign gnirehtet process.",
        ));
    }

    let Some(mut child) = relay.child.take() else {
        relay.owned_by_session = false;
        let payload = RelayStatePayload {
            state: "relay_stopped".into(),
            port: Some(relay.port),
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
        format!("Stopping session-owned relay pid={pid}"),
    );

    let _ = child.kill();
    let _ = child.wait();
    relay.owned_by_session = false;

    let payload = RelayStatePayload {
        state: "relay_stopped".into(),
        port: Some(relay.port),
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
        .map_err(|_| OrchError::coded("internal", "relay lock poisoned"))?;

    if let Some(child) = relay.child.as_mut() {
        match child.try_wait() {
            Ok(None) => {
                return Ok(RelayStatePayload {
                    state: "relay_running".into(),
                    port: Some(relay.port),
                    pid: Some(child.id()),
                    owned_by_session: relay.owned_by_session,
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
