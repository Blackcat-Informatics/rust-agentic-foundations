// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The debt record and the ratchet of strict mode, on repositories on a disk:
//! the census fixture set mapped one to one (growth fails and the write
//! refuses, shrinkage fails until the write, no record means every finding
//! stands and the write creates it, a finding of the default rules stops the
//! write, two writes are byte-identical, the write writes only the record),
//! and the rows of the record judged by the rules of the entries.

mod support;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use raf_okf::{DebtLine, DebtRecord, Location, RafDocument};
use raf_source::{MemoryTree, RelPath, Tree, Workspace};
use raf_workplan::debt::{DEBT_KINDS, DebtChange, DebtError};
use raf_workplan::record::RecordOptions;
use raf_workplan::{Finding, Gate, Workplan};
use support::cases::debt_record;
use support::disk::{Scratch, changed, put, read, snapshot};
use support::strict;
use support::synced::options;

fn memory(root: &Path) -> MemoryTree {
    MemoryTree::try_from_iter(
        snapshot(root)
            .into_iter()
            .map(|(path, bytes)| (path, String::from_utf8(bytes).unwrap())),
    )
    .unwrap()
}

fn write_memory(tree: &mut MemoryTree) -> Result<raf_workplan::debt::DebtWrite, DebtError> {
    let source = Workspace::load(tree, &["."]).unwrap();
    Workplan::load(tree, &source).write_debt_in(tree, &RecordOptions::new())
}

const DEBT: &str = "work/debt/default.md";

/// The coverage record with `strictness: strict` and `extra` after its scope
/// and exclusions.
fn coverage(strictness: &str, extra: &str) -> String {
    format!(
        "---\ntype: Coverage\nid: coverage:default\n{strictness}scope: []\nexclusions: []\n{extra}---\n"
    )
}

/// A repository on a disk: the strict-clean repository with the capability
/// `capability`, the library `library` and the coverage record `record`,
/// synced.
fn repository(tag: &str, capability: &str, library: &str, record: &str) -> Scratch {
    let scratch = Scratch::new(tag);
    let root = scratch.path();
    for (path, text) in strict::source_files() {
        put(root, path, &text);
    }
    put(root, strict::LIB, library);
    set(root, capability);
    put(root, strict::COVERAGE, record);
    sync(root);
    scratch
}

/// Sync the generated blocks, creating the index.
fn sync(root: &Path) {
    let opened = Gate::workspace(root).open().expect("a repository");
    let report = opened.workplan().sync(&options()).expect("a sync");
    Workplan::write(&report, root).expect("the sync is written");
}

/// Replace the capability document and sync its generated blocks.
fn set(root: &Path, capability: &str) {
    put(root, strict::CAPABILITY, capability);
    sync(root);
}

fn strict_record() -> String {
    coverage("", "")
}

/// The capability with the sections `sections` made thin.
fn thin(sections: &[&str]) -> String {
    let mut text = strict::capability();
    for section in sections {
        text = strict::replaced(
            &text,
            &format!("## {section}\n\n{}\n", strict::PROSE),
            &format!(
                "## {section}\n\n{}\n",
                strict::words(19 - extra_words(section))
            ),
        );
    }
    text
}

fn extra_words(section: &str) -> usize {
    match section {
        "Usage" => 2,
        "Constraints" => 5,
        _ => 0,
    }
}

/// The library with the function `name` holding `body` lines after `run`.
fn library_with(functions: &[(&str, usize)]) -> String {
    let mut text = strict::LIBRARY.to_string();
    for (name, body) in functions {
        let _ = write!(text, "\nfn {name}() {{\n");
        for _ in 0..*body {
            text.push_str("    let _ = 1;\n");
        }
        text.push_str("}\n");
    }
    text
}

fn gate(root: &Path) -> Gate {
    Gate::workspace(root)
}

fn found(root: &Path) -> Vec<Finding> {
    gate(root).check()
}

fn kinds(root: &Path) -> Vec<&'static str> {
    found(root).iter().map(|finding| finding.kind).collect()
}

/// The thin sections Examples and Explanation under a strict record.
fn two_thin() -> Scratch {
    repository(
        "debt-two",
        &thin(&["Examples", "Explanation"]),
        strict::LIBRARY,
        &strict_record(),
    )
}

