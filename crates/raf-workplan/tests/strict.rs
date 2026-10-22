// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The complete default policy: each rule beside its accepted neighbor,
//! binding-reached packages, removed selector refusals and default Gate callers.

mod support;

use raf_okf::Location;
use raf_source::MemoryTree;
use raf_workplan::{Finding, Gate, Workplan};
use support::cases::strict_cases;
use support::disk::{Scratch, put};
use support::strict as fixture;
use support::{Repo, check_in, section, workspace_of};

/// The strict-clean repository with the capability document `capability`
/// and the coverage record `coverage`.
/// `record` with a ceiling of the orphan lines of `other`, which these
/// fixtures leave unbound on purpose, so the line report of the complete policy holds
/// it apart from the rule a test is about.
fn roomy(record: &str) -> String {
    let keys = record
        .strip_suffix("---\n")
        .expect("the record ends with its fence");
    format!("{keys}ceilings:\n  - crate: other\n    lines: 100\n---\n")
}

fn repo(capability: String, coverage: String) -> Repo {
    Repo::from([
        (fixture::CAPABILITY.to_string(), capability),
        (fixture::COVERAGE.to_string(), coverage),
    ])
}

/// The findings of `capability` under the complete default policy.
fn judged(capability: String) -> Vec<Finding> {
    check_in(
        &repo(capability, fixture::coverage("")),
        &workspace_of(&fixture::source_files()),
    )
}

/// The kinds of `findings`, in order.
fn kinds(findings: &[Finding]) -> Vec<&'static str> {
    findings.iter().map(|finding| finding.kind).collect()
}

/// The findings of `capability`, which must be exactly one, of `kind` at `at`
/// in the capability document.
fn only(capability: String, kind: &str, at: &Location) -> Finding {
    let found = judged(capability);
    assert_eq!(kinds(&found), [kind], "{found:#?}");
    assert_eq!(
        (found[0].file.as_str(), &found[0].location),
        (fixture::CAPABILITY, at)
    );
    found[0].clone()
}

#[test]
fn the_complete_policy_accepts_the_documented_repository() {
    assert_eq!(judged(fixture::capability()), [], "the complete policy");
}

#[test]
fn a_capability_with_no_example_is_refused_and_one_with_one_example_is_not() {
    let none = fixture::replaced(
        &fixture::capability(),
        fixture::EXAMPLES,
        "  examples: []\n",
    );
    let found = judged(none);
    assert_eq!(
        kinds(&found),
        ["strict-example-missing", "strict-usage-unbound"],
        "{found:#?}"
    );
    assert_eq!(found[0].location, Location::Key("bindings".to_string()));
    assert!(found[0].fix.contains("bindings.examples"), "{found:#?}");
    assert_eq!(judged(fixture::capability()), []);
}

#[test]
fn verification_without_the_matrix_is_refused_and_with_the_block_is_not() {
    let capability = fixture::capability();
    let missing = fixture::replaced(&capability, &format!("\n{}", fixture::MATRIX), "");
    only(
        missing.clone(),
        "strict-matrix-missing",
        &section("Verification"),
    );
    // A matrix in another section is not the matrix of Verification.
    let elsewhere = fixture::replaced(
        &missing,
        fixture::USAGE_SPAN,
        &format!("{}\n\n{}", fixture::USAGE_SPAN, fixture::MATRIX),
    );
    only(elsewhere, "strict-matrix-missing", &section("Verification"));
    assert_eq!(judged(capability), []);
}

#[test]
fn a_section_of_nineteen_words_is_thin_and_one_of_twenty_is_not() {
    for name in [
        "Purpose",
        "Contract",
        "Usage",
        "Examples",
        "Constraints",
        "Explanation",
        "Verification",
    ] {
        let finding = only(fixture::thin(name), "strict-section-thin", &section(name));
        assert!(
            finding.message.contains("19 words") && finding.message.contains("fewer than 20"),
            "{finding:#?}"
        );
        assert!(
            finding.fix.contains("a generated block") && finding.fix.contains("a code span"),
            "{finding:#?}"
        );
    }
    assert_eq!(raf_workplan::strict::MIN_SECTION_WORDS, 20);
    assert_eq!(fixture::words(19), fixture::THIN);
    assert_eq!(fixture::words(20), fixture::PROSE);
    assert_eq!(judged(fixture::capability()), []);
    // Twenty words with the Usage line and with the limitation count are enough.
    for (name, prose) in [("Usage", 18), ("Constraints", 15)] {
        let exact = fixture::replaced(
            &fixture::capability(),
            &format!("## {name}\n\n{}\n", fixture::PROSE),
            &format!("## {name}\n\n{}\n", fixture::words(prose)),
        );
        assert_eq!(judged(exact), [], "{name}");
    }
}

