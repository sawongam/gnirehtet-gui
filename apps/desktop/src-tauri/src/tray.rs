//! System tray and what closing the window does.
//!
//! Closing the window must never silently cut a connected phone off, and
//! must never leave the app running with no way back to it. So:
//!
//! - Nothing connected: the window closes and the app exits.
//! - Connected, tray visible: the window hides; the tray restores or quits.
//! - Connected, no tray: the window stays and asks to disconnect and quit.
//!
//! Tray visibility is checked at close time, not at startup: on Linux the
//! panel that hosts tray icons can start after the app (e.g. at sign-in), or
//! not exist at all (GNOME without an AppIndicator extension).

use crate::session::Session;
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Emitter, Manager, WindowEvent,
};

/// Asks the webview to confirm disconnecting and quitting.
pub const QUIT_REQUESTED_EVENT: &str = "quit-requested";

const MAIN_WINDOW: &str = "main";
const IDLE_STATUS: &str = "Not connected";
const ACTIVE_STATUS: &str = "Sharing this computer's connection";

#[derive(Default)]
struct Tray {
    icon: Option<TrayIcon>,
    status: Option<MenuItem<tauri::Wry>>,
}

#[derive(Debug, PartialEq, Eq)]
enum CloseAction {
    Exit,
    Hide,
    Confirm,
}

fn close_action(session_active: bool, tray_visible: bool) -> CloseAction {
    match (session_active, tray_visible) {
        (false, _) => CloseAction::Exit,
        (true, true) => CloseAction::Hide,
        (true, false) => CloseAction::Confirm,
    }
}

pub fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        if let Err(error) = window
            .show()
            .and_then(|_| window.unminimize())
            .and_then(|_| window.set_focus())
        {
            log::error!(target: "App", "Failed to restore the window: {error}");
        }
    }
}

/// Creates the tray icon if this desktop can show one. Never fails: without a
/// tray, closing the window falls back to asking before quitting.
pub fn setup(app: &App) {
    let tray = if platform::indicator_library_present() {
        build(app).unwrap_or_else(|error| {
            log::warn!(target: "App", "System tray unavailable: {error}");
            Tray::default()
        })
    } else {
        log::warn!(
            target: "App",
            "System tray unavailable: install libayatana-appindicator3 to keep \
             Gnirehtet running in the tray when the window is closed"
        );
        Tray::default()
    };
    app.manage(tray);
}

fn build(app: &App) -> tauri::Result<Tray> {
    let status = MenuItem::with_id(app, "status", IDLE_STATUS, false, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "Show Gnirehtet", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&status, &show, &quit])?;
    let mut builder = TrayIconBuilder::with_id(MAIN_WINDOW)
        .tooltip("Gnirehtet")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_window(app),
            // RunEvent::Exit stops the relay and the phone's VPN.
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    Ok(Tray {
        icon: Some(builder.build(app)?),
        status: Some(status),
    })
}

/// Reflects whether a phone is using the connection in the tray menu.
pub fn set_active(app: &AppHandle, active: bool) {
    let Some(tray) = app.try_state::<Tray>() else {
        return;
    };
    let (text, tooltip) = if active {
        (ACTIVE_STATUS, "Gnirehtet — connected")
    } else {
        (IDLE_STATUS, "Gnirehtet")
    };
    if let Some(status) = &tray.status {
        let _ = status.set_text(text);
    }
    if let Some(icon) = &tray.icon {
        let _ = icon.set_tooltip(Some(tooltip));
    }
}

fn tray_visible(app: &AppHandle) -> bool {
    app.try_state::<Tray>()
        .is_some_and(|tray| tray.icon.is_some())
        && platform::tray_host_present()
}

pub fn on_window_event(window: &tauri::Window, event: &WindowEvent) {
    if window.label() != MAIN_WINDOW {
        return;
    }
    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };
    let app = window.app_handle();
    let active = app
        .try_state::<Arc<Session>>()
        .is_some_and(|session| session.is_active());
    let action = match close_action(active, active && tray_visible(app)) {
        CloseAction::Hide => match window.hide() {
            Ok(()) => CloseAction::Hide,
            Err(error) => {
                log::error!(target: "App", "Failed to hide the window: {error}");
                CloseAction::Confirm
            }
        },
        action => action,
    };
    match action {
        // The last window closing exits the app, which runs the cleanup.
        CloseAction::Exit => {}
        CloseAction::Hide => {
            api.prevent_close();
            log::info!(target: "App", "Still connected; Gnirehtet keeps running in the tray");
        }
        CloseAction::Confirm => {
            api.prevent_close();
            let _ = window.emit(QUIT_REQUESTED_EVENT, ());
        }
    }
}

#[cfg(target_os = "linux")]
mod platform {
    /// The names libappindicator-sys tries. It panics when none loads, and
    /// the app is built with `panic = "abort"`, so check first.
    const LIBRARIES: [&str; 4] = [
        "libayatana-appindicator3.so.1",
        "libappindicator3.so.1",
        "libayatana-appindicator3.so",
        "libappindicator3.so",
    ];

    pub fn indicator_library_present() -> bool {
        LIBRARIES
            .iter()
            // SAFETY: these are the libraries the tray loads anyway; their
            // initializers have no preconditions.
            .any(|name| unsafe { libloading::Library::new(name) }.is_ok())
    }

    /// Whether a panel hosting StatusNotifier icons is running. Without one,
    /// AppIndicator icons are not shown anywhere.
    pub fn tray_host_present() -> bool {
        use gio::glib::{ToVariant, VariantTy};

        let Ok(bus) = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE) else {
            return false;
        };
        bus.call_sync(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "NameHasOwner",
            Some(&("org.kde.StatusNotifierWatcher",).to_variant()),
            Some(VariantTy::new("(b)").expect("valid type string")),
            gio::DBusCallFlags::NONE,
            1000,
            gio::Cancellable::NONE,
        )
        .ok()
        .and_then(|reply| reply.get::<(bool,)>())
        .is_some_and(|(present,)| present)
    }
}

#[cfg(not(target_os = "linux"))]
mod platform {
    pub fn indicator_library_present() -> bool {
        true
    }

    pub fn tray_host_present() -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{close_action, CloseAction};

    #[test]
    fn closing_when_idle_exits() {
        assert_eq!(close_action(false, true), CloseAction::Exit);
        assert_eq!(close_action(false, false), CloseAction::Exit);
    }

    #[test]
    fn closing_while_connected_never_strands_the_phone() {
        assert_eq!(close_action(true, true), CloseAction::Hide);
        assert_eq!(close_action(true, false), CloseAction::Confirm);
    }
}
