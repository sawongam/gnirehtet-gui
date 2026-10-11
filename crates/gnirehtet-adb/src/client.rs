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
 * Modified for gnirehtet-gui: reusable AdbClient with bounded command
 * execution, captured output, device listing and argument validation.
 */

use crate::error::{Cmd, CommandExecutionError, ProcessIoError, ProcessStatusError};
use crate::monitor::AdbMonitor;
use log::*;
use std::ffi::{OsStr, OsString};
use std::io::Read;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::process::{self, ExitStatus, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const TAG: &str = "AdbClient";

/// Android package of the gnirehtet client.
pub const CLIENT_PACKAGE: &str = "com.genymobile.gnirehtet";

/// APK versionCode expected by the pinned gnirehtet v2.5.1 relay.
pub const REQUIRED_APK_VERSION_CODE: &str = "9";

/// Default relay port, shared by the relay and the Android client.
pub const DEFAULT_PORT: u16 = 31416;

const CLIENT_ACTIVITY: &str = "com.genymobile.gnirehtet/.GnirehtetActivity";
const TUNNEL_SOCKET: &str = "localabstract:gnirehtet";

const TIMEOUT_QUERY: Duration = Duration::from_secs(15);
const TIMEOUT_SHELL: Duration = Duration::from_secs(30);
const TIMEOUT_INSTALL: Duration = Duration::from_secs(180);

/// ADB and APK locations.
#[derive(Clone, Debug)]
pub struct AdbConfig {
    pub adb_path: PathBuf,
    pub apk_path: PathBuf,
    pub adbd_addr: SocketAddr,
}

impl Default for AdbConfig {
    /// `ADB` / `GNIREHTET_APK` environment variables, else `adb` on `PATH` and
    /// `gnirehtet.apk` in the working directory (upstream behavior).
    fn default() -> Self {
        Self {
            adb_path: default_adb_path(),
            apk_path: std::env::var_os("GNIREHTET_APK")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("gnirehtet.apk")),
            adbd_addr: SocketAddr::from(([127, 0, 0, 1], 5037)),
        }
    }
}

impl AdbConfig {
    pub fn from_env() -> Self {
        Self::default()
    }
}

pub(crate) fn default_adb_path() -> PathBuf {
    std::env::var_os("ADB")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("adb"))
}

/// Options passed to the Android client when starting the VPN.
#[derive(Clone, Debug)]
pub struct VpnOptions<'a> {
    /// Comma-separated IPv4 DNS servers (client default: 8.8.8.8).
    pub dns_servers: Option<&'a str>,
    /// Comma-separated IPv4 CIDR routes (client default: 0.0.0.0/0).
    pub routes: Option<&'a str>,
    pub port: u16,
}

impl Default for VpnOptions<'_> {
    fn default() -> Self {
        Self::new(DEFAULT_PORT)
    }
}

impl VpnOptions<'_> {
    pub fn new(port: u16) -> Self {
        Self {
            dns_servers: None,
            routes: None,
            port,
        }
    }
}

/// Result of [`AdbClient::ensure_adb`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdbStatus {
    pub path: String,
    pub version: String,
}

/// One row of `adb devices -l`.
///
/// `state` is the raw adb state (`device`, `unauthorized`, `offline`, ...).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdbDevice {
    pub serial: String,
    pub state: String,
    pub model: Option<String>,
    pub product: Option<String>,
}

/// Captured result of a finished command.
struct Output {
    status: ExitStatus,
    stdout: String,
    stderr: String,
}

impl Output {
    fn combined(&self) -> String {
        let mut text = self.stdout.trim().to_string();
        let stderr = self.stderr.trim();
        if !stderr.is_empty() {
            if !text.is_empty() {
                text.push('\n');
            }
            text.push_str(stderr);
        }
        text
    }
}

