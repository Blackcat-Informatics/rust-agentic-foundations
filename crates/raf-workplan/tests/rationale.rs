// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Current rationale: an architecture page is linked, a decision page and a
//! decision of an Explanation end with their conditions to revisit, and a
//! link to a page or a heading that does not exist fails the link check.

mod support;

use std::fmt::Write as _;
use std::path::Path;
use std::time::{Duration, Instant};

use raf_okf::Location;
use raf_source::MemoryTree;
use raf_workplan::{Finding, Gate, LinkEnd, Workplan};
use support::cases::{unlinked, with_explanation_text, with_page_text};
use support::fixtures::{BETA, CHANGE, GAMMA, OVERVIEW, OVERVIEW_LINK, mutate, valid};
use support::{Repo, assert_clean, check, demo_workspace, line_of};

const RULES: [&str; 3] = [
    "architecture-page-unlinked",
    "decision-conditions-missing",
    "explanation-conditions-missing",
];

/// The findings of the three rules of current rationale.
fn rationale(repo: &Repo) -> Vec<Finding> {
    check(repo)
        .into_iter()
        .filter(|finding| RULES.contains(&finding.kind))
        .collect()
}

fn at(repo: &Repo, path: &str, needle: &str) -> Location {
    Location::Line(line_of(&repo[path], needle))
}

/// The one finding of `findings`, which must be of `kind` in `file` at `at`.
fn only(findings: &[Finding], kind: &str, file: &str, at: &Location) -> Finding {
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let finding = &findings[0];
    assert_eq!(
        (finding.kind, finding.file.as_str(), &finding.location),
        (kind, file, at),
        "{finding:#?}"
    );
    finding.clone()
}

const DECISION: &str = "## Decision\n\nThe pages are flat.\n";
const CONDITIONS: &str = "## Conditions to revisit\n\n- A page needs a subdirectory.\n";

#[test]
fn a_decision_page_linked_from_a_capability_with_its_conditions_passes() {
    let repo = with_page_text(&format!("{DECISION}\n{CONDITIONS}"));
    assert_clean(&check(&repo));
    // The link is an edge from gamma's Explanation to the page.
    let memory: MemoryTree = repo.iter().collect();
    let workspace = demo_workspace();
    let plan = Workplan::load(&memory, &workspace);
    assert!(plan.graph().links().iter().any(|edge| matches!(
        (edge.from(), edge.to()),
        (LinkEnd::Section(from), LinkEnd::Architecture(to))
            if from.to_string() == "cap:gamma#explanation" && to == "arch:overview"
    )));
}

#[test]
fn an_unlinked_page_is_reported_by_name_and_a_link_to_itself_does_not_count() {
    let repo = unlinked();
    let finding = only(
        &check(&repo),
        "architecture-page-unlinked",
        OVERVIEW,
        &Location::Document,
    );
    assert!(finding.message.contains("arch:overview"), "{finding:#?}");
    assert!(finding.fix.contains("link the page"), "{finding:#?}");

    // A link from the page to itself, with or without an anchor, is no link.
    for own in [
        "See [this page](overview.md).",
        "See [the top](#overview).",
        "See [this page](overview.md#overview).",
    ] {
        let mut repo = unlinked();
        let page = repo.get_mut(OVERVIEW).expect("the page");
        *page = page.replacen("Body text.", &format!("Body text. {own}"), 1);
        only(
            &check(&repo),
            "architecture-page-unlinked",
            OVERVIEW,
            &Location::Document,
        );
    }
}

