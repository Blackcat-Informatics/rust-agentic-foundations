// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `coverage claims` through the built binary: every claim of the delivered
//! capabilities with the tests its markers name, exit 0 when each names one,
//! 1 when one names none and 2 for a capability that is not loaded, the table
//! and typed RDF graph holding the same rows in the same sorted order,
//! the same bytes on every run, and nothing written.

mod support;

use raf_workplan::Gate;
use raf_workplan::coverage::render;
use support::{Repo, rdf::Rdf, strict};

/// The table of the strict-clean capability: its Contract is one claim and its
/// Constraints two, each covered by the marker that names the bound test. The
/// key of a Constraints unit is `c-` and the first sixteen hexadecimal digits
/// of the SHA-256 digest of its text, and Constraints sorts before Contract.
const CLEAN: &str = "\
| Capability | Section | Claim | Line | Tested by |
| --- | --- | --- | --- | --- |
| cap:demo | constraints | c-615f0857bd835c69 | 39 | `demo::test_crate::it::case` |
| cap:demo | constraints | c-262503de63f407d1 | 42 | `demo::test_crate::it::case` |
| cap:demo | contract | opening | 24 | `demo::test_crate::it::case` |
";

/// The rows of a table, without its two header lines.
fn rows(table: &str) -> Vec<Vec<String>> {
    table
        .lines()
        .skip(2)
        .map(|row| {
            row.trim_matches('|')
                .split(" | ")
                .map(|cell| cell.trim().to_string())
                .collect()
        })
        .collect()
}

/// The table the library renders for the repository at `repo`.
fn library_table(repo: &Repo) -> String {
    let opened = Gate::workspace(repo.root())
        .programs(raf_workplan::exec::Programs::new(repo.root()).discover_git())
        .open()
        .expect("a repository");
    render::claims_table(&opened.workplan().claims_report())
}

#[test]
fn every_claim_names_a_test_prints_the_report_and_exits_zero() {
    let repo = Repo::strict("claims-clean", &strict::capability(), "");
    let before = repo.snapshot();
    let run = repo.both(&["coverage", "claims"]).expect(0);
    assert_eq!(run.stdout, CLEAN, "{run:#?}");
    assert_eq!(run.stderr, "");
    assert_eq!(run.stdout, library_table(&repo));
    let machine = repo
        .both(&["--format", "turtle", "coverage", "claims"])
        .expect(0);
    let rdf = Rdf::of(&machine);
    let claims = rdf.rows("ClaimCoverage");
    assert_eq!(claims.len(), 3);
    assert!(
        claims
            .iter()
            .all(|c| c.required("capability") == "cap:demo")
    );
    assert_eq!(
        claims.iter().map(|c| c.number("line")).collect::<Vec<_>>(),
        [39, 42, 24]
    );
    assert!(
        claims
            .iter()
            .all(|c| c.children("test")[0].required("path") == "demo::test_crate::it::case")
    );
    // A repeated run and the capability named produce the same bytes.
    assert_eq!(repo.both(&["coverage", "claims"]).expect(0), run);
    assert_eq!(
        repo.both(&["coverage", "claims", "--capability", "cap:demo"])
            .expect(0),
        run
    );
    // A report whose every claim names a test is an outcome: `-q` hides it.
    assert_eq!(
        repo.both(&["-q", "coverage", "claims"]).expect(0).stdout,
        ""
    );
    assert_eq!(repo.snapshot(), before, "the report wrote");
}

