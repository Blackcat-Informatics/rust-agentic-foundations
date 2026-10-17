// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The record types: evidence, assessments and coverage. One fixture per rule,
//! each beside a neighbouring record that is accepted.

use raf_okf::{
    Actor, AssessmentVerdict, CapabilitySection, DocType, Document, EvidenceSubject,
    EvidenceVerdict, ExclusionItem, InputKind, Location, Profile, RafDocument, Rule, Violation,
};

const DIGEST: &str = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";
const COMMIT: &str = "b499578a1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f";

const EVIDENCE_PATH: &str = "work/evidence/memory-bound.md";
const ASSESSMENT_PATH: &str = "work/assessments/reconcile-explanation.md";
const COVERAGE_PATH: &str = "work/coverage/default.md";

fn evidence() -> String {
    format!(
        "---\n# SPDX-FileCopyrightText: 2026 Example\n# SPDX-License-Identifier: CC-BY-4.0\n\
         type: Evidence\nid: evidence:memory-bound\nsubject: command\ninputs:\n  \
         - kind: symbol\n    name: raf_okf::Document::parse\n    digest: {DIGEST}\n\
         verdict: pass\nby: agent\nfor: [cap:okf-profile]\n\
         command: \"cargo bench parse\"\nrevision: {COMMIT}\n\
         values:\n  peak_mib: 41\n  seconds: 0.5\nlimits:\n  peak_mib: 64\n---\n"
    )
}

fn assessment() -> String {
    format!(
        "---\ntype: Assessment\nid: assessment:reconcile-explanation\n\
         subject: cap:workplan-reconcile#explanation\ninputs:\n  \
         - kind: section\n    name: cap:workplan-reconcile#explanation\n    digest: {DIGEST}\n  \
         - kind: symbol\n    name: raf_workplan::Workplan::reconcile\n    digest: {DIGEST}\n  \
         - kind: record\n    name: evidence:memory-bound\n    digest: {DIGEST}\n\
         verdict: accepted\nby: human\n---\n"
    )
}

fn coverage() -> String {
    "---\ntype: Coverage\nid: coverage:default\nscope: [raf-workplan, cargo-workplan]\n\
     exclusions:\n  - item: raf_workplan::exec::tests::*\n    reason: \"Unit tests of the process rows.\"\n  \
     - item: raf_workplan::Workplan::empty\n    reason: \"Built by the load only.\"\n---\n"
        .to_string()
}

fn validate(text: &str, path: &str) -> Result<RafDocument, Vec<Violation>> {
    let document = Document::parse(text).unwrap_or_else(|error| panic!("{error} in {text:?}"));
    Profile.validate(&document, path)
}

fn clean(text: &str, path: &str) -> RafDocument {
    validate(text, path).unwrap_or_else(|found| panic!("{path} is not clean: {found:#?}"))
}

fn key(name: &str) -> Location {
    Location::Key(name.to_string())
}

/// Require exactly one violation, at `at`, with exactly `rule`.
fn assert_only(text: &str, path: &str, at: &Location, rule: &Rule) {
    match validate(text, path) {
        Ok(typed) => panic!("expected {rule:?} for {path}, got {typed:#?}"),
        Err(found) => {
            assert_eq!(found.len(), 1, "{found:#?}");
            assert_eq!((found[0].at(), found[0].rule()), (at, rule), "{found:#?}");
        }
    }
}

/// Replace the first `from` in `text` with `to`, requiring that it occurs.
fn mutate(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "fixture has no `{from}`");
    text.replacen(from, to, 1)
}