#[test]
fn code_spans_fenced_code_comments_and_the_heading_are_not_prose() {
    let padded = fixture::replaced(
        &fixture::thin("Examples"),
        &format!("## Examples\n\n{}\n", fixture::THIN),
        &format!(
            "## Examples\n\n{}\n\n`one two three four`\n\n```text\nfive six seven eight\n```\n\n<!-- nine ten eleven -->\n",
            fixture::THIN
        ),
    );
    only(padded, "strict-section-thin", &section("Examples"));
}

#[test]
fn usage_that_names_no_bound_example_is_refused_and_a_code_span_of_it_is_not() {
    let capability = fixture::capability();
    let plain = fixture::replaced(&capability, fixture::USAGE_SPAN, "Call it once.");
    only(plain, "strict-usage-unbound", &section("Usage"));
    // A code span of a path that no example binds substantiates nothing.
    let other = fixture::replaced(&capability, "`demo::run`", "`demo::other`");
    only(other, "strict-usage-unbound", &section("Usage"));
    assert_eq!(judged(capability), []);
}

#[test]
fn constraints_without_a_list_item_are_refused_and_one_list_item_is_not() {
    let capability = fixture::capability();
    let paragraph = fixture::replaced(
        &capability,
        fixture::LIMITATION,
        "One limitation of the demo.",
    );
    only(
        paragraph,
        "strict-limitations-missing",
        &section("Constraints"),
    );
    for item in [
        "- One limitation of the demo.",
        "* One limitation of the demo.",
        "1. One limitation of the demo.",
    ] {
        let listed = fixture::replaced(&capability, fixture::LIMITATION, item);
        assert_eq!(judged(listed), [], "{item}");
    }
}

/// A verification matrix whose rows hold five words: three in its header and
/// two in its row.
const FILLED_MATRIX: &str = "<!-- raf:generated:verification-matrix -->\n| Item | Kind | Section |\n| --- | --- | --- |\n| `demo::test_crate::it::case` | test | [Verification](#verification) |\n<!-- raf:generated:end -->\n";

#[test]
fn the_words_of_a_generated_block_are_not_prose_of_its_section() {
    // Nineteen words of prose beside a matrix that holds five words is thin.
    let thin = fixture::replaced(
        &fixture::thin("Verification"),
        fixture::MATRIX,
        FILLED_MATRIX,
    );
    let finding = only(thin, "strict-section-thin", &section("Verification"));
    assert!(finding.message.contains("19 words"), "{finding:#?}");
    // The neighbour: twenty words of prose beside the same matrix.
    let twenty = fixture::replaced(&fixture::capability(), fixture::MATRIX, FILLED_MATRIX);
    assert_eq!(judged(twenty), []);
}

/// The strict-clean capability with every content rule broken: no example,
/// no matrix, thin sections, no example named in Usage and no list item.
fn broken_everywhere() -> String {
    let mut text = fixture::replaced(
        &fixture::capability(),
        fixture::EXAMPLES,
        "  examples: []\n",
    );
    text = fixture::replaced(&text, &format!("\n{}", fixture::MATRIX), "");
    text = fixture::replaced(&text, fixture::USAGE_SPAN, "Call it.");
    text = fixture::replaced(&text, fixture::LIMITATION, "No list.");
    fixture::replaced(
        &text,
        &format!("## Purpose\n\n{}\n", fixture::PROSE),
        "## Purpose\n\nShort.\n",
    )
}

#[test]
fn a_capability_that_is_not_delivered_has_no_finding_of_the_content_rules() {
    let delivered = broken_everywhere();
    let found = judged(delivered.clone());
    for kind in [
        "strict-example-missing",
        "strict-matrix-missing",
        "strict-section-thin",
        "strict-usage-unbound",
        "strict-limitations-missing",
    ] {
        assert!(kinds(&found).contains(&kind), "{kind}: {found:#?}");
    }
    for state in ["implementing", "designed", "considered"] {
        let other = fixture::replaced(&delivered, "state: delivered", &format!("state: {state}"));
        let strict = judged(other.clone());
        assert!(
            !kinds(&strict)
                .iter()
                .any(|kind| kind.starts_with("strict-")),
            "{state}: {strict:#?}"
        );
        // Whatever the default rules say of the document, strict adds nothing.
        assert_eq!(strict, judged(other), "{state}");
    }
}

