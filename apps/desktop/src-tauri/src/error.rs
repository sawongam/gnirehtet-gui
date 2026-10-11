use gnirehtet_controller::gnirehtet_adb::CommandExecutionError;
use gnirehtet_controller::ControllerError;
use serde::Serialize;
use std::fmt;

/// Error returned by every command: a stable `code` the UI maps to help
/// text, and a human-readable `message` with the details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommandError {
    pub code: &'static str,
    pub message: String,
}

impl CommandError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Map an adb failure, using `fallback` when the failure has no more
    /// specific meaning than "this operation failed".
    pub fn adb(err: CommandExecutionError, fallback: &'static str) -> Self {
        Self::new(err.code().unwrap_or(fallback), err.to_string())
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for CommandError {}

impl From<ControllerError> for CommandError {
    fn from(err: ControllerError) -> Self {
        Self::new(err.code(), err.to_string())
    }
}

pub type CommandResult<T> = Result<T, CommandError>;
