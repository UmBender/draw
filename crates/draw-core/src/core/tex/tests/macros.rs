//! Tests of the TeX subset.

use super::*;

// ---- T112: user macros (ADR-T112-1) ----

fn table(blocks: &[&str]) -> MacroTable {
    MacroTable::from_texts(blocks.iter().copied())
}

fn expanded(text: &str, table: &MacroTable) -> String {
    expand_macros(text, table).source
}

#[test]
fn newcommand_without_args_expands_in_formula() {
    let t = table(&["$\\newcommand{\\R}{\\mathbb{R}}$"]);
    assert_eq!(expanded("$x\\in\\R$", &t), "$x\\in\\mathbb{R}$");
    assert_eq!(expanded("$$\\R$$", &t), "$$\\mathbb{R}$$");
}

#[test]
fn newcommand_with_args_fills_slots() {
    let t = table(&[
        "$\\newcommand{\\modul}[1]{\\left|#1\\right|}$",
        "$\\newcommand{\\pair}[2]{#1+#2}$",
    ]);
    assert_eq!(expanded("$\\modul{x}$", &t), "$\\left|x\\right|$");
    assert_eq!(expanded("$\\pair{a}b$", &t), "$a+b$");
    assert_eq!(expanded("$\\pair{a b}{c}$", &t), "$a b+c$");
}

#[test]
fn macro_with_too_few_arguments_is_left_as_typed() {
    let t = table(&["$\\newcommand{\\modul}[1]{|#1|}$"]);
    assert_eq!(expanded("$\\modul$", &t), "$\\modul$");
}

#[test]
fn definition_draws_nothing() {
    let t = table(&["$\\newcommand{\\R}{x}$"]);
    assert_eq!(expanded("$\\newcommand{\\R}{x}$", &t), "");
    assert_eq!(expanded("a $\\newcommand{\\R}{x}$ b", &t), "a  b");
    assert_eq!(expanded("$\\newcommand{\\R}{x} y$", &t), "$ y$");
}

#[test]
fn macros_outside_formulas_are_plain_text() {
    let t = table(&["$\\newcommand{\\R}{x}$"]);
    assert_eq!(expanded("\\R $\\R$", &t), "\\R $x$");
}

#[test]
fn table_higher_block_wins() {
    let t = table(&["$\\newcommand{\\a}{1}$", "$\\newcommand{\\a}{2}$"]);
    assert_eq!(expanded("$\\a$", &t), "$2$");
}

#[test]
fn renewcommand_defines_like_newcommand() {
    let t = table(&["$\\newcommand{\\a}{1}$", "$\\renewcommand{\\a}{3}$"]);
    assert_eq!(expanded("$\\a$", &t), "$3$");
    assert_eq!(
        table(&["$\\renewcommand{\\b}{4}$"])
            .get("b")
            .map(|m| m.params),
        Some(0)
    );
}

#[test]
fn builtin_cannot_be_redefined() {
    let t = table(&[
        "$\\newcommand{\\alpha}{x}$",
        "$\\newcommand{\\frac}[2]{y}$",
        "$\\newcommand{\\choose}{z}$",
    ]);
    assert_eq!(
        expanded("$\\alpha\\frac{1}{2}$", &t),
        "$\\alpha\\frac{1}{2}$"
    );
    assert!(t.get("alpha").is_none() && t.get("frac").is_none());
    assert_eq!(t.ignored(), ["alpha", "frac", "choose"]);
}

#[test]
fn self_recursive_macro_draws_literal_source() {
    let t = table(&["$\\newcommand{\\a}{\\a\\a}$"]);
    assert_eq!(expanded("$x\\a$", &t), "$x\\a$");
}

#[test]
fn exponential_macro_is_bounded() {
    let t = table(&[
        "$\\newcommand{\\x}{\\y\\y\\y\\y}\\newcommand{\\y}{\\z\\z\\z\\z}$",
        "$\\newcommand{\\z}{\\w\\w\\w\\w}\\newcommand{\\w}{\\v\\v\\v\\v}$",
        "$\\newcommand{\\v}{\\u\\u\\u\\u}\\newcommand{\\u}{\\t\\t\\t\\t}$",
        "$\\newcommand{\\t}{\\s\\s\\s\\s}\\newcommand{\\s}{ab}$",
    ]);
    assert_eq!(expanded("$\\x$", &t), "$\\x$");
}

#[test]
fn expansion_reports_user_names_it_met() {
    let t = table(&["$\\newcommand{\\a}{\\b}\\newcommand{\\b}{1}$"]);
    let e = expand_macros("$\\a\\alpha\\q$", &t);
    assert_eq!(e.source, "$1\\alpha\\q$");
    let mut names = e.names;
    names.sort();
    assert_eq!(names, ["a", "b", "q"]);
}

#[test]
fn print_parse_unchanged_by_macros() {
    let text = "a $\\newcommand{\\modul}[1]{\\left|#1\\right|}$ $\\modul{x}$";
    assert_eq!(print(&parse(text)), text);
}

#[test]
fn macro_hash_changes_with_definition() {
    let a = table(&["$\\newcommand{\\a}{1}$"]);
    let b = table(&["$\\newcommand{\\a}{2}$"]);
    assert_eq!(a.hash_of("zzz"), 0);
    assert_ne!(a.hash_of("a"), 0);
    assert_ne!(a.hash_of("a"), b.hash_of("a"));
}

proptest! {
    #[test]
    fn expand_macros_never_panics_and_is_bounded(
        text in "[\\\\$a-z#0-9{}\\[\\] ]{0,60}",
    ) {
        let t = table(&[&text, "$\\newcommand{\\a}[1]{\\a#1\\a}$"]);
        let e = expand_macros(&text, &t);
        prop_assert!(e.source.chars().count() <= text.chars().count() + MACRO_SIZE);
    }
}
