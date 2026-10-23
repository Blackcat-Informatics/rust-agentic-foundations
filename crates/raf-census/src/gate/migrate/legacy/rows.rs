// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Foreign application rows projected through the native RDF builder.

use std::collections::{BTreeMap, BTreeSet};

use raf_rdf::{Builder, TermValue as Term, Triple, vocab};
use raf_source::Workspace;
use toml::Value;

use super::super::{UnmappedRow, remap_members};
use super::{
    anchor,
    baseline::Historical,
    class,
    fields::{self, Fields},
    integer, put,
};
use crate::gate::owners::Owners;
use crate::{Error, Fingerprint, ReportError, report::mint};

pub(super) fn job(
    graph: &mut Builder,
    mut row: Fields,
    historical: &Historical,
    workspace: &Workspace,
    owners: &Owners<'_>,
    ids: &mut BTreeSet<String>,
    unmapped: &mut Vec<UnmappedRow>,
) -> Result<(), Error> {
    let id = row.text("id", None)?;
    if !ids.insert(id.clone()) {
        return Err(row.bad("id", "duplicate job id"));
    }
    let node = mint::job(&id);
    put(graph, "", "job", Term::iri(&node))?;
    class(graph, &node, "LedgerJob")?;
    header(graph, &node, &id, &mut row, unmapped)?;
    let mut forbidden = row.table("forbidden")?;
    for (key, flat, predicate) in [
        ("names", "forbidden_names", "forbiddenName"),
        ("patterns", "forbidden_patterns", "forbiddenPattern"),
    ] {
        let values = aliases(&mut row, flat, &mut forbidden, key)?;
        for text in fields::strings(values, &format!("job {id} forbidden {key}"))? {
            if key == "patterns" {
                unmapped.push(UnmappedRow {
                    location: format!("job {id} forbidden pattern"),
                    original: text.clone(),
                    reason: "foreign pattern declarations require conversion to the native pattern schema before adoption".into(),
                });
            }
            if key == "names"
                && let Err(error) = purrdf_jsonschema::ecma::compile(&text)
            {
                unmapped.push(UnmappedRow {
                    location: format!("job {id} forbidden name"),
                    original: text.clone(),
                    reason: format!("native name expression refused: {error}"),
                });
            }
            put(graph, &node, predicate, Term::simple_literal(text))?;
        }
    }
    let values = aliases(&mut row, "forbidden_constants", &mut forbidden, "constants")?;
    constants(graph, &node, values, &id)?;
    let values = aliases(
        &mut row,
        "forbidden_fingerprints",
        &mut forbidden,
        "fingerprints",
    )?;
    for original in fields::strings(values, &format!("job {id} fingerprints"))? {
        let mapped = if original.starts_with("body:") || original.starts_with("shim:") {
            historical
                .get(&original)
                .ok_or_else(|| "baseline has no member symbol for the fingerprint".to_owned())
                .and_then(|members| {
                    remap_members(
                        original.starts_with("shim:"),
                        members.iter().map(String::as_str),
                        workspace,
                        owners,
                    )
                })
        } else {
            original
                .parse::<Fingerprint>()
                .map(|fingerprint| BTreeSet::from([fingerprint]))
                .map_err(|error| error.to_string())
        };
        match mapped {
            Ok(values) => {
                for value in values {
                    put(
                        graph,
                        &node,
                        "forbiddenFingerprint",
                        Term::simple_literal(value.to_string()),
                    )?;
                }
            }
            Err(reason) => {
                put(
                    graph,
                    &node,
                    "forbiddenFingerprint",
                    Term::simple_literal(&original),
                )?;
                unmapped.push(UnmappedRow {
                    location: format!("job {id} fingerprint {original}"),
                    original,
                    reason,
                });
            }
        }
    }
    for (index, variant) in row.rows("variant")?.into_iter().enumerate() {
        emit_variant(graph, &node, index, variant, unmapped)?;
    }
    extension(graph, &node, &mut row, unmapped)?;
    forbidden.finish(unmapped);
    row.finish(unmapped);
    Ok(())
}