/// Build a `Command` that never opens a console window on Windows, where a GUI
/// process spawning a console program otherwise flashes a terminal each time.
pub fn background_command(program: impl AsRef<OsStr>) -> process::Command {
    #[allow(unused_mut)]
    let mut command = process::Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}

/// ADB operations used to install, start and stop the gnirehtet client.
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

    /// Install (or replace) the client APK.
    pub fn install(&self, serial: Option<&str>) -> Result<(), CommandExecutionError> {
        info!(target: TAG, "Installing gnirehtet client...");
        if !self.config.apk_path.is_file() {
            return Err(CommandExecutionError::ApkMissing {
                path: self.config.apk_path.display().to_string(),
            });
        }
        let args = vec![
            OsString::from("install"),
            OsString::from("-r"),
            self.config.apk_path.clone().into_os_string(),
        ];
        let output = self.exec(serial, args, TIMEOUT_INSTALL)?;
        // Older adb versions print "Failure [...]" and still exit 0.
        if output.stdout.contains("Failure") {
            return Err(CommandExecutionError::Rejected {
                cmd: self.cmd(serial, &["install", "-r", "<apk>"]),
                output: output.combined(),
            });
        }
        Ok(())
    }

    pub fn uninstall(&self, serial: Option<&str>) -> Result<(), CommandExecutionError> {
        info!(target: TAG, "Uninstalling gnirehtet client...");
        self.exec(
            serial,
            os_args(&["uninstall", CLIENT_PACKAGE]),
            TIMEOUT_SHELL,
        )?;
        Ok(())
    }

    pub fn reinstall(&self, serial: Option<&str>) -> Result<(), CommandExecutionError> {
        self.uninstall(serial)?;
        self.install(serial)
    }

    /// Forward the device's `localabstract:gnirehtet` socket to the relay port.
    pub fn tunnel(&self, serial: Option<&str>, port: u16) -> Result<(), CommandExecutionError> {
        let target = format!("tcp:{port}");
        self.exec(
            serial,
            os_args(&["reverse", TUNNEL_SOCKET, &target]),
            TIMEOUT_QUERY,
        )?;
        Ok(())
    }

    /// Remove the reverse tunnel created by [`Self::tunnel`].
    pub fn remove_tunnel(&self, serial: Option<&str>) -> Result<(), CommandExecutionError> {
        self.exec(
            serial,
            os_args(&["reverse", "--remove", TUNNEL_SOCKET]),
            TIMEOUT_QUERY,
        )?;
        Ok(())
    }

    /// Install the client if needed, set up the tunnel and ask the client to
    /// start its VPN. The user must still accept the VPN prompt on the device.
    pub fn start(
        &self,
        serial: Option<&str>,
        opts: &VpnOptions<'_>,
    ) -> Result<(), CommandExecutionError> {
        opts.dns_servers.map(validate_dns_servers).transpose()?;
        opts.routes.map(validate_routes).transpose()?;
        self.install_if_needed(serial)?;
        self.tunnel(serial, opts.port)?;
        self.start_client(serial, opts)
    }

    /// Install the client unless the expected version is already installed.
    /// Returns whether an installation happened.
    pub fn install_if_needed(&self, serial: Option<&str>) -> Result<bool, CommandExecutionError> {
        if !self.must_install_client(serial)? {
            return Ok(false);
        }
        self.install(serial)?;
        // Give the package manager time to register the intent filters.
        thread::sleep(Duration::from_millis(500));
        Ok(true)
    }

    /// Send the START intent to an installed client (the tunnel must exist).
    pub fn start_client(
        &self,
        serial: Option<&str>,
        opts: &VpnOptions<'_>,
    ) -> Result<(), CommandExecutionError> {
        let dns_servers = opts.dns_servers.map(validate_dns_servers).transpose()?;
        let routes = opts.routes.map(validate_routes).transpose()?;

        info!(target: TAG, "Starting client...");
        let mut args = vec![
            "shell".to_string(),
            "am".into(),
            "start".into(),
            "-a".into(),
            "com.genymobile.gnirehtet.START".into(),
            "-n".into(),
            CLIENT_ACTIVITY.into(),
        ];
        if let Some(dns_servers) = dns_servers {
            args.extend(["--esa".into(), "dnsServers".into(), dns_servers]);
        }
        if let Some(routes) = routes {
            args.extend(["--esa".into(), "routes".into(), routes]);
        }
        self.exec_activity(serial, args)
    }

    /// Ask the client to stop its VPN.
    pub fn stop(&self, serial: Option<&str>) -> Result<(), CommandExecutionError> {
        info!(target: TAG, "Stopping client...");
        let args = [
            "shell",
            "am",
            "start",
            "-a",
            "com.genymobile.gnirehtet.STOP",
            "-n",
            CLIENT_ACTIVITY,
        ];
        self.exec_activity(serial, args.iter().map(|s| s.to_string()).collect())
    }

    pub fn restart(
        &self,
        serial: Option<&str>,
        opts: &VpnOptions<'_>,
    ) -> Result<(), CommandExecutionError> {
        self.stop(serial)?;
        self.start(serial, opts)
    }

    /// `true` when the client is absent or its versionCode differs from
    /// [`REQUIRED_APK_VERSION_CODE`].
    pub fn must_install_client(&self, serial: Option<&str>) -> Result<bool, CommandExecutionError> {
        info!(target: TAG, "Checking gnirehtet client...");
        let output = self.exec(
            serial,
            os_args(&["shell", "dumpsys", "package", CLIENT_PACKAGE]),
            TIMEOUT_SHELL,
        )?;
        Ok(installed_version_code(&output.stdout) != Some(REQUIRED_APK_VERSION_CODE))
    }

    /// Start the client on a background thread (used by `autostart`).
    pub fn async_start(
        &self,
        serial: Option<&str>,
        dns_servers: Option<&str>,
        routes: Option<&str>,
        port: u16,
    ) {
        let client = self.clone();
        let serial = serial.map(String::from);
        let dns_servers = dns_servers.map(String::from);
        let routes = routes.map(String::from);
        thread::spawn(move || {
            let opts = VpnOptions {
                dns_servers: dns_servers.as_deref(),
                routes: routes.as_deref(),
                port,
            };
            if let Err(err) = client.start(serial.as_deref(), &opts) {
                error!(target: TAG, "Cannot start client: {err}");
            }
        });
    }

    pub fn monitor(&self, callback: Box<dyn crate::monitor::AdbMonitorCallback>) -> AdbMonitor {
        AdbMonitor::with_config(
            callback,
            self.config.adb_path.to_string_lossy(),
            self.config.adbd_addr,
        )
    }

    /// Start the adb server if needed and report the adb version.
    pub fn ensure_adb(&self) -> Result<AdbStatus, CommandExecutionError> {
        self.exec(None, os_args(&["start-server"]), TIMEOUT_QUERY)?;
        let output = self.exec(None, os_args(&["version"]), TIMEOUT_QUERY)?;
        let version = output
            .stdout
            .lines()
            .next()
            .unwrap_or("adb")
            .trim()
            .to_string();
        Ok(AdbStatus {
            path: self.config.adb_path.display().to_string(),
            version,
        })
    }

    /// List attached devices (an empty list is not an error).
    pub fn list_devices(&self) -> Result<Vec<AdbDevice>, CommandExecutionError> {
        let output = self.exec(None, os_args(&["devices", "-l"]), TIMEOUT_QUERY)?;
        Ok(parse_adb_devices_l(&output.stdout))
    }

    /// `am start` exits 0 even when the activity could not be started, so the
    /// output must be inspected.
    fn exec_activity(
        &self,
        serial: Option<&str>,
        args: Vec<String>,
    ) -> Result<(), CommandExecutionError> {
        let output = self.exec(
            serial,
            args.iter().map(OsString::from).collect(),
            TIMEOUT_SHELL,
        )?;
        let combined = output.combined();
        if combined
            .lines()
            .any(|line| line.trim_start().starts_with("Error"))
        {
            return Err(CommandExecutionError::Rejected {
                cmd: self.cmd(serial, &args),
                output: combined,
            });
        }
        Ok(())
    }

    fn cmd<S: AsRef<str>>(&self, serial: Option<&str>, args: &[S]) -> Cmd {
        let mut all: Vec<String> = Vec::new();
        if let Some(serial) = serial {
            all.push("-s".into());
            all.push(serial.into());
        }
        all.extend(args.iter().map(|a| a.as_ref().to_string()));
        Cmd::new(self.config.adb_path.to_string_lossy(), all)
    }

    fn exec(
        &self,
        serial: Option<&str>,
        args: Vec<OsString>,
        timeout: Duration,
    ) -> Result<Output, CommandExecutionError> {
        let mut full: Vec<OsString> = Vec::with_capacity(args.len() + 2);
        if let Some(serial) = serial {
            full.push("-s".into());
            full.push(serial.into());
        }
        full.extend(args);

        let display: Vec<String> = full
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let cmd = Cmd::new(self.config.adb_path.to_string_lossy(), display);
        debug!(target: TAG, "Execute: {cmd}");

        let output = run_with_timeout(&self.config.adb_path, &full, timeout, &cmd)?;
        if !output.status.success() {
            return Err(ProcessStatusError::new(cmd, output.status, output.combined()).into());
        }
        Ok(output)
    }
}

