//! Supervision of the external `gnirehtet relay` process.

use crate::error::ControllerError;
use gnirehtet_adb::background_command;
use log::*;
use std::io::{self, Read};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, ChildStdout, ExitStatus, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const TAG: &str = "RelaySupervisor";

/// How long the relay may take to bind its port after being spawned.
const START_TIMEOUT: Duration = Duration::from_secs(5);
const START_POLL_INTERVAL: Duration = Duration::from_millis(25);

/// Output pipes of a started relay. The caller must keep draining both,
/// otherwise the relay blocks once a pipe buffer fills up.
#[derive(Debug)]
pub struct RelayStdio {
    pub stdout: ChildStdout,
    pub stderr: ChildStderr,
}

/// Check whether `127.0.0.1:port`, the relay listen address, is free.
pub fn probe_relay_port(port: u16) -> Result<(), ControllerError> {
    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::AddrInUse => {
            Err(ControllerError::PortInUse { port })
        }
        Err(err) => Err(ControllerError::Io(err)),
    }
}

struct RunningRelay {
    child: Child,
    port: u16,
}

/// Owns at most one relay process and never touches relays it did not start.
///
/// The relay is killed when the supervisor is dropped.
pub struct RelaySupervisor {
    binary: PathBuf,
    relay: Option<RunningRelay>,
}

impl RelaySupervisor {
    pub fn new(binary: impl Into<PathBuf>) -> Self {
        Self {
            binary: binary.into(),
            relay: None,
        }
    }

    /// The relay executable (absolute path, or a name resolved via `PATH`).
    pub fn binary(&self) -> &Path {
        &self.binary
    }

    pub fn is_running(&self) -> bool {
        self.relay.is_some()
    }

    pub fn pid(&self) -> Option<u32> {
        self.relay.as_ref().map(|r| r.child.id())
    }

    pub fn port(&self) -> Option<u16> {
        self.relay.as_ref().map(|r| r.port)
    }

    /// Spawn `gnirehtet relay -p <port>` and wait until it listens.
    ///
    /// Fails without spawning anything if the port is already taken, so a
    /// foreign relay is never mistaken for ours.
    pub fn start(&mut self, port: u16) -> Result<RelayStdio, ControllerError> {
        self.poll()?;
        if let Some(relay) = &self.relay {
            return Err(ControllerError::RelayAlreadyRunning { port: relay.port });
        }
        probe_relay_port(port)?;

        let has_dir = self
            .binary
            .parent()
            .is_some_and(|dir| !dir.as_os_str().is_empty());
        if has_dir && !self.binary.is_file() {
            return Err(ControllerError::RelayNotFound {
                path: self.binary.display().to_string(),
            });
        }

        info!(target: TAG, "Starting {} relay -p {}", self.binary.display(), port);
        let mut child = background_command(&self.binary)
            .args(["relay", "-p", &port.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|source| ControllerError::RelaySpawn {
                path: self.binary.display().to_string(),
                source,
            })?;

        if let Err(err) = wait_until_listening(&mut child, port) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(err);
        }

        let (stdout, stderr) = match (child.stdout.take(), child.stderr.take()) {
            (Some(stdout), Some(stderr)) => (stdout, stderr),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(ControllerError::RelayStartFailed {
                    port,
                    detail: "output pipes unavailable".into(),
                });
            }
        };

        info!(target: TAG, "Relay running (pid {}, port {})", child.id(), port);
        self.relay = Some(RunningRelay { child, port });
        Ok(RelayStdio { stdout, stderr })
    }

    /// Kill the relay if one is running. Returns whether one was running.
    pub fn stop(&mut self) -> bool {
        let Some(mut relay) = self.relay.take() else {
            return false;
        };
        info!(target: TAG, "Stopping relay (pid {})", relay.child.id());
        let _ = relay.child.kill();
        let _ = relay.child.wait();
        true
    }

    /// Non-blocking check: returns the exit status if the relay has exited
    /// since the last call, and forgets it.
    pub fn poll(&mut self) -> io::Result<Option<ExitStatus>> {
        let Some(relay) = self.relay.as_mut() else {
            return Ok(None);
        };
        match relay.child.try_wait()? {
            None => Ok(None),
            Some(status) => {
                warn!(target: TAG, "Relay exited unexpectedly ({status})");
                self.relay = None;
                Ok(Some(status))
            }
        }
    }
}

