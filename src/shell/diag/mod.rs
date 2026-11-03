//! Always-on diagnostics: a flight recorder in memory and a capped session
//! log on disk (T47, ADR-T47-1).
//!
//! The ring of [`event::Record`]s keeps the last moments of the session.
//! It reaches the disk only as a dump: on a panic, on the first anomaly of
//! a kind, or on `Ctrl+Shift+L`. The session log keeps a few lines per
//! session (start-up, devices, anomalies, exit) and never grows past its
//! cap. Plain text keys and typed characters are never recorded.

pub mod detect;
pub mod event;
pub mod file;
pub mod ring;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::diag::event::{Anomaly, CharClass, Event, FrameKind, KeyTag};
    use crate::shell::input_map::Collector;
    use macroquad::input::KeyCode;
    use macroquad::miniquad::{EventHandler, KeyMods};
    use std::io;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use std::time::{Duration, Instant};

    const NO_MODS: KeyMods = KeyMods {
        shift: false,
        ctrl: false,
        alt: false,
        logo: false,
    };

    /// An empty directory of its own for one test.
    fn temp_dir(name: &str) -> io::Result<PathBuf> {
        let dir = std::env::temp_dir().join(format!(
            "draw-diag-{}-{name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    fn diag(dir: &Path, level: Level) -> Diag {
        Diag::new(Settings {
            dir: Some(dir.to_path_buf()),
            level,
            start_unix: Duration::from_secs(1_762_092_192),
            capacity: 64,
        })
    }

    fn log_text(dir: &Path) -> String {
        std::fs::read_to_string(dir.join(LOG_FILE)).unwrap_or_default()
    }

    fn dumps(dir: &Path) -> io::Result<Vec<PathBuf>> {
        let mut found = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
            if name.is_some_and(|n| n.starts_with("dump-")) {
                found.push(path);
            }
        }
        found.sort();
        Ok(found)
    }

    fn at(diag: &Diag, millis: u64) -> Instant {
        diag.start() + Duration::from_millis(millis)
    }

    #[test]
    fn level_unset_is_info() {
        // Arrange / Act / Assert
        assert_eq!(parse_level(None), (Level::Info, None));
        assert_eq!(parse_level(Some("  ")), (Level::Info, None));
    }

    #[test]
    fn level_names_parse_any_case() {
        // Arrange
        let cases = [
            ("off", Level::Off),
            ("ERROR", Level::Error),
            ("Warn", Level::Warn),
            ("info", Level::Info),
            (" debug ", Level::Debug),
            ("TRACE", Level::Trace),
        ];

        for (value, level) in cases {
            // Act / Assert
            assert_eq!(parse_level(Some(value)), (level, None), "{value}");
        }
    }

    #[test]
    fn level_unknown_is_info_with_warning() {
        // Arrange / Act
        let (level, warning) = parse_level(Some("loud"));

        // Assert
        assert_eq!(level, Level::Info);
        let warning = warning.unwrap_or_default();
        assert!(warning.contains(LEVEL_ENV), "{warning}");
        assert!(warning.contains("loud"), "{warning}");
    }

    #[test]
    fn level_below_threshold_writes_nothing() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("threshold")?;
        let mut warn = diag(&dir, Level::Warn);

        // Act
        warn.log(Level::Info, "hello");
        warn.log(Level::Warn, "bad thing");

        // Assert
        let text = log_text(&dir);
        assert!(!text.contains("hello"), "{text}");
        assert!(text.contains("WARN bad thing"), "{text}");

        // Arrange / Act: `off` opens no file at all.
        let quiet = temp_dir("threshold-off")?;
        diag(&quiet, Level::Off).log(Level::Error, "nothing");

        // Assert
        assert!(!quiet.join(LOG_FILE).exists());
        Ok(())
    }

    #[test]
    fn anomaly_lines_are_rate_limited() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("limited")?;
        let mut diag = diag(&dir, Level::Info);

        // Act
        for n in 0..15 {
            let when = at(&diag, n);
            diag.record(when, Event::Anomaly(Anomaly::HeldKeyPressedAgain));
        }

        // Assert
        let text = log_text(&dir);
        let lines = |needle: &str| text.lines().filter(|l| l.contains(needle)).count();
        assert_eq!(lines("WARN anomaly: held key pressed again"), 10, "{text}");
        assert_eq!(lines("suppressed"), 1, "{text}");
        assert_eq!(diag.anomaly_count(Anomaly::HeldKeyPressedAgain), 15);
        Ok(())
    }

    #[test]
    fn backend_overflow_is_anomaly() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("overflow")?;
        let mut diag = diag(&dir, Level::Info);

        // Act
        let when = at(&diag, 1);
        diag.record(when, Event::BackendDropped { count: 7 });

        // Assert
        assert_eq!(diag.anomaly_count(Anomaly::BackendOverflow), 1);
        let text = log_text(&dir);
        assert!(text.contains(Anomaly::BackendOverflow.describe()), "{text}");
        Ok(())
    }

    #[test]
    fn dump_writes_header_records_and_counts() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("dump")?;
        let mut diag = diag(&dir, Level::Info);
        let first = at(&diag, 1);
        diag.record(
            first,
            Event::KeyDown {
                key: KeyTag::Named(KeyCode::Escape),
                repeat: false,
            },
        );
        let second = at(&diag, 2);
        diag.record(
            second,
            Event::Frame {
                kind: FrameKind::Full,
                micros: 1234,
                shapes: 9,
            },
        );

        // Act
        let path = diag.dump("manual").unwrap_or_default();

        // Assert
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
        let name = name.unwrap_or_default();
        assert!(name.starts_with("dump-") && name.ends_with("-manual.log"), "{name}");
        let text = std::fs::read_to_string(&path)?;
        assert!(text.starts_with("draw diagnostics dump\n"), "{text}");
        assert!(text.contains("reason: manual"), "{text}");
        assert!(text.contains(concat!("version: ", env!("CARGO_PKG_VERSION"))), "{text}");
        let escape = text.find("Escape").unwrap_or(usize::MAX);
        let frame = text.find("frame").unwrap_or(0);
        assert!(escape < frame, "oldest first: {text}");
        assert!(text.contains("anomalies: none"), "{text}");
        let log = log_text(&dir);
        assert!(log.contains("dump written"), "{log}");
        Ok(())
    }

    #[test]
    fn first_anomaly_of_a_kind_dumps_once() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("auto-dump")?;
        let mut diag = diag(&dir, Level::Info);

        // Act
        for n in 0..3 {
            let when = at(&diag, n);
            diag.record(when, Event::Anomaly(Anomaly::HeldKeyPressedAgain));
        }
        let after_one_kind = dumps(&dir)?.len();
        let when = at(&diag, 10);
        diag.record(when, Event::Anomaly(Anomaly::CharsPerPress));

        // Assert
        assert_eq!(after_one_kind, 1);
        assert_eq!(dumps(&dir)?.len(), 2);
        Ok(())
    }

    #[test]
    fn auto_dumps_are_capped_per_session() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("auto-cap")?;
        let mut diag = diag(&dir, Level::Info);

        // Act
        for (n, anomaly) in Anomaly::ALL.into_iter().enumerate() {
            let when = at(&diag, n as u64);
            diag.record(when, Event::Anomaly(anomaly));
        }
        let automatic = dumps(&dir)?.len();
        let manual = diag.dump("manual");

        // Assert
        assert!(Anomaly::ALL.len() > AUTO_DUMPS);
        assert_eq!(AUTO_DUMPS, 3);
        assert_eq!(automatic, AUTO_DUMPS);
        assert!(manual.is_some());
        assert_eq!(dumps(&dir)?.len(), AUTO_DUMPS + 1);
        Ok(())
    }

    #[test]
    fn dump_of_typed_text_names_no_letter() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("privacy")?;
        let mut diag = diag(&dir, Level::Trace);
        let mut collector = Collector::new();
        let keys = [
            (KeyCode::S, 's'),
            (KeyCode::E, 'e'),
            (KeyCode::C, 'c'),
            (KeyCode::R, 'r'),
            (KeyCode::E, 'e'),
            (KeyCode::T, 't'),
        ];
        for (code, ch) in keys {
            collector.key_down_event(code, NO_MODS, false);
            collector.char_event(ch, NO_MODS, false);
            collector.key_up_event(code, NO_MODS);
        }
        let mut records = Vec::new();
        collector.take_records(&mut records);

        // Act
        for (when, event) in records {
            diag.record(when, event);
        }
        let text = std::fs::read_to_string(diag.dump("manual").unwrap_or_default())?;

        // Assert
        let keyed: Vec<&str> = text.lines().filter(|l| l.contains("key-")).collect();
        assert_eq!(keyed.len(), 12, "{text}");
        assert!(keyed.iter().all(|l| l.ends_with("text")), "{text}");
        let chars = text.lines().filter(|l| l.contains("char letter")).count();
        assert_eq!(chars, 6, "{text}");
        let log = log_text(&dir);
        let traced: Vec<&str> = log.lines().filter(|l| l.contains("key-")).collect();
        assert_eq!(traced.len(), 12, "trace copies the records: {log}");
        assert!(traced.iter().all(|l| l.ends_with("text")), "{log}");
        for logged in [&text, &log] {
            assert!(!logged.to_lowercase().contains("secret"));
        }
        Ok(())
    }

    #[test]
    fn panic_logs_error_and_dumps() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("panic")?;
        let mut diag = diag(&dir, Level::Info);

        // Act
        diag.on_panic("boom at src/core/shape.rs:1:2");

        // Assert
        let text = log_text(&dir);
        assert!(text.contains("ERROR panic: boom at src/core/shape.rs:1:2"), "{text}");
        let found = dumps(&dir)?;
        assert_eq!(found.len(), 1);
        assert!(found[0].to_string_lossy().ends_with("-panic.log"));
        Ok(())
    }

    #[test]
    fn try_with_locked_recorder_returns_none() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("locked")?;
        let recorder = Mutex::new(Some(diag(&dir, Level::Info)));

        // Act
        let guard = recorder.lock();
        let while_locked = try_with_in(&recorder, |_| ());
        drop(guard);
        let after = try_with_in(&recorder, |_| ());

        // Assert
        assert_eq!(while_locked, None);
        assert_eq!(after, Some(()));
        Ok(())
    }

    #[test]
    fn start_line_names_version_build_and_settings() {
        // Arrange
        let info = StartInfo {
            version: "1.4.3",
            build_id: Some("abcdef"),
            pid: 4242,
            backend: "wayland",
            session: Some("wayland"),
            desktop: Some("Hyprland"),
            tablet: "pen",
            level: Level::Debug,
        };

        // Act
        let line = start_line(&info);
        let unknown = start_line(&StartInfo {
            build_id: None,
            session: None,
            desktop: None,
            ..info
        });

        // Assert
        for part in ["1.4.3", "abcdef", "4242", "wayland", "Hyprland", "pen", "debug"] {
            assert!(line.contains(part), "{part}: {line}");
        }
        assert!(unknown.contains("unknown"), "{unknown}");
        assert!(!line.contains('\n'), "{line}");
    }

    #[test]
    fn exit_line_names_reason_uptime_and_anomalies() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("exit")?;
        let mut diag = diag(&dir, Level::Info);
        for n in 0..2 {
            let when = at(&diag, n);
            diag.record(when, Event::Anomaly(Anomaly::RepeatBurst));
        }
        let end = diag.start() + Duration::from_secs(3 * 3600 + 2 * 60 + 5);

        // Act
        let line = diag.exit_line("window closed", end);

        // Assert
        assert!(line.contains("window closed"), "{line}");
        assert!(line.contains("3h02m05s"), "{line}");
        assert!(line.contains("2 anomalies"), "{line}");
        Ok(())
    }

    #[test]
    fn recording_at_info_writes_no_file() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("quiet")?;
        let mut diag = diag(&dir, Level::Info);
        let before = std::fs::metadata(dir.join(LOG_FILE)).map_or(0, |m| m.len());

        // Act
        for n in 0..10_000_u32 {
            let when = at(&diag, u64::from(n));
            diag.record(
                when,
                Event::KeyDown {
                    key: KeyTag::Text,
                    repeat: false,
                },
            );
            diag.record(
                when,
                Event::Char {
                    class: CharClass::Letter,
                    repeat: false,
                },
            );
            diag.record(
                when,
                Event::Frame {
                    kind: FrameKind::Overlay,
                    micros: 150,
                    shapes: 10,
                },
            );
        }

        // Assert
        let after = std::fs::metadata(dir.join(LOG_FILE)).map_or(0, |m| m.len());
        assert_eq!(after, before);
        assert!(dumps(&dir)?.is_empty());
        Ok(())
    }

    /// Release builds only, like `tests/perf.rs`:
    /// `cargo test --release --lib record_cost -- --ignored`.
    #[test]
    #[ignore = "timing; run in release mode"]
    fn record_cost_is_under_a_microsecond() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("cost")?;
        let mut diag = Diag::new(Settings {
            dir: Some(dir.clone()),
            level: Level::Info,
            start_unix: Duration::ZERO,
            capacity: RING_CAPACITY,
        });
        let start = diag.start();
        let runs: u32 = 1_000_000;

        // Act
        let timer = Instant::now();
        for n in 0..runs {
            let event = Event::KeyDown {
                key: KeyTag::Text,
                repeat: n % 2 == 0,
            };
            diag.record(start + Duration::from_micros(u64::from(n)), event);
        }
        let average = timer.elapsed() / runs;

        // Assert
        println!("record: {average:?} per event");
        if !cfg!(debug_assertions) {
            assert!(average < Duration::from_micros(1), "{average:?}");
        }
        Ok(())
    }
}
