//! Pen tablet detection at start-up (T34, ADR-T34-1).
//!
//! [`choose`] is a pure function over the `--tablet` mode and the text of
//! `/proc/bus/input/devices`; [`detect`] reads the real device list.
//! With a pen attached the editor gets the tap slop of
//! [`PEN_TAP_SLOP_PX`].

use crate::core::tap::PEN_TAP_SLOP_PX;
use crate::shell::options::TabletMode;

/// The kernel's list of input devices (world-readable).
pub const DEVICES_PATH: &str = "/proc/bus/input/devices";

/// Whether pen mode is on and what to tell the user.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Choice {
    /// Pen taps are held still.
    pub pen: bool,
    /// One stderr line when pen mode is on.
    pub notice: Option<String>,
}

impl Choice {
    /// The editor's tap slop for this choice: [`PEN_TAP_SLOP_PX`] in pen
    /// mode, `0` otherwise.
    #[must_use]
    pub fn tap_slop_px(&self) -> f32 {
        if self.pen { PEN_TAP_SLOP_PX } else { 0.0 }
    }
}

/// The name of the first device in `devices` (the text of
/// [`DEVICES_PATH`]) with the `BTN_TOOL_PEN` key and the `ABS_X` and
/// `ABS_Y` axes.
#[must_use]
pub fn find_pen(devices: &str) -> Option<&str> {
    let mut name = None;
    let mut pen_key = false;
    let mut axes = false;
    // Blocks end at a blank line; the final "" closes the last one.
    for line in devices.lines().chain(std::iter::once("")) {
        let line = line.trim_end();
        if line.is_empty() {
            if pen_key && axes && name.is_some() {
                return name;
            }
            (name, pen_key, axes) = (None, false, false);
        } else if let Some(value) = line.strip_prefix("N: Name=") {
            name = Some(value.trim_matches('"'));
        } else if let Some(mask) = line.strip_prefix("B: KEY=") {
            pen_key = has_bit(mask, BTN_TOOL_PEN);
        } else if let Some(mask) = line.strip_prefix("B: ABS=") {
            axes = has_bit(mask, ABS_X) && has_bit(mask, ABS_Y);
        }
    }
    None
}

/// `BTN_TOOL_PEN` in `linux/input-event-codes.h`.
const BTN_TOOL_PEN: usize = 0x140;
/// `ABS_X` in `linux/input-event-codes.h`.
const ABS_X: usize = 0x00;
/// `ABS_Y` in `linux/input-event-codes.h`.
const ABS_Y: usize = 0x01;

/// Whether `bit` is set in `mask`, a `/proc` bitmap: hexadecimal words of
/// `usize::BITS` bits (the kernel's `long`), most significant first.
fn has_bit(mask: &str, bit: usize) -> bool {
    let word_bits = usize::BITS as usize;
    mask.split_whitespace()
        .rev()
        .nth(bit / word_bits)
        .and_then(|word| usize::from_str_radix(word, 16).ok())
        .is_some_and(|word| (word >> (bit % word_bits)) & 1 == 1)
}

/// Pen mode for `mode` (`--tablet`) and `devices`, the text of
/// [`DEVICES_PATH`] (`None` when unreadable).
#[must_use]
pub fn choose(mode: TabletMode, devices: Option<&str>) -> Choice {
    match mode {
        TabletMode::On => Choice {
            pen: true,
            notice: Some(format!(
                "draw: --tablet on: pen taps within {PEN_TAP_SLOP_PX} px are clicks"
            )),
        },
        TabletMode::Off => Choice::default(),
        TabletMode::Auto => {
            let pen = devices.and_then(find_pen);
            Choice {
                pen: pen.is_some(),
                notice: pen.map(|name| {
                    format!(
                        "draw: pen tablet {name:?} found: taps within {PEN_TAP_SLOP_PX} px \
                         are clicks; --tablet off turns this off"
                    )
                }),
            }
        }
    }
}

/// [`choose`] applied to the real device list.
#[must_use]
pub fn detect(mode: TabletMode) -> Choice {
    let devices = std::fs::read_to_string(DEVICES_PATH).ok();
    choose(mode, devices.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tap::PEN_TAP_SLOP_PX;
    use proptest::prelude::*;

    /// Blocks of the owner's `/proc/bus/input/devices` (2026-10-20, `x86_64`):
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

        // Act
        let found = choose(TabletMode::Auto, Some(OWNER_DEVICES));
        let absent = choose(TabletMode::Auto, Some(&others));
        let unreadable = choose(TabletMode::Auto, None);

        // Assert
        assert!(found.pen);
        assert!(
            found
                .notice
                .as_deref()
                .is_some_and(|n| n.contains("HUION 420 Pen") && n.contains("--tablet off")),
            "{found:?}"
        );
        assert!(!absent.pen && absent.notice.is_none());
        assert!(!unreadable.pen && unreadable.notice.is_none());
    }

    #[test]
    fn choose_on_and_off_force_pen_mode() {
        // Act
        let on = choose(TabletMode::On, Some(""));
        let off = choose(TabletMode::Off, Some(OWNER_DEVICES));

        // Assert
        assert!(on.pen);
        assert!(
            on.notice
                .as_deref()
                .is_some_and(|n| n.contains("--tablet on"))
        );
        assert!(!off.pen && off.notice.is_none());
    }

    #[test]
    fn choose_on_gives_the_pen_tap_slop() {
        assert!((choose(TabletMode::On, None).tap_slop_px() - PEN_TAP_SLOP_PX).abs() < 1e-6);
        assert!(choose(TabletMode::Off, None).tap_slop_px().abs() < f32::EPSILON);
    }
}