/// Run a program, capturing stdout/stderr, killing it after `timeout`.
fn run_with_timeout(
    program: &Path,
    args: &[OsString],
    timeout: Duration,
    cmd: &Cmd,
) -> Result<Output, CommandExecutionError> {
    let mut child = background_command(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| ProcessIoError::new(cmd.clone(), err))?;

    let stdout = read_in_background(child.stdout.take());
    let stderr = read_in_background(child.stderr.take());

    let deadline = Instant::now() + timeout;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(CommandExecutionError::Timeout {
                cmd: cmd.clone(),
                after: timeout,
            });
        }
        thread::sleep(Duration::from_millis(20));
    };

    // `adb` may fork its server daemon, which can inherit our pipes and keep
    // them open indefinitely: never block on EOF once the process has exited.
    let grace = Instant::now() + Duration::from_millis(300);
    Ok(Output {
        status,
        stdout: collect_until(&stdout, grace),
        stderr: collect_until(&stderr, grace),
    })
}

fn read_in_background<R: Read + Send + 'static>(pipe: Option<R>) -> mpsc::Receiver<Vec<u8>> {
    let (tx, rx) = mpsc::channel();
    if let Some(mut pipe) = pipe {
        thread::spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match pipe.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                }
            }
        });
    }
    rx
}

