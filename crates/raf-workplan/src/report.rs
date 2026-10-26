// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Typed finding and item-coverage projections into the shared ground profile.
//!
//! These adapters retain domain fields, absent locations, and named source
//! provenance. A coverage gap or exclusion is an observation, not evidence of a
//! successful test execution. Human tables and command exit policy are separate.

use crate::{
    Finding,
    coverage::{CoverageReport, Gap, GapKind, SurfaceError, SurfaceFailure},
};
use raf_okf::Location;
use raf_rdf::{Builder, Error, Format, Graph, TermBox, TermValue, Triple, vocab};
use std::collections::BTreeMap;

fn count(value: usize) -> TermValue {
    TermValue::integer(i128::try_from(value).expect("supported pointer width fits i128"))
}

/// Closed class/property registry of the finding and item-coverage projections.
/// RDF structural predicates and XSD primitive datatypes use their standard IRIs.
pub const TERMS: &[&str] = &[
    "Report",
    "Finding",
    "CoveragePackage",
    "CoverageGap",
    "CoverageExcluded",
    "CoverageError",
    "SourceLocation",
    "reportForm",
    "command",
    "result",
    "ordinal",
    "kind",
    "file",
    "line",
    "section",
    "key",
    "message",
    "fix",
    "package",
    "inScope",
    "path",
    "item",
    "exclusion",
    "reason",
    "target",
    "root",
    "failure",
    "failurePath",
    "items",
    "coveredItems",
    "tests",
    "coveredTests",
    "direction",
    "source",
];

/// Additional class/property terms of the typed command result projections.
/// The complete workplan registry is this list together with [`TERMS`].
pub const COMMAND_TERMS: &[&str] = &[
    "Argument",
    "Assessment",
    "AssessmentContext",
    "AssessmentRecorded",
    "Binding",
    "BindingGap",
    "BindingResolution",
    "Capability",
    "CapabilityReference",
    "Ceiling",
    "ChangeReference",
    "ClaimCoverage",
    "ClaimCoverageReport",
    "ClaimReference",
    "Completed",
    "ContextEvidence",
    "ContextSymbol",
    "CrateLines",
    "CurrentInput",
    "DebtWritten",
    "Definition",
    "Dependant",
    "DirectoryReference",
    "EvidenceFailure",
    "EvidenceRecorded",
    "EvidenceSupport",
    "ExceptedLines",
    "FileReference",
    "Guidance",
    "History",
    "HistoryEntry",
    "HistorySource",
    "Impact",
    "Input",
    "LineRange",
    "LinesError",
    "LinesReport",
    "Measure",
    "MeasureReference",
    "ModuleLines",
    "MovedInput",
    "NewChange",
    "Obligation",
    "OpenChange",
    "OpenChanges",
    "Preview",
    "Refusal",
    "Refused",
    "RemovedBinding",
    "Rewritten",
    "SectionLink",
    "SectionReference",
    "State",
    "TestReference",
    "UnitLines",
    "VerificationRun",
    "advice",
    "affects",
    "argument",
    "binding",
    "by",
    "capability",
    "ceiling",
    "ceilingRow",
    "change",
    "claim",
    "commit",
    "condition",
    "count",
    "counted",
    "covered",
    "current",
    "date",
    "debt",
    "decimal",
    "definition",
    "dependant",
    "dependsOn",
    "digest",
    "directory",
    "document",
    "ended",
    "entry",
    "error",
    "evidence",
    "excepted",
    "exception",
    "exitCode",
    "failed",
    "failures",
    "findings",
    "first",
    "for",
    "from",
    "gap",
    "guidance",
    "id",
    "identifier",
    "ignored",
    "input",
    "last",
    "lastResolved",
    "limit",
    "lines",
    "link",
    "list",
    "listed",
    "measure",
    "module",
    "moved",
    "name",
    "named",
    "number",
    "obligation",
    "of",
    "okLines",
    "openChange",
    "orphan",
    "outcome",
    "passed",
    "percent",
    "programKind",
    "range",
    "record",
    "recordPath",
    "removal",
    "removed",
    "repeated",
    "replaced",
    "resolves",
    "revision",
    "run",
    "selected",
    "shippingCounted",
    "shippingOrphan",
    "slug",
    "state",
    "subject",
    "symbol",
    "syncNeeded",
    "template",
    "test",
    "testCounted",
    "testOrphan",
    "text",
    "title",
    "unit",
    "unresolved",
    "value",
    "verdict",
];

