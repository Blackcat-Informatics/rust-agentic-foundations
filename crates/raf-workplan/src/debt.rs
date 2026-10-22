// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The debt record of the complete policy and the ratchet over it.
//!
//! A repository that adopts [the complete policy](crate::strict) with findings it has
//! not yet paid records them in `work/debt/default.md`, a record of type `Debt`
//! that a tool writes ([`Gate::write_debt`], `cargo workplan debt --write`) and
//! a person reviews. The rule is the one `arch:debt-ratchet` states for every
//! ratchet of the repository, applied to the findings of the complete policy:
//!
//! * the record is owned by this tool and holds only its debt;
//! * an entry is keyed, and a finding that is present and not recorded is
//!   growth: it is reported as it is;
//! * a recorded entry that is not present, or a count that is below its
//!   record, fails as [`debt-resolved`](crate::Finding::KINDS) until the write
//!   lowers the record, so the record always equals the present debt;
//! * the write refuses growth against a record that exists, creates the record
//!   when none exists (adoption), refuses while a finding that is not debt
//!   exists, and takes the present state;
//! * the control is the review of the diff of the record.
//!
//! # The entries
//!
//! The `findings` rows hold a finding of one of [`DEBT_KINDS`], keyed by the
//! kind and a subject: the claim key `cap:<slug>#<section>/<key>` for
//! `strict-claim-uncovered`, the capability `cap:<slug>` for
//! `strict-example-missing`, the section `cap:<slug>#<section>` for the
//! other content rules and `strict-claim-unit-collapsed` (the section the rule
//! judges: Verification, Usage, Constraints and Contract for the four with
//! one, the section of the finding for `strict-section-thin`), and the
//! reported path of the item or test for the three coverage kinds. A finding
//! of another kind is never debt: a finding of the default rules, the
//! obligations of a change (they belong to an open change), an unreadable
//! surface, and `coverage-lines-over-ceiling` as a row of `findings`.
//!
//! The `lines` rows hold orphan lines (see [`crate::coverage::lines`]) as a
//! count for each item that holds some, in a package that is over a ceiling of
//! the coverage record: the crate-qualified item path and the number of
//! orphan lines it holds. Ranges are reported by `coverage lines` and never
//! recorded. A package over a ceiling with a row for some of its items is
//! judged by its rows: an item above its row, or with no row, is
//! `coverage-lines-over-ceiling` naming the row, and an item below its row is
//! `debt-resolved`. A package over a ceiling with no row is judged by its
//! ceilings. A package that is within its ceilings holds no debt, and its rows
//! are `debt-resolved`.
//!
//! # The write
//!
//! [`Gate::write_debt`] and [`Opened::write_debt`] write the present debt:
//!
//! * [`DebtError::Findings`] while a finding exists that no row can hold: a
//!   finding of the default rules, a generated block that is stale, a surface
//!   that cannot be listed, a report that cannot count the lines. Nothing is
//!   written;
//! * [`DebtError::Growth`] when the record exists and a finding is present that
//!   it does not hold, or an item holds more lines than its row, or has no
//!   row. Nothing is written;
//! * otherwise the record is created when none exists, and replaced when the
//!   present debt differs from it; two writes of the same state write the same
//!   bytes, and the second leaves the file untouched.
//!
//! Only `work/debt/default.md` is written. A record with SPDX comment lines
//! keeps them when the write is given none.
//!
//! ```rust
//! use std::fs;
//!
//! use raf_workplan::debt::DebtChange;
//! use raf_workplan::{Gate, SyncOptions, Workplan};
//!
//! let scratch = purrdf_testkit::TempDir::new_in(std::env::temp_dir())?;
//! let dir = scratch.path();
//! let sections: String = ["Purpose", "Contract", "Usage", "Examples", "Constraints", "Explanation", "Verification"]
//!     .iter()
//!     .map(|name| format!("## {name}\n\nThe {name} of the demo.\n\n"))
//!     .collect();
//! let capability = format!(
//!     "---\ntype: Capability\nid: cap:demo\ntitle: \"Demo\"\ndescription: \"One line.\"\n\
//!      state: delivered\nbindings:\n  code: [demo::run]\n  tests: [demo::tests::it_runs]\n  examples: []\n\
//!      depends: []\n---\n\n# Demo\n\n{sections}"
//! );
//! let library = "pub fn run() {}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn it_runs() {\n        super::run();\n    }\n}\n";
//! let files = [
//!     ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n".to_string()),
//!     ("src/lib.rs", library.to_string()),
//!     ("docs/capabilities/demo.md", capability),
//!     (
//!         "work/coverage/default.md",
//!         "---\ntype: Coverage\nid: coverage:default\nscope: []\nexclusions: []\n---\n"
//!             .to_string(),
//!     ),
//! ];
//! for (path, text) in files {
//!     let file = dir.join(path);
//!     fs::create_dir_all(file.parent().expect("a file has a directory"))?;
//!     fs::write(file, text)?;
//! }
//!
//! // Sync the generated blocks, so the repository has no finding of the
//! // default rules; its strict findings are then its debt.
//! let gate = Gate::workspace(&dir);
//! let opened = gate.open()?;
//! let synced = opened.workplan().sync(&SyncOptions::new("2026 Example", "CC-BY-4.0"))?;
//! Workplan::write(&synced, &dir)?;
//! assert!(!gate.check().is_empty());
//!
//! // The write holds them, and the repository then passes.
//! let written = gate.write_debt()?;
//! assert_eq!(written.change(), DebtChange::Created);
//! assert_eq!(written.path(), "work/debt/default.md");
//! assert!(written.findings() > 0);
//! assert!(gate.check().is_empty());
//! // The same state writes nothing.
//! assert_eq!(gate.write_debt()?.change(), DebtChange::Unchanged);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use raf_okf::{CapabilitySection, DebtFinding, DebtLine, DebtRecord, Location, is_kebab};