#[test]
fn a_valid_evidence_record_validates_to_its_typed_value() {
    let RafDocument::Evidence(record) = clean(&evidence(), EVIDENCE_PATH) else {
        panic!("an evidence record");
    };
    assert_eq!(record.header.id, "evidence:memory-bound");
    assert_eq!(record.header.slug, "memory-bound");
    assert_eq!(record.subject, EvidenceSubject::Command);
    assert_eq!(record.inputs.len(), 1);
    assert_eq!(record.inputs[0].kind, InputKind::Symbol);
    assert_eq!(record.inputs[0].name, "raf_okf::Document::parse");
    assert_eq!(record.inputs[0].digest, DIGEST);
    assert_eq!(record.verdict, EvidenceVerdict::Pass);
    assert_eq!(record.by, Actor::Agent);
    assert_eq!(record.covers, ["cap:okf-profile"]);
    assert_eq!(record.command.as_deref(), Some("cargo bench parse"));
    assert_eq!(record.revision.as_deref(), Some(COMMIT));
    let values: Vec<(&str, &str, f64)> = record
        .values
        .iter()
        .map(|m| (m.name.as_str(), m.value.text(), m.value.value()))
        .collect();
    assert_eq!(values, [("peak_mib", "41", 41.0), ("seconds", "0.5", 0.5)]);
    assert_eq!(record.limits.len(), 1);
    assert_eq!(record.limits[0].value.text(), "64");
}

#[test]
fn evidence_from_a_bound_test_has_no_command_and_may_have_no_measures() {
    let text = format!(
        "---\ntype: Evidence\nid: evidence:memory-bound\n\
         subject: binding:raf_okf::test_crate::bounds::a_case\ninputs:\n  \
         - kind: symbol\n    name: raf_okf::Document::parse\n    digest: {DIGEST}\n\
         verdict: fail\nby: human\nfor: [cap:okf-profile, cap:source-index]\n---\n"
    );
    let RafDocument::Evidence(record) = clean(&text, EVIDENCE_PATH) else {
        panic!("an evidence record");
    };
    assert_eq!(
        record.subject,
        EvidenceSubject::Binding("raf_okf::test_crate::bounds::a_case".to_string())
    );
    assert_eq!(record.verdict, EvidenceVerdict::Fail);
    assert_eq!(record.by, Actor::Human);
    assert_eq!((record.command, record.revision), (None, None));
    assert!(record.values.is_empty() && record.limits.is_empty());
    // A failing record may hold a value above its limit: that is why it fails.
    let over = mutate(&evidence(), "verdict: pass", "verdict: fail")
        .replace("peak_mib: 41", "peak_mib: 80");
    clean(&over, EVIDENCE_PATH);
}

#[test]
fn a_valid_assessment_validates_to_its_typed_value() {
    let RafDocument::Assessment(record) = clean(&assessment(), ASSESSMENT_PATH) else {
        panic!("an assessment");
    };
    assert_eq!(record.header.id, "assessment:reconcile-explanation");
    assert_eq!(record.subject.capability, "cap:workplan-reconcile");
    assert_eq!(record.subject.section, CapabilitySection::Explanation);
    assert_eq!(record.subject.text(), "cap:workplan-reconcile#explanation");
    let kinds: Vec<InputKind> = record.inputs.iter().map(|input| input.kind).collect();
    assert_eq!(kinds, InputKind::ALL);
    assert_eq!(record.verdict, AssessmentVerdict::Accepted);
    assert_eq!(record.by, Actor::Human);
    let rejected = mutate(&assessment(), "verdict: accepted", "verdict: rejected");
    let RafDocument::Assessment(record) = clean(&rejected, ASSESSMENT_PATH) else {
        panic!("an assessment");
    };
    assert_eq!(record.verdict, AssessmentVerdict::Rejected);
}

