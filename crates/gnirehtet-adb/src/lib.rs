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
 * gnirehtet-adb: ADB extract for gnirehtet-gui (Phase 1+).
 * ByteBuffer is local — this crate must NOT depend on relaylib.
 */

//! ADB client and device monitor extracted from Genymobile gnirehtet.
//!
//! Phase 1 goal: reusable `AdbClient` without coupling to the mio relay path.
//! Also exposes `ensure_adb` / `list_devices` for Desktop discovery/health.

pub mod buffer;
pub mod client;
pub mod error;
pub mod monitor;

pub use crate::buffer::ByteBuffer;
pub use crate::client::{
    parse_adb_devices_l, AdbClient, AdbConfig, AdbDevice, AdbStatus, VpnOptions,
    REQUIRED_APK_VERSION_CODE,
};
pub use crate::error::{
    Cmd, CommandExecutionError, ProcessIoError, ProcessStatusError, Termination,
};
pub use crate::monitor::{AdbMonitor, AdbMonitorCallback};
