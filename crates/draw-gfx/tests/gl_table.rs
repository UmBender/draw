//! The GL function table (T145 AC-2): pure loading, no context needed.

#![allow(clippy::unwrap_used)]

use std::ffi::c_void;

use draw_gfx::gpu::gl::{Gl, GlError};

extern "C" fn stand_in() {}

/// A non-null address; the table is loaded but never called.
fn some_address() -> *const c_void {
    stand_in as extern "C" fn() as *const c_void
}

#[test]
fn all_entry_points_present_loads() {
    let gl = Gl::load_with(|_| Some(some_address()));

    assert!(gl.is_ok());
}

#[test]
fn missing_entry_point_is_reported() {
    let err = Gl::load_with(|name| (name != "glDrawElements").then(some_address)).err();

    assert_eq!(
        err,
        Some(GlError::MissingEntryPoints(vec![
            "glDrawElements".to_owned()
        ]))
    );
}

#[test]
fn every_missing_entry_point_is_listed() {
    let err = Gl::load_with(|_| None).err();

    let Some(GlError::MissingEntryPoints(names)) = err else {
        panic!("expected missing entry points, got {err:?}");
    };
    assert_eq!(names.len(), Gl::ENTRY_POINTS.len());
    assert!(names.iter().any(|n| n == "glReadPixels"));
}

#[test]
fn missing_entry_point_text_names_the_function() {
    let err = GlError::MissingEntryPoints(vec!["glClear".to_owned()]);

    assert!(err.to_string().contains("glClear"));
}