#[test]
fn a_valid_coverage_record_validates_and_an_empty_scope_opts_out_visibly() {
    let RafDocument::Coverage(record) = clean(&coverage(), COVERAGE_PATH) else {
        panic!("a coverage record");
    };
    assert_eq!(record.header.id, "coverage:default");
    assert_eq!(record.scope, ["raf-workplan", "cargo-workplan"]);
    assert_eq!(
        record.exclusions[0].item,
        ExclusionItem::Prefix("raf_workplan::exec::tests".to_string())
    );
    assert_eq!(
        record.exclusions[0].reason,
        "Unit tests of the process rows."
    );
    assert_eq!(
        record.exclusions[1].item,
        ExclusionItem::Item("raf_workplan::Workplan::empty".to_string())
    );
    let empty = "---\ntype: Coverage\nid: coverage:default\nscope: []\nexclusions: []\n---\n";
    let RafDocument::Coverage(record) = clean(empty, COVERAGE_PATH) else {
        panic!("a coverage record");
    };
    assert!(record.scope.is_empty() && record.exclusions.is_empty());
}

#[test]
fn a_verdict_is_one_of_the_verdicts_of_its_type() {
    for (text, path, doc_type, from, found) in [
        (
            evidence(),
            EVIDENCE_PATH,
            DocType::Evidence,
            "verdict: pass",
            "accepted",
        ),
        (
            evidence(),
            EVIDENCE_PATH,
            DocType::Evidence,
            "verdict: pass",
            "PASS",
        ),
        (
            assessment(),
            ASSESSMENT_PATH,
            DocType::Assessment,
            "verdict: accepted",
            "pass",
        ),
        (
            assessment(),
            ASSESSMENT_PATH,
            DocType::Assessment,
            "verdict: accepted",
            "maybe",
        ),
    ] {
        let broken = mutate(&text, from, &format!("verdict: {found}"));
        let rule = Rule::BadVerdict {
            doc_type,
            found: found.to_string(),
        };
        assert_only(&broken, path, &key("verdict"), &rule);
        clean(&text, path);
    }
}

#[test]
fn by_is_human_or_agent_and_never_a_name() {
    for found in ["Ada Lovelace", "Human", "bot", "someone@example.invalid"] {
        let broken = mutate(&assessment(), "by: human", &format!("by: {found}"));
        let rule = Rule::BadActor {
            found: found.to_string(),
        };
        assert_only(&broken, ASSESSMENT_PATH, &key("by"), &rule);
        let message = rule.to_string();
        assert!(message.contains("never the name of a person"), "{message}");
    }
    clean(
        &mutate(&assessment(), "by: human", "by: agent"),
        ASSESSMENT_PATH,
    );
}

#[test]
fn a_digest_is_sixty_four_lowercase_hex_digits() {
    let upper = DIGEST.to_ascii_uppercase();
    let short = &DIGEST[..63];
    let long = format!("{DIGEST}0");
    let not_hex = DIGEST.replacen('9', "g", 1);
    for found in [upper.as_str(), short, long.as_str(), not_hex.as_str()] {
        let broken = mutate(&evidence(), DIGEST, found);
        let rule = Rule::BadDigest {
            found: found.to_string(),
        };
        assert_only(&broken, EVIDENCE_PATH, &key("inputs"), &rule);
    }
    clean(&evidence(), EVIDENCE_PATH);
}

#[test]
fn a_revision_is_a_full_commit_id() {
    let sha256 = format!("{COMMIT}{}", &COMMIT[..24]);
    clean(&mutate(&evidence(), COMMIT, &sha256), EVIDENCE_PATH);
    let upper = COMMIT.to_ascii_uppercase();
    for found in ["b499578", upper.as_str(), "main", "HEAD~1"] {
        let broken = mutate(&evidence(), COMMIT, found);
        let rule = Rule::BadRevision {
            found: found.to_string(),
        };
        assert_only(&broken, EVIDENCE_PATH, &key("revision"), &rule);
    }
}

