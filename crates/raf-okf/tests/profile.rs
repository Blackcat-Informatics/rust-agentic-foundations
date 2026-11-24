// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The profile validator: one fixture per rule, each beside a neighbouring
//! document that is accepted.

use std::fmt::Write as _;

use raf_okf::{
    CapabilitySection, ContractEffect, DocType, Document, LinkForm, Location, Outcome, Profile,
    RafDocument, ReviewKind, Rule, State, TimestampError, Violation,
};

const SECTIONS: [&str; 7] = [
    "Purpose",
    "Contract",
    "Usage",
    "Examples",
    "Constraints",
    "Explanation",
    "Verification",
];

/// A capability with seven sections. `extra` goes first in `Purpose`.
fn capability(slug: &str, state: &str, depends: &str, extra: &str) -> String {
    let (code, tests) = if state == "delivered" {
        ("[src/lib.rs]", "[tests/it.rs]")
    } else {
        ("[]", "[]")
    };
    let sections: Vec<String> = SECTIONS
        .iter()
        .map(|name| {
            let first = if *name == "Purpose" { extra } else { "" };
            format!("## {name}\n\n{first}Text for {name}.\n")
        })
        .collect();
    format!(
        "---\n# SPDX-FileCopyrightText: 2026 Example\n# SPDX-License-Identifier: CC-BY-4.0\n\
         type: Capability\nid: cap:{slug}\ntitle: \"Title of {slug}\"\n\
         description: A description of {slug}.\nstate: {state}\n\
         bindings:\n  code: {code}\n  tests: {tests}\n  examples: []\n\
         depends: {depends}\n---\n\n# {slug}\n\n{}",
        sections.join("\n")
    )
}

const ARCHITECTURE: &str = "---\ntype: Architecture\nid: arch:overview\ntitle: Overview\n\
     description: How it fits.\n---\n\n# Overview\n\nBody text.\n";

const CHANGE: &str = "---\ntype: Change\nid: change:add-alpha\ntitle: Add alpha\n\
     description: Adds alpha and beta.\n\
     changes: [cap:alpha, cap:beta]\naffects:\n  - cap:alpha#contract\n  - cap:beta#purpose\n\
     contract_effect: changes\nitems:\n  - claim: \"alpha reads input\"\n    outcome: implemented\n  \
     - claim: \"beta is sketched\"\n---\n\n## Summary\n\nAdds both.\n";

const ALPHA_PATH: &str = "docs/capabilities/alpha.md";
const OVERVIEW_PATH: &str = "docs/architecture/overview.md";
const CHANGE_PATH: &str = "work/changes/add-alpha.md";
const REVIEW_PATH: &str = "work/reviews/ledger-design.md";
const REVIEW: &str = include_str!("fixtures/review.md");
const REVIEW_FINDING_SHAPE: Rule = Rule::WrongShape {
    expected: "closed finding rows with id, severity, location and status; locations use item:path or file:path#Lfirst-Llast with positive ordered lines; fixed requires revision, accepted and false-positive require reason",
};

fn alpha() -> String {
    capability("alpha", "designed", "[cap:beta]", "")
}

#[expect(
    clippy::panic,
    reason = "the assertion reports the unexpected outcome and terminates the test"
)]
fn parse(text: &str) -> Document {
    Document::parse(text).unwrap_or_else(|error| panic!("{error} in {text:?}"))
}

fn validate(text: &str, path: &str) -> Result<RafDocument, Vec<Violation>> {
    Profile.validate(&parse(text), path)
}

#[expect(
    clippy::panic,
    reason = "the assertion reports the unexpected outcome and terminates the test"
)]
fn violations(text: &str, path: &str) -> Vec<Violation> {
    match validate(text, path) {
        Ok(typed) => panic!("expected violations for {path}, got {typed:?}"),
        Err(found) => found,
    }
}

#[expect(
    clippy::panic,
    reason = "the assertion reports the unexpected outcome and terminates the test"
)]
fn clean(text: &str, path: &str) -> RafDocument {
    match validate(text, path) {
        Ok(typed) => typed,
        Err(found) => panic!("expected {path} to be clean, got {found:#?}"),
    }
}

fn key(name: &str) -> Location {
    Location::Key(name.to_string())
}

fn section(name: &str) -> Location {
    Location::Section(name.to_string())
}

/// Require a violation at `at` with exactly `rule`.
fn assert_hit(found: &[Violation], at: &Location, rule: &Rule) {
    assert!(
        found.iter().any(|v| v.at() == at && v.rule() == rule),
        "expected {rule:?} at {at:?}, got {found:#?}"
    );
}

/// Replace the first `from` in `text` with `to`, requiring that it occurs.
fn mutate(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "fixture has no `{from}`");
    text.replacen(from, to, 1)
}

#[expect(
    clippy::expect_used,
    reason = "fixture prerequisites and required observations carry their assertion messages"
)]
fn review_with_findings(findings: &str) -> String {
    let (head, _) = REVIEW
        .split_once("findings:\n")
        .expect("Review fixture findings");
    format!("{head}{findings}---\n")
}

#[test]
#[expect(
    clippy::panic,
    reason = "the assertion reports the unexpected outcome and terminates the test"
)]
fn review_finding_status_selects_the_required_and_allowed_keys() {
    // Every invalid row gets the same single profile finding, even when
    // several fields fail and the next authored occurrence is identical.
    let unsorted = mutate(REVIEW, "  - id: accepted\n", "  - id: z-accepted\n");
    let RafDocument::Review(review) = clean(&unsorted, REVIEW_PATH) else {
        panic!("expected Review fixture");
    };
    assert_eq!(
        review
            .findings
            .iter()
            .map(|finding| finding.id.as_str())
            .collect::<Vec<_>>(),
        ["fixed", "open", "spurious", "z-accepted"],
    );
    let common = "  - id: issue\n    severity: low\n    location: item:demo::read\n";
    for disposition in [
        "    status: open\n    reason: Unresolved.\n    note: Extra.\n",
        "    status: fixed\n",
        "    status: accepted\n    revision: abcdef0123456789abcdef0123456789abcdef01\n",
        "    status: false-positive\n",
        "    status: [open]\n",
        "    status: closed\n",
        "    status: fixed\n    revision: short\n",
    ] {
        let text = review_with_findings(&format!(
            "findings:\n{common}{disposition}{common}{disposition}"
        ));
        let found = violations(&text, REVIEW_PATH);
        assert_eq!(found.len(), 2, "{disposition}: {found:#?}");
        for finding in &found {
            assert_eq!(
                (finding.at(), finding.rule()),
                (&key("findings"), &REVIEW_FINDING_SHAPE),
                "{disposition}"
            );
        }
    }
}

#[test]
fn review_finding_lists_refuse_wrong_shapes_and_count_only_valid_ids() {
    // Reject malformed lists before any row checks. Only valid rows enter
    // duplicate detection, whose finding follows the authored row findings.
    let text = review_with_findings("findings: [plain]\n");
    let found = violations(&text, REVIEW_PATH);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(
        (found[0].at(), found[0].rule()),
        (
            &key("findings"),
            &Rule::WrongShape {
                expected: "a list of structured review findings"
            }
        )
    );

    let valid =
        "  - id: repeated\n    severity: low\n    location: item:demo::read\n    status: open\n";
    let invalid = format!("{valid}    reason: Unresolved.\n");
    for (rows, expected) in [
        (format!("{valid}{invalid}"), vec![REVIEW_FINDING_SHAPE]),
        (
            format!("{valid}{invalid}{valid}"),
            vec![
                REVIEW_FINDING_SHAPE,
                Rule::WrongShape {
                    expected: "findings with distinct kebab-case identifiers",
                },
            ],
        ),
    ] {
        let text = review_with_findings(&format!("findings:\n{rows}"));
        let found = violations(&text, REVIEW_PATH);
        assert_eq!(
            found.iter().map(Violation::rule).collect::<Vec<_>>(),
            expected.iter().collect::<Vec<_>>()
        );
        assert!(found.iter().all(|finding| finding.at() == &key("findings")));
    }
}

/// The two optional capability flags read `true` or `false`, and a neighbouring
/// invalid shape is refused at its own key.
#[expect(
    clippy::panic,
    reason = "the assertion reports the unexpected outcome and terminates the test"
)]
fn capability_flags_read_true_or_false_and_refuse_other_shapes() {
    for flag in ["measured_evidence_required", "review_strict"] {
        for (value, expected) in [
            ("true", Some(true)),
            ("false", Some(false)),
            ("\"true\"", Some(true)),
            ("TRUE", None),
            ("yes", None),
            ("[true]", None),
        ] {
            let text = alpha().replace("depends:", &format!("{flag}: {value}\ndepends:"));
            let Some(expected) = expected else {
                let found = violations(&text, ALPHA_PATH);
                assert_eq!(found.len(), 1, "{found:#?}");
                assert_hit(
                    &found,
                    &key(flag),
                    &Rule::WrongShape {
                        expected: "true or false",
                    },
                );
                continue;
            };
            let RafDocument::Capability(cap) = clean(&text, ALPHA_PATH) else {
                panic!("a capability")
            };
            let read = if flag == "review_strict" {
                cap.review_strict
            } else {
                cap.measured_evidence_required
            };
            assert_eq!(read, expected, "{flag}: {value}");
        }
    }
}

