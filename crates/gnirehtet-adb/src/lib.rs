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
 * Modified for gnirehtet-gui: ADB client extracted from the gnirehtet relay
 * so that it can be reused without the relay (mio) dependencies.
 */

//! ADB client and device monitor for gnirehtet.

mod buffer;
pub mod client;
pub mod error;
pub mod monitor;

pub use crate::client::{
    background_command, validate_dns_servers, validate_routes, AdbClient, AdbConfig, AdbDevice,
    AdbStatus, VpnOptions, CLIENT_PACKAGE, DEFAULT_PORT, REQUIRED_APK_VERSION_CODE,
};
pub use crate::error::{
    Cmd, CommandExecutionError, ProcessIoError, ProcessStatusError, Termination,
};
pub use crate::monitor::{AdbMonitor, AdbMonitorCallback};