#[test]
fn the_subject_takes_the_form_of_its_type() {
    for found in [
        "run",
        "binding:",
        "binding:solo",
        "binding:a::-b",
        "cap:x#contract",
    ] {
        let broken = mutate(
            &evidence(),
            "subject: command",
            &format!("subject: \"{found}\""),
        );
        let rule = Rule::BadSubject {
            doc_type: DocType::Evidence,
            found: found.to_string(),
        };
        assert_only(&broken, EVIDENCE_PATH, &key("subject"), &rule);
    }
    for found in [
        "cap:workplan-reconcile#nonsense",
        "cap:workplan-reconcile",
        "command",
    ] {
        let broken = mutate(
            &assessment(),
            "subject: cap:workplan-reconcile#explanation",
            &format!("subject: {found}"),
        );
        let rule = Rule::BadSubject {
            doc_type: DocType::Assessment,
            found: found.to_string(),
        };
        assert_only(&broken, ASSESSMENT_PATH, &key("subject"), &rule);
    }
}

#[test]
fn the_command_key_is_present_exactly_when_the_subject_is_command() {
    let without = mutate(&evidence(), "command: \"cargo bench parse\"\n", "");
    assert_only(
        &without,
        EVIDENCE_PATH,
        &key("subject"),
        &Rule::CommandSubject { has_command: false },
    );
    let bound = mutate(
        &evidence(),
        "subject: command",
        "subject: binding:raf_okf::test_crate::bounds::a_case",
    );
    assert_only(
        &bound,
        EVIDENCE_PATH,
        &key("subject"),
        &Rule::CommandSubject { has_command: true },
    );
    clean(
        &mutate(&bound, "command: \"cargo bench parse\"\n", ""),
        EVIDENCE_PATH,
    );
    let empty = mutate(&evidence(), "\"cargo bench parse\"", "\"\"");
    assert_only(&empty, EVIDENCE_PATH, &key("command"), &Rule::EmptyText);
}

#[test]
fn input_rows_have_exactly_kind_name_and_digest() {
    let extra = mutate(
        &evidence(),
        &format!("    digest: {DIGEST}\n"),
        &format!("    digest: {DIGEST}\n    note: x\n"),
    );
    assert_only(
        &extra,
        EVIDENCE_PATH,
        &key("inputs"),
        &Rule::RowUnknownKey {
            list: "inputs",
            key: "note".to_string(),
        },
    );
    let missing = mutate(&evidence(), &format!("    digest: {DIGEST}\n"), "");
    assert_only(
        &missing,
        EVIDENCE_PATH,
        &key("inputs"),
        &Rule::RowMissingKey {
            list: "inputs",
            key: "digest",
        },
    );
    let scalar = format!(
        "inputs:\n  - kind: symbol\n    name: raf_okf::Document::parse\n    digest: {DIGEST}\n"
    );
    let flat = mutate(&evidence(), &scalar, "inputs: [raf_okf::Document::parse]\n");
    assert_only(
        &flat,
        EVIDENCE_PATH,
        &key("inputs"),
        &Rule::WrongShape {
            expected: "a list of maps with `kind`, `name` and `digest`",
        },
    );
    let empty = mutate(&evidence(), &scalar, "inputs: []\n");
    assert_only(
        &empty,
        EVIDENCE_PATH,
        &key("inputs"),
        &Rule::EmptyList { list: "inputs" },
    );
    clean(&evidence(), EVIDENCE_PATH);
}

