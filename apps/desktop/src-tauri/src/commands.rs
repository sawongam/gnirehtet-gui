//! IPC commands. Each one runs on a blocking worker thread: adb calls can
//! take seconds, and synchronous Tauri commands would run on the UI thread.

use crate::error::{CommandError, CommandResult};
use crate::paths::{self, ToolPaths};
use crate::session::Session;
use crate::types::{
    AdbInfo, ConnectOptions, DeviceInfo, Environment, RelayStatus, SessionSnapshot,
};
use std::sync::Arc;
use tauri::{AppHandle, State};

async fn blocking<T, F>(session: &State<'_, Arc<Session>>, f: F) -> CommandResult<T>
where
    T: Send + 'static,
    F: FnOnce(&Session) -> CommandResult<T> + Send + 'static,
{
    let session = Arc::clone(session.inner());
    tauri::async_runtime::spawn_blocking(move || f(&session))
        .await
        .map_err(|err| CommandError::new("INTERNAL", err.to_string()))?
}

#[tauri::command]
pub async fn check_adb(session: State<'_, Arc<Session>>) -> CommandResult<AdbInfo> {
    blocking(&session, |s| s.check_adb()).await
}

#[tauri::command]
pub async fn list_devices(session: State<'_, Arc<Session>>) -> CommandResult<Vec<DeviceInfo>> {
    blocking(&session, |s| s.list_devices()).await
}

#[tauri::command]
pub fn get_session(session: State<'_, Arc<Session>>) -> SessionSnapshot {
    session.snapshot()
}

#[tauri::command]
pub async fn connect(
    session: State<'_, Arc<Session>>,
    serial: String,
    options: ConnectOptions,
) -> CommandResult<RelayStatus> {
    let session = Arc::clone(session.inner());
    tauri::async_runtime::spawn_blocking(move || session.connect(&serial, options))
        .await
        .map_err(|err| CommandError::new("INTERNAL", err.to_string()))?
}

#[tauri::command]
pub async fn disconnect(session: State<'_, Arc<Session>>) -> CommandResult<RelayStatus> {
    blocking(&session, |s| Ok(s.disconnect())).await
}

#[tauri::command]
pub async fn install_client(session: State<'_, Arc<Session>>, serial: String) -> CommandResult<()> {
    blocking(&session, move |s| s.install_client(&serial)).await
}

#[tauri::command]
pub async fn repair_tunnel(session: State<'_, Arc<Session>>, serial: String) -> CommandResult<()> {
    blocking(&session, move |s| s.repair_tunnel(&serial)).await
}

#[tauri::command]
pub fn get_environment(app: AppHandle, tools: State<'_, ToolPaths>) -> Environment {
    Environment {
        app_version: app.package_info().version.to_string(),
        adb_path: tools.adb.display().to_string(),
        relay_path: tools.relay.display().to_string(),
        relay_found: paths::is_available(&tools.relay),
        apk_path: tools.apk.display().to_string(),
        apk_found: tools.apk.is_file(),
    }
}

const REPO_URL: &str = "https://github.com/sawongam/gnirehtet-gui";

/// Open the project page in the default browser. The URL is fixed so the
/// webview cannot use this to launch arbitrary targets.
#[tauri::command]
pub fn open_repo() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    let mut cmd = std::process::Command::new("xdg-open");
    #[cfg(target_os = "macos")]
    let mut cmd = std::process::Command::new("open");
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut cmd = std::process::Command::new("cmd");
        cmd.args(["/C", "start", ""]);
        cmd
    };
    cmd.arg(REPO_URL)
        .spawn()
        .map(drop)
        .map_err(|e| e.to_string())
}
