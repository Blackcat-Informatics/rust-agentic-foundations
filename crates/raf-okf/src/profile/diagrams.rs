// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed authoring declarations for named automatic semantic views.

use crate::{Entry, ItemValue, Map, Rule, Scalar, Value};
use std::{error::Error, fmt, str::FromStr};

use super::Check;

/// An automatic relationship family supplied by Workplan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagramKind {
    /// Authored claim occurrences naming an evidence record.
    EvidenceClaims,
    /// Cargo package dependency declarations and separately identified lock facts.
    WorkspaceDependencies,
    /// Authored capability dependencies.
    CapabilityDependencies,
    /// Implementation, test and example bindings and their resolved source sites.
    Ownership,
    /// Evidence support and prose-assessment currency.
    Coverage,
    /// Changes, affected sections and dependent capabilities.
    ChangeImpact,
}

impl DiagramKind {
    /// Every supported family, in stable vocabulary order.
    pub const ALL: [Self; 6] = [
        Self::EvidenceClaims,
        Self::WorkspaceDependencies,
        Self::CapabilityDependencies,
        Self::Ownership,
        Self::Coverage,
        Self::ChangeImpact,
    ];

    /// The external declaration and command spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EvidenceClaims => "evidence-claims",
            Self::WorkspaceDependencies => "workspace-dependencies",
            Self::CapabilityDependencies => "capability-dependencies",
            Self::Ownership => "ownership",
            Self::Coverage => "coverage",
            Self::ChangeImpact => "change-impact",
        }
    }

    /// Read the closed family vocabulary without accepting aliases.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == name)
    }
}

/// An unsupported automatic view family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramKindError(String);

impl fmt::Display for DiagramKindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown diagram kind `{}`; use {}",
            self.0,
            DiagramKind::ALL.map(DiagramKind::as_str).join(", ")
        )
    }
}
impl Error for DiagramKindError {}
impl FromStr for DiagramKind {
    type Err = DiagramKindError;
    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Self::from_name(name).ok_or_else(|| DiagramKindError(name.to_owned()))
    }
}

/// The semantic grouping used for an overview and its linked detail views.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiagramGrouping {
    /// Show the selected relationships without aggregation.
    #[default]
    None,
    /// Group source nodes by their owning Cargo package.
    Package,
    /// Group document relationships by their owning capability.
    Capability,
}

impl DiagramGrouping {
    /// The external grouping spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Package => "package",
            Self::Capability => "capability",
        }
    }
}

/// One closed, named view declared by a capability or architecture document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramDeclaration {
    /// Unique kebab name within the owning document.
    pub name: String,
    /// Automatic semantic family.
    pub kind: DiagramKind,
    /// `workspace`, a package name, or the document id accepted by the family.
    pub scope: String,
    /// Explicit focus ids; an empty list selects the entire declared scope.
    pub focus: Vec<String>,
    /// Overview grouping; aggregated members remain available in detail views.
    pub grouping: DiagramGrouping,
    /// Selected relationship names; an empty list selects all family relations.
    pub relations: Vec<String>,
    /// Maximum native structural statements in a bundle.
    pub max_statements: usize,
    /// Maximum native visible terms in a bundle.
    pub max_terms: usize,
}

const KEYS: [&str; 8] = [
    "name",
    "kind",
    "scope",
    "focus",
    "grouping",
    "relations",
    "max_statements",
    "max_terms",
];
const EXPECTED: &str = "at most 64 diagram rows with unique kebab name, supported kind and nonempty scope; optional focus/relations lists, grouping and positive bounded limits";

fn text<'a>(row: &'a Map, key: &str) -> Option<&'a str> {
    row.get(key)?
        .value()
        .as_scalar()
        .map(Scalar::text)
        .filter(|text| !text.trim().is_empty())
}

fn strings(row: &Map, key: &str) -> Option<Vec<String>> {
    let Some(entry) = row.get(key) else {
        return Some(Vec::new());
    };
    let mut values = Vec::new();
    for item in entry.value().as_list()?.items() {
        let value = item.value().as_scalar()?.text();
        if value.trim().is_empty() || values.iter().any(|known| known == value) {
            return None;
        }
        values.push(value.to_owned());
    }
    Some(values)
}

fn limit(row: &Map, key: &str, default: usize, maximum: usize) -> Option<usize> {
    match row.get(key) {
        None => Some(default),
        Some(entry) => entry
            .value()
            .as_scalar()?
            .text()
            .parse::<usize>()
            .ok()
            .filter(|value| *value > 0 && *value <= maximum),
    }
}

fn declaration(row: &Map) -> Option<DiagramDeclaration> {
    if row
        .entries()
        .iter()
        .any(|entry| !KEYS.contains(&entry.key()))
    {
        return None;
    }
    let name = text(row, "name")?;
    if !super::is_kebab(name) {
        return None;
    }
    let kind = DiagramKind::from_name(text(row, "kind")?)?;
    let grouping = match row.get("grouping") {
        None => DiagramGrouping::None,
        Some(_) => match text(row, "grouping")? {
            "none" => DiagramGrouping::None,
            "package" => DiagramGrouping::Package,
            "capability" => DiagramGrouping::Capability,
            _ => return None,
        },
    };
    Some(DiagramDeclaration {
        name: name.to_owned(),
        kind,
        scope: text(row, "scope")?.to_owned(),
        focus: strings(row, "focus")?,
        grouping,
        relations: strings(row, "relations")?,
        max_statements: limit(row, "max_statements", 500, 10_000)?,
        max_terms: limit(row, "max_terms", 1500, 30_000)?,
    })
}

pub(super) fn check(check: &mut Check<'_>, entry: &Entry) -> Option<Vec<DiagramDeclaration>> {
    let Value::List(list) = entry.value() else {
        check.key("diagrams", Rule::WrongShape { expected: EXPECTED });
        return None;
    };
    if list.items().len() > 64 {
        check.key("diagrams", Rule::WrongShape { expected: EXPECTED });
        return None;
    }
    let mut found: Vec<DiagramDeclaration> = Vec::new();
    for (index, item) in list.items().iter().enumerate() {
        let parsed = match item.value() {
            ItemValue::Map(row) => declaration(row),
            ItemValue::Scalar(_) => None,
        };
        match parsed {
            Some(value) if !found.iter().any(|known| known.name == value.name) => found.push(value),
            _ => {
                check.key(
                    &format!("diagrams[{index}]"),
                    Rule::WrongShape { expected: EXPECTED },
                );
                return None;
            }
        }
    }
    Some(found)
}
