//! The Wayland seat, keyboard and pointer (T169): the pure decoders without
//! a compositor, and the whole path against a private headless sway driven
//! with `wtype` and `swaymsg seat - cursor`. A missing sway or `wtype` fails
//! the tests, it never skips them.

#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

#[path = "common/sway.rs"]
mod sway;

use std::time::{Duration, Instant};

use draw_sys::platform::{
    BackendEvent, Device, DeviceChange, Platform, RawButton, RawEvent, RawKey,
};
use draw_sys::wayland::WaylandPlatform;
use draw_sys::wayland::input::{
    AxisFrame, MAX_REPEATS_PER_POLL, SeatDevices, fixed_to_f32, keymap_usable, pointer_position,
    raw_button_from_evdev, raw_key_from_keysym, repeats_due, repeats_to_send, wheel_of_frame,
};
use draw_sys::wayland::xkb::Xkb;
use sway::Sway;

const CAP_POINTER: u32 = 1;
const CAP_KEYBOARD: u32 = 2;
const CAP_TOUCH: u32 = 4;

// ---- seat decisions (AC-1) ----

#[test]
fn seat_devices_first_capabilities_create_keyboard_and_pointer() {
    let mut seat = SeatDevices::default();

    let changes = seat.on_capabilities(CAP_POINTER | CAP_KEYBOARD);

    assert!(changes.contains(&(Device::Keyboard, DeviceChange::Create)));
    assert!(changes.contains(&(Device::Pointer, DeviceChange::Create)));
    assert_eq!(changes.len(), 2);
}

#[test]
fn seat_devices_capabilities_resent_keep_devices() {
    let mut seat = SeatDevices::default();
    seat.on_capabilities(CAP_POINTER | CAP_KEYBOARD);

    let again = seat.on_capabilities(CAP_POINTER | CAP_KEYBOARD);

    assert!(again.is_empty());
}

#[test]
fn seat_devices_lost_capability_releases_device() {
    let mut seat = SeatDevices::default();
    seat.on_capabilities(CAP_POINTER | CAP_KEYBOARD);

    let changes = seat.on_capabilities(CAP_POINTER);

    assert_eq!(changes, vec![(Device::Keyboard, DeviceChange::Release)]);
    assert!(!seat.has_keyboard());
    assert!(seat.has_pointer());
}

#[test]
fn seat_devices_never_create_touch() {
    let mut seat = SeatDevices::default();

    let changes = seat.on_capabilities(CAP_TOUCH);

    assert!(changes.is_empty());
}

#[test]
fn regression_T45_capabilities_resent_keeps_one_keyboard() {
    let mut seat = SeatDevices::default();
    let mut keyboards = 0;
    for _ in 0..5 {
        for (device, change) in seat.on_capabilities(CAP_POINTER | CAP_KEYBOARD) {
            if device == Device::Keyboard && change == DeviceChange::Create {
                keyboards += 1;
            }
        }
    }

    assert_eq!(keyboards, 1);
}

// ---- keyboard decoders (AC-2) ----

#[test]
fn keymap_usable_accepts_xkb_v1_with_a_size() {
    assert!(keymap_usable(1, 4096));
}

#[test]
fn regression_T79_no_keymap_is_unusable() {
    assert!(!keymap_usable(0, 0));
    assert!(!keymap_usable(0, 4096));
    assert!(!keymap_usable(1, 0));
    assert!(!keymap_usable(7, 4096));
}

#[test]
fn xkb_garbage_keymap_is_none() {
    let xkb = Xkb::load().expect("libxkbcommon is required");

    assert!(xkb.keymap_from_text(b"this is not a keymap").is_none());
    assert!(xkb.keymap_from_text(b"").is_none());
}

#[test]
fn raw_key_letters_map_from_either_case() {
    assert_eq!(raw_key_from_keysym(0x61), RawKey::Letter('A'));
    assert_eq!(raw_key_from_keysym(0x5a), RawKey::Letter('Z'));
}

