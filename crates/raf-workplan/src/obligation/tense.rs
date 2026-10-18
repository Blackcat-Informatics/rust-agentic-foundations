// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The phrases that mark text as a proposal or a promise, and the scan that
//! finds them.
//!
//! Delivered and implementing capability documents state what the system
//! does. A sentence that says what it will do, what is planned or what is
//! proposed describes a different state. The scan is a fixed list of phrases
//! ([`PHRASES`]); a hit is a finding to rewrite, not a proof that the sentence
//! is wrong.
//!
//! # What is read
//!
//! The scan reads the prose lines of a body, which `raf-okf` has already
//! cleared of fenced code, HTML comments and code spans, and leaves out the
//! lines of PROPOSED blocks (they are proposals by definition) and of
//! generated blocks (they are derived). The text of a link destination, the
//! part of `[text](destination)` in parentheses, is not read.
//!
//! # How a phrase matches
//!
//! * Text is cut into words, each a run of letters, digits and `_`. A phrase
//!   matches whole words only, so `goodwill`, `William` and `willing` do not
//!   match `will`.
//! * Case is ignored, except that `PROPOSED` written entirely in capitals is
//!   the name of the marker and not wording; `WILL` and `NOT YET` are read.
//! * The words of a phrase are separated by exactly one space or by one line
//!   break (a paragraph wrapped in the middle of a phrase). A phrase does not
//!   continue over a blank line, punctuation, a code span or a block of
//!   another kind.
//! * Several hits of one phrase on one line are one finding.

use raf_okf::{Body, CapabilitySection};

use crate::work::{Work, add};

/// The phrases, each of one to three words.
///
/// The list is short on purpose; each phrase says that something is not true
/// yet:
///
/// * `will`: the future tense.
/// * `is planned`, `are planned`: a plan stated as a fact.
/// * `proposed`: text that belongs in a PROPOSED block.
/// * `to be implemented`: work not done.
/// * `in the future`: later work.
/// * `going to`: the colloquial future.
/// * `not yet`: a status that has not been reached.
pub const PHRASES: [&str; 8] = [
    "will",
    "is planned",
    "are planned",
    "proposed",
    "to be implemented",
    "in the future",
    "going to",
    "not yet",
];

/// A phrase found in the prose of a body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Hit {
    /// The 1-based line of the first word.
    pub(crate) line: usize,
    /// The phrase, from [`PHRASES`].
    pub(crate) phrase: &'static str,
    /// The section that holds the line, when it is one of the seven.
    pub(crate) section: Option<CapabilitySection>,
}

/// The phrases in the prose of `body`, in line order.
pub(crate) fn find(body: &Body) -> Vec<Hit> {
    let lines = body
        .prose_lines()
        .iter()
        .filter(|line| !line.in_proposed() && !line.in_generated())
        .map(|line| {
            (
                line.line(),
                line.text(),
                line.section().and_then(CapabilitySection::from_slug),
            )
        });
    scan(lines)
        .into_iter()
        .map(|(line, phrase, section)| Hit {
            line,
            phrase,
            section,
        })
        .collect()
}

/// One word of the text.
struct Word<'a, T> {
    text: &'a str,
    line: usize,
    tag: T,
    /// Whether only one space or one line break separates the word from the
    /// one before it.
    joins_previous: bool,
}

/// The phrases in `lines` (1-based line number, text, a tag carried to the
/// hit), in order. The cost is linear in the size of the text.
pub(crate) fn scan<'a, T: Copy>(
    lines: impl IntoIterator<Item = (usize, &'a str, T)>,
) -> Vec<(usize, &'static str, T)> {
    let mut words: Vec<Word<'a, T>> = Vec::new();
    // The line of the last word read and whether the line ends after it.
    let mut open_end: Option<usize> = None;
    for (line, text, tag) in lines {
        let spans = word_spans(text);
        let (Some(first), Some(last)) = (spans.first(), spans.last()) else {
            continue;
        };
        let leads = text[..first.0].trim().is_empty();
        let trails = text[last.1..].trim().is_empty();
        let mut previous_end: Option<usize> = None;
        for (start, end) in spans {
            let joins_previous = match previous_end {
                Some(before) => matches!(&text[before..start], " "),
                None => leads && open_end.is_some_and(|prior| prior + 1 == line),
            };
            words.push(Word {
                text: &text[start..end],
                line,
                tag,
                joins_previous,
            });
            previous_end = Some(end);
        }
        open_end = trails.then_some(line);
    }
    let mut hits = Vec::new();
    // The phrases already reported for the line being read: one finding for
    // a phrase and a line, however often the line says it.
    let mut reported = [false; PHRASES.len()];
    let mut reported_line = 0;
    for at in 0..words.len() {
        let line = words[at].line;
        if line != reported_line {
            reported = [false; PHRASES.len()];
            reported_line = line;
        }
        for (index, phrase) in PHRASES.into_iter().enumerate() {
            if !reported[index] && matches_at(&words, at, phrase) {
                reported[index] = true;
                hits.push((line, phrase, words[at].tag));
            }
        }
    }
    hits
}

/// The phrase that is also the name of the PROPOSED marker, which is not
/// wording when it is written in capitals.
const MARKER_WORD: &str = "proposed";

/// Whether the words from `at` spell `phrase`. The marker word counts only in
/// the case of prose.
fn matches_at<T>(words: &[Word<'_, T>], at: usize, phrase: &str) -> bool {
    add(Work::PhraseTests, 1);
    let mut spelled = false;
    for (offset, part) in phrase.split(' ').enumerate() {
        let Some(word) = words.get(at + offset) else {
            return false;
        };
        if !word.text.eq_ignore_ascii_case(part) || (offset > 0 && !word.joins_previous) {
            return false;
        }
        spelled |= word.text.chars().any(char::is_lowercase);
    }
    spelled || phrase != MARKER_WORD
}

/// The byte ranges of the words of a line, leaving out link destinations.
fn word_spans(text: &str) -> Vec<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut spans = Vec::new();
    let mut word: Option<usize> = None;
    let mut at = 0;
    while at < text.len() {
        let Some(c) = text[at..].chars().next() else {
            break;
        };
        let next = at + c.len_utf8();
        add(Work::ScanBytes, c.len_utf8());
        if c.is_alphanumeric() || c == '_' {
            word.get_or_insert(at);
        } else {
            if let Some(start) = word.take() {
                spans.push((start, at));
            }
            if c == ']' && bytes.get(next) == Some(&b'(') {
                // The destination of a link is a path or an address, not
                // wording: skip to its closing parenthesis.
                at = text[next..]
                    .find(')')
                    .map_or(text.len(), |close| next + close + 1);
                add(Work::ScanBytes, at - next);
                continue;
            }
        }
        at = next;
    }
    if let Some(start) = word {
        spans.push((start, text.len()));
    }
    spans
}
