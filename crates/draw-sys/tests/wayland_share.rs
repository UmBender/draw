//! The shared GL handle of `WaylandPlatform` (T171 AC-0, ADR-T171-1) against
//! a private headless sway. A missing sway fails the tests, it never skips
//! them.

#![allow(clippy::unwrap_used)]

#[path = "common/sway.rs"]
mod sway;

use std::time::{Duration, Instant};

use draw_sys::egl::GlContext;
use draw_sys::platform::{Platform, RawEvent};
use draw_sys::wayland::WaylandPlatform;
use sway::Sway;

fn open(sway: &Sway) -> WaylandPlatform {
    WaylandPlatform::connect(Some(&sway.socket), "T171 title", "draw-test").unwrap()
}

#[test]
fn wayland_shared_gl_makes_current_and_swaps() {
    let sway = Sway::start();
    let platform = open(&sway);

    let shared = platform.share_gl();

    assert_eq!(shared.make_current(), Ok(()));
    assert!(shared.proc_address("glClear").is_some());
    assert_eq!(shared.swap_buffers(), Ok(()));
    assert_eq!(shared.window_size(), (800, 600));
}

#[test]
fn wayland_shared_gl_reports_the_resized_window_size() {
    let sway = Sway::start();
    let mut platform = open(&sway);
    let shared = platform.share_gl();
    sway.msg(&["output", "HEADLESS-1", "resolution", "640x480"]);

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut events: Vec<RawEvent> = Vec::new();
    while shared.window_size() != (640, 480) && Instant::now() < deadline {
        platform.poll(&mut events);
        std::thread::sleep(Duration::from_millis(20));
    }

    assert_eq!(shared.window_size(), (640, 480));
}

#[test]
fn wayland_shared_gl_dropped_before_the_platform_closes_cleanly() {
    let sway = Sway::start();
    let platform = open(&sway);
    let shared = platform.share_gl();
    assert_eq!(shared.make_current(), Ok(()));

    drop(shared);
    drop(platform);

    // The compositor is still alive and a new window opens: nothing crashed.
    drop(open(&sway));
}
