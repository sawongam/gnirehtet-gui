mod orchestrator;

use orchestrator::{teardown_owned_relay, OrchestratorState};
use tauri::{Manager, RunEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .manage(OrchestratorState::default())
        .invoke_handler(tauri::generate_handler![
            orchestrator::ensure_adb,
            orchestrator::list_devices,
            orchestrator::start_relay,
            orchestrator::stop_relay,
            orchestrator::get_relay_state,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        // P0-Q1/Q3: tear down owned relay on quit; never kill shared adb server.
        if matches!(
            event,
            RunEvent::Exit | RunEvent::ExitRequested { .. }
        ) {
            if let Some(state) = app_handle.try_state::<OrchestratorState>() {
                teardown_owned_relay(app_handle, state.inner(), "shutdown");
            }
        }
    });
}
