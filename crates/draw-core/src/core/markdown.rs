//! Converts Markdown notes into a [`DocFile`] (`.draw` document).
//!
//! Unsupported formatting (bold, italic, strikethrough, inline code,
//! images, link targets, HTML tags, fenced code blocks) is stripped or
//! ignored; headings map to distinct text sizes, `$…$` / `$$…$$` TeX math
//! is preserved for [`text_layout`], and `---`
//! thematic breaks split pages (up to [`PAGE_COUNT`]).

use crate::core::command::Command;
use crate::core::editor::{Editor, PAGE_COUNT};
use crate::core::file::DocFile;
use crate::core::geom::Vec2;
use crate::core::palette::ColorId;
use crate::core::shape::{Shape, Style};
use crate::core::snapshot::MAX_TEXT_SIZE_PX;
use crate::core::text_layout;
use crate::core::tools::text::clean_paste;

/// Left margin of generated blocks, in world units.
const MARGIN_X: f32 = 80.0;
/// Top margin of each page, in world units.
const MARGIN_Y: f32 = 80.0;
/// Vertical gap between consecutive blocks, in world units.
const BLOCK_GAP: f32 = 14.0;
/// Extra vertical space before a heading (except at the top of a page).
const HEADING_TOP_GAP: f32 = 10.0;
/// Body text size in world units (`text_size_px(3.0)`).
pub const BODY_SIZE_PX: f32 = 18.0;

/// Font size in world units for ATX heading level `1..=6`.
#[must_use]
pub fn heading_size(level: usize) -> f32 {
    let size: f32 = match level {
        1 => 40.0,
        2 => 34.0,
        3 => 28.0,
        4 => 24.0,
        5 => 21.0,
        _ => 19.0,
    };
    size.min(MAX_TEXT_SIZE_PX)
}

#[derive(Debug, Clone, PartialEq)]
enum Item {
    Block {
        text: String,
        size: f32,
        is_heading: bool,
    },
    PageBreak,
}

/// Converts Markdown `source` into a validated [`DocFile`].
#[must_use]
pub fn markdown_to_doc(source: &str) -> DocFile {
    let items = parse_items(source);
    let mut editor = Editor::new();
    let mut page = 0usize;
    let mut y = MARGIN_Y;
    let mut page_has_content = false;

    for item in items {
        match item {
            Item::PageBreak => {
                if page_has_content && page + 1 < PAGE_COUNT {
                    page += 1;
                    editor.apply(Command::GoToPage(page));
                    y = MARGIN_Y;
                    page_has_content = false;
                }
            }
            Item::Block {
                text,
                size,
                is_heading,
            } => {
                let cleaned = clean_paste(&text);
                if cleaned.trim().is_empty() {
                    continue;
                }
                if is_heading && page_has_content {
                    y += HEADING_TOP_GAP;
                }
                let origin = Vec2::new(MARGIN_X, y);
                let bounds = text_layout::bounds(origin, &cleaned, size);
                let shape = Shape::Text {
                    origin,
                    text: cleaned,
                    size,
                    style: Style {
                        color: ColorId::INK,
                        width: 3.0,
                    },
                };
                editor.insert_shape(shape);
                y += bounds.height().max(size) + BLOCK_GAP;
                page_has_content = true;
            }
        }
    }

    if page != 0 {
        editor.apply(Command::GoToPage(0));
    }
    editor.file()
}

fn flush_para(items: &mut Vec<Item>, paragraph: &mut Vec<String>) {
    if !paragraph.is_empty() {
        items.push(Item::Block {
            text: paragraph.join("\n"),
            size: BODY_SIZE_PX,
            is_heading: false,
        });
        paragraph.clear();
    }
}

fn push_math_block(items: &mut Vec<Item>, lines: &[String]) {
    let joined = lines.join(" ").trim().to_owned();
    if !joined.is_empty() {
        items.push(Item::Block {
            text: format!("${joined}$"),
            size: BODY_SIZE_PX,
            is_heading: false,
        });
    }
}

