// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The schema table and the timestamp rule.

use std::path::{Path, PathBuf};

use raf_okf::{DocType, TimestampError, ValueKind, validate_timestamp};

fn keys(doc_type: DocType) -> Vec<&'static str> {
    doc_type.keys().iter().map(|spec| spec.key).collect()
}

/// The architecture page, from the gate root when it is set.
fn architecture_page() -> String {
    let root = std::env::var_os("RAF_GATE_ROOT").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        PathBuf::from,
    );
    std::fs::read_to_string(root.join("docs/architecture/okf-profile.md")).expect("the page")
}

/// A row of the type table: the type, its directory, and each key with
/// whether the page marks it optional.
type PageRow = (String, String, Vec<(String, bool)>);

/// The rows of the type table of the architecture page.
fn page_rows(page: &str) -> Vec<PageRow> {
    let table = page
        .split("| Type | Directory | Keys, in writing order |")
        .nth(1)
        .expect("the page has the type table");
    table
        .lines()
        .skip(2)
        .take_while(|line| line.starts_with('|'))
        .map(|line| {
            let cells: Vec<&str> = line.split('|').map(str::trim).collect();
            let keys = cells[3]
                .split(", ")
                .map(|key| {
                    let (optional, key) = match key.strip_prefix("optional ") {
                        Some(key) => (true, key),
                        None => (false, key),
                    };
                    (key.trim_matches('`').to_string(), optional)
                })
                .collect();
            (
                cells[1].to_string(),
                cells[2].trim_matches('`').to_string(),
                keys,
            )
        })
        .collect()
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
            "depends",
            "review_required"
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
    assert_eq!(
        keys(DocType::Evidence),
        [
            "type",
            "id",
            "tags",
            "timestamp",
            "subject",
            "inputs",
            "verdict",
            "by",
            "for",
            "command",
            "revision",
            "values",
            "limits"
        ]
    );
    assert_eq!(
        keys(DocType::Assessment),
        [
            "type",
            "id",
            "tags",
            "timestamp",
            "subject",
            "inputs",
            "verdict",
            "by"
        ]
    );
    assert_eq!(
        keys(DocType::Coverage),
        [
            "type",
            "id",
            "tags",
            "timestamp",
            "scope",
            "exclusions",
            "ceilings",
            "exceptions"
        ]
    );
    assert_eq!(
        keys(DocType::Debt),
        ["type", "id", "tags", "timestamp", "findings", "lines"]
    );
    // The page states the same table: one row per type, in the order of
    // `DocType::ALL`, with the directory, the keys in order and which of them
    // are optional.
    let rows = page_rows(&architecture_page());
    assert_eq!(rows.len(), DocType::ALL.len(), "{rows:#?}");
    for (doc_type, (name, directory, page_keys)) in DocType::ALL.into_iter().zip(rows) {
        assert_eq!(name, doc_type.name());
        assert_eq!(directory, doc_type.directory(), "{name}");
        let table: Vec<(String, bool)> = doc_type
            .keys()
            .iter()
            .map(|spec| (spec.key.to_string(), !spec.required))
            .collect();
        assert_eq!(page_keys, table, "{name}");
    }
}

