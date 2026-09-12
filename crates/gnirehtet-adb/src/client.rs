/*
 * Copyright (C) 2017 Genymobile
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * Adapted for gnirehtet-gui Phase 1: AdbClient API extract (headers preserved).
 */

use crate::error::{Cmd, CommandExecutionError, ProcessIoError, ProcessStatusError};
use crate::monitor::AdbMonitor;
use log::*;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process;
use std::thread;
use std::time::Duration;

const TAG: &str = "AdbClient";

/// APK versionCode expected by upstream gnirehtet v2.5.1 — do not change lightly.
pub const REQUIRED_APK_VERSION_CODE: &str = "9";

/// Configuration for ADB path, APK path, and adbd address.
#[derive(Clone, Debug)]
pub struct AdbConfig {
    pub adb_path: PathBuf,
    pub apk_path: PathBuf,
    pub adbd_addr: SocketAddr,
}

impl Default for AdbConfig {
    fn default() -> Self {
        Self {
            adb_path: default_adb_path().into(),
            apk_path: default_apk_path().into(),
            adbd_addr: SocketAddr::from(([127, 0, 0, 1], 5037)),
        }
    }
}

impl AdbConfig {
    pub fn from_env() -> Self {
        Self::default()
    }
}

/// VPN / reverse-tether options passed to the Android client intent.
#[derive(Clone, Debug, Default)]
pub struct VpnOptions<'a> {
    pub dns_servers: Option<&'a str>,
    pub routes: Option<&'a str>,
    pub port: u16,
}

impl<'a> VpnOptions<'a> {
    pub fn new(port: u16) -> Self {
        Self {
            dns_servers: None,
            routes: None,
            port,
        }
    }
}

/// Result of `ensure_adb` — path + version line from `adb version`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdbStatus {
    pub path: String,
    pub version: String,
}

/// One row from `adb devices` / `adb devices -l`.
///
/// `state` is the raw adb state token (`device`, `unauthorized`, `offline`, …).
/// Optional `model` / `product` come from `-l` properties when present.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdbDevice {
    pub serial: String,
    pub state: String,
    pub model: Option<String>,
    pub product: Option<String>,
}

fn default_adb_path() -> String {
    if let Some(env_adb) = std::env::var_os("ADB") {
        env_adb.into_string().expect("invalid ADB value")
    } else {
        "adb".to_string()
    }
}

/// Resolve APK path: explicit later (caller) > `GNIREHTET_APK` > first existing
/// bundled candidate > `"gnirehtet.apk"` (upstream cwd-relative default).
fn default_apk_path() -> String {
    if let Some(env_apk) = std::env::var_os("GNIREHTET_APK") {
        return env_apk
            .into_string()
            .expect("invalid GNIREHTET_APK value");
    }
    for candidate in apk_search_candidates() {
        if candidate.is_file() {
            return candidate.to_string_lossy().into_owned();
        }
    }
    "gnirehtet.apk".to_string()
}

/// Candidates for bundled / workspace APK (no settings store yet).
pub fn apk_search_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    out.push(PathBuf::from("resources/gnirehtet.apk"));
    out.push(PathBuf::from("gnirehtet.apk"));
    if let Ok(cwd) = std::env::current_dir() {
        out.push(cwd.join("resources/gnirehtet.apk"));
        out.push(cwd.join("gnirehtet.apk"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            // Dev: target/debug → ../../../resources ; packaged: next to exe / resources/
            for rel in [
                "resources/gnirehtet.apk",
                "../resources/gnirehtet.apk",
                "../../resources/gnirehtet.apk",
                "../../../resources/gnirehtet.apk",
                "../../../../resources/gnirehtet.apk",
            ] {
                out.push(dir.join(rel));
            }
        }
    }
    out
}

/// Public helper: setting override > env > bundled file > default name.
pub fn resolve_apk_path(setting: Option<&Path>) -> PathBuf {
    if let Some(p) = setting {
        if !p.as_os_str().is_empty() {
            return p.to_path_buf();
        }
    }
    PathBuf::from(default_apk_path())
}

/// Reusable ADB orchestration API (install / start / stop / tunnel / …).
#[derive(Clone, Debug)]
pub struct AdbClient {
    config: AdbConfig,
}

