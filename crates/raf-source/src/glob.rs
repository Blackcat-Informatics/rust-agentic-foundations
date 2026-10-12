// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The glob patterns of `workspace.members` and `workspace.exclude`.
//!
//! A pattern is a `/`-separated path. Each segment may use `*` (any run of
//! characters), `?` (one character) and `[...]` (one character of a set, with
//! ranges such as `a-z` and `!` or `^` to negate). `**`, a backslash and an
//! unclosed bracket are outside the supported syntax and are refused, never
//! ignored.

/// One piece of a segment pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Piece {
    Literal(char),
    One,
    Run,
    Set {
        negated: bool,
        ranges: Vec<(char, char)>,
    },
}

fn segment(text: &str) -> Result<Vec<Piece>, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut pieces = Vec::new();
    let mut at = 0;
    while at < chars.len() {
        match chars[at] {
            '*' => {
                if chars.get(at + 1) == Some(&'*') {
                    return Err("`**` is not supported".to_owned());
                }
                pieces.push(Piece::Run);
            }
            '?' => pieces.push(Piece::One),
            '\\' => return Err("a backslash is not supported".to_owned()),
            '[' => {
                let mut cursor = at + 1;
                let negated = matches!(chars.get(cursor), Some('!' | '^'));
                if negated {
                    cursor += 1;
                }
                let mut ranges = Vec::new();
                let mut first = true;
                loop {
                    let Some(&c) = chars.get(cursor) else {
                        return Err("a `[` is not closed".to_owned());
                    };
                    if c == ']' && !first {
                        break;
                    }
                    first = false;
                    if chars.get(cursor + 1) == Some(&'-')
                        && chars.get(cursor + 2).is_some_and(|end| *end != ']')
                    {
                        ranges.push((c, chars[cursor + 2]));
                        cursor += 3;
                    } else {
                        ranges.push((c, c));
                        cursor += 1;
                    }
                }
                pieces.push(Piece::Set { negated, ranges });
                at = cursor;
            }
            c => pieces.push(Piece::Literal(c)),
        }
        at += 1;
    }
    Ok(pieces)
}

/// True when `piece` (not a run) matches the character.
fn fits(piece: &Piece, c: char) -> bool {
    match piece {
        Piece::Literal(l) => *l == c,
        Piece::One => true,
        Piece::Set { negated, ranges } => {
            ranges.iter().any(|(low, high)| *low <= c && c <= *high) != *negated
        }
        Piece::Run => false,
    }
}

/// Match a segment in time proportional to the pattern times the name: on a
/// mismatch the scan goes back only to the last `*`, one character further.
fn matches_segment(pieces: &[Piece], text: &[char]) -> bool {
    let (mut p, mut t) = (0, 0);
    let mut star: Option<(usize, usize)> = None;
    while t < text.len() {
        match pieces.get(p) {
            Some(Piece::Run) => {
                star = Some((p, t));
                p += 1;
            }
            Some(piece) if fits(piece, text[t]) => {
                p += 1;
                t += 1;
            }
            _ => match star {
                Some((star_p, star_t)) => {
                    p = star_p + 1;
                    t = star_t + 1;
                    star = Some((star_p, star_t + 1));
                }
                None => return false,
            },
        }
    }
    pieces[p..].iter().all(|piece| *piece == Piece::Run)
}

/// A parsed pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Pattern {
    segments: Vec<Vec<Piece>>,
}

impl Pattern {
    /// Parse a pattern.
    ///
    /// # Errors
    ///
    /// Returns what is outside the supported syntax.
    pub(crate) fn parse(text: &str) -> Result<Self, String> {
        // One trailing `/` is allowed, as in cargo: `crates/*/` is `crates/*`.
        let trimmed = text.trim_start_matches("./");
        let trimmed = trimmed.strip_suffix('/').map_or(trimmed, |rest| rest);
        if trimmed.is_empty() || trimmed.starts_with('/') {
            return Err("the pattern is empty or absolute".to_owned());
        }
        let segments = trimmed
            .split('/')
            .map(|part| {
                if part.is_empty() {
                    Err("a path segment is empty (`//` or a trailing `/`)".to_owned())
                } else {
                    segment(part)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { segments })
    }

    /// True when no segment has a wildcard, so the pattern names one path.
    pub(crate) fn is_literal(&self) -> bool {
        (0..self.segments.len()).all(|index| self.literal(index).is_some())
    }

    /// The number of path segments.
    pub(crate) fn len(&self) -> usize {
        self.segments.len()
    }

    /// The text of segment `index` when it has no wildcard.
    pub(crate) fn literal(&self, index: usize) -> Option<String> {
        self.segments
            .get(index)?
            .iter()
            .map(|piece| match piece {
                Piece::Literal(c) => Some(*c),
                Piece::One | Piece::Run | Piece::Set { .. } => None,
            })
            .collect()
    }

    /// True when segment `index` matches `name`.
    pub(crate) fn segment_matches(&self, index: usize, name: &str) -> bool {
        let chars: Vec<char> = name.chars().collect();
        self.segments
            .get(index)
            .is_some_and(|pieces| matches_segment(pieces, &chars))
    }

    /// True when the pattern matches the path or a directory above it.
    pub(crate) fn matches_prefix(&self, parts: &[&str]) -> bool {
        self.segments.len() <= parts.len() && self.matches_start(parts)
    }

    fn matches_start(&self, parts: &[&str]) -> bool {
        self.segments.iter().zip(parts).all(|(pieces, part)| {
            let chars: Vec<char> = part.chars().collect();
            matches_segment(pieces, &chars)
        })
    }
}