use crate::coverage::lines::{self, LinesReport};
use crate::finding::kind;
use crate::gate::{GateError, Opened};
use crate::record::{self, Record, RecordError, RecordOptions};
use crate::strict;
use crate::{Finding, Gate, Loaded, Workplan, spdx};

/// The kinds of finding a debt record can hold as a row of `findings`: the
/// seven rules of the complete policy for a delivered capability and the three
/// coverage gaps of the packages that the complete policy alone enforces.
pub const DEBT_KINDS: [&str; 10] = [
    kind::STRICT_EXAMPLE_MISSING,
    kind::STRICT_MATRIX_MISSING,
    kind::STRICT_CLAIM_UNCOVERED,
    kind::STRICT_CLAIM_UNIT_COLLAPSED,
    kind::STRICT_SECTION_THIN,
    kind::STRICT_USAGE_UNBOUND,
    kind::STRICT_LIMITATIONS_MISSING,
    kind::COVERAGE_ITEM_UNBOUND,
    kind::COVERAGE_REEXPORT_UNBOUND,
    kind::COVERAGE_TEST_UNBOUND,
];

/// The key a finding of the complete policy is held against: its kind and subject.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Key {
    kind: &'static str,
    subject: String,
}

/// A finding of the complete policy and the row of `findings` that can hold it, if
/// any.
#[derive(Debug, Clone)]
pub(crate) struct Owed {
    pub(crate) finding: Finding,
    pub(crate) key: Option<Key>,
}

impl Owed {
    /// A finding a row of `findings` can hold, keyed by its kind and `subject`.
    pub(crate) fn held(finding: Finding, subject: impl Into<String>) -> Self {
        let key = Key {
            kind: finding.kind,
            subject: subject.into(),
        };
        Self {
            finding,
            key: Some(key),
        }
    }

    /// A finding no row can hold.
    pub(crate) fn unheld(finding: Finding) -> Self {
        Self { finding, key: None }
    }
}

/// What the debt record and the present findings of the complete policy say.
#[derive(Debug, Clone, Default)]
pub(crate) struct Judged {
    /// The findings of the complete policy that stand, then those of the record, in
    /// order.
    pub(crate) reported: Vec<Finding>,
    /// The findings of the complete policy that no row can hold.
    pub(crate) unheld: Vec<Finding>,
    /// The findings that are growth against a record that exists.
    pub(crate) growth: Vec<Finding>,
    /// The findings of the complete policy that a row can hold: the present debt.
    pub(crate) findings: BTreeSet<Key>,
    /// The orphan lines of each item of a package over a ceiling: the present
    /// line debt.
    pub(crate) lines: BTreeMap<String, usize>,
}