fn parse_items(source: &str) -> Vec<Item> {
    let normalized = source.replace("\r\n", "\n").replace('\r', "\n");
    let mut items = Vec::new();
    let mut paragraph: Vec<String> = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    let mut math_block: Option<Vec<String>> = None;
    let mut in_html_comment = false;

    for raw_line in normalized.split('\n') {
        let trimmed = raw_line.trim();

        if in_html_comment {
            if trimmed.contains("-->") {
                in_html_comment = false;
            }
            continue;
        }

        if let Some((fence_ch, fence_len)) = fence {
            if is_closing_fence(trimmed, fence_ch, fence_len) {
                fence = None;
            }
            continue;
        }

        if let Some(lines) = &mut math_block {
            if trimmed == "$$" {
                push_math_block(&mut items, lines);
                math_block = None;
            } else if !trimmed.is_empty() {
                lines.push(trimmed.to_owned());
            }
            continue;
        }

        if let Some(opened) = opening_fence(trimmed) {
            flush_para(&mut items, &mut paragraph);
            fence = Some(opened);
            continue;
        }

        if trimmed.starts_with("<!--") {
            flush_para(&mut items, &mut paragraph);
            if !trimmed.contains("-->") {
                in_html_comment = true;
            }
            continue;
        }

        if trimmed == "$$" {
            flush_para(&mut items, &mut paragraph);
            math_block = Some(Vec::new());
            continue;
        }

        if is_thematic_break(trimmed) {
            flush_para(&mut items, &mut paragraph);
            items.push(Item::PageBreak);
            continue;
        }

        if trimmed.is_empty() || is_link_definition(trimmed) || is_table_separator(trimmed) {
            if trimmed.is_empty() {
                flush_para(&mut items, &mut paragraph);
            }
            continue;
        }

        if let Some((level, title)) = parse_atx_heading(trimmed) {
            flush_para(&mut items, &mut paragraph);
            let clean = strip_inline(title).trim().to_owned();
            if !clean.is_empty() {
                items.push(Item::Block {
                    text: clean,
                    size: heading_size(level),
                    is_heading: true,
                });
            }
            continue;
        }

        let line = strip_blockquote_prefix(raw_line);
        let clean = strip_inline(line).trim_end().to_owned();
        if !clean.trim().is_empty() {
            paragraph.push(clean);
        }
    }

    if let Some(lines) = math_block {
        push_math_block(&mut items, &lines);
    }
    flush_para(&mut items, &mut paragraph);
    items
}

fn opening_fence(trimmed: &str) -> Option<(char, usize)> {
    let first = trimmed.chars().next()?;
    if first != '`' && first != '~' {
        return None;
    }
    let len = trimmed.chars().take_while(|&c| c == first).count();
    (len >= 3).then_some((first, len))
}

fn is_closing_fence(trimmed: &str, ch: char, min_len: usize) -> bool {
    let len = trimmed.chars().take_while(|&c| c == ch).count();
    len >= min_len && trimmed[len * ch.len_utf8()..].trim().is_empty()
}

fn is_thematic_break(trimmed: &str) -> bool {
    let mut kind: Option<char> = None;
    let mut count = 0usize;
    for ch in trimmed.chars() {
        if ch == ' ' || ch == '\t' {
            continue;
        }
        if matches!(ch, '-' | '*' | '_') && kind.is_none_or(|k| k == ch) {
            kind = Some(ch);
            count += 1;
        } else {
            return false;
        }
    }
    count >= 3
}

fn is_link_definition(trimmed: &str) -> bool {
    let Some(rest) = trimmed.strip_prefix('[') else {
        return false;
    };
    let Some((_, after)) = rest.split_once("]:") else {
        return false;
    };
    !after.trim().is_empty()
}

fn is_table_separator(trimmed: &str) -> bool {
    trimmed.contains('-')
        && trimmed.contains('|')
        && trimmed
            .chars()
            .all(|c| matches!(c, '|' | '-' | ':' | ' ' | '\t'))
}

fn parse_atx_heading(trimmed: &str) -> Option<(usize, &str)> {
    let level = trimmed.bytes().take_while(|&b| b == b'#').count();
    if !(1..=6).contains(&level) {
        return None;
    }
    let rest = &trimmed[level..];
    if !rest.is_empty() && !rest.starts_with([' ', '\t']) {
        return None;
    }
    let mut content = rest.trim();
    let stripped_end = content.trim_end_matches('#');
    if stripped_end.len() < content.len()
        && (stripped_end.is_empty() || stripped_end.ends_with([' ', '\t']))
    {
        content = stripped_end.trim_end();
    }
    Some((level, content))
}

