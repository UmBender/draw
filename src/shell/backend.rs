//! Window backend chosen at start-up from `DRAW_BACKEND` (T28,
//! ADR-T28-1).
//!
//! [`choose`] is a pure function over the variable's value; [`from_env`]
//! feeds it the real one. X11 (on Wayland sessions through Xwayland) is
//! the default; the native Wayland backend is experimental and falls back
//! to X11 when it cannot connect.

use std::ffi::OsStr;

use macroquad::miniquad::conf::LinuxBackend;

/// Environment variable naming the window backend.
pub const BACKEND_ENV: &str = "DRAW_BACKEND";

/// The window backend draw asks miniquad for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Backend {
    /// X11 only, through Xwayland on Wayland sessions (the default).
    #[default]
    X11,
    /// Native Wayland, falling back to X11 (experimental).
    Wayland,
}

/// The backend picked from [`BACKEND_ENV`] and what to tell the user.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Choice {
    /// The backend to use.
    pub backend: Backend,
    /// One stderr line when the value was not understood.
    pub warning: Option<String>,
}

impl Backend {
    /// The miniquad setting for this backend.
    #[must_use]
    pub fn linux_backend(self) -> LinuxBackend {
        match self {
            Self::X11 => LinuxBackend::X11Only,
            Self::Wayland => LinuxBackend::WaylandWithX11Fallback,
        }
    }

    /// One stderr line to print at start-up, if this backend needs one.
    #[must_use]
    pub fn notice(self) -> Option<&'static str> {
        match self {
            Self::X11 => None,
            Self::Wayland => Some(
                "draw: DRAW_BACKEND=wayland is experimental; \
                 unset it to go back to X11 if something misbehaves",
            ),
        }
    }
}

/// Picks the backend from the value of [`BACKEND_ENV`] (`None` when
/// unset).
#[must_use]
pub fn choose(value: Option<&OsStr>) -> Choice {
    let Some(value) = value else {
        return Choice::default();
    };
    let text = value.to_string_lossy();
    let text = text.trim();
    if text.is_empty() || text.eq_ignore_ascii_case("x11") {
        Choice::default()
    } else if text.eq_ignore_ascii_case("wayland") {
        Choice {
            backend: Backend::Wayland,
            warning: None,
        }
    } else {
        Choice {
            backend: Backend::X11,
            warning: Some(format!(
                "draw: unknown {BACKEND_ENV} value {text:?}, using x11 \
                 (expected x11 or wayland)"
            )),
        }
    }
}

/// [`choose`] applied to the real environment.
#[must_use]
pub fn from_env() -> Choice {
    choose(std::env::var_os(BACKEND_ENV).as_deref())
}

/// Wheel units a native Wayland compositor reports per wheel notch.
///
/// miniquad passes `wl_pointer.axis` values through unchanged; libinput
/// compositors (`wlroots`, Hyprland, `KWin`) send 15 per notch and Mutter 10,
/// where X11 sends 1. Dividing by 15 makes one notch one zoom step.
pub const WAYLAND_WHEEL_UNITS_PER_NOTCH: f32 = 15.0;

