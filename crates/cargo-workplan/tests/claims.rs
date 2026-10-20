// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `coverage claims` through the built binary: every claim of the delivered
//! capabilities with the tests its markers name, exit 0 when each names one,
//! 1 when one names none and 2 for a capability that is not loaded, the table
//! and the JSON array holding the same rows in the same sorted order, the same
//! bytes on every run and under `--strict`, and nothing written.

mod support;

use raf_workplan::Gate;
use raf_workplan::coverage::render;
use support::{Repo, array, member_names, strict, strings};

/// The members of a `coverage-claim` object, in order.
const CLAIM: [&str; 6] = ["kind", "capability", "section", "claim", "line", "tests"];

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

/// The same rows as JSON.
const CLEAN_JSON: &str = "[
{\"kind\":\"coverage-claim\",\"capability\":\"cap:demo\",\"section\":\"constraints\",\"claim\":\"c-615f0857bd835c69\",\"line\":39,\"tests\":[\"demo::test_crate::it::case\"]},
{\"kind\":\"coverage-claim\",\"capability\":\"cap:demo\",\"section\":\"constraints\",\"claim\":\"c-262503de63f407d1\",\"line\":42,\"tests\":[\"demo::test_crate::it::case\"]},
{\"kind\":\"coverage-claim\",\"capability\":\"cap:demo\",\"section\":\"contract\",\"claim\":\"opening\",\"line\":24,\"tests\":[\"demo::test_crate::it::case\"]}
]
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
    let opened = Gate::workspace(repo.root()).open().expect("a repository");
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
    let json = repo
        .both(&["--format", "json", "coverage", "claims"])
        .expect(0);
    assert_eq!(json.stdout, CLEAN_JSON);
    // Two runs, `--strict` and the capability named: the same bytes.
    assert_eq!(repo.both(&["coverage", "claims"]).expect(0), run);
    assert_eq!(
        repo.both(&["--strict", "coverage", "claims"]).expect(0),
        run
    );
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
fn a_claim_that_names_no_test_is_a_finding_and_exits_one_in_text_and_json_alike() {
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
    let json = repo
        .both(&["--format", "json", "coverage", "claims"])
        .expect(1);
    assert_eq!(member_names(&json), vec![strings(&CLAIM); 3]);
    let objects = array(&json);
    // The JSON holds the rows of the table, in the same order.
    let from_json: Vec<Vec<String>> = objects
        .iter()
        .map(|object| {
            let tests: Vec<String> = object["tests"]
                .as_array()
                .expect("a list of tests")
                .iter()
                .map(|path| format!("`{}`", path.as_str().expect("a path")))
                .collect();
            vec![
                object["capability"].as_str().expect("an id").to_string(),
                object["section"].as_str().expect("a section").to_string(),
                object["claim"].as_str().expect("a key").to_string(),
                object["line"].to_string(),
                tests.join(", "),
            ]
        })
        .collect();
    assert_eq!(from_json, table);
    assert!(
        objects
            .iter()
            .all(|object| object["kind"] == "coverage-claim")
    );
    // Sorted by capability, section and line.
    let mut sorted = table.clone();
    sorted.sort_by(|a, b| {
        (&a[0], &a[1], a[3].parse::<usize>().ok()).cmp(&(&b[0], &b[1], b[3].parse::<usize>().ok()))
    });
    assert_eq!(sorted, table);
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