#[test]
#[expect(
    clippy::panic,
    reason = "the assertion reports the unexpected outcome and terminates the test"
)]
fn valid_documents_of_each_type_validate_to_typed_values() {
    review_location_neighbors();
    review_kind_neighbors();
    let RafDocument::Capability(cap) = clean(&alpha(), ALPHA_PATH) else {
        panic!("a capability");
    };
    assert_eq!(cap.header.id, "cap:alpha");
    assert_eq!(cap.header.slug, "alpha");
    assert_eq!(cap.header.title, "Title of alpha");
    assert_eq!(cap.header.tags, None);
    assert_eq!(cap.header.timestamp, None);
    assert_eq!(cap.state, State::Designed);
    let none: Vec<String> = Vec::new();
    assert_eq!(
        (
            &cap.bindings.code,
            &cap.bindings.tests,
            &cap.bindings.examples
        ),
        (&none, &none, &none)
    );
    assert_eq!(cap.depends, ["cap:beta"]);
    assert_eq!(cap.review_required, []);
    for (list, expected) in [
        ("[]", Some(Vec::new())),
        (
            "[security, correctness, design, performance, documentation]",
            Some(ReviewKind::ALL.to_vec()),
        ),
        ("[security, security]", None),
        ("[unknown]", None),
        ("[security, unknown, bogus]", None),
        ("security", None),
    ] {
        let text = alpha().replace("depends:", &format!("review_required: {list}\ndepends:"));
        match expected {
            Some(expected) => {
                let RafDocument::Capability(cap) = clean(&text, ALPHA_PATH) else {
                    panic!("a capability")
                };
                assert_eq!(cap.review_required, expected);
            }
            // Every refused member is one finding at the key.
            None => assert_eq!(
                violations(&text, ALPHA_PATH)
                    .iter()
                    .filter(|v| v.at() == &key("review_required"))
                    .count(),
                1,
                "{list}"
            ),
        }
    }
    // A list of mappings is one refused value at the key, not one finding per row.
    let mappings = alpha().replace("depends:", "review_required:\n  - a: b\n  - c: d\ndepends:");
    assert_eq!(
        violations(&mappings, ALPHA_PATH)
            .iter()
            .filter(|v| v.at() == &key("review_required"))
            .count(),
        1
    );

    assert!(!cap.measured_evidence_required);
    assert!(!cap.review_strict);
    capability_flags_read_true_or_false_and_refuse_other_shapes();

    let RafDocument::Change(change) = clean(CHANGE, CHANGE_PATH) else {
        panic!("a change");
    };
    assert_eq!(change.changes, ["cap:alpha", "cap:beta"]);
    let affects: Vec<(&str, CapabilitySection)> = change
        .affects
        .iter()
        .map(|a| (a.capability.as_str(), a.section))
        .collect();
    assert_eq!(
        affects,
        [
            ("cap:alpha", CapabilitySection::Contract),
            ("cap:beta", CapabilitySection::Purpose)
        ]
    );
    assert_eq!(change.contract_effect, ContractEffect::Changes);
    assert_eq!(change.items.len(), 2);
    assert_eq!(change.items[0].claim, "alpha reads input");
    assert_eq!(change.items[0].outcome, Some(Outcome::Implemented));
    assert_eq!(change.items[1].outcome, None);

    let RafDocument::Architecture(page) = clean(ARCHITECTURE, OVERVIEW_PATH) else {
        panic!("an architecture page");
    };
    assert_eq!(page.header.description, "How it fits.");
    architecture_model_neighbors();
}

#[test]
#[expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "the valid Architecture fixture must round-trip and recover the same typed decision"
)]
fn a_decision_relation_round_trips_the_native_authoring_carrier() {
    // The carrier preserves the authored mapping, and the profile recovers its typed section pair.
    let text = ARCHITECTURE.replacen(
        "---\n\n# Overview",
        "supersedes:\n  order: ['arch:earlier#layout', 'arch:design#inputs']\n  limits: ['arch:design#limits']\n---\n\n# Overview\n\n## Decision\n\n### Order\n\n### Limits\n",
        1,
    );
    let document = parse(&text);
    let dataset = document.to_dataset(OVERVIEW_PATH).unwrap();
    let restored = Document::from_dataset(&dataset, OVERVIEW_PATH).unwrap();
    assert_eq!(restored, document);
    let original = clean(&text, OVERVIEW_PATH);
    let round_trip = clean(&restored.render(), OVERVIEW_PATH);
    assert_eq!(round_trip, original);
    let RafDocument::Architecture(page) = round_trip else {
        panic!("an architecture page");
    };
    let [limits, order] = page.supersedes.as_slice() else {
        panic!("two section replacements");
    };
    assert_eq!(limits.section(), "limits");
    assert_eq!(order.section(), "order");
    assert_eq!(
        order
            .targets()
            .iter()
            .map(|target| (target.architecture(), target.section()))
            .collect::<Vec<_>>(),
        [("arch:earlier", "layout"), ("arch:design", "inputs")]
    );
}

#[expect(
    clippy::panic,
    reason = "the assertion reports the unexpected outcome and terminates the test"
)]
fn review_kind_neighbors() {
    // The scalar Review field admits decoded spellings, including quoted ones.
    let quoted = mutate(REVIEW, "kind: design", "kind: \"design\"");
    let RafDocument::Review(review) = clean(&quoted, REVIEW_PATH) else {
        panic!("a review");
    };
    assert_eq!(review.kind, ReviewKind::Design);
    for kind in ["Design", "[design]"] {
        let text = mutate(REVIEW, "kind: design", &format!("kind: {kind}"));
        let found = violations(&text, REVIEW_PATH);
        assert_eq!(found.len(), 1, "{found:#?}");
        assert_hit(
            &found,
            &key("kind"),
            &Rule::WrongShape {
                expected: "security, correctness, design, performance or documentation",
            },
        );
    }
}

#[expect(
    clippy::panic,
    reason = "the assertion reports the unexpected outcome and terminates the test"
)]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn architecture_model_neighbors() {
    let model = "model:
  components:
    - id: service
      label: Service
      kind: component
      responsibility: Accept a submitted request
      bindings:
        code: [demo::service]
  actors:
    - id: caller
      label: Caller
      responsibility: Submit a request
  relations:
    - id: submit
      from: caller
      to: service
      label: Submit
      protocol: HTTP
      claim: declared
  artifacts:
    - id: receipt
      label: Receipt
      type: Receipt
      producers: [service]
      consumers: [caller]
  scenarios:
    - id: request
      label: Handle a request
      participants: [caller, service]
      steps:
        - id: accept
          relation: submit
          label: Accept the submitted request
          order: 0
";
    let text = ARCHITECTURE.replacen("---\n\n# Overview", &format!("{model}---\n\n# Overview"), 1);
    let document = parse(&text);
    let dataset = document.to_dataset(OVERVIEW_PATH).unwrap();
    let restored = Document::from_dataset(&dataset, OVERVIEW_PATH).unwrap();
    assert_eq!(
        restored, document,
        "model declarations retain the native authoring carrier"
    );
    let RafDocument::Architecture(page) = clean(&restored.render(), OVERVIEW_PATH) else {
        panic!("an architecture page");
    };
    assert_eq!(page.model.components[0].bindings.code, ["demo::service"]);
    assert_eq!(page.model.scenarios[0].steps[0].relation, "submit");
    for (from, to, location) in [
        ("model:\n", "model:\n  extra: unknown\n", "model.extra"),
        (
            "kind: component",
            "kind: service",
            "model.components[0].kind",
        ),
        (
            "code: [demo::service]",
            "code: []",
            "model.components[0].bindings",
        ),
        (
            "claim: declared",
            "claim: observed",
            "model.relations[0].claim",
        ),
        ("order: 0", "order: -1", "model.scenarios[0].steps[0].order"),
        (
            "participants: [caller, service]",
            "participants: []",
            "model.scenarios[0].participants",
        ),
        (
            "producers: [service]",
            "producers: []",
            "model.artifacts[0].producers",
        ),
        (
            "consumers: [caller]",
            "consumers: []",
            "model.artifacts[0].consumers",
        ),
    ] {
        let invalid = text.replacen(from, to, 1);
        assert!(
            violations(&invalid, OVERVIEW_PATH)
                .iter()
                .any(|found| found.at() == &key(location)),
            "{location}"
        );
    }
    for (length, accepted) in [(4096, true), (4097, false)] {
        let changed = text.replace("label: Service", &format!("label: {}", "x".repeat(length)));
        assert_eq!(validate(&changed, OVERVIEW_PATH).is_ok(), accepted);
    }
    for (count, accepted) in [(256, true), (257, false)] {
        let changed = architecture_with_actors(count);
        assert_eq!(validate(&changed, OVERVIEW_PATH).is_ok(), accepted);
    }
}

#[expect(
    clippy::expect_used,
    reason = "fixture prerequisites and required observations carry their assertion messages"
)]
fn architecture_with_actors(count: usize) -> String {
    let mut actors = String::new();
    for index in 0..count {
        writeln!(
            actors,
            "    - id: actor-{index}\n      label: Actor\n      responsibility: Supply input"
        )
        .expect("writing actors into a String succeeds");
    }
    ARCHITECTURE.replacen(
        "---\n\n# Overview",
        &format!("model:\n  actors:\n{actors}---\n\n# Overview"),
        1,
    )
}

