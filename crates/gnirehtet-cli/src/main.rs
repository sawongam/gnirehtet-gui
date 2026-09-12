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
 * Phase 1: thin CLI dispatching to gnirehtet-adb::AdbClient and relaylib::relay.
 */

extern crate chrono;
extern crate ctrlc;
#[macro_use]
extern crate log;
extern crate gnirehtet_adb as adb;
extern crate relaylib;

mod cli_args;
mod logger;

use crate::cli_args::CommandLineArguments;
use adb::{AdbClient, CommandExecutionError, VpnOptions};
use std::env;
use std::process::exit;
use std::thread;

const TAG: &str = "Main";

const COMMANDS: &[&dyn Command] = &[
    &InstallCommand,
    &UninstallCommand,
    &ReinstallCommand,
    &RunCommand,
    &AutorunCommand,
    &StartCommand,
    &AutostartCommand,
    &StopCommand,
    &RestartCommand,
    &TunnelCommand,
    &RelayCommand,
];

trait Command {
    fn command(&self) -> &'static str;
    fn accepted_parameters(&self) -> u8;
    fn description(&self) -> &'static str;
    fn execute(&self, args: &CommandLineArguments) -> Result<(), CommandExecutionError>;
}

struct InstallCommand;
struct UninstallCommand;
struct ReinstallCommand;
struct RunCommand;
struct AutorunCommand;
struct StartCommand;
struct AutostartCommand;
struct StopCommand;
struct RestartCommand;
struct TunnelCommand;
struct RelayCommand;

fn client() -> AdbClient {
    AdbClient::from_env()
}

impl Command for InstallCommand {
    fn command(&self) -> &'static str {
        "install"
    }
    fn accepted_parameters(&self) -> u8 {
        cli_args::PARAM_SERIAL
    }
    fn description(&self) -> &'static str {
        "Install the client on the Android device and exit.\n\
         If several devices are connected via adb, then serial must be\n\
         specified."
    }
    fn execute(&self, args: &CommandLineArguments) -> Result<(), CommandExecutionError> {
        client().install(args.serial())
    }
}

impl Command for UninstallCommand {
    fn command(&self) -> &'static str {
        "uninstall"
    }
    fn accepted_parameters(&self) -> u8 {
        cli_args::PARAM_SERIAL
    }
    fn description(&self) -> &'static str {
        "Uninstall the client from the Android device and exit.\n\
         If several devices are connected via adb, then serial must be\n\
         specified."
    }
    fn execute(&self, args: &CommandLineArguments) -> Result<(), CommandExecutionError> {
        client().uninstall(args.serial())
    }
}

impl Command for ReinstallCommand {
    fn command(&self) -> &'static str {
        "reinstall"
    }
    fn accepted_parameters(&self) -> u8 {
        cli_args::PARAM_SERIAL
    }
    fn description(&self) -> &'static str {
        "Uninstall then install."
    }
    fn execute(&self, args: &CommandLineArguments) -> Result<(), CommandExecutionError> {
        client().reinstall(args.serial())
    }
}

impl Command for RunCommand {
    fn command(&self) -> &'static str {
        "run"
    }
    fn accepted_parameters(&self) -> u8 {
        cli_args::PARAM_SERIAL
            | cli_args::PARAM_DNS_SERVERS
            | cli_args::PARAM_ROUTES
            | cli_args::PARAM_PORT
    }
    fn description(&self) -> &'static str {
        "Enable reverse tethering for exactly one device:\n  \
         - install the client if necessary;\n  \
         - start the client;\n  \
         - start the relay server;\n  \
         - on Ctrl+C, stop both the relay server and the client."
    }
    fn execute(&self, args: &CommandLineArguments) -> Result<(), CommandExecutionError> {
        cmd_run(
            args.serial(),
            args.dns_servers(),
            args.routes(),
            args.port(),
        )
    }
}

impl Command for AutorunCommand {
    fn command(&self) -> &'static str {
        "autorun"
    }
    fn accepted_parameters(&self) -> u8 {
        cli_args::PARAM_DNS_SERVERS | cli_args::PARAM_ROUTES | cli_args::PARAM_PORT
    }
    fn description(&self) -> &'static str {
        "Enable reverse tethering for all devices:\n  \
         - monitor devices and start clients (autostart);\n  \
         - start the relay server."
    }
    fn execute(&self, args: &CommandLineArguments) -> Result<(), CommandExecutionError> {
        cmd_autorun(args.dns_servers(), args.routes(), args.port())
    }
}