#[test]
fn an_input_kind_is_one_the_type_takes_and_its_name_fits_the_kind() {
    // Evidence is made against symbols only.
    let section = mutate(&evidence(), "kind: symbol", "kind: section").replace(
        "name: raf_okf::Document::parse",
        "name: cap:okf-profile#contract",
    );
    assert_only(
        &section,
        EVIDENCE_PATH,
        &key("inputs"),
        &Rule::BadInputKind {
            doc_type: DocType::Evidence,
            found: "section".to_string(),
        },
    );
    let unknown = mutate(&assessment(), "kind: record", "kind: file");
    assert_only(
        &unknown,
        ASSESSMENT_PATH,
        &key("inputs"),
        &Rule::BadInputKind {
            doc_type: DocType::Assessment,
            found: "file".to_string(),
        },
    );
    for (from, found, kind) in [
        (
            "name: cap:workplan-reconcile#explanation\n    digest",
            "cap:workplan-reconcile#nowhere",
            InputKind::Section,
        ),
        (
            "name: raf_workplan::Workplan::reconcile",
            "Workplan",
            InputKind::Symbol,
        ),
        (
            "name: evidence:memory-bound",
            "change:memory-bound",
            InputKind::Record,
        ),
    ] {
        let to = match from.split_once('\n') {
            Some((_, rest)) => format!("name: {found}\n{rest}"),
            None => format!("name: {found}"),
        };
        let broken = mutate(&assessment(), from, &to);
        let rule = Rule::BadInputName {
            kind,
            found: found.to_string(),
        };
        assert_only(&broken, ASSESSMENT_PATH, &key("inputs"), &rule);
    }
    clean(&assessment(), ASSESSMENT_PATH);
}

#[test]
fn an_assessment_holds_the_section_row_of_its_subject() {
    let other = mutate(
        &assessment(),
        "name: cap:workplan-reconcile#explanation",
        "name: cap:workplan-reconcile#usage",
    );
    assert_only(
        &other,
        ASSESSMENT_PATH,
        &key("inputs"),
        &Rule::SubjectInputMissing {
            subject: "cap:workplan-reconcile#explanation".to_string(),
        },
    );
    clean(&assessment(), ASSESSMENT_PATH);
}

#[test]
fn a_list_that_holds_each_entry_once_refuses_a_repeat() {
    let row =
        format!("  - kind: symbol\n    name: raf_okf::Document::parse\n    digest: {DIGEST}\n");
    let twice = mutate(&evidence(), &row, &format!("{row}{row}"));
    assert_only(
        &twice,
        EVIDENCE_PATH,
        &key("inputs"),
        &Rule::DuplicateEntry {
            list: "inputs",
            entry: "symbol raf_okf::Document::parse".to_string(),
        },
    );
    let covers = mutate(
        &evidence(),
        "for: [cap:okf-profile]",
        "for: [cap:okf-profile, cap:okf-profile]",
    );
    assert_only(
        &covers,
        EVIDENCE_PATH,
        &key("for"),
        &Rule::DuplicateEntry {
            list: "for",
            entry: "cap:okf-profile".to_string(),
        },
    );
    let scope = mutate(
        &coverage(),
        "[raf-workplan, cargo-workplan]",
        "[raf-workplan, raf-workplan]",
    );
    assert_only(
        &scope,
        COVERAGE_PATH,
        &key("scope"),
        &Rule::DuplicateEntry {
            list: "scope",
            entry: "raf-workplan".to_string(),
        },
    );
    let exclusions = mutate(
        &coverage(),
        "raf_workplan::Workplan::empty",
        "raf_workplan::exec::tests::*",
    );
    assert_only(
        &exclusions,
        COVERAGE_PATH,
        &key("exclusions"),
        &Rule::DuplicateEntry {
            list: "exclusions",
            entry: "raf_workplan::exec::tests::*".to_string(),
        },
    );
    clean(&evidence(), EVIDENCE_PATH);
    clean(&coverage(), COVERAGE_PATH);
}

#[test]
fn for_lists_at_least_one_capability_id() {
    let empty = mutate(&evidence(), "for: [cap:okf-profile]", "for: []");
    assert_only(
        &empty,
        EVIDENCE_PATH,
        &key("for"),
        &Rule::EmptyList { list: "for" },
    );
    let bad = mutate(&evidence(), "for: [cap:okf-profile]", "for: [okf-profile]");
    assert_only(
        &bad,
        EVIDENCE_PATH,
        &key("for"),
        &Rule::BadCapabilityId {
            entry: "okf-profile".to_string(),
        },
    );
}

