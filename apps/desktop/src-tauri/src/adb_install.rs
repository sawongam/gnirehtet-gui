//! Downloads Google's Android platform-tools into the app's data directory,
//! so people without adb can get it with one click.
//!
//! The archive comes straight from Google over HTTPS and is unpacked into a
//! fresh directory that replaces the previous one only once it is complete.
//! Nothing outside the app's own data directory is touched, so no admin
//! rights are needed.

use crate::error::{CommandError, CommandResult};
use crate::paths;
use log::{info, warn};
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const TAG: &str = "AdbInstall";

/// The real archive is 7–15 MB; anything far bigger is not what we expect.
const MAX_DOWNLOAD_BYTES: u64 = 200 * 1024 * 1024;

/// Progress is reported at most this often (in bytes received).
const PROGRESS_STEP: u64 = 128 * 1024;

static INSTALLING: AtomicBool = AtomicBool::new(false);

/// Google's "latest platform-tools" archive for this platform, if it has one.
/// Google only builds Linux platform-tools for x86-64.
pub fn download_url() -> Option<&'static str> {
    macro_rules! latest {
        ($os:literal) => {
            concat!(
                "https://dl.google.com/android/repository/platform-tools-latest-",
                $os,
                ".zip"
            )
        };
    }
    if cfg!(target_os = "windows") {
        Some(latest!("windows"))
    } else if cfg!(target_os = "macos") {
        Some(latest!("darwin"))
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Some(latest!("linux"))
    } else {
        None
    }
}

/// Download platform-tools into `tools_dir` and return the adb inside it.
/// `progress` receives (bytes received, total bytes if known).
pub fn install(tools_dir: &Path, progress: impl Fn(u64, Option<u64>)) -> CommandResult<PathBuf> {
    let url = download_url().ok_or_else(|| {
        CommandError::new(
            "ADB_INSTALL_UNSUPPORTED",
            "Google does not publish platform-tools for this platform.",
        )
    })?;
    if INSTALLING.swap(true, Ordering::SeqCst) {
        return Err(CommandError::new(
            "ADB_INSTALL_BUSY",
            "adb is already being installed.",
        ));
    }
    let result = install_from(url, tools_dir, &progress);
    INSTALLING.store(false, Ordering::SeqCst);
    if let Err(err) = &result {
        warn!(target: TAG, "Installing adb failed: {}", err.message);
    }
    result
}

fn install_from(
    url: &str,
    tools_dir: &Path,
    progress: &dyn Fn(u64, Option<u64>),
) -> CommandResult<PathBuf> {
    let parent = tools_dir
        .parent()
        .ok_or_else(|| failed("the data directory is invalid"))?;
    fs::create_dir_all(parent)
        .map_err(|e| failed(format!("cannot create {}: {e}", parent.display())))?;

    let archive = parent.join("platform-tools.zip.part");
    let staging = parent.join("platform-tools.new");

    info!(target: TAG, "Downloading {url}");
    let result = download(url, &archive, progress)
        .and_then(|()| extract(&archive, &staging))
        .and_then(|()| replace_dir(&staging, tools_dir));
    let _ = fs::remove_file(&archive);
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result?;

    let adb = paths::adb_in(tools_dir);
    info!(target: TAG, "Installed {}", adb.display());
    Ok(adb)
}

fn failed(message: impl Into<String>) -> CommandError {
    CommandError::new("ADB_INSTALL_FAILED", message)
}

fn download_failed(message: impl Into<String>) -> CommandError {
    CommandError::new("ADB_DOWNLOAD_FAILED", message)
}

fn download(url: &str, dest: &Path, progress: &dyn Fn(u64, Option<u64>)) -> CommandResult<()> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(15))
        .timeout_read(Duration::from_secs(30))
        .build();
    let response = agent
        .get(url)
        .call()
        .map_err(|e| download_failed(format!("could not download platform-tools: {e}")))?;
    let total = response
        .header("Content-Length")
        .and_then(|len| len.parse::<u64>().ok());
    if total.is_some_and(|len| len > MAX_DOWNLOAD_BYTES) {
        return Err(download_failed("the download is unexpectedly large"));
    }

    let mut reader = response.into_reader().take(MAX_DOWNLOAD_BYTES + 1);
    let file =
        File::create(dest).map_err(|e| failed(format!("cannot write {}: {e}", dest.display())))?;
    let mut writer = BufWriter::new(file);
    let mut buf = vec![0u8; 64 * 1024];
    let mut received = 0u64;
    let mut reported = 0u64;
    progress(0, total);
    loop {
        let n = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => {
                return Err(download_failed(format!(
                    "the download was interrupted: {e}"
                )))
            }
        };
        writer
            .write_all(&buf[..n])
            .map_err(|e| failed(format!("cannot write {}: {e}", dest.display())))?;
        received += n as u64;
        if received > MAX_DOWNLOAD_BYTES {
            return Err(download_failed("the download is unexpectedly large"));
        }
        if received - reported >= PROGRESS_STEP {
            reported = received;
            progress(received, total);
        }
    }
    writer
        .flush()
        .map_err(|e| failed(format!("cannot write {}: {e}", dest.display())))?;
    if total.is_some_and(|len| len != received) {
        return Err(download_failed("the download ended early"));
    }
    progress(received, total);
    Ok(())
}