fn review_location_neighbors() {
    let text = include_str!("fixtures/review.md");
    let path = "work/reviews/ledger-design.md";
    assert!(matches!(clean(text, path), RafDocument::Review(_)));
    let invalid = text.replace("#L1-L3", "#L3-L1");
    let found = violations(&invalid, path);
    assert!(
        found
            .iter()
            .any(|violation| violation.at() == &key("findings"))
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "the assertion reports the unexpected outcome and terminates the test"
)]
fn a_delivered_capability_with_code_and_tests_is_typed_with_its_bindings() {
    let text = capability("gamma", "delivered", "[cap:beta, cap:alpha]", "");
    let RafDocument::Capability(cap) = clean(&text, "docs/capabilities/gamma.md") else {
        panic!("a capability");
    };
    assert_eq!(cap.state, State::Delivered);
    assert_eq!(cap.bindings.code, ["src/lib.rs"]);
    assert_eq!(cap.bindings.tests, ["tests/it.rs"]);
    assert_eq!(cap.depends, ["cap:beta", "cap:alpha"]);
}

#[test]
fn an_unknown_key_is_reported_at_that_key() {
    let text = mutate(ARCHITECTURE, "title: Overview", "title: Overview\nextra: 1");
    let found = violations(&text, OVERVIEW_PATH);
    assert_eq!(found.len(), 1);
    assert_hit(
        &found,
        &key("extra"),
        &Rule::UnknownKey {
            key: "extra".into(),
        },
    );
    assert_eq!(found[0].file(), OVERVIEW_PATH);
}

/// The frontmatter block of each key of a type, in table order.
fn parts(doc_type: DocType) -> Vec<(&'static str, &'static str)> {
    match doc_type {
        DocType::Capability => vec![
            ("type", "type: Capability\n"),
            ("id", "id: cap:alpha\n"),
            ("title", "title: Alpha\n"),
            ("description", "description: Does alpha.\n"),
            ("state", "state: designed\n"),
            (
                "bindings",
                "bindings:\n  code: []\n  tests: []\n  examples: []\n",
            ),
            ("depends", "depends: []\n"),
            ("review_required", "review_required: []\n"),
            ("review_strict", "review_strict: true\n"),
            (
                "measured_evidence_required",
                "measured_evidence_required: true\n",
            ),
            ("diagrams", "diagrams: []\n"),
            ("model", "model:\n  actors: []\n"),
        ],
        DocType::Change => vec![
            ("type", "type: Change\n"),
            ("id", "id: change:alpha\n"),
            ("title", "title: Alpha\n"),
            ("description", "description: Does alpha.\n"),
            ("changes", "changes: [cap:alpha]\n"),
            ("affects", "affects:\n  - cap:alpha#contract\n"),
            ("contract_effect", "contract_effect: none\n"),
            ("items", "items: []\n"),
        ],
        DocType::Architecture => vec![
            ("type", "type: Architecture\n"),
            ("id", "id: arch:alpha\n"),
            ("title", "title: Alpha\n"),
            ("description", "description: Does alpha.\n"),
            ("diagrams", "diagrams: []\n"),
            ("model", "model:\n  actors: []\n"),
            (
                "supersedes",
                "supersedes:\n  order: ['arch:earlier#layout']\n",
            ),
        ],
        DocType::Evidence => vec![
            ("type", "type: Evidence\n"),
            ("id", "id: evidence:alpha\n"),
            ("subject", "subject: command\n"),
            ("inputs", EVIDENCE_INPUTS),
            ("verdict", "verdict: pass\n"),
            ("by", "by: agent\n"),
            ("for", "for: [cap:alpha]\n"),
            ("command", "command: \"cargo bench parse\"\n"),
            ("revision", REVISION),
            ("values", "values:\n  peak_mib: 41\n"),
            ("limits", "limits:\n  peak_mib: 64\n"),
            ("units", "units:\n  peak_mib: MiB\n"),
            ("conditions", "conditions: \"the reference host\"\n"),
        ],
        DocType::Assessment => vec![
            ("type", "type: Assessment\n"),
            ("id", "id: assessment:alpha\n"),
            ("subject", "subject: cap:alpha#contract\n"),
            ("inputs", ASSESSMENT_INPUTS),
            ("verdict", "verdict: accepted\n"),
            ("by", "by: human\n"),
        ],
        DocType::Coverage => vec![
            ("type", "type: Coverage\n"),
            ("id", "id: coverage:default\n"),
            ("scope", "scope: [raf-workplan, cargo-raf-workplan]\n"),
            (
                "exclusions",
                "exclusions:\n  - item: raf_workplan::exec::tests::*\n    reason: \"Unit tests that guard internals.\"\n",
            ),
        ],
        DocType::Debt => vec![
            ("type", "type: Debt\n"),
            ("id", "id: debt:default\n"),
            (
                "findings",
                "findings:\n  - kind: strict-claim-uncovered\n    subject: \"cap:alpha#contract/opening\"\n",
            ),
            ("lines", "lines:\n  - item: demo::real\n    orphans: 4\n"),
        ],
        DocType::Review => vec![
            ("type", "type: Review\n"),
            ("id", "id: review:alpha\n"),
            ("timestamp", "timestamp: 2030-01-01T00:00:00Z\n"),
            ("scope", "scope: [crate:demo]\n"),
            ("revision", REVISION),
            ("kind", "kind: correctness\n"),
            ("inputs", EVIDENCE_INPUTS),
            ("verdict", "verdict: clean\n"),
            ("by", "by: human\n"),
            ("findings", "findings: []\n"),
            ("findings_ref", "findings_ref: docs/findings/alpha.md\n"),
        ],
    }
}

const EVIDENCE_INPUTS: &str = "inputs:\n  - kind: symbol\n    name: demo::real\n    digest: 9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08\n";

const ASSESSMENT_INPUTS: &str = "inputs:\n  - kind: section\n    name: cap:alpha#contract\n    digest: 9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08\n  - kind: symbol\n    name: demo::real\n    digest: 9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08\n  - kind: record\n    name: evidence:alpha\n    digest: 9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08\n";

const REVISION: &str = "revision: b499578a1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f\n";

fn body_for(doc_type: DocType) -> String {
    match doc_type {
        DocType::Capability => SECTIONS
            .iter()
            .map(|name| format!("## {name}\n\ntext\n"))
            .collect::<Vec<_>>()
            .join("\n"),
        DocType::Change => "## Summary\n\ntext\n".to_string(),
        DocType::Architecture => "# Alpha\n\ntext\n".to_string(),
        DocType::Evidence
        | DocType::Assessment
        | DocType::Coverage
        | DocType::Debt
        | DocType::Review => String::new(),
    }
}

fn document_of(doc_type: DocType, skip: Option<&str>, extra: &str) -> String {
    let front: String = parts(doc_type)
        .into_iter()
        .filter(|(name, _)| Some(*name) != skip)
        .map(|(_, block)| block)
        .collect();
    format!("---\n{front}{extra}---\n\n{}", body_for(doc_type))
}

fn path_of(doc_type: DocType) -> String {
    let slug = doc_type.singleton().map_or("alpha", |slug| slug);
    format!("{}/{slug}.md", doc_type.directory())
}

#[test]
fn every_required_key_of_every_type_is_reported_when_missing() {
    for doc_type in DocType::ALL {
        let path = path_of(doc_type);
        clean(&document_of(doc_type, None, ""), &path);
        for spec in doc_type.keys().iter().filter(|spec| spec.required) {
            if spec.key == "type" {
                continue;
            }
            let found = violations(&document_of(doc_type, Some(spec.key), ""), &path);
            // Without `changes`, the `affects` entries name a capability that
            // is not listed, which is a second violation.
            if spec.key != "changes" {
                assert_eq!(
                    found.len(),
                    1,
                    "{} {}: {found:#?}",
                    doc_type.name(),
                    spec.key
                );
            }
            assert_hit(
                &found,
                &key(spec.key),
                &Rule::MissingKey {
                    key: spec.key.to_string(),
                },
            );
        }
    }
}

/// The frontmatter blocks of keys that `doc_type` does not have but another
/// type does, as `(key, block)`. The keys come from the key tables of the
/// other types; the blocks are the valid values from `parts`.
fn foreign_keys(doc_type: DocType) -> Vec<(&'static str, &'static str)> {
    let mut found: Vec<(&'static str, &'static str)> = Vec::new();
    for other in DocType::ALL.into_iter().filter(|other| *other != doc_type) {
        for spec in other.keys() {
            let own = doc_type.keys().iter().any(|mine| mine.key == spec.key);
            if own || found.iter().any(|(name, _)| *name == spec.key) {
                continue;
            }
            if let Some(block) = parts(other)
                .into_iter()
                .find_map(|(name, block)| (name == spec.key).then_some(block))
            {
                found.push((spec.key, block));
            } else {
                // Only the optional keys shared by every type have no block.
                assert!(!spec.required, "{} has no fixture block", spec.key);
            }
        }
    }
    found
}