impl AdbClient {
    pub fn new(config: AdbConfig) -> Self {
        Self { config }
    }

    pub fn from_env() -> Self {
        Self::new(AdbConfig::from_env())
    }

    pub fn config(&self) -> &AdbConfig {
        &self.config
    }

    pub fn adb_path(&self) -> &Path {
        &self.config.adb_path
    }

    pub fn apk_path(&self) -> &Path {
        &self.config.apk_path
    }

    pub fn install(&self, serial: Option<&str>) -> Result<(), CommandExecutionError> {
        info!(target: TAG, "Installing gnirehtet client...");
        if !self.config.apk_path.is_file() {
            return Err(CommandExecutionError::ApkMissing {
                path: self.config.apk_path.display().to_string(),
            });
        }
        let apk = self
            .config
            .apk_path
            .to_str()
            .expect("apk path is not valid UTF-8")
            .to_string();
        self.exec_adb(serial, vec!["install".into(), "-r".into(), apk])
    }

    pub fn uninstall(&self, serial: Option<&str>) -> Result<(), CommandExecutionError> {
        info!(target: TAG, "Uninstalling gnirehtet client...");
        self.exec_adb(serial, vec!["uninstall", "com.genymobile.gnirehtet"])
    }

    pub fn reinstall(&self, serial: Option<&str>) -> Result<(), CommandExecutionError> {
        self.uninstall(serial)?;
        self.install(serial)?;
        Ok(())
    }

    pub fn tunnel(&self, serial: Option<&str>, port: u16) -> Result<(), CommandExecutionError> {
        self.exec_adb(
            serial,
            vec![
                "reverse".into(),
                "localabstract:gnirehtet".into(),
                format!("tcp:{}", port),
            ],
        )
    }

    pub fn start(
        &self,
        serial: Option<&str>,
        opts: &VpnOptions<'_>,
    ) -> Result<(), CommandExecutionError> {
        if self.must_install_client(serial)? {
            self.install(serial)?;
            // wait a bit after the app is installed so that intent actions are correctly
            // registered
            thread::sleep(Duration::from_millis(500));
        }

        info!(target: TAG, "Starting client...");
        self.tunnel(serial, opts.port)?;

        let mut adb_args: Vec<String> = vec![
            "shell".into(),
            "am".into(),
            "start".into(),
            "-a".into(),
            "com.genymobile.gnirehtet.START".into(),
            "-n".into(),
            "com.genymobile.gnirehtet/.GnirehtetActivity".into(),
        ];
        if let Some(dns_servers) = opts.dns_servers {
            adb_args.push("--esa".into());
            adb_args.push("dnsServers".into());
            adb_args.push(dns_servers.into());
        }
        if let Some(routes) = opts.routes {
            adb_args.push("--esa".into());
            adb_args.push("routes".into());
            adb_args.push(routes.into());
        }
        self.exec_adb(serial, adb_args)
    }

    pub fn stop(&self, serial: Option<&str>) -> Result<(), CommandExecutionError> {
        info!(target: TAG, "Stopping client...");
        self.exec_adb(
            serial,
            vec![
                "shell",
                "am",
                "start",
                "-a",
                "com.genymobile.gnirehtet.STOP",
                "-n",
                "com.genymobile.gnirehtet/.GnirehtetActivity",
            ],
        )
    }

    pub fn restart(
        &self,
        serial: Option<&str>,
        opts: &VpnOptions<'_>,
    ) -> Result<(), CommandExecutionError> {
        self.stop(serial)?;
        self.start(serial, opts)?;
        Ok(())
    }