/// Judge the findings of the complete policy against the debt record.
pub(crate) fn judge(plan: &Workplan<'_>) -> Judged {
    let record = plan.debt();
    let (invalid_rows, recorded) = record.map_or_else(Default::default, invalid);
    let mut out = Judged::default();
    for owed in strict::owed(plan) {
        let Some(key) = owed.key else {
            out.unheld.push(owed.finding.clone());
            out.reported.push(owed.finding);
            continue;
        };
        let held = recorded
            .findings
            .contains(&(key.kind, key.subject.as_str()));
        out.findings.insert(key);
        if !held {
            if record.is_some() {
                out.growth.push(owed.finding.clone());
            }
            out.reported.push(owed.finding);
        }
    }
    let mut resolved = Vec::new();
    if let Some(judged) = lines::over_ceilings(plan) {
        lines_judged(plan, judged, &recorded, &mut out, &mut resolved);
    }
    out.reported.extend(invalid_rows);
    if let Some(record) = record {
        for row in &record.value().findings {
            let pair = (row.kind.as_str(), row.subject.as_str());
            if !recorded.findings.contains(&pair) {
                continue;
            }
            let present = out
                .findings
                .iter()
                .any(|key| (key.kind, key.subject.as_str()) == pair);
            if !present {
                out.reported.push(Finding::new(
                    kind::DEBT_RESOLVED,
                    record.path(),
                    Location::Key("findings".to_string()),
                    format!(
                        "the recorded `{}` for `{}` is no longer raised, so the record holds debt that is paid",
                        row.kind, row.subject
                    ),
                    RESOLVED_FIX,
                ));
            }
        }
    }
    out.reported.extend(resolved);
    out
}

/// The fix of `debt-resolved`.
const RESOLVED_FIX: &str = "lower the record with `cargo workplan debt --write` or `Gate::write_debt`; the record always equals the present debt";

/// The rows of the record that hold debt, by what they name.
#[derive(Debug, Default)]
struct Recorded<'r> {
    /// The `(kind, subject)` of each valid row of `findings`.
    findings: BTreeSet<(&'r str, &'r str)>,
    /// The item and count of each valid row of `lines`.
    lines: BTreeMap<&'r str, usize>,
}

/// The `debt-entry-invalid` findings of the rows of the record, in record
/// order, and the rows that are valid.
fn invalid(record: &Loaded<DebtRecord>) -> (Vec<Finding>, Recorded<'_>) {
    let mut out = Vec::new();
    let mut recorded = Recorded::default();
    for row in &record.value().findings {
        let problem = if !DEBT_KINDS.contains(&row.kind.as_str()) {
            Some(format!(
                "`{}` is not a finding a debt record can hold; the kinds are {}",
                row.kind,
                DEBT_KINDS.join(", ")
            ))
        } else if !subject_fits(&row.kind, &row.subject) {
            Some(format!(
                "`{}` is not the subject of a `{}`: {}",
                row.subject,
                row.kind,
                subject_form(&row.kind)
            ))
        } else {
            None
        };
        match problem {
            Some(problem) => out.push(entry_invalid(record, "findings", &problem)),
            None => {
                recorded
                    .findings
                    .insert((row.kind.as_str(), row.subject.as_str()));
            }
        }
    }
    for row in &record.value().lines {
        if !item_fits(&row.item) {
            out.push(entry_invalid(
                record,
                "lines",
                &format!(
                    "`{}` is not an item path: it is the crate-qualified path of an item, with a `::` and no space or control character at either end",
                    row.item
                ),
            ));
        } else if row.orphans == 0 {
            out.push(entry_invalid(
                record,
                "lines",
                &format!(
                    "`{}` records 0 orphan lines, which is no debt; a row holds at least one line",
                    row.item
                ),
            ));
        } else {
            recorded.lines.insert(row.item.as_str(), row.orphans);
        }
    }
    (out, recorded)
}

fn entry_invalid(record: &Loaded<DebtRecord>, list: &str, problem: &str) -> Finding {
    Finding::new(
        kind::DEBT_ENTRY_INVALID,
        record.path(),
        Location::Key(list.to_string()),
        format!("a row of `{list}` is invalid: {problem}"),
        "remove the row, or run `cargo workplan debt --write` to write the present debt; a row is written by the tool and not by hand",
    )
}

/// Whether `text` is a capability id.
fn capability(text: &str) -> bool {
    text.strip_prefix("cap:").is_some_and(is_kebab)
}