#[test]
fn without_a_record_every_strict_finding_stands_and_the_write_creates_it() {
    let repo = two_thin();
    let root = repo.path();
    let before = snapshot(root);
    assert!(!root.join(DEBT).exists());
    for _ in 0..2 {
        assert_eq!(kinds(root), ["strict-section-thin", "strict-section-thin"]);
        assert_eq!(snapshot(root), before, "ordinary checks do not adopt debt");
        assert!(!root.join(DEBT).exists());
    }
    let written = gate(root).write_debt().expect("the adoption writes");
    assert_eq!(written.change(), DebtChange::Created);
    assert_eq!(written.path(), DEBT);
    assert_eq!((written.findings(), written.lines()), (2, 0));
    // The write wrote the record and nothing else.
    assert_eq!(changed(&before, &snapshot(root)), [DEBT]);
    assert_eq!(
        read(root, DEBT),
        "---\ntype: Debt\nid: debt:default\nfindings:\n  \
         - kind: strict-section-thin\n    subject: cap:demo#examples\n  \
         - kind: strict-section-thin\n    subject: cap:demo#explanation\nlines: []\n---\n"
    );
    // The repository then passes, and the record loads as the typed value.
    assert_eq!(found(root), []);
    let opened = gate(root).open().expect("opens");
    let plan = opened.workplan();
    let record = plan.debt().expect("the record loads");
    assert_eq!(record.path(), DEBT);
    assert_eq!(record.value().findings.len(), 2);
    assert_eq!(record.value().findings[0].subject, "cap:demo#examples");
}

/// The strict-clean capability with seven rules of strict mode broken.
fn broken() -> String {
    let mut text = strict::capability();
    for (from, to) in [
        // A Contract of forty words with no clause start and no marker.
        (
            format!("## Contract\n\n{}\n{}\n", strict::PROSE, strict::MARKER),
            format!("## Contract\n\n{} {}\n", strict::PROSE, strict::PROSE),
        ),
        (strict::EXAMPLES.to_string(), "  examples: []\n".to_string()),
        (format!("\n{}", strict::MATRIX), String::new()),
        (
            strict::LIMITATION.to_string(),
            "One limitation of the demo.".to_string(),
        ),
        (
            format!("## Purpose\n\n{}\n", strict::PROSE),
            format!("## Purpose\n\n{}\n", strict::words(19)),
        ),
    ] {
        text = strict::replaced(&text, &from, &to);
    }
    text
}

#[test]
fn every_kind_a_row_can_hold_is_written_with_a_subject_of_its_form_and_then_holds() {
    let repo = repository(
        "debt-kinds",
        &broken(),
        &format!(
            "{}\npub fn extra() {{}}\n\nmod made {{\n    macro_rules! make {{\n        () => {{\n            pub struct Made;\n        }};\n    }}\n    make!();\n}}\npub use made::Made;\n",
            strict::LIBRARY
        ),
        &strict_record(),
    );
    let root = repo.path();
    put(
        root,
        "tests/it.rs",
        &format!("{}\n#[test]\nfn other() {{}}\n", strict::TEST),
    );
    let mut present: Vec<&str> = kinds(root);
    present.sort_unstable();
    present.dedup();
    let wanted = [
        "coverage-item-unbound",
        "coverage-lines-over-ceiling",
        "coverage-reexport-unbound",
        "coverage-test-unbound",
        "strict-claim-uncovered",
        "strict-claim-unit-collapsed",
        "strict-example-missing",
        "strict-limitations-missing",
        "strict-matrix-missing",
        "strict-section-thin",
        "strict-usage-unbound",
    ];
    assert_eq!(present, wanted);
    gate(root).write_debt().expect("adopts");
    // No row is invalid, and the repository passes with every one of them.
    assert_eq!(found(root), []);
    let opened = gate(root).open().expect("opens");
    let plan = opened.workplan();
    let record = plan.debt().expect("the record");
    let mut recorded: Vec<(&str, &str)> = record
        .value()
        .findings
        .iter()
        .map(|row| (row.kind.as_str(), row.subject.as_str()))
        .collect();
    recorded.sort_unstable();
    let rows: Vec<String> = recorded
        .iter()
        .map(|(kind, subject)| format!("{kind} {subject}"))
        .collect();
    assert_eq!(
        rows,
        [
            "coverage-item-unbound demo::extra",
            "coverage-reexport-unbound demo::Made",
            "coverage-test-unbound demo::test_crate::it::other",
            "strict-claim-uncovered cap:demo#contract/opening",
            "strict-claim-unit-collapsed cap:demo#contract",
            "strict-example-missing cap:demo",
            "strict-limitations-missing cap:demo#constraints",
            "strict-matrix-missing cap:demo#verification",
            "strict-section-thin cap:demo#purpose",
            "strict-usage-unbound cap:demo#usage",
        ]
    );
}

