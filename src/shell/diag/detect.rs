//! Anomaly detectors and the per-topic line limiter (ADR-T47-1).
//!
//! The detectors are small state machines fed in event order. They see a
//! key's real code only to keep the set of held keys; nothing they keep
//! is written anywhere.

use std::time::Duration;

use macroquad::input::KeyCode;

use crate::shell::diag::event::{Anomaly, Device, Event};

/// Input after a gap longer than this writes an idle line.
pub const IDLE_GAP: Duration = Duration::from_secs(300);

/// Repeat timer expirations per wake that are still normal.
pub const REPEAT_BURST: u32 = 2;

/// Lines one [`Topic`] may write per session before it is suppressed.
pub const LINES_PER_TOPIC: u32 = 10;

/// Most keys tracked as held at once; the buffer never grows past it.
const HELD_KEYS: usize = 32;

/// Keys held right now, and the chars of the last press.
#[derive(Debug, Clone, Default)]
pub struct KeyWatch {
    /// Keys down without a key up since.
    held: Vec<KeyCode>,
    /// Non-repeat chars since the last non-repeat key down; `None` after a
    /// repeat or before the first press.
    chars: Option<u8>,
}

impl KeyWatch {
    /// No key held.
    #[must_use]
    pub fn new() -> Self {
        Self {
            held: Vec::with_capacity(HELD_KEYS),
            chars: None,
        }
    }

    /// A key went down. A non-repeat press of a held key is
    /// [`Anomaly::HeldKeyPressedAgain`].
    pub fn key_down(&mut self, code: KeyCode, repeat: bool) -> Option<Anomaly> {
        if repeat {
            self.chars = None;
            return None;
        }
        self.chars = Some(0);
        if self.held.contains(&code) {
            return Some(Anomaly::HeldKeyPressedAgain);
        }
        if self.held.len() < HELD_KEYS {
            self.held.push(code);
        }
        None
    }

    /// A key went up.
    pub fn key_up(&mut self, code: KeyCode) {
        self.held.retain(|held| *held != code);
    }

    /// A char arrived. The second non-repeat char of one press is
    /// [`Anomaly::CharsPerPress`].
    pub fn char(&mut self, repeat: bool) -> Option<Anomaly> {
        if repeat {
            return None;
        }
        let chars = self.chars.as_mut()?;
        *chars = chars.saturating_add(1);
        (*chars == 2).then_some(Anomaly::CharsPerPress)
    }

    /// Keyboard focus moved: key ups may have gone to another window.
    pub fn focus_changed(&mut self) {
        self.held.clear();
        self.chars = None;
    }
}

/// When input last arrived.
#[derive(Debug, Clone, Copy, Default)]
pub struct IdleWatch {
    /// Microseconds since start of the last input.
    last: Option<u64>,
}

impl IdleWatch {
    /// No input yet.
    #[must_use]
    pub fn new() -> Self {
        Self { last: None }
    }

    /// Input at `micros`; the gap since the last input when it is longer
    /// than [`IDLE_GAP`].
    pub fn input(&mut self, micros: u64) -> Option<Duration> {
        let last = self.last.replace(micros.max(self.last.unwrap_or(0)))?;
        let gap = Duration::from_micros(micros.saturating_sub(last));
        (gap > IDLE_GAP).then_some(gap)
    }
}

/// Backend events that should not happen.
#[derive(Debug, Clone, Copy, Default)]
pub struct BackendWatch {
    /// Keyboards created so far.
    keyboards: u32,
}

impl BackendWatch {
    /// Nothing seen yet.
    #[must_use]
    pub fn new() -> Self {
        Self { keyboards: 0 }
    }