#[test]
fn an_unknown_or_foreign_key_is_reported_on_every_type() {
    let unknown = |name: &str| Rule::UnknownKey {
        key: name.to_string(),
    };
    let mut foreign_seen = 0;
    for doc_type in DocType::ALL {
        let path = path_of(doc_type);
        clean(&document_of(doc_type, None, ""), &path);

        let found = violations(&document_of(doc_type, None, "bogus: 1\n"), &path);
        assert_eq!(found.len(), 1, "{}: {found:#?}", doc_type.name());
        assert_hit(&found, &key("bogus"), &unknown("bogus"));

        // The foreign keys derive from the tables, so they follow the tables;
        // this list is the architecture page's own assignment, and it is what
        // fails when a table of this type is widened.
        let names: Vec<&str> = foreign_keys(doc_type).iter().map(|(n, _)| *n).collect();
        let expected = expected_foreign_keys(doc_type);
        assert_eq!(names, expected, "{}", doc_type.name());

        // A key that is valid on another type, with a value valid there.
        for (name, block) in foreign_keys(doc_type) {
            foreign_seen += 1;
            let found = violations(&document_of(doc_type, None, block), &path);
            assert_eq!(found.len(), 1, "{} {name}: {found:#?}", doc_type.name());
            assert_hit(&found, &key(name), &unknown(name));
        }
    }
    // The fixed assignment includes optional capability policy and review
    // references. Shared scope, revision and findings keys occur only once;
    // Review rejects the document keys and the remaining record-only keys.
    assert_eq!(foreign_seen, 22 + 26 + 27 + 21 + 28 + 30 + 30 + 24);
}

/// Key findings precede value findings, which retain schema order.
#[test]
fn mixed_header_violations_keep_their_key_and_diagnostic_order() {
    let baseline = alpha();
    clean(&baseline, ALPHA_PATH);
    let text = mutate(
        &baseline,
        "title: \"Title of alpha\"\ndescription: A description of alpha.\nstate: designed\n",
        "zeta: 1\nresource: forbidden\naardvark: 2\nstate: done\n\
         measured_evidence_required: TRUE\nreview_strict: TRUE\n",
    );
    let found = violations(&text, ALPHA_PATH);
    let expected = [
        ("zeta", Rule::UnknownKey { key: "zeta".into() }),
        (
            "resource",
            Rule::ResourceKey {
                doc_type: DocType::Capability,
            },
        ),
        (
            "aardvark",
            Rule::UnknownKey {
                key: "aardvark".into(),
            },
        ),
        (
            "title",
            Rule::MissingKey {
                key: "title".into(),
            },
        ),
        (
            "description",
            Rule::MissingKey {
                key: "description".into(),
            },
        ),
        (
            "state",
            Rule::BadState {
                found: "done".into(),
            },
        ),
        (
            "review_strict",
            Rule::WrongShape {
                expected: "true or false",
            },
        ),
        (
            "measured_evidence_required",
            Rule::WrongShape {
                expected: "true or false",
            },
        ),
    ];
    assert_eq!(found.len(), expected.len());
    for (violation, (name, rule)) in found.iter().zip(expected) {
        assert_eq!(violation.file(), ALPHA_PATH);
        assert_eq!(violation.at(), &key(name));
        assert_eq!(violation.rule(), &rule);
    }
}

fn expected_foreign_keys(doc_type: DocType) -> Vec<&'static str> {
    match doc_type {
        DocType::Capability => [
            &CHANGE_KEYS[..],
            &["supersedes"],
            &EVIDENCE_KEYS,
            &COVERAGE_KEYS,
            &DEBT_KEYS,
            &REVIEW_KEYS,
        ]
        .concat(),
        DocType::Change => [
            &CAPABILITY_KEYS[..],
            &["diagrams", "model", "supersedes"],
            &EVIDENCE_KEYS,
            &COVERAGE_KEYS,
            &DEBT_KEYS,
            &REVIEW_KEYS,
        ]
        .concat(),
        DocType::Architecture => [
            &CAPABILITY_KEYS[..],
            &CHANGE_KEYS,
            &EVIDENCE_KEYS,
            &COVERAGE_KEYS,
            &DEBT_KEYS,
            &REVIEW_KEYS,
        ]
        .concat(),
        DocType::Evidence => [
            &HEADER_KEYS[..],
            &CAPABILITY_KEYS,
            &["diagrams", "model"],
            &CHANGE_KEYS,
            &["supersedes"],
            &COVERAGE_KEYS,
            &DEBT_KEYS,
            &REVIEW_KEYS,
        ]
        .concat(),
        DocType::Assessment => [
            &HEADER_KEYS[..],
            &CAPABILITY_KEYS,
            &["diagrams", "model"],
            &CHANGE_KEYS,
            &["supersedes"],
            &EVIDENCE_KEYS[4..],
            &COVERAGE_KEYS,
            &DEBT_KEYS,
            &REVIEW_KEYS,
        ]
        .concat(),
        DocType::Coverage => [
            &HEADER_KEYS[..],
            &CAPABILITY_KEYS,
            &["diagrams", "model"],
            &CHANGE_KEYS,
            &["supersedes"],
            &EVIDENCE_KEYS,
            &DEBT_KEYS,
            &REVIEW_KEYS,
        ]
        .concat(),
        DocType::Debt => [
            &HEADER_KEYS[..],
            &CAPABILITY_KEYS,
            &["diagrams", "model"],
            &CHANGE_KEYS,
            &["supersedes"],
            &EVIDENCE_KEYS,
            &COVERAGE_KEYS,
            &REVIEW_KEYS,
        ]
        .concat(),
        DocType::Review => [
            &HEADER_KEYS[..],
            &CAPABILITY_KEYS,
            &["diagrams", "model"],
            &CHANGE_KEYS,
            &["supersedes"],
            &EVIDENCE_KEYS[..1],
            &EVIDENCE_KEYS[4..6],
            &EVIDENCE_KEYS[7..],
            &COVERAGE_KEYS[1..],
            &DEBT_KEYS[1..],
        ]
        .concat(),
    }
}

/// The keys of each type beyond the shared ones, in table order.
const HEADER_KEYS: [&str; 2] = ["title", "description"];
const CAPABILITY_KEYS: [&str; 6] = [
    "state",
    "bindings",
    "depends",
    "review_required",
    "review_strict",
    "measured_evidence_required",
];
const CHANGE_KEYS: [&str; 4] = ["changes", "affects", "contract_effect", "items"];
const EVIDENCE_KEYS: [&str; 11] = [
    "subject",
    "inputs",
    "verdict",
    "by",
    "for",
    "command",
    "revision",
    "values",
    "limits",
    "units",
    "conditions",
];
const COVERAGE_KEYS: [&str; 2] = ["scope", "exclusions"];
const DEBT_KEYS: [&str; 2] = ["findings", "lines"];
const REVIEW_KEYS: [&str; 2] = ["kind", "findings_ref"];

#[test]
fn a_nested_unknown_key_is_reported_in_bindings_and_in_a_change_item() {
    let bindings = mutate(
        &alpha(),
        "  examples: []\n",
        "  examples: []\n  extra: []\n",
    );
    let found = violations(&bindings, ALPHA_PATH);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_hit(&found, &key("bindings"), &Rule::BindingsKeys);
    clean(&alpha(), ALPHA_PATH);

    let item = with_items("items:\n  - claim: \"x\"\n    extra: 1\n");
    let found = violations(&item, CHANGE_PATH);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_hit(
        &found,
        &key("items"),
        &Rule::ItemUnknownKey {
            key: "extra".into(),
        },
    );
    clean(&with_items("items:\n  - claim: \"x\"\n"), CHANGE_PATH);
}

#[test]
fn a_missing_type_key_is_the_only_violation() {
    let text = document_of(DocType::Architecture, Some("type"), "");
    let found = violations(&text, OVERVIEW_PATH);
    assert_eq!(found.len(), 1);
    assert_hit(
        &found,
        &key("type"),
        &Rule::MissingKey { key: "type".into() },
    );
    let list = mutate(ARCHITECTURE, "type: Architecture", "type: [Architecture]");
    assert_hit(
        &violations(&list, OVERVIEW_PATH),
        &key("type"),
        &Rule::WrongShape {
            expected: "a scalar",
        },
    );
}

#[test]
fn the_key_resource_has_its_own_rule_on_every_type() {
    for doc_type in DocType::ALL {
        let path = path_of(doc_type);
        let extra = "resource: https://example.invalid/x\n";
        let found = violations(&document_of(doc_type, None, extra), &path);
        assert_eq!(found.len(), 1, "{found:#?}");
        assert_hit(&found, &key("resource"), &Rule::ResourceKey { doc_type });
        let message = found[0].to_string();
        assert!(message.contains("`resource`"), "{message}");
        for spec in doc_type.keys() {
            assert!(message.contains(spec.key), "{message} lacks {}", spec.key);
        }
        // Without it the document is clean.
        clean(&document_of(doc_type, None, ""), &path);
    }
}

#[test]
fn an_unknown_type_is_one_violation_at_the_type_key() {
    let concept = mutate(ARCHITECTURE, "type: Architecture", "type: Concept");
    let found = violations(&concept, OVERVIEW_PATH);
    assert_eq!(found.len(), 1);
    assert_hit(
        &found,
        &key("type"),
        &Rule::UnknownType {
            found: "Concept".into(),
        },
    );
    clean(ARCHITECTURE, OVERVIEW_PATH);
    // The name is exact.
    let lower = mutate(ARCHITECTURE, "type: Architecture", "type: architecture");
    assert_eq!(violations(&lower, OVERVIEW_PATH).len(), 1);
}