impl Drop for RelaySupervisor {
    fn drop(&mut self) {
        self.stop();
    }
}

fn wait_until_listening(child: &mut Child, port: u16) -> Result<(), ControllerError> {
    let deadline = Instant::now() + START_TIMEOUT;
    loop {
        if let Some(status) = child.try_wait()? {
            let output = read_exited_output(child);
            let detail = output
                .lines()
                .map(str::trim)
                .rfind(|line| !line.is_empty())
                .map(|line| format!("{line} ({status})"))
                .unwrap_or_else(|| format!("exited immediately ({status})"));
            return Err(ControllerError::RelayStartFailed { port, detail });
        }
        match probe_relay_port(port) {
            Err(ControllerError::PortInUse { .. }) => return Ok(()),
            Err(err) => return Err(err),
            Ok(()) => {}
        }
        if Instant::now() >= deadline {
            return Err(ControllerError::RelayStartFailed {
                port,
                detail: format!("not listening after {}s", START_TIMEOUT.as_secs()),
            });
        }
        thread::sleep(START_POLL_INTERVAL);
    }
}

/// Read what an exited relay printed, without blocking if a pipe stays open.
fn read_exited_output(child: &mut Child) -> String {
    let (tx, rx) = mpsc::channel();
    if let Some(mut stdout) = child.stdout.take() {
        let tx = tx.clone();
        thread::spawn(move || {
            let mut text = String::new();
            let _ = stdout.read_to_string(&mut text);
            let _ = tx.send(text);
        });
    }
    if let Some(mut stderr) = child.stderr.take() {
        let tx = tx.clone();
        thread::spawn(move || {
            let mut text = String::new();
            let _ = stderr.read_to_string(&mut text);
            let _ = tx.send(text);
        });
    }
    drop(tx);
    let deadline = Instant::now() + Duration::from_millis(300);
    let mut output = String::new();
    while let Ok(text) = rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
        output.push_str(&text);
    }
    output
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::fs::PermissionsExt;

    /// Tests bind ephemeral ports and exec freshly written scripts; running
    /// them concurrently races on both (port reuse, ETXTBSY), so serialize.
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn serial() -> std::sync::MutexGuard<'static, ()> {
        SERIAL
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// A fake relay: a shell script (the port is bound by the test itself,
    /// standing in for the real relay's listener).
    fn fake_relay(name: &str, script: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gnirehtet-controller-{}-{}",
            name,
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("gnirehtet");
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(file, "#!/bin/sh\n{script}").unwrap();
        drop(file);
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    fn free_port() -> u16 {
        TcpListener::bind(("127.0.0.1", 0))
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    /// Bind `port` shortly after the relay is spawned, as the real relay would.
    fn listen_soon(port: u16, hold: Duration) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            let listener = TcpListener::bind(("127.0.0.1", port)).unwrap();
            thread::sleep(hold);
            drop(listener);
        })
    }

    #[test]
    fn probe_detects_port_in_use() {
        let _serial = serial();
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let err = probe_relay_port(port).expect_err("port busy");
        assert!(matches!(err, ControllerError::PortInUse { port: p } if p == port));
        assert_eq!(err.code(), "PORT_IN_USE");
        drop(listener);
        probe_relay_port(port).expect("port free again");
    }

    #[test]
    fn start_refuses_busy_port_without_spawning() {
        let _serial = serial();
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let mut supervisor = RelaySupervisor::new("/definitely/not/spawned");
        let err = supervisor.start(port).expect_err("busy");
        assert!(matches!(err, ControllerError::PortInUse { .. }));
        assert!(!supervisor.is_running());
    }

    #[test]
    fn missing_binary_is_reported() {
        let _serial = serial();
        let mut supervisor = RelaySupervisor::new("/no/such/dir/gnirehtet");
        let err = supervisor.start(free_port()).expect_err("missing");
        assert_eq!(err.code(), "RELAY_NOT_FOUND");
    }

    #[test]
    fn start_waits_for_listener_and_exposes_output() {
        let _serial = serial();
        let binary = fake_relay(
            "start",
            "echo 'relay-out'\necho 'relay-err' >&2\nexec sleep 30",
        );
        let port = free_port();
        let listener = listen_soon(port, Duration::from_secs(2));

        let mut supervisor = RelaySupervisor::new(&binary);
        let stdio = supervisor.start(port).expect("start");
        assert!(supervisor.is_running());
        assert_eq!(supervisor.port(), Some(port));
        assert!(supervisor.pid().is_some());

        let mut line = String::new();
        BufReader::new(stdio.stdout).read_line(&mut line).unwrap();
        assert_eq!(line.trim(), "relay-out");
        line.clear();
        BufReader::new(stdio.stderr).read_line(&mut line).unwrap();
        assert_eq!(line.trim(), "relay-err");

        let err = supervisor.start(port).expect_err("already running");
        assert_eq!(err.code(), "RELAY_ALREADY_RUNNING");

        assert!(supervisor.stop());
        assert!(!supervisor.is_running());
        assert!(!supervisor.stop());
        listener.join().unwrap();
    }

    #[test]
    fn immediate_exit_reports_relay_output() {
        let _serial = serial();
        let binary = fake_relay("exit", "echo 'Address already in use' >&2\nexit 3");
        let mut supervisor = RelaySupervisor::new(&binary);
        let err = supervisor.start(free_port()).expect_err("exits");
        assert_eq!(err.code(), "RELAY_START_FAILED");
        assert!(err.to_string().contains("Address already in use"), "{err}");
        assert!(!supervisor.is_running());
    }

    #[test]
    fn poll_reports_crash_once() {
        let _serial = serial();
        let binary = fake_relay("crash", "sleep 0.5\nexit 42");
        let port = free_port();
        let listener = listen_soon(port, Duration::from_millis(200));

        let mut supervisor = RelaySupervisor::new(&binary);
        let _stdio = supervisor.start(port).expect("start");
        thread::sleep(Duration::from_millis(900));
        let status = supervisor.poll().unwrap().expect("exited");
        assert_eq!(status.code(), Some(42));
        assert!(!supervisor.is_running());
        assert!(supervisor.poll().unwrap().is_none());
        listener.join().unwrap();
    }

    #[test]
    fn drop_kills_relay() {
        let _serial = serial();
        let binary = fake_relay("drop", "exec sleep 30");
        let port = free_port();
        let listener = listen_soon(port, Duration::from_millis(300));

        let mut supervisor = RelaySupervisor::new(&binary);
        let _stdio = supervisor.start(port).expect("start");
        let pid = supervisor.pid().unwrap();
        drop(supervisor);
        let alive = std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success();
        assert!(!alive, "relay {pid} survived drop");
        listener.join().unwrap();
    }

    /// End-to-end check against a real relay binary, when one is provided.
    #[test]
    fn real_relay_starts_and_stops() {
        let _serial = serial();
        let Some(binary) = std::env::var_os("GNIREHTET_TEST_RELAY") else {
            return;
        };
        let port = free_port();
        let mut supervisor = RelaySupervisor::new(PathBuf::from(binary));
        let stdio = supervisor.start(port).expect("real relay starts");
        let started = BufReader::new(stdio.stdout)
            .lines()
            .take(5)
            .map_while(Result::ok)
            .any(|line| line.contains("Relay server started"));
        assert!(started);
        let mut other = RelaySupervisor::new(supervisor.binary().to_path_buf());
        assert_eq!(other.start(port).expect_err("busy").code(), "PORT_IN_USE");
        assert!(supervisor.stop());
        probe_relay_port(port).expect("port released");
    }
}