/// Whether `text` is `cap:<slug>#<section>`, of `only` when given.
fn section(text: &str, only: Option<CapabilitySection>) -> bool {
    text.split_once('#').is_some_and(|(id, slug)| {
        capability(id)
            && CapabilitySection::from_slug(slug)
                .is_some_and(|found| only.is_none_or(|w| w == found))
    })
}

/// Whether `text` is a claim key `cap:<slug>#<section>/<key>` of the Contract
/// or the Constraints.
fn claim(text: &str) -> bool {
    text.split_once('#').is_some_and(|(id, rest)| {
        rest.split_once('/').is_some_and(|(slug, key)| {
            capability(id)
                && matches!(
                    CapabilitySection::from_slug(slug),
                    Some(CapabilitySection::Contract | CapabilitySection::Constraints)
                )
                && !key.is_empty()
                && key.chars().all(|c| c.is_alphanumeric() || c == '-')
        })
    })
}

/// Whether `text` is an item path: it has a `::` and neither a control
/// character nor a space at either end.
fn item_fits(text: &str) -> bool {
    text.contains("::")
        && text == text.trim()
        && !text.chars().any(char::is_control)
        && !text.starts_with("::")
        && !text.ends_with("::")
}

/// Whether `subject` has the form of the subject of `kind`.
fn subject_fits(kind_name: &str, subject: &str) -> bool {
    match kind_name {
        kind::STRICT_EXAMPLE_MISSING => capability(subject),
        kind::STRICT_MATRIX_MISSING => section(subject, Some(CapabilitySection::Verification)),
        kind::STRICT_USAGE_UNBOUND => section(subject, Some(CapabilitySection::Usage)),
        kind::STRICT_LIMITATIONS_MISSING => section(subject, Some(CapabilitySection::Constraints)),
        kind::STRICT_CLAIM_UNIT_COLLAPSED => section(subject, Some(CapabilitySection::Contract)),
        kind::STRICT_SECTION_THIN => section(subject, None),
        kind::STRICT_CLAIM_UNCOVERED => claim(subject),
        kind::COVERAGE_ITEM_UNBOUND
        | kind::COVERAGE_REEXPORT_UNBOUND
        | kind::COVERAGE_TEST_UNBOUND => item_fits(subject) && !subject.contains(' '),
        _ => false,
    }
}

/// The form of the subject of `kind`, for a message.
fn subject_form(kind_name: &str) -> &'static str {
    match kind_name {
        kind::STRICT_EXAMPLE_MISSING => "it is a capability, `cap:<slug>`",
        kind::STRICT_MATRIX_MISSING => "it is `cap:<slug>#verification`",
        kind::STRICT_USAGE_UNBOUND => "it is `cap:<slug>#usage`",
        kind::STRICT_LIMITATIONS_MISSING => "it is `cap:<slug>#constraints`",
        kind::STRICT_CLAIM_UNIT_COLLAPSED => "it is `cap:<slug>#contract`",
        kind::STRICT_SECTION_THIN => "it is a section, `cap:<slug>#<section>`",
        kind::STRICT_CLAIM_UNCOVERED => {
            "it is a claim key, `cap:<slug>#contract/<key>` or `cap:<slug>#constraints/<key>`"
        }
        _ => "it is the reported path of an item or a test, such as `crate::item`",
    }
}