#[test]
fn a_document_must_sit_directly_in_the_directory_of_its_type() {
    for (path, declared) in [
        ("docs/capabilities/overview.md", DocType::Architecture),
        ("work/overview.md", DocType::Architecture),
        ("docs/architecture/sub/overview.md", DocType::Architecture),
        ("overview.md", DocType::Architecture),
        ("docs/architecture/overview.txt", DocType::Architecture),
        ("docs/architecture/.md", DocType::Architecture),
    ] {
        let found = violations(ARCHITECTURE, path);
        assert_hit(&found, &key("type"), &Rule::WrongDirectory { declared });
    }
    let change = mutate(ARCHITECTURE, "type: Architecture", "type: Change");
    assert!(violations(&change, OVERVIEW_PATH).iter().any(|v| v.rule()
        == &Rule::WrongDirectory {
            declared: DocType::Change
        }));
    clean(ARCHITECTURE, "./docs/architecture/overview.md");
    clean(&alpha(), ALPHA_PATH);
    clean(CHANGE, CHANGE_PATH);
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn an_id_needs_the_prefix_of_its_type_a_kebab_slug_and_the_file_name() {
    use raf_okf::{DocumentIdentityError, MAX_TEXT_BYTES};
    for kind in DocType::ALL {
        let slug = kind.singleton().unwrap_or("alpha");
        let id = format!("{}{slug}", kind.id_prefix());
        assert!(
            kind.document_iri(&id)
                .unwrap()
                .starts_with(raf_okf::DOCUMENT_IRI_BASE)
        );
        for bad in ["", "Wrong", "cap:a--b", "evidence:Upper", "cap:../outside"] {
            assert!(
                matches!(
                    kind.document_iri(bad),
                    Err(DocumentIdentityError::Invalid(_))
                ),
                "{kind:?}: {bad}"
            );
        }
    }
    for kind in [DocType::Coverage, DocType::Debt] {
        assert!(matches!(
            kind.document_iri(&format!("{}other", kind.id_prefix())),
            Err(DocumentIdentityError::Invalid(Rule::SingletonId { .. }))
        ));
    }
    assert_eq!(
        DocType::Capability
            .document_iri(&format!("cap:{}", "a".repeat(MAX_TEXT_BYTES)))
            .unwrap_err(),
        DocumentIdentityError::TooLarge
    );
    for (from, to, path, prefix) in [
        ("id: cap:alpha", "id: change:alpha", ALPHA_PATH, "cap:"),
        ("id: cap:alpha", "id: cap:Alpha", ALPHA_PATH, "cap:"),
        ("id: cap:alpha", "id: cap:a--b", ALPHA_PATH, "cap:"),
        ("id: cap:alpha", "id: cap:al_pha", ALPHA_PATH, "cap:"),
        ("id: cap:alpha", "id: \"cap:\"", ALPHA_PATH, "cap:"),
    ] {
        let found = violations(&mutate(&alpha(), from, to), path);
        assert_hit(&found, &key("id"), &Rule::InvalidId { prefix });
    }
    let change = mutate(CHANGE, "id: change:add-alpha", "id: cap:add-alpha");
    assert_hit(
        &violations(&change, CHANGE_PATH),
        &key("id"),
        &Rule::InvalidId { prefix: "change:" },
    );
    let page = mutate(ARCHITECTURE, "id: arch:overview", "id: overview");
    assert_hit(
        &violations(&page, OVERVIEW_PATH),
        &key("id"),
        &Rule::InvalidId { prefix: "arch:" },
    );

    let mismatch = mutate(&alpha(), "id: cap:alpha", "id: cap:other");
    assert_hit(
        &violations(&mismatch, ALPHA_PATH),
        &key("id"),
        &Rule::IdFileName {
            id: "cap:other".into(),
            stem: "alpha".into(),
        },
    );
    clean(&mismatch, "docs/capabilities/other.md");
    let list = mutate(&alpha(), "id: cap:alpha", "id: [cap:alpha]");
    assert_hit(
        &violations(&list, ALPHA_PATH),
        &key("id"),
        &Rule::WrongShape {
            expected: "a scalar",
        },
    );
}

#[test]
fn a_title_and_a_description_are_non_empty_strings() {
    for (from, to, name) in [
        ("title: \"Title of alpha\"", "title: \"\"", "title"),
        ("title: \"Title of alpha\"", "title: [a, b]", "title"),
        ("title: \"Title of alpha\"", "title: '  '", "title"),
        (
            "description: A description of alpha.",
            "description:",
            "description",
        ),
        (
            "description: A description of alpha.",
            "description: [x]",
            "description",
        ),
    ] {
        let found = violations(&mutate(&alpha(), from, to), ALPHA_PATH);
        assert_hit(&found, &key(name), &Rule::EmptyText);
    }
    clean(
        &mutate(&alpha(), "title: \"Title of alpha\"", "title: x"),
        ALPHA_PATH,
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "the assertion reports the unexpected outcome and terminates the test"
)]
fn a_state_is_one_of_the_four() {
    let bad = violations(
        &mutate(&alpha(), "state: designed", "state: done"),
        ALPHA_PATH,
    );
    assert_hit(
        &bad,
        &key("state"),
        &Rule::BadState {
            found: "done".into(),
        },
    );
    let list = violations(
        &mutate(&alpha(), "state: designed", "state: [designed]"),
        ALPHA_PATH,
    );
    assert_hit(
        &list,
        &key("state"),
        &Rule::WrongShape {
            expected: "a scalar",
        },
    );
    for state in ["considered", "designed", "implementing"] {
        clean(
            &mutate(&alpha(), "state: designed", &format!("state: {state}")),
            ALPHA_PATH,
        );
    }
    let quoted = mutate(&alpha(), "state: designed", "state: \"designed\"");
    let RafDocument::Capability(capability) = clean(&quoted, ALPHA_PATH) else {
        panic!("a capability");
    };
    assert_eq!(capability.state, State::Designed);
}

#[test]
#[expect(
    clippy::panic,
    reason = "the assertion reports the unexpected outcome and terminates the test"
)]
fn bindings_have_exactly_the_keys_code_tests_and_examples_each_a_list() {
    // Missing and extra binding keys yield one violation at the map.
    let renamed = mutate(&alpha(), "  examples: []\n", "  docs: []\n");
    let missing = mutate(&alpha(), "  examples: []\n", "");
    for text in [&renamed, &missing] {
        let found = violations(text, ALPHA_PATH);
        assert_eq!(found.len(), 1, "{found:#?}");
        assert_hit(&found, &key("bindings"), &Rule::BindingsKeys);
    }
    let scalar = mutate(
        &alpha(),
        "bindings:\n  code: []\n  tests: []\n  examples: []\n",
        "bindings: none\n",
    );
    assert_hit(
        &violations(&scalar, ALPHA_PATH),
        &key("bindings"),
        &Rule::WrongShape {
            expected: "a map of code, tests and examples lists",
        },
    );
    let not_list = mutate(&alpha(), "  code: []", "  code: src");
    assert_hit(
        &violations(&not_list, ALPHA_PATH),
        &key("bindings"),
        &Rule::BindingNotAList {
            binding: "code".into(),
        },
    );
    let implementing = mutate(&alpha(), "state: designed", "state: implementing");
    let block = mutate(
        &implementing,
        "  code: []",
        "  code:\n    - src/a.rs\n    - src/b.rs",
    );
    let RafDocument::Capability(cap) = clean(&block, ALPHA_PATH) else {
        panic!("a capability");
    };
    assert_eq!(cap.bindings.code, ["src/a.rs", "src/b.rs"]);
}

#[test]
#[expect(
    clippy::expect_used,
    reason = "fixture prerequisites and required observations carry their assertion messages"
)]
fn bindings_by_state_designed_and_considered_bind_nothing_delivered_binds_code_and_tests() {
    for state in ["designed", "considered"] {
        let text = mutate(
            &mutate(&alpha(), "state: designed", &format!("state: {state}")),
            "  code: []",
            "  code: [src/lib.rs]",
        );
        let found = violations(&text, ALPHA_PATH);
        let parsed = State::from_name(state).expect("state");
        assert_hit(
            &found,
            &key("bindings"),
            &Rule::BindingNotEmpty {
                state: parsed,
                binding: "code".into(),
            },
        );
    }
    let delivered = capability("alpha", "delivered", "[]", "");
    clean(&delivered, ALPHA_PATH);
    let no_code = mutate(&delivered, "  code: [src/lib.rs]", "  code: []");
    let found = violations(&no_code, ALPHA_PATH);
    assert_hit(
        &found,
        &key("bindings"),
        &Rule::BindingRequired {
            binding: "code".into(),
        },
    );
    let no_tests = mutate(&delivered, "  tests: [tests/it.rs]", "  tests: []");
    assert_hit(
        &violations(&no_tests, ALPHA_PATH),
        &key("bindings"),
        &Rule::BindingRequired {
            binding: "tests".into(),
        },
    );
    // Examples may be empty when delivered, and implementing binds freely.
    clean(
        &mutate(&alpha(), "state: designed", "state: implementing"),
        ALPHA_PATH,
    );
    clean(
        &mutate(
            &mutate(&alpha(), "state: designed", "state: implementing"),
            "  code: []",
            "  code: [src/lib.rs]",
        ),
        ALPHA_PATH,
    );
}

