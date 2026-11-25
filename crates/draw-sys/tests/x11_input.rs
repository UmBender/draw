//! Keyboard, pointer and `XInput2` of `X11Platform` under a private Xvfb
//! (T176 AC-1 to AC-3). A missing `Xvfb` or `xdotool` fails the test.

#![allow(clippy::unwrap_used)]

#[path = "common/xvfb.rs"]
mod xvfb;

use std::process::Command;
use std::time::{Duration, Instant};

use draw_sys::platform::{Platform, RawButton, RawEvent, RawKey};
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

fn xdotool(server: &xvfb::Xvfb, args: &[&str]) -> String {
    let out = Command::new("xdotool")
        .env("DISPLAY", &server.display)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("xdotool is required for the X11 tests: {e}"));
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

/// Opens a window that has the keyboard focus, with its map events drained.
fn focused(server: &xvfb::Xvfb) -> X11Platform {
    let mut platform = open(server);
    let id = platform.window_id().to_string();
    xdotool(server, &["windowfocus", &id]);
    collect(&mut platform, Duration::from_millis(150));
    platform
}

/// Every event that arrives within `span`.
fn collect(platform: &mut X11Platform, span: Duration) -> Vec<RawEvent> {
    let deadline = Instant::now() + span;
    let mut all = Vec::new();
    while Instant::now() < deadline {
        platform.poll(&mut all);
        std::thread::sleep(Duration::from_millis(10));
    }
    all
}

fn key_down(event: &RawEvent, key: RawKey) -> bool {
    matches!(event, RawEvent::Key { key: k, down: true, .. } if *k == key)
}

#[test]
fn x11_key_by_xdotool_reports_key_and_char() {
    let server = xvfb::Xvfb::start();
    let mut platform = focused(&server);

    xdotool(&server, &["key", "a"]);
    let events = collect(&mut platform, Duration::from_millis(300));

    assert!(events.iter().any(|e| key_down(e, RawKey::Letter('A'))));
    assert!(events.iter().any(|e| matches!(
        e,
        RawEvent::Char {
            ch: 'a',
            repeat: false,
            ..
        }
    )));
    assert!(events.iter().any(|e| matches!(
        e,
        RawEvent::Key {
            key: RawKey::Letter('A'),
            down: false,
            ..
        }
    )));
}

#[test]
fn x11_ctrl_key_reports_mods_and_no_char() {
    let server = xvfb::Xvfb::start();
    let mut platform = focused(&server);

    xdotool(&server, &["key", "ctrl+a"]);
    let events = collect(&mut platform, Duration::from_millis(300));

    assert!(events.iter().any(|e| matches!(
        e,
        RawEvent::Key { key: RawKey::Letter('A'), mods, down: true, .. } if mods.ctrl
    )));
    assert!(events.iter().all(|e| !matches!(e, RawEvent::Char { .. })));
}

#[test]
fn x11_held_key_reports_repeat_until_release() {
    // Regression T45 (the X11 counterpart): one press, then repeats, one
    // release; never a release/press pair per repeat.
    let server = xvfb::Xvfb::start();
    let mut platform = focused(&server);

    xdotool(&server, &["keydown", "a"]);
    let mut events = collect(&mut platform, Duration::from_millis(1500));
    xdotool(&server, &["keyup", "a"]);
    events.extend(collect(&mut platform, Duration::from_millis(200)));

    let presses = |repeat: bool| {
        events
            .iter()
            .filter(|e| matches!(e, RawEvent::Key { key: RawKey::Letter('A'), down: true, repeat: r, .. } if *r == repeat))
            .count()
    };
    let releases = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                RawEvent::Key {
                    key: RawKey::Letter('A'),
                    down: false,
                    ..
                }
            )
        })
        .count();
    assert_eq!(presses(false), 1);
    assert!(presses(true) >= 1);
    assert_eq!(releases, 1);
}

#[test]
fn x11_focus_events_reach_poll() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server);
    let id = platform.window_id().to_string();

    xdotool(&server, &["windowfocus", &id]);
    let events = collect(&mut platform, Duration::from_millis(200));

    assert!(events.iter().any(|e| matches!(e, RawEvent::Focus(true))));
}

#[test]
fn x11_regression_t28_key_release_after_focus_loss_clears_state() {
    // The window loses focus while `a` is held: its release goes elsewhere.
    // The next press of `a` is a new press, not a repeat.
    let server = xvfb::Xvfb::start();
    let mut platform = focused(&server);
    let id = platform.window_id().to_string();
    xdotool(&server, &["keydown", "a"]);
    collect(&mut platform, Duration::from_millis(100));
    xdotool(&server, &["windowunmap", &id]);
    let events = collect(&mut platform, Duration::from_millis(200));
    assert!(events.iter().any(|e| matches!(e, RawEvent::Focus(false))));
    xdotool(&server, &["keyup", "a"]);

    xdotool(&server, &["windowmap", &id]);
    xdotool(&server, &["windowfocus", &id]);
    collect(&mut platform, Duration::from_millis(200));
    xdotool(&server, &["key", "a"]);
    let events = collect(&mut platform, Duration::from_millis(300));

    assert!(events.iter().any(|e| matches!(
        e,
        RawEvent::Key {
            key: RawKey::Letter('A'),
            down: true,
            repeat: false,
            ..
        }
    )));
}

