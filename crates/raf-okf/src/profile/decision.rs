// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Architecture decision section references in the authored profile.

use std::collections::BTreeSet;
use std::fmt;

use super::{Check, DocType, Entry, Location, Rule, is_kebab};

const KEY: &str = "supersedes";

/// A named section of an Architecture decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionSectionRef {
    architecture: String,
    section: String,
}

impl DecisionSectionRef {
    fn parse(text: &str) -> Option<Self> {
        let (architecture, section) = text.split_once('#')?;
        let document_slug = architecture.strip_prefix(DocType::Architecture.id_prefix())?;
        (is_kebab(document_slug) && is_kebab(section)).then(|| Self {
            architecture: architecture.to_owned(),
            section: section.to_owned(),
        })
    }

    /// The Architecture page ID, including its prefix.
    #[must_use]
    pub fn architecture(&self) -> &str {
        &self.architecture
    }

    /// The anchor of a named decision section.
    #[must_use]
    pub fn section(&self) -> &str {
        &self.section
    }
}

impl fmt::Display for DecisionSectionRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}#{}", self.architecture, self.section)
    }
}

/// The sections a named section of this page supersedes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionSupersession {
    section: String,
    targets: Vec<DecisionSectionRef>,
}

impl DecisionSupersession {
    /// The local decision section anchor.
    #[must_use]
    pub fn section(&self) -> &str {
        &self.section
    }

    /// Distinct named sections superseded by the local section, in author order.
    #[must_use]
    pub fn targets(&self) -> &[DecisionSectionRef] {
        &self.targets
    }
}

pub(super) fn check(check: &mut Check<'_>, entry: &Entry) -> Option<Vec<DecisionSupersession>> {
    let Some(sections) = entry.value().as_map() else {
        check.key(
            entry.key(),
            Rule::WrongShape {
                expected: "a map of named decision sections to replacement targets",
            },
        );
        return None;
    };
    let mut result = Vec::with_capacity(sections.entries().len());
    for row in sections.entries() {
        if let Some(value) = declaration(check, row) {
            result.push(value);
        }
    }
    if result.len() != sections.entries().len() {
        return None;
    }
    purrdf_lex::walk::try_sort_unstable_by(&mut result, |left, right| {
        Ok::<_, std::convert::Infallible>(left.section.cmp(&right.section))
    })
    .unwrap_or_else(|never| match never {});
    Some(result)
}

fn location(line: usize) -> Location {
    if line == 0 {
        Location::Key(KEY.to_owned())
    } else {
        Location::Line(line)
    }
}

fn declaration(check: &mut Check<'_>, row: &Entry) -> Option<DecisionSupersession> {
    let section = row.key();
    let Some(list) = row.value().as_list().filter(|_| is_kebab(section)) else {
        check.add(
            location(row.line()),
            Rule::WrongShape {
                expected: "a kebab-case section anchor mapped to a list of Architecture decision sections",
            },
        );
        return None;
    };
    if list.items().is_empty() {
        check.add(location(row.line()), Rule::EmptyList { list: KEY });
        return None;
    }
    let mut seen = BTreeSet::new();
    let mut targets = Vec::with_capacity(list.items().len());
    for item in list.items() {
        let reference = item.value().as_scalar().and_then(|scalar| {
            let text = scalar.text();
            DecisionSectionRef::parse(text).map(|target| (text, target))
        });
        let Some((text, target)) = reference else {
            check.add(
                location(item.line()),
                Rule::WrongShape {
                    expected: "an Architecture decision section reference arch:<slug>#<section>",
                },
            );
            continue;
        };
        if !seen.insert(text) {
            check.add(
                location(item.line()),
                Rule::DuplicateEntry {
                    list: KEY,
                    entry: text.to_owned(),
                },
            );
            continue;
        }
        targets.push(target);
    }
    (targets.len() == list.items().len()).then(|| DecisionSupersession {
        section: section.to_owned(),
        targets,
    })
}
