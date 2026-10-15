# Embedded font

`AdwaitaMono-Regular-subset.ttf` is a subset of **Adwaita Mono Regular**
(version 34.1, Iosevka-based) under the SIL Open Font License 1.1
(`OFL.txt`; the font declares no reserved font names). It is embedded in the
binary for text blocks and labels — see ADR-T29-1 in `docs/decisions/`.

Rebuilt with [fontTools](https://github.com/fonttools/fonttools):

```sh
pyftsubset AdwaitaMono-Regular.ttf \
  --unicodes="U+0020-007E,U+00A0-017F,U+0370-03FF,U+2000-206F,U+2070-209F,U+2100-214F,U+2190-21FF,U+2200-22FF,U+2300-23FF,U+25A0-25FF,U+27E6-27FF,U+FFFD" \
  --layout-features='' --no-hinting --desubroutinize --name-IDs='*' \
  --output-file=AdwaitaMono-Regular-subset.ttf
```

Every glyph is 600/1000 em wide; `core::text_layout` relies on that.

# Formula font

`LatinModernMath-draw-subset.otf` is a subset of **Latin Modern Math**
(version 1.959, by B. Jackowski, P. Strzelczyk and P. Pianowski, from
[CTAN `lm-math`](https://ctan.org/pkg/lm-math)) under the GUST Font License
(`GUST-FONT-LICENSE.txt`, an instance of the LaTeX Project Public License).
The license asks derived files to be renamed, hence the `-draw-subset` file
name. It typesets `$…$` formulas — see ADR-T33-1 in `docs/decisions/`.

Rebuilt with fontTools (the `MATH` table is dropped; its constants live in
`core::tex`):

```sh
pyftsubset latinmodern-math.otf \
  --unicodes="U+0020-007E,U+00A0-017F,U+0370-03FF,U+2000-206F,U+2100-214F,U+2190-21FF,U+2200-22FF,U+2308-230B,U+27E8-27E9,U+1D434-1D467,U+1D49C-1D4B5,U+1D6E2-1D71B" \
  --layout-features='' --no-hinting --desubroutinize --name-IDs='*' \
  --drop-tables+=MATH,FFTM \
  --output-file=LatinModernMath-draw-subset.otf
```

Then regenerate the per-glyph metrics `core` lays formulas out with:

```sh
scripts/math_metrics.py latinmodern-math.otf \
  assets/fonts/LatinModernMath-draw-subset.otf src/core/math_metrics.rs
```

`math_metrics_match_the_embedded_font` fails until the table matches the font.
