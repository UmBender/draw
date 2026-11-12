//! Always-on diagnostics: a flight recorder in memory and a capped session
//! log on disk (T47, ADR-T47-1).
//!
//! The ring of [`event::Record`]s keeps the last moments of the session.
//! It reaches the disk only as a dump: on a panic, on the first anomaly of
//! a kind, or on `Ctrl+Shift+L`. The session log keeps a few lines per
//! session (start-up, devices, anomalies, exit) and never grows past its
//! cap. Plain text keys and typed characters are never recorded.

pub mod detect;
pub mod e2e;
pub mod event;
pub mod file;
pub mod ring;

use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::{Mutex, PoisonError, TryLockError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use macroquad::miniquad::diag as backend_diag;

use crate::shell::diag::detect::{BackendWatch, IdleWatch, Limiter, Topic, Verdict};
use crate::shell::diag::event::{Anomaly, Event, Record, from_backend};
use crate::shell::diag::file::{
    LOG_CAP, MAX_DUMPS, SessionLog, dump_name, log_line, prune_dumps, utc,
};
use crate::shell::diag::ring::Ring;
use crate::shell::{backend, crash, tablet};

/// Environment variable that sets the [`Level`].
pub const LEVEL_ENV: &str = "DRAW_LOG";

/// Name of the session log in the state directory.
pub const LOG_FILE: &str = "draw.log";

/// Records the flight recorder keeps.
pub const RING_CAPACITY: usize = 4096;

/// Automatic dumps (anomalies) one session may write.
pub const AUTO_DUMPS: usize = 3;

/// Frames this slow write an `info` line.
pub const SLOW_FRAME: Duration = Duration::from_millis(100);

/// Frames over this budget write a `debug` line (ADR-T24-5).
pub const FRAME_BUDGET: Duration = Duration::from_millis(16);

/// How much the session log says. Each level includes the ones before it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    /// No session log and no automatic dumps.
    Off,
    /// Panics and protocol errors.
    Error,
    /// Anomalies.
    Warn,
    /// Start-up, devices, idle, slow frames, dumps and exit (the default).
    Info,
    /// Focus changes and frames over budget.
    Debug,
    /// Every flight recorder record.
    Trace,
}

impl Level {
    /// Every level, quietest first.
    const ALL: [Self; 6] = [
        Self::Off,
        Self::Error,
        Self::Warn,
        Self::Info,
        Self::Debug,
        Self::Trace,
    ];

    /// The level's name as `DRAW_LOG` spells it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }

    /// The name in a log line: `ERROR`, `WARN`, …
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
            Self::Trace => "TRACE",
        }
    }
}

/// The level a `DRAW_LOG` value asks for, and a warning for an unknown
/// one. Unset or blank is [`Level::Info`].
#[must_use]
pub fn parse_level(value: Option<&str>) -> (Level, Option<String>) {
    let Some(value) = value.map(str::trim).filter(|v| !v.is_empty()) else {
        return (Level::Info, None);
    };
    match Level::ALL
        .into_iter()
        .find(|level| level.name().eq_ignore_ascii_case(value))
    {
        Some(level) => (level, None),
        None => (
            Level::Info,
            Some(format!(
                "{LEVEL_ENV}={value} is not off, error, warn, info, debug or trace; using info"
            )),
        ),
    }
}

/// How a [`Diag`] starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// Directory for the session log and dumps; `None` writes nothing.
    pub dir: Option<PathBuf>,
    /// What the session log says.
    pub level: Level,
    /// Wall-clock time at start, since the Unix epoch.
    pub start_unix: Duration,
    /// Records the ring keeps.
    pub capacity: usize,
}

/// What the start-up line says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartInfo<'a> {
    /// Crate version.
    pub version: &'a str,
    /// GNU build id of the binary.
    pub build_id: Option<&'a str>,
    /// Process id.
    pub pid: u32,
    /// Window backend.
    pub backend: &'a str,
    /// `XDG_SESSION_TYPE`.
    pub session: Option<&'a str>,
    /// `XDG_CURRENT_DESKTOP`.
    pub desktop: Option<&'a str>,
    /// Pen tablet mode.
    pub tablet: &'a str,
    /// Log level.
    pub level: Level,
}

