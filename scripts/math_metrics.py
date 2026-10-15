#!/usr/bin/env python3
"""Regenerates the metrics table of src/core/math_metrics.rs (ADR-T33-1).

Usage: math_metrics.py ORIGINAL.otf SUBSET.otf src/core/math_metrics.rs

Rewrites the lines between the `BEGIN GENERATED` and `END GENERATED`
markers: one row per character of the subset's character map, with the
advance, ink box and MATH italic correction of the original font, in em,
y up. Needs fontTools (`pip install fonttools`).
"""

import sys

from fontTools.pens.boundsPen import BoundsPen
from fontTools.ttLib import TTFont

BEGIN = "// BEGIN GENERATED"
END = "// END GENERATED"


def italic_corrections(font):
    """Glyph name -> italic correction in font units."""
    if "MATH" not in font:
        return {}
    info = font["MATH"].table.MathGlyphInfo.MathItalicsCorrectionInfo
    if info is None:
        return {}
    values = [record.Value for record in info.ItalicsCorrection]
    return dict(zip(info.Coverage.glyphs, values))


def rows(original_path, subset_path):
    """The table rows as Rust source lines."""
    original = TTFont(original_path)
    upem = original["head"].unitsPerEm
    cmap = original.getBestCmap()
    glyphs = original.getGlyphSet()
    italic = italic_corrections(original)

    def em(value):
        return f"{value / upem:.4f}"

    out = []
    for code in sorted(TTFont(subset_path).getBestCmap()):
        name = cmap.get(code)
        if name is None:
            continue
        pen = BoundsPen(glyphs)
        glyphs[name].draw(pen)
        x_min, y_min, x_max, y_max = pen.bounds or (0, 0, 0, 0)
        advance = original["hmtx"][name][0]
        values = ", ".join(
            em(v) for v in (advance, x_min, x_max, y_min, y_max, italic.get(name, 0))
        )
        out.append(f"    ('\\u{{{code:04X}}}', m({values})),")
    return out


def main(original_path, subset_path, target_path):
    with open(target_path, encoding="utf-8") as f:
        lines = f.read().split("\n")
    begin = next(i for i, line in enumerate(lines) if line.strip() == BEGIN)
    end = next(i for i, line in enumerate(lines) if line.strip() == END)
    table = rows(original_path, subset_path)
    lines[begin + 1 : end] = table
    with open(target_path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
    print(f"{len(table)} rows", file=sys.stderr)


if __name__ == "__main__":
    if len(sys.argv) != 4:
        sys.exit(__doc__)
    main(*sys.argv[1:])
