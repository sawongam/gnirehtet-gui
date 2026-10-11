//! Locating `adb`, the relay executable and the client APK.
//!
//! Each can be overridden with an environment variable (`ADB`,
//! `GNIREHTET_BIN`, `GNIREHTET_APK`); otherwise the copies bundled with the
//! app are used, and `adb` is looked up on `PATH` and in the usual Android
//! SDK locations.

use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

const EXE_SUFFIX: &str = env::consts::EXE_SUFFIX;

#[derive(Debug, Clone)]
pub struct ToolPaths {
    pub adb: PathBuf,
    pub relay: PathBuf,
    pub apk: PathBuf,
}

impl ToolPaths {
    /// `resource_dir` is where Tauri installed bundled resources (the APK).
    pub fn resolve(resource_dir: Option<&Path>) -> Self {
        let exe_dir = env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(Path::to_path_buf));
        Self {
            adb: resolve_adb(),
            relay: resolve_relay(exe_dir.as_deref()),
            apk: resolve_apk(resource_dir),
        }
    }
}

fn env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn resolve_adb() -> PathBuf {
    if let Some(path) = env_path("ADB") {
        return path;
    }
    let name = format!("adb{EXE_SUFFIX}");
    if find_on_path(Path::new(&name)).is_some() {
        return PathBuf::from(name);
    }
    sdk_dirs()
        .into_iter()
        .map(|sdk| sdk.join("platform-tools").join(&name))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| PathBuf::from(name))
}

/// Android SDK roots, from the environment and each platform's default.
fn sdk_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = ["ANDROID_HOME", "ANDROID_SDK_ROOT"]
        .iter()
        .filter_map(|name| env_path(name))
        .collect();
    let home = env_path("HOME").or_else(|| env_path("USERPROFILE"));
    if cfg!(target_os = "windows") {
        if let Some(local) = env_path("LOCALAPPDATA") {
            dirs.push(local.join("Android").join("Sdk"));
        }
    } else if cfg!(target_os = "macos") {
        if let Some(home) = home {
            dirs.push(home.join("Library/Android/sdk"));
        }
    } else if let Some(home) = home {
        dirs.push(home.join("Android/Sdk"));
    }
    dirs
}

fn resolve_relay(exe_dir: Option<&Path>) -> PathBuf {
    if let Some(path) = env_path("GNIREHTET_BIN") {
        return path;
    }
    // Tauri installs the bundled relay next to the app executable, with the
    // target-triple suffix removed. It is named `gnirehtet-relay` so that it
    // cannot clash with a distribution's own `gnirehtet` package.
    if let Some(dir) = exe_dir {
        let bundled = dir.join(format!("gnirehtet-relay{EXE_SUFFIX}"));
        if bundled.is_file() {
            return bundled;
        }
    }
    PathBuf::from(format!("gnirehtet{EXE_SUFFIX}"))
}

fn resolve_apk(resource_dir: Option<&Path>) -> PathBuf {
    if let Some(path) = env_path("GNIREHTET_APK") {
        return path;
    }
    let bundled = resource_dir.map(|dir| dir.join("gnirehtet.apk"));
    if let Some(bundled) = bundled.as_ref().filter(|path| path.is_file()) {
        return bundled.clone();
    }
    // Development checkout: `resources/gnirehtet.apk` at the repository root.
    #[cfg(debug_assertions)]
    {
        let checkout =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../resources/gnirehtet.apk");
        if checkout.is_file() {
            return checkout;
        }
    }
    bundled.unwrap_or_else(|| PathBuf::from("gnirehtet.apk"))
}

/// Whether `program` names an existing file, directly or through `PATH`.
pub fn is_available(program: &Path) -> bool {
    find_on_path(program).is_some()
}

fn find_on_path(program: &Path) -> Option<PathBuf> {
    let has_dir = program
        .parent()
        .is_some_and(|dir| !dir.as_os_str().is_empty());
    if program.is_absolute() || has_dir {
        return program.is_file().then(|| program.to_path_buf());
    }
    let path: OsString = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|dir| dir.join(program))
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_paths_are_checked_directly() {
        assert!(!is_available(Path::new("/no/such/tool")));
        let this = env::current_exe().unwrap();
        assert!(is_available(&this));
    }

    #[cfg(unix)]
    #[test]
    fn bare_names_are_looked_up_on_path() {
        assert!(is_available(Path::new("sh")));
        assert!(!is_available(Path::new("definitely-not-a-real-tool-xyz")));
    }

    #[test]
    fn bundled_relay_is_found_next_to_the_executable() {
        let dir = env::temp_dir().join(format!("gnirehtet-relay-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let relay = dir.join(format!("gnirehtet-relay{EXE_SUFFIX}"));
        std::fs::write(&relay, b"relay").unwrap();
        if env::var_os("GNIREHTET_BIN").is_none() {
            assert_eq!(resolve_relay(Some(&dir)), relay);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn bundled_apk_is_preferred() {
        let dir = env::temp_dir().join(format!("gnirehtet-paths-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let apk = dir.join("gnirehtet.apk");
        std::fs::write(&apk, b"apk").unwrap();
        if env::var_os("GNIREHTET_APK").is_none() {
            assert_eq!(resolve_apk(Some(&dir)), apk);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
