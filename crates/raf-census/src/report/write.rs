// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Domain assertions inserted directly into the shared native RDF builder.
use super::projection::{JobRef, Site, UnitRef, member_site, site_id};
use super::{Baseline, Index, MemberRow, ReportError as E, mint, shape, vocab};
use crate::{Error, Fingerprint, GroupKind, MatchClass};
use raf_rdf::{Builder, Graph, TermBox, TermValue as Term, Triple};

fn triple(id: &str, key: &str, object: Term) -> Triple {
    Triple {
        subject: Term::iri(id),
        predicate: Term::iri(shape::property(key)),
        object,
    }
}
pub(crate) fn put(graph: &mut Builder, id: &str, key: &str, value: Term) -> Result<(), E> {
    graph
        .insert(triple(id, key, value))
        .map_err(|error| match error {
            raf_rdf::Error::Repeated => E::Repeated(format!("{id}: census:{key}")),
            error => E::Ground(error),
        })
}
pub(crate) fn class(graph: &mut Builder, id: &str, name: &str) -> Result<(), E> {
    put(graph, id, "type", shape::iri(name))
}
pub(crate) fn num(graph: &mut Builder, id: &str, key: &str, value: usize) -> Result<(), E> {
    put(graph, id, key, shape::integer(value as u128))
}
pub(crate) fn text(graph: &mut Builder, id: &str, key: &str, value: &str) -> Result<(), E> {
    put(graph, id, key, Term::simple_literal(value))
}
pub(super) fn fingerprint(value: &Fingerprint, nf: u32) -> Result<Term, E> {
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
fn unit(graph: &mut Builder, site: Site<'_>) -> Result<String, E> {
    let id = site_id(site);
    let (symbol, file, line, start_byte) = site;
    let assertions = [
        ("type", shape::iri("Unit")),
        ("symbol", Term::simple_literal(symbol)),
        ("file", Term::simple_literal(file)),
        ("line", shape::integer(line as u128)),
        ("startByte", shape::integer(start_byte as u128)),
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
        let member = unit(graph, member_site(member))?;
        put(graph, &id, "member", Term::iri(member))?;
    }
    Ok(())
}
/// State a grouping scope on the root. The complete scope states nothing, so a
/// report or record computed without exclusions keeps its earlier bytes.
pub(super) fn scope(graph: &mut Builder, scope: &crate::GroupingScope) -> Result<(), E> {
    if scope.excludes_test_items() {
        put(graph, "", "excludeTestItems", Term::boolean(true))?;
    }
    for dir in scope.excluded_dirs() {
        text(graph, "", "excludeDir", dir)?;
    }
    Ok(())
}
pub(super) fn root(graph: &mut Builder, name: &str, nf: u32) -> Result<(), E> {
    class(graph, "", name)?;
    num(graph, "", "normalForm", nf as usize)?;
    num(graph, "", "reportForm", vocab::REPORT_FORM as usize)
}
pub(super) fn baseline_graph(report: &Baseline) -> Result<Graph, E> {
    let mut graph = Builder::new();
    root(&mut graph, "Baseline", report.normal_form)?;
    num(&mut graph, "", "unitCount", report.units)?;
    num(&mut graph, "", "minTokens", report.min_tokens)?;
    scope(&mut graph, &report.scope)?;
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
            let member = unit(&mut graph, member_site(m))?;
            put(&mut graph, &id, "member", Term::iri(member))?;
        }
    }
    for t in &report.hex_tables {
        let id = unit(&mut graph, (&t.symbol, &t.file, t.line, t.start_byte))?;
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
    scope(&mut graph, &report.scope)?;
    for u in &report.units {
        index_unit(&mut graph, UnitRef::Indexed(u), report.normal_form)?;
    }
    for source in &report.reference_sources {
        super::reference::source(&mut graph, source)?;
        put(
            &mut graph,
            "",
            "referenceSource",
            Term::iri(source.identity()?),
        )?;
    }
    for witness in &report.reference_witnesses {
        super::reference::witness(&mut graph, witness, report.normal_form)?;
    }
    for row in &report.reference_constants {
        for member in row.first_party.iter().chain(&row.reference) {
            let id = mint::unit(&member.symbol, &member.file, member.line, member.start_byte);
            if !has_unit(&graph, &id)? {
                return Err(E::Dangling(id));
            }
        }
        super::reference::constant(&mut graph, row)?;
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
            if !has_unit(&graph, &id)? {
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
        job(&mut graph, &JobRef::from(j))?;
    }
    graph.finish().map_err(E::Ground)
}

pub(super) fn has_unit(graph: &Builder, id: &str) -> Result<bool, E> {
    graph
        .contains(triple(id, "type", shape::iri("Unit")))
        .map_err(E::Ground)
}

pub(super) fn index_unit(graph: &mut Builder, u: UnitRef<'_>, nf: u32) -> Result<String, E> {
    let site = u.site();
    let id = site_id(site);
    if has_unit(graph, &id)? {
        return Err(E::Repeated(id));
    }
    unit(graph, site)?;
    let (package, shipping, tokens, body, shim) = u.fields();
    text(graph, &id, "package", package)?;
    if let Some(source) = u.source() {
        put(graph, &id, "referenceSource", Term::iri(source))?;
    }
    num(graph, &id, "tokens", tokens)?;
    put(graph, &id, "shipping", Term::boolean(shipping))?;
    if let Some(fp) = body {
        put(graph, &id, "body", fingerprint(fp, nf)?)?;
    }
    if let Some(fp) = shim {
        put(graph, &id, "shim", fingerprint(fp, nf)?)?;
    }
    Ok(id)
}

pub(super) fn match_class(class: MatchClass) -> Term {
    shape::iri(match class {
        MatchClass::Home => "Home",
        MatchClass::Variant => "Variant",
        MatchClass::Copy => "Copy",
    })
}

pub(super) fn job(graph: &mut Builder, j: &JobRef<'_>) -> Result<String, E> {
    let id = mint::job(j.id);
    if graph
        .contains(triple(&id, "type", shape::iri("Job")))
        .map_err(E::Ground)?
    {
        return Err(E::DuplicateJob(j.id.to_owned()));
    }
    class(graph, &id, "Job")?;
    text(graph, &id, "jobId", j.id)?;
    text(graph, &id, "homeSymbol", j.home)?;
    num(graph, &id, "copies", j.copies)?;
    for (site, role) in j.matches() {
        let unit = site_id(site);
        if !has_unit(graph, &unit)? {
            return Err(E::Dangling(unit));
        }
        put(graph, &id, "match", Term::iri(&unit))?;
        let name = mint::reifier(j.id, &unit);
        class(graph, &name, "Match")?;
        put(
            graph,
            &name,
            "reifies",
            Term::Triple {
                s: TermBox::new(Term::iri(&id)),
                p: TermBox::new(Term::iri(shape::property("match"))),
                o: TermBox::new(Term::iri(&unit)),
            },
        )?;
        put(graph, &name, "matchClass", match_class(role))?;
    }
    Ok(id)
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
