//! gnirehtet-controller — Phase 2 session orchestration (sidecar-first).
//!
//! Depends on `gnirehtet-adb` only. Does **not** link `gnirehtet-relay` / relaylib.
//! Desktop and CLI can share this for relay child lifecycle + ADB verb mirrors.

pub mod error;
pub mod relay;
pub mod session;

pub use crate::error::ControllerError;
pub use crate::relay::{
    probe_relay_port, spawn_relay, spawn_relay_with_stdio, RelayProcess, RelayStdio,
    DEFAULT_RELAY_PORT,
};
pub use crate::session::{ControllerConfig, RunOptions, SessionController};

// Re-export common adb types so Desktop can depend primarily on controller later.
pub use gnirehtet_adb::{
    AdbClient, AdbConfig, AdbDevice, AdbStatus, CommandExecutionError, VpnOptions,
};