#[test]
fn raw_key_digits_and_named_keys_map() {
    assert_eq!(raw_key_from_keysym(0x35), RawKey::Digit(5));
    assert_eq!(raw_key_from_keysym(0xff1b), RawKey::Escape);
    assert_eq!(raw_key_from_keysym(0x20), RawKey::Space);
    assert_eq!(raw_key_from_keysym(0xff09), RawKey::Tab);
    assert_eq!(raw_key_from_keysym(0xff08), RawKey::Backspace);
    assert_eq!(raw_key_from_keysym(0xffff), RawKey::Delete);
    assert_eq!(raw_key_from_keysym(0xff0d), RawKey::Enter);
    assert_eq!(raw_key_from_keysym(0xff8d), RawKey::Enter);
    assert_eq!(raw_key_from_keysym(0xff50), RawKey::Home);
    assert_eq!(raw_key_from_keysym(0xff57), RawKey::End);
    assert_eq!(raw_key_from_keysym(0xff51), RawKey::ArrowLeft);
    assert_eq!(raw_key_from_keysym(0xff52), RawKey::ArrowUp);
    assert_eq!(raw_key_from_keysym(0xff53), RawKey::ArrowRight);
    assert_eq!(raw_key_from_keysym(0xff54), RawKey::ArrowDown);
    assert_eq!(raw_key_from_keysym(0x5b), RawKey::BracketLeft);
    assert_eq!(raw_key_from_keysym(0x5d), RawKey::BracketRight);
    assert_eq!(raw_key_from_keysym(0xffe1), RawKey::LeftShift);
    assert_eq!(raw_key_from_keysym(0xffe2), RawKey::RightShift);
    assert_eq!(raw_key_from_keysym(0xffe3), RawKey::LeftControl);
    assert_eq!(raw_key_from_keysym(0xffe4), RawKey::RightControl);
    assert_eq!(raw_key_from_keysym(0xffe9), RawKey::LeftAlt);
    assert_eq!(raw_key_from_keysym(0xffea), RawKey::RightAlt);
}

#[test]
fn raw_key_unknown_keysym_is_other() {
    assert_eq!(raw_key_from_keysym(0xffbe), RawKey::Other(0xffbe));
}

#[test]
fn repeats_due_before_the_delay_is_zero() {
    assert_eq!(repeats_due(25, 600, Duration::from_millis(599), 0), 0);
}

#[test]
fn repeats_due_after_the_delay_follows_the_rate() {
    // 25 per second: one every 40 ms, the first at the delay.
    assert_eq!(repeats_due(25, 600, Duration::from_millis(600), 0), 1);
    assert_eq!(repeats_due(25, 600, Duration::from_millis(680), 0), 3);
    assert_eq!(repeats_due(25, 600, Duration::from_millis(680), 3), 0);
}

#[test]
fn repeats_due_rate_zero_never_repeats() {
    assert_eq!(repeats_due(0, 600, Duration::from_secs(60), 0), 0);
    assert_eq!(repeats_due(-5, 600, Duration::from_secs(60), 0), 0);
}

#[test]
fn regression_T45_stalled_loop_drops_surplus_repeats() {
    let due = repeats_due(25, 600, Duration::from_secs(600), 0);
    assert!(due > MAX_REPEATS_PER_POLL);

    assert_eq!(
        repeats_to_send(25, 600, Duration::from_secs(600), 0),
        MAX_REPEATS_PER_POLL
    );
}

// ---- pointer decoders (AC-3) ----

#[test]
fn fixed_to_f32_reads_24_8() {
    assert!((fixed_to_f32(256) - 1.0).abs() < 1e-6);
    assert!((fixed_to_f32(-384) + 1.5).abs() < 1e-6);
}

#[test]
fn pointer_position_scales_to_physical_pixels() {
    let (x, y) = pointer_position(100 * 256, 50 * 256, 2);

    assert!((x - 200.0).abs() < 1e-4);
    assert!((y - 100.0).abs() < 1e-4);
}

#[test]
fn raw_button_maps_evdev_codes() {
    assert_eq!(raw_button_from_evdev(0x110), RawButton::Left);
    assert_eq!(raw_button_from_evdev(0x111), RawButton::Right);
    assert_eq!(raw_button_from_evdev(0x112), RawButton::Middle);
    assert_eq!(raw_button_from_evdev(0x113), RawButton::Other);
}

#[test]
fn wheel_smooth_axis_flips_the_vertical_sign() {
    let frame = AxisFrame {
        dx: 0.0,
        dy: 10.0,
        v120_x: None,
        v120_y: None,
    };

    assert_eq!(wheel_of_frame(&frame), Some((0.0, -10.0)));
}

#[test]
fn wheel_value120_wins_over_the_smooth_axis() {
    let frame = AxisFrame {
        dx: 0.0,
        dy: 15.0,
        v120_x: None,
        v120_y: Some(120),
    };

    assert_eq!(wheel_of_frame(&frame), Some((0.0, -10.0)));
}