/// The orphan lines of each item that holds some: the package of its first
/// unit and its lines.
type Present<'r> = BTreeMap<&'r str, (&'r str, usize)>;

/// Judge the orphan lines against the ceilings and the rows of `lines`: the
/// findings that stand go to `out.reported`, those of rows now paid to
/// `resolved`.
fn lines_judged(
    plan: &Workplan<'_>,
    (report, overs): (LinesReport, Vec<lines::Over>),
    recorded: &Recorded<'_>,
    out: &mut Judged,
    resolved: &mut Vec<Finding>,
) {
    if !report.errors().is_empty() {
        for over in overs {
            out.unheld.push(over.finding.clone());
            out.reported.push(over.finding);
        }
        return;
    }
    let record = plan.debt();
    let over_packages: BTreeSet<&str> = overs
        .iter()
        .filter_map(|over| over.package.as_deref())
        .collect();
    let mut present: Present<'_> = BTreeMap::new();
    for unit in report.orphans() {
        let entry = present.entry(unit.item()).or_insert((unit.package(), 0));
        entry.1 += unit.lines();
    }
    // The present debt: the items of the packages over a ceiling.
    let held: Present<'_> = present
        .iter()
        .filter(|(_, (package, _))| over_packages.contains(package))
        .map(|(item, found)| (*item, *found))
        .collect();
    out.lines = held
        .iter()
        .map(|(item, (_, count))| ((*item).to_string(), *count))
        .collect();
    // A package with a row for one of its items is judged by its rows.
    let by_rows: BTreeSet<&str> = held
        .iter()
        .filter(|(item, _)| recorded.lines.contains_key(*item))
        .map(|(_, (package, _))| *package)
        .collect();
    for over in overs {
        let by_row = over
            .package
            .as_deref()
            .is_some_and(|package| by_rows.contains(package));
        if !by_row {
            if record.is_some() {
                out.growth.push(over.finding.clone());
            }
            out.reported.push(over.finding);
        }
    }
    let debt_path = record.map_or("work/debt/default.md", |record| record.path());
    for (item, (package, count)) in &held {
        if !by_rows.contains(package) {
            continue;
        }
        let message = match recorded.lines.get(item) {
            None => format!(
                "`{item}` of `{package}` holds {} and the debt record has no row for it, so its lines are over the ceilings of `{package}`",
                lines_text(*count)
            ),
            Some(row) if count > row => format!(
                "`{item}` of `{package}` holds {}, over the {} its row of `lines` records",
                lines_text(*count),
                lines_text(*row)
            ),
            Some(_) => continue,
        };
        let finding = Finding::new(
            kind::COVERAGE_LINES_OVER_CEILING,
            debt_path,
            Location::Key("lines".to_string()),
            message,
            "bind or reach the items that explain the orphan lines, or add an exception with its reason; the debt record only ever shrinks, and is lowered with `cargo workplan debt --write`",
        );
        out.growth.push(finding.clone());
        out.reported.push(finding);
    }
    if let Some(record) = record {
        for (item, row) in &recorded.lines {
            let reason = match (held.get(item), present.get(item)) {
                (Some((_, count)), _) if count < row => {
                    format!("now holds {}", lines_text(*count))
                }
                (Some(_), _) => continue,
                (None, Some((package, count))) => format!(
                    "holds {} and `{package}` is within its ceilings",
                    lines_text(*count)
                ),
                (None, None) => "holds no orphan line".to_string(),
            };
            resolved.push(Finding::new(
                kind::DEBT_RESOLVED,
                record.path(),
                Location::Key("lines".to_string()),
                format!(
                    "the record holds {} of `{item}` as debt, and the item {reason}",
                    lines_text(*row)
                ),
                RESOLVED_FIX,
            ));
        }
    }
}

/// `1 orphan line`, `5 orphan lines`.
fn lines_text(count: usize) -> String {
    if count == 1 {
        "1 orphan line".to_string()
    } else {
        format!("{count} orphan lines")
    }
}

/// What a write did to the record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DebtChange {
    /// There was no record, and the write created it: adoption.
    Created,
    /// The record existed and held other debt, and the write replaced it.
    Replaced,
    /// The record already held the present debt, and the file is untouched.
    Unchanged,
}

impl DebtChange {
    /// The word for the change as the command line prints it: `created`,
    /// `replaced` or `unchanged`.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Replaced => "replaced",
            Self::Unchanged => "unchanged",
        }
    }
}

/// What [`Gate::write_debt`] wrote: the record, what was done to it and what
/// it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DebtWrite {
    path: String,
    change: DebtChange,
    findings: usize,
    lines: usize,
}

impl DebtWrite {
    /// The repository-relative path of the record, `work/debt/default.md`.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// What the write did to the record.
    #[must_use]
    pub fn change(&self) -> DebtChange {
        self.change
    }

    /// The rows of `findings` the record holds.
    #[must_use]
    pub fn findings(&self) -> usize {
        self.findings
    }

    /// The rows of `lines` the record holds.
    #[must_use]
    pub fn lines(&self) -> usize {
        self.lines
    }
}

