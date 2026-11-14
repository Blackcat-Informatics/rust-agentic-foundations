// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Normal form, forbidden constant, sanction and debt ratchet cases.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use purrdf_jsonschema::ecma::MatchLimits;
use raf_rdf::TermValue as Term;

use super::{
    Row, SelfTestCase, case, check, clean, copies, finding, gate, job, ledger, min_tokens, package,
    text, workspace,
};
use crate::report::shape;
use crate::{DebtGroup, DebtRecord, GateReport, GroupKind, MatchClass, NORMAL_FORM};

/// One algorithm written twice: renamed parameters and locals, a dropped
/// value adapter and a hexadecimal spelling of the same integer.
pub(super) const RENAMED: [&str; 2] = [
    "pub fn first(values: &[u64]) -> u64 { let mut sum = 0; for value in values { sum += value * 3; } sum }",
    "pub fn second(inputs: &[u64]) -> u64 { let mut total = 0; for input in inputs { total += input.clone() * 0x3; } total }",
];

/// Two bodies of one token count whose loop and branch differ.
pub(super) const CONTROL_FLOW: [&str; 2] = [
    "pub fn first(limit: u64) -> u64 { let mut sum = 0; let mut step = 1; while sum < limit { sum += step * 3; step += 1; } sum }",
    "pub fn second(limit: u64) -> u64 { let mut sum = 0; let mut step = 1; if sum < limit { sum += step * 3; step += 1; } sum }",
];

const CONSTANT_HOME: &str = "/// Home.\npub fn home() -> u32 { 7 }\npub use home as alias;";

const CLONES: &str = "/// Home.\npub fn home(a: u64) -> u64 { a * a + a * 2 }\npub use home as alias;\npub fn copy(b: u64) -> u64 { b * b + b * 2 }";

const FORWARDERS: &str = "/// Home.\npub fn home(a: u32) -> u32 { helper(a, 1) }\npub fn copy(b: u32) -> u32 { helper(b, 1) }\nfn helper(x: u32, y: u32) -> u32 { x + y }";

const ALTERNATIVES: &str = "/// Unix.\n#[cfg(unix)] pub fn f(a: u64) -> u64 { a * a + a * 2 }\n/// Windows.\n#[cfg(windows)] pub fn f(b: u64) -> u64 { b * b + b * 2 }";

const DIVISION: &str = "pub fn copy(n: usize, d: usize) -> usize { (n + d - 1) / d }";

pub(super) fn normal_form(limits: MatchLimits, out: &mut Vec<SelfTestCase>) {
    out.push(case(
        "normal-form/renamed-respelled-copy-groups",
        groups_as_one(limits, RENAMED),
    ));
    out.push(case(
        "normal-form/same-size-other-control-flow-separates",
        separates(limits, CONTROL_FLOW),
    ));
    out.push(case(
        "constant/respelled-forbidden-constant-is-a-copy",
        respelled_constant(limits),
    ));
}

pub(super) fn sanctions(limits: MatchLimits, out: &mut Vec<SelfTestCase>) {
    let body = |members: &'static [&'static str], source: &str, expected: Option<&str>| {
        let rows = distinct("body", members, "demo::home", []);
        sanctioned(limits, source, rows, None, expected)
    };
    let pair: &[&str] = &["demo::home", "demo::copy"];
    let third = format!("{CLONES}\npub fn third(c: u64) -> u64 {{ c * c + c * 2 }}");
    out.push(case(
        "sanction/distinct-body/exact-set-holds",
        body(pair, CLONES, None),
    ));
    out.push(case(
        "sanction/distinct-body/stale-member",
        body(
            &["demo::home", "demo::gone"],
            CLONES,
            Some("stale ledger path"),
        ),
    ));
    out.push(case(
        "sanction/distinct-body/stale-set",
        body(pair, &third, Some("equals no current group")),
    ));
    out.push(case(
        "sanction/distinct-shim/exact-set-holds",
        sanctioned(
            limits,
            FORWARDERS,
            distinct("shim", pair, "demo::home", []),
            None,
            None,
        ),
    ));
    let counted = |source: &str, expected| {
        let multiplicity = [
            ("#distinct", "multiplicity", Term::iri("#count")),
            ("#count", "type", shape::iri("MemberMultiplicity")),
            ("#count", "symbol", text("demo::f")),
            ("#count", "copies", shape::integer(2)),
        ];
        let rows = distinct("body", &["demo::f"], "demo::f", multiplicity);
        sanctioned(limits, source, rows, None, expected)
    };
    let extra = format!(
        "{ALTERNATIVES}\n/// Other.\n#[cfg(target_os = \"freebsd\")] pub fn f(c: u64) -> u64 {{ c * c + c * 2 }}"
    );
    out.push(case(
        "sanction/distinct-multiplicity/exact-count-holds",
        counted(ALTERNATIVES, None),
    ));
    out.push(case(
        "sanction/distinct-multiplicity/stale-count",
        counted(&extra, Some("declared 2 source units, found 3")),
    ));
    detector_variants(limits, out);
    isomorphic_variants(limits, out);
}

