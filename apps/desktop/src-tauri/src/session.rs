//! Reverse-tethering session: relay process, ADB client commands, and the
//! events that keep the UI in sync.
//!
//! Locking: `operation` serializes user actions (connect, disconnect,
//! install, repair) so they cannot interleave. `relay` and `active` are only
//! held briefly and never across a slow adb call, so device polling stays
//! responsive while an install is running.

use crate::error::{CommandError, CommandResult};
use crate::relay_output::{self, ClientChange};
use crate::types::{
    AdbInfo, ClientEvent, ConnectOptions, DeviceInfo, LogEntry, LogLevel, LogSource, RelayState,
    RelayStatus, SessionSnapshot,
};
use gnirehtet_controller::gnirehtet_adb::{
    validate_dns_servers, validate_routes, AdbClient, VpnOptions,
};
use gnirehtet_controller::{RelayStdio, RelaySupervisor, DEFAULT_RELAY_PORT};
use log::{info, warn};
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const TAG: &str = "Session";

/// How often the relay process is checked for an unexpected exit.
const RELAY_POLL_INTERVAL: Duration = Duration::from_millis(300);

/// Upper bound for stopping the device client when the app exits.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

/// Receives the events the UI listens to.
pub trait EventSink: Send + Sync + 'static {
    fn log(&self, entry: LogEntry);
    fn relay_status(&self, status: &RelayStatus);
    fn relay_client(&self, event: &ClientEvent);
}

#[derive(Debug, Clone)]
struct ActiveSession {
    serial: String,
    port: u16,
}

pub struct Session {
    adb: AdbClient,
    relay: Mutex<RelaySupervisor>,
    active: Mutex<Option<ActiveSession>>,
    operation: Mutex<()>,
    /// Incremented whenever the relay is intentionally started or stopped, so
    /// monitors of a previous relay go quiet instead of reporting a crash.
    generation: AtomicU64,
    connected_clients: AtomicU32,
    events: Arc<dyn EventSink>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    // A panic while holding one of these locks leaves no broken invariant
    // worth refusing to continue for.
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

impl Session {
    pub fn new(adb: AdbClient, relay_binary: PathBuf, events: Arc<dyn EventSink>) -> Arc<Self> {
        Arc::new(Self {
            adb,
            relay: Mutex::new(RelaySupervisor::new(relay_binary)),
            active: Mutex::new(None),
            operation: Mutex::new(()),
            generation: AtomicU64::new(0),
            connected_clients: AtomicU32::new(0),
            events,
        })
    }

    pub fn check_adb(&self) -> CommandResult<AdbInfo> {
        let status = self
            .adb
            .ensure_adb()
            .map_err(|err| CommandError::adb(err, "ADB_PATH_INVALID"))?;
        Ok(AdbInfo {
            path: status.path,
            version: status.version,
        })
    }

    pub fn list_devices(&self) -> CommandResult<Vec<DeviceInfo>> {
        let devices = self
            .adb
            .list_devices()
            .map_err(|err| CommandError::adb(err, "ADB_FAILED"))?;
        Ok(devices
            .into_iter()
            .map(|d| DeviceInfo {
                serial: d.serial,
                state: d.state,
                model: d.model,
                product: d.product,
            })
            .collect())
    }

    pub fn snapshot(&self) -> SessionSnapshot {
        SessionSnapshot {
            relay: self.relay_status(),
            serial: lock(&self.active).as_ref().map(|a| a.serial.clone()),
            connected_clients: self.connected_clients.load(Ordering::SeqCst),
        }
    }

    /// Install (or reinstall) the client APK on a device.
    pub fn install_client(&self, serial: &str) -> CommandResult<()> {
        let serial = validate_serial(serial)?;
        let _operation = lock(&self.operation);
        info!(target: TAG, "Installing the client on {serial}");
        self.adb
            .install(Some(serial))
            .map_err(|err| CommandError::adb(err, "INSTALL_FAILED"))?;
        info!(target: TAG, "Client installed on {serial}");
        Ok(())
    }

    /// Recreate the `adb reverse` tunnel, e.g. after the cable was replugged.
    pub fn repair_tunnel(&self, serial: &str) -> CommandResult<()> {
        let serial = validate_serial(serial)?;
        let _operation = lock(&self.operation);
        let port = lock(&self.active)
            .as_ref()
            .map(|a| a.port)
            .or_else(|| lock(&self.relay).port())
            .unwrap_or(DEFAULT_RELAY_PORT);
        info!(target: TAG, "Restoring the tunnel for {serial} (port {port})");
        self.adb
            .tunnel(Some(serial), port)
            .map_err(|err| CommandError::adb(err, "TUNNEL_FAILED"))
    }

