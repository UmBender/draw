//! Loading C libraries at run time with `dlopen` (ADR-T142-1).

#![allow(unsafe_code)]

#[cfg(test)]
mod tests {
    use super::*;

    const LIBM: &str = "libm.so.6";

    #[test]
    fn open_missing_library_returns_not_found() {
        let err = Library::open("libdraw-does-not-exist.so.9").err();

        assert!(matches!(err, Some(LoadError::LibraryNotFound { .. })));
    }

    #[test]
    fn open_name_with_nul_returns_invalid_name() {
        let err = Library::open("lib\0m.so.6").err();

        assert!(matches!(err, Some(LoadError::InvalidName { .. })));
    }

    #[test]
    fn open_existing_library_succeeds() {
        assert!(Library::open(LIBM).is_ok());
    }

    #[test]
    fn symbol_missing_returns_symbol_not_found() {
        let lib = Library::open(LIBM).unwrap();

        // SAFETY: the symbol does not exist, so no function is ever called.
        let err = unsafe { lib.symbol::<unsafe extern "C" fn()>("draw_no_such_symbol") }.err();

        assert!(matches!(err, Some(LoadError::SymbolNotFound { .. })));
    }

    #[test]
    fn symbol_name_with_nul_returns_invalid_name() {
        let lib = Library::open(LIBM).unwrap();

        // SAFETY: the name is rejected before any lookup.
        let err = unsafe { lib.symbol::<unsafe extern "C" fn()>("co\0s") }.err();

        assert!(matches!(err, Some(LoadError::InvalidName { .. })));
    }

    #[test]
    fn symbol_of_wrong_size_returns_bad_type() {
        let lib = Library::open(LIBM).unwrap();

        // SAFETY: a `u8` is rejected by the size check before it is used.
        let err = unsafe { lib.symbol::<u8>("cos") }.err();

        assert!(matches!(err, Some(LoadError::BadSymbolType { .. })));
    }

    #[test]
    fn symbol_present_can_be_called() {
        let lib = Library::open(LIBM).unwrap();

        // SAFETY: `cos` in libm has the C type `double cos(double)`.
        let cos = unsafe { lib.symbol::<unsafe extern "C" fn(f64) -> f64>("cos") }.unwrap();

        // SAFETY: `cos` takes any double.
        let value = unsafe { cos(0.0) };
        assert!((value - 1.0).abs() < 1e-12);
    }

    #[test]
    fn error_messages_name_the_library_or_symbol() {
        let lib = Library::open(LIBM).unwrap();
        // SAFETY: the symbol does not exist, so no function is ever called.
        let sym = unsafe { lib.symbol::<unsafe extern "C" fn()>("draw_no_such_symbol") }
            .err()
            .unwrap();
        let open = Library::open("libdraw-does-not-exist.so.9").err().unwrap();

        assert!(sym.to_string().contains("draw_no_such_symbol"));
        assert!(open.to_string().contains("libdraw-does-not-exist.so.9"));
    }
}
