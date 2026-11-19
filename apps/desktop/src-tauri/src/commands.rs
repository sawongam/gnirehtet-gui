//! IPC commands. Each one runs on a blocking worker thread: adb calls can
//! take seconds, and synchronous Tauri commands would run on the UI thread.

use crate::adb_install;
use crate::autostart::Autostart;
use crate::error::{CommandError, CommandResult};
use crate::paths::{self, ToolPaths};
use crate::session::Session;
use crate::types::{
    AdbInfo, ConnectOptions, DeviceInfo, DownloadProgress, Environment, RelayStatus,
    SessionSnapshot,
};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

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

/// Whether the app can download adb itself on this platform.
#[tauri::command]
pub fn can_install_adb(tools: State<'_, ToolPaths>) -> bool {
    tools.tools_dir.is_some() && adb_install::download_url().is_some()
}

/// Download Google's platform-tools into the app's data directory and switch
/// to its adb. Progress is reported through `adb-install-progress` events.
#[tauri::command]
pub async fn install_adb(
    app: AppHandle,
    session: State<'_, Arc<Session>>,
    tools: State<'_, ToolPaths>,
) -> CommandResult<AdbInfo> {
    let tools_dir = tools.tools_dir.clone().ok_or_else(|| {
        CommandError::new(
            "ADB_INSTALL_UNSUPPORTED",
            "The app has no data directory to install into.",
        )
    })?;
    blocking(&session, move |s| {
        let adb = adb_install::install(&tools_dir, |received, total| {
            let _ = app.emit("adb-install-progress", DownloadProgress { received, total });
        })?;
        s.set_adb_path(adb);
        s.check_adb()
    })
    .await
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
pub fn get_environment(
    app: AppHandle,
    tools: State<'_, ToolPaths>,
    session: State<'_, Arc<Session>>,
) -> Environment {
    Environment {
        app_version: app.package_info().version.to_string(),
        adb_path: session.adb_path().display().to_string(),
        relay_path: tools.relay.display().to_string(),
        relay_found: paths::is_available(&tools.relay),
        apk_path: tools.apk.display().to_string(),
        apk_found: tools.apk.is_file(),
    }
}

/// Whether the app is registered to start at sign-in.
#[tauri::command]
pub async fn get_autostart(app: AppHandle) -> CommandResult<bool> {
    tauri::async_runtime::spawn_blocking(move || app.state::<Autostart>().is_enabled())
        .await
        .map_err(|err| CommandError::new("INTERNAL", err.to_string()))?
}

/// Registers or unregisters start at sign-in; returns the resulting state.
#[tauri::command]
pub async fn set_autostart(app: AppHandle, enabled: bool) -> CommandResult<bool> {
    tauri::async_runtime::spawn_blocking(move || app.state::<Autostart>().set_enabled(enabled))
        .await
        .map_err(|err| CommandError::new("INTERNAL", err.to_string()))?
}

/// Exits the app. `RunEvent::Exit` stops the relay and the phone's VPN.
#[tauri::command]
pub fn quit(app: AppHandle) {
    app.exit(0);
}

const REPO_URL: &str = "https://github.com/sawongam/gnirehtet-gui";
const PLATFORM_TOOLS_URL: &str = "https://developer.android.com/tools/releases/platform-tools";

/// Open the project page in the default browser.
#[tauri::command]
pub fn open_repo() -> Result<(), String> {
    open_url(REPO_URL)
}

/// Open Google's platform-tools page, which holds its license terms.
#[tauri::command]
pub fn open_adb_terms() -> Result<(), String> {
    open_url(PLATFORM_TOOLS_URL)
}

/// URLs are fixed constants so the webview cannot use these commands to
/// launch arbitrary targets.
fn open_url(url: &str) -> Result<(), String> {
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
    cmd.arg(url).spawn().map(drop).map_err(|e| e.to_string())
}
