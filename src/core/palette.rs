//! Drawing palette and UI theme tokens (Kanagawa Dragon).
//!
//! This is the only place colours are defined (ADR-0012). Shapes store a
//! [`ColorId`] — an index into the fixed drawing palette — so a theme change
//! recolours every existing shape. Values come from the user's Kanagawa
//! Dragon theme (ADR-0015) and are documented in `docs/architecture/Theme.md`.

/// Number of drawing colours, addressable with keys `1`–`6`.
pub const PALETTE_LEN: usize = 6;

/// Placeholder until the real values are filled in.
const UNSET: Rgba = Rgba {
    r: 0,
    g: 0,
    b: 0,
    a: 0,
};

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
        let _ = hex;
        todo!()
    }

    /// Returns `[r, g, b, a]` scaled to `0.0..=1.0`, the form GPU APIs expect.
    #[must_use]
    pub fn to_f32(self) -> [f32; 4] {
        todo!()
    }
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
        let _ = index;
        todo!()
    }

    /// Maps a number key `1`–`6` to ids 0–5; any other digit gives `None`.
    #[must_use]
    pub const fn from_key_digit(digit: u8) -> Option<Self> {
        let _ = digit;
        todo!()
    }

    /// Returns the palette index, guaranteed `< PALETTE_LEN`.
    #[must_use]
    pub const fn index(self) -> usize {
        todo!()
    }
}

/// The drawing colours in key order: ink, red, green, blue, yellow, magenta.
pub const PALETTE: [Rgba; PALETTE_LEN] = [UNSET; PALETTE_LEN];

/// Returns the colour for `id`. Total: every `ColorId` is a valid index.
#[must_use]
pub const fn palette(id: ColorId) -> Rgba {
    let _ = id;
    todo!()
}

/// UI colour tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Theme {
    /// Canvas background.
    pub bg: Rgba,
    /// Toolbar background.
    pub surface: Rgba,
    /// Toolbar and button outlines.
    pub border: Rgba,
    /// Icons and labels.
    pub text: Rgba,
    /// Active tool and selection outline.
    pub accent: Rgba,
    /// Selection and marquee fill.
    pub selection: Rgba,
}

/// The application theme.
pub const THEME: Theme = Theme {
    bg: UNSET,
    surface: UNSET,
    border: UNSET,
    text: UNSET,
    accent: UNSET,
    selection: UNSET,
};

#[cfg(test)]
mod tests {
    use super::*;
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
        let hex = 0x12_34_56;

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
        assert_eq!(Rgba::from_hex(0xab_12_34_56), Rgba::from_hex(0x12_34_56));
    }

    #[test]
    fn to_f32_maps_extremes_to_unit_range() {
        // Arrange
        let black = Rgba::from_hex(0x00_00_00);
        let white = Rgba::from_hex(0xff_ff_ff);

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
            0xc5_c9_c5, // ink
            0xd1_69_61, // red
            0x8a_a8_6e, // green
            0x7f_a8_bc, // blue
            0xce_b6_80, // yellow
            0xaa_88_ac, // magenta
        ];

        for (i, hex) in expected.into_iter().enumerate() {
            // Act
            let id = ColorId::new(i as u8);

            // Assert
            assert_eq!(id.map(palette), Some(Rgba::from_hex(hex)), "index {i}");
        }
    }

    // AC-4

    #[test]
    fn theme_matches_documented_tokens() {
        assert_eq!(THEME.bg, Rgba::from_hex(0x18_16_16));
        assert_eq!(THEME.surface, Rgba::from_hex(0x0d_0c_0c));
        assert_eq!(THEME.border, Rgba::from_hex(0xa6_a6_9c));
        assert_eq!(THEME.text, Rgba::from_hex(0xc5_c9_c5));
        assert_eq!(THEME.accent, Rgba::from_hex(0x7f_a8_bc));
        assert_eq!(THEME.selection, Rgba::from_hex(0x2d_4f_67));
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
