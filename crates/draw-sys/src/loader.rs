//! Loading C libraries at run time with `dlopen` (ADR-T142-1).

#![allow(unsafe_code)]

use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::fmt;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::Deref;
use std::ptr::NonNull;

/// `RTLD_NOW` on Linux: resolve every symbol when the library is loaded.
const RTLD_NOW: c_int = 2;

// TEST: open_existing_library_succeeds
// SAFETY of the declarations: these are the glibc prototypes; each call
// site states why its arguments are valid.
unsafe extern "C" {
    fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> c_int;
    fn dlerror() -> *mut c_char;
}

/// Why a library or a symbol could not be loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    /// A name holds a NUL byte, so it cannot be a C string.
    InvalidName {
        /// The name as given.
        name: String,
    },
    /// `dlopen` failed: the library is missing or cannot be loaded.
    LibraryNotFound {
        /// The library name as given.
        name: String,
        /// The text of `dlerror`.
        reason: String,
    },
    /// `dlsym` found no such symbol in the library.
    SymbolNotFound {
        /// The symbol name as given.
        name: String,
        /// The text of `dlerror`.
        reason: String,
    },
    /// The requested function type does not have the size of a pointer.
    BadSymbolType {
        /// The symbol name as given.
        name: String,
        /// `size_of` the requested type.
        size: usize,
    },
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName { name } => write!(f, "name {name:?} contains a NUL byte"),
            Self::LibraryNotFound { name, reason } => {
                write!(f, "cannot load library {name}: {reason}")
            }
            Self::SymbolNotFound { name, reason } => {
                write!(f, "cannot find symbol {name}: {reason}")
            }
            Self::BadSymbolType { name, size } => write!(
                f,
                "symbol {name} cannot have a {size}-byte type, a function pointer has {} bytes",
                size_of::<*mut c_void>()
            ),
        }
    }
}

impl std::error::Error for LoadError {}

/// A loaded shared library, closed when dropped.
///
/// A [`Symbol`] borrows its library, so it cannot be used after the
/// library is dropped:
///
/// ```compile_fail
/// use draw_sys::loader::Library;
/// let lib = Library::open("libm.so.6").unwrap();
/// let cos = unsafe { lib.symbol::<unsafe extern "C" fn(f64) -> f64>("cos") }.unwrap();
/// drop(lib);
/// let _ = unsafe { cos(0.0) };
/// ```
#[derive(Debug)]
pub struct Library {
    handle: NonNull<c_void>,
}

/// A function of a [`Library`], typed when it was loaded.
#[derive(Debug, Clone, Copy)]
pub struct Symbol<'lib, F: Copy> {
    value: F,
    library: PhantomData<&'lib Library>,
}

impl Library {
    /// Loads the shared library `name` (for example `"libEGL.so.1"`).
    ///
    /// # Errors
    ///
    /// [`LoadError::InvalidName`] if `name` has a NUL byte, and
    /// [`LoadError::LibraryNotFound`] if `dlopen` fails.
    pub fn open(name: &str) -> Result<Self, LoadError> {
        let c_name = c_string(name)?;
        // TEST: open_existing_library_succeeds
        // SAFETY: `c_name` is a valid NUL-terminated string that lives
        // through the call; `RTLD_NOW` is a valid flag.
        let raw = unsafe { dlopen(c_name.as_ptr(), RTLD_NOW) };
        NonNull::new(raw)
            .map(|handle| Self { handle })
            .ok_or_else(|| LoadError::LibraryNotFound {
                name: name.to_owned(),
                reason: last_error(),
            })
    }

    /// Looks up the symbol `name` and gives it the type `F`.
    ///
    /// # Errors
    ///
    /// [`LoadError::InvalidName`] if `name` has a NUL byte,
    /// [`LoadError::BadSymbolType`] if `F` is not pointer sized, and
    /// [`LoadError::SymbolNotFound`] if the library has no such symbol (a
    /// symbol whose address is null counts as not found).
    ///
    /// # Safety
    ///
    /// `F` must be a function pointer type (`unsafe extern "C" fn(..)`)
    /// that matches the real signature of the symbol, and the symbol must
    /// be a function. Calling it through a wrong type is undefined behaviour.
    // TEST: symbol_present_can_be_called
    pub unsafe fn symbol<F: Copy>(&self, name: &str) -> Result<Symbol<'_, F>, LoadError> {
        let c_name = c_string(name)?;
        if size_of::<F>() != size_of::<*mut c_void>() {
            return Err(LoadError::BadSymbolType {
                name: name.to_owned(),
                size: size_of::<F>(),
            });
        }
        // TEST: symbol_present_can_be_called
        // SAFETY: the handle is live (`self` owns it) and `c_name` is a
        // valid NUL-terminated string that lives through the call.
        let raw = unsafe { dlsym(self.handle.as_ptr(), c_name.as_ptr()) };
        if raw.is_null() {
            return Err(LoadError::SymbolNotFound {
                name: name.to_owned(),
                reason: last_error(),
            });
        }
        // TEST: symbol_present_can_be_called
        // SAFETY: `F` has the size of a pointer (checked above), and the
        // caller promised in the contract of this function that `F` is a
        // function pointer type matching the symbol.
        let value = unsafe { std::mem::transmute_copy::<*mut c_void, F>(&raw) };
        Ok(Symbol {
            value,
            library: PhantomData,
        })
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        // TEST: open_existing_library_succeeds
        // SAFETY: the handle came from a successful `dlopen` and is closed
        // once, here. Symbols borrow the library, so none outlives it.
        // A failure to close is not actionable.
        let _ = unsafe { dlclose(self.handle.as_ptr()) };
    }
}

/// Converts a name to a C string, or reports the NUL byte.
fn c_string(name: &str) -> Result<CString, LoadError> {
    CString::new(name).map_err(|_| LoadError::InvalidName {
        name: name.to_owned(),
    })
}

/// The text of the last `dlerror` of this thread.
fn last_error() -> String {
    // TEST: symbol_missing_returns_symbol_not_found
    // SAFETY: `dlerror` returns null or a NUL-terminated string that stays
    // valid until the next `dl*` call of this thread; it is copied at once.
    unsafe {
        let text = dlerror();
        if text.is_null() {
            "unknown error".to_owned()
        } else {
            CStr::from_ptr(text).to_string_lossy().into_owned()
        }
    }
}

impl<F: Copy> Deref for Symbol<'_, F> {
    type Target = F;

    fn deref(&self) -> &F {
        &self.value
    }
}

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
