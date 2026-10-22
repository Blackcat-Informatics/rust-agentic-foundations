// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Review probes of the claim units (scratch only).

mod support;

use raf_source::MemoryTree;
use raf_workplan::{Strictness, Workplan};
use support::strict as fixture;
use support::{Repo, check_in_with, workspace_of};

const MARKER: &str = fixture::MARKER;

fn capability(contract: &str, constraints: &str) -> String {
    let old_contract = format!("## Contract\n\n{}\n{MARKER}\n", fixture::PROSE);
    let old_constraints = format!(
        "## Constraints\n\n{}\n{MARKER}\n\n{}\n{MARKER}\n",
        fixture::PROSE,
        fixture::LIMITATION
    );
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

fn probe(label: &str, contract: &str, constraints: &str) {
    let text = capability(contract, constraints);
    let repo = Repo::from([
        (fixture::CAPABILITY.to_string(), text.clone()),
        (fixture::COVERAGE.to_string(), fixture::coverage("")),
    ]);
    let tree: MemoryTree = repo.iter().collect();
    let source = workspace_of(&fixture::source_files());
    let report = Workplan::load(&tree, &source).claims_report();
    let lines: Vec<&str> = text.lines().collect();
    println!("PROBE {label}");
    for claim in report.claims() {
        let first = lines.get(claim.line() - 1).copied().unwrap_or("?");
        println!(
            "  {} {} line {} {:?} tests={:?}",
            claim.section().slug(),
            claim.claim(),
            claim.line(),
            first,
            claim.tests()
        );
    }
    for finding in check_in_with(&repo, &source, Strictness::Strict) {
        println!("  FINDING {} {:?} {}", finding.kind, finding.location, finding.message);
    }
}

fn clean_constraints() -> String {
    format!("{}\n{MARKER}\n\n{}\n{MARKER}", fixture::PROSE, fixture::LIMITATION)
}

fn covered_contract() -> String {
    format!("{}\n{MARKER}", fixture::PROSE)
}

#[test]
fn probes() {
    let p = fixture::PROSE;
    probe(
        "nested-list-in-contract",
        &format!("**Lead.** {p}\n\n- item\n  - nested **Bold.** x\n\n  **Indented.** in item\n{MARKER}"),
        &clean_constraints(),
    );
    probe(
        "setext-heading-in-contract",
        &format!("Opening words.\n\nA setext heading\n----------------\n\n**Lead.** {p}\n{MARKER}"),
        &clean_constraints(),
    );
    probe(
        "setext-level1-in-contract",
        &format!("Opening words.\n\nA setext heading\n================\n\n**Lead.** {p}\n{MARKER}"),
        &clean_constraints(),
    );
    probe(
        "bold-run-mid-paragraph",
        &format!("**Lead.** {p}\nand then\n**Not a lead.** continues the paragraph.\n{MARKER}"),
        &clean_constraints(),
    );
    probe(
        "bold-after-comment-line",
        &format!("**Lead.** {p}\n<!-- a note -->\n**Second.** after a comment.\n{MARKER}"),
        &clean_constraints(),
    );
    probe(
        "html-block",
        &format!("<div>\n**Inside div.** text\n</div>\n\n**Lead.** {p}\n{MARKER}"),
        &clean_constraints(),
    );
    probe(
        "underscore-emphasis-lead",
        &format!("__Underscore lead.__ {p}\n{MARKER}"),
        &clean_constraints(),
    );
    probe(
        "italic-star-lead",
        &format!("*Italic lead.* {p}\n{MARKER}"),
        &clean_constraints(),
    );
    probe(
        "bold-italic-lead",
        &format!("***Bold italic.*** {p}\n{MARKER}"),
        &clean_constraints(),
    );
    probe(
        "thematic-break-stars",
        &format!("**Lead.** {p}\n{MARKER}\n\n** * **\n\nAfter the break."),
        &clean_constraints(),
    );
    probe(
        "h5-then-bold",
        &format!("**Lead.** {p}\n{MARKER}\n\n##### Five\n**Bold after h5.** text"),
        &clean_constraints(),
    );
    probe(
        "key-collision",
        &format!("**Foo.** a\n{MARKER}\n\n**Foo.** b\n{MARKER}\n\n**Foo 2.** c\n{MARKER}"),
        &clean_constraints(),
    );
    probe(
        "unicode-lead",
        &format!("**Caf\u{e9} \u{dc}ber.** {p}\n{MARKER}"),
        &clean_constraints(),
    );
    probe(
        "only-fence-in-contract",
        "```text\n**Not.** x\n```",
        &clean_constraints(),
    );
    probe(
        "long-opening-plus-lead",
        &format!("{p} {p}\n\n**Lead.** x\n{MARKER}"),
        &clean_constraints(),
    );
    // Constraints.
    probe(
        "table-inside-list",
        &covered_contract(),
        &format!("- item\n\n  | a | b |\n  | --- | --- |\n  | c | d |\n{MARKER}\n- second\n{MARKER}"),
    );
    probe(
        "nested-list-constraints",
        &covered_contract(),
        &format!("- top\n  - nested one\n    - deeper\n  - nested two\n{MARKER}\n\n  continued para\n- next\n{MARKER}"),
    );
    probe(
        "thematic-break-constraints",
        &covered_contract(),
        &format!("- item\n{MARKER}\n\n* * *\n\n- - -\n\n---"),
    );
    probe(
        "empty-items-constraints",
        &covered_contract(),
        &format!("- item\n{MARKER}\n-\n- \n"),
    );
    probe(
        "heading-in-constraints-with-marker",
        &covered_contract(),
        &format!("- item\n{MARKER}\n\n### Sub heading\n{MARKER}\n\nPara under heading.\n{MARKER}"),
    );
    probe(
        "paragraphs-split-by-comment",
        &covered_contract(),
        &format!("- item\n{MARKER}\n\nPara one.\n<!-- a note -->\nPara two.\n"),
    );
    probe(
        "fence-in-constraints",
        &covered_contract(),
        &format!("- item\n{MARKER}\n\n```text\n- not an item\n```\n"),
    );
    probe(
        "html-block-constraints",
        &covered_contract(),
        &format!("- item\n{MARKER}\n\n<details>\n<summary>x</summary>\n\n- inside details\n\n</details>\n"),
    );
    probe(
        "setext-in-constraints",
        &covered_contract(),
        &format!("- item\n{MARKER}\n\nSetext title\n------------\n"),
    );
    probe(
        "tab-list-marker",
        &covered_contract(),
        &format!("-\titem with tab\n{MARKER}\n\n- item\n{MARKER}"),
    );
    probe(
        "emphasis-underscore-constraints",
        &covered_contract(),
        &format!("- _under_ __strong__ item\n{MARKER}"),
    );
}