#[test]
fn a_repository_that_passes_has_no_finding_to_hold_and_the_write_records_nothing() {
    let repo = repository(
        "debt-clean",
        &strict::capability(),
        strict::LIBRARY,
        &strict_record(),
    );
    let root = repo.path();
    assert_eq!(found(root), []);
    let written = gate(root).write_debt().expect("an empty record");
    assert_eq!(written.change(), DebtChange::Created);
    assert_eq!((written.findings(), written.lines()), (0, 0));
    assert_eq!(
        read(root, DEBT),
        "---\ntype: Debt\nid: debt:default\nfindings: []\nlines: []\n---\n"
    );
    assert_eq!(found(root), []);
}

#[test]
fn a_new_strict_finding_fails_and_the_write_refuses_and_writes_nothing() {
    let repo = two_thin();
    let root = repo.path();
    gate(root).write_debt().expect("the adoption writes");
    // The neighbour: the capability as recorded passes.
    assert_eq!(found(root), []);
    set(root, &thin(&["Examples", "Explanation", "Purpose"]));
    let findings = found(root);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "strict-section-thin");
    assert_eq!(
        findings[0].location,
        Location::Section("Purpose".to_string())
    );
    let before = snapshot(root);
    let mut memory = memory(root);
    let before_memory = memory.clone();
    let Err(DebtError::Growth(memory_growth)) = write_memory(&mut memory) else {
        panic!("memory publication must refuse the same debt growth");
    };
    assert_eq!(memory_growth, findings);
    assert_eq!(memory, before_memory);
    let Err(DebtError::Growth(growth)) = gate(root).write_debt() else {
        panic!("the write must refuse growth");
    };
    assert_eq!(growth, findings);
    assert_eq!(snapshot(root), before);
    let refused = DebtError::Growth(growth).to_string();
    assert!(
        refused.contains("would grow by 1 finding(s)") && refused.contains("section `Purpose`"),
        "{refused}"
    );
}

#[test]
fn rewording_a_recorded_constraint_is_growth_beside_its_resolved_old_key() {
    let capability = strict::replaced(
        &strict::capability(),
        &format!("{}\n{}", strict::LIMITATION, strict::MARKER),
        strict::LIMITATION,
    );
    let repo = repository(
        "debt-claim-identity",
        &capability,
        strict::LIBRARY,
        &strict_record(),
    );
    let root = repo.path();
    let key = |root: &Path| {
        let opened = gate(root).open().expect("the repository opens");
        let report = opened.workplan().claims_report();
        let uncovered: Vec<_> = report
            .claims()
            .iter()
            .filter(|claim| !claim.is_covered())
            .collect();
        assert_eq!(uncovered.len(), 1);
        uncovered[0].key()
    };
    let original = key(root);
    gate(root)
        .write_debt()
        .expect("the explicit initial record");
    assert_eq!(found(root), []);
    for ending in [" More detail.", "!"] {
        set(
            root,
            &capability.replace(
                strict::LIMITATION,
                &format!("{}{ending}", strict::LIMITATION),
            ),
        );
        let current = key(root);
        assert_ne!(original, current);
        let findings = found(root);
        let mut kinds: Vec<_> = findings.iter().map(|finding| finding.kind).collect();
        kinds.sort_unstable();
        assert_eq!(
            kinds,
            ["debt-resolved", "strict-claim-uncovered"],
            "{findings:#?}"
        );
        for wanted in [&original, &current] {
            assert!(
                findings
                    .iter()
                    .any(|finding| finding.message.contains(wanted)),
                "{wanted}: {findings:#?}"
            );
        }
        let before = snapshot(root);
        let Err(DebtError::Growth(growth)) = gate(root).write_debt() else {
            panic!("a changed claim cannot silently replace its recorded identity");
        };
        assert_eq!(growth.len(), 1);
        assert_eq!(growth[0].kind, "strict-claim-uncovered");
        assert!(growth[0].message.contains(&current));
        assert_eq!(snapshot(root), before);
        set(root, &capability);
        assert_eq!(found(root), [], "restoring the recorded claim still holds");
    }
}

