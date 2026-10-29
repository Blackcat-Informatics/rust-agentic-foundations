// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Loading a repository: the valid fixture, the model it builds, the flat
//! directories and the files that cannot be read. The names of the ported
//! tests are those of the document-contract test they come from.

mod support;

use raf_okf::{CapabilitySection, ContractEffect, Location, MAX_TEXT_BYTES, Section, State};
use raf_source::MemoryTree;
use raf_workplan::{Loaded, Workplan};
use support::fixtures::{
    ALPHA, BETA, CHANGE, GAMMA, OVERVIEW, capability, capability_section, change_record, mutated,
    paragraph, valid,
};
use support::{
    Repo, assert_cases, assert_clean, assert_hit, assert_kind, check, key, section,
    without_sync_findings,
};

#[test]
fn accepts_valid_fixture() {
    let repo = valid();
    let tree: MemoryTree = repo.iter().collect();
    let workspace = support::demo_workspace();
    let plan = Workplan::load(&tree, &workspace);
    assert_clean(plan.findings());
    assert_clean(&without_sync_findings(plan.check()));
    assert_eq!(
        plan.capabilities()
            .iter()
            .map(Loaded::path)
            .collect::<Vec<_>>(),
        [ALPHA, support::AUXILIARY, BETA, GAMMA]
    );
    assert_eq!(plan.changes().len(), 1);
    assert_eq!(plan.architecture().len(), 1);
    assert_eq!(plan.source().packages().len(), 1);
}

#[test]
fn the_model_holds_typed_values_and_bodies() {
    let repo = valid();
    let tree: MemoryTree = repo.iter().collect();
    let workspace = support::demo_workspace();
    let plan = Workplan::load(&tree, &workspace);
    let paths: Vec<&str> = plan.capabilities().iter().map(Loaded::path).collect();
    assert_eq!(paths, [ALPHA, support::AUXILIARY, BETA, GAMMA]);
    let states: Vec<State> = plan
        .capabilities()
        .iter()
        .map(|c| c.value().state)
        .collect();
    assert_eq!(
        states,
        [
            State::Designed,
            State::Delivered,
            State::Considered,
            State::Delivered
        ]
    );
    let gamma = &plan.capabilities()[3];
    assert_eq!(gamma.value().header.id, "cap:gamma");
    assert_eq!(gamma.value().bindings.code, ["demo::real"]);
    assert_eq!(gamma.value().depends, ["cap:beta", "cap:alpha"]);
    let headings: Vec<&str> = gamma
        .body()
        .sections()
        .iter()
        .map(Section::heading)
        .collect();
    assert_eq!(headings, support::fixtures::SECTIONS);
    let beta = &plan.capabilities()[2];
    assert_eq!(beta.body().proposed_blocks().len(), 1);
    let change = &plan.changes()[0];
    assert_eq!(change.path(), CHANGE);
    assert_eq!(change.value().changes, ["cap:alpha", "cap:beta"]);
    assert_eq!(change.value().contract_effect, ContractEffect::Changes);
    assert_eq!(
        change.value().affects[0].section,
        CapabilitySection::Contract
    );
    let overview = &plan.architecture()[0];
    assert_eq!(overview.path(), OVERVIEW);
    assert_eq!(overview.value().header.title, "Overview");
    assert!(overview.body().text().contains("Body text."));
}

#[test]
fn a_broken_document_is_a_finding_and_not_in_the_model() {
    let repo = support::fixtures::mutate(ALPHA, "state: designed", "state: draft");
    let tree: MemoryTree = repo.iter().collect();
    let workspace = support::demo_workspace();
    let plan = Workplan::load(&tree, &workspace);
    assert_eq!(
        plan.capabilities()
            .iter()
            .map(Loaded::path)
            .collect::<Vec<_>>(),
        [support::AUXILIARY, BETA, GAMMA]
    );
    assert_eq!(plan.findings().len(), 1);
    assert_eq!(plan.findings()[0].kind, "okf-bad-state");
}

#[test]
fn an_empty_repository_has_no_findings() {
    assert_clean(&check(&Repo::new()));
}

