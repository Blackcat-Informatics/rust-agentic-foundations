// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Tracked Markdown outside the document directories, judged by
//! `raf_workplan::plain::check`, and the marker errors a document and a plain
//! file share.

mod support;

use raf_okf::Location;
use raf_workplan::Finding;
use support::fixtures::{OVERVIEW, mutate, mutated, valid};
use support::{assert_clean, assert_hit, plain_check, with_file};

/// Require exactly one finding, of `kind`, in `file` at `line`.
fn assert_one(findings: &[Finding], kind: &str, file: &str, line: usize) {
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind, kind, "{findings:#?}");
    assert_eq!(findings[0].file, file);
    assert_eq!(findings[0].location, Location::Line(line), "{findings:#?}");
}

#[test]
fn rejects_a_link_split_across_lines_in_any_tracked_markdown_file() {
    for path in [
        "README.md",
        "docs/notes.md",
        ".github/PULL_REQUEST_TEMPLATE.md",
        "crates/tool/README.md",
    ] {
        for (text, line, form) in [
            ("# Notes\n\n[text\nmore](README.md)\n", 3, "Inline"),
            ("# Notes\n\n[a](\nREADME.md)\n", 3, "Inline"),
            ("# Notes\n\ntext\n\n![alt\ntext](README.md)\n", 5, "Image"),
        ] {
            let findings = with_file(path, text);
            assert_one(&findings, "okf-split-link", path, line);
            assert!(findings[0].message.contains(form), "{findings:#?}");
        }
    }
}

#[test]
fn rejects_a_link_split_across_lines_in_a_profile_document() {
    let findings = mutated(
        OVERVIEW,
        "Body text.",
        "Body text [split\nlink](../../README.md).",
    );
    assert_one(&findings, "okf-split-link", OVERVIEW, 10);
}

#[test]
fn accepts_one_line_links_and_links_in_fences_beside_the_split_rule() {
    let text = "# Notes\n\n[a](README.md) and [b](README.md)\n[c](README.md)\n\n\
                ```text\n[split\nlink](missing.md)\n```\n\nA `[code span](missing.md)` too.\n";
    assert_clean(&with_file("NOTES.md", text));
}

#[test]
fn does_not_read_links_or_placeholders_inside_html_comments() {
    let broken = "<!-- [gone](missing.md) <a href=\"missing.md\">x</a> -->\n";
    let multi = "<!--\n[gone](missing.md)\n[ref]: ./missing.md\nstill hidden\n-->\n";
    let token = concat!("T", "BD");
    let hidden = format!("<!-- {token} and {} -->\n", concat!("to", "do"));
    for body in [broken, multi, hidden.as_str()] {
        assert_clean(&with_file("NOTES.md", &format!("# Notes\n\n{body}")));
    }
    let findings = mutated(
        OVERVIEW,
        "Body text.",
        &format!("Body text.\n\n{broken}{hidden}"),
    );
    assert_clean(&findings);
    let visible = format!("# Notes\n\n<!-- hidden -->\n[gone](missing.md) {token}\n");
    let findings = with_file("NOTES.md", &visible);
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert_hit(&findings, "link-broken", "NOTES.md", &Location::Line(4));
    assert_hit(&findings, "okf-placeholder", "NOTES.md", &Location::Line(4));
}

#[test]
fn reads_placeholders_in_plain_markdown_outside_code() {
    let token = concat!("fix", "me");
    let findings = with_file("NOTES.md", &format!("# Notes\n\nStill {token} here.\n"));
    assert_one(&findings, "okf-placeholder", "NOTES.md", 3);
    assert!(
        findings[0]
            .message
            .contains(&format!("placeholder `{token}` is not resolved text"))
    );
    let spared = format!("# Notes\n\nUse `{token}` and {token}s.\n\n```text\n{token}\n```\n");
    assert_clean(&with_file("NOTES.md", &spared));
    // Markdown that is not in the scanned places is not read for them.
    assert_clean(&with_file(
        "crates/tool/README.md",
        &format!("# Tool\n\nStill {token} here.\n"),
    ));
}

