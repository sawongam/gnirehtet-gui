//! Controller-level errors. ADB failures reuse `CommandExecutionError`.
//! `PortInUse` maps to ERROR_UX `PORT_IN_USE` (PHASE0_ACCEPTANCE P0-P2).

use gnirehtet_adb::CommandExecutionError;
use std::error;
use std::fmt;
use std::io;

/// Errors from session / relay child orchestration.
#[derive(Debug)]
pub enum ControllerError {
    /// Listen port is already occupied — do **not** claim relay ownership.
    PortInUse { port: u16 },
    /// Failed to spawn the stock `gnirehtet` binary.
    RelaySpawn {
        path: String,
        source: io::Error,
    },
    /// Child exited immediately after spawn (often bind failure).
    RelayStartFailed {
        port: u16,
        detail: String,
    },
    /// A session-owned relay is already running; stop first (no hot-swap).
    RelayAlreadyRunning { port: u16 },
    /// Caller asked to stop but we do not own a relay.
    NoOwnedRelay,
    /// Stock gnirehtet binary path missing / not a file when absolute.
    GnirehtetNotFound { path: String },
    /// Wrapped ADB / process execution error.
    Adb(CommandExecutionError),
    Io(io::Error),
}

impl ControllerError {
    /// ERROR_UX / PHASE0 canonical code when applicable.
    pub fn ux_code(&self) -> Option<&'static str> {
        match self {
            ControllerError::PortInUse { .. } => Some("PORT_IN_USE"),
            ControllerError::RelaySpawn { .. } | ControllerError::RelayStartFailed { .. } => {
                Some("RELAY_START_FAILED")
            }
            ControllerError::GnirehtetNotFound { .. } => Some("RELAY_START_FAILED"),
            ControllerError::Adb(e) => e.adb_ux_code_hint(),
            ControllerError::RelayAlreadyRunning { .. }
            | ControllerError::NoOwnedRelay
            | ControllerError::Io(_) => None,
        }
    }
}

impl fmt::Display for ControllerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ControllerError::PortInUse { port } => {
                write!(
                    f,
                    "Relay listen port {} is already in use (PORT_IN_USE); not claiming ownership",
                    port
                )
            }
            ControllerError::RelaySpawn { path, source } => {
                write!(f, "Failed to spawn gnirehtet at '{}': {}", path, source)
            }
            ControllerError::RelayStartFailed { port, detail } => {
                write!(
                    f,
                    "Relay on port {} exited before listen was confirmed: {}",
                    port, detail
                )
            }
            ControllerError::RelayAlreadyRunning { port } => {
                write!(
                    f,
                    "Session already owns a relay on port {}; stop it before changing port",
                    port
                )
            }
            ControllerError::NoOwnedRelay => {
                write!(f, "No session-owned relay to stop")
            }
            ControllerError::GnirehtetNotFound { path } => {
                write!(f, "gnirehtet binary not found: {}", path)
            }
            ControllerError::Adb(err) => write!(f, "{}", err),
            ControllerError::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl error::Error for ControllerError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ControllerError::RelaySpawn { source, .. } => Some(source),
            ControllerError::Adb(err) => Some(err),
            ControllerError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<CommandExecutionError> for ControllerError {
    fn from(value: CommandExecutionError) -> Self {
        ControllerError::Adb(value)
    }
}

impl From<io::Error> for ControllerError {
    fn from(value: io::Error) -> Self {
        ControllerError::Io(value)
    }
}
