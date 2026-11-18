//! Fuzz of the shell (T141): random raw event sessions run through
//! `app::run` over a scripted `Platform` and `NullGpu`, with no window.
//!
//! `cargo test` runs 256 cases; `PROPTEST_CASES=20000` is the nightly run.
//! Shrunk failures go to `tests/shell_fuzz.proptest-regressions`.
//!
//! The generator never makes the chords that write files (`Ctrl+S`, `Ctrl+E`,
//! any Ctrl+Shift chord), so a session leaves the machine as it was.

use std::cell::RefCell;
use std::future::Future;
use std::pin::pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

use draw::shell::app::run;
use draw::shell::backend::Backend;
use draw::shell::options::{Options, TabletMode};
use draw::shell::recording::{NullGpu, RecordingPlatform};
use draw::shell::tablet;
use draw_sys::platform::{
    Platform, RawButton, RawEvent, RawKey, RawMods, SeatChange, WindowMetrics,
};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

/// Most frames in one session.
const MAX_FRAMES: usize = 40;
/// Most events in one frame.
const MAX_PER_FRAME: usize = 8;
/// Most `schedule_update` calls the loop may make in one frame.
const MAX_WAKES_PER_FRAME: usize = 4;
/// Most clipboard writes the loop may make in one frame.
const MAX_CLIPBOARD_PER_FRAME: usize = 4;

const START: WindowMetrics = WindowMetrics {
    width: 640.0,
    height: 480.0,
    scale: 1.0,
};

/// What the platform saw, shared with the test.
#[derive(Debug, Default)]
struct Stats {
    frames: usize,
    wakes: usize,
    clipboard: usize,
    max_wakes: usize,
    max_clipboard: usize,
}

/// A platform that hands out one scripted batch of events per frame and
/// never finishes its last frame, so `run` stays pending.
struct Scripted {
    inner: RecordingPlatform,
    script: std::vec::IntoIter<Vec<RawEvent>>,
    stats: Rc<RefCell<Stats>>,
}

impl Platform for Scripted {
    fn poll(&mut self, out: &mut Vec<RawEvent>) {
        self.inner.poll(out);
    }
    fn window(&mut self) -> WindowMetrics {
        self.inner.window()
    }
    fn clipboard_text(&mut self) -> Option<String> {
        self.inner.clipboard_text()
    }
    fn set_clipboard_text(&mut self, text: &str) {
        self.stats.borrow_mut().clipboard += 1;
        self.inner.set_clipboard_text(text);
    }
    fn set_clipboard_png(&mut self, png: &[u8]) {
        self.stats.borrow_mut().clipboard += 1;
        self.inner.set_clipboard_png(png);
    }
    fn schedule_update(&mut self) {
        self.stats.borrow_mut().wakes += 1;
        self.inner.schedule_update();
    }
    fn next_frame(&mut self) -> impl Future<Output = ()> {
        let batch = self.script.next();
        {
            let mut stats = self.stats.borrow_mut();
            stats.max_wakes = stats.max_wakes.max(stats.wakes);
            stats.max_clipboard = stats.max_clipboard.max(stats.clipboard);
            stats.wakes = 0;
            stats.clipboard = 0;
            if batch.is_some() {
                stats.frames += 1;
            }
        }
        let ready = batch.is_some();
        for event in batch.into_iter().flatten() {
            self.inner.queue(event);
        }
        async move {
            if !ready {
                std::future::pending::<()>().await;
            }
        }
    }
}

/// Runs `frames` of events and returns what the platform saw.
fn run_session(frames: Vec<Vec<RawEvent>>) -> Stats {
    let stats = Rc::new(RefCell::new(Stats::default()));
    let inner = RecordingPlatform::new(START);
    let count = frames.len();
    let platform = Scripted {
        inner,
        script: frames.into_iter(),
        stats: Rc::clone(&stats),
    };
    let tablet = tablet::detect(TabletMode::Off);
    let mut session = pin!(run(
        platform,
        NullGpu::default(),
        Options::default(),
        Backend::X11,
        tablet
    ));
    let mut cx = Context::from_waker(Waker::noop());
    assert_eq!(session.as_mut().poll(&mut cx), Poll::Pending);
    let stats = stats.take();
    assert_eq!(stats.frames, count, "the loop stopped asking for frames");
    stats
}

/// Whether `event` is one the fuzz must not make.
fn is_forbidden(event: &RawEvent) -> bool {
    match event {
        RawEvent::Key { key, mods, .. } => {
            mods.ctrl && (mods.shift || matches!(key, RawKey::Letter('S' | 'E')))
        }
        RawEvent::Char { mods, .. } => mods.ctrl && mods.shift,
        RawEvent::PointerMove { .. }
        | RawEvent::Button { .. }
        | RawEvent::Wheel { .. }
        | RawEvent::Tablet { .. }
        | RawEvent::Resize(_)
        | RawEvent::Focus(_)
        | RawEvent::Seat(_)
        | RawEvent::Backend(_) => false,
    }
}

