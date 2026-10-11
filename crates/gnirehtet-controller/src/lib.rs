//! Supervision of the stock `gnirehtet relay` process used by the desktop app.
//!
//! The relay runs as a child process rather than in-process so that a relay
//! crash cannot take the GUI down, and so that the shipped relay is the
//! unmodified upstream binary.

pub mod error;
pub mod relay;

pub use crate::error::ControllerError;
pub use crate::relay::{probe_relay_port, RelayStdio, RelaySupervisor};
pub use gnirehtet_adb;
pub use gnirehtet_adb::DEFAULT_PORT as DEFAULT_RELAY_PORT;
