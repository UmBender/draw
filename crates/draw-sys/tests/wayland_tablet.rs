//! The Wayland pen (T170): the pure tool state machine on recorded event
//! sequences (a headless sway has no virtual tablet), and the tablet manager
//! against a private sway.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::wildcard_enum_match_arm,
    non_snake_case
)]

#[path = "common/sway.rs"]
mod sway;

use draw_sys::platform::{RawButton, RawEvent};
use draw_sys::wayland::WaylandPlatform;
use draw_sys::wayland::pen::{
    PRESSURE_MAX, PenTool, Pens, ToolKind, pen_button_from_evdev, pressure_from_wire, tool_kind,
};
use sway::Sway;

const BTN_STYLUS: u32 = 0x14b;
const BTN_STYLUS2: u32 = 0x14c;

/// `wl_fixed_t` of a whole number.
fn fixed(v: i32) -> i32 {
    v * 256
}

/// A tool in proximity of our surface with a pressure of `p`.
fn near(p: u32) -> PenTool {
    let mut tool = PenTool::new();
    tool.proximity_in(true);
    tool.pressure(p);
    tool
}

#[test]
fn pen_frame_motion_sends_sample_then_move() {
    let mut tool = near(32768);
    tool.motion(fixed(10), fixed(20), 2);

    let events = tool.frame();

    assert_eq!(events.len(), 2);
    let RawEvent::Tablet { x, y, pressure } = events[0] else {
        panic!("sample first: {events:?}");
    };
    assert!((x - 20.0).abs() < 1e-3 && (y - 40.0).abs() < 1e-3);
    assert!((pressure - 0.5).abs() < 1e-3);
    assert_eq!(events[1], RawEvent::PointerMove { x: 20.0, y: 40.0 });
}

#[test]
fn pen_frame_tip_change_sends_left_button_once() {
    let mut tool = near(1000);
    tool.motion(fixed(1), fixed(2), 1);
    tool.down();
    tool.down();
    let first = tool.frame();
    tool.down();
    let repeated = tool.frame();
    tool.up();
    let released = tool.frame();

    let downs = |events: &[RawEvent], want: bool| {
        events
            .iter()
            .filter(|e| matches!(e, RawEvent::Button { button: RawButton::Left, down, .. } if *down == want))
            .count()
    };
    assert_eq!(downs(&first, true), 1);
    assert_eq!(downs(&repeated, true), 0);
    assert_eq!(downs(&released, false), 1);
    assert!(!tool.tip_down());
}

#[test]
fn pen_frame_out_of_proximity_drops_events() {
    let mut tool = PenTool::new();
    tool.motion(fixed(5), fixed(5), 1);
    tool.down();

    let events = tool.frame();

    assert!(events.is_empty());
    assert!(!tool.tip_down());
}

#[test]
fn pen_frame_for_another_surface_stays_out_of_proximity() {
    let mut tool = PenTool::new();
    tool.proximity_in(false);
    tool.motion(fixed(5), fixed(5), 1);

    assert!(tool.frame().is_empty());
    assert!(!tool.in_proximity());
}

#[test]
fn pen_proximity_out_lifts_a_tip_that_is_down() {
    let mut tool = near(500);
    tool.motion(fixed(3), fixed(4), 1);
    tool.down();
    tool.frame();
    tool.proximity_out();

    let events = tool.frame();

    assert!(events.iter().any(|e| matches!(
        e,
        RawEvent::Button { button: RawButton::Left, down: false, x, y } if (*x - 3.0).abs() < 1e-3 && (*y - 4.0).abs() < 1e-3
    )));
    assert!(!tool.in_proximity());
    assert!(!tool.tip_down());
}

#[test]
fn pen_pressure_is_normalised_and_clamped() {
    assert!(pressure_from_wire(0).abs() < f32::EPSILON);
    assert!((pressure_from_wire(65535) - 1.0).abs() < 1e-6);
    assert!((pressure_from_wire(u32::MAX) - 1.0).abs() < f32::EPSILON);
    assert!((PRESSURE_MAX - 65535.0).abs() < f32::EPSILON);
}

#[test]
fn pen_barrel_buttons_map_to_middle_and_right() {
    assert_eq!(pen_button_from_evdev(BTN_STYLUS), Some(RawButton::Middle));
    assert_eq!(pen_button_from_evdev(BTN_STYLUS2), Some(RawButton::Right));
    assert_eq!(pen_button_from_evdev(0x110), None);
}

