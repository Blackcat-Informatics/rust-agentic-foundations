// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Claim coverage: the claim units of a delivered capability, the tested-by
//! markers that cover them, the findings of strict mode for an uncovered and a
//! collapsed claim beside the neighbours it accepts, the marker that names a
//! path the capability does not bind, and the report of every claim.

mod support;

use raf_okf::{CapabilitySection, Location};
use raf_source::MemoryTree;
use raf_workplan::strict::ClaimsReport;
use raf_workplan::{Finding, Strictness, Workplan};
use support::strict as fixture;
use support::{Repo, check_in_with, line_of, section, workspace_of};

const CASE: &str = "demo::test_crate::it::case";
const MARKER: &str = fixture::MARKER;

/// The Contract and the Constraints of the strict-clean capability.
fn contract_and_constraints() -> (String, String) {
    (
        format!("## Contract\n\n{}\n{MARKER}\n", fixture::PROSE),
        format!(
            "## Constraints\n\n{}\n{MARKER}\n\n{}\n{MARKER}\n",
            fixture::PROSE,
            fixture::LIMITATION
        ),
    )
}

/// The strict-clean capability with the text of its Contract `contract` and
/// of its Constraints `constraints`.
fn capability(contract: &str, constraints: &str) -> String {
    let (old_contract, old_constraints) = contract_and_constraints();
    let text = fixture::replaced(
        &fixture::capability(),
        &old_contract,
        &format!("## Contract\n\n{contract}\n"),
    );
    fixture::replaced(
        &text,
        &old_constraints,
        &format!("## Constraints\n\n{constraints}\n"),
    )
}

fn repo(capability: String) -> Repo {
    Repo::from([
        (fixture::CAPABILITY.to_string(), capability),
        (fixture::COVERAGE.to_string(), fixture::coverage("")),
    ])
}

/// The findings of `capability` under `strictness`, against the strict source.
fn judged(capability: &str, strictness: Strictness) -> Vec<Finding> {
    check_in_with(
        &repo(capability.to_string()),
        &workspace_of(&fixture::source_files()),
        strictness,
    )
}

/// The claims report of `capability` against the strict source.
fn report(capability: &str) -> ClaimsReport {
    let documents = repo(capability.to_string());
    let tree: MemoryTree = documents.iter().collect();
    let source = workspace_of(&fixture::source_files());
    Workplan::load(&tree, &source).claims_report()
}

/// The section, key, line and tests of each claim of `capability`.
fn claims(capability: &str) -> Vec<(CapabilitySection, String, usize, Vec<String>)> {
    report(capability)
        .claims()
        .iter()
        .map(|claim| {
            (
                claim.section(),
                claim.claim().to_string(),
                claim.line(),
                claim.tests().to_vec(),
            )
        })
        .collect()
}

fn tests(paths: &[&str]) -> Vec<String> {
    paths.iter().map(ToString::to_string).collect()
}

/// The kinds of `findings`, in order.
fn kinds(findings: &[Finding]) -> Vec<&'static str> {
    findings.iter().map(|finding| finding.kind).collect()
}

/// The constraints of the strict-clean capability.
fn clean_constraints() -> String {
    format!(
        "{}\n{MARKER}\n\n{}\n{MARKER}",
        fixture::PROSE,
        fixture::LIMITATION
    )
}

/// A Contract with an opening, bold-lead clauses over several paragraphs, a
/// list and a table, `###` and `####` heading clauses, a heading and a bold
/// run inside fenced code, and a repeated lead.
const CLAUSES: &str = "Some opening words.

**The first.** It holds.

A second paragraph of the first.
<!-- raf:tested-by demo::test_crate::it::case -->

- a list item of the first

| a | table of the first |
| --- | --- |

**The second.** It holds too.

### A heading clause
<!-- raf:tested-by demo::run -->

Text of the heading clause.

```text
### Not a clause

**Not a lead.** Inside the fence, after a blank line.
```

#### A smaller heading

  **Indented.** A paragraph that does not start at the margin.

