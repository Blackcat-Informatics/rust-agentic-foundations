// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Byte-preserving edits to the state in a loaded capability's frontmatter.

use std::io;

use raf_okf::{Capability, Document, ScalarStyle, State};
use raf_source::RelPath;

use crate::Loaded;

pub(super) fn edit(capability: &Loaded<Capability>, next: State) -> io::Result<(RelPath, String)> {
    let head = capability.head();
    let document = Document::parse(head).map_err(io::Error::other)?;
    let entry = document
        .frontmatter()
        .map()
        .get("state")
        .ok_or_else(|| io::Error::other("loaded capability has no state entry"))?;
    let scalar = entry
        .value()
        .as_scalar()
        .filter(|value| value.text() == capability.value().state.name())
        .ok_or_else(|| io::Error::other("loaded capability disagrees with its state entry"))?;
    let mut lines = head.split_inclusive('\n');
    let offset = lines
        .by_ref()
        .take(entry.line().saturating_sub(1))
        .map(str::len)
        .sum::<usize>();
    let line = lines
        .next()
        .ok_or_else(|| io::Error::other("state entry has no frontmatter line"))?;
    let raw = line
        .trim_end_matches(['\r', '\n'])
        .strip_prefix("state: ")
        .ok_or_else(|| io::Error::other("state entry does not begin with the state key"))?;
    let value = match entry.comments().trailing() {
        Some(comment) => raw
            .strip_suffix(comment)
            .ok_or_else(|| io::Error::other("state entry disagrees with its trailing comment"))?,
        None => raw,
    };
    let replacement = match scalar.style() {
        ScalarStyle::Plain => next.name().to_owned(),
        ScalarStyle::SingleQuoted => format!("'{}'", next.name()),
        ScalarStyle::DoubleQuoted => format!("\"{}\"", next.name()),
    };
    let start = offset + "state: ".len();
    let mut text = capability.text();
    text.replace_range(start..start + value.len(), &replacement);
    Ok((
        RelPath::new(capability.path()).map_err(io::Error::other)?,
        text,
    ))
}
