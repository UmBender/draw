//! The X11 window, GLX and EGL contexts and `X11Platform` under a private
//! Xvfb (T175 AC-1 to AC-3). A missing `Xvfb` or `xdotool` fails the test.

#![allow(clippy::unwrap_used)]

#[path = "common/xvfb.rs"]
mod xvfb;

use std::future::Future;
use std::pin::pin;
use std::process::Command;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

use draw_sys::egl::GlContext;
use draw_sys::platform::{Platform, RawEvent};
use draw_sys::x11::{GlBackend, GlSurface, X11Error, X11Platform, send_close_request};

fn open(server: &xvfb::Xvfb, backend: GlBackend) -> X11Platform {
    X11Platform::open(
        Some(&server.display),
        "draw test",
        "draw-test",
        200,
        120,
        backend,
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

fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut cx) {
            return value;
        }
    }
}

/// Polls until `done` accepts an event or two seconds pass.
fn wait_for(platform: &mut X11Platform, done: impl Fn(&X11Platform, &RawEvent) -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut events = Vec::new();
    while Instant::now() < deadline {
        events.clear();
        platform.poll(&mut events);
        if events.iter().any(|e| done(platform, e)) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    false
}

/// The class of `WM_CLASS` (its second string) read with `xprop`:
/// `xdotool getwindowclassname` is missing from the xdotool of Ubuntu's
/// CI runners (T194).
fn wm_class(server: &xvfb::Xvfb, id: &str) -> String {
    let out = Command::new("xprop")
        .env("DISPLAY", &server.display)
        .args(["-id", id, "WM_CLASS"])
        .output()
        .unwrap_or_else(|e| panic!("xprop is required for the X11 tests: {e}"));
    let text = String::from_utf8_lossy(&out.stdout);
    // `WM_CLASS(STRING) = "instance", "class"`
    text.split('"').nth(3).unwrap_or_default().to_owned()
}

#[test]
fn x11_window_has_title_and_class() {
    let server = xvfb::Xvfb::start();
    let platform = open(&server, GlBackend::Glx);
    let id = platform.window_id().to_string();

    let title = xdotool(&server, &["getwindowname", &id]);
    let class = wm_class(&server, &id);

    assert_eq!(title, "draw test");
    assert_eq!(class, "draw-test");
}

#[test]
fn x11_glx_context_clears_and_swaps() {
    let server = xvfb::Xvfb::start();
    let platform = open(&server, GlBackend::Glx);
    let gl = platform.gl();
    assert_eq!(gl.make_current(), Ok(()));
    let clear_color = gl.proc_address("glClearColor").unwrap();
    let clear = gl.proc_address("glClear").unwrap();

    // SAFETY: both are core GL functions with these C prototypes and a
    // context is current on this thread.
    unsafe {
        let clear_color: unsafe extern "C" fn(f32, f32, f32, f32) =
            std::mem::transmute(clear_color);
        let clear: unsafe extern "C" fn(u32) = std::mem::transmute(clear);
        clear_color(0.2, 0.4, 0.6, 1.0);
        clear(0x4000);
    }

    assert_eq!(gl.swap_buffers(), Ok(()));
}

/// T195: GLX took the first RGBA8 config, single-buffered on Mesa, so
/// `glXSwapBuffers` showed nothing on a hardware GPU (black window).
#[test]
#[allow(non_snake_case)] // `regression_<ID>_<slug>` (Workflow)
fn regression_T195_glx_window_config_is_double_buffered() {
    let server = xvfb::Xvfb::start();
    let platform = open(&server, GlBackend::Glx);

    let GlSurface::Glx(context) = platform.gl() else {
        panic!("asked for GLX");
    };

    assert_eq!(context.is_double_buffered(), Ok(true));
}

#[test]
fn x11_egl_fallback_swaps_on_the_window() {
    let server = xvfb::Xvfb::start();
    let platform = open(&server, GlBackend::Egl);
    let gl = platform.gl();

    assert_eq!(gl.make_current(), Ok(()));
    assert!(gl.proc_address("glClear").is_some());
    assert_eq!(gl.swap_buffers(), Ok(()));
}

#[test]
fn x11_missing_display_is_an_error_value() {
    let err = X11Platform::open(Some(":9999"), "t", "c", 10, 10, GlBackend::Glx).err();

    assert!(matches!(err, Some(X11Error::NoDisplay)), "{err:?}");
}

#[test]
fn x11_resize_by_xdotool_reaches_window_metrics() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server, GlBackend::Glx);
    let id = platform.window_id().to_string();

    xdotool(&server, &["windowsize", &id, "150", "90"]);
    let seen = wait_for(
        &mut platform,
        |_, e| matches!(e, RawEvent::Resize(m) if m.width > 149.0 && m.width < 151.0),
    );

    assert!(seen, "no Resize event");
    let metrics = platform.window();
    assert!((metrics.width - 150.0).abs() < 1.0);
    assert!((metrics.height - 90.0).abs() < 1.0);
}