#[test]
fn a_paid_finding_fails_until_the_write_and_then_the_record_equals_the_present_debt() {
    let repo = two_thin();
    let root = repo.path();
    gate(root).write_debt().expect("the adoption writes");
    set(root, &thin(&["Explanation"]));
    let mut memory = memory(root);
    let findings = found(root);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "debt-resolved");
    assert_eq!(findings[0].file, DEBT);
    assert_eq!(findings[0].location, Location::Key("findings".to_string()));
    assert!(
        findings[0].message.contains("`cap:demo#examples`")
            && findings[0].fix.contains("cargo workplan debt --write"),
        "{:#?}",
        findings[0]
    );
    let written = gate(root).write_debt().expect("the record is lowered");
    assert_eq!(
        write_memory(&mut memory).unwrap().change(),
        DebtChange::Replaced
    );
    assert_eq!(
        memory.read(&RelPath::new(DEBT).unwrap(), u64::MAX).unwrap(),
        read(root, DEBT)
    );
    assert_eq!(written.change(), DebtChange::Replaced);
    assert_eq!(written.findings(), 1);
    assert_eq!(found(root), []);
    // The record equals the present debt: an adoption of the same state
    // writes the same bytes.
    let fresh = repository(
        "debt-fresh",
        &thin(&["Explanation"]),
        strict::LIBRARY,
        &strict_record(),
    );
    gate(fresh.path()).write_debt().expect("an adoption");
    assert_eq!(read(root, DEBT), read(fresh.path(), DEBT));
}

#[test]
fn two_writes_are_byte_identical_and_the_second_leaves_the_file_untouched() {
    let repo = two_thin();
    let root = repo.path();
    let mut memory = memory(root);
    assert_eq!(
        write_memory(&mut memory).unwrap().change(),
        DebtChange::Created
    );
    gate(root).write_debt().expect("the first write");
    let first = snapshot(root);
    assert_eq!(
        memory.read(&RelPath::new(DEBT).unwrap(), u64::MAX).unwrap(),
        read(root, DEBT)
    );
    let unchanged = memory.clone();
    assert_eq!(
        write_memory(&mut memory).unwrap().change(),
        DebtChange::Unchanged
    );
    assert_eq!(memory, unchanged);
    assert_eq!(found(root), []);
    assert_eq!(
        snapshot(root),
        first,
        "checking existing debt does not rewrite it"
    );
    let second = gate(root).write_debt().expect("the second write");
    assert_eq!(second.change(), DebtChange::Unchanged);
    assert_eq!(second.change().name(), "unchanged");
    assert_eq!(snapshot(root), first);
}

#[test]
fn a_finding_of_the_default_rules_stops_the_write() {
    let repo = two_thin();
    let root = repo.path();
    // The verification matrix is stale: a finding of the default rules.
    put(
        root,
        strict::CAPABILITY,
        &strict::replaced(
            &thin(&["Examples", "Explanation"]),
            strict::MATRIX,
            "<!-- raf:generated:verification-matrix -->\nstale\n<!-- raf:generated:end -->\n",
        ),
    );
    let before = snapshot(root);
    let Err(DebtError::Findings(blocking)) = gate(root).write_debt() else {
        panic!("a default finding must stop the write");
    };
    assert!(
        blocking
            .iter()
            .any(|finding| finding.kind == "generated-stale"),
        "{blocking:#?}"
    );
    assert!(
        blocking
            .iter()
            .all(|finding| !finding.kind.starts_with("strict-")),
        "{blocking:#?}"
    );
    assert_eq!(snapshot(root), before);
    // The neighbour: once synced, the same repository is adopted.
    sync(root);
    assert_eq!(
        gate(root).write_debt().expect("adopts").change(),
        DebtChange::Created
    );
}

#[test]
fn the_default_gate_records_present_debt_without_a_policy_switch() {
    let repo = repository(
        "debt-default",
        &thin(&["Examples"]),
        strict::LIBRARY,
        &coverage("", ""),
    );
    let written = gate(repo.path()).write_debt().expect("adopts");
    assert_eq!(written.change(), DebtChange::Created);
    assert_eq!(written.findings(), 1);
    assert_eq!(found(repo.path()), []);
}

#[test]
fn an_empty_debt_record_is_judged_without_a_policy_switch() {
    let repo = repository(
        "debt-empty",
        &strict::capability(),
        strict::LIBRARY,
        &coverage("", ""),
    );
    put(repo.path(), DEBT, &debt_record(" []", " []"));
    assert_eq!(found(repo.path()), []);
}

/// The kinds a debt record holds and a subject of the form of each, with a
/// subject that is not.
const SUBJECTS: [(&str, &str, &str); 10] = [
    ("strict-example-missing", "cap:demo", "cap:demo#usage"),
    (
        "strict-matrix-missing",
        "cap:demo#verification",
        "cap:demo#usage",
    ),
    ("strict-usage-unbound", "cap:demo#usage", "cap:demo"),
    (
        "strict-limitations-missing",
        "cap:demo#constraints",
        "cap:demo#bogus",
    ),
    (
        "strict-claim-unit-collapsed",
        "cap:demo#contract",
        "cap:demo#contract/opening",
    ),
    ("strict-section-thin", "cap:demo#examples", "demo#examples"),
    (
        "strict-claim-uncovered",
        "cap:demo#contract/opening",
        "cap:demo#contract",
    ),
    ("coverage-item-unbound", "demo::run", "run"),
    ("coverage-reexport-unbound", "demo::shown", "demo:: shown"),
    ("coverage-test-unbound", "demo::tests::case", "::case"),
];

