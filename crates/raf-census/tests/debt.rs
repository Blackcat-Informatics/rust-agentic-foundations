// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Exact debt comparison and the shared native RDF acceptance boundary.

use std::collections::BTreeMap;

use raf_census::{DebtChange, DebtGroup, DebtRecord, GroupKind, MAX_DEBT_BYTES, NORMAL_FORM};
#[path = "support/layout.rs"]
mod layout;

#[test]
fn oversized_records_refuse_before_parsing_beside_a_readable_record() {
    let text = " ".repeat(MAX_DEBT_BYTES + 1);
    assert!(
        raf_census::upgrade_report_form(
            raf_census::ReportKind::Debt,
            raf_census::MigrationInput {
                path: "oversized-debt",
                text: &text
            },
            Format::Turtle
        )
        .unwrap_err()
        .to_string()
        .contains("maximum is")
    );
    for error in [
        DebtRecord::from_turtle(&text).unwrap_err(),
        DebtRecord::from_ntriples(&text).unwrap_err(),
    ] {
        assert!(error.to_string().contains("maximum is"));
    }
    let valid = DebtRecord::new(NORMAL_FORM, [], BTreeMap::new()).unwrap();
    assert_eq!(
        DebtRecord::from_turtle(&valid.to_turtle().unwrap()).unwrap(),
        valid
    );
}
use raf_rdf::{Builder, Format, Graph, TermValue, Triple, vocab};

fn group(members: &[&str]) -> DebtGroup {
    DebtGroup::new(
        GroupKind::Body,
        members.iter().map(|member| (*member).into()),
    )
    .unwrap()
}

fn record(members: &[&str], copies: usize) -> DebtRecord {
    let groups = if members.is_empty() {
        vec![]
    } else {
        vec![group(members)]
    };
    let jobs = if copies == 0 {
        BTreeMap::new()
    } else {
        BTreeMap::from([("encode".into(), copies)])
    };
    DebtRecord::new(NORMAL_FORM, groups, jobs).unwrap()
}

#[test]
fn counted_cfg_members_round_trip_and_lower_without_changing_symbol_identity() {
    let counted = |count| {
        DebtRecord::new(
            NORMAL_FORM,
            [DebtGroup::with_counts(
                GroupKind::Body,
                BTreeMap::from([("f".into(), count), ("g".into(), 1)]),
            )
            .unwrap()],
            BTreeMap::new(),
        )
        .unwrap()
    };
    let old = counted(2);
    for format in [Format::Turtle, Format::NTriples] {
        let text = match format {
            Format::Turtle => old.to_turtle().unwrap(),
            Format::NTriples => old.to_ntriples().unwrap(),
        };
        layout::assert_upgrade(raf_census::ReportKind::Debt, &text, format);
    }
    assert_eq!(
        DebtRecord::from_turtle(&old.to_turtle().unwrap()).unwrap(),
        old
    );
    assert_eq!(
        DebtRecord::from_ntriples(&old.to_ntriples().unwrap()).unwrap(),
        old
    );
    let lower = counted(1);
    assert_eq!(lower, record(&["f", "g"], 0));
    let changes = old.compare(&lower).unwrap();
    assert_ne!(changes, []);
    assert!(changes.iter().all(|change| !change.is_growth()));
    assert!(
        lower
            .compare(&old)
            .unwrap()
            .iter()
            .any(DebtChange::is_growth)
    );
    let single_symbol =
        DebtGroup::with_counts(GroupKind::Body, BTreeMap::from([("f".into(), 2)])).unwrap();
    let record = DebtRecord::new(NORMAL_FORM, [single_symbol], BTreeMap::new()).unwrap();
    assert_eq!(
        DebtRecord::from_turtle(&record.to_turtle().unwrap()).unwrap(),
        record
    );
    for members in [
        BTreeMap::from([("f".into(), 0)]),
        BTreeMap::from([("f".into(), 1)]),
        BTreeMap::from([("f".into(), usize::MAX), ("g".into(), 1)]),
    ] {
        assert!(DebtGroup::with_counts(GroupKind::Body, members).is_err());
    }
}

