//! Interpretation of the relay's log output.
//!
//! The relay prints lines like
//! `2026-01-01 12:00:00.000 INFO TunnelServer: Client #0 connected`.
//! The Android client only opens its relay connection once the VPN is
//! established, so "Client #N connected" is the signal that the device is
//! actually using the computer's network.

use crate::types::LogLevel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayLine {
    pub level: LogLevel,
    /// Message without the timestamp/level prefix.
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientChange {
    Connected(u32),
    Disconnected(u32),
}

/// Split a relay output line into level and message. Lines that do not follow
/// the relay format keep `fallback` as their level.
pub fn parse_line(line: &str, fallback: LogLevel) -> RelayLine {
    let mut parts = line.splitn(4, ' ');
    let (date, time, level, rest) = (parts.next(), parts.next(), parts.next(), parts.next());
    let looks_timestamped = date.is_some_and(|d| d.len() == 10 && d.as_bytes()[4] == b'-')
        && time.is_some_and(|t| t.contains(':'));
    let level = match level {
        Some("TRACE" | "DEBUG") => Some(LogLevel::Debug),
        Some("INFO") => Some(LogLevel::Info),
        Some("WARN") => Some(LogLevel::Warn),
        Some("ERROR") => Some(LogLevel::Error),
        _ => None,
    };
    match (looks_timestamped, level, rest) {
        (true, Some(level), Some(rest)) => RelayLine {
            level,
            message: rest.to_string(),
        },
        _ => RelayLine {
            level: fallback,
            message: line.to_string(),
        },
    }
}

/// Detect `Client #<id> connected` / `Client #<id> disconnected`.
pub fn client_change(message: &str) -> Option<ClientChange> {
    let rest = &message[message.find("Client #")? + "Client #".len()..];
    let digits_end = rest.find(|c: char| !c.is_ascii_digit())?;
    let id = rest[..digits_end].parse().ok()?;
    match rest[digits_end..].trim() {
        "connected" => Some(ClientChange::Connected(id)),
        "disconnected" => Some(ClientChange::Disconnected(id)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_relay_format() {
        let line = parse_line(
            "2026-09-27 03:29:44.592 INFO Relay: Relay server started",
            LogLevel::Warn,
        );
        assert_eq!(line.level, LogLevel::Info);
        assert_eq!(line.message, "Relay: Relay server started");

        let line = parse_line(
            "2026-09-27 03:29:44.592 ERROR Main: Execution error: Address in use",
            LogLevel::Info,
        );
        assert_eq!(line.level, LogLevel::Error);
    }

    #[test]
    fn keeps_unformatted_lines() {
        let line = parse_line("thread 'main' panicked", LogLevel::Warn);
        assert_eq!(line.level, LogLevel::Warn);
        assert_eq!(line.message, "thread 'main' panicked");
        assert_eq!(parse_line("", LogLevel::Info).message, "");
    }

    #[test]
    fn detects_client_changes() {
        assert_eq!(
            client_change("TunnelServer: Client #0 connected"),
            Some(ClientChange::Connected(0))
        );
        assert_eq!(
            client_change("TunnelServer: Client #12 disconnected"),
            Some(ClientChange::Disconnected(12))
        );
        assert_eq!(client_change("Client #3 connected, sending"), None);
        assert_eq!(client_change("Client # connected"), None);
        assert_eq!(client_change("Relay server started"), None);
    }
}
