// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The one pattern set for issue and pull request numbers and for
//! development-process wording.
//!
//! The patterns are a small hand-made matcher, so no pattern-matching crate
//! is needed. [`banned_in_line`] judges one line of text. The repository
//! gates and the `issue-reference` rule of [`Workplan::check`](crate::Workplan::check)
//! both call it, so there is a single definition of what such a reference is.
//!
//! Phrases below are written with a `[ ]` character class in place of a
//! space. That keeps every pattern from matching its own text, and `unclass`
//! turns the class back into a space for matching.

use std::fmt;

use raf_okf::Location;

use crate::Finding;
use crate::finding::kind;

/// Why a line is banned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Banned {
    /// An issue or pull request number or link.
    IssueNumber,
    /// Wording that refers to the development process that produced a file.
    ProcessWording,
}

impl Banned {
    /// The reason, as a phrase.
    #[must_use]
    pub fn description(self) -> &'static str {
        match self {
            Self::IssueNumber => "issue or pull request number",
            Self::ProcessWording => "development-process wording",
        }
    }
}

impl fmt::Display for Banned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.description())
    }
}

/// Phrases that mark development-process wording (matched case-insensitively).
const PROCESS_PHRASES: [&str; 3] = ["this[ ]issue", "per[ ]the[ ]plan", "this[ ]branch"];

/// A prefix that must be followed by digits to be an issue or change number.
struct NumberedForm {
    /// The literal text before the digits, with `[ ]` standing for a space.
    prefix: &'static str,
    /// Match without regard to letter case.
    ignore_case: bool,
    /// Require a non-word character (or the line start) before the prefix.
    word_start: bool,
    /// Report as process wording rather than as an issue number.
    process: bool,
}

const NUMBERED_FORMS: [NumberedForm; 7] = [
    NumberedForm {
        prefix: "/issues/",
        ignore_case: false,
        word_start: false,
        process: false,
    },
    NumberedForm {
        prefix: "/pull/",
        ignore_case: false,
        word_start: false,
        process: false,
    },
    NumberedForm {
        prefix: "CHANGE[-]",
        ignore_case: false,
        word_start: false,
        process: false,
    },
    NumberedForm {
        prefix: "issue[ ]",
        ignore_case: true,
        word_start: true,
        process: false,
    },
    NumberedForm {
        prefix: "GH[-]",
        ignore_case: true,
        word_start: true,
        process: false,
    },
    NumberedForm {
        prefix: "pull[ ]request[ ]",
        ignore_case: true,
        word_start: true,
        process: false,
    },
    NumberedForm {
        prefix: "task[ ]",
        ignore_case: true,
        word_start: false,
        process: true,
    },
];

