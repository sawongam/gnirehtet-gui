//! Routes `log` records to stderr and, for this app's own components, to the
//! in-app log view.

use crate::session::EventSink;
use crate::types::{LogEntry, LogLevel, LogSource};
use log::{Level, LevelFilter, Log, Metadata, Record};
use std::sync::{Arc, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static SINK: OnceLock<Arc<dyn EventSink>> = OnceLock::new();
static LOGGER: AppLogger = AppLogger;

struct AppLogger;

/// Our components log with short tags ("Session", "AdbClient", ...);
/// dependencies use module paths and are kept out of the UI.
fn is_app_target(target: &str) -> bool {
    !target.contains("::")
}

impl Log for AppLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        if is_app_target(metadata.target()) {
            metadata.level() <= Level::Info
        } else {
            metadata.level() <= Level::Warn
        }
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let message = record.args().to_string();
        eprintln!("[{}] {}: {}", record.level(), record.target(), message);
        if !is_app_target(record.target()) {
            return;
        }
        if let Some(sink) = SINK.get() {
            sink.log(LogEntry {
                timestamp_ms: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0),
                level: match record.level() {
                    Level::Error => LogLevel::Error,
                    Level::Warn => LogLevel::Warn,
                    Level::Info => LogLevel::Info,
                    Level::Debug | Level::Trace => LogLevel::Debug,
                },
                source: LogSource::App,
                message,
            });
        }
    }

    fn flush(&self) {}
}

pub fn init(sink: Arc<dyn EventSink>) {
    let _ = SINK.set(sink);
    if log::set_logger(&LOGGER).is_ok() {
        log::set_max_level(LevelFilter::Info);
    }
}
