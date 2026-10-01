//! Drawing palette and UI theme tokens (Kanagawa Dragon).
//!
//! Owned by T02; filled in by that task.

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