/// The start-up line's message.
#[must_use]
pub fn start_line(info: &StartInfo<'_>) -> String {
    let unknown = |value: Option<&str>| value.unwrap_or("unknown").to_owned();
    format!(
        "start: draw {} build {} pid {} backend {} session {} desktop {} tablet {} log {}",
        info.version,
        unknown(info.build_id),
        info.pid,
        info.backend,
        unknown(info.session),
        unknown(info.desktop),
        info.tablet,
        info.level.name()
    )
}

/// `3h02m05s`, `4m10s` or `12s`.
fn uptime(duration: Duration) -> String {
    let secs = duration.as_secs();
    let (hours, minutes, seconds) = (secs / 3600, secs / 60 % 60, secs % 60);
    if hours > 0 {
        format!("{hours}h{minutes:02}m{seconds:02}s")
    } else if minutes > 0 {
        format!("{minutes}m{seconds:02}s")
    } else {
        format!("{seconds}s")
    }
}

/// The flight recorder, the session log and the detectors of one session.
#[derive(Debug)]
pub struct Diag {
    level: Level,
    ring: Ring<Record>,
    start: Instant,
    start_unix: Duration,
    dir: Option<PathBuf>,
    log: Option<SessionLog>,
    build_id: Option<String>,
    limiter: Limiter,
    backend: BackendWatch,
    idle: IdleWatch,
    /// Anomalies seen, per [`Anomaly::index`].
    anomalies: [u32; Anomaly::COUNT],
    /// Automatic dumps written.
    auto_dumps: usize,
}

impl Diag {
    /// A recorder starting now. Opens the session log in `settings.dir`
    /// unless the level is [`Level::Off`]; a log that cannot be opened is
    /// left out.
    #[must_use]
    pub fn new(settings: Settings) -> Self {
        let log = (settings.level != Level::Off)
            .then_some(settings.dir.as_ref())
            .flatten()
            .and_then(|dir| SessionLog::open(&dir.join(LOG_FILE), LOG_CAP).ok());
        let mut ring = Ring::new(settings.capacity);
        ring.push(Record {
            micros: 0,
            event: Event::Start,
        });
        Self {
            level: settings.level,
            ring,
            start: Instant::now(),
            start_unix: settings.start_unix,
            dir: settings.dir,
            log,
            build_id: None,
            limiter: Limiter::new(),
            backend: BackendWatch::new(),
            idle: IdleWatch::new(),
            anomalies: [0; Anomaly::COUNT],
            auto_dumps: 0,
        }
    }

    /// When the recorder started.
    #[must_use]
    pub fn start(&self) -> Instant {
        self.start
    }

    /// The session's level.
    #[must_use]
    pub fn level(&self) -> Level {
        self.level
    }

    /// Writes the start-up line and keeps the build id for dumps.
    pub fn log_start(&mut self, info: &StartInfo<'_>) {
        self.build_id = info.build_id.map(str::to_owned);
        self.log(Level::Info, &start_line(info));
    }

    /// Microseconds from the start to `at`.
    fn micros(&self, at: Instant) -> u64 {
        u64::try_from(at.saturating_duration_since(self.start).as_micros()).unwrap_or(u64::MAX)
    }

    /// Whether lines at `level` are written.
    fn enabled(&self, level: Level) -> bool {
        level != Level::Off && level <= self.level && self.log.is_some()
    }

