// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! One closed-shape validation path for parsed and caller-built reports.
use super::{
    Baseline, BaselineGroup, ConstantRow, Index, IndexGroup, IndexUnit, JobRow, MatchRow,
    MemberRow, ReportError as E, TableRow, mint, vocab,
};
use crate::{Fingerprint, GroupKind, MatchClass, TableKind};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Term {
    Iri(String),
    String(String),
    Integer(u128),
    Boolean(bool),
    Triple(String, String, String),
}
pub(super) type Node = BTreeMap<String, Vec<Term>>;
pub(super) type Graph = BTreeMap<String, Node>;
pub(super) fn iri(local: &str) -> Term {
    Term::Iri(format!("{}{local}", vocab::CENSUS))
}
pub(super) fn class(node: &Node) -> Result<&str, E> {
    match one(node, "type")? {
        Term::Iri(value) => value
            .strip_prefix(vocab::CENSUS)
            .ok_or_else(|| E::Class(value.clone())),
        value => Err(E::Class(format!("{value:?}"))),
    }
}
pub(super) fn one<'a>(node: &'a Node, key: &str) -> Result<&'a Term, E> {
    match node.get(key).map(Vec::as_slice) {
        None | Some([]) => Err(E::Missing(format!("census:{key}"))),
        Some([value]) => Ok(value),
        _ => Err(E::Repeated(format!("census:{key}"))),
    }
}
pub(super) fn string_ref<'a>(node: &'a Node, key: &str) -> Result<&'a str, E> {
    match one(node, key)? {
        Term::String(v) => Ok(v),
        _ => Err(E::Value(format!("census:{key}: string expected"))),
    }
}
pub(super) fn string(node: &Node, key: &str) -> Result<String, E> {
    string_ref(node, key).map(str::to_owned)
}
pub(super) fn number(node: &Node, key: &str) -> Result<u128, E> {
    match one(node, key)? {
        Term::Integer(v) => Ok(*v),
        _ => Err(E::Value(format!("census:{key}: integer expected"))),
    }
}
pub(super) fn count(node: &Node, key: &str) -> Result<usize, E> {
    usize::try_from(number(node, key)?)
        .map_err(|_| E::Value(format!("census:{key}: usize overflow")))
}
fn target(term: &Term) -> Result<&str, E> {
    match term {
        Term::Iri(v) => Ok(v),
        _ => Err(E::Value("IRI expected".into())),
    }
}
fn fp(term: &Term, nf: u32, kind: GroupKind) -> Result<Fingerprint, E> {
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
fn member(graph: &Graph, id: &str) -> Result<MemberRow, E> {
    let node = graph.get(id).ok_or_else(|| E::Dangling(id.into()))?;
    if class(node)? != "Unit" {
        return Err(E::Value(format!("{id}: unit expected")));
    }
    let row = MemberRow {
        symbol: string(node, "symbol")?,
        file: string(node, "file")?,
        line: count(node, "line")?,
        start_byte: count(node, "startByte")?,
    };
    if id != mint::unit(&row.symbol, &row.file, row.line, row.start_byte) {
        return Err(E::Name(id.into()));
    }
    Ok(row)
}
fn members(graph: &Graph, node: &Node) -> Result<Vec<MemberRow>, E> {
    let mut rows = node
        .get("member")
        .into_iter()
        .flatten()
        .map(|v| member(graph, target(v)?))
        .collect::<Result<Vec<_>, _>>()?;
    rows.sort_by(|a, b| {
        (&a.symbol, &a.file, a.line, a.start_byte).cmp(&(&b.symbol, &b.file, b.line, b.start_byte))
    });
    Ok(rows)
}
fn nf(graph: &Graph, kind: &str) -> Result<u32, E> {
    let root = graph
        .get("")
        .ok_or_else(|| E::Missing("report node <>".into()))?;
    let found = class(root)?;
    if found != kind {
        return Err(E::Kind {
            expected: kind.into(),
            found: found.into(),
        });
    }
    let nf = u32::try_from(number(root, "normalForm")?)
        .map_err(|_| E::Value("normalForm exceeds u32".into()))?;
    let form = u32::try_from(number(root, "reportForm")?)
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
    node: &Node,
    report: &str,
    nf: u32,
    referenced: &mut BTreeSet<String>,
) -> Result<(), E> {
    let class = class(node)?;
    let rows: Vec<_> = vocab::SHAPES
        .iter()
        .filter(|s| s.class == class && (s.report == report || s.report == "both"))
        .collect();
    if rows.is_empty() {
        return Err(E::Class(format!("{id}: census:{class}")));
    }
    for (key, values) in node {
        if key == "type" {
            continue;
        }
        let row = rows
            .iter()
            .find(|r| r.property == key)
            .ok_or_else(|| E::UnknownPredicate(format!("{id}: census:{key}")))?;
        if values.len() > row.max || values.iter().collect::<BTreeSet<_>>().len() != values.len() {
            return Err(E::Repeated(format!("{id}: census:{key}")));
        }
        for value in values {
            let valid = match row.object {
                "string" => matches!(value, Term::String(_)),
                "integer" | "u32" | "report version" => matches!(value, Term::Integer(_)),
                "positive integer" => matches!(value,Term::Integer(v) if *v>0),
                "large integer" => matches!(value,Term::Integer(v) if *v>0xffff),
                "package count" => matches!(value,Term::Integer(v) if *v>=2),
                "boolean" => matches!(value, Term::Boolean(_)),
                "unit" => {
                    let unit = target(value)?;
                    member(graph, unit)?;
                    referenced.insert(unit.to_owned());
                    true
                }
                "body fingerprint" => {
                    fp(value, nf, GroupKind::Body)?;
                    true
                }
                "shim fingerprint" => {
                    fp(value, nf, GroupKind::Shim)?;
                    true
                }
                "table" => {
                    matches!(value,Term::Iri(v) if v==&format!("{}hex-lower",vocab::TABLE)||v==&format!("{}hex-upper",vocab::TABLE))
                }
                "match class" => {
                    matches!(value,Term::Iri(v) if ["Home","Variant","Copy"].iter().any(|c|v==&format!("{}{c}",vocab::CENSUS)))
                }
                "triple term" => matches!(value, Term::Triple(_, _, _)),
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
        if node.get(row.property).map_or(0, Vec::len) < row.min {
            return Err(E::Missing(format!(
                "{id} census:{class}: census:{}",
                row.property
            )));
        }
    }
    Ok(())
}
fn validate(graph: &Graph, report: &str, nf: u32) -> Result<(), E> {
    // Check every class before following references, so a foreign class is
    // diagnosed at its node rather than disguised as a missing unit.
    for (id, node) in graph {
        let class = class(node)?;
        if !vocab::SHAPES
            .iter()
            .any(|row| row.class == class && (row.report == report || row.report == "both"))
        {
            return Err(E::Class(format!("{id}: census:{class}")));
        }
    }
    let mut referenced = BTreeSet::new();
    let mut matches = BTreeMap::new();
    let mut reified = BTreeSet::new();
    let mut copies_by_job: BTreeMap<String, usize> = BTreeMap::new();
    for (id, node) in graph {
        let class = class(node)?;
        validate_properties(graph, id, node, report, nf, &mut referenced)?;
        match class {
            "Baseline" | "Index" if !id.is_empty() => return Err(E::Name(id.clone())),
            "Unit" => {
                member(graph, id)?;
                if node.contains_key("holds") || report == "index" {
                    referenced.insert(id.clone());
                }
            }
            "BodyGroup" | "ShimGroup" => {
                let kind = if class == "BodyGroup" {
                    GroupKind::Body
                } else {
                    GroupKind::Shim
                };
                let fingerprint = fp(one(node, "fingerprint")?, nf, kind)?;
                if id != &format!("#group/{fingerprint}") {
                    return Err(E::Name(id.clone()));
                }
            }
            "SharedConstant" => {
                if id != &format!("#constant/{}", number(node, "value")?) {
                    return Err(E::Name(id.clone()));
                }
                if count(node, "packageCount")? > node["member"].len() {
                    return Err(E::Inconsistent(format!(
                        "{id}: packageCount exceeds members"
                    )));
                }
            }
            "Job" => {
                let job = string(node, "jobId")?;
                if id != &mint::job(&job) {
                    return Err(E::Name(id.clone()));
                }
                for unit in node.get("match").into_iter().flatten() {
                    matches.insert((id.clone(), target(unit)?.to_owned()), job.clone());
                }
            }
            "Match" => {
                let Term::Triple(job, predicate, unit) = one(node, "reifies")? else {
                    return Err(E::Value(format!("{id}: rdf:reifies")));
                };
                if predicate != &format!("{}match", vocab::CENSUS) {
                    return Err(E::Inconsistent(format!("{id}: rdf:reifies predicate")));
                }
                let job_node = graph.get(job).ok_or_else(|| E::Dangling(job.clone()))?;
                let job_id = string(job_node, "jobId")?;
                if id != &mint::reifier(&job_id, unit) {
                    return Err(E::Name(id.clone()));
                }
                if one(node, "matchClass")? == &iri("Copy") {
                    *copies_by_job.entry(job.clone()).or_default() += 1;
                }
                if !reified.insert((job.clone(), unit.clone())) {
                    return Err(E::Repeated(format!("{id}: reifier")));
                }
            }
            _ => {}
        }
    }
    for (id, node) in graph {
        if class(node)? == "Unit" && !referenced.contains(id) {
            return Err(E::Orphan(id.clone()));
        }
        if class(node)? == "Job" {
            let copies = copies_by_job.get(id).copied().unwrap_or(0);
            if copies != count(node, "copies")? {
                return Err(E::Inconsistent(format!("{id}: copies")));
            }
        }
    }
    if matches.keys().cloned().collect::<BTreeSet<_>>() != reified {
        return Err(E::Inconsistent(
            "every asserted match requires exactly one named reifier".into(),
        ));
    }
    Ok(())
}
pub(super) fn baseline(graph: &Graph) -> Result<Baseline, E> {
    let nf = nf(graph, "Baseline")?;
    validate(graph, "baseline", nf)?;
    let root = &graph[""];
    let mut report = Baseline {
        normal_form: nf,
        min_tokens: count(root, "minTokens")?,
        units: count(root, "unitCount")?,
        body_groups: Vec::new(),
        shim_groups: Vec::new(),
        hex_tables: Vec::new(),
        shared_constants: Vec::new(),
    };
    for (id, node) in graph {
        match class(node)? {
            "BodyGroup" | "ShimGroup" => {
                let body = class(node)? == "BodyGroup";
                let kind = if body {
                    GroupKind::Body
                } else {
                    GroupKind::Shim
                };
                let row = BaselineGroup {
                    fingerprint: fp(one(node, "fingerprint")?, nf, kind)?,
                    tokens: count(node, "tokens")?,
                    members: members(graph, node)?,
                };
                if body {
                    report.body_groups.push(row);
                } else {
                    report.shim_groups.push(row);
                }
            }
            "Unit" => {
                let m = member(graph, id)?;
                for value in node.get("holds").into_iter().flatten() {
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
                value: number(node, "value")?,
                packages: count(node, "packageCount")?,
                members: members(graph, node)?,
            }),
            _ => {}
        }
    }
    if report.units
        < graph
            .values()
            .filter(|n| class(n).ok() == Some("Unit"))
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
pub(super) fn index(graph: &Graph) -> Result<Index, E> {
    let nf = nf(graph, "Index")?;
    validate(graph, "index", nf)?;
    let mut report = Index {
        normal_form: nf,
        units: Vec::new(),
        groups: Vec::new(),
        jobs: Vec::new(),
    };
    for (id, node) in graph {
        match class(node)? {
            "Unit" => {
                let m = member(graph, id)?;
                let optional = |key, kind| node.get(key).map(|v| fp(&v[0], nf, kind)).transpose();
                let Term::Boolean(shipping) = one(node, "shipping")? else {
                    return Err(E::Value(format!("{id}: shipping")));
                };
                report.units.push(IndexUnit {
                    symbol: m.symbol,
                    file: m.file,
                    line: m.line,
                    start_byte: m.start_byte,
                    package: string(node, "package")?,
                    shipping: *shipping,
                    tokens: count(node, "tokens")?,
                    body: optional("body", GroupKind::Body)?,
                    shim: optional("shim", GroupKind::Shim)?,
                });
            }
            "BodyGroup" | "ShimGroup" => {
                let kind = if class(node)? == "BodyGroup" {
                    GroupKind::Body
                } else {
                    GroupKind::Shim
                };
                report.groups.push(IndexGroup {
                    kind,
                    fingerprint: fp(one(node, "fingerprint")?, nf, kind)?,
                    tokens: count(node, "tokens")?,
                    members: members(graph, node)?,
                });
            }
            "Job" => {
                let job_id = string(node, "jobId")?;
                let mut found = Vec::new();
                for v in node.get("match").into_iter().flatten() {
                    let unit = target(v)?;
                    let m = member(graph, unit)?;
                    let reifier = &graph[&mint::reifier(&job_id, unit)];
                    let class =
                        match target(one(reifier, "matchClass")?)?.strip_prefix(vocab::CENSUS) {
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
                    home: string(node, "homeSymbol")?,
                    matches: found,
                    copies: count(node, "copies")?,
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
    #[test]
    fn every_required_shape_property_has_a_missing_and_unknown_neighbor() {
        let baseline =
            super::super::read::parse(include_str!("../../tests/fixtures/reports/baseline.ttl"))
                .expect("baseline")
                .0;
        let index =
            super::super::read::parse(include_str!("../../tests/fixtures/reports/index.ttl"))
                .expect("index")
                .0;
        let mut baseline = baseline;
        let mut constant = Node::from([
            ("type".into(), vec![iri("SharedConstant")]),
            ("value".into(), vec![Term::Integer(65536)]),
            ("packageCount".into(), vec![Term::Integer(2)]),
        ]);
        constant.insert(
            "member".into(),
            baseline
                .values()
                .find(|n| class(n).ok() == Some("BodyGroup"))
                .expect("group")["member"]
                .clone(),
        );
        baseline.insert("#constant/65536".into(), constant);
        assert!(super::baseline(&baseline).is_ok());
        assert!(super::index(&index).is_ok());
        for row in vocab::SHAPES.iter().filter(|r| r.report != "debt") {
            for (report, graph) in [("baseline", &baseline), ("index", &index)] {
                if row.report != "both" && row.report != report {
                    continue;
                }
                let id = graph
                    .iter()
                    .find(|(_, n)| class(n).ok() == Some(row.class))
                    .map(|(id, _)| id.clone())
                    .expect("each class has a fixture");
                let mut unknown = graph.clone();
                unknown
                    .get_mut(&id)
                    .expect("node")
                    .insert("foreign".into(), vec![Term::Integer(1)]);
                assert!(
                    matches!(validate(&unknown, report, 1), Err(E::UnknownPredicate(_))),
                    "{}",
                    row.class
                );
                if row.min > 0 {
                    let mut missing = graph.clone();
                    missing.get_mut(&id).expect("node").remove(row.property);
                    assert!(
                        matches!(validate(&missing, report, 1), Err(E::Missing(_))),
                        "{} {}",
                        row.class,
                        row.property
                    );
                }
            }
        }
    }
}
