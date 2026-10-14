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
    ALPHA, BETA, CHANGE, GAMMA, OVERVIEW, capability, change_record, mutated, valid,
};
use support::{Repo, assert_cases, assert_clean, assert_hit, assert_kind, check, key, section};

#[test]
fn accepts_valid_fixture() {
    let repo = valid();
    let tree: MemoryTree = repo.iter().collect();
    let workspace = support::demo_workspace();
    let plan = Workplan::load(&tree, &workspace);
    assert_clean(plan.findings());
    assert_clean(&plan.check());
    assert_eq!(plan.capabilities().len(), 3);
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
    assert_eq!(paths, [ALPHA, BETA, GAMMA]);
    let states: Vec<State> = plan
        .capabilities()
        .iter()
        .map(|c| c.value().state)
        .collect();
    assert_eq!(
        states,
        [State::Designed, State::Considered, State::Delivered]
    );
    let gamma = &plan.capabilities()[2];
    assert_eq!(gamma.value().header.id, "cap:gamma");
    assert_eq!(gamma.value().bindings.code, ["src/lib.rs"]);
    assert_eq!(gamma.value().depends, ["cap:beta", "cap:alpha"]);
    let headings: Vec<&str> = gamma
        .body()
        .sections()
        .iter()
        .map(Section::heading)
        .collect();
    assert_eq!(headings, support::fixtures::SECTIONS);
    let beta = &plan.capabilities()[1];
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
    assert_eq!(plan.capabilities().len(), 2);
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
    let findings = mutated(ALPHA, "## Usage\n\nText for Usage.\n\n", "");
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
    let findings = Workplan::load(&tree, &workspace).check();
    let files: Vec<&str> = findings.iter().map(|f| f.file.as_str()).collect();
    assert_eq!(
        files,
        ["docs/capabilities", "work/changes", "docs/architecture"]
    );
    assert!(findings.iter().all(|f| f.kind == "unreadable-directory"));
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
