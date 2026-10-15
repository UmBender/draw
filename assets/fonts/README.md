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