#[test]
fn depends_is_a_list_of_cap_ids_and_does_not_need_the_capability_to_exist() {
    for (to, rule) in [
        (
            "depends: [beta]",
            Rule::BadCapabilityId {
                entry: "beta".into(),
            },
        ),
        (
            "depends: [cap:Beta]",
            Rule::BadCapabilityId {
                entry: "cap:Beta".into(),
            },
        ),
        (
            "depends: [urn:raf:okf:key-present]",
            Rule::BadCapabilityId {
                entry: "urn:raf:okf:key-present".into(),
            },
        ),
        (
            "depends: cap:beta",
            Rule::WrongShape {
                expected: "a list of cap ids",
            },
        ),
        (
            "depends:\n  - a: b\n  - c: d",
            Rule::WrongShape {
                expected: "a list of cap ids",
            },
        ),
    ] {
        let found = violations(&mutate(&alpha(), "depends: [cap:beta]", to), ALPHA_PATH);
        assert_hit(&found, &key("depends"), &rule);
    }
    clean(
        &mutate(
            &alpha(),
            "depends: [cap:beta]",
            "depends: [cap:does-not-exist]",
        ),
        ALPHA_PATH,
    );
    clean(
        &mutate(&alpha(), "depends: [cap:beta]", "depends: []"),
        ALPHA_PATH,
    );
    clean(
        &mutate(
            &alpha(),
            "depends: [cap:beta]",
            "depends:\n  - cap:beta\n  - cap:gamma",
        ),
        ALPHA_PATH,
    );
    // Every refused member is its own finding in authored order, a repeated
    // member included; the valid members between them are not refused.
    let refused = violations(
        &mutate(
            &alpha(),
            "depends: [cap:beta]",
            "depends: [cap:Beta, urn:raf:okf:key-present, cap:gamma-2, beta, beta]",
        ),
        ALPHA_PATH,
    );
    let entries: Vec<_> = refused
        .iter()
        .filter(|v| v.at() == &key("depends"))
        .map(|v| v.rule().clone())
        .collect();
    let bad = |entry: &str| Rule::BadCapabilityId {
        entry: entry.into(),
    };
    assert_eq!(
        entries,
        [
            bad("cap:Beta"),
            bad("urn:raf:okf:key-present"),
            bad("beta"),
            bad("beta")
        ]
    );
}

#[test]
fn changes_is_a_non_empty_list_of_cap_ids() {
    let from = "changes: [cap:alpha, cap:beta]";
    for (to, rule) in [
        ("changes: []", Rule::EmptyChanges),
        (
            "changes: cap:alpha",
            Rule::WrongShape {
                expected: "a list of cap ids",
            },
        ),
        (
            "changes:\n  - a: b\n  - c: d",
            Rule::WrongShape {
                expected: "a list of cap ids",
            },
        ),
        (
            "changes: [cap:alpha, urn:raf:okf:key-present]",
            Rule::BadCapabilityId {
                entry: "urn:raf:okf:key-present".into(),
            },
        ),
        (
            "changes: [cap:alpha, beta]",
            Rule::BadCapabilityId {
                entry: "beta".into(),
            },
        ),
    ] {
        let found = violations(&mutate(CHANGE, from, to), CHANGE_PATH);
        assert_hit(&found, &key("changes"), &rule);
    }
    // An empty list is refused as empty, never as a missing key; an absent key is
    // the reverse.
    let empty = violations(&mutate(CHANGE, from, "changes: []"), CHANGE_PATH);
    assert!(
        !empty
            .iter()
            .any(|v| matches!(v.rule(), Rule::MissingKey { .. }))
    );
    let absent = violations(
        &mutate(CHANGE, "changes: [cap:alpha, cap:beta]\n", ""),
        CHANGE_PATH,
    );
    assert_hit(
        &absent,
        &key("changes"),
        &Rule::MissingKey {
            key: "changes".into(),
        },
    );
    assert!(!absent.iter().any(|v| v.rule() == &Rule::EmptyChanges));
    clean(CHANGE, CHANGE_PATH);
}

#[test]
fn affects_entries_have_the_form_a_section_of_the_seven_and_a_capability_in_changes() {
    let list = "affects:\n  - cap:alpha#contract\n  - cap:beta#purpose\n";
    let scalar = violations(
        &mutate(CHANGE, list, "affects: cap:alpha#contract\n"),
        CHANGE_PATH,
    );
    assert_hit(
        &scalar,
        &key("affects"),
        &Rule::WrongShape {
            expected: "a list of strings",
        },
    );
    for entry in ["alpha-contract", "cap:alpha", "cap:Alpha#contract"] {
        let found = violations(&mutate(CHANGE, "cap:alpha#contract", entry), CHANGE_PATH);
        assert_hit(
            &found,
            &key("affects"),
            &Rule::BadAffects {
                entry: entry.into(),
            },
        );
    }
    let section = violations(
        &mutate(CHANGE, "cap:alpha#contract", "cap:alpha#nonsense"),
        CHANGE_PATH,
    );
    assert_hit(
        &section,
        &key("affects"),
        &Rule::BadAffectsSection {
            entry: "cap:alpha#nonsense".into(),
            section: "nonsense".into(),
        },
    );
    let missing = violations(
        &mutate(CHANGE, "cap:alpha#contract", "cap:gamma#contract"),
        CHANGE_PATH,
    );
    assert_hit(
        &missing,
        &key("affects"),
        &Rule::AffectsNotInChanges {
            entry: "cap:gamma#contract".into(),
        },
    );
    for section in CapabilitySection::ALL {
        let text = mutate(
            CHANGE,
            "cap:alpha#contract",
            &format!("cap:alpha#{}", section.slug()),
        );
        clean(&text, CHANGE_PATH);
    }
}

#[test]
#[expect(
    clippy::panic,
    reason = "the assertion reports the unexpected outcome and terminates the test"
)]
fn contract_effect_is_none_or_changes() {
    for (to, found) in [
        ("contract_effect: maybe", "maybe"),
        ("contract_effect: [none]", "<not a scalar>"),
    ] {
        let violations = violations(&mutate(CHANGE, "contract_effect: changes", to), CHANGE_PATH);
        assert_hit(
            &violations,
            &key("contract_effect"),
            &Rule::BadContractEffect {
                found: found.into(),
            },
        );
    }
    let RafDocument::Change(change) = clean(
        &mutate(
            CHANGE,
            "contract_effect: changes",
            "contract_effect: \"none\"",
        ),
        CHANGE_PATH,
    ) else {
        panic!("a change");
    };
    assert_eq!(change.contract_effect, ContractEffect::None);
}

#[expect(
    clippy::expect_used,
    reason = "fixture prerequisites and required observations carry their assertion messages"
)]
fn with_items(items: &str) -> String {
    let front_end = CHANGE.find("items:\n").expect("items");
    let body_start = CHANGE.find("\n---\n").expect("fence") + 1;
    let front = CHANGE
        .get(..front_end)
        .expect("fixture field starts on a character boundary");
    let body = CHANGE
        .get(body_start..)
        .expect("fixture fence starts on a character boundary");
    format!("{front}{items}{body}")
}

#[test]
fn items_need_a_claim_known_keys_and_a_known_outcome() {
    // Identical maps each get a finding, in their authored sequence.
    let ordered = with_items(concat!(
        "items:\n  - zebra: 1\n    ant: 2\n    outcome: held\n",
        "  - claim: x\n    zebra: 3\n",
        "  - claim: x\n    zebra: 3\n",
    ));
    let found = violations(&ordered, CHANGE_PATH);
    let expected = [
        Rule::ItemUnknownKey {
            key: "zebra".into(),
        },
        Rule::ItemUnknownKey { key: "ant".into() },
        Rule::ItemClaimEmpty,
        Rule::BadOutcome {
            found: "held".into(),
        },
        Rule::ItemUnknownKey {
            key: "zebra".into(),
        },
        Rule::ItemUnknownKey {
            key: "zebra".into(),
        },
    ];
    assert_eq!(found.len(), expected.len(), "{found:#?}");
    let location = key("items");
    for (finding, rule) in found.iter().zip(expected) {
        assert_eq!(finding.at(), &location);
        assert_eq!(finding.rule(), &rule);
    }
    for (items, rule) in [
        (
            "items:\n  - claim: \"x\"\n    owner: me\n",
            Rule::ItemUnknownKey {
                key: "owner".into(),
            },
        ),
        ("items:\n  - outcome: dropped\n", Rule::ItemClaimEmpty),
        ("items:\n  - claim: \"\"\n", Rule::ItemClaimEmpty),
        ("items:\n  - claim: [a]\n", Rule::ItemClaimEmpty),
        (
            "items:\n  - claim: x\n    outcome: finished\n",
            Rule::BadOutcome {
                found: "finished".into(),
            },
        ),
        (
            "items:\n  - claim: x\n    outcome: moved:change:Bad\n",
            Rule::BadOutcome {
                found: "moved:change:Bad".into(),
            },
        ),
        (
            "items:\n  - claim: x\n    outcome: [dropped]\n",
            Rule::BadOutcome {
                found: "<not a scalar>".into(),
            },
        ),
    ] {
        let found = violations(&with_items(items), CHANGE_PATH);
        assert_hit(&found, &key("items"), &rule);
    }
    let shape = Rule::WrongShape {
        expected: "a list of maps with `claim` and optional `outcome`",
    };
    for items in ["items:\n  - one\n  - two\n", "items: nothing\n"] {
        assert_hit(
            &violations(&with_items(items), CHANGE_PATH),
            &key("items"),
            &shape,
        );
    }
    clean(&with_items("items: []\n"), CHANGE_PATH);
    clean(&with_items("items:\n  - claim: x\n"), CHANGE_PATH);
}

