// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Domain assertions inserted directly into the shared native RDF builder.
use super::{Baseline, Index, MemberRow, ReportError as E, mint, shape, vocab};
use crate::{Error, Fingerprint, GroupKind, MatchClass};
use raf_rdf::{Builder, Graph, TermBox, TermValue as Term, Triple};
use std::collections::BTreeSet;

fn triple(id: &str, key: &str, object: Term) -> Triple {
    Triple {
        subject: Term::iri(id),
        predicate: Term::iri(shape::property(key)),
        object,
    }
}
pub(super) fn put(graph: &mut Builder, id: &str, key: &str, value: Term) -> Result<(), E> {
    graph
        .insert(triple(id, key, value))
        .map_err(|error| match error {
            raf_rdf::Error::Repeated => E::Repeated(format!("{id}: census:{key}")),
            error => E::Ground(error),
        })
}
fn class(graph: &mut Builder, id: &str, name: &str) -> Result<(), E> {
    put(graph, id, "type", shape::iri(name))
}
fn num(graph: &mut Builder, id: &str, key: &str, value: usize) -> Result<(), E> {
    put(graph, id, key, shape::integer(value as u128))
}
fn text(graph: &mut Builder, id: &str, key: &str, value: &str) -> Result<(), E> {
    put(graph, id, key, Term::simple_literal(value))
}
fn fingerprint(value: &Fingerprint, nf: u32) -> Result<Term, E> {
    let (kind, hex) = match value {
        Fingerprint::Body(v) => (GroupKind::Body, v),
        Fingerprint::Shim(v) => (GroupKind::Shim, v),
        _ => return Err(E::Value(format!("group fingerprint {value}"))),
    };
    Ok(Term::iri(format!(
        "{}{}",
        vocab::fingerprint_namespace(nf, kind),
        purrdf_hash::hex::Lower(hex)
    )))
}
fn unit(graph: &mut Builder, row: &MemberRow) -> Result<String, E> {
    let id = mint::unit(&row.symbol, &row.file, row.line, row.start_byte);
    let assertions = [
        ("type", shape::iri("Unit")),
        ("symbol", Term::simple_literal(&row.symbol)),
        ("file", Term::simple_literal(&row.file)),
        ("line", shape::integer(row.line as u128)),
        ("startByte", shape::integer(row.start_byte as u128)),
    ];
    if graph
        .contains(triple(&id, "type", shape::iri("Unit")))
        .map_err(E::Ground)?
    {
        for (key, value) in assertions {
            if !graph.contains(triple(&id, key, value)).map_err(E::Ground)? {
                return Err(E::Inconsistent(format!("{id}: {key}")));
            }
        }
    } else {
        for (key, value) in assertions {
            put(graph, &id, key, value)?;
        }
    }
    Ok(id)
}
fn group(
    graph: &mut Builder,
    fp: &Fingerprint,
    tokens: usize,
    members: &[MemberRow],
    nf: u32,
) -> Result<(), E> {
    let name = match fp {
        Fingerprint::Body(_) => "BodyGroup",
        Fingerprint::Shim(_) => "ShimGroup",
        _ => return Err(E::Value(fp.to_string())),
    };
    let id = format!("#group/{fp}");
    class(graph, &id, name)?;
    put(graph, &id, "fingerprint", fingerprint(fp, nf)?)?;
    num(graph, &id, "tokens", tokens)?;
    for member in members {
        let member = unit(graph, member)?;
        put(graph, &id, "member", Term::iri(member))?;
    }
    Ok(())
}
fn root(graph: &mut Builder, name: &str, nf: u32) -> Result<(), E> {
    class(graph, "", name)?;
    num(graph, "", "normalForm", nf as usize)?;
    num(graph, "", "reportForm", vocab::REPORT_FORM as usize)
}
pub(super) fn baseline_graph(report: &Baseline) -> Result<Graph, E> {
    let mut graph = Builder::new();
    root(&mut graph, "Baseline", report.normal_form)?;
    num(&mut graph, "", "unitCount", report.units)?;
    num(&mut graph, "", "minTokens", report.min_tokens)?;
    for (groups, body) in [(&report.body_groups, true), (&report.shim_groups, false)] {
        for g in groups {
            if !matches!(
                (&g.fingerprint, body),
                (Fingerprint::Body(_), true) | (Fingerprint::Shim(_), false)
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
        let id = format!("#constant/{}", c.value);
        class(&mut graph, &id, "SharedConstant")?;
        put(&mut graph, &id, "value", shape::integer(c.value))?;
        num(&mut graph, &id, "packageCount", c.packages)?;
        for m in &c.members {
            let member = unit(&mut graph, m)?;
            put(&mut graph, &id, "member", Term::iri(member))?;
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
            &mut graph,
            &id,
            "holds",
            Term::iri(format!("{}{}", vocab::TABLE, t.table.name())),
        )?;
    }
    graph.finish().map_err(E::Ground)
}
pub(super) fn index_graph(report: &Index) -> Result<Graph, E> {
    let mut graph = Builder::new();
    root(&mut graph, "Index", report.normal_form)?;
    let mut listed = BTreeSet::new();
    for u in &report.units {
        let row = MemberRow {
            symbol: u.symbol.clone(),
            file: u.file.clone(),
            line: u.line,
            start_byte: u.start_byte,
        };
        let id = mint::unit(&row.symbol, &row.file, row.line, row.start_byte);
        if !listed.insert(id.clone()) {
            return Err(E::Repeated(id));
        }
        unit(&mut graph, &row)?;
        text(&mut graph, &id, "package", &u.package)?;
        num(&mut graph, &id, "tokens", u.tokens)?;
        put(&mut graph, &id, "shipping", Term::boolean(u.shipping))?;
        if let Some(fp) = &u.body {
            put(
                &mut graph,
                &id,
                "body",
                fingerprint(fp, report.normal_form)?,
            )?;
        }
        if let Some(fp) = &u.shim {
            put(
                &mut graph,
                &id,
                "shim",
                fingerprint(fp, report.normal_form)?,
            )?;
        }
    }
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
    let mut jobs = BTreeSet::new();
    for j in &report.jobs {
        if !jobs.insert(j.id.as_str()) {
            return Err(E::DuplicateJob(j.id.clone()));
        }
        let id = mint::job(&j.id);
        class(&mut graph, &id, "Job")?;
        text(&mut graph, &id, "jobId", &j.id)?;
        text(&mut graph, &id, "homeSymbol", &j.home)?;
        num(&mut graph, &id, "copies", j.copies)?;
        for m in &j.matches {
            let unit = mint::unit(&m.symbol, &m.file, m.line, m.start_byte);
            if !listed.contains(&unit) {
                return Err(E::Dangling(unit));
            }
            put(&mut graph, &id, "match", Term::iri(&unit))?;
            let name = mint::reifier(&j.id, &unit);
            class(&mut graph, &name, "Match")?;
            put(
                &mut graph,
                &name,
                "reifies",
                Term::Triple {
                    s: TermBox::new(Term::iri(&id)),
                    p: TermBox::new(Term::iri(shape::property("match"))),
                    o: TermBox::new(Term::iri(&unit)),
                },
            )?;
            put(
                &mut graph,
                &name,
                "matchClass",
                shape::iri(match m.class {
                    MatchClass::Home => "Home",
                    MatchClass::Variant => "Variant",
                    MatchClass::Copy => "Copy",
                }),
            )?;
        }
    }
    graph.finish().map_err(E::Ground)
}
pub(super) fn baseline(report: &Baseline, format: raf_rdf::Format) -> Result<String, Error> {
    let graph = baseline_graph(report)?;
    shape::baseline(&graph)?;
    graph.to_text(format).map_err(|e| E::Ground(e).into())
}
pub(super) fn index(report: &Index, format: raf_rdf::Format) -> Result<String, Error> {
    let graph = index_graph(report)?;
    shape::index(&graph)?;
    graph.to_text(format).map_err(|e| E::Ground(e).into())
}
