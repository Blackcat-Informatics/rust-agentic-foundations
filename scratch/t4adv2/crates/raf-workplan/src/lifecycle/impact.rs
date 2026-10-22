// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! [`Workplan::impact`]: what a change reaches, as one value.

use std::collections::BTreeSet;
use std::fmt;

use crate::Workplan;
use crate::graph::{Binding, SectionId};
use crate::lookup::{CHANGE_PREFIX, ChangeLookupError};

/// A capability that depends on a capability the change lists.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct Dependant {
    /// The id of the capability that depends.
    pub capability: String,
    /// The id of the listed capability it depends on.
    pub depends_on: String,
}

/// A section that links to a section the change affects.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct LinkingSection {
    /// The affected section the link leads to.
    pub target: SectionId,
    /// The section that holds the link.
    pub from: SectionId,
}

/// What a change reaches: the readers and the evidence its record puts in play.
///
/// Every list is sorted and without repeats, so equal inputs give equal values
/// and equal text. The value is a pure function of the loaded documents and
/// the source index; building it writes nothing. `Display` is the text
/// `change impact` prints.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Impact {
    /// The slug of the change, without the `change:` prefix.
    pub change: String,
    /// The capabilities in the `changes` of the record, sorted by id.
    pub capabilities: Vec<String>,
    /// The sections in the `affects` of the record, by capability and then in
    /// the order a capability holds its sections.
    pub sections: Vec<SectionId>,
    /// The bindings of the listed capabilities with what the source index made
    /// of each, by capability, then code, tests and examples, then by path.
    pub bindings: Vec<Binding>,
    /// The capabilities that depend on a listed capability, by capability and
    /// then by the capability depended on.
    pub dependants: Vec<Dependant>,
    /// The sections that link to an affected section, by affected section and
    /// then by the section that links.
    pub linking: Vec<LinkingSection>,
}

impl fmt::Display for Impact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{CHANGE_PREFIX}{}", self.change)?;
        group(f, "capabilities", &self.capabilities, Clone::clone)?;
        group(f, "sections", &self.sections, ToString::to_string)?;
        group(f, "bindings", &self.bindings, |binding| {
            let outcome = match binding.resolution() {
                Ok(_) => "resolves".to_string(),
                Err(error) => format!("does not resolve: {error}"),
            };
            format!(
                "{}  {}  {}  {outcome}",
                binding.capability(),
                binding.kind().key(),
                binding.path()
            )
        })?;
        group(f, "dependants", &self.dependants, |dependant| {
            format!(
                "{} depends on {}",
                dependant.capability, dependant.depends_on
            )
        })?;
        group(f, "linking sections", &self.linking, |link| {
            format!("{} links to {}", link.from, link.target)
        })
    }
}

/// A heading and one line for each item, or `none` for no item.
fn group<T>(
    f: &mut fmt::Formatter<'_>,
    heading: &str,
    items: &[T],
    line: impl Fn(&T) -> String,
) -> fmt::Result {
    writeln!(f, "{heading}:")?;
    if items.is_empty() {
        return writeln!(f, "  none");
    }
    for item in items {
        writeln!(f, "  {}", line(item))?;
    }
    Ok(())
}

impl Workplan<'_> {
    /// What the change `change` reaches: the capabilities it lists, the
    /// sections it affects, the bindings of those capabilities, the
    /// capabilities that depend on them and the sections that link to the
    /// affected sections.
    ///
    /// `change` is the slug of a change record, with or without the `change:`
    /// prefix. The record is read from the loaded documents and the rest from
    /// the graph; nothing is written and nothing is started.
    ///
    /// ```rust
    /// use raf_source::{MemoryTree, Workspace};
    /// use raf_workplan::{ChangeLookupError, Workplan};
    ///
    /// let source = Workspace::load(
    ///     &MemoryTree::from([
    ///         ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
    ///         ("src/lib.rs", ""),
    ///     ]),
    ///     &["."],
    /// )?;
    /// let plan = Workplan::load(&MemoryTree::from([("docs/architecture/.keep", "")]), &source);
    /// assert!(matches!(
    ///     plan.impact("grow"),
    ///     Err(ChangeLookupError::Unknown { .. })
    /// ));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`ChangeLookupError::Unknown`] when no change record has the slug, and
    /// [`ChangeLookupError::NotLoaded`] when a record has it and did not load.
    pub fn impact(&self, change: &str) -> Result<Impact, ChangeLookupError> {
        let record = self.change(change)?.value();
        let graph = self.graph();
        let capabilities: BTreeSet<String> = record.changes.iter().cloned().collect();
        let sections: BTreeSet<SectionId> = record
            .affects
            .iter()
            .map(|entry| SectionId::new(&entry.capability, entry.section))
            .collect();
        let bindings = capabilities
            .iter()
            .flat_map(|id| graph.bindings_of(id).cloned())
            .collect();
        let dependants: BTreeSet<Dependant> = graph
            .depends()
            .iter()
            .filter(|edge| capabilities.contains(edge.to()))
            .map(|edge| Dependant {
                capability: edge.from().to_string(),
                depends_on: edge.to().to_string(),
            })
            .collect();
        let linking: BTreeSet<LinkingSection> = sections
            .iter()
            .flat_map(|target| {
                graph
                    .linking_sections(target)
                    .into_iter()
                    .map(|from| LinkingSection {
                        target: target.clone(),
                        from,
                    })
            })
            .collect();
        Ok(Impact {
            change: record.header.slug.clone(),
            capabilities: capabilities.into_iter().collect(),
            sections: sections.into_iter().collect(),
            bindings,
            dependants: dependants.into_iter().collect(),
            linking: linking.into_iter().collect(),
        })
    }
}
