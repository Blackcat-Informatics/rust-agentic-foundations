// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native storage, exact canonical identity and closed-profile neighbors.
use raf_rdf::{Builder, Error, Format, Graph, TermBox, TermValue as Term, Triple, vocab};

fn triple(subject: &str, object: Term) -> Triple {
    Triple {
        subject: Term::iri(subject),
        predicate: Term::iri("urn:predicate"),
        object,
    }
}
fn quoted(subject: Term, object: Term) -> Term {
    Term::Triple {
        s: TermBox::new(subject),
        p: TermBox::new(Term::iri("urn:predicate")),
        o: TermBox::new(object),
    }
}
fn assertions() -> Vec<Triple> {
    vec![
        triple("#node", Term::simple_literal("é\n\0\"\\\u{85}")),
        triple("", Term::iri("#node")),
        Triple {
            subject: Term::iri("#proof"),
            predicate: Term::iri(vocab::REIFIES),
            object: quoted(Term::iri(""), Term::iri("#node")),
        },
        triple("#integer", Term::integer(123)),
        triple("#boolean", Term::boolean(true)),
        triple("#typed", Term::typed_literal("\tcustom雪", "urn:datatype")),
    ]
}
fn build(assertions: impl IntoIterator<Item = Triple>) -> Result<Graph, Error> {
    let mut builder = Builder::new();
    for assertion in assertions {
        builder.insert(assertion)?;
    }
    builder.finish()
}
fn example() -> Graph {
    build(assertions()).unwrap()
}

#[test]
fn ingestion_order_does_not_change_either_format_or_snapshot() {
    let graph = example();
    let reverse = build(assertions().into_iter().rev()).unwrap();
    let snapshot = graph.snapshot().unwrap();
    for format in [Format::Turtle, Format::NTriples] {
        let text = graph.to_text(format).unwrap();
        assert_eq!(text, reverse.to_text(format).unwrap());
        let parsed = Graph::from_text(&text, format).unwrap();
        assert_eq!(parsed.snapshot().unwrap(), snapshot);
        assert_eq!(parsed.to_text(format).unwrap(), text);
        assert!(parsed.dataset().reifiers().next().is_some());
        let proof = parsed
            .get("#proof")
            .expect("native named statement subject");
        assert_eq!(proof.values(vocab::REIFIES).count(), 1);
        assert!(
            proof
                .values(vocab::REIFIES)
                .next()
                .unwrap()
                .quoted()
                .is_some()
        );
        if format == Format::NTriples {
            assert!(text.starts_with(&format!("<{snapshot}> ")));
        }
    }
    let mut changed = Builder::from_graph(&graph);
    changed
        .insert(triple("", Term::simple_literal("changed fact")))
        .unwrap();
    assert_ne!(changed.finish().unwrap().snapshot().unwrap(), snapshot);
}

#[test]
fn duplicate_native_membership_and_real_subject_queries_have_valid_neighbors() {
    let value = triple("", Term::simple_literal("value"));
    let mut builder = Builder::new();
    builder.insert(value.clone()).unwrap();
    assert!(builder.contains(value.clone()).unwrap());
    assert_eq!(builder.insert(value.clone()), Err(Error::Repeated));
    let graph = builder.finish().unwrap();
    assert!(graph.get("").is_some());
    assert!(
        graph.get("urn:predicate").is_none(),
        "an interned predicate is not an asserted subject"
    );
    let mut edited = Builder::from_graph(&graph);
    assert!(edited.remove(value.clone()).unwrap());
    assert!(!edited.remove(value.clone()).unwrap());
    assert!(
        edited.finish().is_err(),
        "the root has no remaining assertion"
    );
    assert!(build([value]).is_ok());
}

#[test]
fn parsed_statement_duplicates_and_repeated_source_bytes_refuse_with_valid_neighbors() {
    for format in [Format::Turtle, Format::NTriples] {
        let text = example().to_text(format).unwrap();
        let parsed = Graph::from_text(&text, format).unwrap();
        let statement = parsed
            .assertions()
            .find(|t| t.predicate == Term::iri(vocab::REIFIES))
            .unwrap();
        let mut builder = Builder::from_graph(&parsed);
        assert!(builder.contains(statement.clone()).unwrap());
        assert_eq!(builder.insert(statement), Err(Error::Repeated));
        assert_eq!(builder.finish().unwrap().to_text(format).unwrap(), text);
        let assertion = text.lines().find(|line| line.ends_with(" .")).unwrap();
        let repeated = format!("{text}{assertion}\n");
        assert!(matches!(
            Graph::from_text(&repeated, format),
            Err(Error::NotCanonical { .. } | Error::Repeated)
        ));
        assert_eq!(
            Graph::from_text(&text, format)
                .unwrap()
                .to_text(format)
                .unwrap(),
            text
        );
    }
}

#[test]
fn roles_missing_roots_and_dangling_references_refuse_beside_valid_graphs() {
    assert!(build([triple("#node", Term::simple_literal("no root"))]).is_err());
    for value in [
        Term::iri("#absent"),
        quoted(Term::iri("#absent"), Term::simple_literal("v")),
        quoted(Term::iri(""), Term::iri("#absent")),
    ] {
        assert!(build([triple("", value.clone())]).is_err());
        assert!(
            build([
                triple("", value),
                triple("#absent", Term::simple_literal("present"))
            ])
            .is_ok()
        );
    }
    for invalid in [
        Triple {
            subject: Term::simple_literal("subject"),
            ..triple("", Term::iri("urn:o"))
        },
        Triple {
            predicate: Term::iri("#relative"),
            ..triple("", Term::iri("urn:o"))
        },
        triple("urn:foreign", Term::iri("urn:o")),
        triple(
            "",
            quoted(Term::simple_literal("subject"), Term::iri("urn:o")),
        ),
        triple(
            "",
            quoted(Term::iri(""), quoted(Term::iri(""), Term::iri("urn:o"))),
        ),
        triple("", Term::iri("urn:raf:report:construction#external")),
    ] {
        assert!(build([invalid]).is_err());
    }
    assert!(build([triple("", quoted(Term::iri(""), Term::iri("urn:o")))]).is_ok());
}

