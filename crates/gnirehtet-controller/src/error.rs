use std::error;
use std::fmt;
use std::io;

/// Errors raised while supervising the relay process.
#[derive(Debug)]
pub enum ControllerError {
    /// Another process already listens on the relay port.
    PortInUse {
        port: u16,
    },
    /// The relay executable does not exist.
    RelayNotFound {
        path: String,
    },
    /// The relay executable could not be spawned.
    RelaySpawn {
        path: String,
        source: io::Error,
    },
    /// The relay exited or never started listening.
    RelayStartFailed {
        port: u16,
        detail: String,
    },
    /// A relay started by this supervisor is already running.
    RelayAlreadyRunning {
        port: u16,
    },
    Io(io::Error),
}

impl ControllerError {
    /// Stable, machine-readable error code for UI mapping.
    pub fn code(&self) -> &'static str {
        match self {
            ControllerError::PortInUse { .. } => "PORT_IN_USE",
            ControllerError::RelayNotFound { .. } => "RELAY_NOT_FOUND",
            ControllerError::RelaySpawn { source, .. }
                if source.kind() == io::ErrorKind::NotFound =>
            {
                "RELAY_NOT_FOUND"
            }
            ControllerError::RelaySpawn { .. } | ControllerError::RelayStartFailed { .. } => {
                "RELAY_START_FAILED"
            }
            ControllerError::RelayAlreadyRunning { .. } => "RELAY_ALREADY_RUNNING",
            ControllerError::Io(_) => "INTERNAL",
        }
    }
}

impl fmt::Display for ControllerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ControllerError::PortInUse { port } => {
                write!(f, "Port {port} is already in use by another program")
            }
            ControllerError::RelayNotFound { path } => {
                write!(f, "gnirehtet relay executable not found: {path}")
            }
            ControllerError::RelaySpawn { path, source } => {
                write!(f, "Cannot start the relay '{path}': {source}")
            }
            ControllerError::RelayStartFailed { port, detail } => {
                write!(f, "The relay failed to start on port {port}: {detail}")
            }
            ControllerError::RelayAlreadyRunning { port } => {
                write!(f, "The relay is already running on port {port}")
            }
            ControllerError::Io(err) => write!(f, "I/O error: {err}"),
        }
    }
}

impl error::Error for ControllerError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ControllerError::RelaySpawn { source, .. } => Some(source),
            ControllerError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for ControllerError {
    fn from(value: io::Error) -> Self {
        ControllerError::Io(value)
    }
}
