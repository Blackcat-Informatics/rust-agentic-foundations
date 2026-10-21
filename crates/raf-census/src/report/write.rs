// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Canonical Turtle 1.2 and its unkeyed content digest.
use super::{
    Baseline, Index, MemberRow, ReportError as E, mint,
    shape::{self, Graph, Node, Term},
    vocab,
};
use crate::{Error, Fingerprint, GroupKind, MatchClass};
use purrdf_lex::term_syntax;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

fn node(class: &str) -> Node {
    BTreeMap::from([("type".into(), vec![shape::iri(class)])])
}
fn put(node: &mut Node, key: &str, value: Term) {
    node.entry(key.into()).or_default().push(value);
}
fn num(node: &mut Node, key: &str, value: usize) {
    put(node, key, Term::Integer(value as u128));
}
fn text(node: &mut Node, key: &str, value: &str) {
    put(node, key, Term::String(value.into()));
}
fn fingerprint(value: &Fingerprint, nf: u32) -> Result<Term, E> {
    let (kind, hex) = match value {
        Fingerprint::Body(v) => (GroupKind::Body, v),
        Fingerprint::Shim(v) => (GroupKind::Shim, v),
        _ => return Err(E::Value(format!("group fingerprint {value}"))),
    };
    Ok(Term::Iri(format!(
        "{}{}",
        vocab::fingerprint_namespace(nf, kind),
        blake3::Hash::from_bytes(*hex)
    )))
}
fn unit(graph: &mut Graph, row: &MemberRow) -> Result<String, E> {
    let id = mint::unit(&row.symbol, &row.file, row.line, row.start_byte);
    let mut value = node("Unit");
    text(&mut value, "symbol", &row.symbol);
    text(&mut value, "file", &row.file);
    num(&mut value, "line", row.line);
    num(&mut value, "startByte", row.start_byte);
    if let Some(old) = graph.get(&id) {
        for (key, v) in &value {
            if old.get(key) != Some(v) {
                return Err(E::Inconsistent(format!("{id}: {key}")));
            }
        }
    } else {
        graph.insert(id.clone(), value);
    }
    Ok(id)
}
fn group(
    graph: &mut Graph,
    fp: &Fingerprint,
    tokens: usize,
    members: &[MemberRow],
    nf: u32,
) -> Result<(), E> {
    let mut value = node(match fp {
        Fingerprint::Body(_) => "BodyGroup",
        Fingerprint::Shim(_) => "ShimGroup",
        _ => return Err(E::Value(fp.to_string())),
    });
    put(&mut value, "fingerprint", fingerprint(fp, nf)?);
    num(&mut value, "tokens", tokens);
    for member in members {
        put(&mut value, "member", Term::Iri(unit(graph, member)?));
    }
    let id = format!("#group/{fp}");
    if graph.insert(id.clone(), value).is_some() {
        return Err(E::Repeated(id));
    }
    Ok(())
}
pub(super) fn baseline_graph(report: &Baseline) -> Result<Graph, E> {
    let mut graph = Graph::new();
    let mut root = node("Baseline");
    num(&mut root, "normalForm", report.normal_form as usize);
    num(&mut root, "reportForm", vocab::REPORT_FORM as usize);
    num(&mut root, "unitCount", report.units);
    num(&mut root, "minTokens", report.min_tokens);
    graph.insert(String::new(), root);
    for (groups, kind) in [
        (&report.body_groups, GroupKind::Body),
        (&report.shim_groups, GroupKind::Shim),
    ] {
        for g in groups {
            if !matches!(
                (&g.fingerprint, kind),
                (Fingerprint::Body(_), GroupKind::Body) | (Fingerprint::Shim(_), GroupKind::Shim)
            ) {
                return Err(E::Inconsistent("group kind and fingerprint".into()));
            }
            group(
                &mut graph,
                &g.fingerprint,
                g.tokens,
                &g.members,
                report.normal_form,
            )?;
        }
    }
    for c in &report.shared_constants {
        let mut value = node("SharedConstant");
        put(&mut value, "value", Term::Integer(c.value));
        num(&mut value, "packageCount", c.packages);
        for m in &c.members {
            put(&mut value, "member", Term::Iri(unit(&mut graph, m)?));
        }
        let id = format!("#constant/{}", c.value);
        if graph.insert(id.clone(), value).is_some() {
            return Err(E::Repeated(id));
        }
    }
    for t in &report.hex_tables {
        let id = unit(
            &mut graph,
            &MemberRow {
                symbol: t.symbol.clone(),
                file: t.file.clone(),
                line: t.line,
                start_byte: t.start_byte,
            },
        )?;
        put(
            graph.get_mut(&id).expect("inserted"),
            "holds",
            Term::Iri(format!("{}{}", vocab::TABLE, t.table.name())),
        );
    }
    Ok(graph)
}
pub(super) fn index_graph(report: &Index) -> Result<Graph, E> {
    let mut graph = Graph::new();
    let mut root = node("Index");
    num(&mut root, "normalForm", report.normal_form as usize);
    num(&mut root, "reportForm", vocab::REPORT_FORM as usize);
    graph.insert(String::new(), root);
    for u in &report.units {
        let row = MemberRow {
            symbol: u.symbol.clone(),
            file: u.file.clone(),
            line: u.line,
            start_byte: u.start_byte,
        };
        let id = mint::unit(&row.symbol, &row.file, row.line, row.start_byte);
        if graph.contains_key(&id) {
            return Err(E::Repeated(id));
        }
        unit(&mut graph, &row)?;
        let value = graph.get_mut(&id).expect("inserted");
        text(value, "package", &u.package);
        num(value, "tokens", u.tokens);
        put(value, "shipping", Term::Boolean(u.shipping));
        if let Some(fp) = &u.body {
            put(value, "body", fingerprint(fp, report.normal_form)?);
        }
        if let Some(fp) = &u.shim {
            put(value, "shim", fingerprint(fp, report.normal_form)?);
        }
    }
    // Group and job references must name already listed index units.
    let listed: BTreeSet<_> = graph.keys().cloned().collect();
    for g in &report.groups {
        if !matches!(
            (&g.fingerprint, g.kind),
            (Fingerprint::Body(_), GroupKind::Body) | (Fingerprint::Shim(_), GroupKind::Shim)
        ) {
            return Err(E::Inconsistent("group kind and fingerprint".into()));
        }
        for m in &g.members {
            let id = mint::unit(&m.symbol, &m.file, m.line, m.start_byte);
            if !listed.contains(&id) {
                return Err(E::Dangling(id));
            }
        }
        group(
            &mut graph,
            &g.fingerprint,
            g.tokens,
            &g.members,
            report.normal_form,
        )?;
    }
    for j in &report.jobs {
        let id = mint::job(&j.id);
        if graph.contains_key(&id) {
            return Err(E::DuplicateJob(j.id.clone()));
        }
        let mut value = node("Job");
        text(&mut value, "jobId", &j.id);
        text(&mut value, "homeSymbol", &j.home);
        num(&mut value, "copies", j.copies);
        for m in &j.matches {
            let unit = mint::unit(&m.symbol, &m.file, m.line, m.start_byte);
            if !listed.contains(&unit) {
                return Err(E::Dangling(unit));
            }
            put(&mut value, "match", Term::Iri(unit.clone()));
            let mut reifier = node("Match");
            put(
                &mut reifier,
                "reifies",
                Term::Triple(id.clone(), format!("{}match", vocab::CENSUS), unit.clone()),
            );
            put(
                &mut reifier,
                "matchClass",
                shape::iri(match m.class {
                    MatchClass::Home => "Home",
                    MatchClass::Variant => "Variant",
                    MatchClass::Copy => "Copy",
                }),
            );
            let name = mint::reifier(&j.id, &unit);
            if graph.insert(name.clone(), reifier).is_some() {
                return Err(E::Repeated(name));
            }
        }
        graph.insert(id, value);
    }
    Ok(graph)
}
pub(super) fn baseline(report: &Baseline) -> Result<String, Error> {
    let graph = baseline_graph(report)?;
    shape::baseline(&graph)?;
    render(&graph, report.normal_form, "baseline").map_err(Into::into)
}
pub(super) fn index(report: &Index) -> Result<String, Error> {
    let graph = index_graph(report)?;
    shape::index(&graph)?;
    render(&graph, report.normal_form, "index").map_err(Into::into)
}
pub(super) fn prefixes(nf: u32, kind: &str) -> Vec<(&'static str, String)> {
    let mut values = vec![
        ("body", vocab::fingerprint_namespace(nf, GroupKind::Body)),
        ("census", vocab::CENSUS.into()),
        ("shim", vocab::fingerprint_namespace(nf, GroupKind::Shim)),
    ];
    if kind == "baseline" {
        values.push(("table", vocab::TABLE.into()));
    }
    values
}
fn term(value: &Term, namespaces: &[(&str, String)], out: &mut String) {
    match value {
        Term::Iri(iri) => {
            for (prefix, namespace) in namespaces {
                if let Some(local) = iri.strip_prefix(namespace.as_str()) {
                    out.push_str(prefix);
                    out.push(':');
                    out.push_str(local);
                    return;
                }
            }
            term_syntax::write_iri(iri, out);
        }
        Term::String(value) => {
            term_syntax::write_literal(value, &format!("{}string", vocab::XSD), None, None, out);
        }
        Term::Integer(value) => out.push_str(&value.to_string()),
        Term::Boolean(value) => out.push_str(if *value { "true" } else { "false" }),
        Term::Triple(_, _, _) => unreachable!("triple terms are written as named annotations"),
    }
}
fn unit_key(node: &Node) -> (&str, &str, usize, usize) {
    (
        shape::string_ref(node, "symbol").expect("validated"),
        shape::string_ref(node, "file").expect("validated"),
        shape::count(node, "line").expect("validated"),
        shape::count(node, "startByte").expect("validated"),
    )
}
pub(super) fn digest(body: &str) -> String {
    blake3::hash(body.as_bytes()).to_hex().to_string()
}
fn ordered_nodes(graph: &Graph) -> Vec<(&String, &Node)> {
    let mut nodes: Vec<_> = graph
        .iter()
        .filter(|(_, node)| shape::class(node).ok() != Some("Match"))
        .collect();
    nodes.sort_by(|(a, na), (b, nb)| {
        let rank = |n: &Node| match shape::class(n).expect("validated") {
            "Baseline" | "Index" => 0,
            "BodyGroup" | "ShimGroup" => 1,
            "SharedConstant" | "Job" => 2,
            _ => 3,
        };
        rank(na)
            .cmp(&rank(nb))
            .then_with(|| {
                if rank(na) == 1 {
                    nb["member"].len().cmp(&na["member"].len()).then(
                        shape::count(nb, "tokens")
                            .expect("validated")
                            .cmp(&shape::count(na, "tokens").expect("validated")),
                    )
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .then_with(|| {
                if shape::class(na).ok() == Some("SharedConstant") {
                    shape::number(na, "value")
                        .expect("validated")
                        .cmp(&shape::number(nb, "value").expect("validated"))
                } else if rank(na) == 3 {
                    unit_key(na).cmp(&unit_key(nb))
                } else {
                    a.cmp(b)
                }
            })
    });
    nodes
}
pub(super) fn render(graph: &Graph, nf: u32, kind: &str) -> Result<String, E> {
    let mut body = String::from("VERSION \"1.2\"\n");
    let namespaces = prefixes(nf, kind);
    for (prefix, namespace) in &namespaces {
        writeln!(body, "PREFIX {prefix}: <{namespace}>").expect("String write");
    }
    body.push('\n');
    let nodes = ordered_nodes(graph);
    for (index, (id, node)) in nodes.iter().enumerate() {
        if index > 0 {
            body.push('\n');
        }
        term_syntax::write_iri(id, &mut body);
        body.push_str(" a ");
        term(&node["type"][0], &namespaces, &mut body);
        let properties: Vec<_> = node
            .iter()
            .filter(|(key, _)| key.as_str() != "type")
            .collect();
        if properties.is_empty() {
            body.push_str(" .\n");
            continue;
        }
        body.push_str(" ;\n");
        for (p, (key, values)) in properties.iter().enumerate() {
            write!(body, "    census:{key} ").expect("String write");
            let mut values = values.iter().collect::<Vec<_>>();
            if matches!(key.as_str(), "member" | "match") {
                values.sort_by_key(|v| {
                    let Term::Iri(id) = v else {
                        unreachable!("validated")
                    };
                    let n = &graph[id];
                    (
                        shape::string_ref(n, "symbol").expect("validated"),
                        shape::string_ref(n, "file").expect("validated"),
                        shape::count(n, "line").expect("validated"),
                        shape::count(n, "startByte").expect("validated"),
                    )
                });
            } else {
                values.sort();
            }
            for (v, value) in values.iter().enumerate() {
                if v > 0 {
                    body.push_str(" ,\n        ");
                }
                term(value, &namespaces, &mut body);
                if key.as_str() == "match" {
                    let Term::Iri(unit) = value else {
                        unreachable!("validated")
                    };
                    let job = shape::string(node, "jobId")?;
                    let reifier = mint::reifier(&job, unit);
                    body.push_str(" ~ ");
                    term_syntax::write_iri(&reifier, &mut body);
                    body.push_str(" {| a census:Match ; census:matchClass ");
                    term(&graph[&reifier]["matchClass"][0], &namespaces, &mut body);
                    body.push_str(" |}");
                }
            }
            body.push_str(if p + 1 == properties.len() {
                " .\n"
            } else {
                " ;\n"
            });
        }
    }
    let hash = digest(&body);
    body.insert_str(
        "VERSION \"1.2\"\n".len(),
        &format!("BASE <{}{hash}>\n", vocab::BASE),
    );
    Ok(body)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn the_digest_covers_everything_but_the_base_line() {
        assert_ne!(
            digest("VERSION \"1.2\"\nx\n"),
            digest("VERSION \"1.2\"\ny\n")
        );
    }
}
