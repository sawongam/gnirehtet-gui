//! Session-scoped controller: ADB verbs + owned relay child.

use crate::error::ControllerError;
use crate::relay::{
    spawn_relay, spawn_relay_with_stdio, RelayProcess, RelayStdio, DEFAULT_RELAY_PORT,
};
use gnirehtet_adb::{AdbClient, AdbConfig, AdbDevice, AdbStatus, VpnOptions};
use log::*;
use std::path::{Path, PathBuf};

const TAG: &str = "SessionController";

/// Configuration for the controller (adb + stock gnirehtet binary path).
#[derive(Clone, Debug)]
pub struct ControllerConfig {
    pub adb: AdbConfig,
    /// Stock `gnirehtet` binary (PATH name or absolute). Default: `gnirehtet`.
    pub gnirehtet_path: PathBuf,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            adb: AdbConfig::from_env(),
            gnirehtet_path: default_gnirehtet_path(),
        }
    }
}

fn default_gnirehtet_path() -> PathBuf {
    if let Some(p) = std::env::var_os("GNIREHTET_BIN") {
        PathBuf::from(p)
    } else {
        PathBuf::from("gnirehtet")
    }
}

/// Optional overrides for `run` / `start` (DNS, routes, port).
#[derive(Clone, Debug, Default)]
pub struct RunOptions<'a> {
    pub dns_servers: Option<&'a str>,
    pub routes: Option<&'a str>,
    pub port: Option<u16>,
}

/// Session-scoped orchestrator (MVP sidecar — no relaylib).
pub struct SessionController {
    adb: AdbClient,
    gnirehtet_path: PathBuf,
    /// Port chosen for this session (set by start_relay / run / start with port).
    session_port: Option<u16>,
    owned_relay: Option<RelayProcess>,
}

impl SessionController {
    pub fn new(config: ControllerConfig) -> Self {
        Self {
            adb: AdbClient::new(config.adb),
            gnirehtet_path: config.gnirehtet_path,
            session_port: None,
            owned_relay: None,
        }
    }

    pub fn from_env() -> Self {
        Self::new(ControllerConfig::default())
    }

    pub fn adb(&self) -> &AdbClient {
        &self.adb
    }

    pub fn gnirehtet_path(&self) -> &Path {
        &self.gnirehtet_path
    }

    pub fn session_port(&self) -> Option<u16> {
        self.session_port
    }

    pub fn owns_relay(&self) -> bool {
        self.owned_relay.is_some()
    }

    pub fn owned_relay_pid(&self) -> Option<u32> {
        self.owned_relay.as_ref().map(|r| r.pid())
    }

    pub fn ensure_adb(&self) -> Result<AdbStatus, ControllerError> {
        Ok(self.adb.ensure_adb()?)
    }

    pub fn list_devices(&self) -> Result<Vec<AdbDevice>, ControllerError> {
        Ok(self.adb.list_devices()?)
    }

    pub fn install(&self, serial: Option<&str>) -> Result<(), ControllerError> {
        Ok(self.adb.install(serial)?)
    }

    pub fn start(
        &mut self,
        serial: Option<&str>,
        opts: &VpnOptions<'_>,
    ) -> Result<(), ControllerError> {
        // Agree session port with tunnel/client (no hot-swap of an owned relay).
        self.remember_port(opts.port)?;
        Ok(self.adb.start(serial, opts)?)
    }

    pub fn stop(&self, serial: Option<&str>) -> Result<(), ControllerError> {
        Ok(self.adb.stop(serial)?)
    }

    pub fn reset_tunnel(
        &mut self,
        serial: Option<&str>,
        port: Option<u16>,
    ) -> Result<(), ControllerError> {
        let p = port.unwrap_or_else(|| self.effective_port());
        self.remember_port(p)?;
        Ok(self.adb.tunnel(serial, p)?)
    }

    /// Spawn stock `gnirehtet relay -p <port>` as a session-owned child (stdio **inherited**).
    ///
    /// Port policy: explicit `port` > existing session port > `31416`.
    /// Refuses mid-session listen-port hot-swap while a relay is owned.
    /// Probes bind first — `PORT_IN_USE` means no ownership claimed.
    ///
    /// For piped stdout/stderr + Desktop `LogLine` streaming, use
    /// [`Self::start_relay_with_stdio`] instead.
    pub fn start_relay(&mut self, port: Option<u16>) -> Result<&RelayProcess, ControllerError> {
        let listen = self.begin_relay_start(port)?;
        let relay = spawn_relay(&self.gnirehtet_path, listen)?;
        self.finish_relay_start(relay);
        Ok(self.owned_relay.as_ref().unwrap())
    }