#[test]
#[expect(
    clippy::panic,
    reason = "the assertion reports the unexpected outcome and terminates the test"
)]
fn the_three_outcomes_are_typed() {
    let items = "items:\n  - claim: a\n    outcome: implemented\n  - claim: b\n    outcome: dropped\n  - claim: c\n    outcome: moved:change:other-work\n";
    let RafDocument::Change(change) = clean(&with_items(items), CHANGE_PATH) else {
        panic!("a change");
    };
    let outcomes: Vec<Option<Outcome>> = change.items.iter().map(|i| i.outcome.clone()).collect();
    assert_eq!(
        outcomes,
        [
            Some(Outcome::Implemented),
            Some(Outcome::Dropped),
            Some(Outcome::Moved("other-work".into()))
        ]
    );
}

/// The optional `reason` of an item is a non-empty line of text, accepted
/// beside any outcome or none; an empty one, one of spaces and one that is not
/// a scalar are each `ItemReasonEmpty` at `items`, and a placeholder token in it
/// is reported as in the claim.
#[test]
#[expect(
    clippy::panic,
    reason = "the assertion reports the unexpected outcome and terminates the test"
)]
fn an_item_reason_is_a_non_empty_line_beside_any_outcome_and_an_empty_one_is_refused() {
    let items = "items:\n  - claim: a\n    outcome: dropped\n    reason: \"The window allocates while it fills.\"\n  - claim: b\n    outcome: moved:change:other-work\n    reason: Streaming needs the buffer rework.\n  - claim: c\n    reason: Written before the outcome.\n  - claim: d\n    outcome: implemented\n";
    let RafDocument::Change(change) = clean(&with_items(items), CHANGE_PATH) else {
        panic!("a change");
    };
    let reasons: Vec<Option<&str>> = change.items.iter().map(|i| i.reason.as_deref()).collect();
    assert_eq!(
        reasons,
        [
            Some("The window allocates while it fills."),
            Some("Streaming needs the buffer rework."),
            Some("Written before the outcome."),
            None
        ]
    );
    for reason in ["\"\"", "\"   \"", "[a, b]"] {
        let items = format!("items:\n  - claim: x\n    outcome: dropped\n    reason: {reason}\n");
        let found = violations(&with_items(&items), CHANGE_PATH);
        assert_hit(&found, &key("items"), &Rule::ItemReasonEmpty);
        assert_eq!(found.len(), 1, "{reason}: {found:?}");
    }
    let word = token(2);
    let placeholder =
        format!("items:\n  - claim: x\n    outcome: dropped\n    reason: \"Not {word} here.\"\n");
    assert_hit(
        &violations(&with_items(&placeholder), CHANGE_PATH),
        &key("items"),
        &Rule::Placeholder { token: word.into() },
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "the assertion reports the unexpected outcome and terminates the test"
)]
fn tags_are_non_empty_unique_strings_and_a_timestamp_is_rfc_3339() {
    let with = |extra: &str| {
        mutate(
            ARCHITECTURE,
            "title: Overview",
            &format!("title: Overview\n{extra}"),
        )
    };
    let RafDocument::Architecture(page) = clean(
        &with("tags: [b, a]\ntimestamp: 2026-01-01T00:00:00Z"),
        OVERVIEW_PATH,
    ) else {
        panic!("a page");
    };
    assert_eq!(page.header.tags, Some(vec!["a".into(), "b".into()]));
    assert_eq!(
        page.header.timestamp.as_deref(),
        Some("2026-01-01T00:00:00Z")
    );
    clean(&with("tags: []"), OVERVIEW_PATH);
    assert_hit(
        &violations(&with("tags: [a, a]"), OVERVIEW_PATH),
        &key("tags"),
        &Rule::DuplicateTag { tag: "a".into() },
    );
    assert_hit(
        &violations(&with("tags: [\"\"]"), OVERVIEW_PATH),
        &key("tags"),
        &Rule::EmptyTag,
    );
    assert_hit(
        &violations(&with("tags: a"), OVERVIEW_PATH),
        &key("tags"),
        &Rule::WrongShape {
            expected: "a list of strings",
        },
    );
    assert_hit(
        &violations(&with("timestamp: 2026-13-01T00:00:00Z"), OVERVIEW_PATH),
        &key("timestamp"),
        &Rule::BadTimestamp {
            reason: TimestampError::OutOfRange { field: "month" },
        },
    );
    assert_hit(
        &violations(&with("timestamp: yesterday"), OVERVIEW_PATH),
        &key("timestamp"),
        &Rule::BadTimestamp {
            reason: TimestampError::Malformed,
        },
    );
    assert_hit(
        &violations(&with("timestamp: [2026]"), OVERVIEW_PATH),
        &key("timestamp"),
        &Rule::WrongShape {
            expected: "a scalar",
        },
    );
    // Neither key is required.
    let RafDocument::Architecture(plain) = clean(ARCHITECTURE, OVERVIEW_PATH) else {
        panic!("a page");
    };
    assert_eq!((plain.header.tags, plain.header.timestamp), (None, None));
    // They are allowed on a capability and a change as well.
    clean(
        &mutate(&alpha(), "state: designed", "tags: [x]\nstate: designed"),
        ALPHA_PATH,
    );
    clean(
        &mutate(
            CHANGE,
            "contract_effect: changes",
            "timestamp: 2026-01-01T00:00:00+01:00\ncontract_effect: changes",
        ),
        CHANGE_PATH,
    );
}

fn capability_with_section(replacing: &str, with: &str) -> String {
    mutate(&alpha(), replacing, with)
}

#[test]
fn a_capability_holds_the_seven_sections_in_order() {
    let missing = capability_with_section("## Usage\n\nText for Usage.\n\n", "");
    let found = violations(&missing, ALPHA_PATH);
    assert_hit(&found, &section("Usage"), &Rule::SectionMissing);

    let renamed = capability_with_section("## Purpose", "## Swap");
    let found = violations(&renamed, ALPHA_PATH);
    assert_hit(&found, &section("Swap"), &Rule::SectionUnexpected);
    assert_hit(&found, &section("Purpose"), &Rule::SectionMissing);

    let swapped = alpha()
        .replace("## Contract", "## TMP")
        .replace("## Usage", "## Contract")
        .replace("## TMP", "## Usage");
    let found = violations(&swapped, ALPHA_PATH);
    assert_hit(
        &found,
        &Location::Document,
        &Rule::SectionsOutOfOrder {
            found: [
                "Purpose",
                "Usage",
                "Contract",
                "Examples",
                "Constraints",
                "Explanation",
                "Verification",
            ]
            .iter()
            .map(ToString::to_string)
            .collect(),
        },
    );

    let doubled = format!("{}\n## Usage\n\nagain\n", alpha());
    assert_hit(
        &violations(&doubled, ALPHA_PATH),
        &Location::Document,
        &Rule::SectionDuplicated,
    );
    clean(&alpha(), ALPHA_PATH);
}

#[test]
fn a_section_is_empty_when_only_comments_and_generated_blocks_remain() {
    for empty in [
        "",
        "<!-- later -->\n",
        "<!--\nlater\n-->\n",
        "<!-- a --> <!-- b -->\n",
        "<!-- raf:generated:list -->\n| row |\n<!-- raf:generated:end -->\n",
        "<!-- note -->\n<!-- raf:generated:list -->\n| row |\n<!-- raf:generated:end -->\n\n<!-- more -->\n",
    ] {
        let text = capability_with_section("Text for Examples.\n", empty);
        let found = violations(&text, ALPHA_PATH);
        assert_hit(&found, &section("Examples"), &Rule::SectionEmpty);
        assert_eq!(found.len(), 1, "{empty:?}: {found:#?}");
    }
    // One prose line beside a generated block, or beside a comment, is valid.
    clean(
        &capability_with_section(
            "Text for Examples.\n",
            "<!-- raf:generated:list -->\n| row |\n<!-- raf:generated:end -->\n\nOne line of prose.\n",
        ),
        ALPHA_PATH,
    );
    clean(
        &capability_with_section("Text for Examples.\n", "<!-- note -->\nProse.\n"),
        ALPHA_PATH,
    );
    // A fenced block is content.
    clean(
        &capability_with_section("Text for Examples.\n", "```text\ncode\n```\n"),
        ALPHA_PATH,
    );
}

#[test]
fn a_change_needs_a_non_empty_summary() {
    let missing = mutate(CHANGE, "## Summary", "## Notes");
    assert_hit(
        &violations(&missing, CHANGE_PATH),
        &section("Summary"),
        &Rule::SectionMissing,
    );
    for empty in [
        "<!-- soon -->\n",
        "",
        "<!-- raf:generated:list -->\nx\n<!-- raf:generated:end -->\n",
    ] {
        let text = mutate(CHANGE, "Adds both.\n", empty);
        assert_hit(
            &violations(&text, CHANGE_PATH),
            &section("Summary"),
            &Rule::SectionEmpty,
        );
    }
    // Other sections are free.
    clean(&format!("{CHANGE}\n## Notes\n\nmore\n"), CHANGE_PATH);
}

#[test]
fn an_architecture_body_is_not_empty() {
    for empty in ["<!-- nothing yet -->\n", "", "\n\n"] {
        let text = mutate(ARCHITECTURE, "# Overview\n\nBody text.\n", empty);
        assert_hit(
            &violations(&text, OVERVIEW_PATH),
            &Location::Document,
            &Rule::BodyEmpty,
        );
    }
    clean(
        &mutate(ARCHITECTURE, "# Overview\n\nBody text.\n", "x\n"),
        OVERVIEW_PATH,
    );
}

const PROPOSED: &str = "> [!PROPOSED change:add-alpha]\n> The candidate contract.\n\n";