/// Why the debt record was not written. Nothing is written in any case.
#[derive(Debug)]
#[non_exhaustive]
pub enum DebtError {
    /// The repository cannot be opened.
    Open(GateError),
    /// A finding exists that no row can hold: a finding of the default rules
    /// or one the complete policy cannot record. Pay it first.
    Findings(Vec<Finding>),
    /// The debt would grow: the findings are the present debt that the record
    /// does not hold. A debt only shrinks; pay it, or remove the record to
    /// adopt anew.
    Growth(Vec<Finding>),
    /// The record could not be written.
    Record(RecordError),
}

impl fmt::Display for DebtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open(error) => write!(f, "{error}"),
            Self::Findings(findings) => {
                write!(
                    f,
                    "{} finding(s) that no debt record can hold stand, so nothing is written; pay them first",
                    findings.len()
                )?;
                crate::lookup::write_findings(f, findings)
            }
            Self::Growth(findings) => {
                write!(
                    f,
                    "the debt would grow by {} finding(s), so nothing is written; a debt record only ever shrinks, so pay them",
                    findings.len()
                )?;
                crate::lookup::write_findings(f, findings)
            }
            Self::Record(error) => write!(f, "{error}"),
        }
    }
}

impl Error for DebtError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Open(error) => Some(error),
            Self::Record(error) => Some(error),
            Self::Findings(_) | Self::Growth(_) => None,
        }
    }
}

impl From<GateError> for DebtError {
    fn from(error: GateError) -> Self {
        Self::Open(error)
    }
}

impl From<RecordError> for DebtError {
    fn from(error: RecordError) -> Self {
        Self::Record(error)
    }
}

impl Gate {
    /// Write the debt record `work/debt/default.md`: the present debt of
    /// the complete policy, as [the module documentation](crate::debt) states, with
    /// the SPDX lines of [`Gate::record_options`] on a record it creates and
    /// the lines a record already has otherwise.
    ///
    /// # Errors
    ///
    /// A [`DebtError`]: the repository cannot be opened, is not judged strict,
    /// holds a finding no row can hold or a debt that grew, or the record
    /// cannot be written. Nothing is written in any of them.
    pub fn write_debt(&self) -> Result<DebtWrite, DebtError> {
        self.open()?.write_debt(self.record_spdx())
    }
}

impl Opened {
    /// Write the debt record as [`Gate::write_debt`] does, with the SPDX
    /// lines of `options` when the record is created or replaced and `options`
    /// gives some, and the lines the record has otherwise.
    ///
    /// # Errors
    ///
    /// The errors of [`Gate::write_debt`] other than opening the repository.
    pub fn write_debt(&self, options: &RecordOptions) -> Result<DebtWrite, DebtError> {
        let plan = self.workplan();
        let judged = judge(&plan);
        let mut blocking = plan.base_findings();
        blocking.extend(judged.unheld.iter().cloned());
        if !blocking.is_empty() {
            return Err(DebtError::Findings(blocking));
        }
        if plan.debt().is_some() && !judged.growth.is_empty() {
            return Err(DebtError::Growth(judged.growth));
        }
        let debt = DebtRecord::new(
            judged
                .findings
                .iter()
                .map(|key| DebtFinding::new(key.kind, key.subject.as_str()))
                .collect(),
            judged
                .lines
                .iter()
                .map(|(item, count)| DebtLine::new(item.as_str(), *count))
                .collect(),
        );
        let (findings, lines) = (debt.findings.len(), debt.lines.len());
        let record = Record::Debt(debt);
        let options = kept(&plan, options);
        let text = record.text(&options)?;
        let path = record.path()?;
        let change = match plan.debt() {
            Some(existing) if existing.text() == text => DebtChange::Unchanged,
            Some(_) => {
                record::replace(self.dir(), &record, &options)?;
                DebtChange::Replaced
            }
            None => {
                record::create(self.dir(), &record, &options)?;
                DebtChange::Created
            }
        };
        Ok(DebtWrite {
            path: path.to_string(),
            change,
            findings,
            lines,
        })
    }
}

/// `options`, or the SPDX lines of the record that exists when `options`
/// gives none.
fn kept(plan: &Workplan<'_>, options: &RecordOptions) -> RecordOptions {
    if options.copyright.is_some() || options.licence.is_some() {
        return options.clone();
    }
    let Some((copyright, licence)) = plan.debt().and_then(|record| spdx::read(record.head()))
    else {
        return options.clone();
    };
    RecordOptions::new().copyright(copyright).licence(licence)
}

#[cfg(test)]
mod tests;