#[test]
fn pen_tilt_is_kept_and_finite() {
    let mut tool = near(0);

    tool.tilt(fixed(30), fixed(-45));

    let (tx, ty) = tool.tilt_degrees();
    assert!((tx - 30.0).abs() < 1e-3 && (ty + 45.0).abs() < 1e-3);
    assert!(tx.is_finite() && ty.is_finite());
}

#[test]
fn pen_tool_kind_reads_the_wire_type() {
    assert_eq!(tool_kind(0x140), ToolKind::Pen);
    assert_eq!(tool_kind(0x141), ToolKind::Eraser);
    assert_eq!(tool_kind(0x146), ToolKind::Other);
    let mut tool = PenTool::new();
    tool.set_kind(0x141);
    assert_eq!(tool.kind(), ToolKind::Eraser);
}

#[test]
fn pens_shadow_the_pointer_while_a_tool_is_in_proximity() {
    let mut pens = Pens::default();
    pens.add(1);
    pens.add(2);
    assert!(!pens.shadowed());

    pens.get_mut(2).unwrap().proximity_in(true);
    assert!(pens.shadowed());

    pens.get_mut(2).unwrap().proximity_out();
    pens.get_mut(2).unwrap().frame();
    assert!(!pens.shadowed());
    pens.get_mut(1).unwrap().proximity_in(true);
    pens.remove(1);
    assert!(!pens.shadowed());
    assert_eq!(pens.len(), 1);
}

// ---- regressions of the vendored patches ----

#[test]
fn regression_T35_pressure_reaches_the_sample() {
    let mut tool = near(65535);
    tool.motion(fixed(1), fixed(1), 1);

    let events = tool.frame();

    assert!(matches!(events[0], RawEvent::Tablet { pressure, .. } if (pressure - 1.0).abs() < 1e-6));
}

#[test]
fn regression_T40_pointer_shadowed_in_proximity() {
    let mut pens = Pens::default();
    pens.add(7);
    pens.get_mut(7).unwrap().proximity_in(true);

    assert!(pens.shadowed());
}

#[test]
fn regression_pen_barrel_takeover() {
    let mut tool = near(0);
    tool.motion(fixed(2), fixed(2), 1);
    tool.button(BTN_STYLUS, 1);
    tool.button(BTN_STYLUS, 0);
    tool.button(0x110, 1);

    let events = tool.frame();

    let buttons: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            RawEvent::Button { button, down, .. } => Some((*button, *down)),
            _ => None,
        })
        .collect();
    assert_eq!(
        buttons,
        vec![(RawButton::Middle, true), (RawButton::Middle, false)]
    );
}

#[test]
fn regression_pen_tip_bounce() {
    let mut tool = near(800);
    tool.motion(fixed(2), fixed(2), 1);
    tool.down();
    tool.frame();

    // The tip bounces: up and down again inside one frame, state unchanged.
    tool.up();
    tool.down();
    let events = tool.frame();

    assert!(
        !events
            .iter()
            .any(|e| matches!(e, RawEvent::Button { button: RawButton::Left, .. }))
    );
    assert!(tool.tip_down());
}

// ---- fuzz (AC-3) ----

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
fn fuzz_pen_decoders_never_panic() {
    let mut rng = XorShift(0x1234_5678_9abc_def1);
    let mut tool = PenTool::new();
    for _ in 0..30_000 {
        let a = rng.next();
        let n = (a >> 32) as u32;
        let m = a as u32;
        match a % 9 {
            0 => tool.proximity_in(a & 16 != 0),
            1 => tool.proximity_out(),
            2 => tool.down(),
            3 => tool.up(),
            4 => tool.motion(m as i32, n as i32, (a >> 20) as i32),
            5 => tool.pressure(m),
            6 => tool.tilt(m as i32, n as i32),
            7 => tool.button(m, n % 3),
            _ => {
                for e in tool.frame() {
                    if let RawEvent::Tablet { x, y, pressure } = e {
                        assert!(x.is_finite() && y.is_finite());
                        assert!((0.0..=1.0).contains(&pressure));
                    }
                }
            }
        }
        let _ = (pressure_from_wire(m), tool_kind(n), pen_button_from_evdev(m));
    }
}

// ---- against sway ----

#[test]
fn wayland_tablet_manager_is_bound_without_tablets() {
    let sway = Sway::start();

    let mut platform = WaylandPlatform::connect(Some(&sway.socket), "T170", "draw-test").unwrap();
    let mut events = Vec::new();
    for _ in 0..10 {
        draw_sys::platform::Platform::poll(&mut platform, &mut events);
    }

    assert!(
        events
            .iter()
            .all(|e| !matches!(e, RawEvent::Tablet { .. })),
        "no tablet exists: {events:?}"
    );
    assert!(platform.has_tablet_manager());
}