#[test]
fn fixture_builder_is_self_consistent() {
    let doc = capability("delta", "implementing", "[]", "");
    assert!(doc.contains("id: cap:delta"));
    let mut repo = valid();
    repo.insert("docs/capabilities/delta.md".to_string(), doc);
    // A capability that is not delivered is listed by an open record.
    repo.insert(
        "work/changes/add-delta.md".to_string(),
        change_record("add-delta", "[cap:delta]", "  - cap:delta#contract\n", ""),
    );
    assert_clean(&check(&repo));
}

#[test]
fn rejects_unparseable_frontmatter() {
    let findings = mutated(OVERVIEW, "title: Overview", "title Overview");
    assert_hit(
        &findings,
        "okf-parse-expected-key",
        OVERVIEW,
        &Location::Line(4),
    );
}

#[test]
fn rejects_unknown_key() {
    assert_cases(
        ALPHA,
        &[(
            "state: designed",
            "state: designed\ncolour: red",
            "okf-unknown-key",
            key("colour"),
        )],
    );
}

#[test]
fn rejects_type_in_the_wrong_directory() {
    let findings = mutated(OVERVIEW, "type: Architecture", "type: Change");
    assert_hit(&findings, "wrong-directory", OVERVIEW, &key("type"));
    assert_hit(&findings, "okf-wrong-directory", OVERVIEW, &key("type"));
}

#[test]
fn rejects_id_filename_mismatch() {
    let findings = mutated(ALPHA, "id: cap:alpha", "id: cap:other");
    assert_hit(&findings, "okf-id-file-name", ALPHA, &key("id"));
}

#[test]
fn rejects_missing_section() {
    let findings = mutated(ALPHA, &capability_section("Usage", "designed", ""), "");
    assert_hit(&findings, "okf-section-missing", ALPHA, &section("Usage"));
}

#[test]
fn rejects_capability_without_frontmatter() {
    let mut repo = valid();
    repo.insert(
        "docs/capabilities/plain.md".to_string(),
        "# Plain\n\nNo frontmatter here.\n".to_string(),
    );
    assert_hit(
        &check(&repo),
        "stray-markdown",
        "docs/capabilities/plain.md",
        &Location::Line(1),
    );
}

#[test]
fn rejects_unclosed_frontmatter() {
    let findings = mutated(OVERVIEW, "---\n\n# Overview", "# Overview");
    assert_kind(&findings, "okf-parse-unclosed-fence", OVERVIEW);
}

#[test]
fn rejects_proposed_marker_in_architecture() {
    let marker = "> [!PROPOSED change:add-alpha]\n> Not allowed here.";
    let findings = mutated(OVERVIEW, "Body text.", marker);
    assert_hit(
        &findings,
        "okf-proposed-outside-capability",
        OVERVIEW,
        &Location::Line(10),
    );
}

#[test]
fn rejects_proposed_marker_in_change() {
    let marker = "Adds both.\n\n> [!PROPOSED change:add-alpha]\n> Not allowed here.";
    let findings = mutated(CHANGE, "Adds both.", marker);
    assert_hit(
        &findings,
        "okf-proposed-outside-capability",
        CHANGE,
        &Location::Line(21),
    );
}

#[test]
fn rejects_resource_key() {
    // `resource` has its own rule, which is not the unknown-key rule.
    let findings = mutated(
        OVERVIEW,
        "title: Overview",
        "title: Overview\nresource: https://example.invalid/overview",
    );
    assert_hit(&findings, "okf-resource-key", OVERVIEW, &key("resource"));
    assert!(findings.iter().all(|f| f.kind != "okf-unknown-key"));
}

#[test]
fn rejects_nested_documents() {
    for nested in ["docs/capabilities/sub/x.md", "work/changes/sub/y.md"] {
        let mut repo = valid();
        repo.insert(nested.to_string(), "# Nested\n".to_string());
        assert_hit(
            &check(&repo),
            "nested-directory",
            nested,
            &Location::Document,
        );
    }
}