#[test]
fn each_type_states_its_required_and_optional_keys() {
    for doc_type in DocType::ALL {
        for spec in doc_type.keys() {
            // Evidence also leaves out what a run may not have: the command
            // of a bound test, the revision, and the measures. The coverage
            // record leaves out its ceilings and exceptions, none without them.
            let optional = spec.key == "tags"
                || (spec.key == "timestamp" && doc_type != DocType::Review)
                || (doc_type == DocType::Capability && spec.key == "review_required")
                || (doc_type == DocType::Review && spec.key == "findings_ref")
                || (doc_type == DocType::Evidence
                    && matches!(spec.key, "command" | "revision" | "values" | "limits"))
                || (doc_type == DocType::Coverage && matches!(spec.key, "ceilings" | "exceptions"));
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
    assert_eq!(DocType::Evidence.directory(), "work/evidence");
    assert_eq!(DocType::Assessment.directory(), "work/assessments");
    assert_eq!(DocType::Coverage.directory(), "work/coverage");
    assert_eq!(DocType::Debt.directory(), "work/debt");
    assert_eq!(DocType::Capability.id_prefix(), "cap:");
    assert_eq!(DocType::Change.id_prefix(), "change:");
    assert_eq!(DocType::Architecture.id_prefix(), "arch:");
    assert_eq!(DocType::Evidence.id_prefix(), "evidence:");
    assert_eq!(DocType::Assessment.id_prefix(), "assessment:");
    assert_eq!(DocType::Coverage.id_prefix(), "coverage:");
    assert_eq!(DocType::Debt.id_prefix(), "debt:");
}

#[test]
fn singleton_and_record_roles_follow_the_document_types() {
    for doc_type in DocType::ALL {
        let record = matches!(
            doc_type,
            DocType::Evidence
                | DocType::Assessment
                | DocType::Coverage
                | DocType::Debt
                | DocType::Review
        );
        assert_eq!(doc_type.is_record(), record, "{}", doc_type.name());
        // A record has no title or description; a document has both.
        let titled = doc_type.keys().iter().any(|spec| spec.key == "title");
        assert_eq!(titled, !record, "{}", doc_type.name());
        let singleton = matches!(doc_type, DocType::Coverage | DocType::Debt).then_some("default");
        assert_eq!(doc_type.singleton(), singleton, "{}", doc_type.name());
    }
    assert_eq!(
        DocType::listed("or"),
        "Capability, Change, Architecture, Evidence, Assessment, Coverage, Debt or Review"
    );
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
    // Native RFC parsing admits correctly placed leap seconds; RAF's authored
    // timestamp profile deliberately excludes them in every offset.
    for leap in ["2016-12-31T23:59:60Z", "2017-01-01T00:59:60+01:00"] {
        assert_eq!(
            validate_timestamp(leap),
            Err(TimestampError::OutOfRange { field: "second" })
        );
        assert_eq!(validate_timestamp(&leap.replace(":60", ":59")), Ok(()));
    }
}

#[test]
fn timestamp_diagnostics_preserve_lexical_calendar_and_offset_order() {
    use TimestampError::{Malformed, OutOfRange};

    for (text, expected) in [
        ("2026-13-01", Malformed),
        ("2026-13-01T00:00:00", OutOfRange { field: "month" }),
        ("2026-13-01T00:00:00z", OutOfRange { field: "month" }),
        ("2026-13-01T00:00:00.Z", Malformed),
        ("2026-13-01T00:00:xxZ", Malformed),
        ("2026-13-01t00:00:00Z", Malformed),
        ("2026-01-01T00:00:60oops", OutOfRange { field: "second" }),
        ("2026-01-01T00:00:60+99:99", OutOfRange { field: "second" }),
        ("2026-13-01T00:00:60oops", OutOfRange { field: "month" }),
        ("2026-01-01T00:00:60.Z", Malformed),
        ("2026-01-01T00:00:00+99:00oops", Malformed),
        ("2026-01-01T00:00:00+99:xx", Malformed),
        (
            "2026-01-01T00:00:00+99:00",
            OutOfRange {
                field: "offset hour",
            },
        ),
        ("2026-01-01T00:00:00+00:99oops", Malformed),
        (
            "2026-01-01T00:00:00+00:99",
            OutOfRange {
                field: "offset minute",
            },
        ),
        ("2016-12-31T23:59:60Z", OutOfRange { field: "second" }),
        ("2017-01-01T00:59:60+01:00", OutOfRange { field: "second" }),
    ] {
        assert_eq!(validate_timestamp(text), Err(expected), "{text}");
    }
    for text in [
        "2026-12-01T00:00:00Z",
        "2026-01-01T00:00:59+23:59",
        "2026-01-01T00:00:00.1-00:59",
        "2016-12-31T23:59:59Z",
        "2017-01-01T00:59:59+01:00",
    ] {
        assert_eq!(validate_timestamp(text), Ok(()), "{text}");
    }
}
