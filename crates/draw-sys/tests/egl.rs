//! EGL contexts on a headless surface (T145 AC-1). These need a working EGL
//! (Mesa's llvmpipe is enough); without it they fail, they do not skip.

#![allow(clippy::unwrap_used)]

use draw_sys::egl::{ContextError, EglContext, GlContext};

#[test]
fn egl_headless_context_makes_current() {
    let ctx = EglContext::headless(16, 16).unwrap();

    assert_eq!(ctx.make_current(), Ok(()));
}

#[test]
fn egl_context_resolves_gl_entry_points() {
    let ctx = EglContext::headless(16, 16).unwrap();
    ctx.make_current().unwrap();

    assert!(ctx.proc_address("glClear").is_some());
    assert!(ctx.proc_address("glDrawElements").is_some());
    assert!(ctx.proc_address("glNoSuchFunction").is_none());
}

#[test]
fn egl_missing_library_is_an_error_value() {
    let err = EglContext::headless_with_library("libdraw-none.so.9", 16, 16).err();

    assert!(matches!(err, Some(ContextError::Load(_))), "{err:?}");
}

#[test]
fn egl_zero_size_pbuffer_is_an_error_value() {
    let err = EglContext::headless(0, 16).err();

    assert_eq!(err, Some(ContextError::BadSize));
}

#[test]
fn egl_proc_address_with_nul_is_none() {
    let ctx = EglContext::headless(16, 16).unwrap();

    assert!(ctx.proc_address("gl\0Clear").is_none());
}

#[test]
fn egl_error_text_names_the_failing_call() {
    let err = ContextError::Call {
        call: "eglInitialize",
        code: 0x3001,
    };

    assert!(err.to_string().contains("eglInitialize"));
    assert!(err.to_string().contains("0x3001"));
}