impl Command for StartCommand {
    fn command(&self) -> &'static str {
        "start"
    }
    fn accepted_parameters(&self) -> u8 {
        cli_args::PARAM_SERIAL
            | cli_args::PARAM_DNS_SERVERS
            | cli_args::PARAM_ROUTES
            | cli_args::PARAM_PORT
    }
    fn description(&self) -> &'static str {
        "Start a client on the Android device and exit.\n\
         If several devices are connected via adb, then serial must be\n\
         specified.\n\
         If -d is given, then make the Android device use the specified\n\
         DNS server(s). Otherwise, use 8.8.8.8 (Google public DNS).\n\
         If -r is given, then only reverse tether the specified routes.\n\
         Otherwise, use 0.0.0.0/0 (redirect the whole traffic).\n\
         If -p is given, then make the relay server listen on the specified\n\
         port. Otherwise, use port 31416.\n\
         If the client is already started, then do nothing, and ignore\n\
         the other parameters.\n\
         10.0.2.2 is mapped to the host 'localhost'."
    }
    fn execute(&self, args: &CommandLineArguments) -> Result<(), CommandExecutionError> {
        let opts = VpnOptions {
            dns_servers: args.dns_servers(),
            routes: args.routes(),
            port: args.port(),
        };
        client().start(args.serial(), &opts)
    }
}

impl Command for AutostartCommand {
    fn command(&self) -> &'static str {
        "autostart"
    }
    fn accepted_parameters(&self) -> u8 {
        cli_args::PARAM_DNS_SERVERS | cli_args::PARAM_ROUTES | cli_args::PARAM_PORT
    }
    fn description(&self) -> &'static str {
        "Listen for device connexions and start a client on every detected\n\
         device.\n\
         Accept the same parameters as the start command (excluding the\n\
         serial, which will be taken from the detected device)."
    }
    fn execute(&self, args: &CommandLineArguments) -> Result<(), CommandExecutionError> {
        cmd_autostart(args.dns_servers(), args.routes(), args.port())
    }
}

impl Command for StopCommand {
    fn command(&self) -> &'static str {
        "stop"
    }
    fn accepted_parameters(&self) -> u8 {
        cli_args::PARAM_SERIAL
    }
    fn description(&self) -> &'static str {
        "Stop the client on the Android device and exit.\n\
         If several devices are connected via adb, then serial must be\n\
         specified."
    }
    fn execute(&self, args: &CommandLineArguments) -> Result<(), CommandExecutionError> {
        client().stop(args.serial())
    }
}

impl Command for RestartCommand {
    fn command(&self) -> &'static str {
        "restart"
    }
    fn accepted_parameters(&self) -> u8 {
        cli_args::PARAM_SERIAL
            | cli_args::PARAM_DNS_SERVERS
            | cli_args::PARAM_ROUTES
            | cli_args::PARAM_PORT
    }
    fn description(&self) -> &'static str {
        "Stop then start."
    }
    fn execute(&self, args: &CommandLineArguments) -> Result<(), CommandExecutionError> {
        let opts = VpnOptions {
            dns_servers: args.dns_servers(),
            routes: args.routes(),
            port: args.port(),
        };
        client().restart(args.serial(), &opts)
    }
}

impl Command for TunnelCommand {
    fn command(&self) -> &'static str {
        "tunnel"
    }
    fn accepted_parameters(&self) -> u8 {
        cli_args::PARAM_SERIAL | cli_args::PARAM_PORT
    }
    fn description(&self) -> &'static str {
        "Set up the 'adb reverse' tunnel.\n\
         If a device is unplugged then plugged back while gnirehtet is\n\
         active, resetting the tunnel is sufficient to get the\n\
         connection back."
    }
    fn execute(&self, args: &CommandLineArguments) -> Result<(), CommandExecutionError> {
        client().tunnel(args.serial(), args.port())
    }
}

impl Command for RelayCommand {
    fn command(&self) -> &'static str {
        "relay"
    }
    fn accepted_parameters(&self) -> u8 {
        cli_args::PARAM_NONE | cli_args::PARAM_PORT
    }
    fn description(&self) -> &'static str {
        "Start the relay server in the current terminal."
    }
    fn execute(&self, args: &CommandLineArguments) -> Result<(), CommandExecutionError> {
        cmd_relay(args.port())?;
        Ok(())
    }
}

