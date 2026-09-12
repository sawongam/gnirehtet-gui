use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdbInfo {
    pub path: String,
    pub version: String,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub serial: String,
    pub adb_state: String,
    pub model: Option<String>,
    pub product: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayStatePayload {
    /// relay_stopped | relay_starting | relay_running | relay_error
    pub state: String,
    pub port: Option<u16>,
    pub pid: Option<u32>,
    pub owned_by_session: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogLine {
    pub timestamp_ms: u64,
    pub level: String,
    pub source: String,
    pub message: String,
}

/// App `Error` event — codes must be ERROR_UX / ERROR_CODE_MAP canonical.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub message: String,
    pub serial: Option<String>,
}
