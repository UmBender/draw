//! The Wayland window against a private headless sway (T168 AC-2, AC-3,
//! AC-5). A missing sway fails the tests, it never skips them.

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "common/sway.rs"]
mod sway;

use std::future::Future;
use std::path::Path;
use std::pin::pin;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

use draw_sys::egl::GlContext;
use draw_sys::platform::{Platform, RawEvent};
use draw_sys::wayland::{Connection, WaylandError, WaylandPlatform};
use sway::Sway;

fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut cx) {
            return value;
        }
    }
}

fn platform(sway: &Sway) -> WaylandPlatform {
    WaylandPlatform::connect(Some(&sway.socket), "T168 title", "draw-test").unwrap()
}

/// Polls `platform` until `wanted` finds something, or fails after 5 s.
fn pump_until<T>(
    platform: &mut WaylandPlatform,
    mut wanted: impl FnMut(&mut WaylandPlatform, &[RawEvent]) -> Option<T>,
) -> T {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let mut events = Vec::new();
        platform.poll(&mut events);
        if let Some(found) = wanted(platform, &events) {
            return found;
        }
        assert!(Instant::now() < deadline, "the compositor did not answer");
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn wayland_missing_server_is_an_error_value() {
    let err = Connection::connect(Some(Path::new("/nonexistent/wayland-none"))).err();

    assert!(matches!(err, Some(WaylandError::Connect(_))), "{err:?}");
}

#[test]
fn wayland_connect_binds_the_globals() {
    let sway = Sway::start();

    let conn = Connection::connect(Some(&sway.socket)).unwrap();

    assert!(conn.has_global("wl_compositor"));
    assert!(conn.has_global("xdg_wm_base"));
    assert!(conn.has_global("wl_output"));
    assert!(!conn.has_global("wl_no_such_global"));
}

#[test]
fn wayland_window_configures_with_a_size() {
    let sway = Sway::start();
    let conn = Connection::connect(Some(&sway.socket)).unwrap();

    let window = conn
        .create_window("T168 title", "draw-test")
        .unwrap()
        .wait_configure()
        .unwrap();

    // Sway lets an unmapped window choose its size: the fallback is used.
    let (width, height) = window.size();
    assert!(width > 0 && height > 0);
    // Once mapped, sway configures the tile: the whole 800x600 output.
    let window = window.open_gl().unwrap();
    window.swap().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while window.size() != (800, 600) {
        assert!(Instant::now() < deadline, "size is {:?}", window.size());
        conn.dispatch(Duration::from_millis(20)).unwrap();
        window.apply_configure();
    }
}

#[test]
fn wayland_window_sets_title_and_app_id() {
    let sway = Sway::start();
    let conn = Connection::connect(Some(&sway.socket)).unwrap();
    let window = conn
        .create_window("T168 title", "draw-test")
        .unwrap()
        .wait_configure()
        .unwrap()
        .open_gl()
        .unwrap();
    window.swap().unwrap();

    let tree = sway.msg(&["-t", "get_tree"]);

    assert!(tree.contains("\"T168 title\""), "{tree}");
    assert!(tree.contains("\"draw-test\""), "{tree}");
}

#[test]
fn wayland_egl_context_renders_and_swaps() {
    let sway = Sway::start();
    let conn = Connection::connect(Some(&sway.socket)).unwrap();
    let window = conn
        .create_window("T168 title", "draw-test")
        .unwrap()
        .wait_configure()
        .unwrap()
        .open_gl()
        .unwrap();

    assert_eq!(window.context().make_current(), Ok(()));
    assert!(window.context().proc_address("glClear").is_some());
    assert_eq!(window.swap(), Ok(()));
}

/// Opens a connection, a GL window and one swap, then drops all of it.
fn open_swap_and_drop(socket: &Path) {
    let conn = Connection::connect(Some(socket)).unwrap();
    let window = conn
        .create_window("T168 title", "draw-test")
        .unwrap()
        .wait_configure()
        .unwrap()
        .open_gl()
        .unwrap();
    window.swap().unwrap();
}

/// Regression (ADR-T168-2): the EGL display of a closed connection must not
/// be reused by a later connection at the same `wl_display` address.
#[test]
fn wayland_many_connections_in_sequence_and_threads() {
    let sway = Sway::start();

    for _ in 0..50 {
        open_swap_and_drop(&sway.socket);
    }
    std::thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| {
                for _ in 0..12 {
                    open_swap_and_drop(&sway.socket);
                }
            });
        }
    });
}

