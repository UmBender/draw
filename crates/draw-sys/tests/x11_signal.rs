//! Quit signals end the X11 loop the way a window-manager close does (T177
//! AC-2). One test: the signal flag is process-wide.

#![allow(clippy::unwrap_used)]

#[path = "common/xvfb.rs"]
mod xvfb;

use std::process::Command;
use std::time::{Duration, Instant};

use draw_sys::platform::Platform;
use draw_sys::signal;
use draw_sys::x11::{GlBackend, X11Platform};

fn send(signal: &str) {
    let pid = std::process::id().to_string();
    assert!(
        Command::new("kill")
            .args([signal, &pid])
            .status()
            .unwrap()
            .success()
    );
}

fn poll_until_close(platform: &mut X11Platform) -> bool {
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut events = Vec::new();
    while Instant::now() < deadline {
        platform.poll(&mut events);
        if platform.close_requested() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    false
}

#[test]
fn x11_quit_signals_set_close_requested() {
    let server = xvfb::Xvfb::start();
    assert!(signal::install_quit());
    let mut platform = X11Platform::open(
        Some(&server.display),
        "draw test",
        "draw-test",
        100,
        80,
        GlBackend::Glx,
    )
    .unwrap();
    let mut events = Vec::new();
    platform.poll(&mut events);
    assert!(!platform.close_requested());
    assert_eq!(platform.quit_signal(), None);

    send("-INT");

    assert!(poll_until_close(&mut platform), "SIGINT was not seen");
    assert_eq!(platform.quit_signal(), Some(2));

    let mut second = X11Platform::open(
        Some(&server.display),
        "draw test",
        "draw-test",
        100,
        80,
        GlBackend::Glx,
    )
    .unwrap();
    send("-TERM");

    assert!(poll_until_close(&mut second), "SIGTERM was not seen");
    assert_eq!(second.quit_signal(), Some(15));
}