fn row(kind: &str, subject: &str) -> String {
    format!("\n  - kind: {kind}\n    subject: \"{subject}\"")
}

fn invalid_in(root: &Path, record: &str) -> Vec<Finding> {
    put(root, DEBT, record);
    found(root)
        .into_iter()
        .filter(|finding| finding.kind == "debt-entry-invalid")
        .collect()
}

#[test]
fn a_row_of_a_kind_that_is_no_debt_or_a_subject_of_another_form_is_invalid() {
    let repo = repository(
        "debt-invalid",
        &strict::capability(),
        strict::LIBRARY,
        &strict_record(),
    );
    let root = repo.path();
    assert_eq!(DEBT_KINDS.len(), SUBJECTS.len());
    for (kind, fits, other) in SUBJECTS {
        assert!(DEBT_KINDS.contains(&kind), "{kind}");
        // The neighbour: a row of the form is no invalid entry (it is paid,
        // which is another finding).
        assert_eq!(invalid_in(root, &debt_record(&row(kind, fits), " []")), []);
        let invalid = invalid_in(root, &debt_record(&row(kind, other), " []"));
        assert_eq!(invalid.len(), 1, "{kind} {other}: {invalid:#?}");
        assert_eq!(invalid[0].file, DEBT);
        assert_eq!(invalid[0].location, Location::Key("findings".to_string()));
        assert!(invalid[0].message.contains(other), "{}", invalid[0].message);
    }
    for kind in [
        "link-broken",
        "coverage-lines-over-ceiling",
        "coverage-surface-unreadable",
        "section-untouched",
        "debt-resolved",
        "nonsense",
    ] {
        let invalid = invalid_in(root, &debt_record(&row(kind, "cap:demo"), " []"));
        assert_eq!(invalid.len(), 1, "{kind}: {invalid:#?}");
        assert!(
            invalid[0]
                .message
                .contains("is not a finding a debt record can hold"),
            "{}",
            invalid[0].message
        );
    }
}

#[test]
fn a_row_of_lines_that_names_no_item_or_no_line_is_invalid() {
    let repo = repository(
        "debt-invalid-lines",
        &strict::capability(),
        strict::LIBRARY,
        &strict_record(),
    );
    let root = repo.path();
    let lines = |item: &str, orphans: &str| {
        debt_record(
            " []",
            &format!("\n  - item: \"{item}\"\n    orphans: {orphans}"),
        )
    };
    // The neighbour: a row of an item that holds lines is no invalid entry.
    assert_eq!(invalid_in(root, &lines("demo::helper", "3")), []);
    for (item, orphans) in [
        ("helper", "3"),
        ("::helper", "3"),
        ("demo::", "3"),
        (" demo::helper", "3"),
        ("demo::helper", "0"),
    ] {
        let invalid = invalid_in(root, &lines(item, orphans));
        assert_eq!(invalid.len(), 1, "{item} {orphans}: {invalid:#?}");
        assert_eq!(invalid[0].location, Location::Key("lines".to_string()));
    }
}

#[test]
fn an_invalid_debt_row_is_refused_without_a_policy_selector() {
    let repo = repository(
        "debt-invalid-default",
        &strict::capability(),
        strict::LIBRARY,
        &coverage("", ""),
    );
    let root = repo.path();
    put(
        root,
        DEBT,
        &debt_record(&row("link-broken", "cap:demo"), " []"),
    );
    // The record is not judged, so its rows are not resolved; its invalid row
    // and the record itself are findings.
    assert_eq!(kinds(root), ["debt-entry-invalid"]);
    put(root, strict::COVERAGE, &strict_record());
    assert_eq!(kinds(root), ["debt-entry-invalid"]);
}

#[test]
fn an_invalid_row_holds_nothing_so_the_write_replaces_it() {
    let repo = repository(
        "debt-invalid-replaced",
        &strict::capability(),
        strict::LIBRARY,
        &strict_record(),
    );
    let root = repo.path();
    put(
        root,
        DEBT,
        &debt_record(&row("link-broken", "cap:demo"), " []"),
    );
    let written = gate(root).write_debt().expect("the record is replaced");
    assert_eq!(written.change(), DebtChange::Replaced);
    assert_eq!(found(root), []);
}