    /// The anomaly `event` shows, if any.
    pub fn event(&mut self, event: &Event) -> Option<Anomaly> {
        match *event {
            Event::RepeatTimer { expirations } if expirations > REPEAT_BURST => {
                Some(Anomaly::RepeatBurst)
            }
            Event::DeviceCreated(Device::Keyboard) => {
                self.keyboards = self.keyboards.saturating_add(1);
                (self.keyboards > 1).then_some(Anomaly::KeyboardRecreated)
            }
            Event::BackendDropped { count } if count > 0 => Some(Anomaly::BackendOverflow),
            Event::Start
            | Event::Capabilities { .. }
            | Event::DeviceCreated(_)
            | Event::DeviceReleased(_)
            | Event::KeyboardEnter
            | Event::KeyboardLeave
            | Event::WlKey { .. }
            | Event::RepeatInfo { .. }
            | Event::RepeatTimer { .. }
            | Event::Keymap
            | Event::KeymapIgnored { .. }
            | Event::ToolAdded
            | Event::ToolRemoved
            | Event::ProximityIn
            | Event::ProximityOut
            | Event::ProtocolError { .. }
            | Event::BackendDropped { .. }
            | Event::KeyDown { .. }
            | Event::KeyUp { .. }
            | Event::Char { .. }
            | Event::PointerDown(_)
            | Event::PointerUp(_)
            | Event::Wheel
            | Event::PointerMoves { .. }
            | Event::Frame { .. }
            | Event::Tool(_)
            | Event::Revision(_)
            | Event::Idle { .. }
            | Event::Anomaly(_)
            | Event::DumpRequested => None,
        }
    }
}

/// What a rate-limited line is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Topic {
    /// One kind of anomaly.
    Anomaly(Anomaly),
    /// A frame of [`crate::shell::diag::SLOW_FRAME`] or more.
    SlowFrame,
}

impl Topic {
    /// Index into the limiter's counters.
    const fn index(self) -> usize {
        match self {
            Self::Anomaly(anomaly) => anomaly.index(),
            Self::SlowFrame => Anomaly::COUNT,
        }
    }
}

/// What to do with one more line of a topic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Write the line.
    Write,
    /// Write a one-off "further lines suppressed" notice instead.
    Notice,
    /// Write nothing.
    Skip,
}

/// Counts lines per [`Topic`].
#[derive(Debug, Clone, Default)]
pub struct Limiter {
    counts: [u32; Anomaly::COUNT + 1],
}

