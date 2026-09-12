//! Session-owned stock `gnirehtet relay` child process.

use crate::error::ControllerError;
use log::*;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, ChildStdout, Command, Stdio};
use std::thread;
use std::time::Duration;

const TAG: &str = "RelayProcess";

/// Upstream / CLI default relay listen port.
pub const DEFAULT_RELAY_PORT: u16 = 31416;

/// Piped stdout/stderr from a relay child started with `start_relay_with_stdio`.
///
/// **Concurrency:** take these under the `SessionController` lock, then read on
/// Desktop threads / channels **outside** the mutex. Desktop owns the single
/// `LogLine` pump — the controller does not spawn log-reader threads.
#[derive(Debug)]
pub struct RelayStdio {
    pub stdout: ChildStdout,
    pub stderr: ChildStderr,
}

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

/// Spawn `gnirehtet relay -p <port>` with **inherited** stdio (Desktop spawn/stop-first path).
///
/// Does not claim ownership for the caller — the returned `RelayProcess` is the
/// ownership token. If the child exits immediately, returns `RelayStartFailed`
/// and does not leave a live child.
pub fn spawn_relay(gnirehtet_path: &Path, port: u16) -> Result<RelayProcess, ControllerError> {
    let (relay, stdio) = spawn_relay_inner(gnirehtet_path, port, false)?;
    debug_assert!(stdio.is_none());
    Ok(relay)
}

/// Spawn `gnirehtet relay -p <port>` with **piped** stdout/stderr for LogLine streaming.
///
/// Same port probe + immediate-exit checks as [`spawn_relay`]. Callers must
/// drain the returned pipes (or the child may block on a full pipe buffer).
pub fn spawn_relay_with_stdio(
    gnirehtet_path: &Path,
    port: u16,
) -> Result<(RelayProcess, RelayStdio), ControllerError> {
    let (relay, stdio) = spawn_relay_inner(gnirehtet_path, port, true)?;
    Ok((
        relay,
        stdio.expect("piped spawn must return RelayStdio"),
    ))
}

fn spawn_relay_inner(
    gnirehtet_path: &Path,
    port: u16,
    pipe_stdio: bool,
) -> Result<(RelayProcess, Option<RelayStdio>), ControllerError> {
    probe_relay_port(port)?;

    if gnirehtet_path.is_absolute() && !gnirehtet_path.is_file() {
        return Err(ControllerError::GnirehtetNotFound {
            path: gnirehtet_path.display().to_string(),
        });
    }

    let path_str = gnirehtet_path.to_string_lossy().to_string();
    info!(
        target: TAG,
        "Spawning relay: {:?} relay -p {} (stdio={})",
        gnirehtet_path,
        port,
        if pipe_stdio { "piped" } else { "inherit" }
    );

    let stdout_cfg = if pipe_stdio {
        Stdio::piped()
    } else {
        Stdio::inherit()
    };
    let stderr_cfg = if pipe_stdio {
        Stdio::piped()
    } else {
        Stdio::inherit()
    };

    let mut child = Command::new(gnirehtet_path)
        .args(["relay", "-p", &port.to_string()])
        .stdout(stdout_cfg)
        .stderr(stderr_cfg)
        .spawn()
        .map_err(|source| ControllerError::RelaySpawn {
            path: path_str.clone(),
            source,
        })?;

    let stdio = if pipe_stdio {
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| ControllerError::RelayStartFailed {
                port,
                detail: "stdout pipe missing after piped spawn".into(),
            })?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| ControllerError::RelayStartFailed {
                port,
                detail: "stderr pipe missing after piped spawn".into(),
            })?;
        Some(RelayStdio { stdout, stderr })
    } else {
        None
    };

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

    Ok((
        RelayProcess {
            child,
            port,
            path: gnirehtet_path.to_path_buf(),
        },
        stdio,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use std::os::unix::fs::PermissionsExt;

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

    /// Stub binary that accepts `relay -p <port>` like stock gnirehtet, writes
    /// lines, then sleeps so ownership can be claimed.
    #[test]
    fn spawn_with_stdio_exposes_readable_pipes() {
        let dir = std::env::temp_dir().join(format!(
            "gnirehtet-controller-stdio-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let stub = dir.join("fake-gnirehtet");
        {
            let mut f = std::fs::File::create(&stub).unwrap();
            writeln!(
                f,
                "#!/bin/sh\n# args: relay -p <port>\necho 'relay-stdout-line'\necho 'relay-stderr-line' >&2\nexec sleep 30\n"
            )
            .unwrap();
        }
        let mut perms = std::fs::metadata(&stub).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&stub, perms).unwrap();

        // Free ephemeral port for the pre-spawn probe (stub does not bind).
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let (relay, stdio) = spawn_relay_with_stdio(&stub, port).expect("stub spawn");
        assert_eq!(relay.port(), port);
        assert!(relay.pid() > 0);

        let mut out = BufReader::new(stdio.stdout);
        let mut err = BufReader::new(stdio.stderr);
        let mut line = String::new();
        out.read_line(&mut line).unwrap();
        assert_eq!(line.trim(), "relay-stdout-line");
        line.clear();
        err.read_line(&mut line).unwrap();
        assert_eq!(line.trim(), "relay-stderr-line");

        relay.kill_and_wait().unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn spawn_with_stdio_port_in_use_before_ownership() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let err = spawn_relay_with_stdio(Path::new("gnirehtet"), port).expect_err("busy");
        assert!(matches!(err, ControllerError::PortInUse { .. }));
        drop(listener);
    }
}
