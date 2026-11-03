//! Files on disk: UTC timestamps, the capped session log and dump files
//! (ADR-T47-1).
//!
//! The session log is opened once and written one line at a time. A line
//! that would push it past its cap first moves it to `<name>.1`, so the
//! two files together never hold more than twice the cap.

use std::fs::File;
use std::io;
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
    let _ = since_epoch;
    todo!()
}

/// `unix_secs` as `YYYYMMDDTHHMMSSZ`, for file names.
#[must_use]
pub fn utc_stamp(unix_secs: u64) -> String {
    let _ = unix_secs;
    todo!()
}

/// One session log line: `<utc> +<uptime>s <LEVEL> <message>\n`.
#[must_use]
pub fn log_line(unix: Duration, uptime: Duration, level: Level, message: &str) -> String {
    let _ = (unix, uptime, level, message);
    todo!()
}

/// `dump-<stamp>-<reason>.log`, the reason lowercased, other characters
/// than ASCII letters and digits turned into `-`, at most 32 of them.
#[must_use]
pub fn dump_name(unix_secs: u64, reason: &str) -> String {
    let _ = (unix_secs, reason);
    todo!()
}

/// Deletes the oldest `dump-*.log` files in `dir` (by name, which sorts by
/// time) until `keep` are left.
///
/// # Errors
///
/// When `dir` cannot be read or a file cannot be removed.
pub fn prune_dumps(dir: &Path, keep: usize) -> io::Result<()> {
    let _ = (dir, keep);
    todo!()
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
        let _ = (path, cap);
        todo!()
    }

    /// Appends `line` (cut to the cap), rotating first if it would not fit.
    ///
    /// # Errors
    ///
    /// When the file cannot be renamed, reopened or written.
    pub fn write_line(&mut self, line: &str) -> io::Result<()> {
        let _ = (line, &self.file, self.cap);
        todo!()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::diag::Level;
    use proptest::prelude::*;
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    /// An empty directory of its own for one test.
    fn temp_dir(name: &str) -> std::io::Result<PathBuf> {
        let dir = std::env::temp_dir().join(format!(
            "draw-diag-file-{}-{name}",
            std::process::id()
        ));
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
        assert_eq!(text, "2025-11-02T14:03:12.345Z +12.345s WARN seat: keyboard\n");
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