    /// Records `event` at `at` and runs the detectors and the log rules on
    /// it. Writes nothing unless the event's line is at or below the level.
    pub fn record(&mut self, at: Instant, event: Event) {
        let micros = self.micros(at);
        let record = Record { micros, event };
        self.ring.push(record);
        if self.enabled(Level::Trace) {
            self.log(Level::Trace, &format!("record {record}"));
        }
        match event {
            Event::Anomaly(anomaly) => self.anomaly(anomaly),
            Event::Frame { micros, .. } => {
                self.frame(Duration::from_micros(u64::from(micros)), event);
            }
            Event::Capabilities { .. }
            | Event::DeviceCreated(_)
            | Event::DeviceReleased(_)
            | Event::RepeatInfo { .. }
            | Event::KeymapIgnored { .. }
            | Event::ToolAdded
            | Event::ToolRemoved => {
                if self.enabled(Level::Info) {
                    self.log(Level::Info, &event.to_string());
                }
            }
            Event::KeyboardEnter | Event::KeyboardLeave => {
                if self.enabled(Level::Debug) {
                    self.log(Level::Debug, &event.to_string());
                }
            }
            Event::ProtocolError { .. } => self.log(Level::Error, &event.to_string()),
            Event::Start
            | Event::WlKey { .. }
            | Event::RepeatTimer { .. }
            | Event::Keymap
            | Event::ProximityIn
            | Event::ProximityOut
            | Event::BackendDropped { .. }
            | Event::KeyDown { .. }
            | Event::KeyUp { .. }
            | Event::Char { .. }
            | Event::PointerDown(_)
            | Event::PointerUp(_)
            | Event::Wheel
            | Event::PointerMoves { .. }
            | Event::Tool(_)
            | Event::Revision(_)
            | Event::Idle { .. }
            | Event::DumpRequested => {}
        }
        if event.is_input()
            && let Some(gap) = self.idle.input(micros)
        {
            let secs = u32::try_from(gap.as_secs()).unwrap_or(u32::MAX);
            self.ring.push(Record {
                micros,
                event: Event::Idle { secs },
            });
            if self.enabled(Level::Info) {
                self.log(Level::Info, &format!("input after {} idle", uptime(gap)));
            }
        }
        if let Some(anomaly) = self.backend.event(&event) {
            self.record(at, Event::Anomaly(anomaly));
        }
    }

    /// Counts `anomaly`, writes its line under the limiter and dumps the
    /// first one of each kind, up to [`AUTO_DUMPS`] per session.
    fn anomaly(&mut self, anomaly: Anomaly) {
        let count = &mut self.anomalies[anomaly.index()];
        *count = count.saturating_add(1);
        let first = *count == 1;
        match self.limiter.check(Topic::Anomaly(anomaly)) {
            Verdict::Write => {
                self.log(Level::Warn, &format!("anomaly: {}", anomaly.describe()));
            }
            Verdict::Notice => self.log(
                Level::Warn,
                &format!("further anomaly lines suppressed: {}", anomaly.describe()),
            ),
            Verdict::Skip => {}
        }
        if first && self.auto_dumps < AUTO_DUMPS && self.level != Level::Off {
            self.auto_dumps += 1;
            self.dump(anomaly.slug());
        }
    }

    /// Writes the slow-frame or over-budget line of a frame of `time`.
    fn frame(&mut self, time: Duration, event: Event) {
        if time >= SLOW_FRAME && self.enabled(Level::Info) {
            match self.limiter.check(Topic::SlowFrame) {
                Verdict::Write => self.log(Level::Info, &format!("slow {event}")),
                Verdict::Notice => self.log(Level::Info, "further slow frame lines suppressed"),
                Verdict::Skip => {}
            }
        } else if time > FRAME_BUDGET && self.enabled(Level::Debug) {
            self.log(Level::Debug, &format!("over budget: {event}"));
        }
    }

    /// Wall-clock time now, since the Unix epoch.
    fn unix_now(&self) -> Duration {
        self.start_unix + self.start.elapsed()
    }

    /// Writes `message` to the session log if `level` is enabled. A log
    /// that fails to write is closed for the rest of the session.
    pub fn log(&mut self, level: Level, message: &str) {
        if !self.enabled(level) {
            return;
        }
        let line = log_line(self.unix_now(), self.start.elapsed(), level, message);
        if let Some(log) = &mut self.log
            && log.write_line(&line).is_err()
        {
            self.log = None;
        }
    }

    /// The dump's text: header, records oldest first, anomaly counts.
    fn dump_text(&self, reason: &str, now: Duration) -> String {
        let mut text = String::with_capacity(64 * (self.ring.len() + 16));
        // Writing to a String cannot fail.
        let _ = write!(
            text,
            "draw diagnostics dump\nreason: {reason}\nversion: {}\nbuild id: {}\nstarted: {}\nwritten: {}\nlevel: {}\nrecords: {} (oldest first, seconds since start)\n",
            env!("CARGO_PKG_VERSION"),
            self.build_id.as_deref().unwrap_or("unknown"),
            utc(self.start_unix),
            utc(now),
            self.level.name(),
            self.ring.len(),
        );
        for record in self.ring.iter() {
            let _ = writeln!(text, "{record}");
        }
        text.push_str("anomalies:");
        let mut any = false;
        for anomaly in Anomaly::ALL {
            let count = self.anomaly_count(anomaly);
            if count > 0 {
                let _ = write!(text, " {} {count};", anomaly.describe());
                any = true;
            }
        }
        if !any {
            text.push_str(" none");
        }
        text.push('\n');
        text
    }

