//! Window backend chosen at start-up from `--backend` and the session
//! (T28, ADR-T28-1, T46, ADR-T46-1, T154).
//!
//! [`choose`] is a pure function over the flag and `WAYLAND_DISPLAY`. Native Wayland
//! is the default on Wayland sessions and falls back to X11 when it
//! cannot connect; X11 (Xwayland on Wayland sessions) stays supported
//! (ADR-T128-1).

use std::ffi::OsStr;

use macroquad::miniquad::conf::LinuxBackend;

/// Environment variable set in a Wayland session.
pub const WAYLAND_DISPLAY_ENV: &str = "WAYLAND_DISPLAY";

/// The window backend draw asks miniquad for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Backend {
    /// X11 only, through Xwayland on Wayland sessions.
    X11,
    /// Native Wayland, falling back to X11 (the default on Wayland
    /// sessions).
    Wayland,
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

    /// The backend called `name` (any case, trimmed), if there is one.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let name = name.trim();
        [Self::X11, Self::Wayland]
            .into_iter()
            .find(|backend| backend.name().eq_ignore_ascii_case(name))
    }

    /// The name `--backend` takes for this backend.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::X11 => "x11",
            Self::Wayland => "wayland",
        }
    }
}

/// The backend for `requested` (`--backend`, `None` when not given) in a
/// session whose [`WAYLAND_DISPLAY_ENV`] is `wayland_display`: the flag
/// when there is one, else Wayland in a Wayland session and X11 elsewhere.
#[must_use]
pub fn choose(requested: Option<Backend>, wayland_display: Option<&OsStr>) -> Backend {
    requested.unwrap_or(if is_wayland_session(wayland_display) {
        Backend::Wayland
    } else {
        Backend::X11
    })
}

/// Whether `wayland_display` (the value of [`WAYLAND_DISPLAY_ENV`])
/// names a Wayland session.
fn is_wayland_session(wayland_display: Option<&OsStr>) -> bool {
    wayland_display.is_some_and(|value| !value.is_empty())
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
    if backend == Backend::Wayland && is_wayland_session(wayland_display) {
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

    fn display() -> &'static OsStr {
        OsStr::new("wayland-1")
    }

    #[test]
    fn choose_without_flag_follows_the_session() {
        // Arrange, Act, Assert
        assert_eq!(choose(None, Some(display())), Backend::Wayland);
        assert_eq!(choose(None, None), Backend::X11);
        assert_eq!(choose(None, Some(OsStr::new(""))), Backend::X11);
    }

    #[test]
    fn choose_honours_the_flag_in_any_session() {
        // Arrange, Act, Assert
        assert_eq!(choose(Some(Backend::X11), Some(display())), Backend::X11);
        assert_eq!(choose(Some(Backend::X11), None), Backend::X11);
        assert_eq!(
            choose(Some(Backend::Wayland), Some(display())),
            Backend::Wayland
        );
        // miniquad falls back to X11 here
        assert_eq!(choose(Some(Backend::Wayland), None), Backend::Wayland);
    }

    #[test]
    fn from_name_ignores_case_and_surrounding_space() {
        // Arrange, Act, Assert
        assert_eq!(Backend::from_name(" Wayland "), Some(Backend::Wayland));
        assert_eq!(Backend::from_name("WAYLAND"), Some(Backend::Wayland));
        assert_eq!(Backend::from_name("\tX11\n"), Some(Backend::X11));
        assert_eq!(Backend::from_name("wl"), None);
        assert_eq!(Backend::from_name(""), None);
    }

    #[test]
    fn backend_linux_backend_maps_to_miniquad() {
        // Arrange / Act / Assert
        assert_eq!(Backend::X11.linux_backend(), LinuxBackend::X11Only);
        assert_eq!(
            Backend::Wayland.linux_backend(),
            LinuxBackend::WaylandWithX11Fallback
        );
    }

    #[test]
    fn x11_is_not_marked_deprecated() {
        // Arrange
        let texts = [
            include_str!("../../README.md"),
            include_str!("../../packaging/flatpak/io.github.UmBender.draw.yml"),
            include_str!("../../packaging/runtime-libs"),
            include_str!("../../docs/features/Native Wayland.md")
                .split("## Rollback")
                .next()
                .unwrap_or_default(),
        ];

        // Act / Assert
        for text in texts {
            // Titles of linked notes keep the word; the prose must not.
            let hit = text.lines().find(|line| {
                let prose = line.split("[[").next().unwrap_or_default();
                !line.starts_with("task:")
                    && !line.starts_with("adrs:")
                    && prose.to_lowercase().contains("deprecat")
            });
            assert_eq!(hit, None);
        }
    }

    #[test]
    fn readme_documents_wayland_default_and_x11_supported() {
        // Arrange
        let readme = include_str!("../../README.md");

        // Act
        let row = readme
            .lines()
            .find(|line| line.starts_with("| `--backend` |"))
            .unwrap_or_default();
        let experimental = readme
            .split("### Experimental")
            .nth(1)
            .and_then(|rest| rest.split("\n## ").next())
            .unwrap_or_default();

        // Assert
        assert!(row.contains("`wayland` (default"), "{row}");
        assert!(row.contains("`x11`"), "{row}");
        assert!(!row.contains("removed"), "{row}");
        assert!(!row.contains("experimental"), "{row}");
        assert!(
            !experimental.contains("--backend"),
            "native Wayland is not experimental"
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
        fn choose_follows_flag_then_session(
            flag in proptest::option::of(prop_oneof![Just(Backend::X11), Just(Backend::Wayland)]),
            session in any::<bool>(),
        ) {
            let display = session.then_some(OsStr::new("wayland-0"));
            let expected = flag.unwrap_or(if session { Backend::Wayland } else { Backend::X11 });
            prop_assert_eq!(choose(flag, display), expected);
        }
    }
}
