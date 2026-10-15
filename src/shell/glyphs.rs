#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::{Vec2, approx_eq};
    use crate::shell::app::TEXT_FONT;
    use proptest::prelude::*;

    const EPS: f32 = 1e-4;

    fn atlas() -> GlyphAtlas {
        match GlyphAtlas::new(TEXT_FONT) {
            Ok(atlas) => atlas,
            Err(err) => panic!("embedded font loads: {err}"),
        }
    }

    fn entry(xmin: f32, ymin: f32, w: u16, h: u16) -> AtlasEntry {
        AtlasEntry {
            page: 0,
            rect: TexelRect {
                x: 100,
                y: 200,
                w,
                h,
            },
            xmin,
            ymin,
            advance: 12.0,
        }
    }

    fn overlaps(a: TexelRect, b: TexelRect) -> bool {
        a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
    }

    // ---- AC-2 glyph quads --------------------------------------------------

    #[test]
    fn glyph_quad_sits_on_baseline_after_bearing() {
        // Arrange: bitmap 10×20 texels, 2 right of the pen, bottom 3 below
        // the baseline.
        let e = entry(2.0, -3.0, 10, 20);

        // Act
        let quad = glyph_quad(Vec2::new(50.0, 100.0), &e, 1.0);

        // Assert
        let Some(quad) = quad else {
            panic!("a non-empty bitmap has a quad");
        };
        assert!(quad.min.approx_eq(Vec2::new(52.0, 83.0), EPS), "{quad:?}");
        assert!(quad.max.approx_eq(Vec2::new(62.0, 103.0), EPS), "{quad:?}");
    }

    #[test]
    fn glyph_quad_scales_bitmap_by_scale() {
        // Arrange
        let e = entry(2.0, -3.0, 10, 20);

        // Act
        let quad = glyph_quad(Vec2::new(50.0, 100.0), &e, 0.5);

        // Assert: bearing and size both halve.
        let Some(quad) = quad else {
            panic!("a non-empty bitmap has a quad");
        };
        assert!(quad.min.approx_eq(Vec2::new(51.0, 91.5), EPS), "{quad:?}");
        assert!(quad.max.approx_eq(Vec2::new(56.0, 101.5), EPS), "{quad:?}");
    }

    #[test]
    fn glyph_quad_uvs_cover_entry_texels() {
        // Arrange
        let e = entry(0.0, 0.0, 10, 20);
        let side = f32::from(PAGE_SIDE);

        // Act
        let quad = glyph_quad(Vec2::ZERO, &e, 1.0);

        // Assert
        let Some(quad) = quad else {
            panic!("a non-empty bitmap has a quad");
        };
        assert!(
            quad.uv_min
                .approx_eq(Vec2::new(100.0 / side, 200.0 / side), EPS)
        );
        assert!(
            quad.uv_max
                .approx_eq(Vec2::new(110.0 / side, 220.0 / side), EPS)
        );
    }

    #[test]
    fn glyph_quad_empty_bitmap_is_none() {
        assert!(glyph_quad(Vec2::ZERO, &entry(0.0, 0.0, 0, 0), 1.0).is_none());
        assert!(glyph_quad(Vec2::ZERO, &entry(0.0, 0.0, 5, 0), 1.0).is_none());
    }

    #[test]
    fn glyph_quad_non_finite_is_none() {
        let e = entry(0.0, 0.0, 10, 20);

        assert!(glyph_quad(Vec2::new(f32::NAN, 0.0), &e, 1.0).is_none());
        assert!(glyph_quad(Vec2::ZERO, &e, f32::INFINITY).is_none());
        assert!(glyph_quad(Vec2::ZERO, &e, f32::NAN).is_none());
        assert!(glyph_quad(Vec2::ZERO, &e, 0.0).is_none());
    }

    #[test]
    fn raster_px_scales_by_dpi_and_clamps() {
        assert_eq!(raster_px(20, 1.0), 20);
        assert_eq!(raster_px(20, 1.5), 30);
        assert_eq!(raster_px(20, 1.25), 25);
        assert_eq!(raster_px(12, 1.1), 14, "rounded up, like macroquad");
        assert_eq!(raster_px(256, 4.0), MAX_RASTER_PX);
        assert_eq!(raster_px(20, f32::NAN), 20, "bad DPI counts as 1");
        assert_eq!(raster_px(20, -2.0), 20);
        assert_eq!(raster_px(0, 1.0), 1);
    }

    #[test]
    fn glyph_scale_at_dpi_two_keeps_logical_size() {
        // Arrange: 30 px text from the 32 px raster, at DPI 2.
        let rpx = raster_px(32, 2.0);

        // Act
        let scale = glyph_scale(30.0, rpx);

        // Assert: one em of texels covers 30 logical pixels.
        assert_eq!(rpx, 64);
        assert!(approx_eq(scale * f32::from(rpx), 30.0, EPS));
    }

    // ---- AC-3 packer -------------------------------------------------------

    #[test]
    fn packer_first_rect_is_at_gap() {
        let mut packer = ShelfPacker::new(64);

        let rect = packer.pack(10, 8);

        assert_eq!(
            rect,
            Some(TexelRect {
                x: GAP,
                y: GAP,
                w: 10,
                h: 8
            })
        );
    }

    #[test]
    fn packer_rejects_rect_larger_than_page() {
        let mut packer = ShelfPacker::new(64);

        assert_eq!(packer.pack(64, 4), None);
        assert_eq!(packer.pack(4, 64 - 2 * GAP + 1), None);
        assert!(packer.pack(64 - 2 * GAP, 64 - 2 * GAP).is_some());
    }

    #[test]
    fn packer_full_page_returns_none() {
        // Arrange: 32-texel cells with gaps, 64-texel page: one per page.
        let mut packer = ShelfPacker::new(64);
        assert!(packer.pack(32, 32).is_some());

        // Act
        let next = packer.pack(32, 32);

        // Assert
        assert_eq!(next, None);
        packer.clear();
        assert!(packer.pack(32, 32).is_some(), "clear empties the page");
    }

    proptest! {
        #[test]
        fn packer_never_overlaps_or_leaves_page(
            sizes in prop::collection::vec((0u16..300, 0u16..300), 1..200),
        ) {
            let side = 1024;
            let mut packer = ShelfPacker::new(side);
            let mut placed: Vec<TexelRect> = Vec::new();
            for (w, h) in sizes {
                if let Some(r) = packer.pack(w, h) {
                    prop_assert_eq!((r.w, r.h), (w, h));
                    prop_assert!(r.x >= GAP && r.y >= GAP);
                    prop_assert!(u32::from(r.x) + u32::from(w) + u32::from(GAP) <= u32::from(side));
                    prop_assert!(u32::from(r.y) + u32::from(h) + u32::from(GAP) <= u32::from(side));
                    let grown = |t: TexelRect| TexelRect {
                        x: t.x - GAP / 2, y: t.y - GAP / 2, w: t.w + GAP, h: t.h + GAP,
                    };
                    for other in &placed {
                        prop_assert!(
                            r.w == 0 || r.h == 0 || other.w == 0 || other.h == 0
                                || !overlaps(grown(r), grown(*other)),
                            "{:?} overlaps {:?}", r, other
                        );
                    }
                    placed.push(r);
                }
            }
        }

        #[test]
        fn packer_is_deterministic(
            sizes in prop::collection::vec((0u16..300, 0u16..300), 1..100),
        ) {
            let run = || {
                let mut packer = ShelfPacker::new(1024);
                sizes.iter().map(|&(w, h)| packer.pack(w, h)).collect::<Vec<_>>()
            };
            prop_assert_eq!(run(), run());
        }
    }

    // ---- AC-3 atlas --------------------------------------------------------

    #[test]
    fn atlas_embedded_font_loads() {
        assert!(GlyphAtlas::new(TEXT_FONT).is_ok());
        assert!(GlyphAtlas::new(b"not a font").is_err());
    }

    #[test]
    fn atlas_rasterizes_each_key_once() {
        // Arrange
        let mut atlas = atlas();

        // Act
        let first = atlas.prepare('a', 20);
        let again = atlas.prepare('a', 20);
        let other_size = atlas.prepare('a', 24);

        // Assert
        assert!(first.is_some());
        assert_eq!(first, again);
        assert_eq!(atlas.get('a', 20), first);
        assert_ne!(other_size, first);
        assert_eq!(atlas.take_rasterized(), 2);
        assert_eq!(atlas.take_rasterized(), 0);
        assert_eq!(atlas.get('b', 20), None, "get never rasterizes");
    }

    #[test]
    fn atlas_space_has_no_bitmap() {
        let mut atlas = atlas();

        let space = atlas.prepare(' ', 20);

        let Some(space) = space else {
            panic!("a space has an entry, so it is not looked up again");
        };
        assert_eq!((space.rect.w, space.rect.h), (0, 0));
        assert!(space.advance > 0.0);
    }

    #[test]
    fn atlas_new_page_starts_with_white_block() {
        // Arrange
        let mut atlas = atlas();

        // Act
        atlas.prepare('a', 20);
        let uploads = atlas.take_uploads();

        // Assert: the white block goes up first, opaque white.
        assert_eq!(atlas.page_count(), 1);
        let Some(white) = uploads.first() else {
            panic!("a new page uploads its white block");
        };
        assert_eq!(white.page, 0);
        assert_eq!(
            white.rect,
            TexelRect {
                x: GAP,
                y: GAP,
                w: WHITE_SIDE,
                h: WHITE_SIDE
            }
        );
        assert!(white.rgba.iter().all(|&b| b == 255));
        let centre = f32::from(GAP) + f32::from(WHITE_SIDE) / 2.0;
        let side = f32::from(PAGE_SIDE);
        assert!(approx_eq(WHITE_UV[0], centre / side, EPS));
        assert!(approx_eq(WHITE_UV[1], centre / side, EPS));
    }

    #[test]
    fn atlas_bitmap_matches_fontdue() {
        // Arrange
        let mut atlas = atlas();
        let font = match fontdue::Font::from_bytes(TEXT_FONT, fontdue::FontSettings::default()) {
            Ok(font) => font,
            Err(err) => panic!("font parses: {err}"),
        };
        let (metrics, coverage) = font.rasterize('g', 32.0);

        // Act
        let entry = atlas.prepare('g', 32);
        let uploads = atlas.take_uploads();

        // Assert: same bitmap and metrics as macroquad would draw.
        let Some(entry) = entry else {
            panic!("'g' is in the font");
        };
        assert_eq!(usize::from(entry.rect.w), metrics.width);
        assert_eq!(usize::from(entry.rect.h), metrics.height);
        assert!(approx_eq(entry.xmin, metrics.xmin as f32, EPS));
        assert!(approx_eq(entry.ymin, metrics.ymin as f32, EPS));
        assert!(approx_eq(entry.advance, metrics.advance_width, EPS));
        let Some(glyph) = uploads.iter().find(|u| u.rect == entry.rect) else {
            panic!("the glyph is uploaded");
        };
        let expected: Vec<u8> = coverage.iter().flat_map(|&c| [255, 255, 255, c]).collect();
        assert_eq!(glyph.rgba, expected);
    }

    #[test]
    fn atlas_full_reports_overflow_not_growth() {
        // Arrange: glyphs at the largest raster fill the pages quickly.
        let mut atlas = atlas();

        // Act
        for ch in ('A'..='Z').chain('a'..='z') {
            atlas.prepare(ch, MAX_RASTER_PX);
        }

        // Assert
        assert!(atlas.overflowed());
        assert_eq!(atlas.page_count(), MAX_PAGES);
    }

    #[test]
    fn atlas_clear_resets_entries_and_overflow() {
        // Arrange
        let mut atlas = atlas();
        for ch in ('A'..='Z').chain('a'..='z') {
            atlas.prepare(ch, MAX_RASTER_PX);
        }
        atlas.take_uploads();

        // Act
        atlas.clear();

        // Assert: pages stay, entries go, the white block is reserved again.
        assert!(!atlas.overflowed());
        assert_eq!(atlas.get('A', MAX_RASTER_PX), None);
        assert_eq!(atlas.page_count(), MAX_PAGES);
        let Some(entry) = atlas.prepare('A', 20) else {
            panic!("room again after a clear");
        };
        assert_eq!(entry.page, 0);
        let white = TexelRect {
            x: GAP,
            y: GAP,
            w: WHITE_SIDE,
            h: WHITE_SIDE,
        };
        assert!(!overlaps(entry.rect, white));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn atlas_entries_stay_inside_their_page(
            keys in prop::collection::vec((0usize..60, 0usize..12), 1..80),
        ) {
            let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456αβ∑≤"
                .chars()
                .collect();
            let sizes = [12u16, 16, 20, 24, 32, 40, 48, 64, 96, 128, 192, 256];
            let mut atlas = atlas();
            let mut placed: Vec<AtlasEntry> = Vec::new();
            for (c, s) in keys {
                let Some(e) = atlas.prepare(chars[c % chars.len()], sizes[s]) else {
                    continue;
                };
                let r = e.rect;
                prop_assert!((e.page as usize) < atlas.page_count());
                prop_assert!(u32::from(r.x) + u32::from(r.w) <= u32::from(PAGE_SIDE));
                prop_assert!(u32::from(r.y) + u32::from(r.h) <= u32::from(PAGE_SIDE));
                if r.w > 0 && r.h > 0 && !placed.contains(&e) {
                    for other in &placed {
                        prop_assert!(
                            other.page != e.page || other.rect.w == 0 || !overlaps(r, other.rect)
                        );
                    }
                    placed.push(e);
                }
            }
            for upload in atlas.take_uploads() {
                let r = upload.rect;
                prop_assert_eq!(upload.rgba.len(), 4 * usize::from(r.w) * usize::from(r.h));
            }
        }
    }
}
