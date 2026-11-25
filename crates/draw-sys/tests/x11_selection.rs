//! The X11 selections under a private Xvfb with `xclip` as the other
//! client (T177 AC-1). A missing `Xvfb` or `xclip` fails the test.

#![allow(clippy::unwrap_used)]

#[path = "common/xvfb.rs"]
mod xvfb;

use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use draw_sys::platform::Platform;
use draw_sys::x11::{GlBackend, X11Platform, send_close_request};

/// A PNG-looking payload of `len` bytes that is not text.
fn blob(len: usize) -> Vec<u8> {
    let mut data = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    data.extend((0..len).map(|i| (i.wrapping_mul(31) % 251) as u8 | 0x80));
    data
}

fn open(server: &xvfb::Xvfb) -> X11Platform {
    X11Platform::open(
        Some(&server.display),
        "draw test",
        "draw-test",
        100,
        80,
        GlBackend::Glx,
    )
    .unwrap()
}

/// Runs `xclip -o args`, polling the platform so it can answer; the exit
/// success and the bytes written.
fn xclip_out(server: &xvfb::Xvfb, platform: &mut X11Platform, args: &[&str]) -> (bool, Vec<u8>) {
    let mut child = Command::new("xclip")
        .env("DISPLAY", &server.display)
        .arg("-o")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| panic!("xclip is required for the X11 tests: {e}"));
    let mut stdout = child.stdout.take().unwrap();
    let reader = thread::spawn(move || {
        let mut out = Vec::new();
        stdout.read_to_end(&mut out).unwrap();
        out
    });
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut events = Vec::new();
    let status = loop {
        events.clear();
        platform.poll(&mut events);
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        assert!(Instant::now() < deadline, "xclip -o did not finish");
        thread::sleep(Duration::from_millis(2));
    };
    (status.success(), reader.join().unwrap())
}

/// Makes `xclip -i` own `selection` with `data` as `target`, and waits
/// until it has taken the selection.
fn xclip_own(server: &xvfb::Xvfb, selection: &str, target: &str, data: &[u8]) {
    let mut child = Command::new("xclip")
        .env("DISPLAY", &server.display)
        .args(["-i", "-selection", selection, "-t", target])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| panic!("xclip is required for the X11 tests: {e}"));
    let mut stdin = child.stdin.take().unwrap();
    let data = data.to_vec();
    let writer = thread::spawn(move || {
        stdin.write_all(&data).unwrap();
    });
    writer.join().unwrap();
    assert!(child.wait().unwrap().success());
    // The parent exits right after the fork, but the forked owner sends its
    // `SetSelectionOwner` only when it first flushes: wait until the
    // selection answers, or a read can race it and get nothing (T194).
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let owned = Command::new("xclip")
            .env("DISPLAY", &server.display)
            .args(["-o", "-selection", selection, "-t", "TARGETS"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap_or_else(|e| panic!("xclip is required for the X11 tests: {e}"))
            .success();
        if owned {
            return;
        }
        assert!(Instant::now() < deadline, "xclip never owned {selection}");
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn x11_clipboard_text_is_served_to_xclip() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server);
    platform.set_clipboard_text("héllo clipboard");

    let (ok, out) = xclip_out(&server, &mut platform, &["-selection", "clipboard"]);

    assert!(ok);
    assert_eq!(String::from_utf8(out).unwrap(), "héllo clipboard");
}

#[test]
fn x11_primary_text_is_served_to_xclip() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server);
    platform.set_primary_text("primary text");

    let (ok, out) = xclip_out(&server, &mut platform, &["-selection", "primary"]);

    assert!(ok);
    assert_eq!(String::from_utf8(out).unwrap(), "primary text");
}

#[test]
fn x11_targets_list_text_targets() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server);
    platform.set_clipboard_text("x");

    let (ok, out) = xclip_out(
        &server,
        &mut platform,
        &["-selection", "clipboard", "-t", "TARGETS"],
    );

    assert!(ok);
    let list = String::from_utf8(out).unwrap();
    assert!(list.lines().any(|l| l == "UTF8_STRING"), "{list}");
    assert!(list.lines().any(|l| l == "TARGETS"), "{list}");
    assert!(!list.lines().any(|l| l == "image/png"), "{list}");
}