#[test]
fn x11_pointer_motion_by_xdotool_reports_position() {
    let server = xvfb::Xvfb::start();
    let mut platform = focused(&server);

    xdotool(&server, &["mousemove", "20", "30"]);
    let events = collect(&mut platform, Duration::from_millis(200));

    assert!(events.iter().any(|e| matches!(
        e,
        RawEvent::PointerMove { x, y } if (*x - 20.0).abs() < 1.0 && (*y - 30.0).abs() < 1.0
    )));
}

#[test]
fn x11_button_press_and_release_report_button_and_position() {
    let server = xvfb::Xvfb::start();
    let mut platform = focused(&server);
    xdotool(&server, &["mousemove", "10", "12"]);
    collect(&mut platform, Duration::from_millis(100));

    xdotool(&server, &["click", "3"]);
    let events = collect(&mut platform, Duration::from_millis(200));

    for want_down in [true, false] {
        assert!(events.iter().any(|e| matches!(
            e,
            RawEvent::Button { button: RawButton::Right, x, y, down }
                if *down == want_down && (*x - 10.0).abs() < 1.0 && (*y - 12.0).abs() < 1.0
        )));
    }
}

#[test]
fn x11_wheel_buttons_report_wheel_and_no_button() {
    let server = xvfb::Xvfb::start();
    let mut platform = focused(&server);
    xdotool(&server, &["mousemove", "10", "12"]);
    collect(&mut platform, Duration::from_millis(100));

    xdotool(&server, &["click", "5"]);
    let events = collect(&mut platform, Duration::from_millis(200));

    assert!(events.iter().any(|e| matches!(
        e,
        RawEvent::Wheel { dx, dy } if dx.abs() < 1e-6 && (*dy + 1.0).abs() < 1e-6
    )));
    assert!(events.iter().all(|e| !matches!(e, RawEvent::Button { .. })));
}

#[test]
fn x11_xi2_raw_motion_from_xtest_is_not_a_pen() {
    // Xvfb has no tablet: the XTEST pointer sends raw motion, its device
    // has no pressure axis, so no pen sample is produced.
    let server = xvfb::Xvfb::start();
    let mut platform = focused(&server);
    assert!(platform.pen_supported());

    xdotool(&server, &["mousemove", "5", "6"]);
    xdotool(&server, &["mousemove", "25", "26"]);
    let events = collect(&mut platform, Duration::from_millis(300));

    assert!(
        events
            .iter()
            .any(|e| matches!(e, RawEvent::PointerMove { .. }))
    );
    assert!(events.iter().all(|e| !matches!(e, RawEvent::Tablet { .. })));
}

#[test]
fn regression_t196_raw_motion_arrives_during_button_grab() {
    // Arrange: a button press activates an X11 implicit passive pointer grab.
    // With XInput 2.0, XI_RawMotion stops during the grab (pinning pen
    // pressure at its hover value for the whole stroke); XInput 2.1 keeps
    // delivering raw motion while the button is held.
    let server = xvfb::Xvfb::start();
    let mut platform = focused(&server);
    assert!(platform.pen_supported());
    xdotool(&server, &["mousemove", "10", "10"]);
    collect(&mut platform, Duration::from_millis(150));

    // Act: hold button 1 and drag.
    xdotool(&server, &["mousedown", "1"]);
    collect(&mut platform, Duration::from_millis(100));
    let before = platform.raw_motion_count();
    xdotool(&server, &["mousemove", "40", "45"]);
    xdotool(&server, &["mousemove", "60", "65"]);
    collect(&mut platform, Duration::from_millis(200));
    xdotool(&server, &["mouseup", "1"]);
    collect(&mut platform, Duration::from_millis(100));

    // Assert: raw motion kept arriving during the grab.
    assert!(
        platform.raw_motion_count() > before,
        "XI_RawMotion stopped during button 1 grab (before={before}, after={})",
        platform.raw_motion_count()
    );
}

#[test]
fn x11_startup_queues_debug_lines_for_valuator_devices() {
    // Arrange & Act: opening the platform probes all XInput2 devices.
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server);
    let mut lines = Vec::new();
    while let Some(line) = platform.take_debug_log() {
        lines.push(line);
    }

    // Assert: Xvfb exposes pointer valuators without Abs Pressure.
    assert!(
        lines
            .iter()
            .any(|line| line.contains("valuators") && line.contains("no Abs Pressure")),
        "expected a debug line for Xvfb's valuator devices, got: {lines:?}"
    );
}
