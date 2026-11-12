// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Fixed in-memory workspaces that prove the installed census still finds
//! what it is built to find.
//!
//! Every case builds its workspace in memory and evaluates it through the
//! production [`Gate::check_tree`]. Nothing is read from or written to disk.

mod detectors;
mod engine;
pub(crate) mod fixtures;
#[cfg(test)]
mod tests;

use std::fmt;

use purrdf_jsonschema::ecma::MatchLimits;
use raf_rdf::{Builder, Format, TermValue as Term};
use raf_source::MemoryTree;

use crate::report::{shape, write};
use crate::{DEBT_FILE, Gate, GateReport, LEDGER_FILE, NORMAL_FORM, REPORT_FORM, Rule};

/// The outcome of one self-test case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelfTestCase {
    id: String,
    failure: Option<String>,
}

impl SelfTestCase {
    /// The stable case identity, such as `detector/hex-table/positive`.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    /// Why the census did not hold the case, or `None` when it did.
    #[must_use]
    pub fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }
    /// Whether the census held the case.
    #[must_use]
    pub const fn passed(&self) -> bool {
        self.failure.is_none()
    }
}

/// Every case of one self-test run, in its fixed order.
///
/// `Display` writes one `pass ID` or `fail ID: REASON` line per case and a
/// closing count line. The text holds no path, clock reading or address, so
/// two runs of one build print the same bytes.
#[must_use = "a dropped report loses its failures"]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelfTest {
    cases: Vec<SelfTestCase>,
}

impl SelfTest {
    /// The cases in their fixed order.
    #[must_use]
    pub fn cases(&self) -> &[SelfTestCase] {
        &self.cases
    }
    /// Whether the census held every case.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.cases.iter().all(SelfTestCase::passed)
    }
}

impl fmt::Display for SelfTest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut failed = 0_usize;
        for case in &self.cases {
            match &case.failure {
                None => writeln!(f, "pass {}", case.id)?,
                Some(reason) => {
                    failed += 1;
                    // One line per case, whatever a refusal message holds.
                    let reason: String = reason
                        .chars()
                        .map(|c| if c.is_control() { ' ' } else { c })
                        .collect();
                    writeln!(f, "fail {}: {reason}", case.id)?;
                }
            }
        }
        writeln!(f, "{} cases, {failed} failed", self.cases.len())
    }
}

/// Run the engine and built-in detector cases, then the fixtures of every
/// registered rule, under one matching grant.
pub(crate) fn run(rules: &[Box<dyn Rule>], limits: MatchLimits) -> SelfTest {
    let mut cases = Vec::new();
    engine::normal_form(limits, &mut cases);
    detectors::cases(limits, &mut cases);
    engine::sanctions(limits, &mut cases);
    engine::debt(limits, &mut cases);
    for rule in rules {
        let held = rule
            .fixtures()
            .validate(rule.as_ref())
            .map_err(|error| error.to_string());
        cases.push(case(format!("rule/{}/fixtures", rule.id()), held));
    }
    SelfTest { cases }
}

fn case(id: impl Into<String>, held: Result<(), String>) -> SelfTestCase {
    SelfTestCase {
        id: id.into(),
        failure: held.err(),
    }
}

/// One ledger assertion: subject, census property and object.
type Row = (&'static str, &'static str, Term);

fn text(value: &str) -> Term {
    Term::simple_literal(value)
}

/// Serialize a ledger made of the current version header and `rows`.
fn ledger(rows: impl IntoIterator<Item = Row>) -> Result<String, String> {
    let header = [
        ("", "type", shape::iri("Ledger")),
        ("", "normalForm", shape::integer(NORMAL_FORM.into())),
        ("", "reportForm", shape::integer(REPORT_FORM.into())),
    ];
    let mut graph = Builder::new();
    for (subject, key, object) in header.into_iter().chain(rows) {
        write::put(&mut graph, subject, key, object).map_err(|error| error.to_string())?;
    }
    graph
        .finish()
        .and_then(|graph| graph.to_text(Format::Turtle))
        .map_err(|error| error.to_string())
}

/// The rows of job `number`, homed at `demo::alias`, with its forbidden rows.
fn job(enforced: bool, forbidden: impl IntoIterator<Item = Row>) -> Vec<Row> {
    let mut rows = vec![
        ("", "job", Term::iri("#job")),
        ("#job", "type", shape::iri("LedgerJob")),
        ("#job", "jobId", text("number")),
        ("#job", "summary", text("The approved implementation")),
        ("#job", "homeSymbol", text("demo::alias")),
        ("#job", "enforced", Term::boolean(enforced)),
    ];
    rows.extend(forbidden);
    rows
}

fn min_tokens(count: u128) -> Row {
    ("", "minTokens", shape::integer(count))
}

/// A one-package workspace `demo` with an optional debt record.
fn package(source: &str, ledger: String, debt: Option<String>) -> MemoryTree {
    let mut files = vec![
        (
            "Cargo.toml",
            "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n".to_owned(),
        ),
        ("src/lib.rs", source.to_owned()),
        (LEDGER_FILE, ledger),
    ];
    files.extend(debt.map(|record| (DEBT_FILE, record)));
    files.into_iter().collect()
}

/// A workspace with the home package `demo` and a separate `consumer`.
/// The home source re-exports `home` as the job's `demo::alias`.
fn workspace(home: &str, consumer: &str, ledger: String) -> MemoryTree {
    MemoryTree::from_iter([
        (
            "Cargo.toml",
            "[workspace]\nmembers=['home','consumer']\nresolver='2'\n".to_owned(),
        ),
        (
            "home/Cargo.toml",
            "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n".to_owned(),
        ),
        (
            "consumer/Cargo.toml",
            "[package]\nname='consumer'\nversion='0.1.0'\nedition='2021'\n".to_owned(),
        ),
        ("home/src/lib.rs", format!("{home}\npub use home as alias;")),
        ("consumer/src/lib.rs", consumer.to_owned()),
        (LEDGER_FILE, ledger),
    ])
}

/// A Gate with no registered or required rules, holding only the caller's
/// matching grant; [`Gate::check_tree`] never reads its directory.
fn gate(limits: MatchLimits) -> Gate {
    Gate::workspace(".").matching_limits(limits)
}

fn check(gate: &Gate, tree: &MemoryTree) -> Result<GateReport, String> {
    gate.check_tree(tree)
        .map_err(|error| format!("the census refused the fixture: {error}"))
}

fn finding(report: &GateReport, expected: &str) -> Result<(), String> {
    if report
        .findings()
        .iter()
        .any(|finding| finding.contains(expected))
    {
        Ok(())
    } else {
        Err(format!(
            "no finding contains {expected:?}; findings: {:?}",
            report.findings()
        ))
    }
}

fn clean(report: &GateReport) -> Result<(), String> {
    if report.is_clean() {
        Ok(())
    } else {
        Err(format!("unexpected findings: {:?}", report.findings()))
    }
}

fn copies(report: &GateReport, expected: usize) -> Result<(), String> {
    let found = report.jobs().first().map(|job| job.copies);
    if found == Some(expected) {
        Ok(())
    } else {
        Err(format!(
            "expected {expected} copies of the job, found {found:?}; findings: {:?}",
            report.findings()
        ))
    }
}