#[test]
fn wayland_platform_window_reports_metrics() {
    let sway = Sway::start();
    let mut platform = platform(&sway);

    let metrics = platform.window();

    assert!((metrics.width - 800.0).abs() < f32::EPSILON);
    assert!((metrics.height - 600.0).abs() < f32::EPSILON);
    assert!((metrics.scale - 1.0).abs() < f32::EPSILON);
}

#[test]
fn wayland_resize_event_follows_the_compositor() {
    let sway = Sway::start();
    let mut platform = platform(&sway);
    sway.msg(&["output", "HEADLESS-1", "resolution", "640x480"]);

    let metrics = pump_until(&mut platform, |_, events| {
        events.iter().find_map(|e| {
            if let RawEvent::Resize(m) = e {
                ((m.width - 640.0).abs() < 0.5).then_some(*m)
            } else {
                None
            }
        })
    });

    assert!((metrics.height - 480.0).abs() < 0.5);
    assert!((platform.window().width - 640.0).abs() < 0.5);
}

#[test]
fn wayland_close_is_reported() {
    let sway = Sway::start();
    let mut platform = platform(&sway);
    assert!(!platform.close_requested());
    sway.msg(&["kill"]);

    pump_until(&mut platform, |p, _| p.close_requested().then_some(()));
}

#[test]
fn wayland_output_scale_is_an_integer() {
    let sway = Sway::start();
    let mut platform = platform(&sway);
    sway.msg(&["output", "HEADLESS-1", "scale", "2"]);

    let metrics = pump_until(&mut platform, |p, _| {
        let m = p.window();
        ((m.scale - 2.0).abs() < f32::EPSILON && (m.width - 400.0).abs() < 0.5).then_some(m)
    });

    assert!((metrics.height - 300.0).abs() < 0.5);
}

#[test]
fn wayland_frame_callback_resolves_next_frame() {
    let sway = Sway::start();
    let mut platform = platform(&sway);

    block_on(platform.next_frame());
    block_on(platform.next_frame());

    assert!(!platform.frame_pending());
}

#[test]
fn wayland_schedule_update_requests_a_frame() {
    let sway = Sway::start();
    let mut platform = platform(&sway);
    assert!(!platform.frame_pending());

    platform.schedule_update();

    assert!(platform.frame_pending());
    block_on(platform.next_frame());
    assert!(!platform.frame_pending());
}

#[test]
fn wayland_platform_without_input_devices_returns_only_window_events() {
    let sway = Sway::start();
    let mut platform = platform(&sway);
    block_on(platform.next_frame());

    let mut events = Vec::new();
    platform.poll(&mut events);

    assert!(events.iter().all(|e| matches!(
        e,
        RawEvent::Resize(_) | RawEvent::Backend(_) | RawEvent::Focus(_)
    )));
    // The clipboard works since T170 (`tests/wayland_clipboard.rs`).
    assert_eq!(platform.clipboard_text(), None);
}

#[test]
fn wayland_idle_skips_swap_and_wakes_on_scheduled_update() {
    let sway = Sway::start();
    let mut platform = platform(&sway);
    assert!(platform.skips_idle_swap());

    block_on(platform.next_frame());
    assert!(!platform.frame_pending());

    platform.schedule_update();
    let started = std::time::Instant::now();
    block_on(platform.next_idle());
    assert!(started.elapsed() < Duration::from_millis(50));
}

#[test]
fn wayland_platform_connect_with_zero_msaa_swaps_cleanly() {
    let sway = Sway::start();
    let mut platform =
        WaylandPlatform::connect_with(Some(&sway.socket), "T172 test", "draw-test", 0).unwrap();

    block_on(platform.next_frame());

    assert!(!platform.frame_pending());
}

#[test]
fn wayland_idle_sleeps_until_wakeup() {
    let sway = Sway::start();
    let mut platform = platform(&sway);
    block_on(platform.next_frame());
    assert!(draw_sys::x11::signal::install_quit());

    // A quit signal wakes `next_idle` instead of polling on a 100 ms timer;
    // before the fix, `next_idle` sleeps in 100 ms chunks and returns early
    // on `IDLE_WAIT` without signal wakeups.
    let started = std::time::Instant::now();
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(30));
        // SAFETY: `SIGTERM` (15) has our `install_quit` async-signal-safe handler installed.
        unsafe {
            libc_raise(15);
        }
    });
    block_on(platform.next_idle());
    let _ = draw_sys::x11::signal::take();
    assert!(started.elapsed() < Duration::from_millis(80));
}

unsafe extern "C" {
    #[link_name = "raise"]
    fn libc_raise(sig: i32) -> i32;
}
