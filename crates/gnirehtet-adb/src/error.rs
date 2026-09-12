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
 * Adapted for gnirehtet-gui: expose accessors for Desktop ERROR_UX mapping.
 */

use std::error;
use std::fmt;
use std::io;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

#[derive(Debug)]
pub enum CommandExecutionError {
    ProcessIo(ProcessIoError),
    ProcessStatus(ProcessStatusError),
    Io(io::Error),
    /// Local helper APK path missing or not a file (ERROR_UX `APK_MISSING`).
    ApkMissing { path: String },
}

#[derive(Debug)]
pub struct ProcessStatusError {
    cmd: Cmd,
    termination: Termination,
}

#[derive(Debug)]
pub struct ProcessIoError {
    cmd: Cmd,
    error: io::Error,
}

#[derive(Debug)]
pub struct Cmd {
    command: String,
    args: Vec<String>,
}

#[derive(Debug)]
pub enum Termination {
    Value(i32),
    #[cfg(unix)]
    Signal(i32),
}

impl Termination {
    fn from(status: ExitStatus) -> Self {
        match status.code() {
            Some(code) => Termination::Value(code),
            #[cfg(unix)]
            None => Termination::Signal(status.signal().unwrap()),
            #[cfg(not(unix))]
            None => panic!("Unexpected signal termination on non-unix system"),
        }
    }
}

impl fmt::Display for Cmd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {:?}", self.command, self.args)
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
            args: args.into_iter().map(Into::into).collect::<Vec<_>>(),
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
    pub fn new(cmd: Cmd, status: ExitStatus) -> Self {
        Self {
            cmd,
            termination: Termination::from(status),
        }
    }

    pub fn cmd(&self) -> &Cmd {
        &self.cmd
    }

    pub fn termination(&self) -> &Termination {
        &self.termination
    }
}

impl fmt::Display for ProcessStatusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.termination {
            Termination::Value(code) => {
                write!(f, "Command {} returned with value {}", self.cmd, code)
            }
            #[cfg(unix)]
            Termination::Signal(sig) => {
                write!(f, "Command {} terminated by signal {}", self.cmd, sig)
            }
        }
    }
}

impl error::Error for ProcessStatusError {}

impl ProcessIoError {
    pub fn new(cmd: Cmd, error: io::Error) -> Self {
        Self { cmd, error }
    }

    pub fn cmd(&self) -> &Cmd {
        &self.cmd
    }

    pub fn io_error(&self) -> &io::Error {
        &self.error
    }

    /// `true` when the adb binary could not be spawned (typical `ADB_MISSING` /
    /// `ADB_PATH_INVALID` mapping at the UI boundary).
    pub fn is_not_found(&self) -> bool {
        self.error.kind() == io::ErrorKind::NotFound
    }
}

impl fmt::Display for ProcessIoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Command {} failed: {}", self.cmd, self.error)
    }
}

impl error::Error for ProcessIoError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&self.error)
    }
}

impl CommandExecutionError {
    /// Best-effort ERROR_UX hint for Desktop mapping (not a full taxonomy).
    ///
    /// - `ProcessIo` + `NotFound` → `ADB_MISSING`
    /// - other `ProcessIo` spawn failures → `ADB_PATH_INVALID`
    /// - non-zero exit / signal → `None` (caller uses stage context)
    /// - bare `Io` → `None`
    pub fn adb_ux_code_hint(&self) -> Option<&'static str> {
        match self {
            CommandExecutionError::ProcessIo(err) if err.is_not_found() => Some("ADB_MISSING"),
            CommandExecutionError::ProcessIo(_) => Some("ADB_PATH_INVALID"),
            CommandExecutionError::ApkMissing { .. } => Some("APK_MISSING"),
            CommandExecutionError::ProcessStatus(_) => None,
            CommandExecutionError::Io(_) => None,
        }
    }

    pub fn as_process_io(&self) -> Option<&ProcessIoError> {
        match self {
            CommandExecutionError::ProcessIo(e) => Some(e),
            _ => None,
        }
    }

    pub fn as_process_status(&self) -> Option<&ProcessStatusError> {
        match self {
            CommandExecutionError::ProcessStatus(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for CommandExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CommandExecutionError::ProcessIo(ref err) => write!(f, "{}", err),
            CommandExecutionError::ProcessStatus(ref err) => write!(f, "{}", err),
            CommandExecutionError::Io(ref err) => write!(f, "IO error: {}", err),
            CommandExecutionError::ApkMissing { ref path } => write!(
                f,
                "gnirehtet APK not found at '{}' (APK_MISSING); set GNIREHTET_APK or place resources/gnirehtet.apk",
                path
            ),
        }
    }
}

impl error::Error for CommandExecutionError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            CommandExecutionError::ProcessIo(ref err) => Some(err),
            CommandExecutionError::ProcessStatus(ref err) => Some(err),
            CommandExecutionError::Io(ref err) => Some(err),
            CommandExecutionError::ApkMissing { .. } => None,
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

    #[test]
    fn ux_hint_not_found_is_adb_missing() {
        let err = CommandExecutionError::from(ProcessIoError::new(
            Cmd::new("adb", vec!["version"]),
            io::Error::new(io::ErrorKind::NotFound, "no such file"),
        ));
        assert_eq!(err.adb_ux_code_hint(), Some("ADB_MISSING"));
        assert!(err.as_process_io().unwrap().is_not_found());
    }

    #[test]
    fn ux_hint_other_io_is_path_invalid() {
        let err = CommandExecutionError::from(ProcessIoError::new(
            Cmd::new("/bad/adb", vec!["version"]),
            io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
        ));
        assert_eq!(err.adb_ux_code_hint(), Some("ADB_PATH_INVALID"));
    }

    #[test]
    fn ux_hint_apk_missing() {
        let err = CommandExecutionError::ApkMissing {
            path: "/no/such/gnirehtet.apk".into(),
        };
        assert_eq!(err.adb_ux_code_hint(), Some("APK_MISSING"));
        assert!(err.to_string().contains("APK_MISSING"));
    }
}
