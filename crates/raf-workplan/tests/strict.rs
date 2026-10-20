// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Strict mode: the setting and its floor, each rule of strict mode beside the
//! neighbour it accepts, the packages that strict mode alone enforces, and the
//! default that judges none of it.

mod support;

use raf_okf::Location;
use raf_source::MemoryTree;
use raf_workplan::{Finding, Gate, Strictness, Workplan};
use support::cases::strict_cases;
use support::disk::{Scratch, put};
use support::strict as fixture;
use support::{Repo, check_in_with, section, workspace_of};

/// The strict-clean repository with the capability document `capability`
/// and the coverage record `coverage`.
/// `record` with a ceiling of the orphan lines of `other`, which these
/// fixtures leave unbound on purpose, so the line report of strict mode holds
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

/// The findings of `capability` under a record with no `strictness`, judged
/// with the floor `strictness` against the strict source.
fn judged(capability: String, strictness: Strictness) -> Vec<Finding> {
    check_in_with(
        &repo(capability, fixture::coverage("")),
        &workspace_of(&fixture::source_files()),
        strictness,
    )
}

/// The kinds of `findings`, in order.
fn kinds(findings: &[Finding]) -> Vec<&'static str> {
    findings.iter().map(|finding| finding.kind).collect()
}

/// The findings of `capability` under strict, which must be exactly one, of
/// `kind` at `at` in the capability document; the same document under
/// `default` has none.
fn only(capability: String, kind: &str, at: &Location) -> Finding {
    assert_eq!(judged(capability.clone(), Strictness::Default), []);
    let found = judged(capability, Strictness::Strict);
    assert_eq!(kinds(&found), [kind], "{found:#?}");
    assert_eq!(
        (found[0].file.as_str(), &found[0].location),
        (fixture::CAPABILITY, at)
    );
    found[0].clone()
}

#[test]
fn the_strict_clean_repository_has_no_finding_under_either_strictness() {
    for strictness in Strictness::ALL {
        assert_eq!(
            judged(fixture::capability(), strictness),
            [],
            "{strictness:?}"
        );
    }
}

