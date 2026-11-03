//! Files on disk: UTC timestamps, the capped session log and dump files
//! (ADR-T47-1).
//!
//! The session log is opened once and written one line at a time. A line
//! that would push it past its cap first moves it to `<name>.1`, so the
//! two files together never hold more than twice the cap.

use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::shell::diag::Level;

/// Largest size of the session log, and of its one backup.
pub const LOG_CAP: u64 = 512 * 1024;

/// Dump files kept in the state directory.
pub const MAX_DUMPS: usize = 5;

/// `since_epoch` as `YYYY-MM-DDTHH:MM:SS.mmmZ`.
#[must_use]
pub fn utc(since_epoch: Duration) -> String {
    let (date, time) = civil(since_epoch.as_secs());
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        date.0,
        date.1,
        date.2,
        time.0,
        time.1,
        time.2,
        since_epoch.subsec_millis()
    )
}

/// `unix_secs` as `YYYYMMDDTHHMMSSZ`, for file names.
#[must_use]
pub fn utc_stamp(unix_secs: u64) -> String {
    let (date, time) = civil(unix_secs);
    format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
        date.0, date.1, date.2, time.0, time.1, time.2
    )
}

/// `(year, month, day)` and `(hour, minute, second)` of `unix_secs`, in
/// UTC: Howard Hinnant's `civil_from_days`, for days after the epoch.
fn civil(unix_secs: u64) -> ((u64, u64, u64), (u64, u64, u64)) {
    let days = unix_secs / 86_400;
    let secs = unix_secs % 86_400;
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z % 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + u64::from(month <= 2);
    ((year, month, day), (secs / 3600, secs / 60 % 60, secs % 60))
}

/// One session log line: `<utc> +<uptime>s <LEVEL> <message>\n`. Line
/// breaks in `message` become spaces.
#[must_use]
pub fn log_line(unix: Duration, uptime: Duration, level: Level, message: &str) -> String {
    format!(
        "{} +{}.{:03}s {} {}\n",
        utc(unix),
        uptime.as_secs(),
        uptime.subsec_millis(),
        level.tag(),
        message.replace(['\n', '\r'], " ")
    )
}

/// Most characters of a dump's reason in its file name.
const REASON_CHARS: usize = 32;

/// `dump-<stamp>-<reason>.log`, the reason lowercased, other characters
/// than ASCII letters and digits turned into `-`, at most 32 of them.
#[must_use]
pub fn dump_name(unix_secs: u64, reason: &str) -> String {
    let reason: String = reason
        .chars()
        .take(REASON_CHARS)
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    format!("dump-{}-{reason}.log", utc_stamp(unix_secs))
}

/// Whether `name` is a dump file's name.
fn is_dump(name: &str) -> bool {
    name.starts_with("dump-") && Path::new(name).extension().is_some_and(|ext| ext == "log")
}

/// Deletes the oldest `dump-*.log` files in `dir` (by name, which sorts by
/// time) until `keep` are left.
///
/// # Errors
///
/// When `dir` cannot be read or a file cannot be removed.
pub fn prune_dumps(dir: &Path, keep: usize) -> io::Result<()> {
    let mut dumps = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let name = entry?.file_name();
        if name.to_str().is_some_and(is_dump) {
            dumps.push(name);
        }
    }
    dumps.sort();
    let extra = dumps.len().saturating_sub(keep);
    for name in &dumps[..extra] {
        std::fs::remove_file(dir.join(name))?;
    }
    Ok(())
}

/// The session log file, rotated at its cap.
#[derive(Debug)]
pub struct SessionLog {
    path: PathBuf,
    file: File,
    /// Bytes in the file.
    len: u64,
    cap: u64,
}

impl SessionLog {
    /// Opens `path` for appending, creating it and its directory. A file
    /// already at or over `cap` is rotated first.
    ///
    /// # Errors
    ///
    /// When the directory or the file cannot be created.
    pub fn open(path: &Path, cap: u64) -> io::Result<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let existing = std::fs::metadata(path).map_or(0, |meta| meta.len());
        if existing >= cap && existing > 0 {
            std::fs::rename(path, backup(path))?;
        }
        let file = append(path)?;
        let len = file.metadata()?.len();
        Ok(Self {
            path: path.to_path_buf(),
            file,
            len,
            cap,
        })
    }

    /// Appends `line` (cut to the cap), rotating first if it would not fit.
    ///
    /// # Errors
    ///
    /// When the file cannot be renamed, reopened or written.
    pub fn write_line(&mut self, line: &str) -> io::Result<()> {
        let line = cut(line, usize::try_from(self.cap).unwrap_or(usize::MAX));
        let size = line.len() as u64;
        if self.len > 0 && self.len + size > self.cap {
            std::fs::rename(&self.path, backup(&self.path))?;
            self.file = append(&self.path)?;
            self.len = 0;
        }
        self.file.write_all(line.as_bytes())?;
        self.len += size;
        Ok(())
    }

    /// Bytes in the current file.
    #[must_use]
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Whether the current file is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The file's path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// `<path>.1`, where a rotated log goes.
fn backup(path: &Path) -> PathBuf {
    let mut name = OsString::from(path.as_os_str());
    name.push(".1");
    PathBuf::from(name)
}

/// `path` opened for appending, created if missing.
fn append(path: &Path) -> io::Result<File> {
    OpenOptions::new().create(true).append(true).open(path)
}