/// Gather chunks until the writer closes the pipe or `deadline` passes.
fn collect_until(rx: &mpsc::Receiver<Vec<u8>>, deadline: Instant) -> String {
    let mut bytes = Vec::new();
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match rx.recv_timeout(remaining) {
            Ok(chunk) => bytes.extend_from_slice(&chunk),
            Err(_) => break,
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn os_args(args: &[&str]) -> Vec<OsString> {
    args.iter().map(OsString::from).collect()
}

/// Extract `versionCode=N` from `dumpsys package` output.
fn installed_version_code(dumpsys: &str) -> Option<&str> {
    dumpsys.lines().find_map(|line| {
        let rest = line.trim_start().strip_prefix("versionCode=")?;
        rest.split_whitespace().next()
    })
}

/// Validate a comma-separated list of IPv4 DNS servers.
///
/// Values end up in a device-side shell command line (`adb shell am start`),
/// so anything but plain addresses must be refused.
pub fn validate_dns_servers(value: &str) -> Result<String, CommandExecutionError> {
    let invalid = |reason: String| CommandExecutionError::InvalidArgument {
        name: "DNS servers",
        reason,
    };
    let servers: Vec<&str> = value
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if servers.is_empty() {
        return Err(invalid("expected at least one IPv4 address".into()));
    }
    for server in &servers {
        server
            .parse::<Ipv4Addr>()
            .map_err(|_| invalid(format!("'{server}' is not an IPv4 address")))?;
    }
    Ok(servers.join(","))
}

/// Validate a comma-separated list of IPv4 CIDR routes (e.g. `10.0.0.0/8`).
pub fn validate_routes(value: &str) -> Result<String, CommandExecutionError> {
    let invalid = |reason: String| CommandExecutionError::InvalidArgument {
        name: "routes",
        reason,
    };
    let routes: Vec<&str> = value
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if routes.is_empty() {
        return Err(invalid("expected at least one route".into()));
    }
    for route in &routes {
        let (addr, prefix) = route
            .split_once('/')
            .ok_or_else(|| invalid(format!("'{route}' is not in CIDR form (a.b.c.d/n)")))?;
        addr.parse::<Ipv4Addr>()
            .map_err(|_| invalid(format!("'{addr}' is not an IPv4 address")))?;
        match prefix.parse::<u8>() {
            Ok(bits) if bits <= 32 => {}
            _ => return Err(invalid(format!("'{prefix}' is not a prefix length (0-32)"))),
        }
    }
    Ok(routes.join(","))
}

/// Parse the output of `adb devices` or `adb devices -l`.
pub fn parse_adb_devices_l(stdout: &str) -> Vec<AdbDevice> {
    stdout
        .lines()
        .map(str::trim)
        .filter(|line| {
            !line.is_empty() && !line.starts_with("List of devices") && !line.starts_with('*')
        })
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let serial = parts.next()?;
            let state = parts.next()?;
            // "no permissions (user in plugdev group; ...)" spans several tokens.
            let state = if state == "no" && parts.next() == Some("permissions") {
                "no permissions"
            } else {
                state
            };
            Some(AdbDevice {
                serial: serial.to_string(),
                state: state.to_string(),
                model: device_property(line, "model:"),
                product: device_property(line, "product:"),
            })
        })
        .collect()
}

