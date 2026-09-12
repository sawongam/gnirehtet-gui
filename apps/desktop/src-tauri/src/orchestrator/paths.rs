//! Path resolution for Phase 0: setting (later) > env > bundled / sidecar drop-in.
//!
//! APK: `GNIREHTET_APK` / `resources/gnirehtet.apk` via `gnirehtet_adb::resolve_apk_path`.
//! Binary: `GNIREHTET_BIN` / `binaries/gnirehtet-<triple>` via `ControllerConfig::default`.

use gnirehtet_controller::{resolve_apk_path, ControllerConfig};
use std::path::PathBuf;

/// Build session config from env + local drop-ins (no settings store yet).
pub fn controller_config_from_env() -> ControllerConfig {
    let mut cfg = ControllerConfig::default();
    // AdbConfig::default already resolved APK via env / bundled candidates.
    // Re-assert through public helper so Desktop and adb stay aligned.
    cfg.adb.apk_path = resolve_apk_path(None);
    cfg
}

/// Snapshot of resolved paths for diagnostics / lab notes.
#[derive(Debug, Clone)]
pub struct ResolvedPaths {
    pub gnirehtet_bin: PathBuf,
    pub apk_path: PathBuf,
    pub apk_present: bool,
    pub bin_present: bool,
}

pub fn resolved_paths(cfg: &ControllerConfig) -> ResolvedPaths {
    let apk_path = cfg.adb.apk_path.clone();
    let gnirehtet_bin = cfg.gnirehtet_path.clone();
    ResolvedPaths {
        apk_present: apk_path.is_file(),
        bin_present: gnirehtet_bin.is_file()
            || which_on_path(&gnirehtet_bin),
        gnirehtet_bin,
        apk_path,
    }
}

fn which_on_path(name: &std::path::Path) -> bool {
    if name.is_absolute() || name.parent().map(|p| !p.as_os_str().is_empty()).unwrap_or(false) {
        return name.is_file();
    }
    let Some(path_os) = std::env::var_os("PATH") else {
        return false;
    };
    for dir in std::env::split_paths(&path_os) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return true;
        }
    }
    false
}