    /// Writes the ring to a dump file named for `reason` and returns its
    /// path; `None` without a directory or when writing fails.
    pub fn dump(&mut self, reason: &str) -> Option<PathBuf> {
        let dir = self.dir.clone()?;
        let now = self.unix_now();
        let name = dump_name(now.as_secs(), reason);
        let mut path = dir.join(&name);
        for n in 2..100 {
            if !path.exists() {
                break;
            }
            path = dir.join(name.replace(".log", &format!("-{n}.log")));
        }
        std::fs::create_dir_all(&dir).ok()?;
        std::fs::write(&path, self.dump_text(reason, now)).ok()?;
        // A dump that cannot prune the old ones is still written.
        let _ = prune_dumps(&dir, MAX_DUMPS);
        self.log(Level::Info, &format!("dump written: {}", path.display()));
        Some(path)
    }

    /// The panic hook's part: an `ERROR` line and a `panic` dump.
    pub fn on_panic(&mut self, message: &str) {
        self.log(Level::Error, &format!("panic: {message}"));
        if self.level != Level::Off {
            self.dump("panic");
        }
    }

    /// The exit line's message for `reason` at `now`.
    #[must_use]
    pub fn exit_line(&self, reason: &str, now: Instant) -> String {
        let total: u32 = self.anomalies.iter().sum();
        let noun = if total == 1 { "anomaly" } else { "anomalies" };
        format!(
            "exit: {reason} after {} with {total} {noun}",
            uptime(now.saturating_duration_since(self.start))
        )
    }

    /// How often `anomaly` was seen.
    #[must_use]
    pub fn anomaly_count(&self, anomaly: Anomaly) -> u32 {
        self.anomalies[anomaly.index()]
    }
}

/// The session's recorder, reached by the app loop and the panic hook.
static RECORDER: Mutex<Option<Diag>> = Mutex::new(None);

/// Runs `f` on the recorder in `recorder`, waiting for the lock.
pub fn with_in<R>(recorder: &Mutex<Option<Diag>>, f: impl FnOnce(&mut Diag) -> R) -> Option<R> {
    recorder
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_mut()
        .map(f)
}

/// Runs `f` on the recorder in `recorder` unless it is locked; `None` then
/// or when there is no recorder. For the panic hook, which must not block.
pub fn try_with_in<R>(recorder: &Mutex<Option<Diag>>, f: impl FnOnce(&mut Diag) -> R) -> Option<R> {
    let mut guard = match recorder.try_lock() {
        Ok(guard) => guard,
        Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
        Err(TryLockError::WouldBlock) => return None,
    };
    guard.as_mut().map(f)
}

/// Runs `f` on the session's recorder.
pub fn with<R>(f: impl FnOnce(&mut Diag) -> R) -> Option<R> {
    with_in(&RECORDER, f)
}

/// Runs `f` on the session's recorder unless it is locked.
pub fn try_with<R>(f: impl FnOnce(&mut Diag) -> R) -> Option<R> {
    try_with_in(&RECORDER, f)
}

/// Starts the session's recorder from the environment (`DRAW_LOG`, the
/// state directory, backend and tablet mode) and writes the start-up line.
/// Call once in `main`, after the panic hook.
pub fn init_from_env() {
    let (level, warning) = parse_level(std::env::var(LEVEL_ENV).ok().as_deref());
    let dir = crash::state_dir(
        std::env::var_os("XDG_STATE_HOME").as_deref(),
        std::env::var_os("HOME").as_deref(),
    );
    let start_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let mut diag = Diag::new(Settings {
        dir,
        level,
        start_unix,
        capacity: RING_CAPACITY,
    });
    let build_id = crash::running_build_id();
    let backend = format!("{:?}", backend::from_env().backend).to_lowercase();
    let session = std::env::var("XDG_SESSION_TYPE").ok();
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").ok();
    let tablet = if tablet::from_env().pen {
        "pen"
    } else {
        "mouse"
    };
    diag.log_start(&StartInfo {
        version: env!("CARGO_PKG_VERSION"),
        build_id: build_id.as_deref(),
        pid: std::process::id(),
        backend: &backend,
        session: session.as_deref(),
        desktop: desktop.as_deref(),
        tablet,
        level,
    });
    if let Some(warning) = warning {
        eprintln!("draw: {warning}");
        diag.log(Level::Warn, &warning);
    }
    *RECORDER.lock().unwrap_or_else(PoisonError::into_inner) = Some(diag);
}