#[test]
fn a_link_from_a_capability_a_change_or_another_page_links_the_page() {
    // A capability section other than the Explanation, with an anchor.
    let mut from_usage = unlinked();
    let gamma = from_usage.get_mut(GAMMA).expect("gamma");
    *gamma = gamma.replacen(
        "Text for Usage.",
        "Text for Usage. See [the overview](../architecture/overview.md#overview).",
        1,
    );
    assert_clean(&check(&from_usage));

    // A change record.
    let mut from_change = unlinked();
    let change = from_change.get_mut(CHANGE).expect("the change");
    *change = change.replacen(
        "Adds both.",
        "Adds both. See [the overview](../../docs/architecture/overview.md).",
        1,
    );
    assert_clean(&check(&from_change));

    // Another architecture page links it; that page is itself unlinked.
    let mut from_page = unlinked();
    from_page.insert(
        "docs/architecture/layers.md".to_string(),
        "---\ntype: Architecture\nid: arch:layers\ntitle: Layers\n\
         description: The layers.\n---\n\n# Layers\n\nSee [the overview](overview.md).\n"
            .to_string(),
    );
    let finding = only(
        &check(&from_page),
        "architecture-page-unlinked",
        "docs/architecture/layers.md",
        &Location::Document,
    );
    assert!(finding.message.contains("arch:layers"), "{finding:#?}");

    // A link in Markdown that is not a loaded document does not count: the
    // README of the fixture links nothing here, and a link in a code span or
    // a comment of a capability is no link.
    for hidden in [
        "`[the overview](../architecture/overview.md)`",
        "<!-- [the overview](../architecture/overview.md) -->",
    ] {
        let mut repo = unlinked();
        let gamma = repo.get_mut(GAMMA).expect("gamma");
        *gamma = gamma.replacen(
            "Text for Explanation.",
            &format!("Text for Explanation. {hidden}"),
            1,
        );
        only(
            &check(&repo),
            "architecture-page-unlinked",
            OVERVIEW,
            &Location::Document,
        );
    }
    let mut readme = unlinked();
    readme.insert(
        "README.md".to_string(),
        "# Readme\n\nSee [the overview](docs/architecture/overview.md).\n".to_string(),
    );
    only(
        &check(&readme),
        "architecture-page-unlinked",
        OVERVIEW,
        &Location::Document,
    );
}

#[test]
fn no_page_is_reported_unlinked_while_a_document_that_may_link_it_does_not_load() {
    // Gamma holds the only link; while gamma does not load, its own finding
    // is the one reported.
    let broken = mutate(GAMMA, "  code: [demo::real]", "  code: []");
    let kinds: Vec<&str> = check(&broken).iter().map(|f| f.kind).collect();
    assert_eq!(kinds, ["okf-binding-required"]);
    // Once gamma loads without the link, the page is reported.
    only(
        &check(&unlinked()),
        "architecture-page-unlinked",
        OVERVIEW,
        &Location::Document,
    );
}

#[test]
fn a_decision_page_without_conditions_to_revisit_is_reported_by_name() {
    let repo = with_page_text(DECISION);
    let finding = only(
        &check(&repo),
        "decision-conditions-missing",
        OVERVIEW,
        &at(&repo, OVERVIEW, "## Decision"),
    );
    assert!(finding.message.contains("arch:overview"), "{finding:#?}");
    assert!(
        finding.message.contains("Conditions to revisit"),
        "{finding:#?}"
    );
    assert!(
        finding.fix.contains("reopening the decision"),
        "{finding:#?}"
    );
    // The passing neighbour: the same page with its conditions, before or
    // after the decision.
    assert_clean(&check(&with_page_text(&format!(
        "{DECISION}\n{CONDITIONS}"
    ))));
    assert_clean(&check(&with_page_text(&format!(
        "{CONDITIONS}\n{DECISION}"
    ))));
    // A page without a decision needs no conditions.
    assert_clean(&check(&with_page_text("## Notes\n\nNo decision here.\n")));
}

#[test]
fn an_empty_conditions_list_is_reported_and_a_list_item_with_text_passes() {
    for empty in [
        "",
        "None come to mind.\n",
        "- \n",
        "* * *\n",
        "-x\n",
        "```\n- In a fence.\n```\n",
        "<!-- - In a comment. -->\n",
        "### Details\n\n",
    ] {
        let repo = with_page_text(&format!("{DECISION}\n## Conditions to revisit\n\n{empty}"));
        let finding = only(
            &rationale(&repo),
            "decision-conditions-missing",
            OVERVIEW,
            &at(&repo, OVERVIEW, "## Conditions to revisit"),
        );
        assert!(finding.message.contains("arch:overview"), "{finding:#?}");
    }
    // An empty list is reported on a page without a decision too.
    let repo = with_page_text("## Conditions to revisit\n\nNothing.\n");
    only(
        &rationale(&repo),
        "decision-conditions-missing",
        OVERVIEW,
        &at(&repo, OVERVIEW, "## Conditions to revisit"),
    );
    // The list ends at the next second-level heading.
    let repo = with_page_text(&format!(
        "{DECISION}\n## Conditions to revisit\n\nNothing.\n\n## Later\n\n- Not a condition.\n"
    ));
    only(
        &rationale(&repo),
        "decision-conditions-missing",
        OVERVIEW,
        &at(&repo, OVERVIEW, "## Conditions to revisit"),
    );
    for item in [
        "- A page needs a subdirectory.\n",
        "* Two pages share a decision.\n",
        "+ 3\n",
        "1. The profile gains a type.\n",
        "2) The profile gains a type.\n",
        "  - Indented.\n",
        "- `cargo` changes and the pages follow.\n",
        "### Details\n\n- Under a lower heading.\n",
    ] {
        let repo = with_page_text(&format!("{DECISION}\n## Conditions to revisit\n\n{item}"));
        assert_clean(&check(&repo));
    }
}

