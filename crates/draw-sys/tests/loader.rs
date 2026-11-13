//! The loader through its public API only, so the sanitizer jobs of the
//! nightly workflow (AC-3) exercise the real `dlopen` path.

#![allow(clippy::expect_used)]

use draw_sys::loader::{Library, LoadError};

const LIBM: &str = "libm.so.6";

#[test]
fn libm_cos_through_the_public_api() {
    let lib = Library::open(LIBM).expect("libm.so.6 is part of glibc");

    // SAFETY: `cos` in libm has the C type `double cos(double)`.
    let cos = unsafe { lib.symbol::<unsafe extern "C" fn(f64) -> f64>("cos") }.expect("cos");

    // SAFETY: calling `cos` with a finite double has no preconditions.
    let value = unsafe { cos(0.0) };
    assert!((value - 1.0).abs() < 1e-12);
}

#[test]
fn missing_library_is_an_error_value() {
    let err = Library::open("libdraw-does-not-exist.so.9").err();

    assert!(matches!(err, Some(LoadError::LibraryNotFound { .. })));
}
