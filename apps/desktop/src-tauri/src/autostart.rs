//! Current-user "run at sign-in" registration: a Run registry value on
//! Windows, an XDG autostart entry on Linux, a launch agent on macOS.
//!
//! `auto-launch` writes the path verbatim into a command line on Windows and
//! Linux, so it is quoted here: unquoted, a path with spaces (such as a
//! Windows profile folder) would not launch, and Windows would try
//! `C:\Users\First.exe` first.

use crate::error::{CommandError, CommandResult};
use auto_launch::{AutoLaunch, AutoLaunchBuilder};
use std::path::Path;

/// Name of the registry value / autostart entry. Matches the product name the
/// uninstaller uses.
const APP_NAME: &str = "gnirehtet-gui";

pub struct Autostart(Result<AutoLaunch, String>);

impl Autostart {
    /// `appimage` is the AppImage file when running from one: the executable
    /// itself lives in a mount point that changes on every launch.
    pub fn new(appimage: Option<&Path>) -> Self {
        Self(build(appimage))
    }

    fn launcher(&self) -> CommandResult<&AutoLaunch> {
        self.0
            .as_ref()
            .map_err(|message| CommandError::new("AUTOSTART_UNAVAILABLE", message.clone()))
    }

    pub fn is_enabled(&self) -> CommandResult<bool> {
        self.launcher()?.is_enabled().map_err(failed)
    }

    /// Applies the change and returns the state read back from the OS.
    pub fn set_enabled(&self, enabled: bool) -> CommandResult<bool> {
        let launcher = self.launcher()?;
        if enabled {
            launcher.enable().map_err(failed)?;
        } else if launcher.is_enabled().map_err(failed)? {
            launcher.disable().map_err(failed)?;
        }
        launcher.is_enabled().map_err(failed)
    }

    /// Rewrites an existing registration so it points at this executable,
    /// e.g. after an AppImage was moved or the app was reinstalled elsewhere.
    pub fn refresh(&self) {
        let Ok(launcher) = &self.0 else { return };
        match launcher.is_enabled() {
            Ok(true) => {
                if let Err(err) = launcher.enable() {
                    log::warn!(target: "App", "Could not update the startup entry: {err}");
                }
            }
            Ok(false) => {}
            Err(err) => log::warn!(target: "App", "Could not read the startup entry: {err}"),
        }
    }
}

fn failed(err: auto_launch::Error) -> CommandError {
    CommandError::new("AUTOSTART_FAILED", err.to_string())
}

fn build(appimage: Option<&Path>) -> Result<AutoLaunch, String> {
    let exe = match appimage {
        Some(path) => path.to_path_buf(),
        None => std::env::current_exe().map_err(|e| e.to_string())?,
    };
    let exe = exe
        .to_str()
        .ok_or("The app's path is not valid Unicode")?
        .to_owned();
    let mut builder = AutoLaunchBuilder::new();
    builder
        .set_app_name(APP_NAME)
        .set_app_path(&launch_path(&exe));
    #[cfg(target_os = "macos")]
    builder.set_use_launch_agent(true);
    builder.build().map_err(|e| e.to_string())
}

#[cfg(target_os = "windows")]
fn launch_path(exe: &str) -> String {
    // Windows paths cannot contain `"`.
    format!("\"{exe}\"")
}

#[cfg(target_os = "linux")]
fn launch_path(exe: &str) -> String {
    quote_desktop_exec(exe)
}

/// macOS stores the path as its own launch agent argument; no quoting.
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn launch_path(exe: &str) -> String {
    exe.to_owned()
}

/// Quotes a program path for a desktop entry's `Exec` key, following the
/// Desktop Entry Specification: `"`, `` ` ``, `$` and `\` are escaped with a
/// backslash, and `%` (field codes) is doubled.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn quote_desktop_exec(path: &str) -> String {
    let mut quoted = String::with_capacity(path.len() + 2);
    quoted.push('"');
    for c in path.chars() {
        match c {
            '"' | '`' | '$' | '\\' => {
                quoted.push('\\');
                quoted.push(c);
            }
            '%' => quoted.push_str("%%"),
            _ => quoted.push(c),
        }
    }
    quoted.push('"');
    // The key file format then escapes the backslashes themselves.
    quoted.replace('\\', "\\\\")
}

#[cfg(test)]
mod tests {
    use super::quote_desktop_exec;

    #[test]
    fn quotes_plain_and_spaced_paths() {
        assert_eq!(
            quote_desktop_exec("/usr/bin/gnirehtet-gui"),
            "\"/usr/bin/gnirehtet-gui\""
        );
        assert_eq!(
            quote_desktop_exec("/home/a b/Apps/gnirehtet.AppImage"),
            "\"/home/a b/Apps/gnirehtet.AppImage\""
        );
    }

    #[test]
    fn escapes_reserved_characters() {
        assert_eq!(quote_desktop_exec("/x/100%/a"), "\"/x/100%%/a\"");
        assert_eq!(quote_desktop_exec("/x/$HOME"), "\"/x/\\\\$HOME\"");
        assert_eq!(quote_desktop_exec("/x/a\"b"), "\"/x/a\\\\\"b\"");
    }
}
