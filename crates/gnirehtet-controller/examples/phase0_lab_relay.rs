//! Headless Phase 0 lab: start_relay_with_stdio → owns/pid → stop → port free;
//! mid-run kill → poll_owned_relay exited (P0-R1/R2/R5); APK_MISSING on install.
//!
//! ```bash
//! ./scripts/phase0_lab_relay.sh
//! ```

use gnirehtet_adb::AdbConfig;
use gnirehtet_controller::{
    probe_relay_port, ControllerConfig, RelayStdio, SessionController, DEFAULT_RELAY_PORT,
};
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn main() {
    let mut report: Vec<String> = Vec::new();
    let bin = std::env::var_os("GNIREHTET_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| ControllerConfig::default().gnirehtet_path.clone());

    report.push(format!(
        "date_utc_unix={}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    ));
    report.push(format!("gnirehtet_bin={}", bin.display()));
    report.push(format!("bin_is_file={}", bin.is_file()));
    let apk = AdbConfig::default().apk_path;
    report.push(format!("apk_path={}", apk.display()));
    report.push(format!("apk_is_file={}", apk.is_file()));

    if !bin.is_file() {
        report.push("RESULT=FAIL reason=GNIREHTET_BIN_missing".into());
        print_report(&report);
        std::process::exit(2);
    }

    let port = if probe_relay_port(DEFAULT_RELAY_PORT).is_ok() {
        DEFAULT_RELAY_PORT
    } else {
        free_ephemeral_port()
    };
    report.push(format!("listen_port={port}"));

    let mut session = SessionController::new(ControllerConfig {
        adb: AdbConfig::default(),
        gnirehtet_path: bin.clone(),
    });

    let stdio = match session.start_relay_with_stdio(Some(port)) {
        Ok(s) => s,
        Err(e) => {
            report.push(format!("start_relay_with_stdio=ERR {e}"));
            report.push(format!("ux_code={:?}", e.ux_code()));
            report.push("RESULT=FAIL reason=start".into());
            print_report(&report);
            std::process::exit(1);
        }
    };

    let pid = session.owned_relay_pid().unwrap_or(0);
    report.push(format!("P0-R1_owns_relay={}", session.owns_relay()));
    report.push(format!("P0-R1_pid={pid}"));
    report.push(format!("P0-R1_session_port={:?}", session.session_port()));

    // Drain pipes on side threads so the child is not killed by SIGPIPE.
    let first_line = drain_stdio(stdio);
    if let Some(line) = first_line {
        report.push(format!("log_line={line}"));
    }

    thread::sleep(Duration::from_millis(250));
    let still = session.owns_relay();
    report.push(format!("P0-R1_still_owned={still}"));

    // P0-R5: kill mid-run; poll_owned_relay → exited within ≤3s
    let mut r5_ok = false;
    if std::env::var_os("PHASE0_SKIP_CRASH").is_none() && pid > 0 && still {
        let _ = Command::new("kill")
            .args(["-9", &pid.to_string()])
            .status();
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            match session.poll_owned_relay() {
                Ok(Some(status)) => {
                    report.push(format!("P0-R5_poll_exited=true status={status}"));
                    report.push(format!(
                        "P0-R5_owns_after_poll={}",
                        session.owns_relay()
                    ));
                    r5_ok = !session.owns_relay();
                    break;
                }
                Ok(None) => thread::sleep(Duration::from_millis(100)),
                Err(e) => {
                    report.push(format!("P0-R5_poll_err={e}"));
                    break;
                }
            }
        }
        report.push(format!("P0-R5_detected_within_3s={r5_ok}"));

        if !session.owns_relay() {
            let port2 = if probe_relay_port(port).is_ok() {
                port
            } else {
                free_ephemeral_port()
            };
            match session.start_relay_with_stdio(Some(port2)) {
                Ok(s2) => {
                    let _ = drain_stdio(s2);
                    report.push(format!("P0-R2_restart_port={port2}"));
                }
                Err(e) => report.push(format!("P0-R2_restart_ERR={e}")),
            }
        }
    } else {
        report.push("P0-R5_skipped=true".into());
        r5_ok = true;
    }

    let stop_port = session.session_port().unwrap_or(port);
    if session.owns_relay() {
        match session.stop_relay() {
            Ok(()) => report.push("P0-R2_stop_relay=ok".into()),
            Err(e) => report.push(format!("P0-R2_stop_relay=ERR {e}")),
        }
    } else {
        session.clear_owned_relay();
        report.push("P0-R2_clear_owned_relay=ok".into());
    }
    report.push(format!(
        "P0-R2_owns_after_stop={}",
        session.owns_relay()
    ));
    thread::sleep(Duration::from_millis(300));
    let port_free = probe_relay_port(stop_port).is_ok();
    report.push(format!("P0-R2_port_free_after_stop={port_free}"));

    {
        let s2 = SessionController::new(ControllerConfig {
            adb: AdbConfig {
                apk_path: PathBuf::from("/no/such/gnirehtet-lab.apk"),
                ..AdbConfig::default()
            },
            gnirehtet_path: bin,
        });
        match s2.install(None) {
            Err(e) => {
                report.push(format!("APK_MISSING_err={e}"));
                report.push(format!("APK_MISSING_ux={:?}", e.ux_code()));
            }
            Ok(()) => report.push("APK_MISSING_unexpected_ok".into()),
        }
    }

    let r1 = report.iter().any(|l| l == "P0-R1_owns_relay=true");
    let r2 = report.iter().any(|l| l == "P0-R2_owns_after_stop=false")
        && report.iter().any(|l| l == "P0-R2_port_free_after_stop=true");
    let apk_ok = report
        .iter()
        .any(|l| l.contains("APK_MISSING_ux=Some(\"APK_MISSING\")"));

    report.push(format!("gate_R1={r1}"));
    report.push(format!("gate_R2={r2}"));
    report.push(format!("gate_R5={r5_ok}"));
    report.push(format!("gate_APK_MISSING={apk_ok}"));
    let all = r1 && r2 && r5_ok && apk_ok;
    report.push(if all {
        "RESULT=PASS".into()
    } else {
        "RESULT=FAIL".into()
    });

    print_report(&report);
    std::process::exit(if all { 0 } else { 1 });
}

/// Drain stdout/stderr on background threads; return first stdout line if any soon.
fn drain_stdio(stdio: RelayStdio) -> Option<String> {
    let RelayStdio { stdout, stderr } = stdio;
    let (tx, rx) = std::sync::mpsc::channel();
    thread::spawn(move || {
        let mut first = true;
        for line in BufReader::new(stdout).lines().flatten() {
            if first {
                let _ = tx.send(line);
                first = false;
            }
        }
    });
    thread::spawn(move || {
        for _ in BufReader::new(stderr).lines() {}
    });
    rx.recv_timeout(Duration::from_millis(800)).ok()
}

fn print_report(lines: &[String]) {
    for l in lines {
        println!("{l}");
    }
}

fn free_ephemeral_port() -> u16 {
    for _ in 0..32 {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        if probe_relay_port(port).is_ok() {
            return port;
        }
    }
    panic!("no free ephemeral port");
}