#[test]
fn values_and_limits_are_numbers_and_each_limit_names_a_value() {
    for found in ["fast", "1e3", "4 1", "+1"] {
        let broken = mutate(
            &evidence(),
            "peak_mib: 41",
            &format!("peak_mib: \"{found}\""),
        );
        let rule = Rule::BadMeasure {
            list: "values",
            name: "peak_mib".to_string(),
            found: found.to_string(),
        };
        assert_only(&broken, EVIDENCE_PATH, &key("values"), &rule);
    }
    let shape = mutate(
        &evidence(),
        "values:\n  peak_mib: 41\n  seconds: 0.5\n",
        "values: [41]\n",
    );
    assert_only(
        &shape,
        EVIDENCE_PATH,
        &key("values"),
        &Rule::WrongShape {
            expected: "a map of names to numbers",
        },
    );
    let orphan = mutate(
        &evidence(),
        "limits:\n  peak_mib: 64\n",
        "limits:\n  rss_mib: 64\n",
    );
    assert_only(
        &orphan,
        EVIDENCE_PATH,
        &key("limits"),
        &Rule::LimitWithoutValue {
            name: "rss_mib".to_string(),
        },
    );
    // A limit with no values at all names nothing either.
    let bare = mutate(&evidence(), "values:\n  peak_mib: 41\n  seconds: 0.5\n", "");
    assert_only(
        &bare,
        EVIDENCE_PATH,
        &key("limits"),
        &Rule::LimitWithoutValue {
            name: "peak_mib".to_string(),
        },
    );
    // A failing execution, such as one killed at its time limit, may have
    // printed none of the values its limits name.
    clean(
        &mutate(&bare, "verdict: pass", "verdict: fail"),
        EVIDENCE_PATH,
    );
    clean(
        &mutate(&orphan, "verdict: pass", "verdict: fail"),
        EVIDENCE_PATH,
    );
    // A negative number and a value equal to its limit are accepted.
    let edge = mutate(&evidence(), "peak_mib: 41", "peak_mib: 64")
        .replace("seconds: 0.5", "seconds: -0.25");
    clean(&edge, EVIDENCE_PATH);
}

#[test]
fn a_passing_record_holds_no_value_above_its_limit() {
    let over = mutate(&evidence(), "peak_mib: 41", "peak_mib: 64.5");
    assert_only(
        &over,
        EVIDENCE_PATH,
        &key("verdict"),
        &Rule::VerdictOverLimit {
            name: "peak_mib".to_string(),
        },
    );
    clean(
        &mutate(&over, "verdict: pass", "verdict: fail"),
        EVIDENCE_PATH,
    );
}