#[test]
fn wheel_empty_frame_is_none() {
    assert_eq!(wheel_of_frame(&AxisFrame::default()), None);
}

// ---- fuzz of the decoders (AC-4) ----

struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

#[test]
fn fuzz_keymap_decoder_never_panics() {
    let xkb = Xkb::load().expect("libxkbcommon is required");
    let mut rng = XorShift(0x9e37_79b9_7f4a_7c15);
    for _ in 0..300 {
        let len = (rng.next() % 600) as usize;
        let bytes: Vec<u8> = (0..len).map(|_| rng.next().to_le_bytes()[0]).collect();
        let _ = xkb.keymap_from_text(&bytes);
    }
    for text in [
        &b"xkb_keymap {"[..],
        b"xkb_keymap { xkb_keycodes { }; };",
        b"\xff\xfe\x00\x01",
    ] {
        let _ = xkb.keymap_from_text(text);
    }
}

#[test]
fn fuzz_pure_decoders_never_panic() {
    let mut rng = XorShift(0x2545_f491_4f6c_dd1d);
    for _ in 0..20_000 {
        let a = rng.next();
        let b = rng.next();
        let sym = (a & 0xffff_ffff) as u32;
        let fixed = (b & 0xffff_ffff) as u32 as i32;
        let _ = raw_key_from_keysym(sym);
        let _ = raw_button_from_evdev(sym);
        let _ = keymap_usable(sym, (b >> 32) as u32);
        let (x, y) = pointer_position(fixed, (a >> 32) as u32 as i32, fixed % 9);
        assert!(x.is_finite() && y.is_finite());
        let _ = repeats_due(
            fixed,
            (b >> 32) as u32 as i32,
            Duration::from_nanos(a),
            (b >> 8) as u32,
        );
        let _ = repeats_to_send(fixed, 0, Duration::MAX, u32::MAX);
        let frame = AxisFrame {
            dx: fixed_to_f32(fixed),
            dy: f32::from_bits(sym),
            v120_x: Some(fixed),
            v120_y: None,
        };
        if let Some((dx, dy)) = wheel_of_frame(&frame) {
            assert!(dx.is_finite() && dy.is_finite());
        }
        let mut seat = SeatDevices::default();
        let _ = seat.on_capabilities(sym);
        let _ = seat.on_capabilities((b >> 32) as u32);
    }
}

// ---- the whole path against sway ----

fn platform(sway: &Sway) -> WaylandPlatform {
    WaylandPlatform::connect(Some(&sway.socket), "T169 title", "draw-test").unwrap()
}