#[test]
fn x11_png_is_served_as_image_png() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server);
    let png = blob(5000);
    platform.set_clipboard_png(&png);

    let (ok, out) = xclip_out(
        &server,
        &mut platform,
        &["-selection", "clipboard", "-t", "image/png"],
    );

    assert!(ok);
    assert_eq!(out, png);
}

#[test]
fn x11_large_png_is_served_with_incr() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server);
    let png = blob(900_000);
    platform.set_clipboard_png(&png);

    let (ok, out) = xclip_out(
        &server,
        &mut platform,
        &["-selection", "clipboard", "-t", "image/png"],
    );

    assert!(ok);
    assert_eq!(out.len(), png.len());
    assert!(out == png, "the INCR transfer corrupted the image");
}

#[test]
fn x11_reads_text_set_by_xclip() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server);
    xclip_own(&server, "clipboard", "UTF8_STRING", "olá mundo".as_bytes());

    let text = platform.clipboard_text();

    assert_eq!(text.as_deref(), Some("olá mundo"));
}

#[test]
fn x11_reads_primary_set_by_xclip() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server);
    xclip_own(&server, "primary", "UTF8_STRING", b"selected");

    assert_eq!(platform.primary_text().as_deref(), Some("selected"));
}

#[test]
fn x11_reads_png_set_by_xclip() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server);
    let png = blob(3000);
    xclip_own(&server, "clipboard", "image/png", &png);

    assert_eq!(platform.clipboard_png(), Some(png));
}

#[test]
fn x11_reads_large_png_by_incr() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server);
    let png = blob(1_200_000);
    xclip_own(&server, "clipboard", "image/png", &png);

    let got = platform.clipboard_png();

    assert_eq!(got.as_ref().map(Vec::len), Some(png.len()));
    assert!(got == Some(png), "the INCR transfer corrupted the image");
}

#[test]
fn x11_empty_clipboard_reads_none() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server);

    assert_eq!(platform.clipboard_text(), None);
    assert_eq!(platform.clipboard_png(), None);
}

#[test]
fn x11_own_clipboard_reads_back_without_a_round_trip() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server);
    platform.set_clipboard_text("mine");

    assert_eq!(platform.clipboard_text().as_deref(), Some("mine"));
}

#[test]
fn x11_unknown_target_is_refused() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server);
    platform.set_clipboard_text("x");

    let (ok, _) = xclip_out(
        &server,
        &mut platform,
        &["-selection", "clipboard", "-t", "text/html"],
    );

    assert!(!ok);
}

#[test]
fn x11_events_during_a_read_are_not_lost() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server);
    xclip_own(&server, "clipboard", "UTF8_STRING", b"abc");
    send_close_request(Some(&server.display), platform.window_id()).unwrap();

    let text = platform.clipboard_text();
    let mut events = Vec::new();
    platform.poll(&mut events);

    assert_eq!(text.as_deref(), Some("abc"));
    assert!(platform.close_requested(), "the close request was dropped");
}

#[test]
fn regression_t115_png_is_not_offered_as_text() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server);
    platform.set_clipboard_png(&blob(100));

    let (ok, _) = xclip_out(
        &server,
        &mut platform,
        &["-selection", "clipboard", "-t", "UTF8_STRING"],
    );

    assert!(!ok);
}

#[test]
fn regression_t115_new_text_replaces_png() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server);
    platform.set_clipboard_png(&blob(100));
    platform.set_clipboard_text("later");

    let (ok, targets) = xclip_out(
        &server,
        &mut platform,
        &["-selection", "clipboard", "-t", "TARGETS"],
    );

    assert!(ok);
    assert!(!String::from_utf8(targets).unwrap().contains("image/png"));
}