pub(super) fn debt(limits: MatchLimits, out: &mut Vec<SelfTestCase>) {
    let counted = |present: usize, recorded: usize, expected: Option<&str>| {
        let mut source = CONSTANT_HOME.to_owned();
        for index in 0..present {
            let _ = write!(source, "\npub fn copy_{index}() -> u32 {{ 7 }}");
        }
        let jobs = BTreeMap::from([("number".to_owned(), recorded)]);
        let forbidden = [("#job", "forbiddenConstant", shape::integer(7))];
        record(&[], jobs).and_then(|debt| {
            sanctioned(limits, &source, job(false, forbidden), Some(debt), expected)
        })
    };
    let grouped = |source: &str, recorded: &[&str], expected| {
        record(recorded, BTreeMap::new())
            .and_then(|debt| sanctioned(limits, source, Vec::new(), Some(debt), expected))
    };
    let both = RENAMED.join("\n");
    out.push(case("debt/exact-record-holds", counted(2, 2, None)));
    out.push(case(
        "debt/rising-copies-refused",
        counted(2, 1, Some("growth cannot be recorded")),
    ));
    out.push(case(
        "debt/falling-copies-require-lowering",
        counted(1, 2, Some("to lower the record")),
    ));
    out.push(case(
        "debt/new-group-refused",
        grouped(&both, &[], Some("growth cannot be recorded")),
    ));
    out.push(case(
        "debt/resolved-group-requires-lowering",
        grouped(
            RENAMED[0],
            &["demo::first", "demo::second"],
            Some("to lower the record"),
        ),
    ));
}

/// Whether the two sources form exactly one body group.
pub(super) fn groups_as_one(limits: MatchLimits, sources: [&str; 2]) -> Result<(), String> {
    let report = evaluate(limits, &sources.join("\n"), Vec::new(), None)?;
    let groups: Vec<Vec<&str>> = report
        .census()
        .body_groups()
        .map(|group| group.member_symbols().collect())
        .collect();
    if groups == [["demo::first", "demo::second"]] {
        Ok(())
    } else {
        Err(format!(
            "expected one group of both bodies, found {groups:?}"
        ))
    }
}

/// Whether two fingerprinted bodies of one size stay ungrouped.
pub(super) fn separates(limits: MatchLimits, sources: [&str; 2]) -> Result<(), String> {
    let report = evaluate(limits, &sources.join("\n"), Vec::new(), None)?;
    let census = report.census();
    let prints: Vec<_> = ["demo::first", "demo::second"]
        .iter()
        .map(|symbol| census.unit(symbol).and_then(|unit| unit.print.as_ref()))
        .collect();
    let [Some(first), Some(second)] = prints[..] else {
        return Err("a body has no print".into());
    };
    if first.tokens != second.tokens || first.body.is_none() || second.body.is_none() {
        return Err(format!(
            "the bodies must be fingerprinted at one size, found {} and {} tokens",
            first.tokens, second.tokens
        ));
    }
    match census.body_groups().next() {
        None => Ok(()),
        Some(group) => Err(format!(
            "the bodies grouped: {:?}",
            group.member_symbols().collect::<Vec<_>>()
        )),
    }
}

fn respelled_constant(limits: MatchLimits) -> Result<(), String> {
    let source = format!("{CONSTANT_HOME}\npub fn copy() -> u32 {{ 0x0007_u32 + 1 }}");
    let forbidden = [("#job", "forbiddenConstant", shape::integer(7))];
    let report = evaluate(limits, &source, job(true, forbidden), None)?;
    copies(&report, 1)?;
    finding(&report, "demo::copy")
}

/// A detector variant sanctions one consumer site of `div-ceil-by-hand`.
fn detector_variants(limits: MatchLimits, out: &mut Vec<SelfTestCase>) {
    let site = |consumer: &'static str, symbol, file, expected: Option<&str>| {
        let forbidden = [(
            "#job",
            "forbiddenFingerprint",
            text("rule:div-ceil-by-hand"),
        )];
        let mut rows = job(true, forbidden);
        rows.extend(variant(symbol, file, "div-ceil-by-hand"));
        let tree = workspace("/// Home.\npub fn home() {}", consumer, ledger(rows)?);
        variant_verdict(&check(&gate(limits), &tree)?, expected)
    };
    let file = "consumer/src/lib.rs";
    out.push(case(
        "sanction/variant-detector/exact-site-holds",
        site(DIVISION, "consumer::copy", file, None),
    ));
    out.push(case(
        "sanction/variant-detector/stale-path",
        site(DIVISION, "consumer::gone", file, Some("stale ledger path")),
    ));
    out.push(case(
        "sanction/variant-detector/stale-file",
        site(
            DIVISION,
            "consumer::copy",
            "consumer/src/other.rs",
            Some("does not exist in"),
        ),
    ));
    out.push(case(
        "sanction/variant-detector/stale-detector",
        site(
            "pub fn copy(n: usize, d: usize) -> usize { n.div_ceil(d) }",
            "consumer::copy",
            file,
            Some("no longer matches detector"),
        ),
    ));
}

