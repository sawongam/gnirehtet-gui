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

fn default_adb_path() -> String {
    if let Some(env_adb) = std::env::var_os("ADB") {
        env_adb.into_string().expect("invalid ADB value")
    } else {
        "adb".to_string()
    }
}

fn default_apk_path() -> String {
    if let Some(env_apk) = std::env::var_os("GNIREHTET_APK") {
        env_apk.into_string().expect("invalid GNIREHTET_APK value")
    } else {
        "gnirehtet.apk".to_string()
    }
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
