// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `coverage code` and `coverage tests` through the built binary: the gaps of
//! the whole workspace (exit 1) or none (exit 0), the exclusions with their
//! reasons, `--crate` by either form of a package name, `-q`, the JSON
//! objects, and the reports of this repository.

mod support;

use std::path::PathBuf;

use support::{COVERAGE, Repo, array, capability, member_names, run_both, strings};

/// The members of a `coverage-gap` object, in order.
const GAP: [&str; 8] = [
    "kind", "gap", "crate", "in_scope", "path", "item", "file", "line",
];

/// The package `demo` with one public function and one test, and a capability
/// that binds neither.
fn unbound(tag: &str) -> Repo {
    let repo = Repo::bare(tag);
    repo.put(
        "docs/capabilities/alpha.md",
        &capability("alpha", "considered", ["[]", "[]", "[]"], "[]", &[]),
    );
    repo
}

#[test]
fn gaps_in_either_direction_are_printed_and_exit_one() {
    let repo = unbound("coverage-gaps");
    let before = repo.snapshot();
    let code = repo.both(&["coverage", "code"]).expect(1);
    assert_eq!(
        code.stdout,
        "| Gap | Crate | Path | Location |\n| --- | --- | --- | --- |\n\
         | public item | demo | `demo::real` | src/lib.rs:2 |\n"
    );
    assert_eq!(code.stderr, "");
    let tests = repo.both(&["coverage", "tests"]).expect(1);
    assert_eq!(
        tests.stdout,
        "| Gap | Crate | Path | Location |\n| --- | --- | --- | --- |\n\
         | test | demo | `demo::test_crate::it::case` | tests/it.rs:2 |\n"
    );
    // A gap is a finding: `-q` keeps it.
    assert_eq!(repo.both(&["-q", "coverage", "code"]).expect(1), code);
    let json = repo
        .both(&["--format", "json", "coverage", "code"])
        .expect(1);
    assert_eq!(member_names(&json), [strings(&GAP)]);
    assert_eq!(
        array(&json),
        [serde_json::json!({
            "kind": "coverage-gap",
            "gap": "public item",
            "crate": "demo",
            "in_scope": false,
            "path": "demo::real",
            "item": "demo::real",
            "file": "src/lib.rs",
            "line": 2,
        })]
    );
    // The reports write nothing.
    assert_eq!(repo.snapshot(), before);
}

#[test]
fn an_exclusion_takes_a_gap_out_and_a_report_without_gaps_exits_zero() {
    let repo = unbound("coverage-excluded");
    repo.put(
        COVERAGE,
        "---\ntype: Coverage\nid: coverage:default\nscope: []\nexclusions:\n  - item: \"demo::real\"\n    reason: \"Kept for the demonstration.\"\n---\n",
    );
    let run = repo.both(&["coverage", "code"]).expect(0);
    assert_eq!(
        run.stdout,
        "| Gap | Crate | Path | Location |\n| --- | --- | --- | --- |\n\
         \n| Excluded | Crate | Path | Location | Exclusion | Reason |\n| --- | --- | --- | --- | --- | --- |\n\
         | public item | demo | `demo::real` | src/lib.rs:2 | `demo::real` | Kept for the demonstration. |\n"
    );
    // No gap: an outcome, which `-q` hides.
    assert_eq!(repo.both(&["-q", "coverage", "code"]).expect(0).stdout, "");
    assert_eq!(
        repo.both(&["-q", "--format", "json", "coverage", "code"])
            .expect(0)
            .stdout,
        "[]\n"
    );
    let json = repo
        .both(&["--format", "json", "coverage", "code"])
        .expect(0);
    let mut excluded = strings(&GAP);
    excluded.extend(strings(&["exclusion", "reason"]));
    assert_eq!(member_names(&json), [excluded]);
    assert_eq!(array(&json)[0]["kind"], "coverage-excluded");
    // The other direction still has its gap.
    repo.both(&["coverage", "tests"]).expect(1);
}