#[test]
fn headings_match_exactly_at_their_level() {
    // Only `## Decision` makes a decision page: case, other text and other
    // levels do not.
    for not_a_decision in [
        "## decision\n\nText.\n",
        "## Decisions\n\nText.\n",
        "## The Decision\n\nText.\n",
        "### Decision\n\nText.\n",
        "# Decision\n\nText.\n",
    ] {
        assert_clean(&check(&with_page_text(not_a_decision)));
    }
    // Spaces around the text and closing marks are not part of it.
    for spelled in ["##   Decision  \n\nText.\n", "## Decision ##\n\nText.\n"] {
        let repo = with_page_text(spelled);
        only(
            &rationale(&repo),
            "decision-conditions-missing",
            OVERVIEW,
            &at(&repo, OVERVIEW, "Decision"),
        );
    }
    // A conditions heading at another level or in another case does not
    // answer a decision.
    for wrong in [
        "### Conditions to revisit\n\n- A condition.\n",
        "## Conditions To Revisit\n\n- A condition.\n",
    ] {
        let repo = with_page_text(&format!("{DECISION}\n{wrong}"));
        only(
            &rationale(&repo),
            "decision-conditions-missing",
            OVERVIEW,
            &at(&repo, OVERVIEW, "## Decision"),
        );
    }
}

#[test]
fn an_explanation_decision_without_conditions_is_reported_and_with_them_passes() {
    let decided = "### Decision\n\nGamma reads the whole input.\n";
    let conditions = "### Conditions to revisit\n\n- Input arrives in chunks.\n";
    let repo = with_explanation_text(decided);
    let finding = only(
        &check(&repo),
        "explanation-conditions-missing",
        GAMMA,
        &at(&repo, GAMMA, "### Decision"),
    );
    assert!(finding.message.contains("cap:gamma"), "{finding:#?}");
    assert!(
        finding.message.contains("Conditions to revisit"),
        "{finding:#?}"
    );
    assert_clean(&check(&with_explanation_text(&format!(
        "{decided}\n{conditions}"
    ))));

    // Each decision needs its own conditions before the next decision.
    let two = format!("{decided}\n{decided}\n{conditions}");
    let repo = with_explanation_text(&two);
    let first = line_of(&repo[GAMMA], "### Decision");
    only(
        &rationale(&repo),
        "explanation-conditions-missing",
        GAMMA,
        &Location::Line(first),
    );
    assert_clean(&check(&with_explanation_text(&format!(
        "{decided}\n{conditions}\n{decided}\n{conditions}"
    ))));

    // Conditions after the Explanation, in Verification, do not follow it.
    let mut late = with_explanation_text(decided);
    let gamma = late.get_mut(GAMMA).expect("gamma");
    *gamma = gamma.replacen(
        "Text for Verification.",
        &format!("Text for Verification.\n\n{conditions}"),
        1,
    );
    only(
        &rationale(&late),
        "explanation-conditions-missing",
        GAMMA,
        &at(&late, GAMMA, "### Decision"),
    );

    // A conditions subheading whose list is empty is reported at itself.
    let repo = with_explanation_text(&format!(
        "{decided}\n### Conditions to revisit\n\nNone.\n\n### Notes\n\n- Not a condition.\n"
    ));
    only(
        &rationale(&repo),
        "explanation-conditions-missing",
        GAMMA,
        &at(&repo, GAMMA, "### Conditions to revisit"),
    );

    // A `### Decision` in another section is not judged.
    let mut usage = valid();
    let gamma = usage.get_mut(GAMMA).expect("gamma");
    *gamma = gamma.replacen(
        "Text for Usage.",
        "Text for Usage.\n\n### Decision\n\nNot rationale.",
        1,
    );
    assert_clean(&check(&usage));
}

