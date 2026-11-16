//! The Wayland clipboard (T170): the pure transfer decisions, and the whole
//! path against a private headless sway with `wl-copy` and `wl-paste`. A
//! missing tool fails the tests, it never skips them.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_possible_truncation,
    clippy::wildcard_enum_match_arm,
    non_snake_case
)]

#[path = "common/sway.rs"]
mod sway;

use std::io::{Cursor, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use draw_sys::platform::{Platform, RawEvent};
use draw_sys::wayland::WaylandPlatform;
use draw_sys::wayland::transfer::{
    Payload, has_png, pick_text_mime, read_limited, serves, source_mimes, write_payload,
};
use sway::Sway;

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn text_mime_prefers_utf8_over_legacy_names() {
    let offered = names(&[
        "STRING",
        "text/plain",
        "UTF8_STRING",
        "text/plain;charset=utf-8",
    ]);
    assert_eq!(pick_text_mime(&offered), Some("text/plain;charset=utf-8"));
    let legacy = names(&["TEXT", "STRING"]);
    assert_eq!(pick_text_mime(&legacy), Some("TEXT"));
}

#[test]
fn text_mime_none_without_a_text_type() {
    assert_eq!(pick_text_mime(&names(&["image/png", "text/html"])), None);
    assert_eq!(pick_text_mime(&[]), None);
}

#[test]
fn png_offer_is_detected() {
    assert!(has_png(&names(&["text/plain", "image/png"])));
    assert!(!has_png(&names(&["image/jpeg"])));
}

#[test]
fn source_mimes_follow_the_payload() {
    let text = Payload::Text("hi".into());
    let png = Payload::Png(vec![1, 2, 3]);
    assert!(source_mimes(&text).contains(&"text/plain;charset=utf-8"));
    assert_eq!(source_mimes(&png), &["image/png"]);
    assert!(serves(&text, "UTF8_STRING"));
    assert!(!serves(&text, "image/png"));
    assert!(serves(&png, "image/png"));
    assert!(!serves(&png, "text/plain"));
}

#[test]
fn read_limited_stops_at_the_limit() {
    let data = vec![7_u8; 1000];
    let got = read_limited(Cursor::new(&data), 100).unwrap();
    assert_eq!(got.len(), 100);
    let all = read_limited(Cursor::new(&data), 5000).unwrap();
    assert_eq!(all.len(), 1000);
}

struct Closed;

impl Write for Closed {
    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        Err(std::io::ErrorKind::BrokenPipe.into())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn write_payload_ignores_a_closed_reader() {
    assert!(write_payload(Closed, b"data").is_ok());
    let mut out = Vec::new();
    write_payload(&mut out, b"data").unwrap();
    assert_eq!(out, b"data");
}

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
fn fuzz_transfer_decoders_never_panic() {
    let mut rng = XorShift(0x0bad_cafe_dead_beef);
    for _ in 0..5_000 {
        let n = (rng.next() % 6) as usize;
        let offered: Vec<String> = (0..n)
            .map(|_| {
                let len = (rng.next() % 12) as usize;
                (0..len)
                    .map(|_| char::from(32 + (rng.next() % 90) as u8))
                    .collect()
            })
            .collect();
        let _ = pick_text_mime(&offered);
        let _ = has_png(&offered);
        let bytes: Vec<u8> = (0..rng.next() % 64).map(|_| rng.next() as u8).collect();
        let _ = read_limited(Cursor::new(&bytes), (rng.next() % 80) as usize);
        for m in &offered {
            let _ = serves(&Payload::Png(bytes.clone()), m);
            let _ = serves(&Payload::Text(m.clone()), m);
        }
    }
}

// ---- against sway ----

struct Focused {
    platform: WaylandPlatform,
    _keyboard: sway::Keyboard,
}

impl std::ops::Deref for Focused {
    type Target = WaylandPlatform;
    fn deref(&self) -> &WaylandPlatform {
        &self.platform
    }
}

impl std::ops::DerefMut for Focused {
    fn deref_mut(&mut self) -> &mut WaylandPlatform {
        &mut self.platform
    }
}

/// A platform with keyboard focus; the virtual keyboard that gives the seat
/// its capability lives as long as the returned value.
fn focused(sway: &Sway) -> Focused {
    let keyboard = sway.keyboard(&["-s", "20000", "a"]);
    let mut platform = WaylandPlatform::connect(Some(&sway.socket), "T170", "draw-test").unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut events = Vec::new();
    loop {
        platform.poll(&mut events);
        if events.iter().any(|e| matches!(e, RawEvent::Focus(true))) {
            return Focused {
                platform,
                _keyboard: keyboard,
            };
        }
        assert!(Instant::now() < deadline, "no keyboard focus: {events:?}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn tool(sway: &Sway, program: &str) -> Command {
    let mut cmd = Command::new(program);
    cmd.env("WAYLAND_DISPLAY", &sway.socket)
        .env_remove("DISPLAY")
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    cmd
}

/// Runs `wl-paste` while polling `platform`; returns its stdout.
fn paste(sway: &Sway, platform: &mut WaylandPlatform, args: &[&str]) -> Vec<u8> {
    let mut child = tool(sway, "wl-paste").args(args).spawn().expect("wl-paste");
    let mut stdout = child.stdout.take().unwrap();
    let reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut stdout, &mut buf).unwrap();
        buf
    });
    let deadline = Instant::now() + Duration::from_secs(15);
    let mut events = Vec::new();
    while !reader.is_finished() {
        platform.poll(&mut events);
        assert!(Instant::now() < deadline, "wl-paste did not finish");
        std::thread::sleep(Duration::from_millis(5));
    }
    let _ = child.wait();
    reader.join().unwrap()
}

fn copy(sway: &Sway, args: &[&str], data: &[u8]) {
    let mut child = tool(sway, "wl-copy")
        .args(args)
        .stdin(Stdio::piped())
        .spawn()
        .expect("wl-copy");
    child.stdin.take().unwrap().write_all(data).unwrap();
    child.wait().unwrap();
}

fn clipboard_until(platform: &mut WaylandPlatform, want: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut events = Vec::new();
    loop {
        platform.poll(&mut events);
        if let Some(text) = platform.clipboard_text().filter(|text| text == want) {
            return text;
        }
        assert!(Instant::now() < deadline, "clipboard never became the text");
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn wayland_clipboard_reads_text_from_wl_copy() {
    let sway = Sway::start();
    let mut platform = focused(&sway);

    copy(&sway, &[], "from wl-copy ✓".as_bytes());

    assert_eq!(
        clipboard_until(&mut platform, "from wl-copy ✓"),
        "from wl-copy ✓"
    );
}

#[test]
fn wayland_clipboard_text_reaches_wl_paste() {
    let sway = Sway::start();
    let mut platform = focused(&sway);

    platform.set_clipboard_text("from draw ✓");
    let got = paste(&sway, &mut platform, &["-n"]);

    assert_eq!(String::from_utf8(got).unwrap(), "from draw ✓");
    assert_eq!(platform.clipboard_text().as_deref(), Some("from draw ✓"));
}

#[test]
fn wayland_clipboard_png_reaches_wl_paste() {
    let sway = Sway::start();
    let mut platform = focused(&sway);
    let png: Vec<u8> = (0..300_u32).map(|i| (i % 251) as u8).collect();

    platform.set_clipboard_png(&png);
    let got = paste(&sway, &mut platform, &["--type", "image/png"]);

    assert_eq!(got, png);
}

#[test]
fn wayland_clipboard_large_text_round_trips() {
    let sway = Sway::start();
    let mut platform = focused(&sway);
    let big = "0123456\n".repeat(4 * 1024 * 1024 / 8);

    platform.set_clipboard_text(&big);
    let got = paste(&sway, &mut platform, &["-n"]);
    assert_eq!(got.len(), big.len());
    assert_eq!(got, big.as_bytes());

    copy(&sway, &[], big.as_bytes());
    assert_eq!(clipboard_until(&mut platform, &big).len(), big.len());
}

#[test]
fn wayland_clipboard_empty_is_none() {
    let sway = Sway::start();
    let mut platform = focused(&sway);

    assert_eq!(platform.clipboard_text(), None);
}
