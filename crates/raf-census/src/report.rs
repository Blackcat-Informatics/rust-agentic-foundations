// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The baseline and index reports.
//!
//! Both are closed schemas: a text with an unknown key, a missing key or a
//! value of the wrong type is not a report. `to_json` renders sorted keys,
//! two-space indentation and a trailing newline, so equal reports are equal
//! bytes. Fingerprints are the text of their `Display` form and integer
//! constants are hexadecimal text, since a constant can exceed what a JSON
//! number carries.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::census::Census;
use crate::group::{Group, GroupKind, Member};
use crate::job::{JobReport, MatchClass};
use crate::{Error, NORMAL_FORM};

/// Where a unit is, as a report row.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemberRow {
    /// The symbol of the unit.
    pub symbol: String,
    /// The file, relative to the workspace root.
    pub file: String,
    /// The 1-based line of the unit's name.
    pub line: usize,
}

impl From<&Member> for MemberRow {
    fn from(member: &Member) -> Self {
        Self {
            symbol: member.symbol.clone(),
            file: member.file.clone(),
            line: member.line,
        }
    }
}

/// A group of the baseline report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaselineGroup {
    /// The shared fingerprint.
    pub fingerprint: String,
    /// The tokens of the value form of the shared body.
    pub tokens: usize,
    /// The units, ordered by symbol, then file, then line.
    pub members: Vec<MemberRow>,
}

/// A unit that holds a hexadecimal digit table.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableRow {
    /// The symbol of the unit.
    pub symbol: String,
    /// The file, relative to the workspace root.
    pub file: String,
    /// The 1-based line of the unit's name.
    pub line: usize,
    /// The table's name after `table:`.
    pub table: String,
}

/// An integer constant above `0xffff` held by units of two or more packages.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstantRow {
    /// The value, as lowercase hexadecimal text with a `0x` prefix.
    pub value: String,
    /// The number of packages that hold it.
    pub packages: usize,
    /// The units that hold it.
    pub members: Vec<MemberRow>,
}

/// The baseline report: what the workspace repeats.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Baseline {
    /// The normal form the fingerprints belong to.
    pub normal_form: u32,
    /// The value-token floor of a body fingerprint.
    pub min_tokens: usize,
    /// The number of units.
    pub units: usize,
    /// The body groups.
    pub body_groups: Vec<BaselineGroup>,
    /// The shim groups.
    pub shim_groups: Vec<BaselineGroup>,
    /// The hexadecimal digit tables.
    pub hex_tables: Vec<TableRow>,
    /// The large constants shared across packages.
    pub shared_constants: Vec<ConstantRow>,
}

/// One unit of the index report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexUnit {
    /// The symbol of the unit.
    pub symbol: String,
    /// The package of the unit.
    pub package: String,
    /// The file, relative to the workspace root.
    pub file: String,
    /// The 1-based line of the unit's name.
    pub line: usize,
    /// Whether the unit ships.
    pub shipping: bool,
    /// The tokens of the value form of the body; 0 for a unit that is not a function.
    pub tokens: usize,
    /// The `body` fingerprint, when the body reaches `min_tokens`.
    #[serde(deserialize_with = "required")]
    pub body: Option<String>,
    /// The `shim` fingerprint the unit groups under; none for a unit that is
    /// not an eligible forwarder.
    #[serde(deserialize_with = "required")]
    pub shim: Option<String>,
}

/// Reads a key that must be present: its value may be `null`, but a missing
/// key is an error, which a plain `Option` field would read as `None`.
fn required<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// A group of the index report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexGroup {
    /// Whether the units share a body or a shim.
    pub kind: GroupKind,
    /// The shared fingerprint.
    pub fingerprint: String,
    /// The units, ordered by symbol, then file, then line.
    pub members: Vec<MemberRow>,
}

/// One match of a job.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchRow {
    /// The symbol of the unit.
    pub symbol: String,
    /// How the unit stands to the job.
    pub class: MatchClass,
}

/// One job of the index report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobRow {
    /// The job's id.
    pub id: String,
    /// The job's home symbol.
    pub home: String,
    /// The matching units.
    pub matches: Vec<MatchRow>,
    /// The number of matches that are copies.
    pub copies: usize,
}

/// The index report: every unit with its fingerprints, every group, every job.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Index {
    /// The normal form the fingerprints belong to.
    pub normal_form: u32,
    /// Every unit, ordered by symbol, then file, then line.
    pub units: Vec<IndexUnit>,
    /// Every group, in the order of [`Census::groups`].
    pub groups: Vec<IndexGroup>,
    /// The jobs the caller matched, in the order given.
    pub jobs: Vec<JobRow>,
}