/// Replace each `[ ]` and `[-]` character class with its literal character.
fn unclass(pattern: &str) -> String {
    pattern.replace("[ ]", " ").replace("[-]", "-")
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The character before byte offset `index`, if any.
fn char_before(line: &str, index: usize) -> Option<char> {
    line.get(..index).and_then(|s| s.chars().next_back())
}

/// Byte offsets where `needle` occurs in `line`, optionally ignoring case.
fn find_all(line: &str, needle: &str, ignore_case: bool) -> Vec<usize> {
    if ignore_case {
        // Lowercasing can change byte lengths for non-ASCII text, so fold
        // ASCII only; every needle here is ASCII.
        line.to_ascii_lowercase()
            .match_indices(&needle.to_ascii_lowercase())
            .map(|(index, _)| index)
            .collect()
    } else {
        line.match_indices(needle).map(|(index, _)| index).collect()
    }
}

/// True when a digit directly follows an occurrence of the form's prefix.
fn numbered_form_hit(line: &str, form: &NumberedForm) -> bool {
    let needle = unclass(form.prefix);
    find_all(line, &needle, form.ignore_case)
        .into_iter()
        .any(|index| {
            if form.word_start && char_before(line, index).is_some_and(is_word) {
                return false;
            }
            line.get(index + needle.len()..)
                .and_then(|rest| rest.chars().next())
                .is_some_and(|c| c.is_ascii_digit())
        })
}

/// A `PR` label followed by a number, with or without a space or hash, and
/// with a non-word character before `PR`.
fn pull_request_label(line: &str) -> bool {
    line.match_indices("PR").any(|(index, _)| {
        if char_before(line, index).is_some_and(is_word) {
            return false;
        }
        let rest = line.get(index + 2..).unwrap_or_default();
        let rest = rest.strip_prefix(' ').unwrap_or(rest);
        let rest = rest.strip_prefix('#').unwrap_or(rest);
        rest.chars().next().is_some_and(|c| c.is_ascii_digit())
    })
}

/// An acceptance-criterion label: `AC` and digits, delimited by non-word text.
fn acceptance_label(lower: &str) -> bool {
    lower.match_indices("ac").any(|(index, _)| {
        let rest = lower.get(index + 2..).unwrap_or_default();
        let digits = rest.chars().take_while(char::is_ascii_digit).count();
        digits > 0
            && !char_before(lower, index).is_some_and(is_word)
            && !rest.chars().nth(digits).is_some_and(is_word)
    })
}

/// "the plan" followed by any one character and `s` (a possessive).
fn plan_possessive(lower: &str) -> bool {
    let needle = unclass("the[ ]plan");
    lower.match_indices(&needle).any(|(index, _)| {
        let mut rest = lower
            .get(index + needle.len()..)
            .unwrap_or_default()
            .chars();
        rest.next().is_some() && rest.next() == Some('s')
    })
}

/// A `#` followed by one to five decimal digits that reads as a reference.
///
/// The character before `#` must not be `&` or `#`; a word character before
/// it (a cross-repository reference, or a keyword with no space) does not
/// spare the token. The run of word characters after `#` must be one to five
/// digits, so a letter a-f (a hex colour such as `#1f2`) or a longer run
/// (`#123456`) is spared. A three- or four-digit token is also spared when it
/// directly follows `"` or `'`, which is how a colour attribute value is
/// written in markup. After `:` or a space only a token with a hex letter is
/// spared, so an all-digit token after a keyword and a colon, or an all-digit
/// CSS colour, fails closed; write such a colour with letters or as `#112233`.
fn hash_number(line: &str) -> bool {
    line.match_indices('#').any(|(index, _)| {
        let before = char_before(line, index);
        if before.is_some_and(|c| c == '&' || c == '#') {
            return false;
        }
        let run: String = line
            .get(index + 1..)
            .unwrap_or_default()
            .chars()
            .take_while(|c| is_word(*c))
            .collect();
        let numeric = (1..=5).contains(&run.len()) && run.chars().all(|c| c.is_ascii_digit());
        let colour_context = matches!(before, Some('"' | '\''));
        numeric && !((3..=4).contains(&run.len()) && colour_context)
    })
}

/// Why a line is banned, or `None` when it is clean.
///
/// A line is banned when it holds an issue or pull request number (a hash and
/// up to five digits, an issue or pull request label with a number, a tracker
/// link) or wording that refers to the development process (a task number, an
/// acceptance label, a reference to a plan or a branch). A colour written as a
/// hexadecimal triple and numbers in prose are not references.
#[must_use]
pub fn banned_in_line(line: &str) -> Option<Banned> {
    if hash_number(line)
        || pull_request_label(line)
        || NUMBERED_FORMS
            .iter()
            .filter(|form| !form.process)
            .any(|form| numbered_form_hit(line, form))
    {
        return Some(Banned::IssueNumber);
    }
    let lower = line.to_lowercase();
    let process = PROCESS_PHRASES
        .iter()
        .any(|phrase| lower.contains(&unclass(phrase)))
        || plan_possessive(&lower)
        || NUMBERED_FORMS
            .iter()
            .filter(|form| form.process)
            .any(|form| numbered_form_hit(&lower, form))
        || acceptance_label(&lower);
    process.then_some(Banned::ProcessWording)
}

/// The `issue-reference` findings of the text of the file `file`: one for each
/// line [`banned_in_line`] bans.
pub(crate) fn scan(file: &str, text: &str, out: &mut Vec<Finding>) {
    for (index, line) in text.lines().enumerate() {
        if let Some(banned) = banned_in_line(line) {
            out.push(Finding::new(
                kind::ISSUE_REFERENCE,
                file,
                Location::Line(index + 1),
                format!("{banned} is banned"),
                "describe the system, not the tracker or the process that produced the file; remove the reference",
            ));
        }
    }
}
