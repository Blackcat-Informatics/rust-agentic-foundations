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
    use std::ffi::OsString;
    assert_eq!(
        RegisteredOperation::from_args([]).unwrap(),
        RegisteredOperation::Check
    );
    assert_eq!(
        RegisteredOperation::from_args([OsString::from("--census-debt-write-v1")]).unwrap(),
        RegisteredOperation::WriteDebt
    );
    assert_eq!(
        RegisteredOperation::from_args([OsString::from("--census-index-v1")]).unwrap(),
        RegisteredOperation::Index
    );
    for args in [
        vec!["--write"],
        vec!["--unknown"],
        vec!["--census-debt-write-v1", "--census-debt-write-v1"],
        vec!["--census-debt-write-v1", "extra"],
        vec!["--census-index-v1", "extra"],
        vec!["--census-index-v1", "--census-debt-write-v1"],
    ] {
        assert!(RegisteredOperation::from_args(args.into_iter().map(OsString::from)).is_err());
    }
    let receipt = debt_receipt();
    let text = receipt.to_ntriples().unwrap();
    let decoded = RegisteredDebtWrite::from_ntriples(&text).unwrap();
    assert_eq!(decoded.counts(), Some((2, 3)));
    assert_eq!(decoded.exit_code(), 0);
    assert_eq!(decoded.error_message(), None);
    assert_eq!(decoded.to_ntriples().unwrap(), text);
    assert!(RegisteredCheck::from_ntriples(&text).is_err());
    let error =
        RegisteredDebtWrite::from_write(&Err(Error::Options("writer cause\nλ".into()))).unwrap();
    let decoded = RegisteredDebtWrite::from_ntriples(&error.to_ntriples().unwrap()).unwrap();
    assert_eq!(decoded.counts(), None);
    assert_eq!(decoded.exit_code(), 1);
    assert!(decoded.error_message().unwrap().contains("writer cause\nλ"));
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
    let receipt = debt_receipt().to_ntriples().unwrap();
    assert!(RegisteredDebtWrite::from_ntriples(&response().to_ntriples().unwrap()).is_err());
    for (key, old) in [
        ("protocolVersion", Term::iri(DEBT_WRITE_PROTOCOL_VERSION)),
        ("status", Term::simple_literal("written")),
        ("groupCount", shape::integer(2)),
        ("jobCount", shape::integer(3)),
    ] {
        for replacement in [None, Some(Term::simple_literal("wrong"))] {
            let text = edit_debt(&receipt, |graph| {
                replace(graph, "", key, old.clone(), replacement);
            });
            assert!(RegisteredDebtWrite::from_ntriples(&text).is_err(), "{key}");
        }
        let text = edit_debt(&receipt, |graph| {
            put(graph, "", key, Term::simple_literal("duplicate")).unwrap();
        });
        assert!(RegisteredDebtWrite::from_ntriples(&text).is_err(), "{key}");
    }
    for (key, value) in [
        ("unknown", shape::integer(1)),
        ("errorMessage", Term::simple_literal("contradiction")),
        ("type", shape::iri("RegisteredCheck")),
    ] {
        let text = edit_debt(&receipt, |graph| put(graph, "", key, value).unwrap());
        assert!(RegisteredDebtWrite::from_ntriples(&text).is_err());
    }
    let error = RegisteredDebtWrite::from_write(&Err(Error::Options("cause".into())))
        .unwrap()
        .to_ntriples()
        .unwrap();
    for (key, value) in [
        ("groupCount", shape::integer(0)),
        ("jobCount", shape::integer(0)),
    ] {
        let text = edit_debt(&error, |graph| put(graph, "", key, value).unwrap());
        assert!(RegisteredDebtWrite::from_ntriples(&text).is_err());
    }
    let text = edit_debt(&error, |graph| {
        replace(
            graph,
            "",
            "errorMessage",
            Term::simple_literal(Error::Options("cause".into()).to_string()),
            None,
        );
    });
    assert!(RegisteredDebtWrite::from_ntriples(&text).is_err());
    assert!(RegisteredDebtWrite::from_ntriples(&receipt).is_ok());
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
    let receipt = debt_receipt().to_ntriples().unwrap();
    let text = edit_debt(&receipt, |graph| {
        put(graph, "#orphan", "type", shape::iri("RegisteredDebtWrite")).unwrap();
    });
    assert!(RegisteredDebtWrite::from_ntriples(&text).is_err());
    for value in [
        Term::integer(-1),
        Term::simple_literal("2"),
        shape::integer(u128::MAX),
    ] {
        let text = edit_debt(&receipt, |graph| {
            replace(graph, "", "groupCount", shape::integer(2), Some(value));
        });
        assert!(RegisteredDebtWrite::from_ntriples(&text).is_err());
    }
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
    let debt = debt_receipt().to_ntriples().unwrap();
    assert!(RegisteredDebtWrite::from_ntriples(debt.trim_end_matches('\n')).is_err());
    assert!(RegisteredDebtWrite::from_ntriples(&(debt.clone() + "\n")).is_err());
    assert!(RegisteredDebtWrite::from_ntriples(&debt.replace('\n', "\r\n")).is_err());
    assert!(RegisteredDebtWrite::from_ntriples(&"x".repeat(MAX_CHECK_RESPONSE_BYTES + 1)).is_err());
    assert!(RegisteredDebtWrite::from_ntriples(&debt).is_ok());
    let text = response().to_ntriples().unwrap();
    assert!(RegisteredCheck::from_ntriples(text.trim_end_matches('\n')).is_err());
    assert!(RegisteredCheck::from_ntriples(&(text.clone() + "\n")).is_err());
    assert!(RegisteredCheck::from_ntriples(&text.replace('\n', "\r\n")).is_err());
    assert!(RegisteredCheck::from_ntriples(&"x".repeat(MAX_CHECK_RESPONSE_BYTES + 1)).is_err());
    assert!(RegisteredCheck::from_ntriples(&text).is_ok());
}

fn debt_receipt() -> RegisteredDebtWrite {
    RegisteredDebtWrite::from_write(&Ok(crate::DebtWrite {
        path: "census-debt.ttl".into(),
        groups: 2,
        jobs: 3,
    }))
    .unwrap()
}

fn edit_debt(text: &str, change: impl FnOnce(&mut Builder)) -> String {
    let graph = Graph::from_text(text, Format::NTriples).unwrap();
    let mut builder = Builder::from_graph(&graph);
    change(&mut builder);
    builder.finish().unwrap().to_text(Format::NTriples).unwrap()
}
