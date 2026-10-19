/*
 * Copyright (C) 2017 Genymobile
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * Modified for gnirehtet-gui: captured command output, timeouts, argument
 * validation and machine-readable error codes.
 */

use std::error;
use std::fmt;
use std::io;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;
use std::time::Duration;

#[derive(Debug)]
pub enum CommandExecutionError {
    /// The command could not be spawned (binary missing, not executable, ...).
    ProcessIo(ProcessIoError),
    /// The command ran and exited unsuccessfully.
    ProcessStatus(ProcessStatusError),
    /// The command exited successfully but its output reports a failure
    /// (`am start` exits 0 even when the activity cannot be started).
    Rejected {
        cmd: Cmd,
        output: String,
    },
    /// The command did not finish in time and was killed.
    Timeout {
        cmd: Cmd,
        after: Duration,
    },
    /// The client APK to install does not exist.
    ApkMissing {
        path: String,
    },
    /// A user-supplied value was refused before reaching the device shell.
    InvalidArgument {
        name: &'static str,
        reason: String,
    },
    Io(io::Error),
}

#[derive(Debug)]
pub struct ProcessStatusError {
    cmd: Cmd,
    termination: Termination,
    output: String,
}

#[derive(Debug)]
pub struct ProcessIoError {
    cmd: Cmd,
    error: io::Error,
}

#[derive(Debug, Clone)]
pub struct Cmd {
    command: String,
    args: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Termination {
    Value(i32),
    Signal(i32),
    Unknown,
}

impl Termination {
    fn from(status: ExitStatus) -> Self {
        if let Some(code) = status.code() {
            return Termination::Value(code);
        }
        #[cfg(unix)]
        if let Some(signal) = status.signal() {
            return Termination::Signal(signal);
        }
        Termination::Unknown
    }
}

impl fmt::Display for Cmd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.command)?;
        for arg in &self.args {
            write!(f, " {arg}")?;
        }
        Ok(())
    }
}

impl Cmd {
    pub fn new<S1, S2>(command: S1, args: Vec<S2>) -> Cmd
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            command: command.into(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }
}

impl ProcessStatusError {
    pub fn new(cmd: Cmd, status: ExitStatus, output: impl Into<String>) -> Self {
        Self {
            cmd,
            termination: Termination::from(status),
            output: output.into(),
        }
    }

    pub fn cmd(&self) -> &Cmd {
        &self.cmd
    }

    pub fn termination(&self) -> Termination {
        self.termination
    }

    /// Combined stdout/stderr of the failed command, trimmed.
    pub fn output(&self) -> &str {
        &self.output
    }
}

impl fmt::Display for ProcessStatusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.termination {
            Termination::Value(code) => write!(f, "`{}` exited with code {}", self.cmd, code)?,
            Termination::Signal(sig) => write!(f, "`{}` was killed by signal {}", self.cmd, sig)?,
            Termination::Unknown => write!(f, "`{}` failed", self.cmd)?,
        }
        if let Some(line) = last_meaningful_line(&self.output) {
            write!(f, ": {line}")?;
        }
        Ok(())
    }
}

impl error::Error for ProcessStatusError {}

impl ProcessIoError {
    pub fn new(cmd: Cmd, error: io::Error) -> Self {
        Self { cmd, error }
    }

    /// `true` when the program could not be found.
    pub fn is_not_found(&self) -> bool {
        self.error.kind() == io::ErrorKind::NotFound
    }
}

impl fmt::Display for ProcessIoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cannot run `{}`: {}", self.cmd.command, self.error)
    }
}

impl error::Error for ProcessIoError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&self.error)
    }
}

impl CommandExecutionError {
    /// Stable, machine-readable error code for UI mapping.
    ///
    /// Returns `None` when the failure has no specific meaning beyond the
    /// operation that produced it; callers then pick an operation-level code.
    pub fn code(&self) -> Option<&'static str> {
        match self {
            CommandExecutionError::ProcessIo(err) if err.is_not_found() => Some("ADB_MISSING"),
            CommandExecutionError::ProcessIo(_) => Some("ADB_PATH_INVALID"),
            CommandExecutionError::ProcessStatus(err) => classify_adb_output(err.output()),
            CommandExecutionError::Rejected { output, .. } => classify_adb_output(output),
            CommandExecutionError::Timeout { .. } => Some("ADB_TIMEOUT"),
            CommandExecutionError::ApkMissing { .. } => Some("APK_MISSING"),
            CommandExecutionError::InvalidArgument { .. } => Some("INVALID_ARGUMENT"),
            CommandExecutionError::Io(_) => None,
        }
    }
}