/// Turns a generated event into one the fuzz may make.
fn sanitize(event: RawEvent) -> RawEvent {
    if !is_forbidden(&event) {
        return event;
    }
    match event {
        RawEvent::Key {
            key, down, repeat, ..
        } => RawEvent::Key {
            key: if key == RawKey::Tab {
                RawKey::Space
            } else {
                key
            },
            mods: RawMods::default(),
            down,
            repeat,
        },
        RawEvent::Char { repeat, .. } => RawEvent::Char {
            ch: ' ',
            mods: RawMods::default(),
            repeat,
        },
        other @ (RawEvent::PointerMove { .. }
        | RawEvent::Button { .. }
        | RawEvent::Wheel { .. }
        | RawEvent::Tablet { .. }
        | RawEvent::Resize(_)
        | RawEvent::Focus(_)
        | RawEvent::Seat(_)
        | RawEvent::Backend(_)) => other,
    }
}

fn mods() -> impl Strategy<Value = RawMods> {
    (any::<bool>(), any::<bool>(), any::<bool>()).prop_map(|(shift, ctrl, alt)| RawMods {
        shift,
        ctrl,
        alt,
    })
}

fn coord() -> impl Strategy<Value = f32> {
    prop_oneof![
        8 => 0.0f32..700.0,
        1 => -1.0e6f32..1.0e6,
        1 => Just(0.0f32),
    ]
}

fn key() -> impl Strategy<Value = RawKey> {
    prop_oneof![
        10 => (b'A'..=b'Z').prop_map(|b| RawKey::Letter(char::from(b))),
        4 => (0u8..=9).prop_map(RawKey::Digit),
        1 => Just(RawKey::Escape),
        1 => Just(RawKey::Space),
        1 => Just(RawKey::Tab),
        2 => Just(RawKey::Backspace),
        1 => Just(RawKey::Delete),
        2 => Just(RawKey::Enter),
        1 => Just(RawKey::Home),
        1 => Just(RawKey::End),
        1 => Just(RawKey::ArrowLeft),
        1 => Just(RawKey::ArrowRight),
        1 => Just(RawKey::ArrowUp),
        1 => Just(RawKey::ArrowDown),
        1 => Just(RawKey::BracketLeft),
        1 => Just(RawKey::BracketRight),
        1 => Just(RawKey::LeftShift),
        1 => Just(RawKey::LeftControl),
        1 => Just(RawKey::LeftAlt),
        1 => any::<u32>().prop_map(RawKey::Other),
    ]
}

fn button() -> impl Strategy<Value = RawButton> {
    prop_oneof![
        4 => Just(RawButton::Left),
        1 => Just(RawButton::Middle),
        1 => Just(RawButton::Right),
        1 => Just(RawButton::Other),
    ]
}

fn raw_event() -> impl Strategy<Value = RawEvent> {
    let size = (1.0f32..4000.0, 1.0f32..4000.0, 0.5f32..4.0).prop_map(|(width, height, scale)| {
        RawEvent::Resize(WindowMetrics {
            width,
            height,
            scale,
        })
    });
    prop_oneof![
        10 => (key(), mods(), any::<bool>(), any::<bool>())
            .prop_map(|(key, mods, down, repeat)| RawEvent::Key { key, mods, down, repeat }),
        4 => (any::<char>(), mods(), any::<bool>())
            .prop_map(|(ch, mods, repeat)| RawEvent::Char { ch, mods, repeat }),
        8 => (coord(), coord()).prop_map(|(x, y)| RawEvent::PointerMove { x, y }),
        6 => (button(), coord(), coord(), any::<bool>())
            .prop_map(|(button, x, y, down)| RawEvent::Button { button, x, y, down }),
        2 => (-20.0f32..20.0, -20.0f32..20.0).prop_map(|(dx, dy)| RawEvent::Wheel { dx, dy }),
        6 => (coord(), coord(), -0.5f32..1.5)
            .prop_map(|(x, y, pressure)| RawEvent::Tablet { x, y, pressure }),
        1 => size,
        1 => any::<bool>().prop_map(RawEvent::Focus),
        1 => prop_oneof![
            (0u32..4).prop_map(|id| RawEvent::Seat(SeatChange::Added(id))),
            (0u32..4).prop_map(|id| RawEvent::Seat(SeatChange::Removed(id))),
        ],
    ]
    .prop_map(sanitize)
}

fn session() -> impl Strategy<Value = Vec<Vec<RawEvent>>> {
    prop::collection::vec(
        prop::collection::vec(raw_event(), 0..=MAX_PER_FRAME),
        0..=MAX_FRAMES,
    )
}

/// Failures are saved to `tests/shell_fuzz.proptest-regressions`.
fn config() -> ProptestConfig {
    ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
            "proptest-regressions",
        ))),
        ..ProptestConfig::default()
    }
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn shell_session_never_panics_and_runs_every_frame(frames in session()) {
        // Arrange / Act: `run_session` checks every scripted frame ran.
        let stats = run_session(frames);

        // Assert
        prop_assert!(stats.frames <= MAX_FRAMES);
    }

    #[test]
    fn shell_session_platform_calls_per_frame_are_bounded(frames in session()) {
        // Arrange / Act
        let stats = run_session(frames);

        // Assert
        prop_assert!(stats.max_wakes <= MAX_WAKES_PER_FRAME, "{stats:?}");
        prop_assert!(stats.max_clipboard <= MAX_CLIPBOARD_PER_FRAME, "{stats:?}");
    }

    #[test]
    fn generated_events_avoid_chords_that_write_files(frames in session()) {
        // Arrange / Act / Assert
        for event in frames.iter().flatten() {
            prop_assert!(!is_forbidden(event), "{event:?}");
        }
    }
}
