// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The generated blocks: the exact bytes of each kind, where each may sit, the
//! marker errors, staleness (an edited, a removed and an added row), the
//! rewrite of each line ending, the text sync leaves alone, and the index.

mod support;

use std::collections::BTreeSet;

use raf_workplan::{Finding, SyncError, SyncOptions, Workplan};
use support::synced::{
    ALPHA, BETA, DELTA, GAMMA, GROW, INDEX, MATRIX, OVERVIEW, RELATIONSHIPS, apply, base,
    check_all, document, generated_findings, index_header, options, sync, synced, unsynced,
    with_plan,
};
use support::{Repo, line_of};

/// The text between the markers of the first block of `kind` in `text`.
fn block(text: &str, kind: &str) -> String {
    let open = format!("<!-- raf:generated:{kind} -->\n");
    let start = text
        .find(&open)
        .unwrap_or_else(|| panic!("no `{kind}` block in:\n{text}"))
        + open.len();
    let end = text[start..]
        .find("<!-- raf:generated:end -->")
        .unwrap_or_else(|| panic!("`{kind}` block is not closed"));
    text[start..start + end].to_string()
}

/// `repo` with the first `from` of `path` replaced by `to`.
fn replaced(repo: &Repo, path: &str, from: &str, to: &str) -> Repo {
    let mut next = repo.clone();
    let text = next
        .get_mut(path)
        .unwrap_or_else(|| panic!("the fixture has no {path}"));
    assert!(text.contains(from), "{path} has no `{from}`");
    *text = text.replacen(from, to, 1);
    next
}

/// The generated findings of `repo` as `(kind, file)`.
fn pairs(repo: &Repo) -> Vec<(&'static str, String)> {
    generated_findings(repo)
        .into_iter()
        .map(|f| (f.kind, f.file))
        .collect()
}

// --- the exact bytes of each kind -------------------------------------------

#[test]
fn the_synced_base_has_no_finding_at_all() {
    let repo = base();
    assert!(check_all(&repo).is_empty(), "{:#?}", check_all(&repo));
}