/// The longest prefix of `text` of at most `max` bytes that ends on a char
/// boundary.
fn cut(text: &str, max: usize) -> &str {
    if text.len() <= max {
        return text;
    }
    let end = (0..=max)
        .rev()
        .find(|&at| text.is_char_boundary(at))
        .unwrap_or(0);
    &text[..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::diag::Level;
    use proptest::prelude::*;
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    /// An empty directory of its own for one test.
    fn temp_dir(name: &str) -> std::io::Result<PathBuf> {
        let dir =
            std::env::temp_dir().join(format!("draw-diag-file-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    fn len(path: &Path) -> u64 {
        std::fs::metadata(path).map_or(0, |m| m.len())
    }

    fn line(bytes: usize) -> String {
        let mut text = "x".repeat(bytes.saturating_sub(1));
        text.push('\n');
        text
    }

    #[test]
    fn utc_formats_known_instants() {
        // Arrange / Act / Assert
        assert_eq!(utc(Duration::ZERO), "1970-01-01T00:00:00.000Z");
        assert_eq!(
            utc(Duration::from_secs(951_782_400)),
            "2000-02-29T00:00:00.000Z"
        );
        assert_eq!(
            utc(Duration::from_millis(4_102_444_799_999)),
            "2099-12-31T23:59:59.999Z"
        );
        assert_eq!(utc_stamp(1_762_092_192), "20251102T140312Z");
    }

    #[test]
    fn log_line_has_time_uptime_level_and_message() {
        // Arrange / Act
        let text = log_line(
            Duration::from_millis(1_762_092_192_345),
            Duration::from_millis(12_345),
            Level::Warn,
            "seat: keyboard",
        );

        // Assert
        assert_eq!(
            text,
            "2025-11-02T14:03:12.345Z +12.345s WARN seat: keyboard\n"
        );
    }

    #[test]
    fn session_log_rotates_before_cap() -> std::io::Result<()> {
        // Arrange
        let dir = temp_dir("rotate")?;
        let path = dir.join("draw.log");
        let mut log = SessionLog::open(&path, 100)?;

        // Act
        for _ in 0..3 {
            log.write_line(&line(40))?;
        }

        // Assert
        assert_eq!(len(&dir.join("draw.log.1")), 80);
        assert_eq!(len(&path), 40);
        assert_eq!(log.len(), 40);
        Ok(())
    }

    #[test]
    fn session_log_rotates_oversized_file_at_open() -> std::io::Result<()> {
        // Arrange
        let dir = temp_dir("oversized")?;
        let path = dir.join("draw.log");
        std::fs::write(&path, line(150))?;

        // Act
        let log = SessionLog::open(&path, 100)?;

        // Assert
        assert_eq!(len(&dir.join("draw.log.1")), 150);
        assert_eq!(len(&path), 0);
        assert_eq!(log.len(), 0);
        Ok(())
    }

    #[test]
    fn session_log_appends_to_small_file_at_open() -> std::io::Result<()> {
        // Arrange
        let dir = temp_dir("append")?;
        let path = dir.join("draw.log");
        std::fs::write(&path, line(30))?;

        // Act
        let mut log = SessionLog::open(&path, 100)?;
        log.write_line(&line(10))?;

        // Assert
        assert_eq!(len(&path), 40);
        assert!(!dir.join("draw.log.1").exists());
        Ok(())
    }

    #[test]
    fn session_log_creates_its_directory() -> std::io::Result<()> {
        // Arrange
        let dir = temp_dir("mkdir")?;
        let path = dir.join("state/draw/draw.log");

        // Act
        let mut log = SessionLog::open(&path, 100)?;
        log.write_line("hi\n")?;

        // Assert
        assert_eq!(len(&path), 3);
        Ok(())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]
        #[test]
        fn session_log_never_exceeds_cap(
            lines in proptest::collection::vec(0_usize..300, 1..40),
        ) {
            // Arrange
            let dir = temp_dir("cap").map_err(|e| TestCaseError::fail(e.to_string()))?;
            let path = dir.join("draw.log");
            let mut log = SessionLog::open(&path, 256).map_err(|e| TestCaseError::fail(e.to_string()))?;

            for bytes in lines {
                // Act
                log.write_line(&line(bytes)).map_err(|e| TestCaseError::fail(e.to_string()))?;

                // Assert
                prop_assert!(len(&path) <= 256);
                prop_assert!(len(&dir.join("draw.log.1")) <= 256);
                prop_assert_eq!(log.len(), len(&path));
            }
        }
    }

    #[test]
    fn dump_name_has_utc_stamp_and_reason() {
        // Arrange / Act / Assert
        assert_eq!(dump_name(0, "panic"), "dump-19700101T000000Z-panic.log");
        assert_eq!(
            dump_name(1_762_092_192, "Bad Reason/../x"),
            "dump-20251102T140312Z-bad-reason----x.log"
        );
    }

    #[test]
    fn prune_dumps_keeps_newest_five() -> std::io::Result<()> {
        // Arrange
        let dir = temp_dir("prune")?;
        for second in 0..8 {
            std::fs::write(dir.join(dump_name(second, "x")), "")?;
        }
        std::fs::write(dir.join("draw.log"), "")?;
        std::fs::write(dir.join("other.txt"), "")?;

        // Act
        prune_dumps(&dir, MAX_DUMPS)?;

        // Assert
        let mut left = Vec::new();
        for entry in std::fs::read_dir(&dir)? {
            left.push(entry?.file_name().to_string_lossy().into_owned());
        }
        left.sort();
        let mut expected: Vec<String> = (3..8).map(|second| dump_name(second, "x")).collect();
        expected.extend(["draw.log".to_owned(), "other.txt".to_owned()]);
        expected.sort();
        assert_eq!(MAX_DUMPS, 5);
        assert_eq!(left, expected);
        Ok(())
    }
}
