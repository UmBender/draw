//! The X11 key, pointer and pen decoders without a server (T176 AC-1 to
//! AC-4). The fuzz uses a seeded xorshift generator: `proptest` is not a
//! dev-dependency of this crate.

#![allow(clippy::unwrap_used)]

use draw_sys::platform::{RawButton, RawEvent, RawKey, RawMods};
use draw_sys::x11::event::{EVENT_BYTES, XEvent, decode};
use draw_sys::x11::keys::{
    KeyTracker, button_from_x, char_from_keysym, group_of, key_from_keysym, level_for,
    mods_from_state, wheel_delta,
};
use draw_sys::x11::pen::{PenTracker, PressureAxis, normalise, valuator};

fn event_of(kind: i32) -> [u8; EVENT_BYTES] {
    let mut raw = [0u8; EVENT_BYTES];
    raw[0..4].copy_from_slice(&kind.to_ne_bytes());
    raw
}

fn put_i32(raw: &mut [u8], at: usize, value: i32) {
    raw[at..at + 4].copy_from_slice(&value.to_ne_bytes());
}

fn put_u32(raw: &mut [u8], at: usize, value: u32) {
    raw[at..at + 4].copy_from_slice(&value.to_ne_bytes());
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

// ---- keys ----

#[test]
fn keys_keysym_letters_map_to_uppercase_letters() {
    assert_eq!(key_from_keysym(0x61, 38), RawKey::Letter('A'));
    assert_eq!(key_from_keysym(0x5a, 52), RawKey::Letter('Z'));
    assert_eq!(key_from_keysym(0x30, 19), RawKey::Digit(0));
    assert_eq!(key_from_keysym(0x39, 18), RawKey::Digit(9));
}

#[test]
fn keys_keysym_special_keys_map_to_named_keys() {
    assert_eq!(key_from_keysym(0xff1b, 9), RawKey::Escape);
    assert_eq!(key_from_keysym(0x20, 65), RawKey::Space);
    assert_eq!(key_from_keysym(0xff09, 23), RawKey::Tab);
    assert_eq!(key_from_keysym(0xfe20, 23), RawKey::Tab);
    assert_eq!(key_from_keysym(0xff08, 22), RawKey::Backspace);
    assert_eq!(key_from_keysym(0xffff, 119), RawKey::Delete);
    assert_eq!(key_from_keysym(0xff0d, 36), RawKey::Enter);
    assert_eq!(key_from_keysym(0xff8d, 104), RawKey::Enter);
    assert_eq!(key_from_keysym(0xff50, 110), RawKey::Home);
    assert_eq!(key_from_keysym(0xff57, 115), RawKey::End);
    assert_eq!(key_from_keysym(0xff51, 113), RawKey::ArrowLeft);
    assert_eq!(key_from_keysym(0xff52, 111), RawKey::ArrowUp);
    assert_eq!(key_from_keysym(0xff53, 114), RawKey::ArrowRight);
    assert_eq!(key_from_keysym(0xff54, 116), RawKey::ArrowDown);
    assert_eq!(key_from_keysym(0x5b, 34), RawKey::BracketLeft);
    assert_eq!(key_from_keysym(0x5d, 35), RawKey::BracketRight);
    assert_eq!(key_from_keysym(0xffe1, 50), RawKey::LeftShift);
    assert_eq!(key_from_keysym(0xffe2, 62), RawKey::RightShift);
    assert_eq!(key_from_keysym(0xffe3, 37), RawKey::LeftControl);
    assert_eq!(key_from_keysym(0xffe4, 105), RawKey::RightControl);
    assert_eq!(key_from_keysym(0xffe9, 64), RawKey::LeftAlt);
    assert_eq!(key_from_keysym(0xffea, 108), RawKey::RightAlt);
}

#[test]
fn keys_unknown_keysym_is_other_with_evdev_code() {
    // F1 has keycode 67; the code is the evdev one (XKB keycode minus 8).
    assert_eq!(key_from_keysym(0xffbe, 67), RawKey::Other(59));
    assert_eq!(key_from_keysym(0, 3), RawKey::Other(0));
}

#[test]
fn keys_char_from_keysym_latin1_and_unicode() {
    assert_eq!(char_from_keysym(0x61), Some('a'));
    assert_eq!(char_from_keysym(0x20), Some(' '));
    assert_eq!(char_from_keysym(0xe9), Some('é'));
    assert_eq!(char_from_keysym(0x0100_20ac), Some('€'));
    assert_eq!(char_from_keysym(0xffb7), Some('7'));
}

#[test]
fn keys_char_from_keysym_rejects_controls() {
    assert_eq!(char_from_keysym(0xff1b), None);
    assert_eq!(char_from_keysym(0xffe1), None);
    assert_eq!(char_from_keysym(0x01), None);
    assert_eq!(char_from_keysym(0x7f), None);
    assert_eq!(char_from_keysym(0x0100_d800), None);
    assert_eq!(char_from_keysym(0x0111_0000), None);
}

#[test]
fn keys_mods_from_state_reads_shift_control_alt() {
    assert_eq!(mods_from_state(0), RawMods::default());
    assert_eq!(
        mods_from_state(1 << 2),
        RawMods {
            shift: false,
            ctrl: true,
            alt: false
        }
    );
    assert_eq!(
        mods_from_state(1 | (1 << 3)),
        RawMods {
            shift: true,
            ctrl: false,
            alt: true
        }
    );
    // Caps lock (1 << 1) and the logo key (1 << 6) are not reported.
    assert_eq!(mods_from_state((1 << 1) | (1 << 6)), RawMods::default());
}

#[test]
fn keys_level_uses_shift_lock_and_altgr() {
    assert_eq!(level_for(0, true), 0);
    assert_eq!(level_for(1, false), 1);
    // Caps lock shifts letters only, and shift undoes it.
    assert_eq!(level_for(1 << 1, true), 1);
    assert_eq!(level_for(1 << 1, false), 0);
    assert_eq!(level_for(1 | (1 << 1), true), 0);
    // AltGr (Mod5) selects the third level, with shift the fourth.
    assert_eq!(level_for(1 << 7, false), 2);
    assert_eq!(level_for(1 | (1 << 7), false), 3);
    assert_eq!(group_of(1 << 13), 1);
    assert_eq!(group_of(3 << 13), 3);
    assert_eq!(group_of(0xffff_ffff), 3);
}

#[test]
fn key_tracker_second_press_is_repeat() {
    let mut keys = KeyTracker::default();

    assert!(!keys.press(38));
    assert!(keys.press(38));
    assert!(keys.press(38));
}

#[test]
fn key_tracker_press_after_release_is_not_repeat() {
    let mut keys = KeyTracker::default();
    keys.press(38);
    keys.release(38);

    assert!(!keys.press(38));
    // Another key is unaffected.
    assert!(!keys.press(39));
}

#[test]
fn key_tracker_clear_forgets_held_keys() {
    let mut keys = KeyTracker::default();
    keys.press(38);

    keys.clear();

    assert!(!keys.press(38));
}

#[test]
fn keys_button_maps_left_middle_right_other() {
    assert_eq!(button_from_x(1), RawButton::Left);
    assert_eq!(button_from_x(2), RawButton::Middle);
    assert_eq!(button_from_x(3), RawButton::Right);
    assert_eq!(button_from_x(8), RawButton::Other);
    assert_eq!(button_from_x(0), RawButton::Other);
}

#[test]
fn keys_wheel_buttons_map_to_deltas() {
    assert_eq!(wheel_delta(4), Some((0.0, 1.0)));
    assert_eq!(wheel_delta(5), Some((0.0, -1.0)));
    assert_eq!(wheel_delta(6), Some((1.0, 0.0)));
    assert_eq!(wheel_delta(7), Some((-1.0, 0.0)));
    assert_eq!(wheel_delta(1), None);
    assert_eq!(wheel_delta(8), None);
}

// ---- events ----

#[test]
fn event_decode_key_reads_keycode_and_state() {
    let mut raw = event_of(2);
    put_u32(&mut raw, 80, 0x14);
    put_u32(&mut raw, 84, 38);
    let mut up = raw;
    put_i32(&mut up, 0, 3);

    assert_eq!(
        decode(&raw),
        XEvent::Key {
            down: true,
            keycode: 38,
            state: 0x14
        }
    );
    assert_eq!(
        decode(&up),
        XEvent::Key {
            down: false,
            keycode: 38,
            state: 0x14
        }
    );
}

#[test]
fn event_decode_button_and_motion_read_position() {
    let mut button = event_of(4);
    put_i32(&mut button, 64, 12);
    put_i32(&mut button, 68, -3);
    put_u32(&mut button, 84, 5);
    let mut release = button;
    put_i32(&mut release, 0, 5);
    let mut motion = event_of(6);
    put_i32(&mut motion, 64, 7);
    put_i32(&mut motion, 68, 9);

    assert_eq!(
        decode(&button),
        XEvent::Button {
            down: true,
            button: 5,
            x: 12,
            y: -3
        }
    );
    assert_eq!(
        decode(&release),
        XEvent::Button {
            down: false,
            button: 5,
            x: 12,
            y: -3
        }
    );
    assert_eq!(decode(&motion), XEvent::Motion { x: 7, y: 9 });
}

#[test]
fn event_decode_focus_in_and_out() {
    assert_eq!(decode(&event_of(9)), XEvent::Focus(true));
    assert_eq!(decode(&event_of(10)), XEvent::Focus(false));
}

#[test]
fn event_decode_generic_reads_extension_and_evtype() {
    let mut raw = event_of(35);
    put_i32(&mut raw, 32, 131);
    put_i32(&mut raw, 36, 17);

    assert_eq!(
        decode(&raw),
        XEvent::Generic {
            extension: 131,
            evtype: 17
        }
    );
}

// ---- pen ----

const AXIS: PressureAxis = PressureAxis {
    number: 2,
    min: 0.0,
    max: 2000.0,
};

#[test]
fn pen_valuator_reads_packed_values_by_mask() {
    // Valuators 0, 2 and 5 are set: values are packed in that order.
    let mask = [0b0010_0101u8];
    let values = [10.0, 20.0, 30.0];

    assert_eq!(valuator(&mask, &values, 0), Some(10.0));
    assert_eq!(valuator(&mask, &values, 2), Some(20.0));
    assert_eq!(valuator(&mask, &values, 5), Some(30.0));
}

#[test]
fn pen_valuator_unset_bit_is_none() {
    let mask = [0b0000_0001u8];
    let values = [10.0];

    assert_eq!(valuator(&mask, &values, 1), None);
    assert_eq!(valuator(&mask, &values, 99), None);
    assert_eq!(valuator(&mask, &values, -1), None);
    // A mask that promises more values than there are.
    assert_eq!(valuator(&[0b11], &[1.0], 1), None);
}

#[test]
fn pen_normalise_clamps_to_unit_range() {
    assert!((normalise(1000.0, AXIS) - 0.5).abs() < 1e-6);
    assert!((normalise(-5.0, AXIS)).abs() < 1e-6);
    assert!((normalise(9000.0, AXIS) - 1.0).abs() < 1e-6);
}

#[test]
fn pen_normalise_degenerate_range_is_zero() {
    let flat = PressureAxis {
        number: 0,
        min: 5.0,
        max: 5.0,
    };

    assert!(normalise(5.0, flat).abs() < 1e-6);
    assert!(normalise(f64::NAN, AXIS).abs() < 1e-6);
}

#[test]
fn pen_tracker_keeps_last_pressure_when_valuator_missing() {
    let mut pen = PenTracker::default();
    pen.raw_event(7, Some(AXIS), &[0b100], &[1000.0]);

    // A raw event that carries only x and y.
    pen.raw_event(7, Some(AXIS), &[0b011], &[1.0, 2.0]);

    assert!((pen.pressure().unwrap() - 0.5).abs() < 1e-6);
}

#[test]
fn pen_tracker_device_without_axis_clears_pressure() {
    let mut pen = PenTracker::default();
    pen.raw_event(7, Some(AXIS), &[0b100], &[1000.0]);

    pen.raw_event(3, None, &[0b011], &[1.0, 2.0]);

    assert_eq!(pen.pressure(), None);
}

#[test]
fn pen_tracker_new_pen_starts_from_zero() {
    let mut pen = PenTracker::default();
    pen.raw_event(7, Some(AXIS), &[0b100], &[1000.0]);

    // Another pen sends motion without pressure.
    pen.raw_event(8, Some(AXIS), &[0b011], &[1.0, 2.0]);

    assert!(pen.pressure().unwrap().abs() < 1e-6);
}

#[test]
fn pen_sample_is_none_without_pressure() {
    let pen = PenTracker::default();

    assert_eq!(pen.sample(1.0, 2.0), None);
}

#[test]
fn regression_t35_pressure_sample_precedes_press_and_release() {
    // Recorded: a raw event with pressure 1500 of 2000, then press, motion
    // and release of the tip. Every core event is preceded by a sample.
    let mut pen = PenTracker::default();
    pen.raw_event(7, Some(AXIS), &[0b100], &[1500.0]);

    let sample = pen.sample(10.0, 20.0).unwrap();

    assert!(matches!(
        sample,
        RawEvent::Tablet { x, y, pressure }
            if (x - 10.0).abs() < 1e-6 && (y - 20.0).abs() < 1e-6 && (pressure - 0.75).abs() < 1e-6
    ));
    // The tip lifts: pressure 0 is still a sample, not "no pen".
    pen.raw_event(7, Some(AXIS), &[0b100], &[0.0]);
    assert!(matches!(
        pen.sample(10.0, 20.0),
        Some(RawEvent::Tablet { pressure, .. }) if pressure.abs() < 1e-6
    ));
}

// ---- fuzz ----

#[test]
fn fuzz_event_decode_random_bytes_never_panics() {
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let kinds = [2, 3, 4, 5, 6, 9, 10, 22, 33, 35, 0, -1, i32::MAX];
    for round in 0..20_000usize {
        let len = (rng.next() % 200) as usize;
        let mut bytes: Vec<u8> = (0..len).map(|_| rng.next() as u8).collect();
        if round % 2 == 0 && bytes.len() >= 4 {
            let kind = kinds[(rng.next() % kinds.len() as u64) as usize];
            bytes[0..4].copy_from_slice(&kind.to_ne_bytes());
        }
        let _ = decode(&bytes);
    }
}

#[test]
fn fuzz_keys_random_keysyms_never_panic() {
    let mut rng = Rng(0x1234_5678_9abc_def1);
    let mut tracker = KeyTracker::default();
    for _ in 0..50_000 {
        let sym = rng.next() as u32;
        let code = rng.next() as u32;
        let state = rng.next() as u32;
        let _ = key_from_keysym(sym, code);
        let _ = char_from_keysym(sym);
        let _ = mods_from_state(state);
        let _ = level_for(state, state.is_multiple_of(2));
        let _ = group_of(state);
        let _ = button_from_x(code);
        let _ = wheel_delta(code);
        if state.is_multiple_of(3) {
            tracker.release(code % 300);
        } else {
            let _ = tracker.press(code % 300);
        }
    }
}

#[test]
fn fuzz_pen_valuator_random_masks_never_panic() {
    let mut rng = Rng(0xdead_beef_cafe_f00d);
    let mut pen = PenTracker::default();
    for _ in 0..20_000 {
        let mask_len = (rng.next() % 6) as usize;
        let mask: Vec<u8> = (0..mask_len).map(|_| rng.next() as u8).collect();
        let value_count = (rng.next() % 12) as usize;
        let values: Vec<f64> = (0..value_count)
            .map(|_| f64::from_bits(rng.next()))
            .collect();
        let number = (rng.next() % 70) as i32 - 3;
        let _ = valuator(&mask, &values, number);
        let axis = (!rng.next().is_multiple_of(4)).then_some(PressureAxis {
            number,
            min: f64::from_bits(rng.next()),
            max: f64::from_bits(rng.next()),
        });
        pen.raw_event((rng.next() % 4) as i32, axis, &mask, &values);
        if let Some(p) = pen.pressure() {
            assert!((0.0..=1.0).contains(&p));
        }
    }
}