impl Limiter {
    /// Nothing written yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Counts one more line of `topic` and says what to write.
    pub fn check(&mut self, topic: Topic) -> Verdict {
        let count = &mut self.counts[topic.index()];
        *count = count.saturating_add(1);
        match (*count).cmp(&LINES_PER_TOPIC) {
            std::cmp::Ordering::Less | std::cmp::Ordering::Equal => Verdict::Write,
            std::cmp::Ordering::Greater if *count == LINES_PER_TOPIC + 1 => Verdict::Notice,
            std::cmp::Ordering::Greater => Verdict::Skip,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::diag::event::{Anomaly, Event};
    use macroquad::input::KeyCode;
    use macroquad::miniquad::diag::Device;
    use std::time::Duration;

    fn micros(duration: Duration) -> u64 {
        duration.as_micros() as u64
    }

    #[test]
    fn key_watch_held_key_pressed_again_is_anomaly() {
        // Arrange
        let mut watch = KeyWatch::new();

        // Act
        let first = watch.key_down(KeyCode::A, false);
        let second = watch.key_down(KeyCode::A, false);

        // Assert
        assert_eq!(first, None);
        assert_eq!(second, Some(Anomaly::HeldKeyPressedAgain));
    }

    #[test]
    fn key_watch_repeat_is_not_anomaly() {
        // Arrange
        let mut watch = KeyWatch::new();
        watch.key_down(KeyCode::A, false);

        // Act / Assert
        for _ in 0..3 {
            assert_eq!(watch.key_down(KeyCode::A, true), None);
        }
    }

    #[test]
    fn key_watch_release_then_press_is_not_anomaly() {
        // Arrange
        let mut watch = KeyWatch::new();
        watch.key_down(KeyCode::A, false);
        watch.key_down(KeyCode::B, false);

        // Act
        watch.key_up(KeyCode::A);
        let again = watch.key_down(KeyCode::A, false);

        // Assert
        assert_eq!(again, None);
    }

    #[test]
    fn key_watch_two_chars_per_press_is_anomaly() {
        // Arrange
        let mut watch = KeyWatch::new();
        watch.key_down(KeyCode::A, false);

        // Act
        let first = watch.char(false);
        let second = watch.char(false);
        let third = watch.char(false);

        // Assert
        assert_eq!(first, None);
        assert_eq!(second, Some(Anomaly::CharsPerPress));
        assert_eq!(third, None, "one anomaly per press");
    }

    #[test]
    fn key_watch_repeat_chars_are_not_anomaly() {
        // Arrange
        let mut watch = KeyWatch::new();
        watch.key_down(KeyCode::A, false);
        assert_eq!(watch.char(false), None);

        // Act
        watch.key_down(KeyCode::A, true);
        let repeats = [watch.char(true), watch.char(true)];
        watch.key_down(KeyCode::B, false);
        let next = watch.char(false);

        // Assert
        assert_eq!(repeats, [None, None]);
        assert_eq!(next, None);
    }

    #[test]
    fn key_watch_focus_change_forgets_held_keys() {
        // Arrange
        let mut watch = KeyWatch::new();
        watch.key_down(KeyCode::A, false);

        // Act
        watch.focus_changed();
        let again = watch.key_down(KeyCode::A, false);

        // Assert
        assert_eq!(again, None);
    }

    #[test]
    fn idle_watch_reports_gap_over_five_minutes() {
        // Arrange
        let mut watch = IdleWatch::new();
        let gap = IDLE_GAP + Duration::from_secs(1);

        // Act
        let first = watch.input(0);
        let after = watch.input(micros(gap));

        // Assert
        assert_eq!(IDLE_GAP, Duration::from_secs(300));
        assert_eq!(first, None);
        assert_eq!(after, Some(gap));
    }

    #[test]
    fn idle_watch_short_gap_is_quiet() {
        // Arrange
        let mut watch = IdleWatch::new();
        watch.input(0);

        // Act / Assert
        assert_eq!(watch.input(micros(Duration::from_secs(200))), None);
        assert_eq!(watch.input(micros(Duration::from_secs(400))), None);
        assert_eq!(watch.input(micros(IDLE_GAP)), None, "earlier input");
    }

    #[test]
    fn backend_watch_repeat_burst_is_anomaly() {
        // Arrange
        let mut watch = BackendWatch::new();

        // Act
        let normal = watch.event(&Event::RepeatTimer { expirations: 2 });
        let burst = watch.event(&Event::RepeatTimer { expirations: 3 });

        // Assert
        assert_eq!(normal, None);
        assert_eq!(burst, Some(Anomaly::RepeatBurst));
    }

    #[test]
    fn backend_watch_second_keyboard_is_anomaly() {
        // Arrange
        let mut watch = BackendWatch::new();
        watch.event(&Event::DeviceCreated(Device::Keyboard));
        watch.event(&Event::DeviceReleased(Device::Keyboard));

        // Act
        let again = watch.event(&Event::DeviceCreated(Device::Keyboard));

        // Assert
        assert_eq!(again, Some(Anomaly::KeyboardRecreated));
    }

    #[test]
    fn backend_watch_first_keyboard_is_quiet() {
        // Arrange
        let mut watch = BackendWatch::new();

        // Act
        let pointer = watch.event(&Event::DeviceCreated(Device::Pointer));
        let keyboard = watch.event(&Event::DeviceCreated(Device::Keyboard));

        // Assert
        assert_eq!((pointer, keyboard), (None, None));
    }

    #[test]
    fn backend_watch_dropped_events_are_overflow() {
        // Arrange
        let mut watch = BackendWatch::new();

        // Act / Assert
        assert_eq!(
            watch.event(&Event::BackendDropped { count: 5 }),
            Some(Anomaly::BackendOverflow)
        );
    }

    #[test]
    fn limiter_allows_ten_then_one_notice() {
        // Arrange
        let mut limiter = Limiter::new();
        let topic = Topic::SlowFrame;

        // Act
        let verdicts: Vec<_> = (0..12).map(|_| limiter.check(topic)).collect();
        let other = limiter.check(Topic::Anomaly(Anomaly::RepeatBurst));

        // Assert
        assert_eq!(LINES_PER_TOPIC, 10);
        assert!(verdicts[..10].iter().all(|v| *v == Verdict::Write));
        assert_eq!(verdicts[10], Verdict::Notice);
        assert_eq!(verdicts[11], Verdict::Skip);
        assert_eq!(other, Verdict::Write, "topics are counted apart");
    }
}