#[test]
fn absolute_iri_and_literal_profile_refusals_keep_valid_neighbors() {
    for value in [
        "https://[",
        "urn:a[b",
        "urn:a#b#c",
        "urn:a%",
        "urn:a%GG",
        "urn:a\u{7f}",
        "urn:a\u{85}",
        "relative",
        "1bad:a",
    ] {
        assert!(build([triple("", Term::iri(value))]).is_err(), "{value:?}");
    }
    for value in [
        "https://[::1]/é",
        "urn:example:a",
        "https://example.org/a%FF",
        "https://example.org/%2f",
        "https://example.org/雪",
    ] {
        assert!(build([triple("", Term::iri(value))]).is_ok(), "{value}");
    }
    for value in [
        Term::lang_literal("text", "en"),
        Term::typed_literal("text", vocab::RDF.to_owned() + "langString"),
        Term::typed_literal("text", vocab::RDF.to_owned() + "dirLangString"),
        Term::typed_literal("text", "relative"),
    ] {
        assert!(build([triple("", value.clone())]).is_err());
        assert!(build([triple("", quoted(Term::iri(""), value))]).is_err());
    }
    assert!(build([triple("", Term::typed_literal("text", "urn:datatype"))]).is_ok());
}

#[test]
fn exact_bytes_include_cr_final_lf_and_multibyte_diagnostic_neighbors() {
    for format in [Format::Turtle, Format::NTriples] {
        let text = example().to_text(format).unwrap();
        assert!(matches!(
            Graph::from_text(&text.replace('\n', "\r\n"), format),
            Err(Error::NotCanonical { line: 1 })
        ));
        assert!(
            matches!(Graph::from_text(text.strip_suffix('\n').unwrap(), format), Err(Error::NotCanonical { line }) if line == text.lines().count())
        );
        for altered in [
            format!("{text}\n"),
            format!(" {text}"),
            text.replace("é", "\\u00E9"),
            text.replace(" .\n", "  .\n"),
        ] {
            assert!(matches!(
                Graph::from_text(&altered, format),
                Err(Error::NotCanonical { .. })
            ));
        }
        assert_eq!(
            Graph::from_text(&text, format)
                .unwrap()
                .to_text(format)
                .unwrap(),
            text
        );
    }
}

#[test]
fn both_readers_preserve_role_language_depth_and_ownership_refusals() {
    for format in [Format::Turtle, Format::NTriples] {
        let graph = example();
        let text = graph.to_text(format).unwrap();
        for altered in [
            text.replace("urn:datatype", purrdf_iri::vocab::rdf::LANG_STRING),
            text.replace("urn:datatype", purrdf_iri::vocab::rdf::DIR_LANG_STRING),
            text.replace("<<( ", "<<( <<( "),
            text.replace("urn:predicate", "#relative"),
            text.replace("#node>", "#missing>"),
            text.replace("\\n", "\\q"),
            text.replace("é", "\\uD800"),
            text.replace("é", "\\U00110000"),
            text.replace("\"123\"", "\"124\""),
        ] {
            assert_ne!(altered, text);
            assert!(Graph::from_text(&altered, format).is_err());
        }
        let root = if format == Format::Turtle {
            "<>".into()
        } else {
            format!("<{}>", graph.snapshot().unwrap())
        };
        let invalid = text.replacen(
            &format!("{root} <urn:predicate>"),
            "\"subject\" <urn:predicate>",
            1,
        );
        assert_ne!(invalid, text);
        assert!(Graph::from_text(&invalid, format).is_err());
        assert!(Graph::from_text("", format).is_err());
        assert!(
            Graph::from_text(
                &text,
                if format == Format::Turtle {
                    Format::NTriples
                } else {
                    Format::Turtle
                }
            )
            .is_err()
        );
        assert_eq!(
            Graph::from_text(&text, format).unwrap().snapshot().unwrap(),
            graph.snapshot().unwrap()
        );
    }
}

#[test]
fn pinned_registry_expansions_remain_distinct_from_native_canonical_bytes() {
    let turtle = include_str!("fixtures/profile.ttl");
    let nt = include_str!("fixtures/profile.nt");
    let a = Graph::from_text(turtle, Format::Turtle).unwrap();
    let b = Graph::from_text(nt, Format::NTriples).unwrap();
    assert_eq!(a.snapshot().unwrap(), b.snapshot().unwrap());
    assert_eq!(a.to_text(Format::Turtle).unwrap(), turtle);
    assert_eq!(b.to_text(Format::NTriples).unwrap(), nt);
    let mut triples: Vec<_> = nt.lines().collect();
    triples.sort_unstable();
    // A distinct registry revision of the same native parser family emits the
    // actual C1 scalar escaped; this comparison does not rewrite production text.
    let compared = format!("{}\n", triples.join("\n")).replace('\u{85}', "\\u0085");
    assert_eq!(compared, include_str!("fixtures/profile.oracle.nt"));
    assert!(a.assertions().any(|t| matches!(t.object, Term::Literal { lexical_form, .. } if lexical_form == "literal \\u0085")));
}