#[test]
fn a_nested_typed_document_is_nested_and_not_read_as_a_document() {
    let mut repo = valid();
    let text = repo[ALPHA].replace("alpha", "deep");
    repo.insert("docs/capabilities/sub/deep.md".to_string(), text);
    let findings = check(&repo);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "nested-directory");
}

#[test]
fn rejects_typed_document_outside_its_directory() {
    // A typed document in the directory of another type, and a document in a
    // directory that is not a document directory, which nothing loads.
    for (path, from) in [
        ("docs/architecture/stray.md", ALPHA),
        ("docs/capabilities/stray.md", CHANGE),
        ("work/changes/stray.md", OVERVIEW),
    ] {
        let mut repo = valid();
        let text = repo[from]
            .replace("alpha", "stray")
            .replace("overview", "stray");
        repo.insert(path.to_string(), text);
        assert_hit(&check(&repo), "wrong-directory", path, &key("type"));
    }
    for path in ["docs/other/stray.md", "work/stray.md", "stray.md"] {
        let mut repo = valid();
        repo.insert(path.to_string(), repo[ALPHA].clone());
        assert_clean(&check(&repo));
    }
}

#[test]
fn an_index_is_navigation_and_not_a_stray_file() {
    let mut repo = valid();
    repo.insert(
        "docs/capabilities/index.md".to_string(),
        "<!-- raf:generated:catalogue -->\n| alpha |\n<!-- raf:generated:end -->\n".to_string(),
    );
    repo.insert(
        "docs/architecture/index.md".to_string(),
        "# Pages\n".to_string(),
    );
    repo.insert(
        "work/changes/index.md".to_string(),
        "# Changes\n".to_string(),
    );
    assert_clean(&check(&repo));
}

#[test]
fn a_generated_index_may_carry_a_licence_header() {
    let header = concat!(
        "<!--\nSPDX-",
        "FileCopyrightText: 2026 Example\nSPDX-",
        "License-Identifier: CC-BY-4.0\n-->\n"
    );
    let mut repo = valid();
    repo.insert(
        "docs/capabilities/index.md".to_string(),
        format!(
            "{header}\n<!-- raf:generated:catalogue -->\n| alpha |\n<!-- raf:generated:end -->\n"
        ),
    );
    assert_clean(&check(&repo));
}

/// Assembled in pieces so that this file carries no SPDX tag of its own here.
const SPDX_ID: &str = concat!("SPDX-License-", "Identifier: CC-BY-4.0");

const CATALOGUE: &str =
    "<!-- raf:generated:catalogue -->\n| alpha |\n\n<!-- raf:generated:end -->\n";

fn with_index(text: &str) -> Vec<raf_workplan::Finding> {
    let mut repo = valid();
    repo.insert("docs/capabilities/index.md".to_string(), text.to_string());
    check(&repo)
}

#[test]
fn accepts_a_generated_capability_index() {
    assert_clean(&with_index(CATALOGUE));
}

#[test]
fn accepts_a_generated_index_with_an_spdx_header() {
    // The header as a comment of several lines, and as a single-line comment.
    let header = format!("<!--\nSPDX-FileCopyrightText: 2026 Example\n{SPDX_ID}\n-->\n\n");
    assert_clean(&with_index(&format!("{header}{CATALOGUE}")));
    assert_clean(&with_index(&format!("<!-- {SPDX_ID} -->\n{CATALOGUE}")));
}

#[test]
fn files_that_are_not_markdown_are_not_documents() {
    let mut repo = valid();
    for path in [
        "docs/capabilities/notes.txt",
        "docs/architecture/diagram.svg",
        "work/changes/draft.yaml",
        "docs/capabilities/sub/data.json",
    ] {
        repo.insert(path.to_string(), String::new());
    }
    assert_clean(&check(&repo));
}

#[test]
fn an_oversize_file_is_reported_by_name_and_not_skipped() {
    let path = "docs/architecture/huge.md";
    let mut repo = valid();
    repo.insert(path.to_string(), "a".repeat(MAX_TEXT_BYTES + 2));
    let findings = check(&repo);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "unreadable-file");
    assert_eq!(findings[0].file, path);
    assert!(
        findings[0].message.contains(path),
        "{}",
        findings[0].message
    );
}