/// Renders `report` with sorted keys, two-space indentation and a trailing
/// newline.
fn render<T: Serialize>(report: &T) -> Result<String, Error> {
    // A `Value` object is a sorted map, so the keys come out in order.
    let value = serde_json::to_value(report).map_err(|e| Error::Report(e.to_string()))?;
    let mut text =
        serde_json::to_string_pretty(&value).map_err(|e| Error::Report(e.to_string()))?;
    text.push('\n');
    Ok(text)
}

fn parse<T: DeserializeOwned>(text: &str) -> Result<T, Error> {
    serde_json::from_str(text).map_err(|e| Error::Report(e.to_string()))
}

impl Baseline {
    /// The report as JSON: sorted keys, a trailing newline.
    ///
    /// # Errors
    ///
    /// [`Error::Report`] when the report cannot be rendered.
    pub fn to_json(&self) -> Result<String, Error> {
        render(self)
    }

    /// Reads a report written by [`Baseline::to_json`].
    ///
    /// # Errors
    ///
    /// [`Error::Report`] when `text` is not a baseline report: an unknown key,
    /// a missing key or a value of the wrong type.
    pub fn from_json(text: &str) -> Result<Self, Error> {
        parse(text)
    }
}

impl Index {
    /// The report as JSON: sorted keys, a trailing newline.
    ///
    /// # Errors
    ///
    /// [`Error::Report`] when the report cannot be rendered.
    pub fn to_json(&self) -> Result<String, Error> {
        render(self)
    }

    /// Reads a report written by [`Index::to_json`].
    ///
    /// # Errors
    ///
    /// [`Error::Report`] when `text` is not an index report: an unknown key,
    /// a missing key or a value of the wrong type.
    pub fn from_json(text: &str) -> Result<Self, Error> {
        parse(text)
    }
}

fn rows(members: &[Member]) -> Vec<MemberRow> {
    members.iter().map(MemberRow::from).collect()
}

fn baseline_group(group: &Group) -> BaselineGroup {
    BaselineGroup {
        fingerprint: group.fingerprint.to_string(),
        tokens: group.tokens,
        members: rows(&group.members),
    }
}

impl Census {
    /// The baseline report of the loaded workspace.
    #[must_use]
    pub fn baseline(&self) -> Baseline {
        Baseline {
            normal_form: NORMAL_FORM,
            min_tokens: self.options().min_tokens,
            units: self.units().len(),
            body_groups: self.body_groups().map(baseline_group).collect(),
            shim_groups: self.shim_groups().map(baseline_group).collect(),
            hex_tables: self
                .hex_tables()
                .iter()
                .map(|t| TableRow {
                    symbol: t.member.symbol.clone(),
                    file: t.member.file.clone(),
                    line: t.member.line,
                    table: t.table.name().to_owned(),
                })
                .collect(),
            shared_constants: self
                .shared_constants()
                .iter()
                .map(|c| ConstantRow {
                    value: format!("{:#x}", c.value),
                    packages: c.packages,
                    members: rows(&c.members),
                })
                .collect(),
        }
    }

    /// The index report of the loaded workspace, with the jobs the caller
    /// matched through [`Census::match_jobs`].
    #[must_use]
    pub fn index(&self, jobs: &[JobReport]) -> Index {
        let mut units: Vec<IndexUnit> = self
            .units()
            .iter()
            .map(|unit| IndexUnit {
                symbol: unit.symbol.clone(),
                package: unit.package.clone(),
                file: unit.file.clone(),
                line: unit.line,
                shipping: unit.shipping,
                tokens: unit.print.as_ref().map_or(0, |print| print.tokens),
                body: unit
                    .print
                    .as_ref()
                    .and_then(|print| print.body.as_ref())
                    .map(|hashed| hashed.fingerprint.to_string()),
                shim: unit.shim_key.as_ref().map(ToString::to_string),
            })
            .collect();
        units.sort_by(|a, b| (&a.symbol, &a.file, a.line).cmp(&(&b.symbol, &b.file, b.line)));
        Index {
            normal_form: NORMAL_FORM,
            units,
            groups: self
                .groups()
                .iter()
                .map(|group| IndexGroup {
                    kind: group.kind,
                    fingerprint: group.fingerprint.to_string(),
                    members: rows(&group.members),
                })
                .collect(),
            jobs: jobs
                .iter()
                .map(|job| JobRow {
                    id: job.id.clone(),
                    home: job.home.clone(),
                    matches: job
                        .matches
                        .iter()
                        .map(|found| MatchRow {
                            symbol: found.symbol.clone(),
                            class: found.class,
                        })
                        .collect(),
                    copies: job.copies,
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_ends_with_one_newline_and_sorts_keys() {
        let text = render(&MatchRow {
            symbol: "a".into(),
            class: MatchClass::Copy,
        })
        .expect("renders");
        assert_eq!(text, "{\n  \"class\": \"copy\",\n  \"symbol\": \"a\"\n}\n");
    }
}
