// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The schema table and the timestamp rule.

use raf_okf::{DocType, TimestampError, ValueKind, validate_timestamp};

fn keys(doc_type: DocType) -> Vec<&'static str> {
    doc_type.keys().iter().map(|spec| spec.key).collect()
}

#[test]
fn each_type_lists_its_keys_in_the_order_of_the_architecture_page() {
    assert_eq!(
        keys(DocType::Capability),
        [
            "type",
            "id",
            "title",
            "description",
            "tags",
            "timestamp",
            "state",
            "bindings",
            "depends"
        ]
    );
    assert_eq!(
        keys(DocType::Change),
        [
            "type",
            "id",
            "title",
            "description",
            "tags",
            "timestamp",
            "changes",
            "affects",
            "contract_effect",
            "items"
        ]
    );
    assert_eq!(
        keys(DocType::Architecture),
        ["type", "id", "title", "description", "tags", "timestamp"]
    );
}

#[test]
fn tags_and_timestamp_are_optional_and_every_other_key_is_required() {
    for doc_type in DocType::ALL {
        for spec in doc_type.keys() {
            let optional = matches!(spec.key, "tags" | "timestamp");
            assert_eq!(spec.required, !optional, "{} {}", doc_type.name(), spec.key);
        }
        let tags = doc_type
            .keys()
            .iter()
            .find(|s| s.key == "tags")
            .expect("tags");
        assert_eq!(tags.value, ValueKind::Tags);
        let timestamp = doc_type
            .keys()
            .iter()
            .find(|s| s.key == "timestamp")
            .expect("timestamp");
        assert_eq!(timestamp.value, ValueKind::Timestamp);
        assert!(!doc_type.keys().iter().any(|s| s.key == "resource"));
    }
}

#[test]
fn a_type_is_found_by_its_exact_name_only() {
    for doc_type in DocType::ALL {
        assert_eq!(DocType::from_name(doc_type.name()), Some(doc_type));
    }
    for name in ["capability", "CAPABILITY", "Concept", "", "Capability "] {
        assert_eq!(DocType::from_name(name), None, "{name:?}");
    }
    assert_eq!(DocType::Capability.directory(), "docs/capabilities");
    assert_eq!(DocType::Change.directory(), "work/changes");
    assert_eq!(DocType::Architecture.directory(), "docs/architecture");
    assert_eq!(DocType::Capability.id_prefix(), "cap:");
    assert_eq!(DocType::Change.id_prefix(), "change:");
    assert_eq!(DocType::Architecture.id_prefix(), "arch:");
}

#[test]
fn valid_timestamps_are_accepted() {
    for text in [
        "2026-01-01T00:00:00Z",
        "2026-12-31T23:59:59Z",
        "2024-02-29T12:00:00Z",
        "2000-02-29T12:00:00Z",
        "2026-01-01T00:00:00.5Z",
        "2026-01-01T00:00:00.123456789+05:30",
        "2026-01-01T00:00:00-08:00",
    ] {
        assert_eq!(validate_timestamp(text), Ok(()), "{text}");
    }
}

#[test]
fn malformed_and_out_of_range_timestamps_are_refused() {
    for text in [
        "",
        "2026",
        "2026-01-01",
        "2026-01-01T00:00:00",
        "2026-01-01 00:00:00Z",
        "2026-01-01t00:00:00Z",
        "2026-01-01T00:00:00z",
        "2026-1-01T00:00:00Z",
        "2026-01-01T00:00:00.Z",
        "2026-01-01T00:00:00+0530",
        "2026-01-01T00:00:00Z ",
        "2026-01-01T00:00Z",
        "+2026-01-01T00:00:00Z",
        "２０２６-01-01T00:00:00Z",
    ] {
        assert_eq!(
            validate_timestamp(text),
            Err(TimestampError::Malformed),
            "{text:?}"
        );
    }
    for (text, field) in [
        ("2026-13-01T00:00:00Z", "month"),
        ("2026-00-01T00:00:00Z", "month"),
        ("2026-01-00T00:00:00Z", "day"),
        ("2026-01-32T00:00:00Z", "day"),
        ("2026-04-31T00:00:00Z", "day"),
        ("2026-02-29T00:00:00Z", "day"),
        ("1900-02-29T00:00:00Z", "day"),
        ("2026-01-01T24:00:00Z", "hour"),
        ("2026-01-01T00:60:00Z", "minute"),
        ("2026-01-01T00:00:60Z", "second"),
        ("2026-01-01T00:00:00+24:00", "offset hour"),
        ("2026-01-01T00:00:00+00:60", "offset minute"),
    ] {
        assert_eq!(
            validate_timestamp(text),
            Err(TimestampError::OutOfRange { field }),
            "{text}"
        );
    }
}