#[test]
fn a_code_span_inside_a_comment_names_no_example_in_usage() {
    let hidden = fixture::replaced(
        &fixture::capability(),
        fixture::USAGE_SPAN,
        "Call it once. <!-- `demo::run` -->",
    );
    only(hidden, "strict-usage-unbound", &section("Usage"));
    // The neighbour: the same span outside the comment, beside a comment.
    let shown = fixture::replaced(
        &fixture::capability(),
        fixture::USAGE_SPAN,
        "Call `demo::run` once. <!-- a note -->",
    );
    assert_eq!(judged(shown), []);
}

#[test]
fn a_thematic_break_and_an_empty_item_are_no_limitation() {
    let capability = fixture::capability();
    for line in ["* * *", "- - -", "___", "- ", "-", "1. "] {
        let text = fixture::replaced(&capability, fixture::LIMITATION, line);
        only(text, "strict-limitations-missing", &section("Constraints"));
    }
    // The neighbours: an item of text, and an item that is a code span.
    for line in [
        "- One limitation of the demo.",
        "- `demo::run`",
        "2) One more.",
    ] {
        let text = fixture::replaced(&capability, fixture::LIMITATION, line);
        assert_eq!(judged(text), [], "{line}");
    }
}

/// A workspace of the package `demo` and a member `other` with two public
/// functions and a test.
fn two_packages() -> Vec<(&'static str, String)> {
    let mut files = fixture::source_files();
    files[0].1 = fixture::MANIFEST.replace("[workspace]\n", "[workspace]\nmembers = [\"other\"]\n");
    files.extend([
        (
            "other/Cargo.toml",
            "[package]\nname = \"other\"\nversion = \"0.1.0\"\nedition = \"2021\"\n".to_string(),
        ),
        (
            "other/src/lib.rs",
            "pub fn helper() {}\npub fn more() {}\n".to_string(),
        ),
        ("other/tests/it.rs", "#[test]\nfn t() {}\n".to_string()),
    ]);
    files
}

/// The capability with these `code` and `tests` lists.
fn bound(code: &str, tests: &str) -> String {
    fixture::replaced(
        &fixture::replaced(
            &fixture::capability(),
            "  code: [demo::run]\n",
            &format!("  code: {code}\n"),
        ),
        "  tests: [demo::test_crate::it::case]\n",
        &format!("  tests: {tests}\n"),
    )
}

fn in_two_packages(capability: String, scope: &str) -> Vec<Finding> {
    let coverage = roomy(&fixture::replaced(
        &fixture::coverage(""),
        "scope: []",
        &format!("scope: {scope}"),
    ));
    check_in(&repo(capability, coverage), &workspace_of(&two_packages()))
}

#[test]
fn a_binding_reached_package_is_enforced_and_an_unreached_one_is_not() {
    // No binding reaches `other`: it is not enforced, whatever it holds.
    let unreached = bound("[demo::run]", "[demo::test_crate::it::case]");
    assert_eq!(in_two_packages(unreached, "[]"), []);

    // A binding of `other::helper` reaches the package: its other public item
    // and its test are gaps, with the reason the complete policy enforces them.
    let reached = bound("[demo::run, other::helper]", "[demo::test_crate::it::case]");
    let found = in_two_packages(reached, "[]");
    assert_eq!(
        kinds(&found),
        ["coverage-item-unbound", "coverage-test-unbound"],
        "{found:#?}"
    );
    assert_eq!(
        (found[0].file.as_str(), &found[0].location),
        ("other/src/lib.rs", &Location::Line(2))
    );
    assert_eq!(
        found[0].message,
        "the public item `other::more` of `other`, which the complete policy enforces because `cap:demo` binds `other::helper`, is bound by no capability and excluded by no exclusion"
    );
    assert_eq!(
        (found[1].file.as_str(), &found[1].location),
        ("other/tests/it.rs", &Location::Line(2))
    );

    // Bound in both directions, the package is covered.
    let covered = bound(
        "[demo::run, other::helper, other::more]",
        "[demo::test_crate::it::case, other::test_crate::it::t]",
    );
    assert_eq!(in_two_packages(covered, "[]"), []);
}

