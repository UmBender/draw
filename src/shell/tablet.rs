#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tap::PEN_TAP_SLOP_PX;
    use proptest::prelude::*;
    use std::ffi::OsStr;

    /// Blocks of the owner's `/proc/bus/input/devices` (2026-10-20, x86_64):
    /// keyboard, touchpad, the HUION 420 pen and its mouse interface.
    const OWNER_DEVICES: &str = "\
I: Bus=0011 Vendor=0001 Product=0001 Version=ab83
N: Name=\"AT Translated Set 2 keyboard\"
P: Phys=isa0060/serio0/input0
S: Sysfs=/devices/platform/i8042/serio0/input/input4
U: Uniq=
H: Handlers=sysrq kbd leds event4
B: PROP=0
B: EV=120013
B: KEY=402000007 ff803078f800d001 feffffdfffcfffff fffffffffffffffe
B: MSC=10
B: LED=7

I: Bus=0018 Vendor=04f3 Product=3067 Version=0100
N: Name=\"ELAN1200:00 04F3:3067 Touchpad\"
P: Phys=i2c-ELAN1200:00
S: Sysfs=/devices/pci0000:00/0000:00:15.1/i2c_designware.1/i2c-1/i2c-ELAN1200:00/0018:04F3:3067.0009/input/input22
U: Uniq=
H: Handlers=mouse6 event19
B: PROP=5
B: EV=1b
B: KEY=e520 10000 0 0 0 0
B: ABS=2e0800000000003
B: MSC=20

I: Bus=0003 Vendor=256c Product=006e Version=0111
N: Name=\"HUION 420 Pen\"
P: Phys=usb-0000:00:14.0-3/input0
S: Sysfs=/devices/pci0000:00/0000:00:14.0/usb1/1-3/1-3:1.0/0003:256C:006E.000A/input/input35
U: Uniq=HVAN
H: Handlers=mouse1 event13
B: PROP=1
B: EV=1b
B: KEY=1c01 0 0 0 0 0
B: ABS=1000003
B: MSC=10

I: Bus=0003 Vendor=256c Product=006e Version=0111
N: Name=\"HUION 420 Mouse\"
P: Phys=usb-0000:00:14.0-3/input1
S: Sysfs=/devices/pci0000:00/0000:00:14.0/usb1/1-3/1-3:1.1/0003:256C:006E.000B/input/input37
U: Uniq=
H: Handlers=mouse2 event15
B: PROP=0
B: EV=17
B: KEY=1f0000 0 0 0 0
B: REL=1943
B: MSC=10
";

    /// [`OWNER_DEVICES`] without the pen's block.
    fn without_pen() -> String {
        OWNER_DEVICES
            .split("\n\n")
            .filter(|block| !block.contains("HUION 420 Pen"))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn choose_str(value: &str, devices: Option<&str>) -> Choice {
        choose(Some(OsStr::new(value)), devices)
    }

    // ---- AC-3 detection ----------------------------------------------------

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn find_pen_finds_the_huion_420_pen() {
        assert_eq!(find_pen(OWNER_DEVICES), Some("HUION 420 Pen"));
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn find_pen_ignores_keyboards_mice_and_touchpads() {
        // Arrange
        let others = without_pen();

        // Act / Assert
        assert_eq!(find_pen(&others), None);
        assert_eq!(find_pen(""), None);
    }

    #[test]
    fn find_pen_needs_both_the_pen_key_and_the_axes() {
        // BTN_TOOL_PEN (0x140) is bit 0 of the word for bits 320..; ABS_X and
        // ABS_Y are bits 0 and 1. Built with the platform word size.
        let words = 320 / usize::BITS as usize;
        let pen_key = format!("1{}", " 0".repeat(words));
        let pen = |key: &str, abs: &str| format!("N: Name=\"P\"\nB: KEY={key}\nB: ABS={abs}\n");

        assert_eq!(find_pen(&pen(&pen_key, "3")), Some("P"));
        assert_eq!(find_pen(&pen(&pen_key, "1")), None, "no ABS_Y");
        assert_eq!(find_pen(&pen("0", "3")), None, "no BTN_TOOL_PEN");
        assert_eq!(
            find_pen(&format!("N: Name=\"P\"\nB: KEY={pen_key}\n")),
            None
        );
    }

    proptest! {
        #[test]
        fn find_pen_never_panics(text in "(N: Name=\"[a-z ]{0,4}\"|B: (KEY|ABS)=[0-9a-fz ]{0,40}|\n|[ -~]{0,12}){0,12}") {
            let _ = find_pen(&text);
        }
    }

    // ---- AC-3 choice -------------------------------------------------------

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn choose_auto_follows_detection() {
        // Arrange
        let others = without_pen();

        for value in [None, Some(""), Some("auto")] {
            // Act
            let found = choose(value.map(OsStr::new), Some(OWNER_DEVICES));
            let absent = choose(value.map(OsStr::new), Some(&others));
            let unreadable = choose(value.map(OsStr::new), None);

            // Assert
            assert!(found.pen, "{value:?}");
            assert!(found.warning.is_none());
            assert!(
                found
                    .notice
                    .as_deref()
                    .is_some_and(|n| n.contains("HUION 420 Pen")),
                "{found:?}"
            );
            assert!(!absent.pen && absent.notice.is_none() && absent.warning.is_none());
            assert!(!unreadable.pen && unreadable.warning.is_none());
        }
    }

    #[test]
    fn choose_on_and_off_force_pen_mode() {
        // Arrange
        let empty = Some("");

        // Act
        let on = choose_str("on", empty);
        let off = choose_str("off", Some(OWNER_DEVICES));

        // Assert
        assert!(on.pen && on.warning.is_none());
        assert!(on.notice.as_deref().is_some_and(|n| n.contains(TABLET_ENV)));
        assert!(!off.pen && off.notice.is_none() && off.warning.is_none());
    }

    #[test]
    fn choose_ignores_case_and_surrounding_space() {
        for value in ["ON", " on ", "On\n"] {
            assert!(choose_str(value, None).pen, "{value:?}");
        }
        for value in ["OFF", " off", "Off "] {
            assert!(!choose_str(value, Some(OWNER_DEVICES)).pen, "{value:?}");
        }
        assert!(choose_str(" AUTO ", None).warning.is_none());
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn choose_unknown_value_warns_and_detects() {
        for value in ["yes", "1", "pen", "of"] {
            // Act
            let found = choose_str(value, Some(OWNER_DEVICES));
            let absent = choose_str(value, None);

            // Assert
            assert!(found.pen, "{value:?}");
            assert!(!absent.pen, "{value:?}");
            for choice in [&found, &absent] {
                let warning = choice.warning.as_deref().unwrap_or_default();
                assert!(warning.contains(TABLET_ENV) && warning.contains(value.trim()));
            }
        }
    }

    #[test]
    fn choose_on_gives_the_pen_tap_slop() {
        assert!((choose_str("on", None).tap_slop_px() - PEN_TAP_SLOP_PX).abs() < 1e-6);
        assert!(choose_str("off", None).tap_slop_px().abs() < f32::EPSILON);
    }
}
