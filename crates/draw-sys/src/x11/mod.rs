//! The X11 backend, window part (T175): an Xlib connection, a window, a GLX
//! context (EGL as the fallback) and [`X11Platform`]. Keyboard, pointer, pen
//! and clipboard come with T176 and T177.

use std::fmt;

use crate::egl::ContextError;
use crate::loader::LoadError;

pub mod dpi;
pub mod event;
mod platform;

pub use platform::X11Platform;

/// Which GL API creates the context of the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlBackend {
    /// GLX with `GLX_EXT_create_context_es2_profile`.
    Glx,
    /// EGL on X11, the fallback.
    Egl,
}

/// Why the X11 window could not be opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum X11Error {
    /// The X display cannot be opened.
    NoDisplay,
    /// A library or symbol is missing.
    Load(LoadError),
    /// The GL context failed.
    Context(ContextError),
    /// An Xlib call failed.
    Call(&'static str),
}

impl fmt::Display for X11Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDisplay => write!(f, "cannot open the X display"),
            Self::Load(e) => write!(f, "{e}"),
            Self::Context(e) => write!(f, "{e}"),
            Self::Call(call) => write!(f, "{call} failed"),
        }
    }
}

impl std::error::Error for X11Error {}

impl From<LoadError> for X11Error {
    fn from(e: LoadError) -> Self {
        Self::Load(e)
    }
}

impl From<ContextError> for X11Error {
    fn from(e: ContextError) -> Self {
        Self::Context(e)
    }
}