#[test]
fn a_file_other_than_the_one_record_is_a_finding() {
    let repo = repository(
        "debt-other",
        &strict::capability(),
        strict::LIBRARY,
        &strict_record(),
    );
    let root = repo.path();
    put(root, "work/debt/other.md", &debt_record(" []", " []"));
    let findings = found(root);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "singleton-other-file");
    assert_eq!(findings[0].file, "work/debt/other.md");
}

#[test]
fn a_record_that_breaks_the_profile_is_the_finding_of_its_rule_and_stops_the_write() {
    let repo = repository(
        "debt-broken",
        &strict::capability(),
        strict::LIBRARY,
        &strict_record(),
    );
    let root = repo.path();
    put(
        root,
        DEBT,
        "---\ntype: Debt\nid: debt:default\nfindings: []\nlines: []\nextra: 1\n---\n",
    );
    let findings = found(root);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "okf-unknown-key");
    assert!(matches!(
        gate(root).write_debt(),
        Err(DebtError::Findings(_))
    ));
    put(
        root,
        DEBT,
        "---\ntype: Debt\nid: debt:other\nfindings: []\nlines: []\n---\n",
    );
    let findings = found(root);
    assert!(
        findings
            .iter()
            .any(|finding| finding.kind == "okf-singleton-id"
                && finding.fix == "name the record `debt:default` at work/debt/default.md"),
        "{findings:#?}"
    );
}

/// A package over its ceiling of 0 lines with a helper of three lines.
fn orphan_repo(tag: &str, functions: &[(&str, usize)], extra: &str) -> Scratch {
    repository(
        tag,
        &strict::capability(),
        &library_with(functions),
        &coverage("", extra),
    )
}

fn lines_of(root: &Path) -> Vec<(String, usize)> {
    let opened = Gate::workspace(root).open().expect("opens");
    let plan = opened.workplan();
    plan.debt().map_or_else(Vec::new, |record| {
        record
            .value()
            .lines
            .iter()
            .map(|line| (line.item.clone(), line.orphans))
            .collect()
    })
}

#[test]
fn orphan_lines_over_a_ceiling_are_held_per_item_as_a_count() {
    let repo = orphan_repo("debt-lines", &[("helper", 1)], "");
    let root = repo.path();
    // No record: the package is over its ceiling of 0 lines.
    let findings = found(root);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "coverage-lines-over-ceiling");
    assert_eq!(findings[0].file, strict::COVERAGE);
    let before = snapshot(root);
    let written = gate(root).write_debt().expect("adopts");
    assert_eq!(written.change(), DebtChange::Created);
    assert_eq!((written.findings(), written.lines()), (0, 1));
    assert_eq!(changed(&before, &snapshot(root)), [DEBT]);
    assert_eq!(lines_of(root), [("demo::helper".to_string(), 3)]);
    assert_eq!(found(root), []);
    // The column of the report is the lines the record holds.
    let opened = gate(root).open().expect("opens");
    let plan = opened.workplan();
    let report = plan.lines_report();
    assert_eq!(report.crates()[0].orphan(), 3);
    assert_eq!(report.crates()[0].debt(), 3);
    assert!(report.to_string().contains("| 0 lines | 3 |"), "{report}");
    // Without the record the column is 0.
    let none = orphan_repo("debt-lines-none", &[("helper", 1)], "");
    let opened = gate(none.path()).open().expect("opens");
    let plan = opened.workplan();
    assert_eq!(plan.lines_report().crates()[0].debt(), 0);
}

#[test]
fn an_item_that_holds_more_lines_than_its_row_fails_naming_the_row_and_the_write_refuses() {
    let repo = orphan_repo("debt-lines-grow", &[("helper", 1)], "");
    let root = repo.path();
    gate(root).write_debt().expect("adopts");
    put(root, strict::LIB, &library_with(&[("helper", 2)]));
    let findings = found(root);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "coverage-lines-over-ceiling");
    assert_eq!(findings[0].file, DEBT);
    assert_eq!(findings[0].location, Location::Key("lines".to_string()));
    assert!(
        findings[0].message.contains("`demo::helper`")
            && findings[0]
                .message
                .contains("holds 4 orphan lines, over the 3 orphan lines"),
        "{}",
        findings[0].message
    );
    let before = snapshot(root);
    let Err(DebtError::Growth(growth)) = gate(root).write_debt() else {
        panic!("a row that grew is growth");
    };
    assert_eq!(growth, findings);
    assert_eq!(snapshot(root), before);
}

