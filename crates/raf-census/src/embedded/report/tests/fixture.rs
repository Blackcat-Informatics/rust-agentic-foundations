// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Editable RDF fixture data obtained through the actual admitted producer.

use super::super::{Claims, OWNER_NUMBERS, OWNER_TEXT, VOCAB};
use crate::embedded::{pairing, resource::Meter};
use crate::report::shape;
use raf_rdf::{Builder, Format, Graph, Node, TermValue as Term, Triple, Value};

pub(in crate::embedded) fn key(kind: &str, parts: &[String]) -> String {
    let mut account = purrdf_lex::allocation::Resident;
    super::super::key_with_memory(
        kind,
        parts.iter().map(String::as_str),
        &mut purrdf_lex::allocation::Memory::new(&mut account),
    )
    .unwrap()
}

pub(in crate::embedded) fn graph(matches: &pairing::Matches<'_>) -> Graph {
    Claims::produce(matches, |outcome| {
        outcome
            .unwrap()
            .to_text_admitted(Format::Turtle, matches.analysis.meter(), |text| {
                Graph::from_text(&text.unwrap(), Format::Turtle).unwrap()
            })
    })
}

pub(super) fn accepts(graph: &Graph) -> bool {
    roundtrip(&graph.to_text(Format::Turtle).unwrap(), Format::Turtle).is_some()
}

pub(super) fn exhausted_report(matches: &pairing::Matches<'_>, limit: usize) {
    use crate::embedded::{OutputFailureKind, RefusalKind};

    let meter = matches.analysis.meter();
    meter.work(limit - meter.usage().0).unwrap();
    Claims::produce(matches, |outcome| {
        let Err(cause) = outcome else {
            panic!("an exhausted report account must refuse")
        };
        assert_eq!(cause.kind(), OutputFailureKind::Native);
        assert_eq!(cause.refusal(), Some(RefusalKind::Work { limit }));
    });
    assert_eq!(meter.usage().0, limit);
}

pub(super) fn roundtrip(text: &str, format: Format) -> Option<String> {
    let meter = Meter::new(100_000_000, 10_000_000);
    Claims::from_text(text, format, &meter, |outcome| match outcome {
        Ok(claims) => {
            claims.to_text_admitted(format, &meter, |text| Some(text.unwrap().to_string()))
        }
        Err(cause) => {
            assert_eq!(
                cause.kind(),
                crate::embedded::OutputFailureKind::Domain,
                "{cause}"
            );
            assert!(
                cause.refusal().is_none(),
                "a semantic fixture must not exhaust its grant: {cause}"
            );
            None
        }
    })
}

pub(super) fn property(field: &str) -> String {
    if field == "type" {
        raf_rdf::vocab::TYPE.into()
    } else {
        format!("{VOCAB}{field}")
    }
}

pub(super) fn assertion(subject: &str, field: &str, object: Term) -> Triple {
    Triple {
        subject: Term::iri(subject),
        predicate: Term::iri(property(field)),
        object,
    }
}

pub(super) fn put(
    builder: &mut Builder,
    subject: &str,
    field: &str,
    object: Term,
) -> Result<(), raf_rdf::Error> {
    builder.insert(assertion(subject, field, object))
}

pub(super) fn number(value: usize) -> Term {
    shape::integer(value as u128)
}

pub(super) fn one<'a>(node: Node<'a>, field: &str) -> Option<Value<'a>> {
    let mut values = node.values(&property(field));
    let value = values.next()?;
    values.next().is_none().then_some(value)
}

pub(super) fn text<'a>(node: Node<'a>, field: &str) -> Option<&'a str> {
    one(node, field)?
        .literal()
        .filter(|(_, kind)| *kind == raf_rdf::vocab::STRING)
        .map(|(text, _)| text)
}

pub(super) fn count(node: Node<'_>, field: &str) -> Option<usize> {
    usize::try_from(shape::integer_value(one(node, field)?)?).ok()
}

pub(super) fn boolean(node: Node<'_>, field: &str) -> Option<bool> {
    shape::boolean_value(one(node, field)?)
}

pub(super) fn reference<'a>(node: Node<'a>, field: &str) -> Option<&'a str> {
    one(node, field)?.iri().filter(|id| id.starts_with('#'))
}

pub(super) fn class(node: Node<'_>) -> Option<&str> {
    one(node, "type")?.iri()?.strip_prefix(VOCAB)
}

pub(super) fn digest<'a>(node: Node<'a>, field: &str) -> Option<&'a str> {
    let value = text(node, field)?;
    purrdf_hash::hex::decode_32_canonical(value)
        .is_some()
        .then_some(value)
}

pub(super) fn owner_key(function: &str, node: Node<'_>) -> Option<String> {
    let mut parts = vec![function.to_owned()];
    for field in OWNER_TEXT {
        parts.push(text(node, field)?.to_owned());
    }
    for field in OWNER_NUMBERS {
        parts.push(count(node, field)?.to_string());
    }
    Some(key("owner", &parts))
}

pub(super) fn region_key(
    function: &str,
    start: usize,
    end: usize,
    first: usize,
    last: usize,
) -> String {
    key(
        "region",
        &[
            function.into(),
            start.to_string(),
            end.to_string(),
            first.to_string(),
            last.to_string(),
        ],
    )
}

pub(super) fn witness_key(left: &str, right: &str, digest: &str) -> String {
    key("witness", &[left.into(), right.into(), digest.into()])
}
