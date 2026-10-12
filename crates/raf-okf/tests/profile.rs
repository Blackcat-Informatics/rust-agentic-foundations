// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The profile validator: one fixture per rule, each beside a neighbouring
//! document that is accepted.

use raf_okf::{
    CapabilitySection, ContractEffect, DocType, Document, LinkForm, Location, Outcome, Profile,
    RafDocument, Rule, State, TimestampError, Violation,
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

fn alpha() -> String {
    capability("alpha", "designed", "[cap:beta]", "")
}

fn parse(text: &str) -> Document {
    Document::parse(text).unwrap_or_else(|error| panic!("{error} in {text:?}"))
}

fn validate(text: &str, path: &str) -> Result<RafDocument, Vec<Violation>> {
    Profile.validate(&parse(text), path)
}

fn violations(text: &str, path: &str) -> Vec<Violation> {
    match validate(text, path) {
        Ok(typed) => panic!("expected violations for {path}, got {typed:?}"),
        Err(found) => found,
    }
}

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

#[test]
fn valid_documents_of_each_type_validate_to_typed_values() {
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
}

#[test]
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
        ],
    }
}

fn body_for(doc_type: DocType) -> String {
    match doc_type {
        DocType::Capability => SECTIONS
            .iter()
            .map(|name| format!("## {name}\n\ntext\n"))
            .collect::<Vec<_>>()
            .join("\n"),
        DocType::Change => "## Summary\n\ntext\n".to_string(),
        DocType::Architecture => "# Alpha\n\ntext\n".to_string(),
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
    format!("{}/alpha.md", doc_type.directory())
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
fn an_id_needs_the_prefix_of_its_type_a_kebab_slug_and_the_file_name() {
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
}

#[test]
fn bindings_have_exactly_the_keys_code_tests_and_examples_each_a_list() {
    let renamed = mutate(&alpha(), "  examples: []\n", "  docs: []\n");
    assert_hit(
        &violations(&renamed, ALPHA_PATH),
        &key("bindings"),
        &Rule::BindingsKeys,
    );
    let extra = mutate(&alpha(), "  examples: []\n", "  examples: []\n  docs: []\n");
    assert_hit(
        &violations(&extra, ALPHA_PATH),
        &key("bindings"),
        &Rule::BindingsKeys,
    );
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
            "depends: cap:beta",
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
            "changes: [cap:alpha, beta]",
            Rule::BadCapabilityId {
                entry: "beta".into(),
            },
        ),
    ] {
        let found = violations(&mutate(CHANGE, from, to), CHANGE_PATH);
        assert_hit(&found, &key("changes"), &rule);
    }
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
        &mutate(CHANGE, "contract_effect: changes", "contract_effect: none"),
        CHANGE_PATH,
    ) else {
        panic!("a change");
    };
    assert_eq!(change.contract_effect, ContractEffect::None);
}

fn with_items(items: &str) -> String {
    let front_end = CHANGE.find("items:\n").expect("items");
    let body_start = CHANGE.find("\n---\n").expect("fence") + 1;
    format!("{}{items}{}", &CHANGE[..front_end], &CHANGE[body_start..])
}

#[test]
fn items_need_a_claim_known_keys_and_a_known_outcome() {
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

#[test]
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