/// Polls until `done` holds for the events gathered so far; fails after 5 s.
fn collect_until(
    platform: &mut WaylandPlatform,
    done: impl Fn(&[RawEvent]) -> bool,
) -> Vec<RawEvent> {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut all = Vec::new();
    loop {
        platform.poll(&mut all);
        if done(&all) {
            return all;
        }
        assert!(Instant::now() < deadline, "events missing; got {all:?}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn count_backend(events: &[RawEvent], wanted: BackendEvent) -> usize {
    events
        .iter()
        .filter(|e| matches!(e, RawEvent::Backend(r) if r.event == wanted))
        .count()
}

/// Opens the platform and waits until it has keyboard focus.
fn focused(sway: &Sway) -> (WaylandPlatform, Vec<RawEvent>) {
    let mut platform = platform(sway);
    let events = collect_until(&mut platform, |e| {
        e.iter().any(|e| matches!(e, RawEvent::Focus(true)))
    });
    (platform, events)
}

#[test]
fn wayland_seat_has_one_keyboard_and_one_pointer() {
    let sway = Sway::start();
    let _keyboard = sway.keyboard(&["-s", "8000", "a"]);
    let _pointer = sway.virtual_pointer();
    let mut platform = platform(&sway);

    let events = collect_until(&mut platform, |e| {
        count_backend(e, BackendEvent::DeviceCreated(Device::Keyboard)) > 0
            && count_backend(e, BackendEvent::DeviceCreated(Device::Pointer)) > 0
    });

    assert_eq!(
        count_backend(&events, BackendEvent::DeviceCreated(Device::Keyboard)),
        1
    );
    assert_eq!(
        count_backend(&events, BackendEvent::DeviceCreated(Device::Pointer)),
        1
    );
}

#[test]
fn wayland_keyboard_focus_is_reported() {
    let sway = Sway::start();
    let _keyboard = sway.keyboard(&["-s", "8000", "a"]);

    let (_platform, events) = focused(&sway);

    assert!(events.iter().any(|e| matches!(e, RawEvent::Focus(true))));
}

#[test]
fn wayland_keyboard_typed_text_arrives() {
    let sway = Sway::start();
    // The virtual keyboard exists first; the keys come 1.5 s later, when
    // the platform has its keyboard and focus.
    let _keyboard = sway.keyboard(&["-s", "1500", "ab", "-s", "8000"]);
    let (mut platform, _) = focused(&sway);

    let events = collect_until(&mut platform, |e| {
        e.iter()
            .filter(|e| matches!(e, RawEvent::Char { .. }))
            .count()
            >= 2
    });

    let chars: String = events
        .iter()
        .filter_map(|e| match e {
            RawEvent::Char { ch, repeat: false, .. } => Some(*ch),
            _ => None,
        })
        .collect();
    assert_eq!(chars, "ab");
    let downs = events
        .iter()
        .filter(|e| matches!(e, RawEvent::Key { key: RawKey::Letter('A'), down: true, .. }))
        .count();
    assert_eq!(downs, 1, "one key press, not doubled (T44, T45): {events:?}");
}

#[test]
fn wayland_keyboard_modifiers_arrive() {
    let sway = Sway::start();
    let _keyboard = sway.keyboard(&["-s", "1500", "-M", "ctrl", "z", "-m", "ctrl", "-s", "8000"]);
    let (mut platform, _) = focused(&sway);

    let events = collect_until(&mut platform, |e| {
        e.iter().any(|e| {
            matches!(e, RawEvent::Key { key: RawKey::Letter('Z'), down: true, .. })
        })
    });

    assert!(events.iter().any(|e| matches!(
        e,
        RawEvent::Key { key: RawKey::Letter('Z'), mods, down: true, .. } if mods.ctrl && !mods.shift
    )));
}

#[test]
fn wayland_pointer_motion_arrives() {
    let sway = Sway::start();
    let _pointer = sway.virtual_pointer();
    let mut platform = platform(&sway);

    sway.cursor(&["set", "250", "150"]);
    let events = collect_until(&mut platform, |e| {
        e.iter().any(|e| matches!(e, RawEvent::PointerMove { .. }))
    });

    let moved = events
        .iter()
        .find_map(|e| match e {
            RawEvent::PointerMove { x, y } => Some((*x, *y)),
            _ => None,
        })
        .unwrap();
    assert!((moved.0 - 250.0).abs() < 2.0 && (moved.1 - 150.0).abs() < 2.0);
}

#[test]
fn wayland_pointer_buttons_arrive() {
    let sway = Sway::start();
    let _pointer = sway.virtual_pointer();
    let mut platform = platform(&sway);
    sway.cursor(&["set", "200", "100"]);
    collect_until(&mut platform, |e| {
        e.iter().any(|e| matches!(e, RawEvent::PointerMove { .. }))
    });

    sway.cursor(&["press", "button1"]);
    sway.cursor(&["release", "button1"]);
    let events = collect_until(&mut platform, |e| {
        e.iter()
            .filter(|e| matches!(e, RawEvent::Button { .. }))
            .count()
            >= 2
    });

    let buttons: Vec<(RawButton, bool, f32, f32)> = events
        .iter()
        .filter_map(|e| match e {
            RawEvent::Button { button, down, x, y } => Some((*button, *down, *x, *y)),
            _ => None,
        })
        .collect();
    assert_eq!(buttons[0].0, RawButton::Left);
    assert!(buttons[0].1);
    assert!(!buttons[1].1);
    assert!((buttons[0].2 - 200.0).abs() < 2.0 && (buttons[0].3 - 100.0).abs() < 2.0);
}

#[test]
fn wayland_pointer_wheel_arrives() {
    let sway = Sway::start();
    let mut pointer = sway.virtual_pointer();
    let mut platform = platform(&sway);
    sway.cursor(&["set", "300", "300"]);
    collect_until(&mut platform, |e| {
        e.iter().any(|e| matches!(e, RawEvent::PointerMove { .. }))
    });

    pointer.wheel(1);
    let events = collect_until(&mut platform, |e| {
        e.iter().any(|e| matches!(e, RawEvent::Wheel { .. }))
    });

    let dy = events
        .iter()
        .find_map(|e| match e {
            RawEvent::Wheel { dy, .. } => Some(*dy),
            _ => None,
        })
        .unwrap();
    assert!(dy.is_finite() && dy != 0.0);
}