fn header(
    graph: &mut Builder,
    node: &str,
    id: &str,
    row: &mut Fields,
    unmapped: &mut Vec<UnmappedRow>,
) -> Result<(), Error> {
    let summary = row.text("summary", Some(""))?;
    if summary.trim().is_empty() || summary.contains(['\r', '\n']) {
        unmapped.push(UnmappedRow {
            location: format!("job {id} summary"),
            original: summary.clone(),
            reason: "a nonempty one-line summary must be authored before policy adoption".into(),
        });
    }
    for (key, value) in [
        ("jobId", id.to_owned()),
        ("summary", summary),
        ("homeSymbol", row.text("home", None)?),
    ] {
        put(graph, node, key, Term::simple_literal(value))?;
    }
    put(
        graph,
        node,
        "enforced",
        Term::boolean(row.flag("enforced", false)?),
    )?;
    for entry in row.strings("entry_points")? {
        put(graph, node, "entryPoint", Term::simple_literal(entry))?;
    }
    Ok(())
}

fn aliases(
    row: &mut Fields,
    flat: &str,
    nested: &mut Fields,
    key: &str,
) -> Result<Option<Value>, Error> {
    match (row.take(flat), nested.take(key)) {
        (Some(_), Some(_)) => Err(row.bad(
            flat,
            "both flat and nested field spellings are present; choose one",
        )),
        (Some(value), None) | (None, Some(value)) => Ok(Some(value)),
        (None, None) => Ok(None),
    }
}

fn constants(
    graph: &mut Builder,
    node: &str,
    value: Option<Value>,
    job: &str,
) -> Result<(), Error> {
    let values = match value {
        None => return Ok(()),
        Some(Value::Array(values)) => values,
        _ => {
            return Err(Error::Options(format!(
                "job {job}: constants must be an array"
            )));
        }
    };
    for value in values {
        let parsed = match value {
            Value::Integer(value) => u128::try_from(value).ok(),
            Value::String(value) => {
                crate::normalize::parse_integer(&value).and_then(|value| value.value)
            }
            _ => None,
        }
        .ok_or_else(|| {
            Error::Options(format!(
                "job {job}: constant is not a nonnegative integer literal"
            ))
        })?;
        put(graph, node, "forbiddenConstant", integer(parsed))?;
    }
    Ok(())
}

fn emit_variant(
    graph: &mut Builder,
    job: &str,
    index: usize,
    mut row: Fields,
    unmapped: &mut Vec<UnmappedRow>,
) -> Result<(), Error> {
    let node = format!("{job}/variant/{index}");
    put(graph, job, "variant", Term::iri(&node))?;
    class(graph, &node, "Variant")?;
    for key in ["symbol", "file", "detector", "criterion", "reason"] {
        put(
            graph,
            &node,
            key,
            Term::simple_literal(row.text(key, None)?),
        )?;
    }
    put(graph, &node, "anchor", anchor(&row.text("anchor", None)?))?;
    row.finish(unmapped);
    Ok(())
}

