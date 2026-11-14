//! GLX contexts under a private Xvfb (T145 AC-1). A missing `Xvfb` fails
//! the test.

#![allow(clippy::unwrap_used)]

#[path = "common/xvfb.rs"]
mod xvfb;

use draw_sys::egl::{ContextError, GlContext};
use draw_sys::glx::GlxContext;

#[test]
fn glx_pbuffer_context_makes_current_under_xvfb() {
    let server = xvfb::Xvfb::start();

    let ctx = GlxContext::pbuffer(Some(&server.display), 16, 16).unwrap();

    assert_eq!(ctx.make_current(), Ok(()));
    assert!(ctx.proc_address("glClear").is_some());
}

#[test]
fn glx_bad_display_is_an_error_value() {
    let err = GlxContext::pbuffer(Some(":9999"), 16, 16).err();

    assert_eq!(err, Some(ContextError::NoDisplay));
}

#[test]
fn glx_zero_size_pbuffer_is_an_error_value() {
    let err = GlxContext::pbuffer(Some(":9999"), 16, 0).err();

    assert_eq!(err, Some(ContextError::BadSize));
}