#[test]
fn headings_and_list_items_in_fenced_code_or_proposed_blocks_do_not_count() {
    // A decision in fenced code, closed or running to the end of the body.
    for fenced in [
        "```markdown\n## Decision\n\nIn a fence.\n```\n",
        "~~~\n## Decision\n~~~\n",
        "```\n## Decision\n\nThe fence is never closed.\n",
    ] {
        assert!(rationale(&with_page_text(fenced)).is_empty(), "{fenced}");
    }
    // Conditions in fenced code do not answer a decision outside it.
    let repo = with_page_text(&format!(
        "{DECISION}\n```\n## Conditions to revisit\n\n- In a fence.\n```\n"
    ));
    only(
        &rationale(&repo),
        "decision-conditions-missing",
        OVERVIEW,
        &at(&repo, OVERVIEW, "## Decision"),
    );
    // An Explanation decision in fenced code.
    let repo = with_explanation_text("```\n### Decision\n```\n");
    assert_clean(&rationale(&repo));

    // In a PROPOSED block of beta's Explanation: a decision is not one until
    // the block is resolved, and a list item of the block is no condition.
    let proposed = |text: &str| {
        let mut repo = mutate(
            CHANGE,
            "  - cap:beta#purpose\n",
            "  - cap:beta#purpose\n  - cap:beta#explanation\n",
        );
        let beta = repo.get_mut(BETA).expect("beta");
        *beta = beta.replacen(
            "Text for Explanation.",
            &format!("Text for Explanation.\n\n{text}"),
            1,
        );
        repo
    };
    let repo = proposed("> [!PROPOSED change:add-alpha]\n> ### Decision\n>\n> Beta reads lines.\n");
    assert_clean(&check(&repo));
    let repo = proposed(
        "### Decision\n\nBeta reads lines.\n\n### Conditions to revisit\n\n\
         > [!PROPOSED change:add-alpha]\n> - Input arrives in chunks.\n",
    );
    only(
        &rationale(&repo),
        "explanation-conditions-missing",
        BETA,
        &at(&repo, BETA, "### Conditions to revisit"),
    );
    // The resolved neighbour passes.
    let repo = proposed(
        "### Decision\n\nBeta reads lines.\n\n### Conditions to revisit\n\n- Input arrives in chunks.\n",
    );
    assert_clean(&check(&repo));
}

#[test]
fn a_capability_link_to_a_missing_page_or_heading_fails_the_link_check() {
    let missing_page = mutate(
        GAMMA,
        "../architecture/overview.md",
        "../architecture/gone.md",
    );
    let findings = check(&missing_page);
    let line = line_of(&missing_page[GAMMA], "gone.md");
    assert!(
        findings
            .iter()
            .any(|f| f.kind == "link-broken" && f.file == GAMMA && f.line() == Some(line)),
        "{findings:#?}"
    );
    // The broken link links nothing, so the page is unlinked too.
    let kinds: Vec<&str> = findings.iter().map(|f| f.kind).collect();
    assert_eq!(
        kinds,
        ["link-broken", "architecture-page-unlinked"],
        "{findings:#?}"
    );

    let missing_heading = mutate(
        GAMMA,
        "../architecture/overview.md",
        "../architecture/overview.md#decision",
    );
    let findings = check(&missing_heading);
    let line = line_of(&missing_heading[GAMMA], "overview.md#decision");
    assert!(
        findings
            .iter()
            .any(|f| f.kind == "anchor-broken" && f.file == GAMMA && f.line() == Some(line)),
        "{findings:#?}"
    );
    let kinds: Vec<&str> = findings.iter().map(|f| f.kind).collect();
    assert_eq!(
        kinds,
        ["anchor-broken", "architecture-page-unlinked"],
        "{findings:#?}"
    );

    // The neighbour: the heading exists once the page has it.
    let mut present = missing_heading;
    let page = present.get_mut(OVERVIEW).expect("the page");
    *page = page.replacen(
        "Body text.\n",
        &format!("Body text.\n\n{DECISION}\n{CONDITIONS}"),
        1,
    );
    assert_clean(&check(&present));
}

