mod commands;
mod error;
mod logger;
mod paths;
mod relay_output;
mod session;
mod types;

use gnirehtet_controller::gnirehtet_adb::{AdbClient, AdbConfig};
use paths::ToolPaths;
use session::{EventSink, Session};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, RunEvent};
use types::{ClientEvent, LogEntry, RelayStatus};

/// Forwards session events to the webview.
struct WebviewEvents(AppHandle);

impl EventSink for WebviewEvents {
    fn log(&self, entry: LogEntry) {
        let _ = self.0.emit("log", entry);
    }

    fn relay_status(&self, status: &RelayStatus) {
        let _ = self.0.emit("relay-status", status);
    }

    fn relay_client(&self, event: &ClientEvent) {
        let _ = self.0.emit("relay-client", event);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::check_adb,
            commands::list_devices,
            commands::get_session,
            commands::connect,
            commands::disconnect,
            commands::install_client,
            commands::repair_tunnel,
            commands::get_environment,
        ])
        .build(tauri::generate_context!())
        .expect("failed to build the application");

    app.run(|app_handle, event| {
        // Never leave the relay running or the device VPN on after exit.
        if let RunEvent::Exit = event {
            if let Some(session) = app_handle.try_state::<Arc<Session>>() {
                session.shutdown();
            }
        }
    });
}
