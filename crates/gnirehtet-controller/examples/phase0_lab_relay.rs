//! Headless Phase 0 lab: R1/R2/R5 + APK_MISSING + **P0-R4** foreign PORT_IN_USE
//! + **P0-Q1** quit/`clear_owned_relay` (no orphan; do not kill adb server).
//!
//! ```bash
//! ./scripts/phase0_lab_relay.sh
//! ```

use gnirehtet_adb::AdbConfig;
use gnirehtet_controller::{
    probe_relay_port, ControllerConfig, ControllerError, RelayStdio, SessionController,
    DEFAULT_RELAY_PORT,
};
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Stdio};
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

    // -------------------------------------------------------------------------
    // P0-R4: foreign listener → PORT_IN_USE; owns_relay=false; foreign still alive
    // -------------------------------------------------------------------------
    let mut r4_ok = false;
    {
        let foreign = TcpListener::bind(("127.0.0.1", 0)).expect("foreign bind");
        let fport = foreign.local_addr().unwrap().port();
        report.push(format!("P0-R4_foreign_port={fport}"));

        let mut c = SessionController::new(ControllerConfig {
            adb: AdbConfig::default(),
            gnirehtet_path: bin.clone(),
        });

        match c.start_relay_with_stdio(Some(fport)) {
            Ok(_) => {
                report.push("P0-R4_start_relay_with_stdio=unexpected_ok".into());
                let _ = c.stop_relay();
            }
            Err(e) => {
                let is_piu = matches!(e, ControllerError::PortInUse { .. });
                let ux = e.ux_code();
                report.push(format!("P0-R4_err={e}"));
                report.push(format!("P0-R4_is_PortInUse={is_piu}"));
                report.push(format!("P0-R4_ux_code={ux:?}"));
                report.push(format!("P0-R4_owns_relay={}", c.owns_relay()));

                // Foreign listener must still hold the port (we must not kill it).
                let still_busy = probe_relay_port(fport).is_err();
                let foreign_alive = foreign.local_addr().is_ok() && still_busy;
                report.push(format!("P0-R4_foreign_still_alive={foreign_alive}"));
                report.push(format!("P0-R4_port_still_busy={still_busy}"));

                // Same policy via inherited-stdio start_relay.
                let mut start_relay_piu = false;
                match c.start_relay(Some(fport)) {
                    Err(e2) => {
                        start_relay_piu = matches!(e2, ControllerError::PortInUse { .. })
                            && e2.ux_code() == Some("PORT_IN_USE");
                        report.push(format!(
                            "P0-R4_start_relay_also_PortInUse={start_relay_piu}"
                        ));
                        report.push(format!("P0-R4_owns_after_start_relay={}", c.owns_relay()));
                    }
                    Ok(_) => {
                        report.push("P0-R4_start_relay=unexpected_ok".into());
                        let _ = c.stop_relay();
                    }
                }

                r4_ok = is_piu
                    && ux == Some("PORT_IN_USE")
                    && start_relay_piu
                    && !c.owns_relay()
                    && foreign_alive
                    && still_busy;
            }
        }
        // Drop foreign last so it stays alive through the checks above.
        drop(foreign);
        report.push(format!("P0-R4_pass={r4_ok}"));
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

    // -------------------------------------------------------------------------
    // P0-Q1: quit path = clear_owned_relay → process gone, port free; no adb kill
    // -------------------------------------------------------------------------
    let mut q1_ok = false;
    {
        let qport = if probe_relay_port(DEFAULT_RELAY_PORT).is_ok() {
            DEFAULT_RELAY_PORT
        } else {
            free_ephemeral_port()
        };
        let mut q = SessionController::new(ControllerConfig {
            adb: AdbConfig::default(),
            gnirehtet_path: bin.clone(),
        });
        match q.start_relay_with_stdio(Some(qport)) {
            Ok(stdio) => {
                let _ = drain_stdio(stdio);
                let qpid = q.owned_relay_pid().unwrap_or(0);
                report.push(format!("P0-Q1_owns_before={}", q.owns_relay()));
                report.push(format!("P0-Q1_pid={qpid}"));
                report.push(format!("P0-Q1_port={qport}"));

                let adb_before = adb_server_pids();
                report.push(format!("P0-Q1_adb_pids_before={adb_before:?}"));

                // Quit path (controller-level; Desktop maps quit → clear_owned_relay).
                q.clear_owned_relay();
                report.push(format!("P0-Q1_owns_after_clear={}", q.owns_relay()));

                // Intentional clear must not surface as crash via poll (no owned child).
                let poll_after = q.poll_owned_relay().ok().flatten();
                report.push(format!(
                    "P0-Q1_poll_after_clear_is_none={}",
                    poll_after.is_none()
                ));

                thread::sleep(Duration::from_millis(300));
                let proc_gone = qpid == 0 || !process_alive(qpid);
                let port_free_q = probe_relay_port(qport).is_ok();
                report.push(format!("P0-Q1_process_gone={proc_gone}"));
                report.push(format!("P0-Q1_port_free={port_free_q}"));

                let adb_after = adb_server_pids();
                report.push(format!("P0-Q1_adb_pids_after={adb_after:?}"));
                // clear_owned_relay must not kill a shared adb server (P0-Q3 policy).
                let adb_untouched = adb_before == adb_after;
                report.push(format!("P0-Q1_adb_server_untouched={adb_untouched}"));
                if adb_before.is_empty() {
                    report.push("P0-Q1_adb_note=no_adb_server_present_lab_ok".into());
                }

                q1_ok = !q.owns_relay()
                    && poll_after.is_none()
                    && proc_gone
                    && port_free_q
                    && adb_untouched;
            }
            Err(e) => {
                report.push(format!("P0-Q1_start_ERR={e}"));
            }
        }
        report.push(format!("P0-Q1_pass={q1_ok}"));
    }

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
    report.push(format!("gate_R4={r4_ok}"));
    report.push(format!("gate_R5={r5_ok}"));
    report.push(format!("gate_Q1={q1_ok}"));
    report.push(format!("gate_APK_MISSING={apk_ok}"));
    let all = r1 && r2 && r4_ok && r5_ok && q1_ok && apk_ok;
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

fn process_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Best-effort PIDs of an `adb` server (empty if adb absent / not running).
fn adb_server_pids() -> Vec<u32> {
    let out = Command::new("pgrep")
        .args(["-f", "adb.*fork-server|adb.*server"])
        .output();
    let Ok(out) = out else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect()
}