fn named(value: &str) -> TermValue {
    TermValue::iri(format!("{}{value}", vocab::REPORT))
}
fn local(value: &str) -> TermValue {
    TermValue::iri(value)
}
fn add(graph: &mut Builder, subject: &str, key: &str, value: TermValue) -> Result<(), Error> {
    graph.insert(Triple {
        subject: local(subject),
        predicate: named(key),
        object: value,
    })
}
fn class(graph: &mut Builder, subject: &str, name: &str) -> Result<(), Error> {
    graph.insert(Triple {
        subject: local(subject),
        predicate: TermValue::iri(vocab::TYPE),
        object: named(name),
    })
}
fn root(command: &str) -> Result<Builder, Error> {
    let mut graph = Builder::new();
    class(&mut graph, "", "Report")?;
    add(&mut graph, "", "reportForm", TermValue::integer(1))?;
    add(
        &mut graph,
        "",
        "command",
        TermValue::simple_literal(command),
    )?;
    Ok(graph)
}
fn member(graph: &mut Builder, ordinal: usize, name: &str) -> Result<String, Error> {
    let id = format!("#result/{ordinal}");
    add(graph, "", "result", local(&id))?;
    class(graph, &id, name)?;
    add(graph, &id, "ordinal", count(ordinal))?;
    Ok(id)
}

/// Project actual check findings without flattening location, message or fix.
///
/// # Errors
/// Refuses an unknown finding kind or invalid graph/term.
pub fn findings_graph(findings: &[Finding]) -> Result<Graph, Error> {
    let mut graph = root("check")?;
    for (ordinal, finding) in findings.iter().enumerate() {
        let id = format!("#result/{ordinal}");
        add(&mut graph, "", "result", local(&id))?;
        add(&mut graph, &id, "ordinal", count(ordinal))?;
        finding_into(&mut graph, &id, finding)?;
    }
    graph.finish()
}

/// Append a typed finding at an owned subject shared by command projections.
/// The caller supplies membership and ordinal links; the finding class and all
/// location fields are written here. Discard the graph when an error occurs.
///
/// # Errors
/// Refuses unknown finding kinds, zero lines, unsupported locations and invalid
/// or duplicate graph terms.
pub fn finding_into(graph: &mut Builder, id: &str, finding: &Finding) -> Result<(), Error> {
    class(graph, id, "Finding")?;
    if !Finding::KINDS.contains(&finding.kind) {
        return Err(Error::Value(format!(
            "unknown finding kind {}",
            finding.kind
        )));
    }
    for (key, text) in [
        ("kind", finding.kind),
        ("file", finding.file.as_str()),
        ("message", finding.message.as_str()),
        ("fix", finding.fix.as_str()),
    ] {
        add(graph, id, key, TermValue::simple_literal(text))?;
    }
    match &finding.location {
        Location::Line(0) => return Err(Error::Value("finding line must be positive".into())),
        Location::Line(value) => add(graph, id, "line", count(*value))?,
        Location::Section(value) => add(graph, id, "section", TermValue::simple_literal(value))?,
        Location::Key(value) => add(graph, id, "key", TermValue::simple_literal(value))?,
        Location::Document => {}
        _ => return Err(Error::Value("unsupported finding location".into())),
    }
    Ok(())
}

/// Render typed findings in either canonical RDF 1.2 format.
///
/// # Errors
/// Propagates term, registry and ground-profile refusals.
pub fn findings(findings: &[Finding], format: Format) -> Result<String, Error> {
    findings_graph(findings)?.to_text(format)
}