#[test]
fn crate_selects_a_package_by_either_name_and_refuses_one_that_is_not_there() {
    let repo = unbound("coverage-crate");
    let whole = repo.both(&["coverage", "tests"]).expect(1);
    assert_eq!(repo.both(&["coverage", "tests", "--crate", "demo"]), whole);
    let run = repo
        .both(&["coverage", "tests", "--crate", "nowhere"])
        .expect(2);
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr
            .contains("no package of the workspace is named `nowhere`; the packages are demo"),
        "{run:#?}"
    );
}

#[test]
fn a_surface_that_cannot_be_listed_is_a_finding_of_the_code_report_only() {
    let repo = Repo::bare("coverage-unlistable");
    // Two glob re-exports bring in the same name: the public surface of the
    // crate root is ambiguous and cannot be listed.
    repo.put(
        "src/lib.rs",
        "pub mod left {\n    pub fn dup() {}\n}\npub mod right {\n    pub fn dup() {}\n}\npub use left::*;\npub use right::*;\n",
    );
    repo.put(
        "docs/capabilities/alpha.md",
        &capability(
            "alpha",
            "delivered",
            ["[demo::left]", "[demo::test_crate::it::case]", "[]"],
            "[]",
            &[],
        ),
    );
    let code = repo.both(&["coverage", "code"]).expect(1);
    assert!(
        code.stdout
            .contains("\n| Crate | Target | Error |\n| --- | --- | --- |\n| demo | `demo` |"),
        "{code:#?}"
    );
    // A finding: `-q` keeps it whole.
    assert_eq!(repo.both(&["-q", "coverage", "code"]).expect(1), code);
    let json = repo
        .both(&["--format", "json", "coverage", "code"])
        .expect(1);
    assert_eq!(
        member_names(&json),
        [strings(&[
            "kind", "crate", "in_scope", "target", "root", "failure", "reexport", "message"
        ])]
    );
    let objects = array(&json);
    assert_eq!(objects[0]["kind"], "coverage-error");
    assert_eq!(objects[0]["crate"], "demo");
    assert_eq!(objects[0]["failure"], "unresolved");
    assert_eq!(objects[0]["root"], "src/lib.rs");
    // The tests direction does not read the surface, and its one test is
    // bound: no gap, exit 0.
    let tests = repo.both(&["coverage", "tests"]).expect(0);
    assert_eq!(
        tests.stdout,
        "| Gap | Crate | Path | Location |\n| --- | --- | --- | --- |\n"
    );
    // The neighbour: one of the two globs, and the surface is listed again;
    // its items are gaps of their own.
    repo.edit("src/lib.rs", "pub use right::*;\n", "");
    let listed = repo
        .both(&["--format", "json", "coverage", "code"])
        .expect(1);
    assert!(
        array(&listed).iter().all(|o| o["kind"] == "coverage-gap"),
        "{listed:#?}"
    );
}

/// The root of this repository.
fn repository() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the repository root")
        .to_string_lossy()
        .into_owned()
}

#[test]
fn this_repository_names_the_gaps_outside_its_scope_and_none_in_it() {
    let root = repository();
    let at = |args: &[&str]| {
        let mut all = vec!["--root", root.as_str(), "--format", "json"];
        all.extend_from_slice(args);
        run_both(&support::elsewhere(), &all)
    };
    // The whole workspace: the crates outside the scope of the record have
    // gaps, and each object says whether its crate is in scope.
    let whole = at(&["coverage", "tests"]).expect(1);
    let gaps = array(&whole);
    for package in ["raf-okf", "raf-source"] {
        assert!(
            gaps.iter()
                .any(|gap| gap["crate"] == package && gap["in_scope"] == false),
            "{package} has no test gap"
        );
    }
    assert!(
        gaps.iter().all(|gap| gap["in_scope"] == false),
        "a crate of the scope has a gap"
    );
    // The crates of the scope have none, by either form of the name.
    for args in [
        &["coverage", "tests", "--crate", "raf-workplan"][..],
        &["coverage", "code", "--crate", "raf_workplan"],
        &["coverage", "tests", "--crate", "cargo-workplan"],
    ] {
        let run = at(args).expect(0);
        assert!(
            array(&run)
                .iter()
                .all(|object| object["kind"] == "coverage-excluded"),
            "{args:?}: {run:#?}"
        );
    }
}
