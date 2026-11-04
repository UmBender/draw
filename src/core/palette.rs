//! The drawing palette and colour helpers.
//!
//! Shapes store a [`ColorId`] — an index into the fixed drawing palette — so
//! a theme change recolours every existing shape (ADR-0012). The colour
//! values live in [`crate::theme`], the one file that defines colours
//! (ADR-T51-1).
#![allow(
    clippy::unreadable_literal,
    reason = "test colours are written 0xRRGGBB, as in the theme file"
)]

use crate::theme;

/// Number of drawing colours, addressable with keys `1`–`6`.
pub const PALETTE_LEN: usize = 6;

/// An 8-bit-per-channel colour with alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgba {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
    /// Alpha channel; `0xff` is opaque.
    pub a: u8,
}

impl Rgba {
    /// Builds an opaque colour from `0xRRGGBB`; bits above the low 24 are ignored.
    #[must_use]
    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: (hex >> 16) as u8,
            g: (hex >> 8) as u8,
            b: hex as u8,
            a: 0xff,
        }
    }

    /// Builds a colour from `0xRRGGBB` with opacity `alpha` (`0` invisible,
    /// `255` opaque); bits above the low 24 are ignored.
    #[must_use]
    pub const fn from_hex_alpha(hex: u32, alpha: u8) -> Self {
        Self {
            a: alpha,
            ..Self::from_hex(hex)
        }
    }

    /// Opaque white. As a tint it leaves a texture's colours unchanged, so
    /// it is not a theme colour (ADR-T51-1).
    pub const NO_TINT: Self = Self {
        r: u8::MAX,
        g: u8::MAX,
        b: u8::MAX,
        a: u8::MAX,
    };

    /// This colour with its alpha scaled by `alpha / 255`, rounded down:
    /// an opaque colour gets exactly `alpha`, `255` changes nothing.
    #[must_use]
    pub const fn faded(self, alpha: u8) -> Self {
        // At most 255 × 255 / 255 = 255, so the cast never truncates.
        let a = (self.a as u16 * alpha as u16 / u8::MAX as u16) as u8;
        Self { a, ..self }
    }

    /// Returns `[r, g, b, a]` scaled to `0.0..=1.0`, the form GPU APIs expect.
    #[must_use]
    pub fn to_f32(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a].map(|c| f32::from(c) / 255.0)
    }

    /// WCAG 2 relative luminance of the colour (alpha ignored): `0.0` for
    /// black, `1.0` for white.
    #[must_use]
    pub fn relative_luminance(self) -> f32 {
        // sRGB channel to linear light (WCAG 2 definition).
        let linear = |c: u8| {
            let s = f32::from(c) / 255.0;
            if s <= 0.039_28 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(self.r) + 0.7152 * linear(self.g) + 0.0722 * linear(self.b)
    }
}