#[test]
fn a_directory_that_cannot_be_listed_is_reported_by_name() {
    let mut repo = Repo::new();
    repo.insert("work/changes".to_string(), "not a directory".to_string());
    let findings = check(&repo);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "unreadable-directory");
    assert_eq!(findings[0].file, "work/changes");
}

#[test]
fn a_tree_that_cannot_be_built_is_a_finding_for_each_directory() {
    // Two keys that name one path make every operation of the tree fail.
    let tree = MemoryTree::from([("docs/a.md", "x"), ("docs//a.md", "y")]);
    let workspace = support::demo_workspace();
    let findings = without_sync_findings(Workplan::load(&tree, &workspace).check());
    let (manifest, directories) = findings.split_last().expect("the diagram manifest finding");
    let files: Vec<&str> = directories.iter().map(|f| f.file.as_str()).collect();
    // The directories in the order of `DocType::ALL`: the documents, then
    // the records.
    assert_eq!(
        files,
        [
            "docs/capabilities",
            "work/changes",
            "docs/architecture",
            "work/evidence",
            "work/assessments",
            "work/coverage",
            "work/debt",
            "work/reviews"
        ]
    );
    assert!(directories.iter().all(|f| f.kind == "unreadable-directory"));
    assert_eq!(manifest.file, "docs/diagrams/manifest.ttl");
    assert_eq!(manifest.kind, "diagram-invalid");
}

#[test]
fn findings_come_in_directory_then_name_order_and_repeat() {
    let mut repo = valid();
    for path in [
        "work/changes/b.md",
        "docs/capabilities/z.md",
        "docs/architecture/m.md",
        "docs/capabilities/a.md",
    ] {
        repo.insert(path.to_string(), "plain\n".to_string());
    }
    let first = check(&repo);
    let files: Vec<&str> = first.iter().map(|f| f.file.as_str()).collect();
    assert_eq!(
        files,
        [
            "docs/capabilities/a.md",
            "docs/capabilities/z.md",
            "work/changes/b.md",
            "docs/architecture/m.md",
        ]
    );
    assert_eq!(first, check(&repo));
}

#[test]
fn a_finding_renders_with_its_place_and_its_fix() {
    let findings = mutated(ALPHA, "state: designed", "state: draft");
    let text = findings[0].to_string();
    assert!(text.starts_with(&format!("{ALPHA}: `state`: ")), "{text}");
    assert!(text.contains("[okf-bad-state]"), "{text}");
    assert!(text.ends_with(&findings[0].fix), "{text}");
}

// --- accepted forms, comments and placeholders ------------------------------

/// Unfinished-work spellings, assembled with `concat!` so this file never
/// holds one as a literal.
const PLACEHOLDER_TOKENS: [&str; 5] = [
    concat!("t", "bd"),
    concat!("to", "do"),
    concat!("fix", "me"),
    concat!("x", "xx"),
    concat!("to be ", "written"),
];

#[test]
fn accepts_quoted_and_block_forms() {
    assert_clean(&mutated(OVERVIEW, "title: Overview", "title: 'It''s'"));
    assert_clean(&mutated(OVERVIEW, "title: Overview", "title: \"a: b\""));
    assert_clean(&mutated(
        ALPHA,
        "depends: [cap:beta]",
        "depends:\n  - cap:beta",
    ));
    assert_clean(&mutated(
        CHANGE,
        "claim: \"alpha reads input\"\n    outcome: implemented",
        "claim: \"a: b\"\n    outcome: dropped",
    ));
    assert_clean(&mutated(
        OVERVIEW,
        "title: Overview",
        "# note\ntitle: Overview\ntags:\n  - one\n  - two",
    ));
}

#[test]
fn accepts_optional_tags_and_timestamp() {
    assert_clean(&mutated(
        OVERVIEW,
        "title: Overview",
        "title: Overview\ntags: [a, b]\ntimestamp: 2026-01-01T00:00:00Z",
    ));
}

