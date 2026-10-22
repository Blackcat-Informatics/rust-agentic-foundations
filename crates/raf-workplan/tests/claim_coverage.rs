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
use raf_workplan::{Finding, Workplan};
use support::strict as fixture;
use support::{Repo, check_in, line_of, section, workspace_of};

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

/// The findings of `capability` under the complete default policy.
fn judged(capability: &str) -> Vec<Finding> {
    check_in(
        &repo(capability.to_string()),
        &workspace_of(&fixture::source_files()),
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
fn fenced_generated_and_proposed_text_make_no_claim_and_evidence_does_not_cover_one() {
    let contract = format!(
        "**Authored.** The current authored guarantee remains a claim.\n<!-- raf:claim evidence=bound -->\n\n```text\n**Fenced.** This is an example of a claim.\n```\n\n<!-- raf:generated:verification-matrix -->\n**Generated.** This is generated text.\n<!-- raf:generated:end -->\n\n> [!PROPOSED change:add-proof]\n> **Proposed.** This is proposed text.\n\n**Neighbor.** The next authored guarantee names its test.\n{MARKER}"
    );
    let text = capability(&contract, &clean_constraints());
    let actual: Vec<_> = claims(&text)
        .into_iter()
        .filter(|(section, ..)| *section == CapabilitySection::Contract)
        .map(|(_, key, _, tests)| (key, tests))
        .collect();
    assert_eq!(
        actual,
        [
            ("authored".to_string(), vec![]),
            ("neighbor".to_string(), tests(&[CASE])),
        ]
    );
    // Resolving the proposal into authored text creates its claim; a
    // tested-by marker, unlike evidence alone, covers the authored claim.
    let resolved = contract
        .replace(
            "> [!PROPOSED change:add-proof]\n> **Proposed.** This is proposed text.",
            "**Proposed.** This is current authored text.",
        )
        .replace("<!-- raf:claim evidence=bound -->", MARKER);
    let resolved = capability(&resolved, &clean_constraints());
    let actual: Vec<_> = claims(&resolved)
        .into_iter()
        .filter(|(section, ..)| *section == CapabilitySection::Contract)
        .map(|(_, key, _, tests)| (key, tests))
        .collect();
    assert_eq!(
        actual,
        [
            ("authored".to_string(), tests(&[CASE])),
            ("proposed".to_string(), vec![]),
            ("neighbor".to_string(), tests(&[CASE])),
        ]
    );
}

#[test]
fn a_bound_test_that_panics_still_covers_a_claim_without_execution() {
    let mut files = fixture::source_files();
    let entry = files
        .iter_mut()
        .find(|(path, _)| *path == "tests/it.rs")
        .expect("the fixture's test source");
    assert_eq!(entry.1, fixture::TEST);
    entry.1 =
        "#[test]\nfn case() { panic!(\"this mapping does not execute the test\"); }\n".to_string();
    let source = workspace_of(&files);
    let mut docs = repo(fixture::capability());
    let tree: MemoryTree = docs.iter().collect();
    let plan = Workplan::load(&tree, &source);
    let synced = plan
        .sync(&raf_workplan::SyncOptions::new("2026 Example", "CC-BY-4.0"))
        .expect("the actual generated blocks synchronize");
    for (path, text) in synced.edits {
        docs.insert(path.to_string(), text);
    }
    let tree: MemoryTree = docs.iter().collect();
    let plan = Workplan::load(&tree, &source);
    assert_eq!(plan.check(), []);
    assert!(plan.claims_report().is_complete());
    assert!(
        plan.claims_report()
            .claims()
            .iter()
            .all(|claim| claim.tests() == [CASE])
    );
    // The ordinary empty-body neighbor has the same complete mapping.
    assert!(report(&fixture::capability()).is_complete());
}

#[test]
fn a_contract_clause_covered_on_its_second_paragraph_is_covered_and_its_neighbour_is_not() {
    let contract = format!(
        "**The first.** It holds.\n\nA second paragraph of the first.\n{MARKER}\n\n**The second.** It holds too, for every input that the parser reads."
    );
    let text = capability(&contract, &clean_constraints());
    let found = judged(&text);
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
    assert_eq!(judged(&covered), []);
}

#[test]
fn a_heading_clause_is_covered_by_a_marker_after_its_heading_and_its_text_needs_none() {
    let contract = format!(
        "### The heading clause\n{MARKER}\n\nThe text of the clause, which needs no marker of its own.\n\n### The other clause\n\nIts text."
    );
    let text = capability(&contract, &clean_constraints());
    let found = judged(&text);
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
                && purrdf_hash::hex::decode_canonical(&key[2..]).is_ok(),
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
    let found = judged(&long);
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
    // The neighbours: twenty words, and the same words under a bold lead.
    let twenty = capability(
        &format!("{}\n{MARKER}", fixture::PROSE),
        &clean_constraints(),
    );
    assert_eq!(judged(&twenty), []);
    let led = capability(
        &format!("**The lead.** {} Again.\n{MARKER}", fixture::PROSE),
        &clean_constraints(),
    );
    assert_eq!(judged(&led), []);
    // Uncovered as well, the one claim is reported once more as uncovered.
    let bare = capability(&format!("{} Again.", fixture::PROSE), &clean_constraints());
    assert_eq!(
        kinds(&judged(&bare)),
        ["strict-claim-unit-collapsed", "strict-claim-uncovered"]
    );
}

#[test]
fn an_unbound_claim_marker_is_refused_beside_covered_neighbors() {
    let unbound = capability(
        &format!(
            "{}\n<!-- raf:tested-by {CASE},demo::test_crate::it::other -->",
            fixture::PROSE
        ),
        &clean_constraints(),
    );

    let found = judged(&unbound);
    assert_eq!(kinds(&found), ["claim-test-unbound"], "the complete policy");
    assert_eq!(
        found[0].location,
        Location::Line(line_of(&unbound, "demo::test_crate::it::other"))
    );
    assert!(
        found[0].message.contains("`demo::test_crate::it::other`")
            && found[0].message.contains("`cap:demo`"),
        "{found:#?}"
    );

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
        kinds(&judged(&code))
            == [
                "claim-test-unbound",
                "strict-example-missing",
                "strict-usage-unbound"
            ],
        "{:#?}",
        judged(&code)
    );
    // The neighbour: every path bound in `tests` or `examples`.
    let bound = capability(
        &format!(
            "{}\n<!-- raf:tested-by {CASE},demo::run -->",
            fixture::PROSE
        ),
        &clean_constraints(),
    );

    assert_eq!(judged(&bound), [], "the complete policy");
}

#[test]
fn a_tested_by_marker_in_usage_names_the_bound_example_as_a_code_span_does() {
    let text = fixture::replaced(
        &fixture::capability(),
        fixture::USAGE_SPAN,
        "Call the function once.\n<!-- raf:tested-by demo::run -->",
    );
    assert_eq!(judged(&text), []);
    let unnamed = fixture::replaced(&fixture::capability(), fixture::USAGE_SPAN, "Call it once.");
    assert_eq!(kinds(&judged(&unnamed)), ["strict-usage-unbound"]);
}

/// The strict-clean capability as `cap:alpha`, whose id sorts before
/// `cap:demo`: the same claims at the same lines.
fn alpha() -> String {
    fixture::replaced(
        &fixture::replaced(&fixture::capability(), "id: cap:demo", "id: cap:alpha"),
        "title: \"Demo\"",
        "title: \"Alpha\"",
    )
}

/// The report of `capability` beside the capability [`alpha`].
fn report_beside_alpha(capability: &str) -> ClaimsReport {
    let mut documents = repo(capability.to_string());
    documents.insert("docs/capabilities/alpha.md".to_string(), alpha());
    let tree: MemoryTree = documents.iter().collect();
    let source = workspace_of(&fixture::source_files());
    Workplan::load(&tree, &source).claims_report()
}

#[test]
fn the_report_lists_every_claim_of_every_delivered_capability_sorted_and_the_same_each_run() {
    let text = capability(CLAUSES, &clean_constraints());
    let first = report_beside_alpha(&text);
    assert_eq!(first, report_beside_alpha(&text));
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
    // The capability sorts first: every claim of `cap:alpha`, its Contract
    // included, comes before the claims of `cap:demo`, and within each
    // Constraints sorts before Contract by its slug.
    let capabilities: Vec<&str> = order.iter().map(|(id, ..)| id.as_str()).collect();
    let split = capabilities
        .iter()
        .position(|id| *id == "cap:demo")
        .expect("cap:demo");
    assert_eq!(split, 3, "{order:?}");
    assert!(capabilities[..split].iter().all(|id| *id == "cap:alpha"));
    assert!(capabilities[split..].iter().all(|id| *id == "cap:demo"));
    assert_eq!(first.claims()[0].section(), CapabilitySection::Constraints);
    assert_eq!(first.claims()[2].section(), CapabilitySection::Contract);
    assert_eq!(
        first.claims()[split].section(),
        CapabilitySection::Constraints
    );
    assert!(!first.is_complete());
    assert_eq!(first.capability("cap:demo"), report(&text));
    assert_eq!(first.capability("cap:alpha").claims().len(), 3);
    assert_eq!(first.capability("cap:other").claims(), []);
    // The strict-clean capability has three claims, each covered.
    let clean = report(&fixture::capability());
    assert_eq!(clean.claims().len(), 3);
    assert!(clean.is_complete());
    // A capability that is not delivered has no claim in the report.
    let designed = fixture::capability().replace("state: delivered", "state: implementing");
    assert_eq!(report(&designed).claims(), []);
}

/// The keys of the Contract claims of `contract`.
fn contract_keys(contract: &str) -> Vec<String> {
    claims(&capability(contract, &clean_constraints()))
        .into_iter()
        .filter(|(section, ..)| *section == CapabilitySection::Contract)
        .map(|(_, key, ..)| key)
        .collect()
}

/// A repeated lead takes the first number that no other key of its section
/// has, whichever comes first, so a lead `Foo 2` and the second `Foo` never
/// share a key.
#[test]
fn a_repeated_key_takes_the_first_number_no_other_key_of_its_section_has_in_either_order() {
    assert_eq!(
        contract_keys("**Foo.** a\n\n**Foo.** b\n\n**Foo 2.** c"),
        ["foo", "foo-3", "foo-2"]
    );
    assert_eq!(
        contract_keys("**Foo 2.** c\n\n**Foo.** a\n\n**Foo.** b"),
        ["foo-2", "foo", "foo-3"]
    );
    // Without a lead that takes a number, the repeats number on from 2.
    assert_eq!(
        contract_keys("**Foo.** a\n\n**Foo.** b\n\n**Foo.** c"),
        ["foo", "foo-2", "foo-3"]
    );
}

/// The lines of the Constraints claims of `constraints`, each with its tests.
fn constraint_lines(constraints: &str) -> (String, Vec<(usize, Vec<String>)>) {
    let text = capability(&format!("{}\n{MARKER}", fixture::PROSE), constraints);
    let found = claims(&text)
        .into_iter()
        .filter(|(section, ..)| *section == CapabilitySection::Constraints)
        .map(|(_, _, line, tests)| (line, tests))
        .collect();
    (text, found)
}

/// A thematic break and a list item with nothing after its marker state no
/// limitation, so neither is a claim of the Constraints, while an item beside
/// them is one.
#[test]
fn a_thematic_break_and_an_empty_item_in_constraints_are_no_claim_and_an_item_beside_them_is_one() {
    let (text, found) = constraint_lines(&format!(
        "- An item.\n{MARKER}\n\n* * *\n\n- - -\n\n---\n\n___\n\n- \n-\n1.\n\n- The neighbour.\n"
    ));
    assert_eq!(
        found,
        [
            (line_of(&text, "- An item."), tests(&[CASE])),
            (line_of(&text, "- The neighbour."), vec![]),
        ]
    );
    let strict = judged(&text);
    assert_eq!(
        strict
            .iter()
            .filter(|finding| finding.kind.starts_with("strict-claim-"))
            .map(|finding| (finding.kind, finding.location.clone()))
            .collect::<Vec<_>>(),
        [(
            "strict-claim-uncovered",
            Location::Line(line_of(&text, "- The neighbour."))
        )]
    );
}

/// A comment line, such as a marker, ends a paragraph or a table of the
/// Constraints, so the paragraph after a marker is a claim of its own; a list
/// item goes on with the lines indented to its content and not with others.
#[test]
fn a_comment_line_ends_a_constraints_paragraph_and_an_item_goes_on_only_with_its_indented_lines() {
    let (text, found) = constraint_lines(&format!(
        "Para one.\n{MARKER}\nPara two is a separate guarantee with no marker.\n\nPara A.\n<!-- a note -->\nPara B.\n\n- An item.\n{MARKER}\n  More of the item.\n\n- Another item.\n{MARKER}\nNot of the item.\n"
    ));
    let at = |needle: &str| line_of(&text, needle);
    assert_eq!(
        found,
        [
            (at("Para one."), tests(&[CASE])),
            (at("Para two is a separate guarantee"), vec![]),
            (at("Para A."), vec![]),
            (at("Para B."), vec![]),
            (at("- An item."), tests(&[CASE])),
            (at("- Another item."), tests(&[CASE])),
            (at("Not of the item."), vec![]),
        ]
    );
}

/// Every heading ends a paragraph of the Contract, so a bold lead directly
/// after a `#####` heading starts a clause, while the heading itself, below
/// `####`, starts none.
#[test]
fn a_bold_lead_directly_after_any_heading_starts_a_contract_clause() {
    assert_eq!(
        contract_keys(&format!(
            "Some opening words.\n{MARKER}\n##### A small heading\n**After the heading.** It holds.\n{MARKER}"
        )),
        ["opening", "after-the-heading"]
    );
}