    /// Start the relay if needed, then install, tunnel and start the client.
    ///
    /// On success the device shows the VPN permission prompt; a `relay-client`
    /// event follows once the user accepts it.
    pub fn connect(
        self: &Arc<Self>,
        serial: &str,
        options: ConnectOptions,
    ) -> CommandResult<RelayStatus> {
        let serial = validate_serial(serial)?;
        let port = options.port.unwrap_or(DEFAULT_RELAY_PORT);
        if port < 1024 {
            return Err(CommandError::new(
                "INVALID_ARGUMENT",
                format!("Invalid port {port}: use a port between 1024 and 65535"),
            ));
        }
        let dns_servers = non_blank(options.dns_servers.as_deref())
            .map(validate_dns_servers)
            .transpose()
            .map_err(|err| CommandError::adb(err, "INVALID_ARGUMENT"))?;
        let routes = non_blank(options.routes.as_deref())
            .map(validate_routes)
            .transpose()
            .map_err(|err| CommandError::adb(err, "INVALID_ARGUMENT"))?;

        let _operation = lock(&self.operation);

        // One device at a time: switching devices stops the previous client.
        // For the same device the session is kept until the new start succeeds,
        // so a failed reconnect can still be cleaned up by `disconnect`.
        let previous = lock(&self.active).clone();
        if let Some(previous) = previous.filter(|p| p.serial != serial) {
            self.stop_client(&previous.serial);
            *lock(&self.active) = None;
        }

        let started_relay = self.ensure_relay(port)?;
        let result = self.start_client(
            serial,
            &VpnOptions {
                dns_servers: dns_servers.as_deref(),
                routes: routes.as_deref(),
                port,
            },
        );
        if let Err(err) = result {
            if started_relay {
                self.stop_relay();
            }
            warn!(target: TAG, "{err}");
            return Err(err);
        }

        *lock(&self.active) = Some(ActiveSession {
            serial: serial.to_string(),
            port,
        });
        info!(target: TAG, "Waiting for the VPN to be accepted on {serial}");
        Ok(self.relay_status())
    }

    /// Stop the client on the device, remove the tunnel and stop the relay.
    ///
    /// Always leaves the computer side stopped, even if the device cannot be
    /// reached; `message` then says so.
    pub fn disconnect(&self) -> RelayStatus {
        let _operation = lock(&self.operation);
        let active = lock(&self.active).take();
        let reached = active
            .as_ref()
            .is_none_or(|active| self.stop_client(&active.serial));
        self.stop_relay();
        let mut status = RelayStatus::stopped();
        if !reached {
            status.message = Some(
                "The device could not be reached; its VPN may still be on until it is reconnected."
                    .into(),
            );
        }
        status
    }

    /// Exit path: stop the relay immediately, then give the device a bounded
    /// amount of time to stop its client. Never touches a device this app did
    /// not start a client on.
    pub fn shutdown(self: &Arc<Self>) {
        self.stop_relay();
        let Some(active) = lock(&self.active).take() else {
            return;
        };
        let (done_tx, done_rx) = mpsc::channel();
        let session = Arc::clone(self);
        thread::spawn(move || {
            session.stop_client(&active.serial);
            let _ = done_tx.send(());
        });
        if done_rx.recv_timeout(SHUTDOWN_TIMEOUT).is_err() {
            warn!(target: TAG, "Timed out stopping the client on exit");
        }
    }

    fn start_client(&self, serial: &str, opts: &VpnOptions<'_>) -> CommandResult<()> {
        let serial = Some(serial);
        if self
            .adb
            .install_if_needed(serial)
            .map_err(|err| CommandError::adb(err, "INSTALL_FAILED"))?
        {
            info!(target: TAG, "Client installed");
        }
        self.adb
            .tunnel(serial, opts.port)
            .map_err(|err| CommandError::adb(err, "TUNNEL_FAILED"))?;
        self.adb
            .start_client(serial, opts)
            .map_err(|err| CommandError::adb(err, "CLIENT_START_FAILED"))
    }