#[test]
fn a_capability_with_no_example_is_refused_and_one_with_one_example_is_not() {
    let none = fixture::replaced(
        &fixture::capability(),
        fixture::EXAMPLES,
        "  examples: []\n",
    );
    let found = judged(none, Strictness::Strict);
    assert_eq!(
        kinds(&found),
        ["strict-example-missing", "strict-usage-unbound"],
        "{found:#?}"
    );
    assert_eq!(found[0].location, Location::Key("bindings".to_string()));
    assert!(found[0].fix.contains("bindings.examples"), "{found:#?}");
    assert_eq!(judged(fixture::capability(), Strictness::Strict), []);
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
    assert_eq!(judged(capability, Strictness::Strict), []);
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
    assert_eq!(judged(fixture::capability(), Strictness::Strict), []);
    // Twenty words with the Usage line and with the limitation count are enough.
    for (name, prose) in [("Usage", 18), ("Constraints", 15)] {
        let exact = fixture::replaced(
            &fixture::capability(),
            &format!("## {name}\n\n{}\n", fixture::PROSE),
            &format!("## {name}\n\n{}\n", fixture::words(prose)),
        );
        assert_eq!(judged(exact, Strictness::Strict), [], "{name}");
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
    assert_eq!(judged(capability, Strictness::Strict), []);
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
        assert_eq!(judged(listed, Strictness::Strict), [], "{item}");
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
    assert_eq!(judged(twenty, Strictness::Strict), []);
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
    let found = judged(delivered.clone(), Strictness::Strict);
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
        let strict = judged(other.clone(), Strictness::Strict);
        assert!(
            !kinds(&strict)
                .iter()
                .any(|kind| kind.starts_with("strict-")),
            "{state}: {strict:#?}"
        );
        // Whatever the default rules say of the document, strict adds nothing.
        assert_eq!(strict, judged(other, Strictness::Default), "{state}");
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
    assert_eq!(judged(shown, Strictness::Strict), []);
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
        assert_eq!(judged(text, Strictness::Strict), [], "{line}");
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

fn in_two_packages(capability: String, scope: &str, strictness: Strictness) -> Vec<Finding> {
    let coverage = roomy(&fixture::replaced(
        &fixture::coverage(""),
        "scope: []",
        &format!("scope: {scope}"),
    ));
    check_in_with(
        &repo(capability, coverage),
        &workspace_of(&two_packages()),
        strictness,
    )
}

#[test]
fn a_bound_package_outside_scope_is_enforced_under_strict_and_an_unbound_one_is_not() {
    // No binding reaches `other`: it is not enforced, whatever it holds.
    let unreached = bound("[demo::run]", "[demo::test_crate::it::case]");
    assert_eq!(in_two_packages(unreached, "[]", Strictness::Strict), []);

    // A binding of `other::helper` reaches the package: its other public item
    // and its test are gaps, with the reason strict mode enforces them.
    let reached = bound("[demo::run, other::helper]", "[demo::test_crate::it::case]");
    assert_eq!(
        in_two_packages(reached.clone(), "[]", Strictness::Default),
        []
    );
    let found = in_two_packages(reached, "[]", Strictness::Strict);
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
        "the public item `other::more` of `other`, which strict mode enforces because `cap:demo` binds `other::helper`, is bound by no capability and excluded by no exclusion"
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
    assert_eq!(in_two_packages(covered, "[]", Strictness::Strict), []);
}

#[test]
fn a_package_of_scope_is_judged_by_the_default_rule_once_under_strict() {
    let reached = bound(
        "[demo::run, other::helper]",
        "[demo::test_crate::it::case, other::test_crate::it::t]",
    );
    let default = in_two_packages(reached.clone(), "[other]", Strictness::Default);
    assert_eq!(kinds(&default), ["coverage-item-unbound"], "{default:#?}");
    assert!(
        default[0]
            .message
            .contains("which the coverage record work/coverage/default.md enforces"),
        "{default:#?}"
    );
    assert_eq!(
        in_two_packages(reached, "[other]", Strictness::Strict),
        default
    );
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
    let found = check_in_with(
        &repo(reached, coverage),
        &workspace_of(&two_packages()),
        Strictness::Strict,
    );
    assert_eq!(found, []);
}

#[test]
fn without_a_coverage_record_strict_mode_enforces_no_package() {
    let reached = bound("[demo::run, other::helper]", "[demo::test_crate::it::case]");
    let docs = Repo::from([(fixture::CAPABILITY.to_string(), reached.clone())]);
    let source = workspace_of(&two_packages());
    let found = check_in_with(&docs, &source, Strictness::Strict);
    assert_eq!(kinds(&found), ["coverage-record-missing"], "{found:#?}");
    // A record that does not load is reported by its own finding alone.
    let broken = repo(reached, fixture::coverage("strictness: strct\n"));
    let found = check_in_with(&broken, &source, Strictness::Strict);
    assert_eq!(kinds(&found), ["okf-bad-strictness"], "{found:#?}");
}

#[test]
fn every_strict_case_reports_nothing_under_default() {
    for case in strict_cases() {
        let default = case.strict(Strictness::Default);
        assert_eq!(default.run(), [], "{}", default.name);
    }
}

/// A plan of the strict documents, with `capability` and the `strictness`
/// line `record` in the coverage record, over the source of two packages.
fn check_of(capability: String, record: &str, floor: Strictness) -> (Vec<Finding>, Vec<Finding>) {
    let docs = repo(capability, roomy(&fixture::coverage(record)));
    let tree: MemoryTree = docs.iter().collect();
    let source = workspace_of(&two_packages());
    let plan = Workplan::load(&tree, &source).with_strictness(floor);
    (plan.findings().to_vec(), plan.check())
}

#[test]
fn the_strict_findings_are_made_by_check_after_every_other_finding_and_never_by_the_load() {
    // A thin section and a gap of a package that strict mode alone enforces,
    // beside a finding of the default rules: a link to nothing.
    let capability = fixture::replaced(
        &fixture::thin("Examples").replace(
            "  code: [demo::run]\n",
            "  code: [demo::run, other::helper]\n",
        ),
        "## Purpose\n\n",
        "## Purpose\n\nSee [nothing](missing.md).\n\n",
    );
    let (default_load, default) = check_of(capability.clone(), "", Strictness::Default);
    for (record, floor) in [
        ("strictness: strict\n", Strictness::Default),
        ("", Strictness::Strict),
    ] {
        let (load, strict) = check_of(capability.clone(), record, floor);
        assert_eq!(load, default_load, "the load is the same under strict");
        assert!(
            !load
                .iter()
                .any(|finding| finding.kind.starts_with("strict-")
                    || finding.kind == "coverage-item-unbound"),
            "{load:#?}"
        );
        assert!(strict.starts_with(&default), "{strict:#?}");
        let added: Vec<&str> = kinds(&strict[default.len()..]);
        assert_eq!(
            added,
            [
                "strict-section-thin",
                "coverage-item-unbound",
                "coverage-test-unbound"
            ],
            "{strict:#?}"
        );
    }
    assert!(kinds(&default).contains(&"link-broken"), "{default:#?}");
}

#[test]
fn the_record_and_the_floor_give_the_stronger_strictness() {
    use Strictness::{Default, Strict};
    let source = workspace_of(&fixture::source_files());
    let strictness = |record: &str, floor: Strictness| {
        let docs = repo(fixture::capability(), fixture::coverage(record));
        let tree: MemoryTree = docs.iter().collect();
        Workplan::load(&tree, &source)
            .with_strictness(floor)
            .strictness()
    };
    // The record says strict: a default floor does not lower it.
    assert_eq!(strictness("strictness: strict\n", Default), Strict);
    // A strict floor raises a record that says default, or says nothing.
    assert_eq!(strictness("strictness: default\n", Strict), Strict);
    assert_eq!(strictness("", Strict), Strict);
    assert_eq!(strictness("strictness: default\n", Default), Default);
    assert_eq!(strictness("", Default), Default);
    // A record that does not load counts as default, and its finding fails the
    // check.
    assert_eq!(strictness("strictness: strct\n", Default), Default);
    assert_eq!(strictness("strictness: strct\n", Strict), Strict);
    // A floor below the one set changes nothing.
    let docs = repo(fixture::capability(), fixture::coverage(""));
    let tree: MemoryTree = docs.iter().collect();
    let plan = Workplan::load(&tree, &source)
        .with_strictness(Strict)
        .with_strictness(Default);
    assert_eq!(plan.strictness(), Strict);
}

#[test]
fn a_misspelt_strictness_is_a_finding_of_the_record_and_strict_and_default_are_not() {
    let source = workspace_of(&fixture::source_files());
    let found = |record: &str| {
        check_in_with(
            &repo(fixture::capability(), fixture::coverage(record)),
            &source,
            Strictness::Default,
        )
    };
    let bad = found("strictness: strct\n");
    assert_eq!(kinds(&bad), ["okf-bad-strictness"], "{bad:#?}");
    assert_eq!(
        (bad[0].file.as_str(), &bad[0].location),
        (fixture::COVERAGE, &Location::Key("strictness".to_string()))
    );
    assert_eq!(found("strictness: strict\n"), []);
    assert_eq!(found("strictness: default\n"), []);
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

/// The kinds `gate` reports and the strictness its opened workplan judges with.
fn through(gate: &Gate) -> (Vec<&'static str>, Strictness, Strictness) {
    let opened = gate.open().expect("the fixture opens");
    let plan = opened.workplan();
    let found: Vec<&'static str> = kinds(&gate.check())
        .into_iter()
        .filter(|kind| *kind != "index-missing" && *kind != "generated-stale")
        .collect();
    (found, opened.strictness_floor(), plan.strictness())
}

#[test]
fn a_strict_gate_judges_strict_and_the_workplans_it_opens_keep_its_floor() {
    use Strictness::{Default, Strict};
    let thin = fixture::thin("Examples");
    let scratch = on_disk(&thin, "");
    let gate = || Gate::workspace(scratch.path()).tracked_paths(Vec::new());
    assert_eq!(through(&gate()), (vec![], Default, Default));
    assert_eq!(
        through(&gate().strict()),
        (vec!["strict-section-thin"], Strict, Strict)
    );
    assert_eq!(
        through(&gate().strictness(Strict)),
        (vec!["strict-section-thin"], Strict, Strict)
    );
    // Nothing lowers the floor once it is raised.
    assert_eq!(
        through(&gate().strict().strictness(Default)),
        (vec!["strict-section-thin"], Strict, Strict)
    );
    // A record that says strict is judged strict through a default gate.
    let recorded = on_disk(&thin, "strictness: strict\n");
    let gate = Gate::workspace(recorded.path()).tracked_paths(Vec::new());
    assert_eq!(
        through(&gate),
        (vec!["strict-section-thin"], Default, Strict)
    );
}

#[test]
fn while_a_capability_document_does_not_load_no_gap_of_a_bound_package_is_reported() {
    let reached = bound("[demo::run, other::helper]", "[demo::test_crate::it::case]");
    let coverage = roomy(&fixture::coverage(""));
    let source = workspace_of(&two_packages());
    // The neighbour: every document loads, and the gaps of `other` are reported.
    let loading = repo(reached.clone(), coverage.clone());
    assert_eq!(
        kinds(&check_in_with(&loading, &source, Strictness::Strict)),
        ["coverage-item-unbound", "coverage-test-unbound"]
    );
    // A second capability document that does not load may bind those items,
    // so no gap is reported; only its own finding is.
    let mut broken = repo(reached, coverage);
    broken.insert(
        "docs/capabilities/broken.md".to_string(),
        "---\ntype: Capability\nid: cap:broken\ntitle: [unclosed\n---\n".to_string(),
    );
    let strict = check_in_with(&broken, &source, Strictness::Strict);
    assert!(
        !strict.is_empty()
            && strict
                .iter()
                .all(|finding| finding.file == "docs/capabilities/broken.md"),
        "{strict:#?}"
    );
    assert_eq!(strict, check_in_with(&broken, &source, Strictness::Default));
}
