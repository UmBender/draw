//! The X11 event decoder and the DPI rules, without a server (T175 AC-2,
//! AC-4). The fuzz uses a seeded xorshift generator: `proptest` is not a
//! dev-dependency of this crate.

#![allow(clippy::unwrap_used)]

use draw_sys::x11::dpi::{resolve_dpi, scale_from_dpi, screen_dpi, window_metrics, xft_dpi};
use draw_sys::x11::event::{CloseAtoms, EVENT_BYTES, XEvent, decode, is_close};

const CONFIGURE_NOTIFY: i32 = 22;
const CLIENT_MESSAGE: i32 = 33;

fn event_of(kind: i32) -> [u8; EVENT_BYTES] {
    let mut raw = [0u8; EVENT_BYTES];
    raw[0..4].copy_from_slice(&kind.to_ne_bytes());
    raw
}

fn configure(width: i32, height: i32) -> [u8; EVENT_BYTES] {
    let mut raw = event_of(CONFIGURE_NOTIFY);
    raw[56..60].copy_from_slice(&width.to_ne_bytes());
    raw[60..64].copy_from_slice(&height.to_ne_bytes());
    raw
}

fn message(message_type: u64, first: u64) -> [u8; EVENT_BYTES] {
    let mut raw = event_of(CLIENT_MESSAGE);
    raw[40..48].copy_from_slice(&message_type.to_ne_bytes());
    raw[48..52].copy_from_slice(&32i32.to_ne_bytes());
    raw[56..64].copy_from_slice(&first.to_ne_bytes());
    raw
}

struct Xorshift(u64);

impl Xorshift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

#[test]
fn decode_configure_notify_reads_the_size() {
    let raw = configure(640, 480);

    let event = decode(&raw);

    assert_eq!(
        event,
        XEvent::Configure {
            width: 640,
            height: 480
        }
    );
}

#[test]
fn decode_client_message_reads_type_and_first_long() {
    let raw = message(77, 99);

    let event = decode(&raw);

    assert_eq!(
        event,
        XEvent::ClientMessage {
            message_type: 77,
            first_long: 99
        }
    );
}

#[test]
fn decode_short_or_unknown_events_are_other() {
    assert_eq!(decode(&[]), XEvent::Other(0));
    assert_eq!(decode(&[1, 0]), XEvent::Other(0));
    assert_eq!(decode(&event_of(2)), XEvent::Other(2));
}

#[test]
fn is_close_needs_protocols_and_delete_atoms() {
    let atoms = CloseAtoms {
        protocols: 10,
        delete_window: 20,
    };

    assert!(is_close(decode(&message(10, 20)), atoms));
    assert!(!is_close(decode(&message(11, 20)), atoms));
    assert!(!is_close(decode(&message(10, 21)), atoms));
    assert!(!is_close(decode(&configure(10, 20)), atoms));
}

#[test]
fn decode_random_events_never_panics() {
    let mut rng = Xorshift(0x9E37_79B9_7F4A_7C15);
    for _ in 0..50_000 {
        let len = (rng.next() % 260) as usize;
        let bytes: Vec<u8> = (0..len).map(|_| rng.next().to_le_bytes()[0]).collect();

        let _ = decode(&bytes);
    }
}

#[test]
fn decode_random_configure_events_keep_the_fields() {
    let mut rng = Xorshift(0xD1B5_4A32_D192_ED03);
    for _ in 0..20_000 {
        let width = (rng.next() % 100_000) as i32;
        let height = (rng.next() % 100_000) as i32;
        let mut raw = configure(width, height);
        for byte in &mut raw[64..] {
            *byte = rng.next().to_le_bytes()[0];
        }

        let event = decode(&raw);

        assert_eq!(
            event,
            XEvent::Configure {
                width: width as u32,
                height: height as u32
            }
        );
    }
}

#[test]
fn decode_negative_sizes_are_zero() {
    let event = decode(&configure(-5, -1));

    assert_eq!(
        event,
        XEvent::Configure {
            width: 0,
            height: 0
        }
    );
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

#[test]
fn dpi_from_xft_resource_reads_the_value() {
    let text = "Xft.antialias:\t1\nXft.dpi:\t144\nXft.hinting:\t1\n";

    assert_eq!(xft_dpi(text), Some(144.0));
    assert_eq!(xft_dpi("Xft.dpi: 96.5"), Some(96.5));
    assert_eq!(xft_dpi("Xft.dpi:"), None);
    assert_eq!(xft_dpi("Xft.dpi: abc"), None);
    assert_eq!(xft_dpi("Xft.dpi: -3"), None);
    assert_eq!(xft_dpi("other.dpi: 100"), None);
}

#[test]
fn dpi_without_resource_falls_back_to_screen_size() {
    assert!(near(resolve_dpi(None, 1920, 508), 96.0));
    assert!(near(resolve_dpi(None, 3840, 508), 192.0));
    assert!(near(
        resolve_dpi(Some("Xft.antialias: 1"), 3840, 508),
        192.0
    ));
    assert!(near(resolve_dpi(Some("Xft.dpi: 120"), 3840, 508), 120.0));
    assert!(near(resolve_dpi(None, 100, 0), 96.0));
    assert_eq!(screen_dpi(0, 100), None);
}

#[test]
fn dpi_scale_is_clamped_and_finite() {
    assert!(near(scale_from_dpi(96.0), 1.0));
    assert!(near(scale_from_dpi(192.0), 2.0));
    assert!(near(scale_from_dpi(48.0), 1.0));
    assert!(near(scale_from_dpi(9999.0), 4.0));
    assert!(near(scale_from_dpi(f32::NAN), 1.0));
    assert!(near(scale_from_dpi(f32::INFINITY), 1.0));
}

#[test]
fn metrics_divide_physical_size_by_scale() {
    let m = window_metrics(800, 600, 2.0);

    assert!(near(m.width, 400.0));
    assert!(near(m.height, 300.0));
    assert!(near(m.scale, 2.0));
}