fn strip_blockquote_prefix(line: &str) -> &str {
    let mut s = line.trim_start_matches([' ', '\t']);
    while let Some(rest) = s.strip_prefix('>') {
        s = rest.strip_prefix(' ').unwrap_or(rest);
        s = s.trim_start_matches([' ', '\t']);
    }
    if s.len() < line.len() { s } else { line }
}

/// Strips unsupported inline Markdown (`**bold**`, `*italic*`, `` `code` ``,
/// `~~strike~~`, `![img](url)`, `[text](url)`, `<tag>`) while keeping `$…$`
/// TeX math untouched and converting inline `$$…$$` to `$…$`.
#[must_use]
pub fn strip_inline(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Escaped character: keep `\$` for TeX literal `$`, unwrap other punctuation.
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            let next = bytes[i + 1];
            if next == b'$' {
                out.push_str("\\$");
                i += 2;
                continue;
            }
            if next.is_ascii_punctuation() {
                out.push(char::from(next));
                i += 2;
                continue;
            }
        }

        // Display/inline TeX math: preserve inner TeX verbatim.
        if bytes[i] == b'$' {
            if bytes.get(i + 1) == Some(&b'$')
                && let Some(end) = find_unescaped(&input[i + 2..], "$$")
            {
                let inner = input[i + 2..i + 2 + end].trim();
                out.push('$');
                out.push_str(inner);
                out.push('$');
                i = i + 2 + end + 2;
                continue;
            }
            if let Some(end) = find_unescaped(&input[i + 1..], "$") {
                out.push_str(&input[i..=i + 1 + end]);
                i = i + 1 + end + 1;
                continue;
            }
        }

        // Inline code span: unwrap backtick delimiters.
        if bytes[i] == b'`' {
            let ticks = bytes[i..].iter().take_while(|&&b| b == b'`').count();
            let delim = &input[i..i + ticks];
            if let Some(end) = input[i + ticks..].find(delim) {
                let code = &input[i + ticks..i + ticks + end];
                out.push_str(code.trim());
                i = i + ticks + end + ticks;
                continue;
            }
        }

        // Image `![alt](url)` or `![alt][ref]`: drop completely.
        if bytes[i] == b'!'
            && bytes.get(i + 1) == Some(&b'[')
            && let Some(next_i) = parse_link_end(input, i + 1).map(|(_, end)| end)
        {
            i = next_i;
            continue;
        }

        // Link `[text](url)` or `[text][ref]`: keep `text` without target.
        if bytes[i] == b'['
            && let Some((label, next_i)) = parse_link_end(input, i)
        {
            out.push_str(&strip_inline(label));
            i = next_i;
            continue;
        }

        // Inline HTML tag `<...>`: drop.
        if bytes[i] == b'<'
            && let Some(end) = input[i + 1..].find('>')
            && is_html_tag(&input[i + 1..i + 1 + end])
        {
            i = i + 1 + end + 1;
            continue;
        }

        // Multi-char emphasis/strikethrough delimiters.
        if matches!(bytes.get(i..i + 2), Some(b"**" | b"__" | b"~~")) {
            i += 2;
            continue;
        }

        // Single `*` or `_` emphasis: keep intraword `_` (like `snake_case`).
        if bytes[i] == b'*' {
            i += 1;
            continue;
        }
        if bytes[i] == b'_' {
            let prev_alnum = i > 0 && bytes[i - 1].is_ascii_alphanumeric();
            let next_alnum = bytes.get(i + 1).is_some_and(u8::is_ascii_alphanumeric);
            if prev_alnum && next_alnum {
                out.push('_');
            }
            i += 1;
            continue;
        }

        let ch = input[i..].chars().next().unwrap_or_default();
        out.push(ch);
        i += ch.len_utf8();
    }

    out
}

fn find_unescaped(haystack: &str, needle: &str) -> Option<usize> {
    let mut offset = 0;
    while let Some(pos) = haystack[offset..].find(needle) {
        let abs = offset + pos;
        let backslashes = haystack[..abs]
            .bytes()
            .rev()
            .take_while(|&b| b == b'\\')
            .count();
        if backslashes.is_multiple_of(2) {
            return Some(abs);
        }
        offset = abs + needle.len();
    }
    None
}