/// Records one frame's events under one lock and empties `records`.
pub fn record_all(records: &mut Vec<(Instant, Event)>) {
    with(|diag| {
        for (at, event) in records.drain(..) {
            diag.record(at, event);
        }
    });
    records.clear();
}

/// Moves the backend's queued events into `out` as records, keys hidden
/// by the privacy rule, with a [`Event::BackendDropped`] record if the
/// queue dropped any. `buffer` is reused between frames.
pub fn take_backend(
    buffer: &mut Vec<backend_diag::BackendRecord>,
    out: &mut Vec<(Instant, Event)>,
) {
    let dropped = backend_diag::take_events_into(buffer);
    if dropped > 0 {
        let count = u32::try_from(dropped).unwrap_or(u32::MAX);
        out.push((Instant::now(), Event::BackendDropped { count }));
    }
    out.extend(buffer.drain(..).map(|r| (r.at, from_backend(r.event))));
}

/// Writes `message` to the session log at `level`.
pub fn log(level: Level, message: &str) {
    with(|diag| diag.log(level, message));
}

/// Writes a dump for `reason` (also the entry point for T48's signal
/// handling) and returns its path.
#[must_use = "tell the user where the dump went"]
pub fn dump(reason: &str) -> Option<PathBuf> {
    with(|diag| {
        diag.record(Instant::now(), Event::DumpRequested);
        diag.dump(reason)
    })
    .flatten()
}

/// The panic hook's part (ADR-T27-3, ADR-T47-1): takes the backend's last
/// events, then writes the `ERROR` line and the `panic` dump. Skips both
/// if the recorder is locked.
pub fn on_panic(message: &str) {
    try_with(|diag| {
        let mut buffer = Vec::new();
        let mut records = Vec::new();
        take_backend(&mut buffer, &mut records);
        for (at, event) in records {
            diag.record(at, event);
        }
        diag.on_panic(message);
    });
}

/// Writes the exit line for `reason`.
pub fn finish(reason: &str) {
    with(|diag| {
        let line = diag.exit_line(reason, Instant::now());
        diag.log(Level::Info, &line);
    });
}

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
        let dir = std::env::temp_dir().join(format!("draw-diag-{}-{name}", std::process::id()));
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
    fn keymap_ignored_is_logged_at_info() -> io::Result<()> {
        // Arrange
        let dir = temp_dir("keymap-ignored")?;
        let mut diag = diag(&dir, Level::Info);

        // Act
        diag.record(Instant::now(), Event::KeymapIgnored { format: 0, size: 0 });

        // Assert: a WARN line would fail the e2e exit check.
        let text = log_text(&dir);
        assert!(
            text.contains("INFO keymap ignored format 0 size 0"),
            "{text}"
        );
        assert!(!text.contains(" WARN "), "{text}");
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
        assert!(
            name.starts_with("dump-") && name.ends_with("-manual.log"),
            "{name}"
        );
        let text = std::fs::read_to_string(&path)?;
        assert!(text.starts_with("draw diagnostics dump\n"), "{text}");
        assert!(text.contains("reason: manual"), "{text}");
        assert!(
            text.contains(concat!("version: ", env!("CARGO_PKG_VERSION"))),
            "{text}"
        );
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
        assert!(
            text.contains("ERROR panic: boom at src/core/shape.rs:1:2"),
            "{text}"
        );
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
        for part in [
            "1.4.3", "abcdef", "4242", "wayland", "Hyprland", "pen", "debug",
        ] {
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
        assert_eq!(dumps(&dir)?, Vec::<PathBuf>::new());
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