/// The texts a floating-point comparison reads as equal or as infinite, each
/// over its limit, and the neighbours equal to it or just under it.
fn exact() -> [(&'static str, &'static str, &'static str); 4] {
    [
        ("9007199254740993", "9007199254740992", "9007199254740991"),
        ("0.30000000000000001", "0.3", "0.29999999999999999"),
        ("1.0000000000000000001", "1", "0.9999999999999999999"),
        // `1` and 399 zeros: just under the limit, and infinite as an `f64`
        // like both of the others.
        (TWO_E400, ONE_E400, &ONE_E400[..400]),
    ]
}

const TWO_E400: &str = concat!(
    "2",
    "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
);

const ONE_E400: &str = concat!(
    "1",
    "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
);

#[test]
fn a_value_over_its_limit_is_judged_on_its_text_and_not_its_nearest_float() {
    for (over, limit, under) in exact() {
        let with = |value: &str| {
            mutate(&evidence(), "peak_mib: 41", &format!("peak_mib: {value}")).replacen(
                "peak_mib: 64",
                &format!("peak_mib: {limit}"),
                1,
            )
        };
        assert_only(
            &with(over),
            EVIDENCE_PATH,
            &key("verdict"),
            &Rule::VerdictOverLimit {
                name: "peak_mib".to_string(),
            },
        );
        clean(&with(limit), EVIDENCE_PATH);
        clean(&with(under), EVIDENCE_PATH);
    }
}

#[test]
fn numbers_compare_exactly_by_sign_whole_part_and_fraction() {
    use std::cmp::Ordering::{Equal, Greater, Less};
    let n = |text: &str| raf_okf::Number::parse(text).expect("a number");
    for (left, right, order) in [
        ("0", "-0", Equal),
        ("-0.000", "0", Equal),
        ("007", "7.000", Equal),
        ("12", "9", Greater),
        ("-12", "-9", Less),
        ("-1", "0", Less),
        ("0.5", "0.45", Greater),
        ("0.05", "0.5", Less),
        ("1.10", "1.1", Equal),
        ("-0.30000000000000001", "-0.3", Less),
        (TWO_E400, ONE_E400, Greater),
    ] {
        assert_eq!(n(left).compare(&n(right)), order, "{left} against {right}");
        assert_eq!(
            n(right).compare(&n(left)),
            order.reverse(),
            "{right} against {left}"
        );
    }
    for (over, limit, under) in exact() {
        assert!(n(over).exceeds(&n(limit)), "{over} over {limit}");
        assert!(!n(limit).exceeds(&n(limit)));
        assert!(!n(under).exceeds(&n(limit)), "{under} under {limit}");
    }
}

#[test]
fn scope_holds_crate_package_names() {
    for entry in ["raf_okf::Body", "-raf", "1crate", "raf okf"] {
        let broken = mutate(
            &coverage(),
            "[raf-workplan, cargo-workplan]",
            &format!("[\"{entry}\"]"),
        );
        let rule = Rule::BadCrateName {
            entry: entry.to_string(),
        };
        assert_only(&broken, COVERAGE_PATH, &key("scope"), &rule);
    }
    clean(
        &mutate(
            &coverage(),
            "[raf-workplan, cargo-workplan]",
            "[raf_okf, Raf-2]",
        ),
        COVERAGE_PATH,
    );
}

#[test]
fn an_exclusion_names_an_item_or_a_module_prefix_below_a_crate_root() {
    for found in ["raf_workplan::*", "raf_workplan"] {
        let broken = mutate(&coverage(), "raf_workplan::Workplan::empty", found);
        let rule = Rule::CrateRootExclusion {
            found: found.to_string(),
        };
        assert_only(&broken, COVERAGE_PATH, &key("exclusions"), &rule);
    }
    for found in [
        "raf_workplan::*::x",
        "raf-workplan::Workplan",
        "raf_workplan::",
        "*",
    ] {
        let broken = mutate(
            &coverage(),
            "raf_workplan::Workplan::empty",
            &format!("\"{found}\""),
        );
        let rule = Rule::BadExclusionItem {
            found: found.to_string(),
        };
        assert_only(&broken, COVERAGE_PATH, &key("exclusions"), &rule);
    }
    // A module one level below the root is the shortest prefix.
    clean(
        &mutate(
            &coverage(),
            "raf_workplan::Workplan::empty",
            "raf_workplan::exec::*",
        ),
        COVERAGE_PATH,
    );
}

#[test]
fn an_exclusion_has_a_reason_with_no_placeholder() {
    let empty = mutate(&coverage(), "\"Built by the load only.\"", "\"  \"");
    assert_only(
        &empty,
        COVERAGE_PATH,
        &key("exclusions"),
        &Rule::EmptyReason {
            item: "raf_workplan::Workplan::empty".to_string(),
        },
    );
    let missing = mutate(&coverage(), "\n    reason: \"Built by the load only.\"", "");
    assert_only(
        &missing,
        COVERAGE_PATH,
        &key("exclusions"),
        &Rule::RowMissingKey {
            list: "exclusions",
            key: "reason",
        },
    );
    let token = concat!("t", "bd");
    let placeholder = mutate(
        &coverage(),
        "Built by the load only.",
        &format!("Reason {token}."),
    );
    assert_only(
        &placeholder,
        COVERAGE_PATH,
        &key("exclusions"),
        &Rule::Placeholder {
            token: token.to_string(),
        },
    );
    clean(&coverage(), COVERAGE_PATH);
}

#[test]
fn coverage_is_one_record_at_its_one_path() {
    // Another slug, in a file of that name.
    let other = mutate(&coverage(), "coverage:default", "coverage:workspace");
    assert_only(
        &other,
        "work/coverage/workspace.md",
        &key("id"),
        &Rule::SingletonId {
            doc_type: DocType::Coverage,
        },
    );
    let message = Rule::SingletonId {
        doc_type: DocType::Coverage,
    }
    .to_string();
    assert!(
        message.contains("`coverage:default` at work/coverage/default.md"),
        "{message}"
    );
    // The right id in a file of another name.
    assert_only(
        &coverage(),
        "work/coverage/workspace.md",
        &key("id"),
        &Rule::IdFileName {
            id: "coverage:default".to_string(),
            stem: "workspace".to_string(),
        },
    );
    clean(&coverage(), COVERAGE_PATH);
}

#[test]
fn a_record_sits_in_its_directory_and_its_id_matches_its_file() {
    for (text, path, doc_type) in [
        (
            evidence(),
            "work/changes/memory-bound.md",
            DocType::Evidence,
        ),
        (
            evidence(),
            "work/evidence/sub/memory-bound.md",
            DocType::Evidence,
        ),
        (
            assessment(),
            "work/evidence/reconcile-explanation.md",
            DocType::Assessment,
        ),
        (coverage(), "work/default.md", DocType::Coverage),
    ] {
        assert_only(
            &text,
            path,
            &key("type"),
            &Rule::WrongDirectory { declared: doc_type },
        );
    }
    let message = Rule::WrongDirectory {
        declared: DocType::Evidence,
    }
    .to_string();
    assert_eq!(
        message,
        "an Evidence document belongs directly in work/evidence"
    );
    assert_only(
        &evidence(),
        "work/evidence/other.md",
        &key("id"),
        &Rule::IdFileName {
            id: "evidence:memory-bound".to_string(),
            stem: "other".to_string(),
        },
    );
    let prefix = mutate(
        &evidence(),
        "id: evidence:memory-bound",
        "id: assessment:memory-bound",
    );
    assert_only(
        &prefix,
        EVIDENCE_PATH,
        &key("id"),
        &Rule::InvalidId {
            prefix: "evidence:",
        },
    );
}

#[test]
fn a_record_has_no_title_description_or_body() {
    let titled = mutate(
        &evidence(),
        "type: Evidence\n",
        "type: Evidence\ntitle: Memory\n",
    );
    assert_only(
        &titled,
        EVIDENCE_PATH,
        &key("title"),
        &Rule::UnknownKey {
            key: "title".to_string(),
        },
    );
    for (text, path, doc_type) in [
        (evidence(), EVIDENCE_PATH, DocType::Evidence),
        (assessment(), ASSESSMENT_PATH, DocType::Assessment),
        (coverage(), COVERAGE_PATH, DocType::Coverage),
    ] {
        let prose = format!("{text}\nThe bound held comfortably.\n");
        assert_only(
            &prose,
            path,
            &Location::Document,
            &Rule::BodyNotEmpty { doc_type },
        );
        // Blank lines and an HTML comment are no body.
        clean(&format!("{text}\n<!-- written by the tool -->\n\n"), path);
    }
}

#[test]
fn a_record_type_is_named_in_the_unknown_type_message() {
    let rule = Rule::UnknownType {
        found: "Note".to_string(),
    };
    assert_eq!(
        rule.to_string(),
        "bad type `Note`; the types are Capability, Change, Architecture, Evidence, Assessment and Coverage"
    );
}
