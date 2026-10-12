// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Shared support for the repository-gate test binaries.
//!
//! This module is the single definition site for two things:
//!
//! * the issue-reference and process-wording patterns, written as a small
//!   hand-made matcher because the root package takes no dependencies;
//! * the list of files the gates judge.
//!
//! The file list comes from the git index (`git ls-files -z`), so untracked
//! scratch files never count. Each path is read under the gate root, which is
//! `RAF_GATE_ROOT` when set and the package directory otherwise. The commit
//! hook points `RAF_GATE_ROOT` at a checkout of the index, so the gates judge
//! what the commit records.
//!
//! Phrases below are written with a `[ ]` character class in place of a space.
//! That keeps every pattern from matching its own text, and
//! [`unclass`] turns the class back into a space for matching.

use std::borrow::Cow;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
/// The character before `#` must not be `&`, a word character or `#`. The run
/// of word characters after `#` must be one to five digits, so a letter a-f
/// (a hex colour such as `#1f2`) or a longer run (`#123456`) is spared. A
/// three- or four-digit token is also spared when it directly follows `"`,
/// `'` or `:`, which is how a colour value is written in markup and styles.
fn hash_number(line: &str) -> bool {
    line.match_indices('#').any(|(index, _)| {
        let before = char_before(line, index);
        if before.is_some_and(|c| c == '&' || c == '#' || is_word(c)) {
            return false;
        }
        let run: String = line
            .get(index + 1..)
            .unwrap_or_default()
            .chars()
            .take_while(|c| is_word(*c))
            .collect();
        let numeric = (1..=5).contains(&run.len()) && run.chars().all(|c| c.is_ascii_digit());
        let colour_context = matches!(before, Some('"' | '\'' | ':'));
        numeric && !((3..=4).contains(&run.len()) && colour_context)
    })
}

/// Why a line is banned, or `None` when it is clean.
pub(crate) fn banned_in_line(line: &str) -> Option<&'static str> {
    if hash_number(line)
        || pull_request_label(line)
        || NUMBERED_FORMS
            .iter()
            .filter(|form| !form.process)
            .any(|form| numbered_form_hit(line, form))
    {
        return Some("issue or pull request number");
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
    process.then_some("development-process wording")
}

/// The directory the gates read files from.
pub(crate) fn gate_root() -> PathBuf {
    env::var_os("RAF_GATE_ROOT").map_or_else(package_dir, PathBuf::from)
}

fn package_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every path in the git index of the package directory, relative and sorted.
///
/// # Panics
///
/// Panics when `git` cannot run or exits unsuccessfully.
pub(crate) fn tracked_paths() -> Vec<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(package_dir())
        .args(["ls-files", "-z"])
        .output()
        .unwrap_or_else(|e| panic!("cannot run `git ls-files -z`: {e}"));
    assert!(
        output.status.success(),
        "`git ls-files -z` failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut paths: Vec<PathBuf> = output
        .stdout
        .split(|b| *b == 0)
        .filter(|name| !name.is_empty())
        .map(|name| {
            PathBuf::from(
                std::str::from_utf8(name)
                    .unwrap_or_else(|e| panic!("tracked path is not UTF-8: {e}")),
            )
        })
        .collect();
    paths.sort();
    paths.dedup();
    paths
}

/// The bytes of a tracked path, read under the gate root.
///
/// # Panics
///
/// Panics when the file is missing or unreadable: a path the index records
/// but the gate root lacks is a failure, never a skip.
pub(crate) fn read_tracked(root: &Path, path: &Path) -> Vec<u8> {
    let full = root.join(path);
    fs::read(&full).unwrap_or_else(|e| {
        panic!(
            "cannot read tracked file {} under {}: {e}",
            path.display(),
            root.display()
        )
    })
}

/// The text of a file, or `None` when the bytes are binary.
///
/// Only a NUL byte marks a file as binary, which matches `git grep -I`. Any
/// other file is scanned: invalid UTF-8 (a Latin-1 file, say) is decoded
/// lossily so a token in it is still found rather than skipped.
pub(crate) fn decode_text(bytes: &[u8]) -> Option<Cow<'_, str>> {
    if bytes.contains(&0) {
        return None;
    }
    Some(String::from_utf8_lossy(bytes))
}
