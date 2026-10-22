// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Text from a document, a record or a commit, made safe to print: each
//! character that changes how a terminal shows the text around it is written
//! as its `\u{..}` escape. The history and coverage tables escape their cells
//! with it, and the command line escapes the text output of the commands that
//! print document text, so no text can move the cursor of a terminal, set its
//! title, clear it, ring it, or reorder how a line reads.

use std::fmt::Write as _;

/// Whether `c` changes how a terminal shows the text around it: a control
/// character, or a bidirectional formatting character (the marks U+061C,
/// U+200E and U+200F, the embeddings and overrides U+202A to U+202E, and the
/// isolates U+2066 to U+2069).
fn hidden(c: char) -> bool {
    c.is_control() || purrdf_text::unicode::is_bidi_control(c)
}

/// `text` with each character that `keep` does not keep and that changes how
/// a terminal shows the text around it as its `\u{..}` escape.
fn escaped(text: &str, keep: impl Fn(char) -> bool) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if hidden(c) && !keep(c) {
            let _ = write!(out, "\\u{{{:x}}}", u32::from(c));
        } else {
            out.push(c);
        }
    }
    out
}

/// `text` on one line: every control character, line feeds and tabs
/// included, and every bidirectional formatting character as its escape.
pub(crate) fn shown(text: &str) -> String {
    escaped(text, |_| false)
}

/// `text` as lines: each control character other than the line feed and the
/// tab, a carriage return and an escape included, and each bidirectional
/// formatting character as its `\u{..}` escape, so the text keeps its lines
/// and indentation and nothing else of it reaches the terminal as a control.
/// Other text, non-ASCII letters and symbols included, is kept as it is.
///
/// ```rust
/// use raf_workplan::escape::shown_lines;
///
/// let text = "It holds.\u{1b}]0;title\u{7}\n\tSo \u{202e}does this.\n";
/// assert_eq!(
///     shown_lines(text),
///     "It holds.\\u{1b}]0;title\\u{7}\n\tSo \\u{202e}does this.\n"
/// );
/// ```
#[must_use]
pub fn shown_lines(text: &str) -> String {
    escaped(text, |c| c == '\n' || c == '\t')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_control_and_bidirectional_formatting_character_is_escaped() {
        assert_eq!(shown("x\u{1b}[31m\u{1f}"), "x\\u{1b}[31m\\u{1f}");
        assert_eq!(shown("one\ntwo\tthree"), "one\\u{a}two\\u{9}three");
        assert_eq!(
            shown("a\u{202e}b\u{2066}c\u{200f}\u{61c}"),
            "a\\u{202e}b\\u{2066}c\\u{200f}\\u{61c}"
        );
        for c in (0u32..0x20).chain(0x7f..0xa0).filter_map(char::from_u32) {
            assert!(
                !shown(&c.to_string()).chars().any(hidden),
                "{:x}",
                u32::from(c)
            );
        }
        // The neighbour: other non-ASCII text is kept.
        assert_eq!(shown("été \u{2014} 名前"), "été \u{2014} 名前");
    }

    #[test]
    fn lines_keep_their_line_feeds_and_tabs_and_lose_every_other_control() {
        assert_eq!(
            shown_lines("A\u{1b}[2J\u{1b}]0;PWNED\u{7}\n\tB\r\n\u{202e}C\n"),
            "A\\u{1b}[2J\\u{1b}]0;PWNED\\u{7}\n\tB\\u{d}\n\\u{202e}C\n"
        );
        assert_eq!(shown_lines("plain\n  text\n"), "plain\n  text\n");
        assert_eq!(shown_lines("été \u{2014} 名前"), "été \u{2014} 名前");
    }
}