#[test]
fn a_verification_matrix_lists_each_test_and_example_by_path_with_its_sections() {
    let repo = base();
    // Examples then a test, though the test sorts after both; the section of
    // each is every section other than Verification that names it in a code
    // span, in document order, and Verification when no section does.
    assert_eq!(
        block(&repo[GAMMA], "verification-matrix"),
        "\
| Item | Kind | Section |
| --- | --- | --- |
| `demo::example_crate::show` | example target | [Usage](#usage), [Examples](#examples) |
| `demo::real` | doctest | [Verification](#verification) |
| `demo::test_crate::it::case` | test | [Contract](#contract) |
"
    );
    // Nothing bound: the header only.
    assert_eq!(
        block(&repo[BETA], "verification-matrix"),
        "| Item | Kind | Section |\n| --- | --- | --- |\n"
    );
}

#[test]
fn rows_are_sorted_by_path_whatever_the_order_of_the_lists() {
    let repo = replaced(
        &unsynced(),
        GAMMA,
        "examples: [demo::example_crate::show, demo::real]",
        "examples: [demo::real, demo::example_crate::flagged, demo::example_crate::show]",
    );
    let repo = synced(&repo);
    assert_eq!(
        block(&repo[GAMMA], "verification-matrix"),
        "\
| Item | Kind | Section |
| --- | --- | --- |
| `demo::example_crate::flagged` | example target | [Verification](#verification) |
| `demo::example_crate::show` | example target | [Usage](#usage), [Examples](#examples) |
| `demo::real` | doctest | [Verification](#verification) |
| `demo::test_crate::it::case` | test | [Contract](#contract) |
"
    );
}

#[test]
fn a_path_named_in_a_fence_or_a_proposal_is_not_a_section_that_it_substantiates() {
    let fenced = "\n```text\n`demo::test_crate::it::case`\n```\n";
    let repo = synced(&replaced(
        &unsynced(),
        GAMMA,
        "Text for Usage of gamma.\n",
        &format!("Text for Usage of gamma.\n{fenced}"),
    ));
    assert!(
        block(&repo[GAMMA], "verification-matrix")
            .contains("| `demo::test_crate::it::case` | test | [Contract](#contract) |\n")
    );
}

#[test]
fn a_relationship_table_lists_each_relation_sorted_by_slug_with_links() {
    let repo = base();
    assert_eq!(
        block(&repo[ALPHA], "relationships"),
        "\
| Relation | Capability |
| --- | --- |
| Depends on | [beta](./beta.md) |
| Depends on | [gamma](./gamma.md) |
| Changed by | [grow](../../work/changes/grow.md) |
"
    );
    assert_eq!(
        block(&repo[BETA], "relationships"),
        "\
| Relation | Capability |
| --- | --- |
| Depends on | [gamma](./gamma.md) |
| Depended on by | [alpha](./alpha.md) |
| Changed by | [grow](../../work/changes/grow.md) |
"
    );
    assert_eq!(
        block(&repo[GAMMA], "relationships"),
        "\
| Relation | Capability |
| --- | --- |
| Depended on by | [alpha](./alpha.md) |
| Depended on by | [beta](./beta.md) |
"
    );
}

#[test]
fn a_capability_with_no_relation_has_a_table_of_the_header_only() {
    let repo = synced(&replaced(
        &unsynced(),
        DELTA,
        "Text for Verification of delta.\n",
        &format!("Text for Verification of delta.\n\n{RELATIONSHIPS}"),
    ));
    // `delta` is only proposed by `grow`, which lists it: one row.
    assert_eq!(
        block(&repo[DELTA], "relationships"),
        "\
| Relation | Capability |
| --- | --- |
| Changed by | [grow](../../work/changes/grow.md) |
"
    );
    // A change that no longer lists it leaves nothing but the header.
    let mut bare = unsynced();
    bare.remove(GROW);
    bare.remove(BETA);
    bare.remove(ALPHA);
    let bare = replaced(
        &bare,
        DELTA,
        "Text for Verification of delta.\n",
        &format!("Text for Verification of delta.\n\n{RELATIONSHIPS}"),
    );
    let bare = replaced(&bare, DELTA, "state: considered", "state: delivered");
    let bare = replaced(
        &bare,
        DELTA,
        "  code: []\n  tests: []",
        "  code: [demo::real]\n  tests: [demo::test_crate::it::case]",
    );
    let bare = replaced(
        &bare,
        DELTA,
        "\n> [!PROPOSED change:grow]\n> The candidate contract.\n",
        "",
    );
    let bare = synced(&bare);
    assert_eq!(
        block(&bare[DELTA], "relationships"),
        "| Relation | Capability |\n| --- | --- |\n"
    );
}

#[test]
fn the_catalogue_is_a_table_per_state_in_order_with_gaps_for_implementing_only() {
    let repo = base();
    assert_eq!(
        repo[INDEX],
        format!(
            "{}\n\
<!-- raf:generated:catalogue -->
## Delivered

| Capability | Description | Gaps |
| --- | --- | --- |
| [gamma](./gamma.md) | Delivered and bound. |  |

## Implementing

| Capability | Description | Gaps |
| --- | --- | --- |
| [beta](./beta.md) | Being built \\| with \"care\". | bindings.tests is empty; bindings.examples is empty |

## Designed

| Capability | Description | Gaps |
| --- | --- | --- |
| [alpha](./alpha.md) | Planned work. |  |

## Considered

| Capability | Description | Gaps |
| --- | --- | --- |
| [delta](./delta.md) | Only considered. |  |
<!-- raf:generated:end -->
",
            index_header()
        )
    );
}

#[test]
fn a_gap_names_an_unresolved_binding_and_an_empty_list() {
    let repo = synced(&replaced(
        &unsynced(),
        BETA,
        "code: [demo::bare]",
        "code: [demo::nope]",
    ));
    assert!(
        block(&repo[INDEX], "catalogue").contains(
            "| bindings.code `demo::nope` does not resolve; bindings.tests is empty; bindings.examples is empty |\n"
        ),
        "{}",
        repo[INDEX]
    );
}

#[test]
fn groups_with_no_capability_are_omitted_and_each_group_is_sorted_by_slug() {
    let delivered = |slug: &str| {
        document(
            slug,
            "Bound.",
            "delivered",
            ["[demo::real]", "[demo::test_crate::it::case]", "[]"],
            "[]",
            &[],
        )
    };
    let mut repo = Repo::new();
    for slug in ["zeta", "a-b", "a", "mid"] {
        repo.insert(format!("docs/capabilities/{slug}.md"), delivered(slug));
    }
    let repo = synced(&repo);
    let table = block(&repo[INDEX], "catalogue");
    let rows: Vec<&str> = table
        .lines()
        .filter(|line| line.starts_with("| ["))
        .map(|line| line.split(" | ").next().unwrap_or(line))
        .collect();
    // `a` before `a-b`, though `a-b.md` sorts before `a.md` as a file name.
    assert_eq!(
        rows,
        [
            "| [a](./a.md)",
            "| [a-b](./a-b.md)",
            "| [mid](./mid.md)",
            "| [zeta](./zeta.md)"
        ]
    );
    let headings: Vec<&str> = table.lines().filter(|l| l.starts_with("## ")).collect();
    assert_eq!(headings, ["## Delivered"]);
}

#[test]
fn cell_text_is_escaped_the_same_way_every_time() {
    let repo = replaced(
        &unsynced(),
        DELTA,
        "description: Only considered.",
        "description: \"a|b <i> [x](y) back\\\\slash\\nsecond\\r\\nthird\"",
    );
    let repo = synced(&repo);
    assert!(
        block(&repo[INDEX], "catalogue").contains(
            "| [delta](./delta.md) | a\\|b &lt;i> \\[x\\](y) back\\\\slash second third |  |\n"
        ),
        "{}",
        repo[INDEX]
    );
    assert!(generated_findings(&repo).is_empty(), "unexpected");
}

// --- placement ---------------------------------------------------------------

#[test]
fn a_matrix_and_a_relationship_table_sit_in_a_section_of_a_capability_document() {
    assert!(generated_findings(&base()).is_empty(), "unexpected");
    for block_text in [MATRIX, RELATIONSHIPS] {
        // Outside every section, above the first second-level heading.
        let repo = replaced(
            &base(),
            ALPHA,
            "# alpha\n\n",
            &format!("# alpha\n\n{block_text}\n"),
        );
        assert_eq!(
            pairs(&repo),
            [("generated-misplaced", ALPHA.to_string())],
            "{block_text}"
        );
    }
}

#[test]
fn a_catalogue_block_is_valid_only_in_the_capability_index() {
    let catalogue = "<!-- raf:generated:catalogue -->\n<!-- raf:generated:end -->\n";
    let mut cases: Vec<(&str, Repo)> = Vec::new();
    cases.push((
        ALPHA,
        replaced(
            &base(),
            ALPHA,
            "Text for Verification of alpha.\n",
            &format!("Text for Verification of alpha.\n\n{catalogue}"),
        ),
    ));
    cases.push((
        OVERVIEW,
        replaced(
            &base(),
            OVERVIEW,
            "Body text.\n",
            &format!("Body text.\n\n{catalogue}"),
        ),
    ));
    cases.push((
        GROW,
        replaced(&base(), GROW, "Grows.\n", &format!("Grows.\n\n{catalogue}")),
    ));
    for (file, repo) in cases {
        assert!(
            pairs(&repo).contains(&("generated-misplaced", file.to_string())),
            "{file}: {:#?}",
            check_all(&repo)
        );
    }
}

#[test]
fn a_matrix_or_a_relationship_table_in_the_index_is_misplaced() {
    for block_text in [MATRIX, RELATIONSHIPS] {
        let mut repo = base();
        let text = repo.get_mut(INDEX).expect("index");
        text.push('\n');
        text.push_str(block_text);
        assert_eq!(
            pairs(&repo),
            [("generated-misplaced", INDEX.to_string())],
            "{block_text}"
        );
    }
}

#[test]
fn blocks_in_a_change_record_or_an_architecture_page_are_misplaced_and_never_rewritten() {
    for (file, anchor, text) in [
        (OVERVIEW, "Body text.\n", MATRIX),
        (GROW, "Grows.\n", RELATIONSHIPS),
    ] {
        let repo = replaced(&base(), file, anchor, &format!("{anchor}\n{text}"));
        assert!(pairs(&repo).contains(&("generated-misplaced", file.to_string())));
        match sync(&repo) {
            Err(SyncError::Refused(findings)) => {
                assert!(
                    findings
                        .iter()
                        .any(|f| f.kind == "generated-misplaced" && f.file == file)
                );
            }
            other => panic!("sync must refuse: {other:?}"),
        }
    }
}

#[test]
fn a_generated_block_does_not_make_a_section_non_empty() {
    // A delivered capability whose Verification holds nothing but a block.
    let only_block = replaced(&base(), GAMMA, "Text for Verification of gamma.\n\n", "");
    assert!(
        check_all(&only_block)
            .iter()
            .any(|f| f.kind == "okf-section-empty" && f.file == GAMMA),
        "{:#?}",
        check_all(&only_block)
    );
    // The same section with prose beside the block is not empty.
    assert!(
        !check_all(&base())
            .iter()
            .any(|f| f.kind == "okf-section-empty")
    );
}

// --- marker errors -------------------------------------------------------------

#[test]
fn a_marker_fixture_for_each_error_nested_unclosed_orphan_end_and_unknown_kind() {
    let anchor = "Text for Purpose of beta.\n";
    let cases: [(&str, &str, &str); 5] = [
        (
            "nested blocks",
            "<!-- raf:generated:relationships -->\n<!-- raf:generated:relationships -->\n<!-- raf:generated:end -->\n",
            "okf-parse-nested-generated",
        ),
        (
            "an unclosed block",
            "<!-- raf:generated:relationships -->\n",
            "okf-parse-unclosed-generated",
        ),
        (
            "an end marker without a start",
            "<!-- raf:generated:end -->\n",
            "okf-parse-orphan-generated-end",
        ),
        (
            "a kind that is not kebab case",
            "<!-- raf:generated:Bad_Kind -->\n<!-- raf:generated:end -->\n",
            "okf-parse-malformed-generated",
        ),
        (
            "an unknown kind",
            "<!-- raf:generated:nope -->\n<!-- raf:generated:end -->\n",
            "generated-unknown-kind",
        ),
    ];
    for (name, text, kind) in cases {
        let repo = if kind == "okf-parse-unclosed-generated" {
            // At the end, so no later block is taken for a nested one.
            let mut repo = base();
            let beta = repo.get_mut(BETA).expect("beta");
            beta.push('\n');
            beta.push_str(text);
            repo
        } else {
            replaced(&base(), BETA, anchor, &format!("{anchor}\n{text}"))
        };
        let found = check_all(&repo);
        let hit = found.iter().find(|f| f.kind == kind && f.file == BETA);
        assert!(
            hit.is_some(),
            "{name}: expected `{kind}` in {BETA}, got {found:#?}"
        );
        // A document that does not read is not synced around: the tables that
        // list it would be wrong. An unknown kind is refused by name.
        match sync(&repo) {
            Err(SyncError::Unloaded(ids)) => {
                assert_eq!(ids, ["cap:beta"], "{name}");
                assert_ne!(kind, "generated-unknown-kind");
            }
            Err(SyncError::Refused(findings)) => {
                assert_eq!(kind, "generated-unknown-kind", "{name}");
                assert!(findings.iter().any(|f| f.kind == kind && f.file == BETA));
            }
            other => panic!("{name}: {other:?}"),
        }
    }
}

// --- staleness ------------------------------------------------------------------

/// The findings of `repo` that say `kind` is stale in `file`.
fn stale(repo: &Repo, file: &str, kind: &str) -> Vec<Finding> {
    check_all(repo)
        .into_iter()
        .filter(|f| {
            f.kind == "generated-stale"
                && f.file == file
                && f.message.contains(&format!("`{kind}`"))
        })
        .collect()
}

/// Require that `broken` is stale for `kind` in `file`, that nothing else is
/// stale, and that one sync makes `fixed` out of it, with nothing left.
fn assert_stale_and_repaired(broken: &Repo, file: &str, kind: &str, fixed: &Repo) {
    let found = stale(broken, file, kind);
    assert_eq!(found.len(), 1, "{file} {kind}: {:#?}", check_all(broken));
    assert!(
        found[0].message.contains(file) && found[0].message.contains(kind),
        "{:?}",
        found[0]
    );
    let all: Vec<(&str, &str)> = generated_findings(broken)
        .iter()
        .map(|f| (f.kind, f.file.as_str()))
        .collect::<Vec<_>>()
        .into_iter()
        .map(|(k, f)| (k, if f == file { file } else { "elsewhere" }))
        .collect();
    assert!(all.iter().all(|(k, _)| *k == "generated-stale"), "{all:?}");
    let repaired = synced(broken);
    assert_eq!(&repaired, fixed, "{file} {kind}");
    assert!(
        check_all(&repaired).is_empty(),
        "{:#?}",
        check_all(&repaired)
    );
    assert!(sync(&repaired).expect("syncs").is_empty(), "unexpected");
}

#[test]
fn an_edited_removed_or_added_matrix_row_is_stale_and_sync_repairs_it() {
    let good = base();
    let edited = replaced(&good, GAMMA, "| test |", "| tests |");
    assert_stale_and_repaired(&edited, GAMMA, "verification-matrix", &good);
    let removed = replaced(
        &good,
        GAMMA,
        "| `demo::real` | doctest | [Verification](#verification) |\n",
        "",
    );
    assert_stale_and_repaired(&removed, GAMMA, "verification-matrix", &good);
    // An added binding: the document now binds one more example.
    let added = replaced(
        &good,
        GAMMA,
        "examples: [demo::example_crate::show, demo::real]",
        "examples: [demo::example_crate::show, demo::real, demo::example_crate::flagged]",
    );
    let found = stale(&added, GAMMA, "verification-matrix");
    assert_eq!(found.len(), 1, "{:#?}", check_all(&added));
    let repaired = synced(&added);
    assert!(
        block(&repaired[GAMMA], "verification-matrix")
            .contains("| `demo::example_crate::flagged` | example target |")
    );
    assert!(check_all(&repaired).is_empty(), "unexpected");
    // Only the matrix block changed.
    assert_eq!(
        repaired[GAMMA].replace(&block(&repaired[GAMMA], "verification-matrix"), ""),
        added[GAMMA].replace(&block(&added[GAMMA], "verification-matrix"), "")
    );
}

#[test]
fn an_edited_removed_or_added_relationship_row_is_stale_and_sync_repairs_it() {
    let good = base();
    let edited = replaced(
        &good,
        GAMMA,
        "| Depended on by | [alpha](./alpha.md) |",
        "| Depended on by | [alphx](./alpha.md) |",
    );
    assert_stale_and_repaired(&edited, GAMMA, "relationships", &good);
    let removed = replaced(&good, GAMMA, "| Depended on by | [beta](./beta.md) |\n", "");
    assert_stale_and_repaired(&removed, GAMMA, "relationships", &good);
    // An added relation: `delta` now depends on `gamma`, so `gamma` gains a row.
    let added = replaced(&good, DELTA, "depends: []", "depends: [cap:gamma]");
    let found = stale(&added, GAMMA, "relationships");
    assert_eq!(found.len(), 1, "{:#?}", check_all(&added));
    let repaired = synced(&added);
    assert!(
        block(&repaired[GAMMA], "relationships")
            .contains("| Depended on by | [delta](./delta.md) |\n")
    );
    assert!(check_all(&repaired).is_empty(), "unexpected");
}

#[test]
fn an_edited_removed_or_added_catalogue_row_is_stale_and_sync_repairs_it() {
    let good = base();
    let edited = replaced(
        &good,
        INDEX,
        "Delivered and bound.",
        "Delivered and edited.",
    );
    assert_stale_and_repaired(&edited, INDEX, "catalogue", &good);
    let removed = replaced(
        &good,
        INDEX,
        "| [delta](./delta.md) | Only considered. |  |\n",
        "",
    );
    assert_stale_and_repaired(&removed, INDEX, "catalogue", &good);
    // An added capability.
    let mut added = good.clone();
    added.insert(
        "docs/capabilities/epsilon.md".to_string(),
        document(
            "epsilon",
            "Newly designed.",
            "designed",
            ["[]", "[]", "[]"],
            "[]",
            &[],
        ),
    );
    added.insert(
        GROW.to_string(),
        good[GROW].replace(
            "[cap:alpha, cap:beta, cap:delta]",
            "[cap:alpha, cap:beta, cap:delta, cap:epsilon]",
        ),
    );
    let found = stale(&added, INDEX, "catalogue");
    assert_eq!(found.len(), 1, "{:#?}", check_all(&added));
    let repaired = synced(&added);
    assert!(
        block(&repaired[INDEX], "catalogue")
            .contains("| [alpha](./alpha.md) | Planned work. |  |\n| [epsilon](./epsilon.md) | Newly designed. |  |\n")
    );
    assert!(
        check_all(&repaired).is_empty(),
        "{:#?}",
        check_all(&repaired)
    );
}

#[test]
fn the_stale_finding_points_at_the_opening_marker_of_the_block() {
    let repo = replaced(&base(), GAMMA, "| test |", "| tests |");
    let line = line_of(&repo[GAMMA], "raf:generated:verification-matrix");
    let found = stale(&repo, GAMMA, "verification-matrix");
    assert_eq!(found[0].location, raf_okf::Location::Line(line));
    assert!(!found[0].fix.is_empty(), "unexpected");
}

// --- text outside a block is not touched --------------------------------------------

/// `text` without the lines of its generated blocks, markers included.
fn outside(text: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_end();
        if trimmed.starts_with("<!-- raf:generated:") && trimmed != "<!-- raf:generated:end -->" {
            inside = true;
        } else if trimmed == "<!-- raf:generated:end -->" {
            inside = false;
        } else if !inside {
            out.push_str(line);
        }
    }
    out
}

#[test]
fn text_outside_a_block_is_never_changed_and_a_file_without_blocks_is_untouched() {
    let marker = "DISTINCT outside text, `code`, tabs\tand  double  spaces, é, <b>&amp;";
    let mut repo = unsynced();
    for path in [ALPHA, BETA, GAMMA] {
        repo = replaced(
            &repo,
            path,
            "## Explanation\n\n",
            &format!("## Explanation\n\n{marker}\n\n"),
        );
    }
    // The file with no block carries the string too.
    repo = replaced(
        &repo,
        DELTA,
        "## Explanation\n\n",
        &format!("## Explanation\n\n{marker}\n\n"),
    );
    let report = sync(&repo).expect("syncs");
    let touched: BTreeSet<&str> = report.edits.iter().map(|(p, _)| p.as_str()).collect();
    assert_eq!(touched, BTreeSet::from([ALPHA, BETA, GAMMA, INDEX]));
    let after = apply(&repo, &report);
    for path in [ALPHA, BETA, GAMMA] {
        assert_eq!(outside(&after[path]), outside(&repo[path]), "{path}");
        assert!(after[path].contains(marker));
        assert_ne!(after[path], repo[path]);
    }
    assert_eq!(
        after[DELTA], repo[DELTA],
        "a file with no block is untouched"
    );
    assert_eq!(after[GROW], repo[GROW]);
    assert_eq!(after[OVERVIEW], repo[OVERVIEW]);
}

#[test]
fn the_frontmatter_of_a_rewritten_document_is_kept_byte_for_byte() {
    // Comments, quoting and key order that the writer would normalise.
    let repo = replaced(
        &unsynced(),
        GAMMA,
        "title: \"Title of gamma\"\n",
        "title: 'Title of gamma'   # kept as written\n",
    );
    let after = synced(&repo);
    let head = |text: &str| text[..text.find("\n# gamma\n").expect("title")].to_string();
    assert_eq!(head(&after[GAMMA]), head(&repo[GAMMA]));
    assert!(after[GAMMA].contains("title: 'Title of gamma'   # kept as written\n"));
}

// --- line endings (the body's own) ----------------------------------------------------

/// `text` with every line break `\r\n`.
fn crlf(text: &str) -> String {
    text.replace('\n', "\r\n")
}

/// `text` with the frontmatter and the body each in the given ending.
fn endings(text: &str, head_crlf: bool, body_crlf: bool) -> String {
    let split = text.find("\n---\n").expect("a closing fence") + "\n---\n".len();
    let part = |part: &str, to_crlf: bool| {
        if to_crlf {
            crlf(part)
        } else {
            part.to_string()
        }
    };
    format!(
        "{}{}",
        part(&text[..split], head_crlf),
        part(&text[split..], body_crlf)
    )
}

fn with_endings(repo: &Repo, head_crlf: bool, body_crlf: bool) -> Repo {
    repo.iter()
        .map(|(path, text)| {
            let text = if path.starts_with("docs/capabilities/") && path != INDEX {
                endings(text, head_crlf, body_crlf)
            } else {
                text.clone()
            };
            (path.clone(), text)
        })
        .collect()
}

#[test]
fn a_crlf_document_stays_all_crlf_and_an_lf_document_all_lf() {
    let good = base();
    // LF in, LF out.
    assert_eq!(synced(&unsynced())[GAMMA], good[GAMMA]);
    // All CRLF in, all CRLF out: the same bytes as the LF result converted.
    let crlf_repo = with_endings(&unsynced(), true, true);
    let after = synced(&crlf_repo);
    for path in [ALPHA, BETA, GAMMA] {
        assert_eq!(after[path], crlf(&good[path]), "{path}");
        assert!(!after[path].replace("\r\n", "").contains('\n'), "{path}");
    }
    assert!(check_all(&after).is_empty(), "{:#?}", check_all(&after));
    assert!(sync(&after).expect("syncs").is_empty(), "unexpected");
}

#[test]
fn a_crlf_frontmatter_over_an_lf_body_gets_lf_blocks_and_stays_clean() {
    let good = base();
    let repo = synced(&with_endings(&unsynced(), true, false));
    for path in [ALPHA, BETA, GAMMA] {
        assert_eq!(repo[path], endings(&good[path], true, false), "{path}");
        let title = format!(
            "\n# {}\n",
            path.trim_start_matches("docs/capabilities/")
                .trim_end_matches(".md")
        );
        let body = &repo[path][repo[path].find(&title).expect("the title")..];
        assert!(!body.contains('\r'), "{path}: the blocks are LF");
    }
    assert!(check_all(&repo).is_empty(), "{:#?}", check_all(&repo));
    assert!(sync(&repo).expect("syncs").is_empty(), "unexpected");
}

#[test]
fn an_lf_frontmatter_over_a_crlf_body_gets_crlf_blocks() {
    let good = base();
    let repo = synced(&with_endings(&unsynced(), false, true));
    for path in [ALPHA, BETA, GAMMA] {
        assert_eq!(repo[path], endings(&good[path], false, true), "{path}");
    }
    assert!(check_all(&repo).is_empty(), "{:#?}", check_all(&repo));
}

#[test]
fn a_body_that_mixes_line_endings_is_refused_by_sync_and_reported_by_check() {
    let mixed = replaced(
        &unsynced(),
        ALPHA,
        "Text for Purpose of alpha.\n",
        "Text for Purpose of alpha.\r\nCRLF line here\r\n",
    );
    let found = check_all(&mixed);
    let hit = found
        .iter()
        .find(|f| f.kind == "mixed-line-endings")
        .unwrap_or_else(|| panic!("{found:#?}"));
    assert_eq!(hit.file, ALPHA);
    // The first line break of the body is `\n`; the first line that differs is
    // the line that was given a `\r\n`.
    assert_eq!(
        hit.location,
        raf_okf::Location::Line(line_of(&mixed[ALPHA], "Text for Purpose of alpha."))
    );
    match sync(&mixed) {
        Err(SyncError::Refused(findings)) => {
            assert!(
                findings
                    .iter()
                    .any(|f| f.kind == "mixed-line-endings" && f.file == ALPHA)
            );
        }
        other => panic!("sync must refuse a mixed body: {other:?}"),
    }
    // A file that holds no block is not judged, and one in a single ending is fine.
    let no_blocks = replaced(
        &unsynced(),
        DELTA,
        "Text for Purpose of delta.\n",
        "Text for Purpose of delta.\r\nmixed\r\n",
    );
    assert!(
        !check_all(&no_blocks)
            .iter()
            .any(|f| f.kind == "mixed-line-endings")
    );
}

#[test]
fn the_sync_of_a_crlf_document_is_idempotent_and_stale_in_crlf_is_repaired_in_crlf() {
    let good = synced(&with_endings(&unsynced(), true, true));
    let broken = replaced(&good, GAMMA, "| test |", "| tests |");
    assert_eq!(stale(&broken, GAMMA, "verification-matrix").len(), 1);
    assert_eq!(synced(&broken), good);
}

// --- the index ------------------------------------------------------------------------

#[test]
fn a_missing_index_is_a_finding_and_sync_creates_it_with_the_spdx_lines() {
    let mut repo = base();
    repo.remove(INDEX);
    let found = check_all(&repo);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].kind, "index-missing");
    assert_eq!(found[0].file, INDEX);
    let report = sync(&repo).expect("syncs");
    assert_eq!(report.edits.len(), 1);
    assert_eq!(report.edits[0].0.as_str(), INDEX);
    let created = &report.edits[0].1;
    assert!(created.starts_with(&index_header()), "{created}");
    assert!(created.starts_with(&format!(
        "{}\n<!-- raf:generated:catalogue -->\n",
        index_header()
    )));
    assert_eq!(created, &base()[INDEX]);
    assert!(check_all(&apply(&repo, &report)).is_empty(), "unexpected");
}

#[test]
fn the_spdx_text_of_a_new_index_is_required_and_must_fit_one_comment_line() {
    let mut repo = base();
    repo.remove(INDEX);
    let refused = |options: SyncOptions| with_plan(&repo, |plan| plan.sync(&options));
    assert!(matches!(
        refused(SyncOptions::default()),
        Err(SyncError::SpdxUnset { field: "copyright" })
    ));
    let mut no_licence = SyncOptions::default();
    no_licence.copyright = Some("2026 Example".to_string());
    assert!(matches!(
        refused(no_licence),
        Err(SyncError::SpdxUnset { field: "licence" })
    ));
    for (copyright, licence, field) in [
        ("", "CC-BY-4.0", "copyright"),
        ("two\nlines", "CC-BY-4.0", "copyright"),
        ("ends --> early", "CC-BY-4.0", "copyright"),
        ("2026 Example", "  ", "licence"),
        ("2026 Example", "MIT\r", "licence"),
    ] {
        match refused(SyncOptions::new(copyright, licence)) {
            Err(SyncError::SpdxInvalid { field: got, .. }) => assert_eq!(got, field),
            other => panic!("{copyright:?} {licence:?}: {other:?}"),
        }
    }
    // The text is checked only when the index has to be created.
    assert!(with_plan(&base(), |plan| plan.sync(&SyncOptions::default())).is_ok());
}

#[test]
fn an_index_with_text_outside_its_blocks_is_rejected_whatever_else_it_holds() {
    for text in [
        "Hand-written navigation.\n",
        "# Capabilities\n\n- [alpha](./alpha.md)\n",
        &format!(
            "{}\nNot generated.\n\n<!-- raf:generated:catalogue -->\n<!-- raf:generated:end -->\n",
            index_header()
        ),
    ] {
        let mut repo = base();
        repo.insert(INDEX.to_string(), text.to_string());
        let found = check_all(&repo);
        let hit = found
            .iter()
            .find(|f| f.kind == "index-hand-written")
            .unwrap_or_else(|| panic!("{text:?}: {found:#?}"));
        assert_eq!(hit.file, INDEX);
        assert!(!hit.fix.is_empty(), "unexpected");
        match sync(&repo) {
            Err(SyncError::Refused(findings)) => {
                assert!(
                    findings.iter().any(|f| f.kind == "index-hand-written"),
                    "{findings:#?}"
                );
            }
            other => panic!("sync must refuse a hand-written index: {other:?}"),
        }
    }
}

#[test]
fn an_index_that_is_only_a_licence_header_and_blank_lines_is_not_a_generated_index() {
    let mut repo = base();
    repo.insert(INDEX.to_string(), format!("{}\n", index_header()));
    let found = check_all(&repo);
    assert!(
        found
            .iter()
            .any(|f| f.kind == "okf-parse-index-empty" && f.file == INDEX),
        "{found:#?}"
    );
    assert!(matches!(sync(&repo), Err(SyncError::Refused(_))));
}

#[test]
fn an_existing_index_keeps_its_own_header_and_a_stale_catalogue_is_rewritten_in_place() {
    let header = format!(
        "<!-- {}2020 Someone Else -->\n<!-- {}MIT -->\n",
        concat!("SPDX-", "FileCopyrightText: "),
        concat!("SPDX-", "License-Identifier: ")
    );
    let mut repo = base();
    let old = repo[INDEX]
        .replace(&index_header(), &header)
        .replace("Planned work.", "Old text.");
    repo.insert(INDEX.to_string(), old);
    let after = synced(&repo);
    assert!(after[INDEX].starts_with(&header));
    assert!(after[INDEX].contains("Planned work."));
    assert!(check_all(&after).is_empty(), "unexpected");
}

#[test]
fn a_repository_without_capabilities_needs_no_index() {
    let mut repo = Repo::new();
    repo.insert(
        OVERVIEW.to_string(),
        support::fixtures::valid()[OVERVIEW].clone(),
    );
    assert!(check_all(&repo).is_empty(), "unexpected");
    assert!(
        with_plan(&repo, |plan| plan.sync(&SyncOptions::default()))
            .expect("syncs")
            .is_empty()
    );
}

#[test]
fn the_catalogue_cannot_be_forged_from_a_description_that_looks_like_a_marker() {
    let repo = replaced(
        &unsynced(),
        DELTA,
        "description: Only considered.",
        "description: \"<!-- raf:generated:end -->\"",
    );
    let after = synced(&repo);
    assert!(check_all(&after).is_empty(), "{:#?}", check_all(&after));
    assert!(block(&after[INDEX], "catalogue").contains("&lt;!-- raf:generated:end -->"));
}

#[test]
fn the_kinds_the_crate_owns_are_the_three_documented_ones() {
    assert_eq!(
        Workplan::GENERATED_KINDS,
        ["verification-matrix", "relationships", "catalogue"]
    );
    let _ = options();
}

// --- blocks do not count as prose, rows are sound ------------------------------------

#[test]
fn a_block_in_a_section_other_than_verification_converges_in_one_sync() {
    // A matrix and a relationship table in Usage: the rows the matrix writes
    // name `demo::real`, and must not make Usage a section that names it.
    let repo = replaced(
        &unsynced(),
        GAMMA,
        "Run `demo::example_crate::show` to see it.\n",
        &format!("Run `demo::example_crate::show` to see it.\n\n{MATRIX}\n{RELATIONSHIPS}"),
    );
    let once = synced(&repo);
    assert!(
        sync(&once).expect("syncs").is_empty(),
        "converges in one sync"
    );
    assert!(check_all(&once).is_empty(), "{:#?}", check_all(&once));
    let usage_matrix = block(&once[GAMMA], "verification-matrix");
    assert!(
        usage_matrix.contains("| `demo::real` | doctest | [Verification](#verification) |\n"),
        "{usage_matrix}"
    );
    // Every matrix of the document is the same table.
    let all: Vec<&str> = once[GAMMA].matches("| `demo::real` |").collect();
    assert_eq!(all.len(), 2);
}

#[test]
fn an_example_that_does_not_resolve_is_listed_as_an_example_target() {
    let repo = replaced(
        &unsynced(),
        GAMMA,
        "examples: [demo::example_crate::show, demo::real]",
        "examples: [demo::example_crate::show, demo::real, demo::example_crate::absent]",
    );
    let repo = synced(&repo);
    assert!(block(&repo[GAMMA], "verification-matrix").contains(
        "| `demo::example_crate::absent` | example target | [Verification](#verification) |\n"
    ));
    assert!(
        check_all(&repo)
            .iter()
            .any(|f| f.kind == "binding-unresolved")
    );
}

#[test]
fn a_relation_listed_twice_is_one_row() {
    // `depends` names `gamma` twice and one change lists `alpha` twice.
    let mut repo = replaced(
        &unsynced(),
        ALPHA,
        "depends: [cap:gamma, cap:beta]",
        "depends: [cap:gamma, cap:gamma, cap:beta]",
    );
    repo = replaced(
        &repo,
        GROW,
        "changes: [cap:alpha, cap:beta, cap:delta]",
        "changes: [cap:alpha, cap:alpha, cap:beta, cap:delta]",
    );
    let repo = synced(&repo);
    let table = block(&repo[ALPHA], "relationships");
    assert_eq!(table.matches("[gamma](./gamma.md)").count(), 1, "{table}");
    assert_eq!(table.matches("[grow]").count(), 1, "{table}");
}

#[test]
fn a_backtick_in_a_binding_path_does_not_break_the_code_span() {
    let repo = replaced(
        &unsynced(),
        GAMMA,
        "examples: [demo::example_crate::show, demo::real]",
        "examples: [\"demo::a`b\", \"`edge\"]",
    );
    let repo = synced(&repo);
    let table = block(&repo[GAMMA], "verification-matrix");
    assert!(
        table.contains("| ``demo::a`b`` | example target |"),
        "{table}"
    );
    assert!(
        table.contains("| `` `edge `` | example target |"),
        "{table}"
    );
    assert!(sync(&repo).expect("syncs").is_empty());
}