**The first.** Again.

**The first.** A third time.
<!-- raf:tested-by demo::test_crate::it::case,demo::run -->";

#[test]
fn a_contract_is_cut_into_clauses_at_bold_leads_and_headings_and_the_text_before_is_the_opening() {
    let text = capability(CLAUSES, &clean_constraints());
    let at = |needle: &str| line_of(&text, needle);
    let contract: Vec<_> = claims(&text)
        .into_iter()
        .filter(|(section, ..)| *section == CapabilitySection::Contract)
        .collect();
    assert_eq!(
        contract,
        [
            (
                CapabilitySection::Contract,
                "opening".to_string(),
                at("Some opening words."),
                vec![]
            ),
            (
                CapabilitySection::Contract,
                "the-first".to_string(),
                at("**The first.** It holds."),
                tests(&[CASE])
            ),
            (
                CapabilitySection::Contract,
                "the-second".to_string(),
                at("**The second.**"),
                vec![]
            ),
            (
                CapabilitySection::Contract,
                "a-heading-clause".to_string(),
                at("### A heading clause"),
                tests(&["demo::run"])
            ),
            (
                CapabilitySection::Contract,
                "a-smaller-heading".to_string(),
                at("#### A smaller heading"),
                vec![]
            ),
            (
                CapabilitySection::Contract,
                "the-first-2".to_string(),
                at("**The first.** Again."),
                vec![]
            ),
            (
                CapabilitySection::Contract,
                "the-first-3".to_string(),
                at("**The first.** A third time."),
                tests(&[CASE, "demo::run"])
            ),
        ]
    );
    let keys: Vec<String> = report(&text)
        .claims()
        .iter()
        .map(raf_workplan::strict::ClaimCoverage::key)
        .collect();
    assert!(
        keys.contains(&"cap:demo#contract/a-heading-clause".to_string()),
        "{keys:?}"
    );
}

#[test]
fn a_contract_clause_covered_on_its_second_paragraph_is_covered_and_its_neighbour_is_not() {
    let contract = format!(
        "**The first.** It holds.\n\nA second paragraph of the first.\n{MARKER}\n\n**The second.** It holds too, for every input that the parser reads."
    );
    let text = capability(&contract, &clean_constraints());
    assert_eq!(judged(&text, Strictness::Default), []);
    let found = judged(&text, Strictness::Strict);
    assert_eq!(kinds(&found), ["strict-claim-uncovered"], "{found:#?}");
    assert_eq!(
        found[0].location,
        Location::Line(line_of(&text, "**The second.**"))
    );
    assert_eq!(
        found[0].message,
        "the claim `cap:demo#contract/the-second` (\"The second. It holds too, for every input that the parser re\") names no test"
    );
    assert!(
        found[0]
            .fix
            .starts_with("put `<!-- raf:tested-by <path>[,<path>...] -->` after the statement"),
        "{found:#?}"
    );
    // The neighbour: a marker after the second clause covers it too.
    let covered = capability(&format!("{contract}\n{MARKER}"), &clean_constraints());
    assert_eq!(judged(&covered, Strictness::Strict), []);
}

#[test]
fn a_heading_clause_is_covered_by_a_marker_after_its_heading_and_its_text_needs_none() {
    let contract = format!(
        "### The heading clause\n{MARKER}\n\nThe text of the clause, which needs no marker of its own.\n\n### The other clause\n\nIts text."
    );
    let text = capability(&contract, &clean_constraints());
    let found = judged(&text, Strictness::Strict);
    assert_eq!(kinds(&found), ["strict-claim-uncovered"], "{found:#?}");
    assert_eq!(
        found[0].location,
        Location::Line(line_of(&text, "### The other clause"))
    );
    assert!(
        found[0]
            .message
            .contains("`cap:demo#contract/the-other-clause` (\"The other clause\")"),
        "{found:#?}"
    );
}