    /// Like [`Self::start_relay`], but pipes stdout/stderr and returns readers.
    ///
    /// Session still tracks ownership/pid/port for `stop_relay` /
    /// `clear_owned_relay` / `Drop`.
    ///
    /// **Concurrency (Desktop):** call under `Mutex<SessionController>`, move the
    /// returned [`RelayStdio`] out of the lock, then pump lines on Desktop threads
    /// / channels. Desktop owns the single `LogLine` pump — this method does
    /// **not** spawn reader threads.
    pub fn start_relay_with_stdio(
        &mut self,
        port: Option<u16>,
    ) -> Result<RelayStdio, ControllerError> {
        let listen = self.begin_relay_start(port)?;
        let (relay, stdio) = spawn_relay_with_stdio(&self.gnirehtet_path, listen)?;
        self.finish_relay_start(relay);
        Ok(stdio)
    }

    /// Shared pre-spawn checks: reap, refuse double-start / hot-swap, resolve port.
    fn begin_relay_start(&mut self, port: Option<u16>) -> Result<u16, ControllerError> {
        self.reap_if_exited();

        if self.owned_relay.is_some() {
            let running_port = self.owned_relay.as_ref().unwrap().port();
            let requested = port.unwrap_or(running_port);
            if requested != running_port {
                return Err(ControllerError::RelayAlreadyRunning {
                    port: running_port,
                });
            }
            return Err(ControllerError::RelayAlreadyRunning {
                port: running_port,
            });
        }

        Ok(port
            .or(self.session_port)
            .unwrap_or(DEFAULT_RELAY_PORT))
    }

    fn finish_relay_start(&mut self, relay: RelayProcess) {
        let listen = relay.port();
        // Freeze session port only after ownership is claimed (bind OK).
        self.session_port = Some(listen);
        info!(
            target: TAG,
            "Owned relay started pid={} port={}",
            relay.pid(),
            relay.port()
        );
        self.owned_relay = Some(relay);
    }

    /// Kill **only** a relay this session started.
    pub fn stop_relay(&mut self) -> Result<(), ControllerError> {
        let Some(relay) = self.owned_relay.take() else {
            return Err(ControllerError::NoOwnedRelay);
        };
        info!(
            target: TAG,
            "Stopping session-owned relay pid={} port={}",
            relay.pid(),
            relay.port()
        );
        let _ = relay.kill_and_wait();
        Ok(())
    }

    /// Quit / Drop helper: clear owned relay so :31416 (or session port) can free.
    /// Never kills a foreign process. Idempotent.
    pub fn clear_owned_relay(&mut self) {
        if let Some(relay) = self.owned_relay.take() {
            info!(
                target: TAG,
                "clear_owned_relay: killing pid={} port={}",
                relay.pid(),
                relay.port()
            );
            let _ = relay.kill_and_wait();
        }
    }

    /// One-click ≈ upstream `run`: start relay child, then adb install/tunnel/start.
    ///
    /// Child-based — does **not** call in-process relaylib.
    /// Uses inherited-stdio [`Self::start_relay`] (not the piped LogLine path).
    pub fn run(
        &mut self,
        serial: Option<&str>,
        opts: &RunOptions<'_>,
    ) -> Result<(), ControllerError> {
        let port = opts.port.unwrap_or_else(|| self.effective_port());
        self.remember_port(port)?;

        if self.owned_relay.is_none() {
            self.start_relay(Some(port))?;
        } else if self.owned_relay.as_ref().map(|r| r.port()) != Some(port) {
            return Err(ControllerError::RelayAlreadyRunning {
                port: self.owned_relay.as_ref().unwrap().port(),
            });
        }

        let vpn = VpnOptions {
            dns_servers: opts.dns_servers,
            routes: opts.routes,
            port,
        };
        // AdbClient::start already tunnels + install-if-needed.
        self.adb.start(serial, &vpn)?;
        Ok(())
    }

    fn effective_port(&self) -> u16 {
        self.session_port.unwrap_or(DEFAULT_RELAY_PORT)
    }

    fn remember_port(&mut self, port: u16) -> Result<(), ControllerError> {
        if let Some(owned) = self.owned_relay.as_ref() {
            if owned.port() != port {
                return Err(ControllerError::RelayAlreadyRunning {
                    port: owned.port(),
                });
            }
        }
        if self.session_port.is_none() {
            self.session_port = Some(port);
        } else if self.session_port != Some(port) && self.owned_relay.is_some() {
            return Err(ControllerError::RelayAlreadyRunning {
                port: self.session_port.unwrap(),
            });
        } else {
            self.session_port = Some(port);
        }
        Ok(())
    }

    /// Non-blocking poll of the session-owned relay (Desktop ≤3s crash poller).
    ///
    /// - `Ok(None)` — no owned relay, or child still running
    /// - `Ok(Some(status))` — child exited; **ownership cleared** (map to `RELAY_CRASHED`)
    /// - `Err(_)` — `try_wait` I/O failure; ownership left unchanged
    ///
    /// Safe to call under `Mutex<SessionController>` briefly; does not touch stdio pipes
    /// (Desktop keeps those on its LogLine pump threads).
    pub fn poll_owned_relay(
        &mut self,
    ) -> std::io::Result<Option<std::process::ExitStatus>> {
        let Some(relay) = self.owned_relay.as_mut() else {
            return Ok(None);
        };
        match relay.try_wait()? {
            None => Ok(None),
            Some(status) => {
                warn!(
                    target: TAG,
                    "Owned relay exited (status={}); clearing ownership",
                    status
                );
                self.owned_relay = None;
                Ok(Some(status))
            }
        }
    }

