// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

use super::*;
use raf_rdf::Triple;

fn response() -> RegisteredCheck {
    RegisteredCheck::build(
        Outcome::Findings,
        ["equal", "equal", "last\nλ"].into_iter(),
        None,
    )
    .unwrap()
}

fn edited(response: &RegisteredCheck, change: impl FnOnce(&mut Builder)) -> String {
    let mut graph = Builder::from_graph(&response.graph);
    change(&mut graph);
    graph.finish().unwrap().to_text(Format::NTriples).unwrap()
}

fn replace(graph: &mut Builder, subject: &str, key: &str, old: Term, new: Option<Term>) {
    assert!(
        graph
            .remove(Triple {
                subject: Term::iri(subject),
                predicate: Term::iri(shape::property(key)),
                object: old
            })
            .unwrap()
    );
    if let Some(new) = new {
        put(graph, subject, key, new).unwrap();
    }
}

#[test]
fn native_responses_preserve_order_multiplicity_and_evaluation_errors() {
    let response = response();
    let text = response.to_ntriples().unwrap();
    let decoded = RegisteredCheck::from_ntriples(&text).unwrap();
    assert!(decoded.findings().eq(["equal", "equal", "last\nλ"]));
    assert_eq!(decoded.exit_code(), 1);
    assert_eq!(decoded.error_message(), None);
    assert_eq!(decoded.to_ntriples().unwrap(), text);
    let clean = RegisteredCheck::build(Outcome::Clean, std::iter::empty(), None).unwrap();
    let decoded = RegisteredCheck::from_ntriples(&clean.to_ntriples().unwrap()).unwrap();
    assert_eq!(decoded.exit_code(), 0);
    assert_eq!(decoded.findings().len(), 0);
    let error = RegisteredCheck::from_check(&Err(Error::Options("broken\ninput".into()))).unwrap();
    let decoded = RegisteredCheck::from_ntriples(&error.to_ntriples().unwrap()).unwrap();
    assert_eq!(decoded.exit_code(), 1);
    assert!(decoded.error_message().unwrap().contains("broken"));
    assert_eq!(decoded.findings().len(), 0);
}

#[test]
fn protocol_version_fields_and_outcomes_are_closed_with_valid_neighbors() {
    let response = response();
    for replacement in [
        None,
        Some(Term::iri("urn:other:protocol")),
        Some(Term::simple_literal(CHECK_PROTOCOL_VERSION)),
    ] {
        let text = edited(&response, |graph| {
            replace(
                graph,
                "",
                "protocolVersion",
                Term::iri(CHECK_PROTOCOL_VERSION),
                replacement,
            );
        });
        assert!(RegisteredCheck::from_ntriples(&text).is_err());
    }
    for (key, value) in [
        ("protocolVersion", Term::iri("urn:other:protocol")),
        ("unknown", Term::simple_literal("value")),
        ("errorMessage", Term::simple_literal("not an error outcome")),
    ] {
        let text = edited(&response, |graph| put(graph, "", key, value).unwrap());
        assert!(RegisteredCheck::from_ntriples(&text).is_err());
    }
    for status in ["clean", "error", "unknown"] {
        let text = edited(&response, |graph| {
            replace(
                graph,
                "",
                "status",
                Term::simple_literal("findings"),
                Some(Term::simple_literal(status)),
            );
        });
        assert!(RegisteredCheck::from_ntriples(&text).is_err());
    }
    let text = edited(&response, |graph| {
        replace(
            graph,
            "",
            "type",
            shape::iri("RegisteredCheck"),
            Some(shape::iri("Other")),
        );
    });
    assert!(RegisteredCheck::from_ntriples(&text).is_err());
    assert!(RegisteredCheck::from_ntriples(&response.to_ntriples().unwrap()).is_ok());
}

#[test]
fn membership_ordinals_and_orphans_cannot_drop_findings() {
    let response = response();
    for count in [0, 2, 4] {
        let text = edited(&response, |graph| {
            replace(
                graph,
                "",
                "findingCount",
                shape::integer(3),
                Some(shape::integer(count)),
            );
        });
        assert!(RegisteredCheck::from_ntriples(&text).is_err());
    }
    let text = edited(&response, |graph| {
        replace(graph, "", "finding", Term::iri("#finding/1"), None);
    });
    assert!(RegisteredCheck::from_ntriples(&text).is_err());
    for ordinal in [0, 3] {
        let text = edited(&response, |graph| {
            replace(
                graph,
                "#finding/1",
                "ordinal",
                shape::integer(1),
                Some(shape::integer(ordinal)),
            );
        });
        assert!(RegisteredCheck::from_ntriples(&text).is_err());
    }
    let text = edited(&response, |graph| {
        put(graph, "#orphan", "type", shape::iri("CensusFinding")).unwrap();
        put(graph, "#orphan", "ordinal", shape::integer(3)).unwrap();
        put(graph, "#orphan", "message", Term::simple_literal("orphan")).unwrap();
    });
    assert!(RegisteredCheck::from_ntriples(&text).is_err());
    let text = edited(&response, |graph| {
        replace(
            graph,
            "#finding/0",
            "message",
            Term::simple_literal("equal"),
            Some(shape::integer(1)),
        );
    });
    assert!(RegisteredCheck::from_ntriples(&text).is_err());
    assert_eq!(
        RegisteredCheck::from_ntriples(&response.to_ntriples().unwrap())
            .unwrap()
            .findings()
            .len(),
        3
    );
}

#[test]
fn canonical_and_size_refusals_do_not_accept_partial_carriers() {
    let text = response().to_ntriples().unwrap();
    assert!(RegisteredCheck::from_ntriples(text.trim_end_matches('\n')).is_err());
    assert!(RegisteredCheck::from_ntriples(&(text.clone() + "\n")).is_err());
    assert!(RegisteredCheck::from_ntriples(&text.replace('\n', "\r\n")).is_err());
    assert!(RegisteredCheck::from_ntriples(&"x".repeat(MAX_CHECK_RESPONSE_BYTES + 1)).is_err());
    assert!(RegisteredCheck::from_ntriples(&text).is_ok());
}
