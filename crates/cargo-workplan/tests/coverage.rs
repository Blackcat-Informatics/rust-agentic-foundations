// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `coverage code` and `coverage tests` through the built binary: the gaps of
//! the whole workspace (exit 1) or none (exit 0), the exclusions with their
//! reasons, `--crate` by either form of a package name, `-q`, typed RDF
//! rows, and the reports of this repository.

mod support;

use std::path::PathBuf;

use support::{COVERAGE, Repo, Shape, capability, rdf::Rdf, run_in};

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
        .both(&["--format", "turtle", "coverage", "code"])
        .expect(1);
    let rdf = Rdf::of(&json);
    let gaps = rdf.rows("CoverageGap");
    assert_eq!(gaps.len(), 1);
    let gap = gaps[0];
    assert_eq!(gap.required("kind"), "public item");
    assert_eq!(gap.required("package"), "demo");
    assert!(!gap.boolean("inScope"));
    assert_eq!(gap.required("path"), "demo::real");
    assert_eq!(gap.required("item"), "demo::real");
    assert_eq!(gap.required("file"), "src/lib.rs");
    assert_eq!(gap.number("line"), 2);
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
    let quiet = repo
        .both(&["-q", "--format", "turtle", "coverage", "code"])
        .expect(0);
    assert_eq!(Rdf::of(&quiet).result_count(), 0);
    let json = repo
        .both(&["--format", "turtle", "coverage", "code"])
        .expect(0);
    let rdf = Rdf::of(&json);
    assert!(rdf.rows("CoverageGap").is_empty());
    let excluded = rdf.rows("CoverageExcluded");
    assert_eq!(excluded.len(), 1);
    assert_eq!(excluded[0].required("exclusion"), "demo::real");
    assert_eq!(
        excluded[0].required("reason"),
        "Kept for the demonstration."
    );
    // The other direction still has its gap.
    repo.both(&["coverage", "tests"]).expect(1);
}

#[test]
fn crate_selects_a_package_by_either_name_and_refuses_one_that_is_not_there() {
    let repo = unbound("coverage-crate");
    repo.edit("Cargo.toml", "\"demo\"", "\"demo-package\"");
    repo.edit(
        "Cargo.toml",
        "[workspace]\n",
        "[workspace]\nmembers = [\"other\"]\n",
    );
    repo.edit("Cargo.lock", "\"demo\"", "\"demo-package\"");
    repo.put(
        "other/Cargo.toml",
        "[package]\nname = \"other\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    repo.put("other/src/lib.rs", "pub fn extra() {}\n");
    repo.put("other/tests/it.rs", "#[test]\nfn case() {}\n");
    repo.edit(
        "Cargo.lock",
        "version = 4\n",
        "version = 4\n\n[[package]]\nname = \"other\"\nversion = \"0.1.0\"\n",
    );
    repo.edit("src/lib.rs", "demo::real", "demo_package::real");
    for direction in ["code", "tests"] {
        let whole = repo
            .both(&["--format", "turtle", "coverage", direction])
            .expect(1);
        assert_eq!(Rdf::of(&whole).rows("CoverageGap").len(), 2);
        for name in ["demo-package", "demo_package"] {
            let selected = repo
                .both(&["--format", "turtle", "coverage", direction, "--crate", name])
                .expect(1);
            let rdf = Rdf::of(&selected);
            let gaps = rdf.rows("CoverageGap");
            assert_eq!(gaps.len(), 1);
            assert_eq!(gaps[0].required("package"), "demo-package");
            assert_eq!(
                rdf.rows("CoveragePackage")
                    .iter()
                    .map(|row| row.required("package"))
                    .collect::<Vec<_>>(),
                ["demo-package"]
            );
        }
    }
    let run = repo
        .both(&["coverage", "tests", "--crate", "nowhere"])
        .expect(2);
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr.contains(
            "no package of the workspace is named `nowhere`; the packages are demo-package, other"
        ),
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
        .both(&["--format", "turtle", "coverage", "code"])
        .expect(1);
    let rdf = Rdf::of(&json);
    let errors = rdf.rows("CoverageError");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].required("package"), "demo");
    assert_eq!(errors[0].required("failure"), "unresolved");
    assert_eq!(errors[0].required("root"), "src/lib.rs");
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
        .both(&["--format", "turtle", "coverage", "code"])
        .expect(1);
    assert!(
        Rdf::of(&listed).rows("CoverageError").is_empty(),
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
fn this_repository_has_no_code_or_test_gaps_in_its_scoped_packages() {
    let root = repository();
    let at = |args: &[&str]| {
        let mut all = vec!["--root", root.as_str(), "--format", "turtle"];
        all.extend_from_slice(args);
        run_in(&support::elsewhere(), Shape::Cargo, &all)
    };
    for direction in ["code", "tests"] {
        let whole = at(&["coverage", direction]).expect(0);
        let rdf = Rdf::of(&whole);
        assert!(rdf.rows("CoverageGap").is_empty(), "{whole:#?}");
        assert!(rdf.rows("CoverageExcluded").is_empty(), "{whole:#?}");
        assert!(rdf.rows("CoverageError").is_empty(), "{whole:#?}");
        let packages = rdf.rows("CoveragePackage");
        assert_eq!(
            packages
                .iter()
                .map(|row| row.required("package"))
                .collect::<Vec<_>>(),
            [
                "cargo-census",
                "cargo-workplan",
                "raf-census",
                "raf-okf",
                "raf-rdf",
                "raf-source",
                "raf-source-fuzz",
                "raf-workplan",
                "rust-agentic-foundations"
            ]
        );
        for row in packages {
            assert!(row.boolean("inScope"), "{whole:#?}");
            assert_eq!(
                row.number("items"),
                row.number("coveredItems"),
                "{whole:#?}"
            );
            assert_eq!(
                row.number("tests"),
                row.number("coveredTests"),
                "{whole:#?}"
            );
        }
    }
}