fn parse_link_end(input: &str, open_bracket: usize) -> Option<(&str, usize)> {
    let bytes = input.as_bytes();
    let mut i = open_bracket + 1;
    let mut depth = 1usize;
    while i < bytes.len() && depth > 0 {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        match bytes[i] {
            b'[' => depth += 1,
            b']' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    if depth != 0 {
        return None;
    }
    let close_bracket = i - 1;
    let label = &input[open_bracket + 1..close_bracket];
    let after = input.get(i..)?;
    if after.starts_with('(') {
        let mut j = i + 1;
        let mut parens = 1usize;
        while j < bytes.len() && parens > 0 {
            if bytes[j] == b'\\' {
                j += 2;
                continue;
            }
            match bytes[j] {
                b'(' => parens += 1,
                b')' => parens -= 1,
                _ => {}
            }
            j += 1;
        }
        (parens == 0).then_some((label, j))
    } else if let Some(rest) = after.strip_prefix('[') {
        let close = rest.find(']')?;
        Some((label, i + 1 + close + 1))
    } else {
        None
    }
}

fn is_html_tag(inner: &str) -> bool {
    let s = inner.trim_start_matches('/');
    s.chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '!')
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn headings_map_to_distinct_sizes_and_keep_tex() {
        let md =
            "# Title\n\n## Subtitle\n\nSome **bold** and *italic* with $x_i^2 + \\frac{a}{b}$.\n";
        let doc = markdown_to_doc(md);
        let Ok(parsed) = DocFile::parse(&doc.to_text()) else {
            panic!("generated doc must parse");
        };
        let shapes: Vec<_> = parsed.0.document().shapes().map(|(_, s)| s).collect();

        assert_eq!(shapes.len(), 3);
        let Shape::Text {
            text: t0, size: s0, ..
        } = shapes[0]
        else {
            panic!("expected text shape");
        };
        let Shape::Text {
            text: t1, size: s1, ..
        } = shapes[1]
        else {
            panic!("expected text shape");
        };
        let Shape::Text {
            text: t2, size: s2, ..
        } = shapes[2]
        else {
            panic!("expected text shape");
        };

        assert_eq!(t0, "Title");
        assert_eq!(t1, "Subtitle");
        assert_eq!(t2, "Some bold and italic with $x_i^2 + \\frac{a}{b}$.");
        assert!(s0 > s1);
        assert!(s1 > s2);
    }

    #[test]
    fn unsupported_markdown_is_ignored_or_unwrapped() {
        let md = "\
![diagram](img.png)

Check [this **link**](https://example.com) and `code` and ~~old~~.

```rust
let ignored = 1;
```

<!-- hidden comment -->

> Quoted *text* with `a_b`.
";
        let doc = markdown_to_doc(md);
        let shapes: Vec<_> = doc.0.document().shapes().map(|(_, s)| s).collect();
        let texts: Vec<&str> = shapes
            .iter()
            .filter_map(|s| match s {
                Shape::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect();

        assert_eq!(
            texts,
            ["Check this link and code and old.", "Quoted text with a_b.",]
        );
    }

    #[test]
    fn display_math_becomes_inline_tex_and_thematic_break_splits_pages() {
        let md = "# Page 1\n\n$$\n\\sum_{i=1}^n i\n$$\n\n---\n\n# Page 2\n";
        let doc = markdown_to_doc(md);
        let text = doc.to_text();
        let Ok(mut parsed) = DocFile::parse(&text) else {
            panic!("generated doc must parse");
        };

        let p0: Vec<_> = parsed
            .0
            .document()
            .shapes()
            .filter_map(|(_, s)| match s {
                Shape::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(p0, ["Page 1", "$\\sum_{i=1}^n i$"]);

        assert!(parsed.0.apply(Command::GoToPage(1)));
        let p1: Vec<_> = parsed
            .0
            .document()
            .shapes()
            .filter_map(|(_, s)| match s {
                Shape::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(p1, ["Page 2"]);
    }

    proptest! {
        #[test]
        fn any_markdown_produces_a_valid_doc_file(md in ".*") {
            let doc = markdown_to_doc(&md);
            let encoded = doc.to_text();
            prop_assert!(DocFile::parse(&encoded).is_ok());
        }
    }
}