#[test]
fn test_and_example_bindings_each_enforce_the_package_they_reach() {
    let mut files = two_packages();
    let (_, library) = files
        .iter_mut()
        .find(|(path, _)| *path == "other/src/lib.rs")
        .expect("the other library");
    *library = library.replace(
        "pub fn helper() {}",
        "/** A documented helper for the real documentation-test binding. */ pub fn helper() {}",
    );
    let source = workspace_of(&files);
    let judged = |capability| check_in(&repo(capability, roomy(&fixture::coverage(""))), &source);
    let unreached = bound("[demo::run]", "[demo::test_crate::it::case]");
    assert_eq!(judged(unreached.clone()), []);

    let test_reached = bound(
        "[demo::run]",
        "[demo::test_crate::it::case, other::test_crate::it::t]",
    );
    let found = judged(test_reached);
    assert_eq!(
        kinds(&found),
        ["coverage-item-unbound", "coverage-item-unbound"],
        "{found:#?}"
    );
    for (finding, name) in found.iter().zip(["helper", "more"]) {
        assert_eq!(finding.file, "other/src/lib.rs");
        assert!(finding.message.contains(&format!("other::{name}")));
        assert!(finding.message.contains("binds `other::test_crate::it::t`"));
    }

    let examples = fixture::EXAMPLES.trim_end();
    let examples = examples
        .strip_suffix(']')
        .expect("the fixture has an inline example list");
    let example_reached = fixture::replaced(
        &unreached,
        fixture::EXAMPLES,
        &format!("{examples}, other::helper]\n"),
    );
    let found = judged(example_reached);
    assert_eq!(
        kinds(&found),
        [
            "coverage-item-unbound",
            "coverage-item-unbound",
            "coverage-test-unbound"
        ],
        "{found:#?}"
    );
    for finding in &found {
        assert!(finding.message.contains("binds `other::helper`"));
    }
    assert_eq!(found[2].file, "other/tests/it.rs");

    let covered = bound(
        "[demo::run, other::helper, other::more]",
        "[demo::test_crate::it::case, other::test_crate::it::t]",
    );
    assert_eq!(judged(covered), []);
}

#[test]
fn a_scoped_package_is_enforced_once_with_a_binding_present() {
    let reached = bound(
        "[demo::run, other::helper]",
        "[demo::test_crate::it::case, other::test_crate::it::t]",
    );
    let default = in_two_packages(reached.clone(), "[other]");
    assert_eq!(kinds(&default), ["coverage-item-unbound"], "{default:#?}");
    assert!(
        default[0]
            .message
            .contains("which the coverage record work/coverage/default.md enforces"),
        "{default:#?}"
    );
    assert_eq!(in_two_packages(reached, "[other]"), default);
}

#[test]
fn an_exclusion_of_the_record_takes_a_gap_of_a_bound_package_out() {
    let reached = bound(
        "[demo::run, other::helper]",
        "[demo::test_crate::it::case, other::test_crate::it::t]",
    );
    let coverage = roomy(&fixture::replaced(
        &fixture::coverage(""),
        "exclusions: []",
        "exclusions:\n  - item: other::more\n    reason: \"Shown to the fixtures only.\"",
    ));
    let found = check_in(&repo(reached, coverage), &workspace_of(&two_packages()));
    assert_eq!(found, []);
}

#[test]
fn a_missing_or_invalid_coverage_record_is_reported_before_package_gaps() {
    let reached = bound("[demo::run, other::helper]", "[demo::test_crate::it::case]");
    let docs = Repo::from([(fixture::CAPABILITY.to_string(), reached.clone())]);
    let source = workspace_of(&two_packages());
    let found = check_in(&docs, &source);
    assert_eq!(kinds(&found), ["coverage-record-missing"], "{found:#?}");
    // A record that does not load is reported by its own finding alone.
    let broken = repo(reached, fixture::coverage("strictness: strct\n"));
    let found = check_in(&broken, &source);
    assert_eq!(kinds(&found), ["okf-unknown-key"], "{found:#?}");
}

#[test]
fn every_policy_refusal_case_is_enforced_by_the_default_check() {
    for case in strict_cases() {
        let found = case.run();
        assert!(
            found.iter().any(|finding| finding.kind == case.kind
                && finding.file == case.file
                && finding.location == case.at),
            "{}: {found:#?}",
            case.name
        );
    }
}

/// A plan of the strict documents, with `capability` and the `strictness`
/// line `record` in the coverage record, over the source of two packages.
fn check_of(capability: String, record: &str) -> (Vec<Finding>, Vec<Finding>) {
    let docs = repo(capability, roomy(&fixture::coverage(record)));
    let tree: MemoryTree = docs.iter().collect();
    let source = workspace_of(&two_packages());
    let plan = Workplan::load(&tree, &source);
    (plan.findings().to_vec(), plan.check())
}