#[test]
fn an_orphan_item_with_no_row_is_growth_in_a_package_judged_by_its_rows() {
    let repo = orphan_repo("debt-lines-new", &[("helper", 1)], "");
    let root = repo.path();
    gate(root).write_debt().expect("adopts");
    put(
        root,
        strict::LIB,
        &library_with(&[("helper", 1), ("other", 1)]),
    );
    let findings = found(root);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "coverage-lines-over-ceiling");
    assert!(
        findings[0].message.contains("`demo::other`")
            && findings[0].message.contains("has no row for it"),
        "{}",
        findings[0].message
    );
    assert!(matches!(gate(root).write_debt(), Err(DebtError::Growth(_))));
    assert_eq!(lines_of(root), [("demo::helper".to_string(), 3)]);
}

#[test]
fn a_package_over_a_ceiling_that_the_record_does_not_name_is_growth_not_adoption() {
    // The record exists and holds findings only: the package newly over its
    // ceiling is debt that grew.
    let repo = orphan_repo("debt-lines-package", &[], "");
    let root = repo.path();
    gate(root).write_debt().expect("adopts");
    put(root, strict::LIB, &library_with(&[("helper", 1)]));
    let findings = found(root);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, strict::COVERAGE);
    assert!(matches!(
        gate(root).write_debt(),
        Err(DebtError::Growth(growth)) if growth == findings
    ));
}

#[test]
fn a_falling_count_fails_until_the_write_lowers_the_row() {
    let repo = orphan_repo("debt-lines-fall", &[("helper", 1)], "");
    let root = repo.path();
    gate(root).write_debt().expect("adopts");
    put(root, strict::LIB, &library_with(&[("helper", 0)]));
    let findings = found(root);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "debt-resolved");
    assert_eq!(findings[0].location, Location::Key("lines".to_string()));
    assert!(
        findings[0]
            .message
            .contains("3 orphan lines of `demo::helper`")
            && findings[0].message.contains("now holds 2 orphan lines"),
        "{}",
        findings[0].message
    );
    // The debt column counts the fewer of the lines and the row.
    let opened = gate(root).open().expect("opens");
    assert_eq!(opened.workplan().lines_report().crates()[0].debt(), 2);
    drop(opened);
    let written = gate(root).write_debt().expect("lowers");
    assert_eq!(written.change(), DebtChange::Replaced);
    assert_eq!(lines_of(root), [("demo::helper".to_string(), 2)]);
    assert_eq!(found(root), []);
}

#[test]
fn an_item_that_is_gone_is_resolved_and_the_write_drops_its_row() {
    let repo = orphan_repo("debt-lines-gone", &[("helper", 1)], "");
    let root = repo.path();
    gate(root).write_debt().expect("adopts");
    put(root, strict::LIB, strict::LIBRARY);
    let findings = found(root);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "debt-resolved");
    assert!(
        findings[0].message.contains("holds no orphan line"),
        "{}",
        findings[0].message
    );
    gate(root).write_debt().expect("lowers");
    assert_eq!(lines_of(root), []);
    assert_eq!(found(root), []);
}

#[test]
fn a_count_back_under_the_ceiling_is_resolved_and_a_package_within_it_holds_no_debt() {
    let repo = orphan_repo("debt-lines-ceiling", &[("helper", 1)], "");
    let root = repo.path();
    gate(root).write_debt().expect("adopts");
    // The ceiling is raised over the lines: the package holds no debt.
    put(
        root,
        strict::COVERAGE,
        &coverage("", "ceilings:\n  - crate: demo\n    lines: 10\n"),
    );
    let findings = found(root);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "debt-resolved");
    assert!(
        findings[0]
            .message
            .contains("`demo` is within its ceilings"),
        "{}",
        findings[0].message
    );
    let written = gate(root).write_debt().expect("lowers");
    assert_eq!((written.findings(), written.lines()), (0, 0));
    assert_eq!(found(root), []);
    // The boundary: a ceiling of exactly the orphan lines holds, one under
    // does not, and then the package is judged by its ceiling again.
    put(
        root,
        strict::COVERAGE,
        &coverage("", "ceilings:\n  - crate: demo\n    lines: 2\n"),
    );
    let findings = found(root);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, "coverage-lines-over-ceiling");
    assert_eq!(findings[0].file, strict::COVERAGE);
}

