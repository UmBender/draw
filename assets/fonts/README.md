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
name. It typesets `$…$` formulas — see ADR-T33-1 and ADR-T55-1 in
`docs/decisions/`.

The ranges: ASCII, Latin-1 and Latin Extended-A, Greek, general
punctuation, combining marks for symbols, letterlike symbols, arrows,
mathematical operators, floor/ceiling, geometric shapes, angle brackets,
long arrows, supplemental arrows, n-ary operators `U+2A00–2A0C`, `⨿`, and
the whole Mathematical Alphanumeric Symbols block (bold, italic, script,
fraktur, double-struck, sans-serif, monospace, Greek and digit styles).
The subset is about 270 KB (budget: under 300 KB).

Rebuilt with fontTools (the `MATH` table is dropped; its constants live in
`core::tex`). `python3 -m fontTools.subset` is the same program as
`pyftsubset`:

```sh
pyftsubset latinmodern-math.otf \
  --unicodes="U+0020-007E,U+00A0-017F,U+0370-03FF,U+2000-206F,U+20D0-20DF,U+2100-214F,U+2190-21FF,U+2200-22FF,U+2308-230B,U+25A0-25FF,U+27E8-27E9,U+27F0-27FF,U+2900-297F,U+2A00-2A0C,U+2A3F,U+1D400-1D7FF" \
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