fn gap(graph: &mut Builder, id: &str, value: &Gap, in_scope: bool) -> Result<(), Error> {
    for (key, text) in [
        ("kind", value.kind().name()),
        ("package", value.package()),
        ("path", value.path()),
        ("item", value.item()),
        ("file", value.file()),
    ] {
        add(graph, id, key, TermValue::simple_literal(text))?;
    }
    add(graph, id, "line", count(value.line()))?;
    add(graph, id, "inScope", TermValue::boolean(in_scope))?;
    let proof = format!("{id}/source");
    class(graph, &proof, "SourceLocation")?;
    add(graph, id, "source", local(&proof))?;
    graph.insert(Triple {
        subject: local(&proof),
        predicate: TermValue::iri(vocab::REIFIES),
        object: TermValue::Triple {
            s: TermBox::new(local(id)),
            p: TermBox::new(named("item")),
            o: TermBox::new(TermValue::simple_literal(value.item())),
        },
    })?;
    add(
        graph,
        &proof,
        "file",
        TermValue::simple_literal(value.file()),
    )?;
    add(graph, &proof, "line", count(value.line()))?;
    Ok(())
}

/// Project item/test coverage, including package counts, exclusions and failures.
///
/// # Errors
/// Refuses an inconsistent package reference or invalid term/graph.
pub fn coverage_graph(report: &CoverageReport, direction: GapKind) -> Result<Graph, Error> {
    let mut graph = root("coverage")?;
    add(
        &mut graph,
        "",
        "direction",
        TermValue::simple_literal(if direction.is_code() { "code" } else { "tests" }),
    )?;
    let mut ordinal = 0;
    for package in report.packages() {
        let id = member(&mut graph, ordinal, "CoveragePackage")?;
        ordinal += 1;
        add(
            &mut graph,
            &id,
            "package",
            TermValue::simple_literal(package.name()),
        )?;
        add(
            &mut graph,
            &id,
            "inScope",
            TermValue::boolean(package.in_scope()),
        )?;
        for (key, value) in [
            ("items", package.items()),
            ("coveredItems", package.covered_items()),
            ("tests", package.tests()),
            ("coveredTests", package.covered_tests()),
        ] {
            add(&mut graph, &id, key, count(value))?;
        }
    }
    let packages: BTreeMap<_, _> = report
        .packages()
        .iter()
        .map(|p| (p.name(), p.in_scope()))
        .collect();
    let in_scope = |gap: &Gap| -> Result<bool, Error> {
        packages
            .get(gap.package())
            .copied()
            .ok_or_else(|| Error::Value(format!("coverage package absent: {}", gap.package())))
    };
    for value in report.gaps(direction) {
        let id = member(&mut graph, ordinal, "CoverageGap")?;
        ordinal += 1;
        gap(&mut graph, &id, value, in_scope(value)?)?;
    }
    for excluded in report
        .excluded()
        .iter()
        .filter(|e| e.gap().kind().is_code() == direction.is_code())
    {
        let id = member(&mut graph, ordinal, "CoverageExcluded")?;
        ordinal += 1;
        gap(&mut graph, &id, excluded.gap(), in_scope(excluded.gap())?)?;
        add(
            &mut graph,
            &id,
            "exclusion",
            TermValue::simple_literal(excluded.exclusion()),
        )?;
        add(
            &mut graph,
            &id,
            "reason",
            TermValue::simple_literal(excluded.reason()),
        )?;
    }
    if direction.is_code() {
        for error in report.errors() {
            let id = member(&mut graph, ordinal, "CoverageError")?;
            ordinal += 1;
            let scope = packages.get(error.package()).copied().ok_or_else(|| {
                Error::Value(format!("coverage package absent: {}", error.package()))
            })?;
            surface_error(&mut graph, &id, error, scope)?;
        }
    }
    graph.finish()
}

fn surface_error(
    graph: &mut Builder,
    id: &str,
    error: &SurfaceError,
    scope: bool,
) -> Result<(), Error> {
    add(graph, id, "inScope", TermValue::boolean(scope))?;
    for (key, text) in [
        ("package", error.package()),
        ("target", error.target()),
        ("root", error.root()),
        ("failure", error.failure().name()),
        ("message", error.message()),
    ] {
        add(graph, id, key, TermValue::simple_literal(text))?;
    }
    if let SurfaceFailure::CarryBudget { path } = error.failure() {
        add(graph, id, "failurePath", TermValue::simple_literal(path))?;
    }
    Ok(())
}

/// Render typed item/test coverage in either canonical RDF 1.2 format.
///
/// # Errors
/// Propagates domain and ground-profile refusals.
pub fn coverage(
    report: &CoverageReport,
    direction: GapKind,
    format: Format,
) -> Result<String, Error> {
    coverage_graph(report, direction)?.to_text(format)
}