#[test]
fn reports_marker_errors_of_plain_markdown_once() {
    for (text, kind) in [
        (
            "# Notes\n\n> [!PROPOSED add-alpha]\n",
            "okf-parse-malformed-proposed",
        ),
        (
            "# Notes\n\n<!-- raf:generated:matrix -->\n",
            "okf-parse-unclosed-generated",
        ),
        (
            "# Notes\n\n<!-- raf:generated:end -->\n",
            "okf-parse-orphan-generated-end",
        ),
    ] {
        assert_one(&with_file("NOTES.md", text), kind, "NOTES.md", 3);
    }
    assert_one(
        &with_file(".github/note.md", "# Note\n\n> [!PROPOSED add-alpha]\n"),
        "okf-parse-malformed-proposed",
        ".github/note.md",
        3,
    );
}

#[test]
fn reads_a_proposed_block_in_plain_markdown_as_outside_a_capability() {
    let text = "# Notes\n\n> [!PROPOSED change:add-alpha]\n> Candidate text.\n";
    let findings = with_file("NOTES.md", text);
    assert_one(&findings, "okf-proposed-outside-capability", "NOTES.md", 3);
    assert!(findings[0].message.contains("`add-alpha`"));
    // A block whose change does not exist is also an orphan.
    let orphan = text.replace("add-alpha", "ghost");
    let findings = with_file("NOTES.md", &orphan);
    assert_hit(
        &findings,
        "proposed-missing-change",
        "NOTES.md",
        &Location::Line(3),
    );
    assert_hit(
        &findings,
        "okf-proposed-outside-capability",
        "NOTES.md",
        &Location::Line(3),
    );
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert_clean(&with_file(
        "crates/tool/README.md",
        "# Tool\n\n> [!PROPOSED change:add-alpha]\n> Candidate text.\n",
    ));
}

#[test]
fn reads_markdown_outside_the_document_directories_with_or_without_frontmatter() {
    let template = "---\nname: Bug\nabout: Report a bug\ntitle: \"[Bug] \"\n---\n\n# Bug\n\nSee [the overview](../../docs/architecture/overview.md).\n";
    assert_clean(&with_file(".github/ISSUE_TEMPLATE/bug.md", template));
    let broken = template.replace("overview.md", "missing.md");
    assert_one(
        &with_file(".github/ISSUE_TEMPLATE/bug.md", &broken),
        "link-broken",
        ".github/ISSUE_TEMPLATE/bug.md",
        9,
    );
}

#[test]
fn rejects_unclosed_frontmatter_in_plain_markdown() {
    assert_one(
        &with_file("NOTES.md", "---\n# Notes\n\nNo closing fence.\n"),
        "okf-parse-unclosed-fence",
        "NOTES.md",
        1,
    );
}

#[test]
fn rejects_generated_marker_errors() {
    for (text, kind) in [
        (
            "Body text.\n\n<!-- raf:generated:end -->\n",
            "okf-parse-orphan-generated-end",
        ),
        (
            "Body text.\n\n<!-- raf:generated:a -->\n<!-- raf:generated:b -->\n<!-- raf:generated:end -->\n",
            "okf-parse-nested-generated",
        ),
        (
            "Body text.\n\n<!-- raf:generated: -->\n",
            "okf-parse-malformed-generated",
        ),
        (
            "Body text.\n\n<!-- raf:generated:Bad Kind -->\n",
            "okf-parse-malformed-generated",
        ),
        (
            "Body text.\n\n<!-- raf:generated:matrix\n",
            "okf-parse-malformed-generated",
        ),
    ] {
        let repo = mutate(OVERVIEW, "Body text.", text);
        let findings = support::check(&repo);
        assert_eq!(findings.len(), 1, "{text}: {findings:#?}");
        assert_eq!(findings[0].kind, kind, "{text}: {findings:#?}");
        assert_eq!(findings[0].file, OVERVIEW);
    }
}

