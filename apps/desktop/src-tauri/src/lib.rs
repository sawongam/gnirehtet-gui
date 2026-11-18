mod autostart;
mod commands;
mod error;
mod logger;
mod paths;
mod relay_output;
mod session;
mod tray;
mod types;

use gnirehtet_controller::gnirehtet_adb::{AdbClient, AdbConfig};
use paths::ToolPaths;
use session::{EventSink, Session};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, RunEvent};
use types::{ClientEvent, LogEntry, RelayState, RelayStatus};

/// Forwards session events to the webview.
struct WebviewEvents(AppHandle);

impl EventSink for WebviewEvents {
    fn log(&self, entry: LogEntry) {
        let _ = self.0.emit("log", entry);
    }

    fn relay_status(&self, status: &RelayStatus) {
        tray::set_active(&self.0, status.state == RelayState::Running);
        let _ = self.0.emit("relay-status", status);
    }

    fn relay_client(&self, event: &ClientEvent) {
        let _ = self.0.emit("relay-client", event);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        // Must be the first plugin. A second launch (e.g. from the menu while
        // the app sits in the tray) brings the running window back instead of
        // starting another relay.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_window(app);
        }))
        .setup(|app| {
            let events: Arc<dyn EventSink> = Arc::new(WebviewEvents(app.handle().clone()));
            logger::init(Arc::clone(&events));

            let tools = ToolPaths::resolve(app.path().resource_dir().ok().as_deref());
            log::info!(
                target: "App",
                "adb: {} | relay: {} | client APK: {}",
                tools.adb.display(),
                tools.relay.display(),
                tools.apk.display()
            );
            let adb = AdbClient::new(AdbConfig {
                adb_path: tools.adb.clone(),
                apk_path: tools.apk.clone(),
                ..AdbConfig::default()
            });
            app.manage(Session::new(adb, tools.relay.clone(), events));
            app.manage(tools);

            // Tauri only exposes the AppImage path on Linux.
            #[cfg(target_os = "linux")]
            let appimage = app.env().appimage;
            #[cfg(not(target_os = "linux"))]
            let appimage: Option<std::ffi::OsString> = None;
            let autostart =
                autostart::Autostart::new(appimage.as_deref().map(std::path::Path::new));
            // Development builds must not repoint the entry at target/debug.
            if !cfg!(debug_assertions) {
                autostart.refresh();
            }
            app.manage(autostart);

            tray::setup(app);
            Ok(())
        })
        .on_window_event(tray::on_window_event)
        .invoke_handler(tauri::generate_handler![
            commands::check_adb,
            commands::list_devices,
            commands::get_session,
            commands::connect,
            commands::disconnect,
            commands::install_client,
            commands::repair_tunnel,
            commands::get_environment,
            commands::get_autostart,
            commands::set_autostart,
            commands::quit,
            commands::open_repo,
        ])
        .build(tauri::generate_context!())
        .expect("failed to build the application");

    app.run(|app_handle, event| match event {
        // Never leave the relay running or the device VPN on after exit.
        RunEvent::Exit => {
            if let Some(session) = app_handle.try_state::<Arc<Session>>() {
                session.shutdown();
            }
        }
        // Clicking the Dock icon brings back a window hidden in the tray.
        #[cfg(target_os = "macos")]
        RunEvent::Reopen {
            has_visible_windows: false,
            ..
        } => tray::show_window(app_handle),
        _ => {}
    });
}
