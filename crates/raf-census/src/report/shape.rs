// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! One closed-shape validation path for parsed and caller-built reports.
use super::{
    Baseline, BaselineGroup, ConstantRow, Index, IndexGroup, IndexUnit, JobRow, MatchRow,
    MemberRow, ReportError as E, TableRow, mint, vocab,
};
use crate::{Fingerprint, GroupKind, MatchClass, TableKind};
use std::collections::{BTreeMap, BTreeSet};

pub(super) use raf_rdf::TermValue as Term;
use raf_rdf::{Graph, Node, Value};
pub(crate) fn property(key: &str) -> String {
    match key {
        "type" => raf_rdf::vocab::TYPE.into(),
        "reifies" => raf_rdf::vocab::REIFIES.into(),
        _ => format!("{}{key}", vocab::CENSUS),
    }
}
pub(crate) fn integer(value: u128) -> Term {
    Term::typed_literal(
        purrdf_xsd::exact::Integer::from(value).canonical_lexical(),
        raf_rdf::vocab::INTEGER,
    )
}
pub(crate) fn iri(local: &str) -> Term {
    Term::iri(format!("{}{local}", vocab::CENSUS))
}
pub(super) fn integer_value(value: Value<'_>) -> Option<u128> {
    let (lexical, datatype) = value.literal()?;
    if datatype != raf_rdf::vocab::INTEGER || lexical.len() > 39 {
        return None;
    }
    let integer = purrdf_xsd::numeric::parse_integer(lexical).ok()?;
    if integer.is_negative() || integer.canonical_lexical() != lexical {
        return None;
    }
    integer.into_bigint().unsigned_abs_u128()
}
pub(super) fn boolean_value(value: Value<'_>) -> Option<bool> {
    let (lexical, datatype) = value.literal()?;
    if datatype != raf_rdf::vocab::BOOLEAN {
        return None;
    }
    let parsed = purrdf_xsd::simple::read_boolean(lexical).ok()?;
    if Term::boolean(parsed) == value.to_owned() {
        Some(parsed)
    } else {
        None
    }
}
fn is_string(value: Value<'_>) -> bool {
    value
        .literal()
        .is_some_and(|(_, dt)| dt == raf_rdf::vocab::STRING)
}
pub(super) fn quoted(value: Value<'_>) -> Result<(&str, &str, &str), E> {
    if let Some((s, p, o)) = value.quoted()
        && let (Some(s), Some(p), Some(o)) = (s.iri(), p.iri(), o.iri())
    {
        return Ok((s, p, o));
    }
    Err(E::Value(
        "census reifies requires three IRI positions".into(),
    ))
}
pub(crate) fn class<'a>(node: &Node<'a>) -> Result<&'a str, E> {
    one(node, "type")?
        .iri()
        .and_then(|v| v.strip_prefix(vocab::CENSUS))
        .ok_or_else(|| E::Class("class IRI expected".into()))
}
pub(crate) fn one<'a>(node: &Node<'a>, key: &str) -> Result<Value<'a>, E> {
    let mut values = node.values(&property(key));
    match (values.next(), values.next()) {
        (None, _) => Err(E::Missing(format!("census:{key}"))),
        (Some(value), None) => Ok(value),
        _ => Err(E::Repeated(format!("census:{key}"))),
    }
}
pub(crate) fn string(node: &Node<'_>, key: &str) -> Result<String, E> {
    let value = one(node, key)?;
    if let Some((text, datatype)) = value.literal()
        && datatype == raf_rdf::vocab::STRING
    {
        return Ok(text.to_owned());
    }
    Err(E::Value(format!("census:{key}: string expected")))
}
pub(super) fn number(node: &Node<'_>, key: &str) -> Result<u128, E> {
    integer_value(one(node, key)?).ok_or_else(|| {
        E::Value(format!(
            "census:{key}: canonical nonnegative integer expected"
        ))
    })
}
pub(crate) fn count(node: &Node<'_>, key: &str) -> Result<usize, E> {
    usize::try_from(number(node, key)?)
        .map_err(|_| E::Value(format!("census:{key}: usize overflow")))
}
fn target(value: Value<'_>) -> Result<&str, E> {
    value.iri().ok_or_else(|| E::Value("IRI expected".into()))
}
pub(super) fn fp(term: Value<'_>, nf: u32, kind: GroupKind) -> Result<Fingerprint, E> {
    let value = target(term)?;
    let prefix = vocab::fingerprint_namespace(nf, kind);
    let hex = value
        .strip_prefix(&prefix)
        .ok_or_else(|| E::Inconsistent(format!("fingerprint {value}, normalForm {nf}")))?;
    let name = match kind {
        GroupKind::Body => "body",
        GroupKind::Shim => "shim",
    };
    let fingerprint = format!("{name}:{hex}")
        .parse()
        .map_err(|_| E::Value(value.into()))?;
    Ok(fingerprint)
}
pub(super) fn member(graph: &Graph, id: &str) -> Result<MemberRow, E> {
    let node = graph.get(id).ok_or_else(|| E::Dangling(id.into()))?;
    if class(&node)? != "Unit" {
        return Err(E::Value(format!("{id}: unit expected")));
    }
    let row = MemberRow {
        symbol: string(&node, "symbol")?,
        file: string(&node, "file")?,
        line: count(&node, "line")?,
        start_byte: count(&node, "startByte")?,
    };
    if id != mint::unit(&row.symbol, &row.file, row.line, row.start_byte) {
        return Err(E::Name(id.into()));
    }
    Ok(row)
}
fn members(graph: &Graph, node: &Node<'_>) -> Result<Vec<MemberRow>, E> {
    let mut rows = node
        .values(&property("member"))
        .map(|v| member(graph, target(v)?))
        .collect::<Result<Vec<_>, _>>()?;
    rows.sort_by(|a, b| {
        (&a.symbol, &a.file, a.line, a.start_byte).cmp(&(&b.symbol, &b.file, b.line, b.start_byte))
    });
    Ok(rows)
}
pub(super) fn nf(graph: &Graph, kind: &str) -> Result<u32, E> {
    let root = graph
        .get("")
        .ok_or_else(|| E::Missing("report node <>".into()))?;
    let found = class(&root)?;
    if found != kind {
        return Err(E::Kind {
            expected: kind.into(),
            found: found.into(),
        });
    }
    let nf = u32::try_from(number(&root, "normalForm")?)
        .map_err(|_| E::Value("normalForm exceeds u32".into()))?;
    let form = u32::try_from(number(&root, "reportForm")?)
        .map_err(|_| E::Value("reportForm exceeds u32".into()))?;
    if form != vocab::REPORT_FORM {
        return Err(E::Form {
            declared: form,
            implemented: vocab::REPORT_FORM,
        });
    }
    Ok(nf)
}
fn validate_properties(
    graph: &Graph,
    id: &str,
    node: &Node<'_>,
    report: &str,
    nf: u32,
    referenced: &mut BTreeSet<String>,
) -> Result<(), E> {
    let class = class(node)?;
    let rows: Vec<_> = vocab::SHAPES
        .iter()
        .filter(|s| s.class == class && applies(s, report))
        .collect();
    if rows.is_empty() {
        return Err(E::Class(format!("{id}: census:{class}")));
    }
    for predicate in node.properties() {
        let key = predicate;
        let key = if key == raf_rdf::vocab::TYPE {
            "type"
        } else if key == raf_rdf::vocab::REIFIES {
            "reifies"
        } else {
            key.strip_prefix(vocab::CENSUS)
                .ok_or_else(|| E::UnknownPredicate(key.to_owned()))?
        };
        if key == "type" {
            continue;
        }
        let row = rows
            .iter()
            .find(|r| r.property == key)
            .ok_or_else(|| E::UnknownPredicate(format!("{id}: census:{key}")))?;
        if node.values(predicate).count() > row.max {
            return Err(E::Repeated(format!("{id}: census:{key}")));
        }
        for value in node.values(predicate) {
            let valid = match row.object {
                "string" => is_string(value),
                "integer" | "u32" | "report version" => integer_value(value).is_some(),
                "positive integer" => integer_value(value).is_some_and(|v| v > 0),
                "large integer" => integer_value(value).is_some_and(|v| v > 0xffff),
                "package count" => integer_value(value).is_some_and(|v| v >= 2),
                "boolean" => boolean_value(value).is_some(),
                "unit" => {
                    let unit = target(value)?;
                    member(graph, unit)?;
                    referenced.insert(unit.to_owned());
                    true
                }
                "job" => class_link(graph, value, "Job", referenced)?,
                "reference source" => class_link(graph, value, "ReferenceSource", referenced)?,
                "reference fingerprint" => {
                    fp(value, nf, GroupKind::Body).or_else(|_| fp(value, nf, GroupKind::Shim))?;
                    true
                }
                "multiplicity" => class_link(graph, value, "MemberMultiplicity", referenced)?,
                "body fingerprint" => {
                    fp(value, nf, GroupKind::Body)?;
                    true
                }
                "shim fingerprint" => {
                    fp(value, nf, GroupKind::Shim)?;
                    true
                }
                "table" => value.iri().is_some_and(|v| {
                    v == format!("{}hex-lower", vocab::TABLE)
                        || v == format!("{}hex-upper", vocab::TABLE)
                }),
                "match class" => value.iri().is_some_and(|v| {
                    ["Home", "Variant", "Copy"]
                        .iter()
                        .any(|c| v == format!("{}{c}", vocab::CENSUS))
                }),
                "triple term" => quoted(value).is_ok(),
                _ => false,
            };
            if !valid {
                return Err(E::Value(format!(
                    "{id}: census:{key}: {} expected",
                    row.object
                )));
            }
        }
    }
    for row in rows {
        if node.values(&property(row.property)).count() < row.min {
            return Err(E::Missing(format!(
                "{id} census:{class}: census:{}",
                row.property
            )));
        }
    }
    Ok(())
}
fn class_link(
    graph: &Graph,
    value: Value<'_>,
    expected: &str,
    referenced: &mut BTreeSet<String>,
) -> Result<bool, E> {
    let id = target(value)?;
    let row = graph.get(id).ok_or_else(|| E::Dangling(id.into()))?;
    if class(&row)? != expected {
        return Err(E::Class(format!("{id}: {expected} expected")));
    }
    referenced.insert(id.to_owned());
    Ok(true)
}
fn applies(row: &vocab::Shape, report: &str) -> bool {
    row.report == report
        || (row.report == "both"
            && (matches!(report, "baseline" | "index")
                || (matches!(report, "job-query" | "fingerprint-query") && row.class == "Unit")))
        || (row.report == "index"
            && ((row.class == "Unit" && matches!(report, "job-query" | "fingerprint-query"))
                || (matches!(row.class, "Job" | "Match") && report == "job-query")
                || (matches!(row.class, "ReferenceSource" | "ReferenceWitness")
                    && matches!(report, "job-query" | "fingerprint-query"))))
}
pub(super) fn validate(graph: &Graph, report: &str, nf: u32) -> Result<(), E> {
    // Check every class before following references, so a foreign class is
    // diagnosed at its node rather than disguised as a missing unit.
    for (id, node) in graph.nodes() {
        let class = class(&node)?;
        if !vocab::SHAPES
            .iter()
            .any(|row| row.class == class && applies(row, report))
        {
            return Err(E::Class(format!("{id}: census:{class}")));
        }
    }
    let mut referenced = BTreeSet::new();
    let mut matches = BTreeMap::new();
    let mut reified = BTreeSet::new();
    let mut copies_by_job: BTreeMap<String, usize> = BTreeMap::new();
    for (id, node) in graph.nodes() {
        let class = class(&node)?;
        validate_properties(graph, &id, &node, report, nf, &mut referenced)?;
        match class {
            "Baseline" | "Index" | "JobQuery" | "CountQuery" | "FingerprintQuery"
                if !id.is_empty() =>
            {
                return Err(E::Name(id.clone()));
            }
            "Unit" => {
                member(graph, &id)?;
                if node.values(&property("holds")).next().is_some() || report == "index" {
                    referenced.insert(id.clone());
                }
            }
            "BodyGroup" | "ShimGroup" => {
                let kind = if class == "BodyGroup" {
                    GroupKind::Body
                } else {
                    GroupKind::Shim
                };
                let fingerprint = fp(one(&node, "fingerprint")?, nf, kind)?;
                if id != format!("#group/{fingerprint}") {
                    return Err(E::Name(id.clone()));
                }
            }
            "SharedConstant" => {
                if id != format!("#constant/{}", number(&node, "value")?) {
                    return Err(E::Name(id.clone()));
                }
                if count(&node, "packageCount")? > node.values(&property("member")).count() {
                    return Err(E::Inconsistent(format!(
                        "{id}: packageCount exceeds members"
                    )));
                }
            }
            "Job" => {
                let job = string(&node, "jobId")?;
                if id != mint::job(&job) {
                    return Err(E::Name(id.clone()));
                }
                for unit in node.values(&property("match")) {
                    matches.insert((id.clone(), target(unit)?.to_owned()), job.clone());
                }
            }
            "Match" => {
                let (job, predicate, unit) = quoted(one(&node, "reifies")?)?;
                if predicate != format!("{}match", vocab::CENSUS) {
                    return Err(E::Inconsistent(format!("{id}: rdf:reifies predicate")));
                }
                let job_node = graph.get(job).ok_or_else(|| E::Dangling(job.to_owned()))?;
                let job_id = string(&job_node, "jobId")?;
                if id != mint::reifier(&job_id, unit) {
                    return Err(E::Name(id.clone()));
                }
                if one(&node, "matchClass")?.iri()
                    == Some(format!("{}Copy", vocab::CENSUS).as_str())
                {
                    *copies_by_job.entry(job.to_owned()).or_default() += 1;
                }
                if !reified.insert((job.to_owned(), unit.to_owned())) {
                    return Err(E::Repeated(format!("{id}: reifier")));
                }
            }
            _ => {}
        }
    }
    for (id, node) in graph.nodes() {
        if matches!(class(&node)?, "Unit" | "MemberMultiplicity") && !referenced.contains(&id) {
            return Err(E::Orphan(id.clone()));
        }
        if class(&node)? == "Job" {
            let copies = copies_by_job.get(&id).copied().unwrap_or(0);
            if copies != count(&node, "copies")? {
                return Err(E::Inconsistent(format!("{id}: copies")));
            }
        }
    }
    if matches.keys().cloned().collect::<BTreeSet<_>>() != reified {
        return Err(E::Inconsistent(
            "every asserted match requires exactly one named reifier".into(),
        ));
    }
    super::reference::validate(graph, nf)?;
    Ok(())
}
pub(super) fn baseline(graph: &Graph) -> Result<Baseline, E> {
    let nf = nf(graph, "Baseline")?;
    validate(graph, "baseline", nf)?;
    let root = graph.get("").expect("validated report root");
    let mut report = Baseline {
        normal_form: nf,
        min_tokens: count(&root, "minTokens")?,
        units: count(&root, "unitCount")?,
        body_groups: Vec::new(),
        shim_groups: Vec::new(),
        hex_tables: Vec::new(),
        shared_constants: Vec::new(),
    };
    for (id, node) in graph.nodes() {
        match class(&node)? {
            "BodyGroup" | "ShimGroup" => {
                let body = class(&node)? == "BodyGroup";
                let kind = if body {
                    GroupKind::Body
                } else {
                    GroupKind::Shim
                };
                let row = BaselineGroup {
                    fingerprint: fp(one(&node, "fingerprint")?, nf, kind)?,
                    tokens: count(&node, "tokens")?,
                    members: members(graph, &node)?,
                };
                if body {
                    report.body_groups.push(row);
                } else {
                    report.shim_groups.push(row);
                }
            }
            "Unit" => {
                let m = member(graph, &id)?;
                for value in node.values(&property("holds")) {
                    let table = if target(value)? == format!("{}hex-lower", vocab::TABLE) {
                        TableKind::HexLower
                    } else {
                        TableKind::HexUpper
                    };
                    report.hex_tables.push(TableRow {
                        symbol: m.symbol.clone(),
                        file: m.file.clone(),
                        line: m.line,
                        start_byte: m.start_byte,
                        table,
                    });
                }
            }
            "SharedConstant" => report.shared_constants.push(ConstantRow {
                value: number(&node, "value")?,
                packages: count(&node, "packageCount")?,
                members: members(graph, &node)?,
            }),
            _ => {}
        }
    }
    if report.units
        < graph
            .nodes()
            .filter(|(_, n)| class(n).ok() == Some("Unit"))
            .count()
    {
        return Err(E::Inconsistent("unitCount below listed units".into()));
    }
    sort_groups(&mut report.body_groups);
    sort_groups(&mut report.shim_groups);
    report.shared_constants.sort_by_key(|r| r.value);
    report.hex_tables.sort_by(|a, b| {
        (&a.symbol, &a.file, a.line, a.start_byte, a.table).cmp(&(
            &b.symbol,
            &b.file,
            b.line,
            b.start_byte,
            b.table,
        ))
    });
    Ok(report)
}
fn sort_groups(groups: &mut [BaselineGroup]) {
    groups.sort_by(|a, b| {
        b.members
            .len()
            .cmp(&a.members.len())
            .then(b.tokens.cmp(&a.tokens))
            .then(a.fingerprint.cmp(&b.fingerprint))
    });
}
fn index_unit(graph: &Graph, id: &str, node: &Node<'_>, nf: u32) -> Result<IndexUnit, E> {
    let m = member(graph, id)?;
    let optional = |key, kind| {
        node.values(&property(key))
            .next()
            .map(|v| fp(v, nf, kind))
            .transpose()
    };
    let Some(shipping) = boolean_value(one(node, "shipping")?) else {
        return Err(E::Value(format!("{id}: shipping")));
    };
    Ok(IndexUnit {
        reference_source: node
            .values(&property("referenceSource"))
            .next()
            .map(|value| target(value).map(str::to_owned))
            .transpose()?,
        symbol: m.symbol,
        file: m.file,
        line: m.line,
        start_byte: m.start_byte,
        package: string(node, "package")?,
        shipping,
        tokens: count(node, "tokens")?,
        body: optional("body", GroupKind::Body)?,
        shim: optional("shim", GroupKind::Shim)?,
    })
}
pub(super) fn index(graph: &Graph) -> Result<Index, E> {
    let nf = nf(graph, "Index")?;
    validate(graph, "index", nf)?;
    let mut report = Index {
        normal_form: nf,
        reference_sources: Vec::new(),
        reference_witnesses: Vec::new(),
        units: Vec::new(),
        groups: Vec::new(),
        jobs: Vec::new(),
    };
    for (id, node) in graph.nodes() {
        match class(&node)? {
            "ReferenceSource" => report
                .reference_sources
                .push(super::reference::read_source(&node)?),
            "ReferenceWitness" => report
                .reference_witnesses
                .push(super::reference::read_witness(graph, &node, nf)?),
            "Unit" => {
                report.units.push(index_unit(graph, &id, &node, nf)?);
            }
            "BodyGroup" | "ShimGroup" => {
                let kind = if class(&node)? == "BodyGroup" {
                    GroupKind::Body
                } else {
                    GroupKind::Shim
                };
                report.groups.push(IndexGroup {
                    kind,
                    fingerprint: fp(one(&node, "fingerprint")?, nf, kind)?,
                    tokens: count(&node, "tokens")?,
                    members: members(graph, &node)?,
                });
            }
            "Job" => {
                let job_id = string(&node, "jobId")?;
                let mut found = Vec::new();
                for v in node.values(&property("match")) {
                    let unit = target(v)?;
                    let m = member(graph, unit)?;
                    let reifier = graph
                        .get(&mint::reifier(&job_id, unit))
                        .expect("validated named reifier");
                    let class =
                        match target(one(&reifier, "matchClass")?)?.strip_prefix(vocab::CENSUS) {
                            Some("Home") => MatchClass::Home,
                            Some("Variant") => MatchClass::Variant,
                            Some("Copy") => MatchClass::Copy,
                            _ => return Err(E::Value("matchClass".into())),
                        };
                    found.push(MatchRow {
                        symbol: m.symbol,
                        file: m.file,
                        line: m.line,
                        start_byte: m.start_byte,
                        class,
                    });
                }
                found.sort_by(|a, b| {
                    (&a.symbol, &a.file, a.line, a.start_byte).cmp(&(
                        &b.symbol,
                        &b.file,
                        b.line,
                        b.start_byte,
                    ))
                });
                report.jobs.push(JobRow {
                    id: job_id,
                    home: string(&node, "homeSymbol")?,
                    matches: found,
                    copies: count(&node, "copies")?,
                });
            }
            _ => {}
        }
    }
    report.units.sort_by(|a, b| {
        (&a.symbol, &a.file, a.line, a.start_byte).cmp(&(&b.symbol, &b.file, b.line, b.start_byte))
    });
    report.groups.sort_by(|a, b| {
        b.members
            .len()
            .cmp(&a.members.len())
            .then(b.tokens.cmp(&a.tokens))
            .then(a.fingerprint.cmp(&b.fingerprint))
    });
    report.jobs.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use raf_rdf::{Builder, Format, Triple};

    #[test]
    fn every_required_shape_property_has_a_missing_and_unknown_neighbor() {
        let baseline = super::super::read::parse(
            include_str!("../../tests/fixtures/reports/baseline.ttl"),
            Format::Turtle,
        )
        .expect("baseline");
        let index = super::super::read::parse(
            include_str!("../../tests/fixtures/reports/index.ttl"),
            Format::Turtle,
        )
        .expect("index");
        let members = baseline
            .nodes()
            .find(|(_, n)| class(n).ok() == Some("BodyGroup"))
            .expect("group")
            .1
            .values(&property("member"));
        let mut builder = Builder::from_graph(&baseline);
        for (key, value) in [
            ("type", iri("SharedConstant")),
            ("value", integer(65536)),
            ("packageCount", integer(2)),
        ] {
            builder
                .insert(Triple {
                    subject: Term::iri("#constant/65536"),
                    predicate: Term::iri(property(key)),
                    object: value,
                })
                .unwrap();
        }
        for member in members {
            builder
                .insert(Triple {
                    subject: Term::iri("#constant/65536"),
                    predicate: Term::iri(property("member")),
                    object: member.to_owned(),
                })
                .unwrap();
        }
        let baseline = builder.finish().unwrap();
        assert!(super::baseline(&baseline).is_ok());
        assert!(super::index(&index).is_ok());
        for row in vocab::SHAPES.iter().filter(|r| r.report != "debt") {
            for (report, graph) in [("baseline", &baseline), ("index", &index)] {
                if row.report != "both" && row.report != report {
                    continue;
                }
                let (id, node) = graph
                    .nodes()
                    .find(|(_, n)| class(n).ok() == Some(row.class))
                    .expect("each class has a fixture");
                let mut unknown = Builder::from_graph(graph);
                unknown
                    .insert(Triple {
                        subject: Term::iri(&id),
                        predicate: Term::iri(property("foreign")),
                        object: integer(1),
                    })
                    .unwrap();
                assert!(
                    matches!(
                        validate(&unknown.finish().unwrap(), report, 1),
                        Err(E::UnknownPredicate(_))
                    ),
                    "{}",
                    row.class
                );
                if row.min > 0 {
                    let mut missing = Builder::from_graph(graph);
                    for value in node.values(&property(row.property)) {
                        assert!(
                            missing
                                .remove(Triple {
                                    subject: Term::iri(&id),
                                    predicate: Term::iri(property(row.property)),
                                    object: value.to_owned()
                                })
                                .unwrap()
                        );
                    }
                    assert!(
                        matches!(
                            validate(&missing.finish().unwrap(), report, 1),
                            Err(E::Missing(_))
                        ),
                        "{} {}",
                        row.class,
                        row.property
                    );
                }
            }
        }
    }
}