fn cmd_run(
    serial: Option<&str>,
    dns_servers: Option<&str>,
    routes: Option<&str>,
    port: u16,
) -> Result<(), CommandExecutionError> {
    let adb = client();
    adb.async_start(serial, dns_servers, routes, port);

    let ctrlc_serial = serial.map(String::from);
    ctrlc::set_handler(move || {
        info!(target: TAG, "Interrupted");
        let serial = ctrlc_serial.as_ref().map(String::as_ref);
        if let Err(err) = AdbClient::from_env().stop(serial) {
            error!(target: TAG, "Cannot stop client: {}", err);
        }
        exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    cmd_relay(port)
}

fn cmd_autorun(
    dns_servers: Option<&str>,
    routes: Option<&str>,
    port: u16,
) -> Result<(), CommandExecutionError> {
    {
        let autostart_dns_servers = dns_servers.map(String::from);
        let autostart_routes = routes.map(String::from);
        thread::spawn(move || {
            let dns_servers = autostart_dns_servers.as_ref().map(String::as_ref);
            let routes = autostart_routes.as_ref().map(String::as_ref);
            if let Err(err) = cmd_autostart(dns_servers, routes, port) {
                error!(target: TAG, "Cannot auto start clients: {}", err);
            }
        });
    }
    cmd_relay(port)
}

fn cmd_autostart(
    dns_servers: Option<&str>,
    routes: Option<&str>,
    port: u16,
) -> Result<(), CommandExecutionError> {
    let adb = client();
    let start_dns_servers = dns_servers.map(String::from);
    let start_routes = routes.map(String::from);
    let mut adb_monitor = adb.monitor(Box::new(move |serial: &str| {
        let dns_servers = start_dns_servers.as_ref().map(String::as_ref);
        let routes = start_routes.as_ref().map(String::as_ref);
        AdbClient::from_env().async_start(Some(serial), dns_servers, routes, port);
    }));
    adb_monitor.monitor();
    Ok(())
}

fn cmd_relay(port: u16) -> Result<(), CommandExecutionError> {
    info!(target: TAG, "Starting relay server on port {}...", port);
    // Phase 1 CLI parity: in-process relaylib::relay. GUI will spawn stock binary later.
    relaylib::relay(port)?;
    Ok(())
}

fn print_usage() {
    let mut msg = "Syntax: gnirehtet (".to_string();
    msg.push_str(COMMANDS[0].command());
    for command in &COMMANDS[1..] {
        msg.push('|');
        msg.push_str(command.command());
    }
    msg.push_str(") ...\n");
    for &command in COMMANDS {
        msg.push('\n');
        append_command_usage(&mut msg, command);
    }
    eprint!("{}", msg);
}

fn append_command_usage(msg: &mut String, command: &dyn Command) {
    msg.push_str("  gnirehtet ");
    msg.push_str(command.command());
    let accepted_parameters = command.accepted_parameters();
    if (accepted_parameters & cli_args::PARAM_SERIAL) != 0 {
        msg.push_str(" [serial]");
    }
    if (accepted_parameters & cli_args::PARAM_DNS_SERVERS) != 0 {
        msg.push_str(" [-d DNS[,DNS2,...]]");
    }
    if (accepted_parameters & cli_args::PARAM_PORT) != 0 {
        msg.push_str(" [-p PORT]");
    }
    if (accepted_parameters & cli_args::PARAM_ROUTES) != 0 {
        msg.push_str(" [-r ROUTE[,ROUTE2,...]]");
    }
    msg.push('\n');
    for desc_line in command.description().split('\n') {
        msg.push_str("      ");
        msg.push_str(desc_line);
        msg.push('\n');
    }
}

fn print_command_usage(command: &dyn Command) {
    let mut msg = String::new();
    append_command_usage(&mut msg, command);
    eprint!("{}", msg);
}

fn main() {
    logger::init().unwrap();
    let mut args = env::args();
    if let Some(command_name) = args.nth(1) {
        let command = COMMANDS
            .iter()
            .find(|&&command| command.command() == command_name);
        match command {
            Some(&command) => {
                let arguments =
                    CommandLineArguments::parse(command.accepted_parameters(), args.collect());
                match arguments {
                    Ok(arguments) => {
                        if let Err(err) = command.execute(&arguments) {
                            error!(target: TAG, "Execution error: {}", err);
                            exit(3);
                        }
                    }
                    Err(err) => {
                        error!(target: TAG, "{}", err);
                        print_command_usage(command);
                        exit(2);
                    }
                }
            }
            None => {
                if command_name == "rt" {
                    error!(
                        target: TAG,
                        "The 'rt' command has been renamed to 'run'. Try 'gnirehtet run' instead."
                    );
                    print_command_usage(&RunCommand);
                } else {
                    error!(target: TAG, "Unknown command: {}", command_name);
                    print_usage();
                }
                exit(1);
            }
        }
    } else {
        print_usage();
    }
}