    /// Best effort: returns whether the device acknowledged the stop.
    fn stop_client(&self, serial: &str) -> bool {
        info!(target: TAG, "Stopping the client on {serial}");
        let stopped = match self.adb.stop(Some(serial)) {
            Ok(()) => true,
            Err(err) => {
                warn!(target: TAG, "Could not stop the client on {serial}: {err}");
                false
            }
        };
        let _ = self.adb.remove_tunnel(Some(serial));
        stopped
    }

    /// Make sure our relay listens on `port`. Returns whether it was started
    /// by this call.
    fn ensure_relay(self: &Arc<Self>, port: u16) -> CommandResult<bool> {
        let mut relay = lock(&self.relay);
        let _ = relay.poll();
        if relay.port() == Some(port) {
            return Ok(false);
        }
        if relay.is_running() {
            self.generation.fetch_add(1, Ordering::SeqCst);
            relay.stop();
        }
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        self.connected_clients.store(0, Ordering::SeqCst);

        info!(target: TAG, "Starting the relay on port {port}");
        let stdio = relay.start(port).map_err(|err| {
            warn!(target: TAG, "{err}");
            CommandError::from(err)
        })?;
        let status = RelayStatus {
            state: RelayState::Running,
            port: Some(port),
            pid: relay.pid(),
            message: None,
        };
        drop(relay);

        self.monitor_relay(stdio, generation);
        self.events.relay_status(&status);
        Ok(true)
    }

    fn stop_relay(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        let was_running = lock(&self.relay).stop();
        self.connected_clients.store(0, Ordering::SeqCst);
        if was_running {
            info!(target: TAG, "Relay stopped");
        }
        self.events.relay_status(&RelayStatus::stopped());
    }

    fn relay_status(&self) -> RelayStatus {
        let relay = lock(&self.relay);
        match relay.pid() {
            Some(pid) => RelayStatus {
                state: RelayState::Running,
                port: relay.port(),
                pid: Some(pid),
                message: None,
            },
            None => RelayStatus::stopped(),
        }
    }

    fn is_current(&self, generation: u64) -> bool {
        self.generation.load(Ordering::SeqCst) == generation
    }

    /// Forward relay output to the log and watch for an unexpected exit.
    fn monitor_relay(self: &Arc<Self>, stdio: RelayStdio, generation: u64) {
        let session = Arc::clone(self);
        thread::spawn(move || {
            session.pump_output(stdio.stdout, LogLevel::Info, generation, true);
        });
        let session = Arc::clone(self);
        thread::spawn(move || {
            session.pump_output(stdio.stderr, LogLevel::Warn, generation, false);
        });
        let session = Arc::clone(self);
        thread::spawn(move || session.watch_relay(generation));
    }

    fn pump_output(
        &self,
        pipe: impl Read,
        fallback: LogLevel,
        generation: u64,
        track_clients: bool,
    ) {
        let mut reader = BufReader::new(pipe);
        let mut buf = Vec::new();
        loop {
            buf.clear();
            match reader.read_until(b'\n', &mut buf) {
                Ok(0) | Err(_) => return,
                Ok(_) => {}
            }
            if !self.is_current(generation) {
                continue; // keep draining so the process never blocks
            }
            let text = String::from_utf8_lossy(&buf);
            let line = relay_output::parse_line(text.trim_end(), fallback);
            if track_clients {
                if let Some(change) = relay_output::client_change(&line.message) {
                    self.on_client_change(change);
                }
            }
            self.events.log(LogEntry {
                timestamp_ms: now_ms(),
                level: line.level,
                source: LogSource::Relay,
                message: line.message,
            });
        }
    }

    fn on_client_change(&self, change: ClientChange) {
        let (connected, client_id) = match change {
            ClientChange::Connected(id) => (true, id),
            ClientChange::Disconnected(id) => (false, id),
        };
        let count = if connected {
            self.connected_clients.fetch_add(1, Ordering::SeqCst) + 1
        } else {
            let previous = self
                .connected_clients
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| {
                    Some(n.saturating_sub(1))
                })
                .unwrap_or(0);
            previous.saturating_sub(1)
        };
        self.events.relay_client(&ClientEvent {
            connected,
            client_id,
            connected_clients: count,
        });
    }

