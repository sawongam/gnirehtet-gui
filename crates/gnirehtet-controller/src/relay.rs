//! Session-owned stock `gnirehtet relay` child process.

use crate::error::ControllerError;
use log::*;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

const TAG: &str = "RelayProcess";

/// Upstream / CLI default relay listen port.
pub const DEFAULT_RELAY_PORT: u16 = 31416;

/// Handle for a relay child this session started.
#[derive(Debug)]
pub struct RelayProcess {
    child: Child,
    port: u16,
    path: PathBuf,
}

impl RelayProcess {
    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn child_mut(&mut self) -> &mut Child {
        &mut self.child
    }

    /// Non-blocking poll: `Ok(None)` still running; `Ok(Some(status))` exited.
    pub fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        self.child.try_wait()
    }

    /// Kill and wait (session teardown / quit helper).
    pub fn kill_and_wait(mut self) -> std::io::Result<()> {
        let _ = self.child.kill();
        let _ = self.child.wait();
        Ok(())
    }
}

/// Probe whether `127.0.0.1:port` can be bound (upstream relay listen address).
///
/// On `AddrInUse`, returns `ControllerError::PortInUse` so callers must **not**
/// claim session ownership (PHASE0_ACCEPTANCE P0-P2).
pub fn probe_relay_port(port: u16) -> Result<(), ControllerError> {
    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(listener) => {
            drop(listener);
            Ok(())
        }
        Err(err) if err.kind() == std::io::ErrorKind::AddrInUse => {
            Err(ControllerError::PortInUse { port })
        }
        Err(err) => Err(ControllerError::Io(err)),
    }
}

/// Spawn `gnirehtet relay -p <port>` after a successful free-port probe.
///
/// Does not claim ownership for the caller — the returned `RelayProcess` is the
/// ownership token. If the child exits immediately, returns `RelayStartFailed`
/// and does not leave a live child.
pub fn spawn_relay(gnirehtet_path: &Path, port: u16) -> Result<RelayProcess, ControllerError> {
    probe_relay_port(port)?;

    if gnirehtet_path.is_absolute() && !gnirehtet_path.is_file() {
        return Err(ControllerError::GnirehtetNotFound {
            path: gnirehtet_path.display().to_string(),
        });
    }

    let path_str = gnirehtet_path.to_string_lossy().to_string();
    info!(
        target: TAG,
        "Spawning relay: {:?} relay -p {}",
        gnirehtet_path,
        port
    );

    let mut child = Command::new(gnirehtet_path)
        .args(["relay", "-p", &port.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| ControllerError::RelaySpawn {
            path: path_str.clone(),
            source,
        })?;

    // Brief settle: if bind raced or binary crashed, do not claim ownership.
    thread::sleep(Duration::from_millis(150));
    match child.try_wait() {
        Ok(Some(status)) => {
            return Err(ControllerError::RelayStartFailed {
                port,
                detail: format!("exited immediately: {}", status),
            });
        }
        Ok(None) => {}
        Err(err) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ControllerError::Io(err));
        }
    }

    Ok(RelayProcess {
        child,
        port,
        path: gnirehtet_path.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn probe_detects_addr_in_use() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind ephemeral");
        let port = listener.local_addr().unwrap().port();
        let err = probe_relay_port(port).expect_err("port should be busy");
        match err {
            ControllerError::PortInUse { port: p } => assert_eq!(p, port),
            other => panic!("expected PortInUse, got {}", other),
        }
        assert_eq!(err.ux_code(), Some("PORT_IN_USE"));
        drop(listener);
        probe_relay_port(port).expect("port free after drop");
    }
}