pub(super) fn distinct(
    graph: &mut Builder,
    index: usize,
    mut row: Fields,
    unmapped: &mut Vec<UnmappedRow>,
) -> Result<(), Error> {
    let node = format!("#distinct/{index}");
    let kind = row.text("kind", None)?;
    if !matches!(kind.as_str(), "body" | "shim") {
        return Err(row.bad("kind", "expected body or shim"));
    }
    let members = row.strings("members")?;
    if members.len() < 2 || members.iter().any(|member| member.trim().is_empty()) {
        return Err(row.bad(
            "members",
            "at least two nonempty physical member declarations are required",
        ));
    }
    put(graph, "", "distinct", Term::iri(&node))?;
    class(graph, &node, "Distinct")?;
    put(graph, &node, "kind", Term::simple_literal(kind))?;
    let mut counts = BTreeMap::new();
    for member in members {
        *counts.entry(member).or_insert(0usize) += 1;
    }
    for (symbol, count) in counts {
        put(graph, &node, "memberSymbol", Term::simple_literal(&symbol))?;
        if count > 1 {
            let counted = format!("{node}/multiplicity/{}", mint::part(&symbol));
            put(graph, &node, "multiplicity", Term::iri(&counted))?;
            class(graph, &counted, "MemberMultiplicity")?;
            put(graph, &counted, "symbol", Term::simple_literal(symbol))?;
            put(graph, &counted, "copies", integer(count as u128))?;
        }
    }
    for key in ["criterion", "reason"] {
        put(
            graph,
            &node,
            key,
            Term::simple_literal(row.text(key, None)?),
        )?;
    }
    put(graph, &node, "anchor", anchor(&row.text("anchor", None)?))?;
    row.finish(unmapped);
    Ok(())
}

fn extension(
    graph: &mut Builder,
    job: &str,
    row: &mut Fields,
    unmapped: &mut Vec<UnmappedRow>,
) -> Result<(), Error> {
    let mut extension = row.table("ext")?;
    for key in [
        "spec",
        "vectors",
        "bench",
        "sites",
        "replaces_external",
        "disposition",
        "owner",
        "external",
    ] {
        if let Some(value) = row.take(key)
            && extension.table.insert(key.into(), value).is_some()
        {
            return Err(row.bad(
                key,
                "both dialect and extension values are present; choose one",
            ));
        }
    }
    if extension.table.is_empty() {
        return Ok(());
    }
    let node = format!("{job}/extension");
    put(graph, job, "extension", Term::iri(&node))?;
    class(graph, &node, "Extension")?;
    for (key, value) in std::mem::take(&mut extension.table) {
        let original = value.to_string();
        let term = match value {
            Value::String(value) => Ok(Term::simple_literal(value)),
            Value::Boolean(value) => Ok(Term::boolean(value)),
            Value::Integer(value) => Ok(Term::typed_literal(
                purrdf_xsd::exact::Integer::from(value).canonical_lexical(),
                vocab::INTEGER,
            )),
            value => json(value).map(|value| {
                Term::typed_literal(
                    purrdf_lex::json::write_compact(&value),
                    format!("{}JSON", vocab::RDF),
                )
            }),
        };
        match term {
            Ok(object) => {
                graph
                    .insert(Triple {
                        subject: Term::iri(&node),
                        predicate: Term::iri(format!(
                            "urn:raf:census:extension:{}",
                            mint::part(&key)
                        )),
                        object,
                    })
                    .map_err(ReportError::Ground)?;
            }
            Err(reason) => unmapped.push(UnmappedRow {
                location: format!("{}.{}", extension.at, key),
                original,
                reason,
            }),
        }
    }
    Ok(())
}

fn json(value: Value) -> Result<purrdf_lex::json::Value, String> {
    use purrdf_lex::json::{Number, Value as Json};
    match value {
        Value::String(value) => Ok(Json::from(value)),
        Value::Boolean(value) => Ok(Json::from(value)),
        Value::Integer(value) => Ok(Json::from(value)),
        Value::Float(value) => Number::from_f64(value)
            .map(Json::Number)
            .ok_or_else(|| "nonfinite TOML float has no declared extension projection".into()),
        Value::Array(values) => values
            .into_iter()
            .map(json)
            .collect::<Result<Vec<_>, _>>()
            .map(Json::from),
        Value::Table(values) => values
            .into_iter()
            .map(|(key, value)| Ok((key, json(value)?)))
            .collect::<Result<Vec<_>, String>>()
            .map(Json::object),
        Value::Datetime(_) => Err(
            "TOML date/time has no declared extension projection; retain and map it explicitly"
                .into(),
        ),
    }
}