/// An isomorphic variant sanctions one clone of a forbidden body fingerprint.
fn isomorphic_variants(limits: MatchLimits, out: &mut Vec<SelfTestCase>) {
    let site = |source: &str, expected: Option<&str>| {
        // The forbidden fingerprint and the adopted debt come from the home's
        // own census, so the case names no normal-form value.
        let found = evaluate(limits, CLONES, vec![min_tokens(1)], None)?;
        let fingerprint = found
            .census()
            .unit("demo::home")
            .and_then(|unit| unit.print.as_ref()?.body.as_ref())
            .ok_or("the home body has no fingerprint")?
            .to_string();
        let debt = found
            .debt()
            .to_turtle()
            .map_err(|error| error.to_string())?;
        let forbidden = [("#job", "forbiddenFingerprint", text(&fingerprint))];
        let mut rows = job(true, forbidden);
        rows.push(min_tokens(1));
        rows.extend(variant("demo::copy", "src/lib.rs", "isomorphic"));
        let report = evaluate(limits, source, rows, Some(debt))?;
        variant_verdict(&report, expected)
    };
    out.push(case(
        "sanction/variant-isomorphic/exact-site-holds",
        site(CLONES, None),
    ));
    out.push(case(
        "sanction/variant-isomorphic/stale-detector",
        site(
            &CLONES.replace("b * 2", "b * 5"),
            Some("no longer matches detector"),
        ),
    ));
}

fn distinct(
    kind: &'static str,
    members: &[&'static str],
    anchor: &str,
    extra: impl IntoIterator<Item = Row>,
) -> Vec<Row> {
    let mut rows = vec![
        min_tokens(1),
        ("", "distinct", Term::iri("#distinct")),
        ("#distinct", "type", shape::iri("Distinct")),
        ("#distinct", "kind", text(kind)),
        ("#distinct", "criterion", text("layering")),
        (
            "#distinct",
            "reason",
            text("Separate target implementations"),
        ),
        (
            "#distinct",
            "anchor",
            Term::iri(format!("{}{anchor}", crate::SYMBOL_ANCHOR_PREFIX)),
        ),
    ];
    rows.extend(
        members
            .iter()
            .map(|member| ("#distinct", "memberSymbol", text(member))),
    );
    rows.extend(extra);
    rows
}

fn variant(symbol: &str, file: &str, detector: &str) -> Vec<Row> {
    vec![
        ("#job", "variant", Term::iri("#variant")),
        ("#variant", "type", shape::iri("Variant")),
        ("#variant", "symbol", text(symbol)),
        ("#variant", "file", text(file)),
        ("#variant", "detector", text(detector)),
        ("#variant", "criterion", text("layering")),
        (
            "#variant",
            "reason",
            text("A separate target implementation"),
        ),
        (
            "#variant",
            "anchor",
            Term::iri(format!("{}demo::home", crate::SYMBOL_ANCHOR_PREFIX)),
        ),
    ]
}

fn record(groups: &[&str], jobs: BTreeMap<String, usize>) -> Result<String, String> {
    let groups = if groups.is_empty() {
        Vec::new()
    } else {
        let members = groups.iter().map(|member| (*member).to_owned());
        vec![DebtGroup::new(GroupKind::Body, members).map_err(|error| error.to_string())?]
    };
    DebtRecord::new(NORMAL_FORM, groups, jobs)
        .and_then(|record| record.to_turtle())
        .map_err(|error| error.to_string())
}

fn evaluate(
    limits: MatchLimits,
    source: &str,
    rows: Vec<Row>,
    debt: Option<String>,
) -> Result<GateReport, String> {
    check(&gate(limits), &package(source, ledger(rows)?, debt))
}

fn sanctioned(
    limits: MatchLimits,
    source: &str,
    rows: Vec<Row>,
    debt: Option<String>,
    expected: Option<&str>,
) -> Result<(), String> {
    verdict(&evaluate(limits, source, rows, debt)?, expected)
}

/// A clean report, or one holding the expected finding.
fn verdict(report: &GateReport, expected: Option<&str>) -> Result<(), String> {
    expected.map_or_else(|| clean(report), |expected| finding(report, expected))
}

/// A clean report whose declared variant site is classified as a variant.
fn variant_verdict(report: &GateReport, expected: Option<&str>) -> Result<(), String> {
    verdict(report, expected)?;
    let classified = report
        .jobs()
        .iter()
        .flat_map(|job| &job.matches)
        .any(|found| found.class == MatchClass::Variant);
    if expected.is_some() || classified {
        Ok(())
    } else {
        Err("the declared variant site was not classified as a variant".into())
    }
}
