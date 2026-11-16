//! The shared GL handle of `X11Platform` (T178 AC-0, ADR-T178-1) under a
//! private Xvfb. A missing `Xvfb` or `xdotool` fails the test.

#![allow(clippy::unwrap_used)]

#[path = "common/xvfb.rs"]
mod xvfb;

use std::process::Command;
use std::time::{Duration, Instant};

use draw_sys::egl::GlContext;
use draw_sys::platform::{Platform, RawEvent};
use draw_sys::x11::{GlBackend, X11Platform};

fn open(server: &xvfb::Xvfb) -> X11Platform {
    X11Platform::open(
        Some(&server.display),
        "draw test",
        "draw-test",
        200,
        120,
        GlBackend::Glx,
    )
    .unwrap()
}

#[test]
fn x11_shared_gl_makes_current_and_swaps() {
    let server = xvfb::Xvfb::start();
    let platform = open(&server);

    let shared = platform.share_gl();

    assert_eq!(shared.make_current(), Ok(()));
    assert!(shared.proc_address("glClear").is_some());
    assert_eq!(shared.swap_buffers(), Ok(()));
    assert_eq!(shared.window_size(), (200, 120));
}

#[test]
fn x11_shared_gl_reports_the_resized_window_size() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server);
    let shared = platform.share_gl();
    let id = platform.window_id().to_string();

    Command::new("xdotool")
        .env("DISPLAY", &server.display)
        .args(["windowsize", &id, "150", "90"])
        .status()
        .unwrap_or_else(|e| panic!("xdotool is required for the X11 tests: {e}"));
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut events: Vec<RawEvent> = Vec::new();
    while shared.window_size() != (150, 90) && Instant::now() < deadline {
        platform.poll(&mut events);
        std::thread::sleep(Duration::from_millis(10));
    }

    assert_eq!(shared.window_size(), (150, 90));
}

#[test]
fn x11_shared_gl_dropped_before_the_platform_closes_cleanly() {
    let server = xvfb::Xvfb::start();
    let platform = open(&server);
    let shared = platform.share_gl();
    assert_eq!(shared.make_current(), Ok(()));

    drop(shared);
    drop(platform);

    // The server is still alive and a new window opens: nothing crashed.
    drop(open(&server));
}
