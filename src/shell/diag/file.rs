//! Files on disk: UTC timestamps, the capped session log and dump files
//! (ADR-T47-1).

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
        let dir = temp_dir("rotate");
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
        let dir = temp_dir("oversized");
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
        let dir = temp_dir("append");
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
        let dir = temp_dir("mkdir");
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
            let dir = temp_dir("cap");
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
        let dir = temp_dir("prune");
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