#[test]
fn constraints_are_cut_into_top_level_items_paragraphs_and_tables() {
    let constraints = format!(
        "- An item that\n  goes on.\n  - A nested item of it.\n\n  A paragraph of the item.\n{MARKER}\n- A second item.\n\nA paragraph outside the list.\n\n| A | table |\n| --- | --- |\n| of | rows |\n{MARKER}\n\n- A second item.\n"
    );
    let text = capability(&format!("{}\n{MARKER}", fixture::PROSE), &constraints);
    let at = |needle: &str| line_of(&text, needle);
    let found: Vec<(String, usize, Vec<String>)> = claims(&text)
        .into_iter()
        .filter(|(section, ..)| *section == CapabilitySection::Constraints)
        .map(|(_, key, line, tests)| (key, line, tests))
        .collect();
    let lines: Vec<(usize, Vec<String>)> = found
        .iter()
        .map(|(_, line, tests)| (*line, tests.clone()))
        .collect();
    assert_eq!(
        lines,
        [
            (at("- An item that"), tests(&[CASE])),
            (at("- A second item."), vec![]),
            (at("A paragraph outside the list."), vec![]),
            (at("| A | table |"), tests(&[CASE])),
            (
                text.lines()
                    .enumerate()
                    .filter(|(_, line)| *line == "- A second item.")
                    .nth(1)
                    .map(|(index, _)| index + 1)
                    .expect("the second copy"),
                vec![]
            ),
        ]
    );
    // A Constraints key is `c-` and sixteen hexadecimal digits, and the same
    // text a second time takes `-2`.
    for (key, ..) in &found[..4] {
        assert!(
            key.len() == 18
                && key.starts_with("c-")
                && key[2..].bytes().all(|b| b.is_ascii_hexdigit()),
            "{key}"
        );
    }
    assert_eq!(found[4].0, format!("{}-2", found[1].0));
    // The key is the digest of the text: other whitespace and a marker keep
    // it, and another word changes it.
    let respaced = capability(
        &format!("{}\n{MARKER}", fixture::PROSE),
        &constraints.replace(
            "- A second item.\n\nA paragraph",
            "- A  second   item.\n<!-- raf:tested-by demo::run -->\n\nA paragraph",
        ),
    );
    let respaced_keys: Vec<String> = claims(&respaced)
        .into_iter()
        .filter(|(section, ..)| *section == CapabilitySection::Constraints)
        .map(|(_, key, ..)| key)
        .collect();
    assert_eq!(respaced_keys[1], found[1].0);
    let reworded = capability(
        &format!("{}\n{MARKER}", fixture::PROSE),
        &constraints.replace(
            "- A second item.\n\nA paragraph",
            "- A third item.\n\nA paragraph",
        ),
    );
    let reworded_keys: Vec<String> = claims(&reworded)
        .into_iter()
        .filter(|(section, ..)| *section == CapabilitySection::Constraints)
        .map(|(_, key, ..)| key)
        .collect();
    assert_ne!(reworded_keys[1], found[1].0);
}

#[test]
fn a_long_contract_with_no_clause_start_is_one_collapsed_claim_and_one_with_clauses_is_not() {
    // Twenty-one words and no clause start: one claim, a finding however it
    // is covered.
    let long = capability(
        &format!("{} Again.\n{MARKER}", fixture::PROSE),
        &clean_constraints(),
    );
    let found = judged(&long, Strictness::Strict);
    assert_eq!(kinds(&found), ["strict-claim-unit-collapsed"], "{found:#?}");
    assert_eq!(found[0].location, section("Contract"));
    assert!(
        found[0].message.contains("21 words") && found[0].fix.contains("`###`"),
        "{found:#?}"
    );
    assert_eq!(
        claims(&long)
            .iter()
            .filter(|(section, ..)| *section == CapabilitySection::Contract)
            .count(),
        1
    );
    assert_eq!(judged(&long, Strictness::Default), []);
    // The neighbours: twenty words, and the same words under a bold lead.
    let twenty = capability(
        &format!("{}\n{MARKER}", fixture::PROSE),
        &clean_constraints(),
    );
    assert_eq!(judged(&twenty, Strictness::Strict), []);
    let led = capability(
        &format!("**The lead.** {} Again.\n{MARKER}", fixture::PROSE),
        &clean_constraints(),
    );
    assert_eq!(judged(&led, Strictness::Strict), []);
    // Uncovered as well, the one claim is reported once more as uncovered.
    let bare = capability(&format!("{} Again.", fixture::PROSE), &clean_constraints());
    assert_eq!(
        kinds(&judged(&bare, Strictness::Strict)),
        ["strict-claim-unit-collapsed", "strict-claim-uncovered"]
    );
}