    fn reap_if_exited(&mut self) {
        let _ = self.poll_owned_relay();
    }
}

impl Drop for SessionController {
    fn drop(&mut self) {
        self.clear_owned_relay();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::probe_relay_port;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn default_port_is_31416() {
        let c = SessionController::from_env();
        assert_eq!(c.effective_port(), DEFAULT_RELAY_PORT);
        assert!(!c.owns_relay());
    }

    #[test]
    fn start_relay_port_in_use_does_not_claim_ownership() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        // Confirm probe semantics used by start_relay.
        assert!(matches!(
            probe_relay_port(port),
            Err(ControllerError::PortInUse { .. })
        ));

        let mut c = SessionController::new(ControllerConfig {
            adb: AdbConfig::default(),
            gnirehtet_path: PathBuf::from("gnirehtet"),
        });
        let err = c.start_relay(Some(port)).expect_err("busy port");
        assert!(matches!(err, ControllerError::PortInUse { .. }));
        assert_eq!(err.ux_code(), Some("PORT_IN_USE"));
        assert!(!c.owns_relay());
        // Session port may be recorded before spawn; ownership must stay false.
        drop(listener);
    }

    #[test]
    fn start_relay_with_stdio_port_in_use_does_not_claim_ownership() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let mut c = SessionController::new(ControllerConfig {
            adb: AdbConfig::default(),
            gnirehtet_path: PathBuf::from("gnirehtet"),
        });
        let err = c
            .start_relay_with_stdio(Some(port))
            .expect_err("busy port");
        assert!(matches!(err, ControllerError::PortInUse { .. }));
        assert!(!c.owns_relay());
        drop(listener);
    }

    #[test]
    fn clear_owned_relay_idempotent_without_child() {
        let mut c = SessionController::from_env();
        c.clear_owned_relay();
        c.clear_owned_relay();
        assert!(c.stop_relay().is_err());
    }

    #[test]
    fn poll_owned_relay_none_without_child() {
        let mut c = SessionController::from_env();
        assert!(c.poll_owned_relay().unwrap().is_none());
    }

    #[test]
    fn poll_owned_relay_clears_on_exit() {
        let dir = std::env::temp_dir().join(format!(
            "gnirehtet-controller-poll-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let stub = dir.join("fake-gnirehtet");
        {
            let mut f = std::fs::File::create(&stub).unwrap();
            writeln!(f, "#!/bin/sh\necho hi\nsleep 1\nexit 42\n").unwrap();
        }
        let mut perms = std::fs::metadata(&stub).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&stub, perms).unwrap();

        let port = free_ephemeral_port();
        let mut c = SessionController::new(ControllerConfig {
            adb: AdbConfig::default(),
            gnirehtet_path: stub,
        });
        let _stdio = c.start_relay_with_stdio(Some(port)).expect("start");
        assert!(c.owns_relay());
        // Child exits immediately after printing; wait briefly then poll.
        std::thread::sleep(std::time::Duration::from_millis(1200));
        let status = c
            .poll_owned_relay()
            .expect("poll ok")
            .expect("should have exited");
        assert!(!c.owns_relay());
        assert_eq!(status.code(), Some(42));
        assert!(c.poll_owned_relay().unwrap().is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn free_ephemeral_port() -> u16 {
        // Bind+drop can race with parallel tests; retry until probe says free.
        for _ in 0..32 {
            let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
            let port = listener.local_addr().unwrap().port();
            drop(listener);
            if crate::relay::probe_relay_port(port).is_ok() {
                return port;
            }
        }
        panic!("could not find a free ephemeral port");
    }

    #[test]
    fn start_relay_with_stdio_owns_and_stop_clears() {
        let dir = std::env::temp_dir().join(format!(
            "gnirehtet-controller-session-stdio-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let stub = dir.join("fake-gnirehtet");
        {
            let mut f = std::fs::File::create(&stub).unwrap();
            writeln!(
                f,
                "#!/bin/sh\necho 'session-out'\necho 'session-err' >&2\nexec sleep 30\n"
            )
            .unwrap();
        }
        let mut perms = std::fs::metadata(&stub).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&stub, perms).unwrap();

        let port = free_ephemeral_port();

        let mut c = SessionController::new(ControllerConfig {
            adb: AdbConfig::default(),
            gnirehtet_path: stub,
        });
        let stdio = c.start_relay_with_stdio(Some(port)).expect("stdio start");
        assert!(c.owns_relay());
        assert_eq!(c.session_port(), Some(port));
        assert!(c.owned_relay_pid().is_some());

        let mut out = BufReader::new(stdio.stdout);
        let mut line = String::new();
        out.read_line(&mut line).unwrap();
        assert_eq!(line.trim(), "session-out");

        c.stop_relay().expect("stop owned");
        assert!(!c.owns_relay());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
