//! Payloads exchanged with the frontend (mirrored in `src/lib/types.ts`).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdbInfo {
    pub path: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub serial: String,
    /// Raw adb state: `device`, `unauthorized`, `offline`, `no permissions`, ...
    pub state: String,
    pub model: Option<String>,
    pub product: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RelayState {
    Stopped,
    Running,
    Crashed,
}

/// Payload of the `relay-status` event and of session commands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayStatus {
    pub state: RelayState,
    pub port: Option<u16>,
    pub pid: Option<u32>,
    pub message: Option<String>,
}

impl RelayStatus {
    pub fn stopped() -> Self {
        Self {
            state: RelayState::Stopped,
            port: None,
            pid: None,
            message: None,
        }
    }
}

/// Snapshot returned by `get_session`, used to restore the UI after a reload.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSnapshot {
    pub relay: RelayStatus,
    /// Device the client was started on, while a session is active.
    pub serial: Option<String>,
    /// Devices currently connected to the relay through the tunnel.
    pub connected_clients: u32,
}

/// Payload of the `relay-client` event: a device VPN connected to or
/// disconnected from the relay.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientEvent {
    pub connected: bool,
    pub client_id: u32,
    pub connected_clients: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogSource {
    App,
    Relay,
}

/// Payload of the `log` event.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub timestamp_ms: u64,
    pub level: LogLevel,
    pub source: LogSource,
    pub message: String,
}

/// Options for `connect`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectOptions {
    pub port: Option<u16>,
    /// Comma-separated IPv4 addresses; empty means the client default.
    pub dns_servers: Option<String>,
    /// Comma-separated IPv4 CIDR routes; empty means all traffic.
    pub routes: Option<String>,
}

/// Resolved locations of external tools, shown in Settings.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub app_version: String,
    pub adb_path: String,
    pub relay_path: String,
    pub relay_found: bool,
    pub apk_path: String,
    pub apk_found: bool,
}
