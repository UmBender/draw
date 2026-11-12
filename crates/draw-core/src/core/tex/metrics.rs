//! Layout constants, kerns and glyph extents of the TeX subset.

use super::{Face, math_metrics};

/// Ascent of a text-face glyph box per unit of scale: the text font's cap
/// height 0.735 with its overshoot.
pub const GLYPH_ASCENT: f32 = 0.75;

/// Descent of a text-face glyph box per unit of scale: the text font's
/// descender.
pub const GLYPH_DESCENT: f32 = 0.22;

/// Ascent of text-face brackets and bars per unit of scale.
pub const TALL_ASCENT: f32 = 0.85;

/// Descent of text-face brackets and bars per unit of scale.
pub const TALL_DESCENT: f32 = 0.17;

/// Height of the math axis (the middle of `−`) above the baseline: the
/// math font's `AxisHeight` (ADR-T33-1).
pub const AXIS: f32 = 0.25;

/// Thickness of fraction bars and root signs: `FractionRuleThickness`.
pub const RULE: f32 = 0.04;

/// Scale of scripts, limits and fraction parts relative to their parent:
/// `ScriptPercentScaleDown`.
pub const SCRIPT_SCALE: f32 = 0.7;

/// Smallest scale of nested scripts: `ScriptScriptPercentScaleDown`.
pub const MIN_SCALE: f32 = 0.5;

/// Smallest rise of a superscript's baseline: `SuperscriptShiftUp`.
pub const SUP_SHIFT: f32 = 0.363;

/// A superscript's baseline sits at most this far below the top of its
/// base: `SuperscriptBaselineDropMax`.
pub const SUP_DROP: f32 = 0.25;

/// Smallest height of a superscript's bottom above the baseline:
/// `SuperscriptBottomMin`.
pub const SUP_BOTTOM_MIN: f32 = 0.108;

/// Smallest drop of a subscript's baseline: `SubscriptShiftDown`.
pub const SUB_SHIFT: f32 = 0.247;

/// A subscript's baseline sits this far above the bottom of its base.
pub const SUB_DROP: f32 = 0.05;

/// Highest top of a subscript above the baseline: `SubscriptTopMax`.
pub const SUB_TOP_MAX: f32 = 0.344;

/// Smallest gap between a superscript and a subscript on one base:
/// `SubSuperscriptGapMin`.
pub const SCRIPT_GAP: f32 = 0.16;

/// Space after scripts: `SpaceAfterScript`.
pub const SCRIPT_SPACE: f32 = 0.056;

/// Gap between a big operator and its limits: `LowerLimitGapMin`.
pub const LIMIT_GAP: f32 = 0.167;

/// Scale of the big operators `∑` and `∏`.
pub const BIG_OP_SCALE: f32 = 1.3;

/// Scale of big operators and integrals in display style: the same glyph
/// as [`BIG_OP_SCALE`], larger (ADR-T105-1).
pub const DISPLAY_OP_SCALE: f32 = 1.8;

/// Smallest gap between a fraction bar and its numerator or denominator:
/// `FractionNumeratorGapMin`.
pub const FRAC_GAP: f32 = 0.04;

/// Smallest rise of a numerator's baseline: `FractionNumeratorShiftUp`.
pub const FRAC_NUM_SHIFT: f32 = 0.394;

/// Smallest drop of a denominator's baseline:
/// `FractionDenominatorShiftDown`.
pub const FRAC_DEN_SHIFT: f32 = 0.345;

/// Space left and right of a fraction's parts.
pub const FRAC_PAD: f32 = 0.1;

/// Gap between a root's content and its overbar.
pub const SQRT_GAP: f32 = 0.1;

/// Width of the root sign before the content.
pub const SQRT_SIGN: f32 = 0.55;

/// Largest scale of a binomial's parentheses.
pub const BINOM_MAX_SCALE: f32 = 3.0;

/// Largest scale of a bracket of `\left` and `\right` (ADR-T89-1).
pub const LEFT_RIGHT_MAX_SCALE: f32 = 12.0;

/// Part of the content's span that a growing bracket must reach, TeX's
/// `\delimiterfactor` (ADR-T89-1).
pub const DELIM_FACTOR: f32 = 0.901;

/// Width of the empty delimiter `.`, TeX's `\nulldelimiterspace`.
pub const NULL_DELIM_SPACE: f32 = 0.12;

/// Gap between an accent's ink and the top of its argument (ADR-T39-1).
pub const ACCENT_GAP: f32 = 0.06;