#[test]
fn a_broken_link_and_a_split_link_are_reported_together_for_a_plain_file() {
    // Both rules apply to one file, in the order of the file.
    let text = "# Notes\n\n[gone](nope.md) and [a](\nREADME.md)\n";
    let mut repo = valid();
    repo.insert("NOTES.md".to_string(), text.to_string());
    let plain = plain_check(&repo, "NOTES.md", &[]);
    let mut kinds: Vec<&str> = plain.iter().map(|f| f.kind).collect();
    kinds.sort_unstable();
    assert_eq!(kinds, ["link-broken", "okf-split-link"]);
}

#[test]
fn an_index_in_a_document_directory_other_than_the_capability_index_is_read_as_markdown() {
    let token = concat!("to", "do");
    for path in ["docs/architecture/index.md", "work/changes/index.md"] {
        let case = |text: &str| {
            let mut repo = valid();
            repo.insert(path.to_string(), text.to_string());
            support::check(&repo)
        };
        assert_clean(&case(
            "# Index\n\n[overview](../../docs/architecture/overview.md) and [home](../../README.md)\n",
        ));
        assert_one(
            &case("# Index\n\n[gone](missing.md)\n"),
            "link-broken",
            path,
            3,
        );
        assert_one(
            &case(&format!("# Index\n\nStill {token}.\n")),
            "okf-placeholder",
            path,
            3,
        );
        assert_one(
            &case("# Index\n\n[a](\nREADME.md)\n"),
            "okf-split-link",
            path,
            3,
        );
        assert_one(
            &case("# Index\n\n<!-- raf:generated:end -->\n"),
            "okf-parse-orphan-generated-end",
            path,
            3,
        );
        let findings = case("# Index\n\n> [!PROPOSED change:ghost]\n> Text.\n");
        assert_hit(
            &findings,
            "okf-proposed-outside-capability",
            path,
            &Location::Line(3),
        );
        assert_hit(
            &findings,
            "proposed-missing-change",
            path,
            &Location::Line(3),
        );
        assert_eq!(findings.len(), 2, "{findings:#?}");
    }
}

#[test]
fn a_claim_marker_outside_a_capability_is_reported_and_a_misspelt_marker_is_refused() {
    let claim = "# Notes\n\nIt holds.\n<!-- raf:claim evidence=bound -->\n";
    assert_one(
        &with_file("docs/notes.md", claim),
        "okf-claim-outside-capability",
        "docs/notes.md",
        4,
    );
    let unknown = "# Notes\n\nIt holds.\n<!-- raf:clam evidence=bound -->\n";
    assert_one(
        &with_file("docs/notes.md", unknown),
        "okf-parse-unknown-marker",
        "docs/notes.md",
        4,
    );
    // The syntax written in a code span is no marker.
    assert_clean(&with_file(
        "docs/notes.md",
        "# Notes\n\nWrite `<!-- raf:claim evidence=bound -->` after it.\n",
    ));
}

#[test]
fn a_tested_by_marker_outside_a_capability_is_reported_and_a_malformed_one_is_refused() {
    let marker = "# Notes\n\nIt holds.\n<!-- raf:tested-by demo::test_crate::it::case -->\n";
    assert_one(
        &with_file("docs/notes.md", marker),
        "okf-tested-by-outside-capability",
        "docs/notes.md",
        4,
    );
    let malformed = "# Notes\n\nIt holds.\n<!-- raf:tested-by demo::a, demo::b -->\n";
    assert_one(
        &with_file("docs/notes.md", malformed),
        "okf-parse-malformed-tested-by",
        "docs/notes.md",
        4,
    );
    // The syntax written in a code span is no marker.
    assert_clean(&with_file(
        "docs/notes.md",
        "# Notes\n\nWrite `<!-- raf:tested-by demo::test_crate::it::case -->` after it.\n",
    ));
}