/// Unpack the `platform-tools/` folder of the archive into `dest`.
fn extract(archive: &Path, dest: &Path) -> CommandResult<()> {
    let bad = |e: &dyn std::fmt::Display| {
        download_failed(format!("the download is not a valid archive: {e}"))
    };
    let _ = fs::remove_dir_all(dest);
    fs::create_dir_all(dest)
        .map_err(|e| failed(format!("cannot create {}: {e}", dest.display())))?;

    let file = File::open(archive).map_err(|e| failed(e.to_string()))?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| bad(&e))?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| bad(&e))?;
        // `enclosed_name` rejects absolute paths and `..`.
        let Some(name) = entry.enclosed_name() else {
            continue;
        };
        let Ok(relative) = name.strip_prefix("platform-tools") else {
            continue;
        };
        if relative.as_os_str().is_empty() {
            continue;
        }
        let out = dest.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&out).map_err(|e| failed(e.to_string()))?;
            continue;
        }
        if let Some(dir) = out.parent() {
            fs::create_dir_all(dir).map_err(|e| failed(e.to_string()))?;
        }
        let mut target = File::create(&out)
            .map_err(|e| failed(format!("cannot write {}: {e}", out.display())))?;
        io::copy(&mut entry, &mut target).map_err(|e| bad(&e))?;
        #[cfg(unix)]
        if let Some(mode) = entry.unix_mode() {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&out, fs::Permissions::from_mode(mode & 0o755))
                .map_err(|e| failed(e.to_string()))?;
        }
    }

    if !paths::adb_in(dest).is_file() {
        return Err(download_failed("the archive does not contain adb"));
    }
    Ok(())
}

fn replace_dir(staging: &Path, tools_dir: &Path) -> CommandResult<()> {
    if tools_dir.exists() {
        fs::remove_dir_all(tools_dir).map_err(|e| {
            failed(format!(
                "cannot replace {} (is adb still running?): {e}",
                tools_dir.display()
            ))
        })?;
    }
    fs::rename(staging, tools_dir)
        .map_err(|e| failed(format!("cannot move platform-tools into place: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use zip::write::SimpleFileOptions;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("gnirehtet-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_zip(path: &Path, entries: &[(&str, &[u8])]) {
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut cursor);
            let options = SimpleFileOptions::default().unix_permissions(0o755);
            for (name, data) in entries {
                zip.start_file(*name, options).unwrap();
                zip.write_all(data).unwrap();
            }
            zip.finish().unwrap();
        }
        fs::write(path, cursor.into_inner()).unwrap();
    }

    #[test]
    fn extracts_platform_tools_and_skips_everything_else() {
        let dir = temp_dir("extract");
        let archive = dir.join("pt.zip");
        let adb = format!("platform-tools/adb{}", std::env::consts::EXE_SUFFIX);
        write_zip(
            &archive,
            &[
                (adb.as_str(), b"adb"),
                ("platform-tools/lib64/libc++.so", b"lib"),
                ("other/readme.txt", b"no"),
                ("../escape.txt", b"no"),
            ],
        );
        let dest = dir.join("out");
        extract(&archive, &dest).unwrap();
        assert!(paths::adb_in(&dest).is_file());
        assert!(dest.join("lib64/libc++.so").is_file());
        assert!(!dest.join("readme.txt").exists());
        assert!(!dir.join("escape.txt").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn archive_without_adb_is_rejected() {
        let dir = temp_dir("noadb");
        let archive = dir.join("pt.zip");
        write_zip(&archive, &[("platform-tools/fastboot", b"x")]);
        let err = extract(&archive, &dir.join("out")).unwrap_err();
        assert_eq!(err.code, "ADB_DOWNLOAD_FAILED");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn replacing_swaps_the_whole_directory() {
        let dir = temp_dir("replace");
        let staging = dir.join("new");
        let tools = dir.join("platform-tools");
        fs::create_dir_all(&staging).unwrap();
        fs::write(staging.join("adb"), b"new").unwrap();
        fs::create_dir_all(&tools).unwrap();
        fs::write(tools.join("stale"), b"old").unwrap();
        replace_dir(&staging, &tools).unwrap();
        assert!(tools.join("adb").is_file());
        assert!(!tools.join("stale").exists());
        assert!(!staging.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    /// Hits the network: `cargo test -- --ignored real_download`.
    #[test]
    #[ignore]
    fn real_download_installs_a_working_adb() {
        let dir = temp_dir("real");
        let tools = dir.join("platform-tools");
        let adb = install(&tools, |received, total| {
            eprintln!("{received} / {total:?}");
        })
        .unwrap();
        let output = std::process::Command::new(&adb)
            .arg("version")
            .output()
            .unwrap();
        assert!(output.status.success());
        eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        let _ = fs::remove_dir_all(&dir);
    }
}