/// Rightward shift of an accent over one italic letter per em of the
/// letter's height: the top of a slanted letter lies right of its middle
/// (ADR-T39-1).
pub const ACCENT_SKEW: f32 = 0.1;

/// Length of the rule `\bar` draws: the ink width of the font's macron.
pub const ACCENT_BAR: f32 = 0.36;

/// Scale of the arrow `\vec` draws.
pub const VEC_SCALE: f32 = 0.5;

/// Space on each side of a relation: TeX's thick space, 5/18 em.
pub const REL_SPACE: f32 = 5.0 / 18.0;

/// Extra space on each side of a wide relation, on top of
/// [`REL_SPACE`]: amsmath's `\;`, a thick space (ADR-T65-1).
pub const WIDE_REL_SPACE: f32 = 5.0 / 18.0;

/// Relations amsmath pads with [`WIDE_REL_SPACE`] (ADR-T65-1).
pub const WIDE_RELATIONS: &[&str] = &["implies", "impliedby"];

/// Space on each side of a binary operator: TeX's medium space, 2/9 em.
pub const BIN_SPACE: f32 = 2.0 / 9.0;

/// Space between a function or big operator and an ordinary atom, and
/// after punctuation: TeX's thin space, 1/6 em.
pub const OP_SPACE: f32 = 1.0 / 6.0;

/// Interword space of `\ ` and `~`: Computer Modern's, 1/3 em
/// (ADR-T99-1).
pub const INTERWORD_SPACE: f32 = 1.0 / 3.0;

/// The spacing commands as typed and their widths in em (ADR-T99-1);
/// `\,` is a glyph of the symbol table instead (ADR-T36-1).
pub const KERNS: &[(&str, f32)] = &[
    ("\\:", 4.0 / 18.0),
    ("\\;", 5.0 / 18.0),
    ("\\!", -3.0 / 18.0),
    ("\\quad", 1.0),
    ("\\qquad", 2.0),
    ("\\ ", INTERWORD_SPACE),
    ("~", INTERWORD_SPACE),
];

/// The width in em of the spacing command typed as `source`; `None` when
/// it is not one of [`KERNS`].
#[must_use]
pub fn kern(source: &str) -> Option<f32> {
    KERNS.iter().find(|(s, _)| *s == source).map(|(_, w)| *w)
}

/// Room kept above and below a formula in its line.
pub const MATH_PAD: f32 = 0.03;

/// The math minus a `-` is drawn as.
pub const MINUS: char = '\u{2212}';

/// Deepest brace nesting parsed as groups; deeper braces are literal, so
/// parsing and layout never recurse without bound.
pub const MAX_DEPTH: usize = 32;

/// Most rows, and most columns, of an environment; past it the environment
/// is drawn as typed (ADR-T90-1).
pub const ENVIRONMENT_MAX_CELLS: usize = 64;

/// Space between the columns of an environment, in em.
pub const ARRAY_COLUMN_GAP: f32 = 1.0;

/// Least height of a row of an environment above its baseline, in em.
pub const ARRAY_ROW_ASCENT: f32 = 0.84;

/// Least depth of a row of an environment below its baseline, in em.
pub const ARRAY_ROW_DESCENT: f32 = 0.36;

/// How much `cases` stretches the least height of its rows, as amsmath's
/// `\arraystretch` does.
pub const CASES_STRETCH: f32 = 1.2;
/// Space on each side of the wider label of an extensible arrow, in em
/// of the arrow's size (ADR-T103-1).
pub const XARROW_PAD: f32 = 0.5;

/// Gap between an extensible arrow's ink and its labels: TeX's
/// `\bigopspacing1` (ADR-T103-1).
pub const XARROW_GAP: f32 = 0.111;

/// Number of rules of the hook of a stretched `\xhookrightarrow`
/// (ADR-T103-1).
pub const XARROW_HOOK_SEGMENTS: usize = 8;
/// The math metrics of a glyph of `face`; `None` for the text face or a
/// character the math font lacks.
pub(super) fn math_glyph_metrics(face: Face, ch: char) -> Option<math_metrics::GlyphMetrics> {
    match face {
        Face::Math => math_metrics::get(ch),
        Face::Text => None,
    }
}

/// Ascent and descent of text-face glyph `ch` per unit of scale.
#[must_use]
pub fn glyph_extent(ch: char) -> (f32, f32) {
    if "()[]{}|⌊⌋⌈⌉∑∏√".contains(ch) {
        (TALL_ASCENT, TALL_DESCENT)
    } else {
        (GLYPH_ASCENT, GLYPH_DESCENT)
    }
}