#[test]
fn x11_wm_close_sets_close_requested() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server, GlBackend::Glx);
    assert!(!platform.close_requested());

    // `xdotool windowquit` needs a window manager, and none runs here.
    send_close_request(Some(&server.display), platform.window_id()).unwrap();
    let mut events = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(2);
    while !platform.close_requested() && Instant::now() < deadline {
        platform.poll(&mut events);
        std::thread::sleep(Duration::from_millis(10));
    }

    assert!(platform.close_requested(), "no WM_DELETE_WINDOW");
}

#[test]
fn x11_platform_poll_reports_the_resize() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server, GlBackend::Glx);
    let id = platform.window_id().to_string();
    let before = platform.window();
    assert!((before.width - 200.0).abs() < 1.0, "{before:?}");

    xdotool(&server, &["windowsize", &id, "120", "100"]);
    let seen = wait_for(&mut platform, |_, e| matches!(e, RawEvent::Resize(_)));

    assert!(seen);
    assert!((platform.window().height - 100.0).abs() < 1.0);
}

#[test]
fn x11_platform_input_and_clipboard_return_nothing() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server, GlBackend::Glx);
    let mut events = Vec::new();

    platform.poll(&mut events);
    platform.set_clipboard_text("x");
    platform.set_clipboard_png(&[1, 2, 3]);

    assert_eq!(platform.clipboard_text(), None);
    assert!(events.iter().all(|e| matches!(
        e,
        RawEvent::Resize(_) | RawEvent::Focus(_) | RawEvent::Backend(_)
    )));
}

#[test]
fn x11_platform_schedule_update_ends_the_wait() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server, GlBackend::Glx);
    // The events of the map are not what this test waits for.
    std::thread::sleep(Duration::from_millis(50));
    platform.poll(&mut Vec::new());

    platform.schedule_update();
    let started = Instant::now();
    block_on(platform.next_frame());
    let scheduled = started.elapsed();
    let started = Instant::now();
    block_on(platform.next_frame());
    let idle = started.elapsed();

    assert!(scheduled < Duration::from_millis(8), "{scheduled:?}");
    assert!(idle >= Duration::from_millis(8), "{idle:?}");
}

#[test]
fn x11_idle_skips_swap_and_wakes_on_scheduled_update() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server, GlBackend::Glx);
    assert!(platform.skips_idle_swap());

    platform.schedule_update();
    let started = Instant::now();
    block_on(platform.next_idle());
    assert!(started.elapsed() < Duration::from_millis(20));
}

#[test]
fn x11_idle_sleeps_until_wakeup() {
    let server = xvfb::Xvfb::start();
    let mut platform = open(&server, GlBackend::Glx);
    // Drain initial map/expose/configure events first.
    std::thread::sleep(Duration::from_millis(50));
    platform.poll(&mut Vec::new());
    assert!(draw_sys::signal::install_quit());

    let started = Instant::now();
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(60));
        // SAFETY: `SIGTERM` (15) has our `install_quit` async-signal-safe handler installed.
        unsafe {
            libc_raise(15);
        }
    });
    block_on(platform.next_idle());
    let elapsed = started.elapsed();
    let _ = draw_sys::signal::take();
    assert!(
        elapsed >= Duration::from_millis(50) && elapsed < Duration::from_millis(200),
        "expected sleep until signal, got {elapsed:?}"
    );
}

unsafe extern "C" {
    #[link_name = "raise"]
    fn libc_raise(sig: i32) -> i32;
}