#[test]
fn a_claim_that_names_no_test_is_a_finding_and_exits_one_in_text_and_rdf_alike() {
    let uncovered = strict::replaced(
        &strict::capability(),
        &format!("{}\n\n## Usage", strict::MARKER),
        "\n## Usage",
    );
    let repo = Repo::strict("claims-uncovered", &uncovered, "");
    let before = repo.snapshot();
    let text = repo.both(&["coverage", "claims"]).expect(1);
    assert_eq!(text.stdout, library_table(&repo));
    let table = rows(&text.stdout);
    assert_eq!(table.len(), 3, "{text:#?}");
    assert!(
        table
            .iter()
            .any(|row| row[1] == "contract" && row[2] == "opening" && row[4].is_empty()),
        "{text:#?}"
    );
    // A finding: `-q` keeps it whole.
    assert_eq!(repo.both(&["-q", "coverage", "claims"]).expect(1), text);
    let machine = repo
        .both(&["--format", "turtle", "coverage", "claims"])
        .expect(1);
    let rdf = Rdf::of(&machine);
    let claims = rdf.rows("ClaimCoverage");
    let from_rdf: Vec<Vec<String>> = claims
        .iter()
        .map(|claim| {
            vec![
                claim.required("capability").into(),
                claim.required("section").into(),
                claim.required("claim").into(),
                claim.number("line").to_string(),
                claim
                    .children("test")
                    .iter()
                    .map(|test| format!("`{}`", test.required("path")))
                    .collect::<Vec<_>>()
                    .join(", "),
            ]
        })
        .collect();
    assert_eq!(from_rdf, table);
    // Sorted by capability, section and line.
    let keys: Vec<_> = claims
        .iter()
        .map(|row| {
            (
                row.required("capability"),
                row.required("section"),
                row.number("line"),
            )
        })
        .collect();
    assert!(keys.windows(2).all(|pair| pair[0] <= pair[1]), "{keys:?}");
    assert_eq!(repo.snapshot(), before, "the report wrote");
}

#[test]
fn a_capability_that_is_not_loaded_exits_two_and_says_why() {
    let repo = Repo::strict("claims-unknown", &strict::capability(), "");
    let run = repo
        .both(&["coverage", "claims", "--capability", "cap:nowhere"])
        .expect(2);
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr
            .contains("no loaded capability is `cap:nowhere`; the capabilities are cap:demo"),
        "{run:#?}"
    );
    // The neighbour: the loaded capability.
    repo.both(&["coverage", "claims", "--capability", "cap:demo"])
        .expect(0);
    // An id that is not one is a usage error before the repository is read.
    let usage = repo
        .both(&["coverage", "claims", "--capability", "demo"])
        .expect(2);
    assert_eq!(usage.stdout, "");
}

/// Beside a second delivered capability whose id sorts before `cap:demo`, the
/// report lists every claim of `cap:alpha` first, its Contract included, and
/// `--capability` prints the rows of the capability it names and no other.
#[test]
fn the_report_sorts_by_capability_first_and_capability_prints_the_one_it_names() {
    let repo = Repo::strict("claims-two", &strict::capability(), "");
    let alpha = strict::replaced(
        &strict::replaced(&strict::capability(), "id: cap:demo", "id: cap:alpha"),
        "title: \"Demo\"",
        "title: \"Alpha\"",
    );
    repo.put("docs/capabilities/alpha.md", &alpha);
    repo.sync();
    let before = repo.snapshot();
    let run = repo.both(&["coverage", "claims"]).expect(0);
    let rows_of = |id: &str| -> Vec<String> {
        CLEAN
            .lines()
            .skip(2)
            .map(|row| row.replace("cap:demo", id))
            .collect()
    };
    let expected: String = CLEAN
        .lines()
        .take(2)
        .map(str::to_string)
        .chain(rows_of("cap:alpha"))
        .chain(rows_of("cap:demo"))
        .fold(String::new(), |mut text, line| {
            text.push_str(&line);
            text.push('\n');
            text
        });
    assert_eq!(run.stdout, expected, "{run:#?}");
    assert_eq!(run.stdout, library_table(&repo));
    let demo = repo
        .both(&["coverage", "claims", "--capability", "cap:demo"])
        .expect(0);
    assert_eq!(demo.stdout, CLEAN);
    let only_alpha = repo
        .both(&["coverage", "claims", "--capability", "cap:alpha"])
        .expect(0);
    assert_eq!(only_alpha.stdout, CLEAN.replace("cap:demo", "cap:alpha"));
    let machine = repo
        .both(&[
            "--format",
            "turtle",
            "coverage",
            "claims",
            "--capability",
            "cap:demo",
        ])
        .expect(0);
    let rdf = Rdf::of(&machine);
    let claims = rdf.rows("ClaimCoverage");
    assert_eq!(claims.len(), 3);
    assert!(
        claims
            .iter()
            .all(|c| c.required("capability") == "cap:demo")
    );
    assert_eq!(
        claims.iter().map(|c| c.number("line")).collect::<Vec<_>>(),
        [39, 42, 24]
    );
    assert!(
        claims
            .iter()
            .all(|c| c.children("test")[0].required("path") == "demo::test_crate::it::case")
    );
    assert_eq!(repo.snapshot(), before, "the report wrote");
}