#[test]
fn the_findings_are_the_same_on_every_run_in_file_and_line_order() {
    let mut repo = unlinked();
    let page = repo.get_mut(OVERVIEW).expect("the page");
    *page = page.replacen(
        "Body text.\n",
        "Body text.\n\n## Conditions to revisit\n\nNone.\n\n## Decision\n\nFlat.\n\n\
         ## Conditions to revisit\n\nStill none.\n",
        1,
    );
    let gamma = repo.get_mut(GAMMA).expect("gamma");
    *gamma = gamma.replacen(
        "Text for Explanation.",
        "Text for Explanation.\n\n### Decision\n\nOne.\n\n### Decision\n\nTwo.\n",
        1,
    );
    let first = rationale(&repo);
    assert_eq!(first, rationale(&repo));
    let summary: Vec<(&str, &str, Option<usize>)> = first
        .iter()
        .map(|f| (f.kind, f.file.as_str(), f.line()))
        .collect();
    let gamma_text = &repo[GAMMA];
    let decisions: Vec<usize> = gamma_text
        .lines()
        .enumerate()
        .filter(|(_, line)| *line == "### Decision")
        .map(|(index, _)| index + 1)
        .collect();
    let page_text = &repo[OVERVIEW];
    let empty: Vec<usize> = page_text
        .lines()
        .enumerate()
        .filter(|(_, line)| *line == "## Conditions to revisit")
        .map(|(index, _)| index + 1)
        .collect();
    assert_eq!(
        summary,
        [
            ("architecture-page-unlinked", OVERVIEW, None),
            ("decision-conditions-missing", OVERVIEW, Some(empty[0])),
            ("decision-conditions-missing", OVERVIEW, Some(empty[1])),
            ("explanation-conditions-missing", GAMMA, Some(decisions[0])),
            ("explanation-conditions-missing", GAMMA, Some(decisions[1])),
        ]
    );
}

#[test]
fn a_hundred_thousand_headings_and_an_unterminated_fence_are_judged_in_bounded_time() {
    // A decision page with a hundred thousand headings under its decision,
    // its conditions at the end.
    let mut page = String::from(DECISION);
    for index in 0..100_000 {
        writeln!(page, "### H{index}").expect("a string takes the text");
    }
    page.push('\n');
    page.push_str(CONDITIONS);
    let repo = with_page_text(&page);
    let started = Instant::now();
    assert_clean(&rationale(&repo));
    // The same page with its conditions behind an unterminated fence.
    let fenced = with_page_text(&format!(
        "{DECISION}\n{}```\n{CONDITIONS}",
        "### H\n".repeat(100_000)
    ));
    let findings = rationale(&fenced);
    only(
        &findings,
        "decision-conditions-missing",
        OVERVIEW,
        &at(&fenced, OVERVIEW, "## Decision"),
    );
    // An Explanation with twenty thousand decisions, each with its conditions,
    // and twenty thousand empty conditions headings on a page.
    let pairs = "### Decision\n\nD.\n\n### Conditions to revisit\n\n- C.\n\n".repeat(20_000);
    assert_clean(&rationale(&with_explanation_text(&pairs)));
    let empty = with_page_text(&"## Conditions to revisit\n\n".repeat(20_000));
    assert_eq!(rationale(&empty).len(), 20_000);
    assert!(
        started.elapsed() < Duration::from_secs(60),
        "took {:?}",
        started.elapsed()
    );
}

#[test]
fn this_repository_links_each_architecture_page_from_a_capability() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let opened = Gate::workspace(&root).open().expect("the repository opens");
    let plan = opened.workplan();
    assert_eq!(plan.architecture().len(), 2);
    for page in plan.architecture() {
        let id = page.value().header.id.as_str();
        assert!(
            plan.graph().links().iter().any(|edge| matches!(
                (edge.from(), edge.to()),
                (LinkEnd::Section(_), LinkEnd::Architecture(to)) if to == id
            )),
            "no capability links {id}"
        );
    }
    let findings: Vec<Finding> = plan
        .check()
        .into_iter()
        .filter(|finding| RULES.contains(&finding.kind))
        .collect();
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn the_valid_fixture_links_its_page_from_gamma() {
    assert!(valid()[GAMMA].contains(OVERVIEW_LINK));
    assert_clean(&check(&valid()));
}