#[test]
fn counted_nodes_refuse_redundant_counts_and_unowned_assertions() {
    let original = DebtRecord::new(
        NORMAL_FORM,
        [DebtGroup::with_counts(GroupKind::Body, BTreeMap::from([("f".into(), 2)])).unwrap()],
        BTreeMap::new(),
    )
    .unwrap();
    let graph = Graph::from_text(&original.to_turtle().unwrap(), Format::Turtle).unwrap();
    let copies = |count: &str| Triple {
        subject: TermValue::iri("#debt/body/f@2/multiplicity/f"),
        predicate: TermValue::iri(format!("{}copies", vocab::REPORT)),
        object: TermValue::typed_literal(count, vocab::INTEGER),
    };
    let mut redundant = Builder::from_graph(&graph);
    assert!(redundant.remove(copies("2")).unwrap());
    redundant.insert(copies("1")).unwrap();
    let text = redundant.finish().unwrap().to_text(Format::Turtle).unwrap();
    assert!(DebtRecord::from_turtle(&text).is_err());
    let mut orphan = Builder::from_graph(&graph);
    for (key, value) in [
        (
            vocab::TYPE.to_owned(),
            TermValue::iri(format!("{}MemberMultiplicity", vocab::REPORT)),
        ),
        (
            format!("{}symbol", vocab::REPORT),
            TermValue::simple_literal("extra"),
        ),
        (
            format!("{}copies", vocab::REPORT),
            TermValue::typed_literal("2", vocab::INTEGER),
        ),
    ] {
        orphan
            .insert(Triple {
                subject: TermValue::iri("#orphan"),
                predicate: TermValue::iri(key),
                object: value,
            })
            .unwrap();
    }
    let text = orphan.finish().unwrap().to_text(Format::Turtle).unwrap();
    assert!(DebtRecord::from_turtle(&text).is_err());
    assert_eq!(
        DebtRecord::from_turtle(&original.to_turtle().unwrap()).unwrap(),
        original
    );
}

#[test]
fn new_groups_supersets_and_rising_copy_counts_are_growth() {
    let empty = record(&[], 0);
    let original = record(&["a", "b"], 2);
    for (before, after) in [
        (&empty, &original),
        (&original, &record(&["a", "b", "c"], 2)),
        (&original, &record(&["a", "b"], 3)),
    ] {
        assert!(
            before
                .compare(after)
                .unwrap()
                .iter()
                .any(DebtChange::is_growth)
        );
    }
    assert_eq!(original.compare(&original).unwrap(), []);
    let superset = original.compare(&record(&["a", "b", "c"], 2)).unwrap();
    assert!(
        superset
            .iter()
            .any(|change| matches!(change, DebtChange::NewGroup(_)))
    );
    assert!(
        superset
            .iter()
            .any(|change| matches!(change, DebtChange::MissingGroup { .. }))
    );
}

#[test]
fn a_present_superset_cannot_hide_a_missing_exact_recorded_group() {
    let smaller = group(&["a", "b"]);
    let larger = group(&["a", "b", "c"]);
    let recorded = DebtRecord::new(
        NORMAL_FORM,
        [smaller.clone(), larger.clone()],
        BTreeMap::new(),
    )
    .unwrap();
    let present = DebtRecord::new(NORMAL_FORM, [larger], BTreeMap::new()).unwrap();
    let changes = recorded.compare(&present).unwrap();
    assert_eq!(
        changes,
        vec![DebtChange::MissingGroup {
            recorded: smaller,
            present: vec![]
        }]
    );
    let added = present.compare(&recorded).unwrap();
    assert!(matches!(&added[..], [DebtChange::NewGroup(_)]));
    assert_eq!(present.compare(&present).unwrap(), []);
}

#[test]
fn resolved_groups_strict_subsets_and_falling_counts_require_a_lower_record() {
    let original = record(&["a", "b", "c"], 3);
    for present in [
        record(&[], 3),
        record(&["a", "b"], 3),
        record(&["a", "b", "c"], 2),
        record(&["a", "b", "c"], 0),
    ] {
        let changes = original.compare(&present).unwrap();
        assert_ne!(changes, []);
        assert!(changes.iter().all(|change| !change.is_growth()));
        assert_eq!(present.compare(&present).unwrap(), []);
    }
}

#[test]
fn a_group_of_a_different_kind_is_new_debt() {
    let original = record(&["a", "b"], 0);
    let shim = DebtGroup::new(GroupKind::Shim, ["a".into(), "b".into()]).unwrap();
    let present = DebtRecord::new(NORMAL_FORM, [shim], BTreeMap::new()).unwrap();
    assert!(
        original
            .compare(&present)
            .unwrap()
            .iter()
            .any(DebtChange::is_growth)
    );
}