#[test]
#[expect(
    clippy::panic,
    reason = "the assertion reports the unexpected outcome and terminates the test"
)]
fn a_proposed_block_is_valid_only_inside_a_section_of_a_capability() {
    let beta = capability("beta", "considered", "[]", PROPOSED);
    let RafDocument::Capability(cap) = clean(&beta, "docs/capabilities/beta.md") else {
        panic!("a capability");
    };
    assert_eq!(cap.state, State::Considered);

    let page = mutate(
        ARCHITECTURE,
        "Body text.",
        "> [!PROPOSED change:add-alpha]\n> Not allowed here.",
    );
    let found = violations(&page, OVERVIEW_PATH);
    assert_hit(
        &found,
        &Location::Line(10),
        &Rule::ProposedOutsideCapability {
            slug: "add-alpha".into(),
        },
    );

    let change = mutate(
        CHANGE,
        "Adds both.",
        "Adds both.\n\n> [!PROPOSED change:add-alpha]\n> Not allowed here.",
    );
    let found = violations(&change, CHANGE_PATH);
    assert!(
        found
            .iter()
            .any(|v| matches!(v.rule(), Rule::ProposedOutsideCapability { .. }))
    );

    let before_sections = mutate(&beta, "# beta\n", &format!("# beta\n\n{PROPOSED}"));
    let outside = before_sections.replace(&format!("Purpose\n\n{PROPOSED}"), "Purpose\n\n");
    let found = violations(&outside, "docs/capabilities/beta.md");
    assert!(
        found.iter().any(|v| v.rule()
            == &Rule::ProposedOutsideSection {
                slug: "add-alpha".into()
            }
            && matches!(v.at(), Location::Line(_))),
        "{found:#?}"
    );
}

fn token(index: usize) -> &'static str {
    [
        concat!("t", "bd"),
        concat!("to", "do"),
        concat!("fix", "me"),
        concat!("x", "xx"),
        concat!("to be ", "written"),
    ][index]
}

#[test]
fn placeholder_tokens_in_prose_are_reported_at_their_line() {
    for index in 0..5 {
        let text = mutate(
            ARCHITECTURE,
            "Body text.",
            &format!("Body text. Still {} here.", token(index)),
        );
        let found = violations(&text, OVERVIEW_PATH);
        assert_eq!(found.len(), 1, "{found:#?}");
        assert_hit(
            &found,
            &Location::Line(10),
            &Rule::Placeholder {
                token: token(index).to_string(),
            },
        );
    }
    for spelling in [
        concat!("T.B", ".D."),
        concat!("t.b", ".d"),
        concat!("To", "Do"),
        concat!("Fix", "ME:"),
        concat!("x", "xX"),
        concat!("to  be", "\twritten"),
        concat!("TO BE ", "WRITTEN"),
    ] {
        let text = mutate(
            ARCHITECTURE,
            "Body text.",
            &format!("Body text. Still {spelling} soon."),
        );
        let found = violations(&text, OVERVIEW_PATH);
        assert!(
            found
                .iter()
                .all(|v| matches!(v.rule(), Rule::Placeholder { .. }))
                && !found.is_empty(),
            "{spelling}: {found:#?}"
        );
    }
}

#[test]
fn placeholder_scan_spares_code_longer_words_and_dotted_prose() {
    let word = ["T", "BD"].concat();
    let body = format!(
        "Body text.\n\n```text\n{word}\n```\n\nUse `{word}` and {word}x and {}s, e.g. a.b.c.\n",
        concat!("x", "xxx"),
    );
    clean(&mutate(ARCHITECTURE, "Body text.\n", &body), OVERVIEW_PATH);
    // Text in an HTML comment is not prose.
    clean(
        &mutate(
            ARCHITECTURE,
            "Body text.\n",
            &format!("Body text.\n\n<!-- {word} -->\n"),
        ),
        OVERVIEW_PATH,
    );
}

#[test]
fn placeholder_tokens_in_frontmatter_values_are_reported_at_the_key() {
    let word = token(0);
    let title = mutate(
        ARCHITECTURE,
        "title: Overview",
        &format!("title: \"Overview {word}\""),
    );
    assert_hit(
        &violations(&title, OVERVIEW_PATH),
        &key("title"),
        &Rule::Placeholder { token: word.into() },
    );
    let description = mutate(
        ARCHITECTURE,
        "description: How it fits.",
        &format!("description: \"{}\"", token(4)),
    );
    assert_hit(
        &violations(&description, OVERVIEW_PATH),
        &key("description"),
        &Rule::Placeholder {
            token: token(4).into(),
        },
    );
    let claim = mutate(
        CHANGE,
        "alpha reads input",
        &format!("alpha {} input", token(1)),
    );
    assert_hit(
        &violations(&claim, CHANGE_PATH),
        &key("items"),
        &Rule::Placeholder {
            token: token(1).into(),
        },
    );
    // Longer words and the neighbouring plain values are fine.
    clean(
        &mutate(
            ARCHITECTURE,
            "title: Overview",
            &format!("title: \"Overview {word}x\""),
        ),
        OVERVIEW_PATH,
    );
}

#[test]
fn future_tense_prose_is_not_a_violation() {
    let text = mutate(
        ARCHITECTURE,
        "Body text.",
        "Body text. The crate will read files later and is going to add a writer.",
    );
    clean(&text, OVERVIEW_PATH);
}

#[test]
fn a_split_link_is_reported_at_its_line_and_a_one_line_link_is_not() {
    let split = mutate(
        ARCHITECTURE,
        "Body text.",
        "Body text. See [the page](\nother.md).",
    );
    let found = violations(&split, OVERVIEW_PATH);
    assert_eq!(found.len(), 1);
    assert_hit(
        &found,
        &Location::Line(10),
        &Rule::SplitLink {
            form: LinkForm::Inline,
        },
    );
    let image = mutate(ARCHITECTURE, "Body text.", "![alt\ntext](p.png)");
    assert_hit(
        &violations(&image, OVERVIEW_PATH),
        &Location::Line(10),
        &Rule::SplitLink {
            form: LinkForm::Image,
        },
    );
    clean(
        &mutate(
            ARCHITECTURE,
            "Body text.",
            "Body text. See [the page](other.md).",
        ),
        OVERVIEW_PATH,
    );
}

#[test]
fn a_document_with_three_faults_returns_three_violations() {
    let text = mutate(
        &mutate(
            &mutate(&alpha(), "state: designed", "state: done"),
            "depends: [cap:beta]",
            "depends: [beta]",
        ),
        "Text for Usage.\n",
        "",
    );
    let found = violations(&text, ALPHA_PATH);
    assert_eq!(found.len(), 3, "{found:#?}");
    assert_hit(
        &found,
        &key("state"),
        &Rule::BadState {
            found: "done".into(),
        },
    );
    assert_hit(
        &found,
        &key("depends"),
        &Rule::BadCapabilityId {
            entry: "beta".into(),
        },
    );
    assert_hit(&found, &section("Usage"), &Rule::SectionEmpty);
}

#[test]
fn a_violation_displays_the_file_the_place_and_the_rule() {
    let text = mutate(&alpha(), "state: designed", "state: done");
    let found = violations(&text, ALPHA_PATH);
    assert_eq!(
        found[0].to_string(),
        "docs/capabilities/alpha.md: `state`: bad state `done`"
    );
    let missing = violations(
        &capability_with_section("## Usage\n\nText for Usage.\n\n", ""),
        ALPHA_PATH,
    );
    assert_eq!(
        missing[0].to_string(),
        "docs/capabilities/alpha.md: section `Usage`: missing section"
    );
    let line = violations(
        &mutate(ARCHITECTURE, "Body text.", &format!("Body {}", token(0))),
        OVERVIEW_PATH,
    );
    assert!(
        line[0]
            .to_string()
            .starts_with("docs/architecture/overview.md: line 10: placeholder `")
    );
}

#[test]
fn a_placeholder_token_in_a_code_span_of_a_frontmatter_value_is_spared() {
    let word = token(0);
    let bare = mutate(
        ARCHITECTURE,
        "title: Overview",
        &format!("title: \"Overview {word}\""),
    );
    assert_hit(
        &violations(&bare, OVERVIEW_PATH),
        &key("title"),
        &Rule::Placeholder { token: word.into() },
    );
    let spanned = mutate(
        ARCHITECTURE,
        "title: Overview",
        &format!("title: \"Overview `{word}`\""),
    );
    clean(&spanned, OVERVIEW_PATH);
    let claim = mutate(
        CHANGE,
        "alpha reads input",
        &format!("alpha reads ``{word}``"),
    );
    clean(&claim, CHANGE_PATH);
}

#[test]
fn an_invalid_changes_entry_does_not_cascade_into_the_affects_entries() {
    let text = mutate(
        CHANGE,
        "changes: [cap:alpha, cap:beta]",
        "changes: [cap:alpha, cap:beta, Bad]",
    );
    let found = violations(&text, CHANGE_PATH);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_hit(
        &found,
        &key("changes"),
        &Rule::BadCapabilityId {
            entry: "Bad".into(),
        },
    );
    // An affects entry that names a capability not listed is still reported.
    let missing = mutate(&text, "cap:alpha#contract", "cap:gamma#contract");
    let found = violations(&missing, CHANGE_PATH);
    assert_eq!(found.len(), 2, "{found:#?}");
    assert_hit(
        &found,
        &key("affects"),
        &Rule::AffectsNotInChanges {
            entry: "cap:gamma#contract".into(),
        },
    );
}