#[test]
fn accepts_comments_after_scalars() {
    for (path, from, to) in [
        (OVERVIEW, "title: Overview", "title: Overview # a note"),
        (
            OVERVIEW,
            "title: Overview",
            "title: \"a # b\" # c\ntags: [one, \"two # 2\"] # c",
        ),
        (
            OVERVIEW,
            "description: How it fits.",
            "description: 'It''s # x' # c",
        ),
        (OVERVIEW, "description: How it fits.", "description: a#b"),
        (
            CHANGE,
            "outcome: implemented",
            "outcome: implemented # done",
        ),
        (ALPHA, "state: designed", "state: designed # still planned"),
        (
            ALPHA,
            "depends: [cap:beta]",
            "depends: [cap:beta] # one edge",
        ),
    ] {
        assert_clean(&mutated(path, from, to));
    }
}

#[test]
fn rejects_sections_left_empty_by_comments() {
    for comment in [
        "<!-- later -->\n",
        "<!--\nlater\n-->\n",
        "<!-- a --> <!-- b -->\n",
    ] {
        let findings = mutated(ALPHA, &paragraph("Examples"), comment);
        assert_hit(&findings, "okf-section-empty", ALPHA, &section("Examples"));
    }
    let findings = mutated(CHANGE, "Adds both.\n", "<!-- soon -->\n");
    assert_hit(&findings, "okf-section-empty", CHANGE, &section("Summary"));
    let findings = mutated(
        OVERVIEW,
        "# Overview\n\nBody text.\n",
        "<!-- nothing yet -->\n",
    );
    assert_hit(&findings, "okf-body-empty", OVERVIEW, &Location::Document);
}

#[test]
fn comments_hide_text_but_not_the_text_beside_them() {
    assert_clean(&mutated(
        ALPHA,
        &paragraph("Examples"),
        "<!-- x -->\ntext\n",
    ));
    assert_clean(&mutated(ALPHA, &paragraph("Examples"), "a <!-- x --> b\n"));
    let findings = mutated(ALPHA, &paragraph("Examples"), "  <!-- x -->\n  \n");
    assert_hit(&findings, "okf-section-empty", ALPHA, &section("Examples"));
}

#[test]
fn rejects_placeholder_tokens() {
    for token in PLACEHOLDER_TOKENS {
        let findings = mutated(
            OVERVIEW,
            "Body text.",
            &format!("Body text. Still {token} here."),
        );
        assert_kind(&findings, "okf-placeholder", OVERVIEW);
    }
    let shouted = concat!("Left TO", "DO: finish");
    assert_kind(
        &mutated(ALPHA, "Text for Usage.", shouted),
        "okf-placeholder",
        ALPHA,
    );
    let mixed_case = concat!("It is To Be ", "Written.");
    assert_kind(
        &mutated(ALPHA, "Text for Usage.", mixed_case),
        "okf-placeholder",
        ALPHA,
    );
}

#[test]
fn rejects_placeholder_spellings() {
    for spelling in [
        concat!("t", "bd"),
        concat!("T.B", ".D."),
        concat!("t.b", ".d"),
        concat!("To", "Do"),
        concat!("fix", "Me"),
        concat!("Fix", "ME:"),
        concat!("x", "xX"),
        concat!("to  be", "\twritten"),
        concat!("TO BE ", "WRITTEN"),
    ] {
        let findings = mutated(
            OVERVIEW,
            "Body text.",
            &format!("Body text. Still {spelling} soon."),
        );
        assert_kind(&findings, "okf-placeholder", OVERVIEW);
    }
}

#[test]
fn placeholder_scan_spares_code_and_longer_words() {
    let token = concat!("T", "BD");
    let fenced = format!("Body text.\n\n```text\n{token}\n```\n\nUse `{token}` and {token}x.\n");
    assert_clean(&mutated(OVERVIEW, "Body text.\n", &fenced));
}

#[test]
fn placeholder_scan_spares_longer_words_and_dotted_prose() {
    let text = format!(
        "Body text. {} and {} and e.g. a.b.c, {}s.\n",
        concat!("t", "bdx"),
        concat!("to", "dos"),
        concat!("x", "xxx"),
    );
    assert_clean(&mutated(OVERVIEW, "Body text.\n", &text));
}