    fn watch_relay(&self, generation: u64) {
        let status = loop {
            thread::sleep(RELAY_POLL_INTERVAL);
            if !self.is_current(generation) {
                return;
            }
            let mut relay = lock(&self.relay);
            if !self.is_current(generation) {
                return;
            }
            match relay.poll() {
                Ok(Some(status)) => break status,
                Ok(None) => {}
                Err(err) => warn!(target: TAG, "Cannot check the relay process: {err}"),
            }
        };
        self.connected_clients.store(0, Ordering::SeqCst);
        let message = format!("The relay stopped unexpectedly ({status})");
        warn!(target: TAG, "{message}");
        self.events.relay_status(&RelayStatus {
            state: RelayState::Crashed,
            port: None,
            pid: None,
            message: Some(message),
        });
    }
}

fn non_blank(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|v| !v.is_empty())
}

fn validate_serial(serial: &str) -> CommandResult<&str> {
    let valid = !serial.is_empty()
        && serial.len() <= 256
        && !serial.chars().any(|c| c.is_whitespace() || c.is_control());
    if valid {
        Ok(serial)
    } else {
        Err(CommandError::new(
            "INVALID_ARGUMENT",
            format!("Invalid device serial '{serial}'"),
        ))
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use gnirehtet_controller::gnirehtet_adb::AdbConfig;
    use std::io::Write;
    use std::net::TcpListener;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::time::Instant;

    #[derive(Default)]
    struct Recorder {
        statuses: Mutex<Vec<RelayStatus>>,
        clients: Mutex<Vec<ClientEvent>>,
    }

    impl EventSink for Recorder {
        fn log(&self, _entry: LogEntry) {}
        fn relay_status(&self, status: &RelayStatus) {
            lock(&self.statuses).push(status.clone());
        }
        fn relay_client(&self, event: &ClientEvent) {
            lock(&self.clients).push(event.clone());
        }
    }

    struct Fixture {
        dir: PathBuf,
        recorder: Arc<Recorder>,
        session: Arc<Session>,
        port: u16,
        _listener: Option<thread::JoinHandle<()>>,
        // Declared last so it is released after everything else is dropped.
        _serial: MutexGuard<'static, ()>,
    }

    /// Fixtures bind ephemeral ports and exec freshly written scripts;
    /// concurrent tests race on both (port reuse, ETXTBSY), so serialize.
    static SERIAL: Mutex<()> = Mutex::new(());

    impl Fixture {
        /// `adb_script` handles adb invocations (`$*` is the argument list);
        /// every invocation is appended to `adb.log`. The fake relay prints
        /// `relay_script` output; the port is bound by the test on its behalf.
        fn new(name: &str, adb_script: &str, relay_script: &str) -> Self {
            let serial = lock(&SERIAL);
            let dir = std::env::temp_dir().join(format!(
                "gnirehtet-session-{}-{}",
                name,
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            let log = dir.join("adb.log");
            let adb = script(
                &dir,
                "adb",
                &format!("echo \"$*\" >> '{}'\n{}", log.display(), adb_script),
            );
            let relay = script(&dir, "gnirehtet", relay_script);
            let apk = dir.join("gnirehtet.apk");
            std::fs::write(&apk, b"apk").unwrap();

            let port = TcpListener::bind(("127.0.0.1", 0))
                .unwrap()
                .local_addr()
                .unwrap()
                .port();
            let recorder = Arc::new(Recorder::default());
            let session = Session::new(
                AdbClient::new(AdbConfig {
                    adb_path: adb,
                    apk_path: apk,
                    ..AdbConfig::default()
                }),
                relay,
                recorder.clone(),
            );
            Self {
                dir,
                recorder,
                session,
                port,
                _listener: None,
                _serial: serial,
            }
        }

        /// Occupy the port shortly after the relay is spawned, as it would.
        fn listen_for_relay(&mut self, hold: Duration) {
            let port = self.port;
            self._listener = Some(thread::spawn(move || {
                thread::sleep(Duration::from_millis(100));
                let listener = bind_retrying(port);
                thread::sleep(hold);
                drop(listener);
            }));
        }

        fn adb_calls(&self) -> Vec<String> {
            std::fs::read_to_string(self.dir.join("adb.log"))
                .unwrap_or_default()
                .lines()
                .map(String::from)
                .collect()
        }

        fn options(&self, dns: Option<&str>) -> ConnectOptions {
            ConnectOptions {
                port: Some(self.port),
                dns_servers: dns.map(String::from),
                routes: None,
            }
        }
    }

    /// Bind `port`, retrying while the supervisor's readiness probe briefly
    /// holds it (the probe binds the same port every few milliseconds).
    fn bind_retrying(port: u16) -> TcpListener {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match TcpListener::bind(("127.0.0.1", port)) {
                Ok(listener) => return listener,
                Err(err) if Instant::now() < deadline => {
                    assert_eq!(err.kind(), std::io::ErrorKind::AddrInUse, "{err}");
                    thread::sleep(Duration::from_millis(1));
                }
                Err(err) => panic!("bind {port}: {err}"),
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
        let path = dir.join(name);
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(file, "#!/bin/sh\n{body}").unwrap();
        drop(file);
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    fn wait_for(condition: impl Fn() -> bool) -> bool {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if condition() {
                return true;
            }
            thread::sleep(Duration::from_millis(20));
        }
        false
    }

    const INSTALLED_ADB: &str = r#"case "$*" in
  *dumpsys*) echo "    versionCode=9 minSdk=21 targetSdk=33" ;;
  *"am start"*) echo "Starting: Intent { cmp=com.genymobile.gnirehtet/.GnirehtetActivity }" ;;
esac
exit 0"#;

    const CONNECTING_RELAY: &str = "sleep 0.3
echo '2026-01-01 00:00:00.000 INFO TunnelServer: Client #0 connected'
exec sleep 30";

    #[test]
    fn connect_and_disconnect() {
        let mut fx = Fixture::new("connect", INSTALLED_ADB, CONNECTING_RELAY);
        fx.listen_for_relay(Duration::from_secs(3));

        let status = fx
            .session
            .connect("SERIAL1", fx.options(Some("1.1.1.1, 9.9.9.9")))
            .expect("connect");
        assert_eq!(status.state, RelayState::Running);
        assert_eq!(status.port, Some(fx.port));

        assert!(wait_for(|| !lock(&fx.recorder.clients).is_empty()));
        let event = lock(&fx.recorder.clients)[0].clone();
        assert!(event.connected);
        assert_eq!(event.connected_clients, 1);
        assert_eq!(fx.session.snapshot().serial.as_deref(), Some("SERIAL1"));

        let calls = fx.adb_calls();
        let port = fx.port;
        assert!(calls.contains(&"-s SERIAL1 shell dumpsys package com.genymobile.gnirehtet".into()));
        assert!(calls.contains(&format!(
            "-s SERIAL1 reverse localabstract:gnirehtet tcp:{port}"
        )));
        assert!(calls.iter().any(|c| c
            .starts_with("-s SERIAL1 shell am start -a com.genymobile.gnirehtet.START")
            && c.ends_with("--esa dnsServers 1.1.1.1,9.9.9.9")));
        assert!(
            !calls.iter().any(|c| c.contains(" install ")),
            "already installed"
        );

        let status = fx.session.disconnect();
        assert_eq!(status.state, RelayState::Stopped);
        assert!(status.message.is_none());
        let calls = fx.adb_calls();
        assert!(calls
            .iter()
            .any(|c| c.starts_with("-s SERIAL1 shell am start -a com.genymobile.gnirehtet.STOP")));
        assert!(calls.contains(&"-s SERIAL1 reverse --remove localabstract:gnirehtet".into()));

        // An intentional stop is never reported as a crash.
        thread::sleep(Duration::from_millis(500));
        assert!(lock(&fx.recorder.statuses)
            .iter()
            .all(|s| s.state != RelayState::Crashed));
        assert!(fx.session.snapshot().serial.is_none());
    }

    #[test]
    fn missing_client_is_installed_first() {
        let adb = r#"case "$*" in
  *dumpsys*) echo "Unable to find package: com.genymobile.gnirehtet" ;;
esac
exit 0"#;
        let mut fx = Fixture::new("install", adb, "exec sleep 30");
        fx.listen_for_relay(Duration::from_secs(3));
        fx.session.connect("S", fx.options(None)).expect("connect");
        let calls = fx.adb_calls();
        let install = calls
            .iter()
            .position(|c| c.starts_with("-s S install -r "))
            .expect("installed");
        let start = calls
            .iter()
            .position(|c| c.contains("am start"))
            .expect("started");
        assert!(install < start);
        fx.session.disconnect();
    }

    #[test]
    fn invalid_options_are_rejected_before_anything_runs() {
        let fx = Fixture::new("invalid", INSTALLED_ADB, "exec sleep 30");
        let err = fx
            .session
            .connect("S", fx.options(Some("8.8.8.8; reboot")))
            .expect_err("invalid dns");
        assert_eq!(err.code, "INVALID_ARGUMENT");

        let err = fx
            .session
            .connect(
                "S",
                ConnectOptions {
                    port: Some(80),
                    ..Default::default()
                },
            )
            .expect_err("privileged port");
        assert_eq!(err.code, "INVALID_ARGUMENT");

        let err = fx
            .session
            .connect("bad serial", fx.options(None))
            .expect_err("serial");
        assert_eq!(err.code, "INVALID_ARGUMENT");

        assert!(fx.adb_calls().is_empty());
        assert_eq!(fx.session.snapshot().relay.state, RelayState::Stopped);
    }

    #[test]
    fn client_start_failure_stops_the_relay_it_started() {
        let adb = r#"case "$*" in
  *dumpsys*) echo "    versionCode=9" ;;
  *"am start"*) echo "Error: Activity not started, unable to resolve Intent" ;;
esac
exit 0"#;
        let mut fx = Fixture::new("reject", adb, "exec sleep 30");
        fx.listen_for_relay(Duration::from_secs(3));
        let err = fx
            .session
            .connect("S", fx.options(None))
            .expect_err("rejected");
        assert_eq!(err.code, "CLIENT_START_FAILED");
        assert!(
            err.message.contains("unable to resolve Intent"),
            "{}",
            err.message
        );
        assert_eq!(fx.session.snapshot().relay.state, RelayState::Stopped);
        assert!(fx.session.snapshot().serial.is_none());
    }

    #[test]
    fn unauthorized_device_is_reported() {
        let adb = "echo 'adb: device unauthorized.' >&2\nexit 1";
        let mut fx = Fixture::new("unauthorized", adb, "exec sleep 30");
        fx.listen_for_relay(Duration::from_secs(3));
        let err = fx
            .session
            .connect("S", fx.options(None))
            .expect_err("unauthorized");
        assert_eq!(err.code, "DEVICE_UNAUTHORIZED");
    }

    #[test]
    fn busy_port_is_reported_without_touching_the_device() {
        let fx = Fixture::new("busy", INSTALLED_ADB, "exec sleep 30");
        let _foreign = TcpListener::bind(("127.0.0.1", fx.port)).unwrap();
        let err = fx.session.connect("S", fx.options(None)).expect_err("busy");
        assert_eq!(err.code, "PORT_IN_USE");
        assert!(fx.adb_calls().is_empty());
    }

    #[test]
    fn relay_crash_is_reported() {
        let mut fx = Fixture::new("crash", INSTALLED_ADB, "sleep 0.4\nexit 7");
        fx.listen_for_relay(Duration::from_millis(200));
        fx.session.connect("S", fx.options(None)).expect("connect");
        assert!(wait_for(|| lock(&fx.recorder.statuses)
            .iter()
            .any(|s| s.state == RelayState::Crashed)));
        assert_eq!(fx.session.snapshot().relay.state, RelayState::Stopped);
        // The session is kept so that disconnect still stops the device client.
        assert_eq!(fx.session.snapshot().serial.as_deref(), Some("S"));
        fx.session.disconnect();
        assert!(fx.adb_calls().iter().any(|c| c.contains("gnirehtet.STOP")));
    }

    #[test]
    fn unreachable_device_still_stops_the_relay() {
        let adb = r#"case "$*" in
  *dumpsys*) echo "    versionCode=9" ;;
  *STOP*) echo "adb: device 'S' not found" >&2; exit 1 ;;
esac
exit 0"#;
        let mut fx = Fixture::new("unplugged", adb, "exec sleep 30");
        fx.listen_for_relay(Duration::from_secs(3));
        fx.session.connect("S", fx.options(None)).expect("connect");
        let status = fx.session.disconnect();
        assert_eq!(status.state, RelayState::Stopped);
        assert!(status.message.is_some());
        assert!(!lock(&fx.session.relay).is_running());
    }

    #[test]
    fn shutdown_without_session_never_touches_devices() {
        let fx = Fixture::new("shutdown", INSTALLED_ADB, "exec sleep 30");
        fx.session.shutdown();
        fx.session.shutdown();
        assert!(fx.adb_calls().is_empty());
    }

    #[test]
    fn missing_apk_is_reported() {
        let fx = Fixture::new("apk", INSTALLED_ADB, "exec sleep 30");
        std::fs::remove_file(fx.dir.join("gnirehtet.apk")).unwrap();
        let err = fx.session.install_client("S").expect_err("no apk");
        assert_eq!(err.code, "APK_MISSING");
    }
}