    /// True when the client APK is missing or its versionCode != REQUIRED_APK_VERSION_CODE.
    pub fn must_install_client(&self, serial: Option<&str>) -> Result<bool, CommandExecutionError> {
        info!(target: TAG, "Checking gnirehtet client...");
        let args = self.create_adb_args(
            serial,
            vec!["shell", "dumpsys", "package", "com.genymobile.gnirehtet"],
        );
        let adb = self.adb_path_string();
        debug!(target: TAG, "Execute: {:?} {:?}", adb, args);
        match process::Command::new(&adb).args(&args[..]).output() {
            Ok(output) => {
                if output.status.success() {
                    // the "regex" crate makes the binary far bigger, so just parse the versionCode
                    // manually
                    let dumpsys = String::from_utf8_lossy(&output.stdout[..]);
                    // read the versionCode of the installed package
                    if let Some(index) = dumpsys.find("    versionCode=") {
                        let start = index + 16; // size of "    versionCode=\""
                        if let Some(end) = (&dumpsys[start..]).find(' ') {
                            let installed_version_code = &dumpsys[start..start + end];
                            Ok(installed_version_code != REQUIRED_APK_VERSION_CODE)
                        } else {
                            // end of versionCode value not found
                            Ok(true)
                        }
                    } else {
                        // versionCode not found
                        Ok(true)
                    }
                } else {
                    let cmd = Cmd::new(adb, args);
                    Err(ProcessStatusError::new(cmd, output.status).into())
                }
            }
            Err(err) => {
                let cmd = Cmd::new(adb, args);
                Err(ProcessIoError::new(cmd, err).into())
            }
        }
    }

    /// Spawn start on a background thread (parity with upstream `async_start`).
    pub fn async_start(
        &self,
        serial: Option<&str>,
        dns_servers: Option<&str>,
        routes: Option<&str>,
        port: u16,
    ) {
        let client = self.clone();
        let start_serial = serial.map(String::from);
        let start_dns = dns_servers.map(String::from);
        let start_routes = routes.map(String::from);
        thread::spawn(move || {
            let serial = start_serial.as_ref().map(String::as_ref);
            let opts = VpnOptions {
                dns_servers: start_dns.as_deref(),
                routes: start_routes.as_deref(),
                port,
            };
            if let Err(err) = client.start(serial, &opts) {
                error!(target: TAG, "Cannot start client: {}", err);
            }
        });
    }

    /// Build an `AdbMonitor` that uses this client's adb path and adbd address.
    pub fn monitor(&self, callback: Box<dyn crate::monitor::AdbMonitorCallback>) -> AdbMonitor {
        AdbMonitor::with_config(
            callback,
            self.adb_path_string(),
            self.config.adbd_addr,
        )
    }


    /// Start the adb server (if needed) and verify the configured `adb_path`.
    ///
    /// Uses real CLI: `adb start-server` then `adb version`.
    /// Failures are `CommandExecutionError` — use `adb_ux_code_hint()` for
    /// ERROR_UX `ADB_MISSING` / `ADB_PATH_INVALID` mapping.
    pub fn ensure_adb(&self) -> Result<AdbStatus, CommandExecutionError> {
        info!(target: TAG, "Ensuring adb is available...");
        // Bring up the daemon using the configured binary (ADB env / AdbConfig).
        self.exec_adb(None, vec!["start-server"])?;

        let adb = self.adb_path_string();
        let args = vec!["version".to_string()];
        debug!(target: TAG, "Execute: {:?} {:?}", adb, args);
        match process::Command::new(&adb).args(&args[..]).output() {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let version = stdout
                        .lines()
                        .next()
                        .unwrap_or("adb")
                        .trim()
                        .to_string();
                    Ok(AdbStatus {
                        path: adb,
                        version,
                    })
                } else {
                    let cmd = Cmd::new(adb, args);
                    Err(ProcessStatusError::new(cmd, output.status).into())
                }
            }
            Err(err) => {
                let cmd = Cmd::new(adb, args);
                Err(ProcessIoError::new(cmd, err).into())
            }
        }
    }

    /// List devices via `adb devices -l` (real adb output shape).
    ///
    /// Empty list is success (`NO_DEVICES` is a UI policy on an empty Vec).
    pub fn list_devices(&self) -> Result<Vec<AdbDevice>, CommandExecutionError> {
        let adb = self.adb_path_string();
        let args = vec!["devices".to_string(), "-l".to_string()];
        debug!(target: TAG, "Execute: {:?} {:?}", adb, args);
        match process::Command::new(&adb).args(&args[..]).output() {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(parse_adb_devices_l(&stdout))
                } else {
                    let cmd = Cmd::new(adb, args);
                    Err(ProcessStatusError::new(cmd, output.status).into())
                }
            }
            Err(err) => {
                let cmd = Cmd::new(adb, args);
                Err(ProcessIoError::new(cmd, err).into())
            }
        }
    }

    pub fn create_adb_args<S: Into<String>>(
        &self,
        serial: Option<&str>,
        args: Vec<S>,
    ) -> Vec<String> {
        let mut command = Vec::<String>::new();
        if let Some(serial) = serial {
            command.push("-s".into());
            command.push(serial.to_string());
        }
        for arg in args {
            command.push(arg.into());
        }
        command
    }

    pub fn exec_adb<S: Into<String>>(
        &self,
        serial: Option<&str>,
        args: Vec<S>,
    ) -> Result<(), CommandExecutionError> {
        let adb_args = self.create_adb_args(serial, args);
        let adb = self.adb_path_string();
        debug!(target: TAG, "Execute: {:?} {:?}", adb, adb_args);
        match process::Command::new(&adb).args(&adb_args[..]).status() {
            Ok(exit_status) => {
                if exit_status.success() {
                    Ok(())
                } else {
                    let cmd = Cmd::new(adb, adb_args);
                    Err(ProcessStatusError::new(cmd, exit_status).into())
                }
            }
            Err(err) => {
                let cmd = Cmd::new(adb, adb_args);
                Err(ProcessIoError::new(cmd, err).into())
            }
        }
    }

    fn adb_path_string(&self) -> String {
        self.config
            .adb_path
            .to_str()
            .expect("adb path is not valid UTF-8")
            .to_string()
    }
}