/// WCAG 2 contrast ratio of two colours, from `1.0` (the same luminance)
/// to `21.0` (black and white); symmetric in its arguments.
#[must_use]
pub fn contrast_ratio(a: Rgba, b: Rgba) -> f32 {
    let (la, lb) = (a.relative_luminance(), b.relative_luminance());
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// Index of a colour in the drawing palette, always `< PALETTE_LEN`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct ColorId(u8);

impl ColorId {
    /// The default drawing colour (key `1`).
    pub const INK: Self = Self(0);

    /// Returns the id for `index`, or `None` if `index >= PALETTE_LEN`.
    #[must_use]
    pub const fn new(index: u8) -> Option<Self> {
        if (index as usize) < PALETTE_LEN {
            Some(Self(index))
        } else {
            None
        }
    }

    /// Maps a number key `1`–`6` to ids 0–5; any other digit gives `None`.
    #[must_use]
    pub const fn from_key_digit(digit: u8) -> Option<Self> {
        // `0` wraps to 255 and is rejected by `new`.
        Self::new(digit.wrapping_sub(1))
    }

    /// Returns the palette index, guaranteed `< PALETTE_LEN`.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// The drawing colours in key order: ink, red, green, blue, yellow, magenta,
/// as set in [`crate::theme`].
pub const PALETTE: [Rgba; PALETTE_LEN] = [
    Rgba::from_hex(theme::INK),
    Rgba::from_hex(theme::RED),
    Rgba::from_hex(theme::GREEN),
    Rgba::from_hex(theme::BLUE),
    Rgba::from_hex(theme::YELLOW),
    Rgba::from_hex(theme::MAGENTA),
];

/// Returns the colour for `id`. Total: every `ColorId` is a valid index.
#[must_use]
pub const fn palette(id: ColorId) -> Rgba {
    // In bounds: `ColorId` can only be built with an index `< PALETTE_LEN`.
    PALETTE[id.index()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::approx_eq;
    use proptest::prelude::*;

    const EPS: f32 = 1e-6;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < EPS
    }

    fn all_close(a: [f32; 4], b: [f32; 4]) -> bool {
        a.iter().zip(b.iter()).all(|(x, y)| close(*x, *y))
    }

    // AC-1

    #[test]
    fn from_hex_splits_channels_and_is_opaque() {
        // Arrange
        let hex = 0x123456;

        // Act
        let c = Rgba::from_hex(hex);

        // Assert
        assert_eq!(
            c,
            Rgba {
                r: 0x12,
                g: 0x34,
                b: 0x56,
                a: 0xff
            }
        );
    }

    #[test]
    fn from_hex_ignores_bits_above_24() {
        assert_eq!(Rgba::from_hex(0xab123456), Rgba::from_hex(0x123456));
    }

    #[test]
    fn to_f32_maps_extremes_to_unit_range() {
        // Arrange
        let black = Rgba::from_hex(0x000000);
        let white = Rgba::from_hex(0xffffff);

        // Act
        let (b, w) = (black.to_f32(), white.to_f32());

        // Assert
        assert!(all_close(b, [0.0, 0.0, 0.0, 1.0]), "{b:?}");
        assert!(all_close(w, [1.0, 1.0, 1.0, 1.0]), "{w:?}");
    }

    #[test]
    fn to_f32_scales_each_channel_by_255() {
        // Arrange
        let c = Rgba {
            r: 51,
            g: 102,
            b: 153,
            a: 204,
        };

        // Act
        let f = c.to_f32();

        // Assert
        assert!(all_close(f, [0.2, 0.4, 0.6, 0.8]), "{f:?}");
    }

    // AC-2

    #[test]
    fn color_id_new_accepts_indices_below_palette_len() {
        for i in 0..PALETTE_LEN as u8 {
            let id = ColorId::new(i);
            assert_eq!(id.map(ColorId::index), Some(usize::from(i)));
        }
    }

    #[test]
    fn color_id_ink_is_index_zero() {
        assert_eq!(ColorId::INK.index(), 0);
        assert_eq!(ColorId::new(0), Some(ColorId::INK));
    }

    proptest! {
        #[test]
        fn color_id_new_rejects_indices_at_or_above_palette_len(
            i in (PALETTE_LEN as u8)..=u8::MAX
        ) {
            prop_assert_eq!(ColorId::new(i), None);
        }

        // AC-3

        #[test]
        fn palette_lookup_total_for_valid_ids(i in any::<u8>()) {
            if let Some(id) = ColorId::new(i) {
                let c = palette(id);
                prop_assert_eq!(c.a, 0xff);
                prop_assert_eq!(c, PALETTE[id.index()]);
            }
        }

        // AC-5

        #[test]
        fn key_digit_outside_one_to_six_is_rejected(d in any::<u8>()) {
            prop_assume!(!(1..=6).contains(&d));
            prop_assert_eq!(ColorId::from_key_digit(d), None);
        }
    }

    #[test]
    fn palette_matches_documented_drawing_colours() {
        // Arrange: docs/architecture/Theme.md, keys 1-6 in order.
        let expected = [
            0xc5c9c5, // ink
            0xd16961, // red
            0x8aa86e, // green
            0x7fa8bc, // blue
            0xceb680, // yellow
            0xaa88ac, // magenta
        ];

        for (i, hex) in expected.into_iter().enumerate() {
            // Act
            let id = ColorId::new(i as u8);

            // Assert
            assert_eq!(id.map(palette), Some(Rgba::from_hex(hex)), "index {i}");
        }
    }

    // ---- T51 AC-2: drawing colours come from the theme file ---------------

    #[test]
    fn palette_reads_drawing_colours_from_theme() {
        // Arrange
        let expected = [
            theme::INK,
            theme::RED,
            theme::GREEN,
            theme::BLUE,
            theme::YELLOW,
            theme::MAGENTA,
        ];

        // Act / Assert
        for (i, hex) in expected.into_iter().enumerate() {
            assert_eq!(PALETTE[i], Rgba::from_hex(hex), "index {i}");
        }
    }

    // ---- T51 AC-4: colour helpers -----------------------------------------

    #[test]
    fn from_hex_alpha_keeps_channels_and_sets_alpha() {
        // Act
        let c = Rgba::from_hex_alpha(0x123456, 63);

        // Assert
        assert_eq!(
            c,
            Rgba {
                r: 0x12,
                g: 0x34,
                b: 0x56,
                a: 63
            }
        );
    }

    #[test]
    fn faded_opaque_gives_alpha() {
        for alpha in [0, 1, 63, 76, 89, 254, 255] {
            assert_eq!(Rgba::from_hex(0xabcdef).faded(alpha).a, alpha);
        }
    }

    #[test]
    fn faded_scales_existing_alpha() {
        // 128 × 128 / 255 = 64.25, truncated.
        assert_eq!(Rgba::from_hex_alpha(0, 128).faded(128).a, 64);
        assert_eq!(Rgba::from_hex_alpha(0x112233, 0).faded(200).a, 0);
    }

    #[test]
    fn no_tint_is_opaque_white() {
        assert_eq!(Rgba::NO_TINT, Rgba::from_hex(0xffffff));
    }

    #[test]
    fn relative_luminance_of_black_and_white() {
        assert!(approx_eq(
            Rgba::from_hex(0x000000).relative_luminance(),
            0.0,
            EPS
        ));
        assert!(approx_eq(
            Rgba::from_hex(0xffffff).relative_luminance(),
            1.0,
            EPS
        ));
    }

    #[test]
    fn relative_luminance_weights_green_most() {
        // WCAG 2 weights: 0.2126 R + 0.7152 G + 0.0722 B.
        let lum = |hex| Rgba::from_hex(hex).relative_luminance();
        assert!(approx_eq(lum(0xff0000), 0.2126, 1e-4));
        assert!(approx_eq(lum(0x00ff00), 0.7152, 1e-4));
        assert!(approx_eq(lum(0x0000ff), 0.0722, 1e-4));
    }

    #[test]
    fn contrast_ratio_black_on_white_is_21() {
        // Act
        let ratio = contrast_ratio(Rgba::from_hex(0x000000), Rgba::from_hex(0xffffff));

        // Assert
        assert!(approx_eq(ratio, 21.0, 1e-3), "{ratio}");
    }

    proptest! {
        #[test]
        fn faded_by_opaque_is_identity(r: u8, g: u8, b: u8, a: u8) {
            let c = Rgba { r, g, b, a };
            prop_assert_eq!(c.faded(u8::MAX), c);
        }

        #[test]
        fn contrast_ratio_symmetric_and_bounded(x in 0_u32..=0xff_ffff, y in 0_u32..=0xff_ffff) {
            // Arrange
            let (a, b) = (Rgba::from_hex(x), Rgba::from_hex(y));

            // Act
            let (ab, ba) = (contrast_ratio(a, b), contrast_ratio(b, a));

            // Assert
            prop_assert!(approx_eq(ab, ba, EPS));
            prop_assert!((1.0 - EPS..=21.0 + 1e-3).contains(&ab), "{}", ab);
            prop_assert!(approx_eq(contrast_ratio(a, a), 1.0, EPS));
        }
    }

    // AC-5

    #[test]
    fn key_digit_mapping() {
        for d in 1..=6_u8 {
            let id = ColorId::from_key_digit(d);
            assert_eq!(id.map(ColorId::index), Some(usize::from(d - 1)));
        }
    }
}