#[test]
fn content_and_binding_findings_follow_load_findings() {
    // A thin section and a gap of a package that the complete policy alone enforces,
    // beside a finding of the default rules: a link to nothing.
    let capability = fixture::replaced(
        &fixture::thin("Examples").replace(
            "  code: [demo::run]\n",
            "  code: [demo::run, other::helper]\n",
        ),
        "## Purpose\n\n",
        "## Purpose\n\nSee [nothing](missing.md).\n\n",
    );
    let (load, checked) = check_of(capability, "");
    assert!(
        !load
            .iter()
            .any(|finding| finding.kind.starts_with("strict-")
                || finding.kind == "coverage-item-unbound"),
        "{load:#?}"
    );
    assert!(checked.starts_with(&load), "{checked:#?}");
    assert!(kinds(&load).contains(&"link-broken"), "{load:#?}");
    let added: Vec<&str> = kinds(&checked[load.len()..])
        .into_iter()
        .filter(|kind| *kind != "generated-stale" && *kind != "index-missing")
        .collect();
    assert_eq!(
        added,
        [
            "strict-section-thin",
            "coverage-item-unbound",
            "coverage-test-unbound"
        ],
        "{checked:#?}"
    );
}

#[test]
fn the_removed_policy_key_is_rejected_even_for_its_former_values() {
    let source = workspace_of(&fixture::source_files());
    for value in ["strict", "default", "strct"] {
        let found = check_in(
            &repo(
                fixture::capability(),
                fixture::coverage(&format!("strictness: {value}\n")),
            ),
            &source,
        );
        assert_eq!(kinds(&found), ["okf-unknown-key"], "{found:#?}");
        assert_eq!(
            (found[0].file.as_str(), &found[0].location),
            (fixture::COVERAGE, &Location::Key("strictness".to_string()))
        );
    }
    assert_eq!(
        check_in(&repo(fixture::capability(), fixture::coverage("")), &source),
        []
    );
}

/// The strict-clean repository on a disk, its capability `capability` and its
/// record's `strictness` line `record`.
fn on_disk(capability: &str, record: &str) -> Scratch {
    let scratch = Scratch::new("strict");
    for (path, text) in fixture::source_files() {
        put(scratch.path(), path, &text);
    }
    put(scratch.path(), fixture::CAPABILITY, capability);
    put(
        scratch.path(),
        fixture::COVERAGE,
        &fixture::coverage(record),
    );
    scratch
}

#[test]
fn the_default_gate_judges_the_complete_policy_in_its_opened_workplan() {
    let scratch = on_disk(&fixture::thin("Examples"), "");
    let gate = Gate::workspace(scratch.path()).tracked_paths(Vec::new());
    let opened = gate.open().expect("the fixture opens");
    let retained = |found: Vec<Finding>| -> Vec<Finding> {
        found
            .into_iter()
            .filter(|finding| finding.kind != "index-missing" && finding.kind != "generated-stale")
            .collect()
    };
    let direct = retained(gate.check());
    assert_eq!(kinds(&direct), ["strict-section-thin"]);
    assert_eq!(direct, retained(opened.workplan().check()));
}

#[test]
fn while_a_capability_document_does_not_load_no_gap_of_a_bound_package_is_reported() {
    let reached = bound("[demo::run, other::helper]", "[demo::test_crate::it::case]");
    let coverage = roomy(&fixture::coverage(""));
    let source = workspace_of(&two_packages());
    // The neighbour: every document loads, and the gaps of `other` are reported.
    let loading = repo(reached.clone(), coverage.clone());
    assert_eq!(
        kinds(&check_in(&loading, &source)),
        ["coverage-item-unbound", "coverage-test-unbound"]
    );
    // A second capability document that does not load may bind those items,
    // so no gap is reported; only its own finding is.
    let mut broken = repo(reached, coverage);
    broken.insert(
        "docs/capabilities/broken.md".to_string(),
        "---\ntype: Capability\nid: cap:broken\ntitle: [unclosed\n---\n".to_string(),
    );
    let strict = check_in(&broken, &source);
    assert!(
        !strict.is_empty()
            && strict
                .iter()
                .all(|finding| finding.file == "docs/capabilities/broken.md"),
        "{strict:#?}"
    );
    assert_eq!(strict, check_in(&broken, &source));
}