#[test]
fn both_native_spellings_round_trip_and_input_order_does_not_change_bytes() {
    let original = record(&["a", "b", "c"], 3);
    let turtle = original.to_turtle().unwrap();
    let ntriples = original.to_ntriples().unwrap();
    let converted = layout::assert_upgrade(raf_census::ReportKind::Debt, &turtle, Format::Turtle);
    let upgraded = DebtRecord::from_turtle(&converted).unwrap();
    layout::assert_upgrade(raf_census::ReportKind::Debt, &ntriples, Format::NTriples);
    assert_eq!(upgraded.compare(&original).unwrap(), []);
    assert!(
        upgraded
            .compare(&record(&["a", "b", "c"], 4))
            .unwrap()
            .iter()
            .any(DebtChange::is_growth)
    );
    assert!(
        upgraded
            .compare(&record(&["a", "b"], 2))
            .unwrap()
            .iter()
            .all(|change| !change.is_growth())
    );
    assert_eq!(DebtRecord::from_turtle(&turtle).unwrap(), original);
    assert_eq!(DebtRecord::from_ntriples(&ntriples).unwrap(), original);
    assert_eq!(record(&["c", "a", "b"], 3).to_turtle().unwrap(), turtle);
    assert_eq!(original.to_turtle().unwrap(), turtle);
    assert!(DebtRecord::from_turtle(&turtle.replace("\"a\"", "\"changed\"")).is_err());
}

#[test]
fn commas_percent_signs_and_unicode_do_not_alias_member_sets() {
    for members in [["a,b", "c"], ["a", "b,c"], ["%2C", "é"]] {
        let original = record(&members, 0);
        assert_eq!(
            DebtRecord::from_turtle(&original.to_turtle().unwrap()).unwrap(),
            original
        );
    }
    assert_ne!(
        record(&["a,b", "c"], 0).to_turtle().unwrap(),
        record(&["a", "b,c"], 0).to_turtle().unwrap()
    );
}

#[test]
fn other_versions_can_be_read_but_cannot_be_compared() {
    let old = DebtRecord::new(NORMAL_FORM + 1, [], BTreeMap::new()).unwrap();
    let decoded = DebtRecord::from_turtle(&old.to_turtle().unwrap()).unwrap();
    assert_eq!(decoded, old);
    assert!(decoded.compare(&record(&[], 0)).is_err());
    assert!(record(&[], 0).compare(&decoded).is_err());
    assert_eq!(record(&[], 0).compare(&record(&[], 0)).unwrap(), []);
}

#[test]
fn invalid_groups_and_zero_copy_rows_are_refused() {
    assert!(DebtGroup::new(GroupKind::Body, ["a".into()]).is_err());
    assert!(DebtGroup::new(GroupKind::Body, ["a".into(), "a".into()]).is_err());
    assert!(DebtGroup::new(GroupKind::Body, ["a".into(), String::new()]).is_err());
    assert!(DebtGroup::new(GroupKind::Body, ["a".into(), "b".into()]).is_ok());
    assert!(DebtRecord::new(NORMAL_FORM, [], BTreeMap::from([("x".into(), 0)])).is_err());
    assert!(DebtRecord::new(NORMAL_FORM, [], BTreeMap::from([(String::new(), 1)])).is_err());
    assert!(DebtRecord::new(0, [], BTreeMap::new()).is_err());
    let duplicate = group(&["a", "b"]);
    assert!(DebtRecord::new(NORMAL_FORM, [duplicate.clone(), duplicate], BTreeMap::new()).is_err());
}

#[test]
fn recomputing_a_digest_does_not_admit_foreign_classes_or_unknown_properties() {
    let original = record(&["a", "b"], 1);
    let graph = Graph::from_text(&original.to_turtle().unwrap(), Format::Turtle).unwrap();
    for (key, value) in [
        ("unknown", TermValue::simple_literal("extra")),
        ("type", TermValue::iri(format!("{}Index", vocab::REPORT))),
    ] {
        let mut changed = Builder::from_graph(&graph);
        if key == "type" {
            assert!(
                changed
                    .remove(Triple {
                        subject: TermValue::iri(""),
                        predicate: TermValue::iri(vocab::TYPE),
                        object: TermValue::iri(format!("{}DebtRecord", vocab::REPORT)),
                    })
                    .unwrap()
            );
        }
        changed
            .insert(Triple {
                subject: TermValue::iri(""),
                predicate: TermValue::iri(format!(
                    "{}{key}",
                    if key == "type" {
                        vocab::RDF
                    } else {
                        vocab::REPORT
                    }
                )),
                object: value,
            })
            .unwrap();
        let canonical = changed.finish().unwrap().to_text(Format::Turtle).unwrap();
        assert!(DebtRecord::from_turtle(&canonical).is_err());
    }
    assert_eq!(
        DebtRecord::from_turtle(&original.to_turtle().unwrap()).unwrap(),
        original
    );
}
