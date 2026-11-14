// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed normative allowances under the selected run's original RDF account.

use super::{Entries, Failure, Record};
use crate::ReportError as E;
use crate::embedded::{
    identity, names,
    output::{BodyFailure, OutputFailure},
    policy, report,
    resource::{Meter, Text},
    schema::{BorrowedSet, Emitter, Graph, Named, Node, Schema},
};
use raf_rdf::{AdmittedGraph, ConstructionExit, ConstructionFailure, Format};

const VOCAB: &str = "https://blackcatinformatics.ca/raf/census/embedded-debt/1#";
fn schema(meter: &Meter) -> Schema<'_> {
    Schema {
        meter,
        vocab: VOCAB,
        label: "embedded-debt",
    }
}

fn endpoint_id<'m>(value: &str, meter: &'m Meter) -> Result<Text<'m>, Failure<'m>> {
    let mut hash = purrdf_hash::blake3::Hasher::new();
    for extent in value.as_bytes().chunks(256) {
        meter.work(extent.len())?;
        hash.update(extent);
    }
    meter.work(1)?;
    Ok(Text::display(
        meter,
        &format_args!(
            "#endpoint/{}",
            purrdf_hash::hex::Lower(hash.finalize().as_bytes())
        ),
    )?)
}
fn pair_id<'m>(
    digest: &[u8; 32],
    left: &str,
    right: &str,
    shape: Schema<'m>,
) -> Result<Text<'m>, Failure<'m>> {
    let digest = Text::display(shape.meter, &purrdf_hash::hex::Lower(digest))?;
    shape.key("pair-v1", [digest.as_ref(), left, right])
}
fn endpoint<'m, 'a>(
    emit: &mut Emitter<'_, '_, '_, 'm>,
    seen: &mut Named<'m, &'a str>,
    identity: &'a str,
) -> Result<Text<'m>, BodyFailure<'m>> {
    let id = endpoint_id(identity, emit.schema.meter)?;
    if let Some(prior) = seen.get(&id)? {
        if !emit.schema.meter.compare(prior, identity)?.is_eq() {
            return Err(emit
                .schema
                .error(
                    E::Inconsistent,
                    &"embedded endpoint naming digest collision",
                )
                .into());
        }
    } else {
        identity::validate_endpoint(identity, emit.schema.meter)?;
        seen.remember(&id, identity)?;
        emit.class(&id, "Endpoint")?;
        emit.text(&id, "identity", identity)?;
    }
    Ok(id)
}
fn graph<'m, R>(
    record: &Record<'m>,
    observe: impl for<'f> FnOnce(Result<Graph<'m>, OutputFailure<'f>>) -> R,
) -> R {
    let meter = record.entries.meter();
    let shape = schema(meter);
    AdmittedGraph::build_observed(
        meter.native(),
        meter.capture(),
        |build| {
            let result = (|| {
                let mut emit = Emitter {
                    build,
                    schema: shape,
                };
                let mut seen = Named::new(meter);
                emit.class("", "EmbeddedDebt")?;
                emit.number("", "debtForm", 1)?;
                for (field, value) in [
                    ("matchingProfile", policy::PROFILE),
                    ("identifierProfile", names::PROFILE),
                    ("preludeProfile", report::PRELUDE_PROFILE),
                ] {
                    emit.text("", field, value)?;
                }
                for (pair, amount) in record.entries.iter() {
                    meter.work(1)?;
                    let left = endpoint(&mut emit, &mut seen, &pair.left)?;
                    let right = endpoint(&mut emit, &mut seen, &pair.right)?;
                    let id = pair_id(&pair.digest, &left, &right, shape)?;
                    emit.class(&id, "PairDebt")?;
                    emit.display(
                        &id,
                        "digest",
                        &purrdf_hash::hex::Lower(&pair.digest),
                        raf_rdf::vocab::STRING,
                    )?;
                    emit.iri(&id, "left", &left)?;
                    emit.iri(&id, "right", &right)?;
                    emit.number(&id, "pairs", amount.get())?;
                    emit.iri("", "pair", &id)?;
                }
                Ok::<(), BodyFailure<'m>>(())
            })();
            result.map_err(|cause| match cause {
                BodyFailure::Native(stopped) => ConstructionExit::Stopped(stopped),
                BodyFailure::Caller(cause) => ConstructionExit::Caller(cause),
            })
        },
        |outcome| match outcome {
            Ok(graph) => observe(Ok(graph)),
            Err(ConstructionFailure::Native(cause)) => observe(Err(OutputFailure::build(&cause))),
            Err(ConstructionFailure::Caller(cause)) => {
                observe(Err(OutputFailure::failure(cause.cause())))
            }
        },
    )
}
fn size<'m>(text: &str, meter: &'m Meter) -> Result<(), Failure<'m>> {
    if text.len() > crate::MAX_DEBT_BYTES {
        return Err(Failure::report(
            meter,
            E::Value,
            &"embedded debt exceeds its serialized byte limit",
        ));
    }
    Ok(())
}
pub(super) fn write<'m, R>(
    record: &Record<'m>,
    format: Format,
    observe: impl for<'f> FnOnce(Result<Text<'m>, OutputFailure<'f>>) -> R,
) -> R {
    let meter = record.entries.meter();
    graph(record, |outcome| match outcome {
        Err(cause) => observe(Err(cause)),
        Ok(graph) => match parse(&graph, meter) {
            Err(cause) => observe(Err(OutputFailure::failure(&cause))),
            Ok(validated) => {
                drop(validated);
                Text::render_admitted(meter, &graph, format, |outcome| match outcome {
                    Err(cause) => observe(Err(cause)),
                    Ok(text) => match size(&text, meter) {
                        Ok(()) => observe(Ok(text)),
                        Err(cause) => observe(Err(OutputFailure::failure(&cause))),
                    },
                })
            }
        },
    })
}
pub(super) fn read<'m, R>(
    text: &str,
    format: Format,
    meter: &'m Meter,
    observe: impl for<'f> FnOnce(Result<Record<'m>, OutputFailure<'f>>) -> R,
) -> R {
    if let Err(cause) = size(text, meter) {
        return observe(Err(OutputFailure::failure(&cause)));
    }
    AdmittedGraph::import_captured(
        text,
        format,
        || meter.native(),
        meter.progress(),
        meter.capture(),
        |outcome| match outcome {
            Err(cause) => observe(Err(OutputFailure::import(&cause))),
            Ok(graph) => match parse(&graph, meter) {
                Ok(record) => observe(Ok(record)),
                Err(cause) => observe(Err(OutputFailure::failure(&cause))),
            },
        },
    )
}
fn shape_node<'m>(
    node: Node<'_>,
    required: &[&str],
    repeated: Option<&str>,
    shape: Schema<'m>,
) -> Result<(), Failure<'m>> {
    for field in std::iter::once("type").chain(required.iter().copied()) {
        shape.one(node, field)?;
    }
    for &(_, predicate) in shape.properties(node)?.iter() {
        let mut known = false;
        for field in std::iter::once("type")
            .chain(required.iter().copied())
            .chain(repeated)
        {
            let property = shape.property(field)?;
            if shape.meter.compare(&property, predicate)?.is_eq() {
                known = true;
                break;
            }
        }
        if !known {
            return Err(shape.error(E::UnknownPredicate, &predicate));
        }
    }
    Ok(())
}
fn endpoint_value<'a, 'm>(
    graph: &'a Graph<'m>,
    id: &str,
    shape: Schema<'m>,
) -> Result<&'a str, Failure<'m>> {
    let node = shape.get(graph, id)?;
    if !shape
        .meter
        .compare(shape.class_named(node, "embedded debt")?, "Endpoint")?
        .is_eq()
    {
        return Err(shape.error(E::Class, &"debt endpoint required"));
    }
    shape_node(node, &["identity"], None, shape)?;
    let identity = shape.text(node, "identity")?;
    identity::validate_endpoint(identity, shape.meter)?;
    let expected = endpoint_id(identity, shape.meter)?;
    if !shape.meter.compare(&expected, id)?.is_eq() {
        return Err(shape.error(E::Name, &id));
    }
    Ok(identity)
}
fn parse<'m>(graph: &Graph<'m>, meter: &'m Meter) -> Result<Record<'m>, Failure<'m>> {
    let shape = schema(meter);
    let root = Failure::query(meter, |memory| Ok(graph.get_with_memory("", memory)?))?
        .ok_or_else(|| shape.error(E::Missing, &"embedded debt root"))?;
    if !meter
        .compare(shape.class_named(root, "embedded debt")?, "EmbeddedDebt")?
        .is_eq()
    {
        return Err(shape.error(E::Class, &"embedded debt root required"));
    }
    shape_node(
        root,
        &[
            "debtForm",
            "matchingProfile",
            "identifierProfile",
            "preludeProfile",
        ],
        Some("pair"),
        shape,
    )?;
    if shape.count(root, "debtForm")? != 1
        || !meter
            .compare(shape.text(root, "matchingProfile")?, policy::PROFILE)?
            .is_eq()
        || !meter
            .compare(shape.text(root, "identifierProfile")?, names::PROFILE)?
            .is_eq()
        || !meter
            .compare(shape.text(root, "preludeProfile")?, report::PRELUDE_PROFILE)?
            .is_eq()
    {
        return Err(shape.error(
            E::Value,
            &"unsupported embedded debt form or policy profile",
        ));
    }
    let mut seen = BorrowedSet::new(meter);
    seen.insert("")?;
    let mut entries = Entries::new(meter);
    let mut total = 0usize;
    for &value in shape.values(root, "pair")?.iter() {
        meter.work(1)?;
        let id = shape
            .iri(value)?
            .ok_or_else(|| shape.error(E::Value, &"debt pair reference required"))?;
        let node = shape.get(graph, id)?;
        if !meter
            .compare(shape.class_named(node, "embedded debt")?, "PairDebt")?
            .is_eq()
        {
            return Err(shape.error(E::Class, &"debt pair required"));
        }
        shape_node(node, &["digest", "left", "right", "pairs"], None, shape)?;
        let left_id = shape.reference_as(node, "left", "owned reference")?;
        let right_id = shape.reference_as(node, "right", "owned reference")?;
        let left = endpoint_value(graph, left_id, shape)?;
        let right = endpoint_value(graph, right_id, shape)?;
        let digest = shape.text(node, "digest")?;
        meter.work(digest.len())?;
        let digest = purrdf_hash::hex::decode_32_canonical(digest).ok_or_else(|| {
            shape.error(E::Value, &"embedded debt requires full lowercase digest")
        })?;
        let count = shape.count(node, "pairs")?;
        if meter.compare(left, right)?.is_gt() || count == 0 {
            return Err(shape.error(
                E::Value,
                &"embedded debt needs canonical endpoint order and positive counts",
            ));
        }
        let expected = pair_id(&digest, left_id, right_id, shape)?;
        if !meter.compare(&expected, id)?.is_eq() {
            return Err(shape.error(E::Name, &id));
        }
        total = total.checked_add(count).ok_or(Failure::Overflow)?;
        if !entries.insert(digest, left, right, count)? {
            return Err(shape.error(E::Repeated, &"embedded debt key"));
        }
        for id in [id, left_id, right_id] {
            seen.insert(id)?;
        }
    }
    for &(id, _) in shape.nodes(graph)?.iter() {
        if !seen.contains(id)? {
            return Err(shape.error(E::Orphan, &id));
        }
    }
    Ok(Record { entries })
}