/// Recognize the device-state errors adb prints, whatever the subcommand.
fn classify_adb_output(output: &str) -> Option<&'static str> {
    let lower = output.to_ascii_lowercase();
    if lower.contains("unauthorized") {
        Some("DEVICE_UNAUTHORIZED")
    } else if lower.contains("device offline") {
        Some("DEVICE_OFFLINE")
    } else if lower.contains("more than one device") {
        Some("MULTIPLE_DEVICES")
    } else if lower.contains("not found") && lower.contains("device")
        || lower.contains("no devices/emulators found")
    {
        Some("DEVICE_NOT_FOUND")
    } else {
        None
    }
}

fn last_meaningful_line(output: &str) -> Option<&str> {
    output.lines().map(str::trim).rfind(|line| !line.is_empty())
}

impl fmt::Display for CommandExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandExecutionError::ProcessIo(err) => write!(f, "{err}"),
            CommandExecutionError::ProcessStatus(err) => write!(f, "{err}"),
            CommandExecutionError::Rejected { cmd, output } => {
                write!(f, "`{cmd}` reported a failure")?;
                if let Some(line) = last_meaningful_line(output) {
                    write!(f, ": {line}")?;
                }
                Ok(())
            }
            CommandExecutionError::Timeout { cmd, after } => {
                write!(f, "`{cmd}` did not finish within {}s", after.as_secs())
            }
            CommandExecutionError::ApkMissing { path } => {
                write!(f, "gnirehtet client APK not found at '{path}'")
            }
            CommandExecutionError::InvalidArgument { name, reason } => {
                write!(f, "Invalid {name}: {reason}")
            }
            CommandExecutionError::Io(err) => write!(f, "I/O error: {err}"),
        }
    }
}

impl error::Error for CommandExecutionError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            CommandExecutionError::ProcessIo(err) => Some(err),
            CommandExecutionError::ProcessStatus(err) => Some(err),
            CommandExecutionError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<ProcessIoError> for CommandExecutionError {
    fn from(error: ProcessIoError) -> Self {
        CommandExecutionError::ProcessIo(error)
    }
}

impl From<ProcessStatusError> for CommandExecutionError {
    fn from(error: ProcessStatusError) -> Self {
        CommandExecutionError::ProcessStatus(error)
    }
}

impl From<io::Error> for CommandExecutionError {
    fn from(error: io::Error) -> Self {
        CommandExecutionError::Io(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    fn exit_status(code: i32) -> ExitStatus {
        ExitStatus::from_raw(code << 8)
    }

    #[test]
    fn not_found_is_adb_missing() {
        let err = CommandExecutionError::from(ProcessIoError::new(
            Cmd::new("adb", vec!["version"]),
            io::Error::new(io::ErrorKind::NotFound, "no such file"),
        ));
        assert_eq!(err.code(), Some("ADB_MISSING"));
    }

    #[test]
    fn other_spawn_failure_is_path_invalid() {
        let err = CommandExecutionError::from(ProcessIoError::new(
            Cmd::new("/bad/adb", vec!["version"]),
            io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
        ));
        assert_eq!(err.code(), Some("ADB_PATH_INVALID"));
    }

    #[test]
    fn apk_missing_code() {
        let err = CommandExecutionError::ApkMissing {
            path: "/no/such/gnirehtet.apk".into(),
        };
        assert_eq!(err.code(), Some("APK_MISSING"));
    }

    #[cfg(unix)]
    #[test]
    fn device_errors_are_classified_from_output() {
        let cases = [
            (
                "adb: device unauthorized.\nThis adb server's $ADB_VENDOR_KEYS",
                "DEVICE_UNAUTHORIZED",
            ),
            ("adb: device offline", "DEVICE_OFFLINE"),
            ("adb: more than one device/emulator", "MULTIPLE_DEVICES"),
            ("adb: device 'XYZ' not found", "DEVICE_NOT_FOUND"),
            ("adb: no devices/emulators found", "DEVICE_NOT_FOUND"),
        ];
        for (output, code) in cases {
            let err = CommandExecutionError::from(ProcessStatusError::new(
                Cmd::new("adb", vec!["shell"]),
                exit_status(1),
                output,
            ));
            assert_eq!(err.code(), Some(code), "output: {output}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn display_includes_last_output_line() {
        let err = ProcessStatusError::new(
            Cmd::new("adb", vec!["install", "x.apk"]),
            exit_status(1),
            "Performing Streamed Install\nadb: failed to install x.apk: INSTALL_FAILED_USER_RESTRICTED\n",
        );
        assert_eq!(
            err.to_string(),
            "`adb install x.apk` exited with code 1: adb: failed to install x.apk: INSTALL_FAILED_USER_RESTRICTED"
        );
    }
}