/// Parse stdout of `adb devices` or `adb devices -l`.
///
/// Skips the `List of devices attached` header and blank lines.
/// Only uses whitespace-separated columns as produced by stock adb.
pub fn parse_adb_devices_l(stdout: &str) -> Vec<AdbDevice> {
    let mut devices = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("List of devices") {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(serial) = parts.next() else { continue };
        let Some(state) = parts.next() else { continue };
        devices.push(AdbDevice {
            serial: serial.to_string(),
            state: state.to_string(),
            model: extract_adb_prop(line, "model:"),
            product: extract_adb_prop(line, "product:"),
        });
    }
    devices
}

fn extract_adb_prop(line: &str, key: &str) -> Option<String> {
    line.split_whitespace()
        .find_map(|tok| tok.strip_prefix(key).map(|v| v.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_devices_l_empty() {
        let out = "List of devices attached\n\n";
        assert!(parse_adb_devices_l(out).is_empty());
    }

    #[test]
    fn parse_devices_l_mixed_states() {
        let out = "\
List of devices attached
emulator-5554          device product:sdk_gphone model:sdk_gphone_x86 device:generic_x86
0123456789ABCDEF       unauthorized
deadbeef               offline transport_id:1
";
        let devices = parse_adb_devices_l(out);
        assert_eq!(devices.len(), 3);
        assert_eq!(devices[0].serial, "emulator-5554");
        assert_eq!(devices[0].state, "device");
        assert_eq!(devices[0].model.as_deref(), Some("sdk_gphone_x86"));
        assert_eq!(devices[0].product.as_deref(), Some("sdk_gphone"));
        assert_eq!(devices[1].serial, "0123456789ABCDEF");
        assert_eq!(devices[1].state, "unauthorized");
        assert!(devices[1].model.is_none());
        assert_eq!(devices[2].state, "offline");
    }

    #[test]
    fn parse_devices_without_l_props() {
        let out = "List of devices attached\nABC\tdevice\n";
        let devices = parse_adb_devices_l(out);
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].serial, "ABC");
        assert_eq!(devices[0].state, "device");
    }
}

#[cfg(test)]
mod apk_path_tests {
    use super::*;
    use crate::error::CommandExecutionError;

    #[test]
    fn install_missing_apk_returns_apk_missing() {
        let client = AdbClient::new(AdbConfig {
            apk_path: PathBuf::from("/no/such/gnirehtet.apk"),
            ..AdbConfig::default()
        });
        let err = client.install(None).expect_err("missing apk");
        assert!(matches!(err, CommandExecutionError::ApkMissing { .. }));
        assert_eq!(err.adb_ux_code_hint(), Some("APK_MISSING"));
    }

    #[test]
    fn resolve_apk_path_honors_setting() {
        let p = PathBuf::from("/custom/setting.apk");
        assert_eq!(resolve_apk_path(Some(&p)), p);
    }
}