fn device_property(line: &str, key: &str) -> Option<String> {
    line.split_whitespace()
        .find_map(|token| token.strip_prefix(key))
        .filter(|value| !value.is_empty())
        .map(|value| value.replace('_', " "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_devices_empty() {
        let out = "List of devices attached\n\n";
        assert!(parse_adb_devices_l(out).is_empty());
    }

    #[test]
    fn parse_devices_mixed_states() {
        let out = "\
* daemon not running; starting now at tcp:5037
* daemon started successfully
List of devices attached
emulator-5554          device product:sdk_gphone model:sdk_gphone_x86 device:generic_x86
0123456789ABCDEF       unauthorized
deadbeef               offline transport_id:1
";
        let devices = parse_adb_devices_l(out);
        assert_eq!(devices.len(), 3);
        assert_eq!(devices[0].serial, "emulator-5554");
        assert_eq!(devices[0].state, "device");
        assert_eq!(devices[0].model.as_deref(), Some("sdk gphone x86"));
        assert_eq!(devices[0].product.as_deref(), Some("sdk gphone"));
        assert_eq!(devices[1].state, "unauthorized");
        assert!(devices[1].model.is_none());
        assert_eq!(devices[2].state, "offline");
    }

    #[test]
    fn parse_devices_without_properties() {
        let devices = parse_adb_devices_l("List of devices attached\nABC\tdevice\n");
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].serial, "ABC");
        assert_eq!(devices[0].state, "device");
    }

    #[test]
    fn parse_devices_no_permissions() {
        let out = "List of devices attached\n\
                   0123 no permissions (missing udev rules? user is in the plugdev group); see [http://developer.android.com/tools/device.html] usb:1-1\n";
        let devices = parse_adb_devices_l(out);
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].state, "no permissions");
    }

    #[test]
    fn version_code_is_extracted() {
        let dumpsys = "Packages:\n  Package [com.genymobile.gnirehtet] (abc):\n    versionCode=9 minSdk=21 targetSdk=33\n";
        assert_eq!(installed_version_code(dumpsys), Some("9"));
        assert_eq!(installed_version_code("Unable to find package"), None);
    }

    #[test]
    fn dns_servers_are_validated() {
        assert_eq!(
            validate_dns_servers("8.8.8.8, 1.1.1.1").unwrap(),
            "8.8.8.8,1.1.1.1"
        );
        assert!(validate_dns_servers("8.8.8.8;reboot").is_err());
        assert!(validate_dns_servers("8.8.8.8 1.1.1.1").is_err());
        assert!(validate_dns_servers("::1").is_err());
        assert!(validate_dns_servers(" , ").is_err());
    }

    #[test]
    fn routes_are_validated() {
        assert_eq!(
            validate_routes("10.0.0.0/8,192.168.1.0/24").unwrap(),
            "10.0.0.0/8,192.168.1.0/24"
        );
        assert!(validate_routes("10.0.0.0").is_err());
        assert!(validate_routes("10.0.0.0/33").is_err());
        assert!(validate_routes("0.0.0.0/0$(id)").is_err());
    }

    #[test]
    fn install_missing_apk_returns_apk_missing() {
        let client = AdbClient::new(AdbConfig {
            apk_path: PathBuf::from("/no/such/gnirehtet.apk"),
            ..AdbConfig::default()
        });
        let err = client.install(None).expect_err("missing apk");
        assert!(matches!(err, CommandExecutionError::ApkMissing { .. }));
        assert_eq!(err.code(), Some("APK_MISSING"));
    }

    #[test]
    fn start_rejects_invalid_options_before_running_adb() {
        let client = AdbClient::new(AdbConfig {
            adb_path: PathBuf::from("/no/such/adb"),
            ..AdbConfig::default()
        });
        let opts = VpnOptions {
            dns_servers: Some("8.8.8.8;reboot"),
            ..VpnOptions::default()
        };
        let err = client.start(None, &opts).expect_err("invalid dns");
        assert_eq!(err.code(), Some("INVALID_ARGUMENT"));
    }

    #[test]
    fn missing_adb_binary_is_reported() {
        let client = AdbClient::new(AdbConfig {
            adb_path: PathBuf::from("/no/such/adb"),
            ..AdbConfig::default()
        });
        let err = client.list_devices().expect_err("no adb");
        assert_eq!(err.code(), Some("ADB_MISSING"));
    }

    #[cfg(unix)]
    #[test]
    fn slow_command_times_out() {
        let cmd = Cmd::new("sleep", vec!["5"]);
        let started = Instant::now();
        let err = run_with_timeout(
            Path::new("sleep"),
            &[OsString::from("5")],
            Duration::from_millis(200),
            &cmd,
        )
        .err()
        .expect("timeout");
        assert!(matches!(err, CommandExecutionError::Timeout { .. }));
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[cfg(unix)]
    #[test]
    fn exited_command_does_not_wait_for_inherited_pipes() {
        // The background `sleep` keeps stdout open after `sh` exits, like a
        // forked adb server daemon would.
        let cmd = Cmd::new("sh", vec!["-c", "echo ok; sleep 5 &"]);
        let started = Instant::now();
        let output = run_with_timeout(
            Path::new("sh"),
            &[OsString::from("-c"), OsString::from("echo ok; sleep 5 &")],
            Duration::from_secs(10),
            &cmd,
        )
        .expect("runs");
        assert!(output.status.success());
        assert_eq!(output.stdout.trim(), "ok");
        assert!(started.elapsed() < Duration::from_secs(3));
    }
}