/// Wheel units per notch for `backend` in a session whose
/// `WAYLAND_DISPLAY` is `wayland_display`: [`WAYLAND_WHEEL_UNITS_PER_NOTCH`]
/// on native Wayland, `1` on X11 (including the fallback without a
/// Wayland session).
#[must_use]
pub fn wheel_units_per_notch(backend: Backend, wayland_display: Option<&OsStr>) -> f32 {
    let wayland_session = wayland_display.is_some_and(|value| !value.is_empty());
    if backend == Backend::Wayland && wayland_session {
        WAYLAND_WHEEL_UNITS_PER_NOTCH
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::ffi::OsStr;

    fn choose_str(value: &str) -> Choice {
        choose(Some(OsStr::new(value)))
    }

    #[test]
    fn choose_unset_or_empty_is_x11_without_warning() {
        // Arrange / Act
        let choices = [choose(None), choose_str(""), choose_str("  ")];

        // Assert
        for choice in choices {
            assert_eq!(choice.backend, Backend::X11);
            assert_eq!(choice.warning, None);
        }
    }

    #[test]
    fn choose_x11_is_x11_without_warning() {
        // Arrange / Act
        let choice = choose_str("x11");

        // Assert
        assert_eq!(choice.backend, Backend::X11);
        assert_eq!(choice.warning, None);
    }

    #[test]
    fn choose_wayland_is_wayland_without_warning() {
        // Arrange / Act
        let choice = choose_str("wayland");

        // Assert
        assert_eq!(choice.backend, Backend::Wayland);
        assert_eq!(choice.warning, None);
    }

    #[test]
    fn choose_ignores_case_and_surrounding_space() {
        // Arrange
        let cases = [
            (" Wayland ", Backend::Wayland),
            ("WAYLAND", Backend::Wayland),
            ("\tX11\n", Backend::X11),
        ];

        for (value, expected) in cases {
            // Act
            let choice = choose_str(value);

            // Assert
            assert_eq!(choice.backend, expected, "{value:?}");
            assert_eq!(choice.warning, None, "{value:?}");
        }
    }

    #[test]
    fn choose_unknown_value_warns_and_keeps_x11() {
        // Arrange
        let cases = ["wl", "xwayland", "way land", "1", "wayland,x11"];

        for value in cases {
            // Act
            let choice = choose_str(value);

            // Assert
            assert_eq!(choice.backend, Backend::X11, "{value:?}");
            let warning = choice.warning.unwrap_or_default();
            assert!(warning.contains(BACKEND_ENV), "{warning}");
            assert!(warning.contains(value.trim()), "{warning}");
        }
    }

    #[test]
    fn choose_non_utf8_value_warns_and_keeps_x11() {
        // Arrange
        #[cfg(unix)]
        let value = {
            use std::os::unix::ffi::OsStrExt;
            OsStr::from_bytes(b"way\xffland")
        };
        #[cfg(not(unix))]
        let value = OsStr::new("\u{fffd}");

        // Act
        let choice = choose(Some(value));

        // Assert
        assert_eq!(choice.backend, Backend::X11);
        assert!(choice.warning.is_some());
    }

    #[test]
    fn backend_linux_backend_maps_to_miniquad() {
        // Arrange / Act / Assert
        assert_eq!(Backend::X11.linux_backend(), LinuxBackend::X11Only);
        assert_eq!(
            Backend::Wayland.linux_backend(),
            LinuxBackend::WaylandWithX11Fallback
        );
        assert_eq!(Backend::default(), Backend::X11);
    }

    #[test]
    fn backend_notice_only_for_wayland() {
        // Arrange / Act
        let x11 = Backend::X11.notice();
        let wayland = Backend::Wayland.notice();

        // Assert
        assert_eq!(x11, None);
        let wayland = wayland.unwrap_or_default();
        assert!(wayland.contains("experimental"), "{wayland}");
        assert!(wayland.contains(BACKEND_ENV), "{wayland}");
    }

    #[test]
    fn warning_and_notice_are_one_line() {
        // Arrange
        let lines = [
            choose_str("bogus").warning.unwrap_or_default(),
            Backend::Wayland.notice().unwrap_or_default().to_owned(),
        ];

        for line in lines {
            // Assert
            assert!(line.starts_with("draw: "), "{line}");
            assert!(!line.contains('\n'), "{line}");
        }
    }

    #[test]
    fn readme_documents_draw_backend_as_experimental() {
        // Arrange
        let readme = include_str!("../../README.md");

        // Act
        let experimental = readme
            .split("### Experimental")
            .nth(1)
            .and_then(|rest| rest.split("\n## ").next())
            .unwrap_or_default();

        // Assert
        assert!(
            readme.contains(&format!("| `{BACKEND_ENV}` |")),
            "variable table"
        );
        assert!(
            experimental.contains(&format!("{BACKEND_ENV}=wayland")),
            "Experimental section"
        );
    }

    #[test]
    fn wheel_units_per_notch_is_one_on_x11() {
        // Arrange / Act
        let on_x11 = wheel_units_per_notch(Backend::X11, Some(OsStr::new("wayland-1")));
        let no_session = wheel_units_per_notch(Backend::Wayland, None);
        let empty = wheel_units_per_notch(Backend::Wayland, Some(OsStr::new("")));

        // Assert: X11 reports one unit per notch, and Wayland without a
        // session falls back to X11
        assert!((on_x11 - 1.0).abs() < f32::EPSILON);
        assert!((no_session - 1.0).abs() < f32::EPSILON);
        assert!((empty - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn wheel_units_per_notch_on_native_wayland() {
        // Arrange / Act
        let units = wheel_units_per_notch(Backend::Wayland, Some(OsStr::new("wayland-1")));

        // Assert
        assert!((units - WAYLAND_WHEEL_UNITS_PER_NOTCH).abs() < f32::EPSILON);
        assert!((WAYLAND_WHEEL_UNITS_PER_NOTCH - 15.0).abs() < f32::EPSILON);
    }

    proptest! {
        #[test]
        fn choose_any_value_warns_unless_known(value in ".*") {
            let choice = choose_str(&value);
            let known = matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "" | "x11" | "wayland"
            );
            prop_assert_eq!(choice.warning.is_none(), known);
            if choice.backend == Backend::Wayland {
                prop_assert!(known);
            }
        }
    }
}