#[test]
fn a_marker_that_names_a_path_the_capability_does_not_bind_is_a_finding_under_either_strictness() {
    let unbound = capability(
        &format!(
            "{}\n<!-- raf:tested-by {CASE},demo::test_crate::it::other -->",
            fixture::PROSE
        ),
        &clean_constraints(),
    );
    for strictness in Strictness::ALL {
        let found = judged(&unbound, strictness);
        assert_eq!(kinds(&found), ["claim-test-unbound"], "{strictness:?}");
        assert_eq!(
            found[0].location,
            Location::Line(line_of(&unbound, "demo::test_crate::it::other"))
        );
        assert!(
            found[0].message.contains("`demo::test_crate::it::other`")
                && found[0].message.contains("`cap:demo`"),
            "{found:#?}"
        );
    }
    // A path bound in `code` alone is not a test either.
    let code = fixture::replaced(
        &capability(
            &format!("{}\n<!-- raf:tested-by demo::run -->", fixture::PROSE),
            &clean_constraints(),
        ),
        fixture::EXAMPLES,
        "  examples: []\n",
    );
    assert!(
        kinds(&judged(&code, Strictness::Default)) == ["claim-test-unbound"],
        "{:#?}",
        judged(&code, Strictness::Default)
    );
    // The neighbour: every path bound in `tests` or `examples`.
    let bound = capability(
        &format!(
            "{}\n<!-- raf:tested-by {CASE},demo::run -->",
            fixture::PROSE
        ),
        &clean_constraints(),
    );
    for strictness in Strictness::ALL {
        assert_eq!(judged(&bound, strictness), [], "{strictness:?}");
    }
}

#[test]
fn a_tested_by_marker_in_usage_names_the_bound_example_as_a_code_span_does() {
    let text = fixture::replaced(
        &fixture::capability(),
        fixture::USAGE_SPAN,
        "Call the function once.\n<!-- raf:tested-by demo::run -->",
    );
    assert_eq!(judged(&text, Strictness::Strict), []);
    let unnamed = fixture::replaced(&fixture::capability(), fixture::USAGE_SPAN, "Call it once.");
    assert_eq!(
        kinds(&judged(&unnamed, Strictness::Strict)),
        ["strict-usage-unbound"]
    );
}

#[test]
fn the_report_lists_every_claim_of_every_delivered_capability_sorted_and_the_same_each_run() {
    let text = capability(CLAUSES, &clean_constraints());
    let first = report(&text);
    assert_eq!(first, report(&text));
    let order: Vec<(String, &str, usize)> = first
        .claims()
        .iter()
        .map(|claim| {
            (
                claim.capability().to_string(),
                claim.section().slug(),
                claim.line(),
            )
        })
        .collect();
    let mut sorted = order.clone();
    sorted.sort();
    assert_eq!(order, sorted);
    // Constraints sorts before Contract by its slug.
    assert_eq!(first.claims()[0].section(), CapabilitySection::Constraints);
    assert!(!first.is_complete());
    assert_eq!(first.capability("cap:demo"), first);
    assert_eq!(first.capability("cap:other").claims(), []);
    // The strict-clean capability has three claims, each covered.
    let clean = report(&fixture::capability());
    assert_eq!(clean.claims().len(), 3);
    assert!(clean.is_complete());
    // A capability that is not delivered has no claim in the report.
    let designed = fixture::capability().replace("state: delivered", "state: implementing");
    assert_eq!(report(&designed).claims(), []);
}