#[test]
fn the_record_keeps_its_spdx_lines_and_a_new_one_takes_those_it_is_given() {
    let repo = two_thin();
    let root = repo.path();
    let options = RecordOptions::new()
        .copyright("2026 Example Org <dev@example.org>")
        .licence("CC-BY-4.0");
    let written = gate(root)
        .record_options(options.clone())
        .write_debt()
        .expect("adopts");
    assert_eq!(written.change(), DebtChange::Created);
    let text = read(root, DEBT);
    assert!(
        text.starts_with("---\n# SPDX-FileCopyrightText: 2026 Example Org <dev@example.org>\n# SPDX-License-Identifier: CC-BY-4.0\ntype: Debt\n"),
        "{text}"
    );
    // Lowered without options, the record keeps its lines.
    set(root, &thin(&["Explanation"]));
    let written = gate(root).write_debt().expect("lowers");
    assert_eq!(written.change(), DebtChange::Replaced);
    let lowered = read(root, DEBT);
    assert!(
        lowered.starts_with(&text[..text.find("type:").expect("a type")]),
        "{lowered}"
    );
    assert!(!lowered.contains("cap:demo#examples"));
    // Given others, they replace them.
    let other = RecordOptions::new().copyright("2027 Other").licence("MIT");
    set(root, &strict::capability());
    gate(root)
        .record_options(other)
        .write_debt()
        .expect("lowers");
    assert!(read(root, DEBT).contains("# SPDX-FileCopyrightText: 2027 Other\n"));
    // One of the two is refused, and nothing is written.
    let before = snapshot(root);
    let half = RecordOptions::new().copyright("2027 Other");
    set(root, &thin(&["Purpose"]));
    let Err(DebtError::Growth(_)) = gate(root).record_options(half.clone()).write_debt() else {
        panic!("new debt is refused before the options are read");
    };
    set(root, &strict::capability());
    let Err(DebtError::Record(error)) = gate(root).record_options(half).write_debt() else {
        panic!("one SPDX line is refused");
    };
    assert!(error.to_string().contains("licence"), "{error}");
    assert_eq!(read(root, DEBT), before_text(&before));
}

/// The text of the record in a snapshot.
fn before_text(snapshot: &BTreeMap<String, Vec<u8>>) -> String {
    String::from_utf8(snapshot[DEBT].clone()).expect("the record is text")
}

#[test]
fn every_error_of_the_write_says_what_to_do() {
    let repo = two_thin();
    let findings = found(repo.path());
    assert_eq!(findings.len(), 2);
    let stopped = DebtError::Findings(findings.clone()).to_string();
    assert!(
        stopped.contains("2 finding(s) that no debt record can hold")
            && stopped.contains("[strict-section-thin]"),
        "{stopped}"
    );
    let grown = DebtError::Growth(findings).to_string();
    assert!(grown.contains("only ever shrinks"), "{grown}");
}

#[test]
fn the_debt_kinds_are_kinds_of_strict_mode_and_the_ten_rules_that_a_row_can_hold() {
    for kind in DEBT_KINDS {
        assert!(Finding::KINDS.contains(&kind), "{kind}");
        assert!(
            kind.starts_with("strict-") || kind.starts_with("coverage-"),
            "{kind}"
        );
    }
    // A finding of the default rules, and the two kinds that are not rows.
    for kind in [
        "link-broken",
        "coverage-lines-over-ceiling",
        "coverage-surface-unreadable",
    ] {
        assert!(!DEBT_KINDS.contains(&kind), "{kind}");
    }
    assert_eq!(DEBT_KINDS.len(), 10);
}

#[test]
fn the_record_is_read_back_as_the_typed_value_the_write_built() {
    let record = DebtRecord::new(Vec::new(), vec![DebtLine::new("demo::helper", 3)]);
    let repo = orphan_repo("debt-typed", &[("helper", 1)], "");
    let root = repo.path();
    gate(root).write_debt().expect("adopts");
    let text = read(root, DEBT);
    let document = raf_okf::Document::parse(&text).expect("the record reads");
    assert_eq!(
        raf_okf::Profile.validate(&document, DEBT),
        Ok(RafDocument::Debt(record))
    );
}

#[test]
fn the_default_debt_writer_judges_all_present_findings() {
    let repo = two_thin();
    let root = repo.path();
    // The record says strict; a floor of default changes nothing.
    let written = gate(root).write_debt().expect("adopts");
    assert_eq!(written.findings(), 2);
}

#[test]
fn the_snapshot_of_every_other_file_is_unchanged_by_a_refused_write() {
    let repo = two_thin();
    let root = repo.path();
    gate(root).write_debt().expect("adopts");
    set(root, &thin(&["Examples", "Explanation", "Usage"]));
    let before: BTreeMap<String, Vec<u8>> = snapshot(root);
    let _ = gate(root).write_debt();
    assert_eq!(snapshot(root), before);
}
