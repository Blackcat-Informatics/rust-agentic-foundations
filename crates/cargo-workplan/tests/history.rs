// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `history` through the built binary and the real `git`: the document and
//! bound-symbol commits of a capability, newest first, in a table and in RDF 1.2
//! with the same entries; a binding that no longer resolves; `--since`; `-q`;
//! the exit 2 of an id no document has and of a shallow clone; and that the
//! tree is left as it was.

mod support;
use support::rdf::Rdf;

use support::history::{ALPHA, commit_on, dated};
use support::{Repo, Scratch, git};

fn commit_of(repo: &Repo, subject: &str) -> String {
    repo.git(&[
        "log",
        "--format=%H",
        "--fixed-strings",
        "--grep",
        subject,
        "-n",
        "1",
    ])
}

#[test]
fn a_capability_lists_its_document_and_symbol_commits_newest_first_and_a_removed_binding() {
    let repo = dated("history-table");
    let before = repo.worktree();
    let run = repo.both(&["history", "cap:alpha"]).expect(0);
    assert_eq!(run.stderr, "");
    let short = |subject: &str| commit_of(&repo, subject)[..7].to_string();
    let removed = commit_of(&repo, "remove gone");
    let parent = repo.git(&["rev-parse", &format!("{removed}^")]);
    assert_eq!(
        run.stdout,
        format!(
            "| Commit | Date | Subject | Source |\n| --- | --- | --- | --- |\n\
             | {} | 2030-01-03 | change real | `demo::real` |\n\
             | {} | 2030-01-02 | edit alpha | document |\n\
             | {} | 2030-01-01 | base | document, `demo::real` |\n\
             \n| Binding | Removed in | Date | Subject | Last resolved |\n| --- | --- | --- | --- | --- |\n\
             | `demo::gone` | {} | 2030-01-04 | remove gone | {} |\n",
            short("change real"),
            short("edit alpha"),
            short("base"),
            &removed[..7],
            &parent[..7],
        )
    );
    // The history reads Git and writes nothing.
    assert_eq!(repo.worktree(), before);
}

#[test]
fn rdf_preserves_full_history_commits_sources_paths_and_removed_bindings() {
    let repo = dated("history-rdf");
    let text = repo.both(&["history", "cap:alpha"]).expect(0);
    for format in ["turtle", "ntriples"] {
        let run = repo
            .both(&["--format", format, "history", "cap:alpha"])
            .expect(0);
        let rdf = Rdf::of(&run);
        let entries = rdf.rows("HistoryEntry");
        let rows: Vec<&str> = text
            .stdout
            .lines()
            .skip(2)
            .take_while(|line| !line.is_empty())
            .collect();
        assert_eq!(entries.len(), rows.len());
        for (ordinal, (entry, row)) in entries.iter().zip(&rows).enumerate() {
            assert_eq!(entry.number("ordinal"), ordinal);
            let commit = entry.required("commit");
            assert_eq!(commit.len(), 40);
            assert_eq!(
                entry.required("date"),
                repo.git(&["show", "-s", "--format=%aI", commit])
            );
            for property in ["source", "path"] {
                for (ordinal, child) in entry.children(property).iter().enumerate() {
                    assert_eq!(child.number("ordinal"), ordinal);
                }
            }
            assert!(row.starts_with(&format!("| {} | ", &commit[..7])));
            assert!(row.contains(&format!("| {} |", entry.required("subject"))));
        }
        let source = entries[0].children("source");
        assert_eq!(source.len(), 1);
        assert_eq!(source[0].required("kind"), "symbol");
        assert_eq!(source[0].required("binding"), "demo::real");
        assert_eq!(
            entries[0].children("path")[0].required("path"),
            "src/lib.rs"
        );
        assert_eq!(entries[1].children("path")[0].required("path"), ALPHA);
        let removed = rdf.rows("RemovedBinding");
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].required("binding"), "demo::gone");
        assert_eq!(removed[0].required("removal"), "found");
        assert_eq!(
            removed[0].required("commit"),
            commit_of(&repo, "remove gone").as_str()
        );
        assert_eq!(removed[0].required("identifier"), "gone");
        assert!(!removed[0].children("directory").is_empty());
        let removed_commit = commit_of(&repo, "remove gone");
        assert_eq!(
            removed[0].required("lastResolved"),
            repo.git(&["rev-parse", &format!("{removed_commit}^")])
        );
    }
}

#[test]
fn since_limits_the_commits_and_a_change_or_an_unknown_id_is_answered_or_refused() {
    let repo = dated("history-since");
    let run = repo
        .both(&["history", "cap:alpha", "--since", "2030-01-03"])
        .expect(0);
    let rows: Vec<&str> = run
        .stdout
        .lines()
        .skip(2)
        .take_while(|l| !l.is_empty())
        .collect();
    assert_eq!(rows.len(), 1, "{run:#?}");
    assert!(rows[0].contains("| change real |"), "{run:#?}");
    // A change record has its own document history only.
    let change = repo.both(&["history", "change:grow"]).expect(0);
    assert_eq!(change.lines().len(), 3, "{change:#?}");
    assert!(change.stdout.contains("| base | document |"), "{change:#?}");
    // An id no document has is exit 2, naming the ids of its kind.
    let unknown = repo.both(&["history", "cap:nowhere"]).expect(2);
    assert_eq!(unknown.stdout, "");
    assert!(unknown.stderr.contains("cap:alpha"), "{unknown:#?}");
    let json = repo
        .both(&["--format", "turtle", "history", "cap:nowhere"])
        .expect(2);
    assert_eq!(json.stdout, "");
}

#[test]
fn a_bound_file_that_is_not_committed_is_exit_two_naming_it_until_it_is() {
    let repo = dated("history-uncommitted");
    repo.put(
        "src/later.rs",
        "/// Written after the last commit.\npub fn later() {}\n",
    );
    repo.edit(
        "src/lib.rs",
        "pub fn real() {",
        "pub mod later;\n\n/// A documented function.\npub fn real() {",
    );
    repo.edit(
        ALPHA,
        "[demo::real, demo::gone]",
        "[demo::real, demo::gone, demo::later::later]",
    );
    let run = repo.both(&["history", "cap:alpha"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr.contains(
            "`git log -L` could not read the lines of the bound symbol `demo::later::later` in `src/later.rs` from `HEAD`"
        ) && run.stderr.contains("one way to read them is to commit `src/later.rs`"),
        "{run:#?}"
    );
    // The neighbour: once the file is committed, it has a history.
    commit_on(&repo, "add later", "2030-01-06");
    let run = repo.both(&["history", "cap:alpha"]).expect(0);
    assert!(
        run.stdout
            .contains("| add later | document, `demo::later::later`, `demo::real` |"),
        "{run:#?}"
    );
}

#[test]
fn quiet_prints_nothing_because_a_history_is_not_a_finding() {
    let repo = dated("history-quiet");
    let run = repo.both(&["-q", "history", "cap:alpha"]).expect(0);
    assert_eq!(run.stdout, "");
    let json = repo
        .both(&["-q", "--format", "turtle", "history", "cap:alpha"])
        .expect(0);
    assert_eq!(Rdf::of(&json).result_count(), 0);
}

#[test]
fn a_shallow_clone_is_exit_two_and_its_full_clone_answers() {
    let repo = dated("history-shallow");
    let copies = Scratch::new("history-shallow-copies");
    let source = format!("file://{}", repo.root().display());
    git(
        copies.path(),
        &["clone", "-q", "--depth", "1", &source, "shallow"],
    );
    git(copies.path(), &["clone", "-q", &source, "full"]);
    let shallow = copies.path().join("shallow").to_string_lossy().into_owned();
    let run = support::run_both(
        &support::elsewhere(),
        &["--root", &shallow, "history", "cap:alpha"],
    )
    .expect(2);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.contains("shallow clone"), "{run:#?}");
    // The neighbour: the full clone answers.
    let full = copies.path().join("full").to_string_lossy().into_owned();
    support::run_both(
        &support::elsewhere(),
        &["--root", &full, "history", "cap:alpha"],
    )
    .expect(0);
}

#[test]
fn stored_reviews_report_item_changes_after_the_reviewed_commit_is_gone() {
    let repo = Repo::bare("review-currency");
    repo.put(
        "docs/capabilities/scope.md",
        &support::capability("scope", "considered", ["[]", "[]", "[]"], "[]", &[]),
    );
    repo.sync();
    repo.put("src/lib.rs", "pub fn stable() {}\npub fn changed() { let _ = 1; }\npub fn removed() {}\npub fn renamed_before() {}\npub mod scoped { pub fn child() {} }\n");
    repo.checkout();
    let revision = repo.git(&["rev-parse", "HEAD"]);
    repo.run(&[
        "review",
        "record",
        "--id",
        "audit",
        "--scope",
        "crate:demo",
        "--revision",
        &revision,
        "--date",
        "2030-01-01T00:00:00Z",
        "--kind",
        "correctness",
        "--verdict",
        "clean",
        "--by",
        "human",
        "--copyright",
        support::COPYRIGHT,
        "--licence",
        support::LICENCE,
    ])
    .expect(0);
    let record = repo.read("work/reviews/audit.md");
    assert!(record.contains(&revision));
    assert!(
        repo.run(&["review", "list"])
            .expect(0)
            .stdout
            .contains("review:audit")
    );
    assert!(
        repo.run(&["review", "show", "review:audit"])
            .expect(0)
            .stdout
            .contains("kind: correctness")
    );
    assert!(
        repo.both(&["review", "status", "crate:demo"])
            .expect(0)
            .stdout
            .contains("current")
    );
    assert!(
        repo.run(&["review", "unreviewed", "--crate", "demo"])
            .expect(0)
            .stdout
            .contains("unreviewed: 0")
    );
    let coverage = repo
        .run(&["--format", "turtle", "review", "coverage"])
        .expect(0);
    let rdf = Rdf::of(&coverage);
    let row = rdf.rows("ReviewCoverage")[0];
    assert_eq!(row.required("reviewed"), row.required("total"));
    assert_eq!(row.required("percentHundredths"), "10000");
    assert_review_carriers(&repo);
    assert_module_review_scope(&repo);
    assert_public_item_scope_order(&repo);
    discard_reviewed_revision(&repo, &revision);
    assert!(
        repo.run(&["review", "status", "crate:demo"])
            .expect(0)
            .stdout
            .contains("current")
    );
    assert_eq!(repo.read("work/reviews/audit.md"), record);
    repo.put("src/lib.rs", "pub fn stable() {}\npub fn changed() { let _ = 2; }\npub fn renamed_after() {}\npub fn added() {}\npub mod scoped { pub fn child() {} }\n");
    assert_review_changes(&repo, &revision);
    assert_review_refusal_neighbors(&repo);
}

fn assert_review_carriers(repo: &Repo) {
    let before = repo.worktree();
    let mut expected = None;
    for (format, media) in review_carriers() {
        let output = repo
            .run(&["--format", format, "review", "show", "audit"])
            .expect(0);
        if format == "turtle" {
            let alias = repo
                .run(&["--format", format, "review", "show", "review:audit"])
                .expect(0);
            assert_eq!(output.stdout, alias.stdout);
        }
        let canonical = review_carrier_rows(&output.stdout, media);
        assert!(canonical.contains("reviewVerdict"));
        assert!(canonical.contains("<<("));
        if let Some(expected) = &expected {
            assert_eq!(&canonical, expected, "{format}");
        } else {
            expected = Some(canonical);
        }
    }
    for command in [
        &["review", "status", "crate:demo"][..],
        &["review", "coverage"][..],
        &["review", "unreviewed", "--crate", "demo"][..],
    ] {
        let mut expected = None;
        for (format, media) in review_carriers() {
            let mut args = vec!["--format", format];
            args.extend_from_slice(command);
            let output = repo.run(&args).expect(0);
            let canonical = review_carrier_rows(&output.stdout, media);
            assert!(!canonical.contains("urn:raf:report:construction"));
            if let Some(expected) = &expected {
                assert_eq!(&canonical, expected, "{args:?}");
            } else {
                expected = Some(canonical);
            }
        }
    }
    assert_eq!(repo.worktree(), before);
}

fn review_carriers() -> [(&'static str, &'static str); 3] {
    [
        ("turtle", "text/turtle"),
        ("ntriples", "application/n-triples"),
        ("json", "application/ld+json"),
    ]
}

fn review_carrier_rows(text: &str, media: &str) -> String {
    let dataset = purrdf_rdf::parse_dataset(text.as_bytes(), media, None)
        .expect("native review carrier parses");
    let rendered = purrdf_rdf::serialize_dataset_with(
        &dataset,
        purrdf_rdf::NativeRdfFormat::NTriples,
        None,
        &purrdf_rdf::SerializeOptions {
            selection: purrdf_rdf::SerializeGraph::Dataset,
            statement_layer: purrdf_rdf::StatementLayer::Emit,
            jsonld_options: None,
        },
    )
    .expect("complete statement layer serializes");
    assert_eq!(rendered.statement_rows_dropped, 0);
    let text = String::from_utf8(rendered.bytes).expect("native RDF is UTF-8");
    let mut rows: Vec<_> = text.lines().collect();
    purrdf_lex::walk::try_sort_unstable_by(&mut rows, |left, right| {
        Ok::<_, std::convert::Infallible>(left.cmp(right))
    })
    .unwrap_or_else(|never| match never {});
    rows.join("\n")
}

fn assert_module_review_scope(repo: &Repo) {
    repo.run(&[
        "review",
        "record",
        "--id",
        "module",
        "--scope",
        "module:demo::scoped",
        "--date",
        "2030-01-01T00:00:00Z",
        "--kind",
        "design",
        "--verdict",
        "clean",
        "--by",
        "human",
        "--copyright",
        support::COPYRIGHT,
        "--licence",
        support::LICENCE,
    ])
    .expect(0);
    let opened = raf_workplan::Gate::workspace(repo.root())
        .programs(raf_workplan::exec::Programs::new(repo.root()).discover_git())
        .open()
        .expect("the recorded module scope loads");
    let plan = opened.workplan();
    let inputs = &plan
        .review_show("module")
        .expect("module review")
        .value()
        .inputs;
    assert_eq!(
        inputs
            .iter()
            .map(|input| input.name.as_str())
            .collect::<Vec<_>>(),
        ["demo::scoped", "demo::scoped::child"]
    );
}

fn assert_public_item_scope_order(repo: &Repo) {
    use raf_okf::{DocType, RecordHeader, ReviewKind, ReviewScope};
    let scope = ReviewScope::Items(vec!["demo::stable".into(), "demo::changed".into()]);
    let duplicate = ReviewScope::Items(vec!["demo::stable".into(), "demo::stable".into()]);
    let opened = raf_workplan::Gate::workspace(repo.root())
        .open()
        .expect("current source opens");
    let plan = opened.workplan();
    let mut record = plan.review_show("audit").unwrap().value().clone();
    record.header = RecordHeader::new(DocType::Review, "items");
    record.header.timestamp = Some("2030-01-01T00:00:00Z".into());
    record.scope = scope.clone();
    record.inputs = plan
        .review_inputs(&scope)
        .expect("unordered distinct item set");
    assert!(plan.review_inputs(&duplicate).is_err());
    assert!(
        plan.review_status(&duplicate, ReviewKind::Correctness)
            .is_err()
    );
    repo.put(
        "work/reviews/items.md",
        &record.to_document().expect("valid item review").render(),
    );
    let reopened = raf_workplan::Gate::workspace(repo.root())
        .open()
        .expect("item review opens");
    let status = reopened
        .workplan()
        .review_status(&scope, ReviewKind::Correctness)
        .expect("original public item set remains queryable");
    assert_eq!(status.review.as_deref(), Some("review:items"));
    assert!(status.is_current());
}

fn discard_reviewed_revision(repo: &Repo, revision: &str) {
    repo.git(&["checkout", "-q", "--orphan", "squashed"]);
    repo.commit("squashed snapshot");
    repo.git(&["branch", "-D", "main", "feature"]);
    repo.git(&["symbolic-ref", "--delete", "refs/remotes/origin/HEAD"]);
    repo.git(&["update-ref", "-d", "refs/remotes/origin/main"]);
    repo.git(&["reflog", "expire", "--expire=all", "--all"]);
    repo.git(&["gc", "--prune=now"]);
    assert!(
        repo.run(&["history", "review:audit"])
            .expect(0)
            .stdout
            .contains("squashed snapshot")
    );
    let objects = repo.git(&[
        "cat-file",
        "--batch-all-objects",
        "--batch-check=%(objectname)",
    ]);
    assert!(
        !objects.lines().any(|object| object == revision),
        "reviewed object survived pruning"
    );
}

fn assert_review_changes(repo: &Repo, revision: &str) {
    let before = repo.worktree();
    let run = repo
        .run(&[
            "--format",
            "turtle",
            "review",
            "changed-since",
            "crate:demo",
        ])
        .expect(1);
    let rdf = Rdf::of(&run);
    let row = rdf.rows("ReviewStatus")[0];
    assert_eq!(row.required("state"), "stale");
    assert!(row.required("diffCommand").contains(revision));
    let changes = row.children("change");
    let changes: Vec<_> = changes
        .iter()
        .map(|row| (row.required("item"), row.required("kind")))
        .collect();
    assert_eq!(
        changes,
        [
            ("demo::added", "added"),
            ("demo::changed", "changed"),
            ("demo::removed", "removed"),
            ("demo::renamed_after", "added"),
            ("demo::renamed_before", "removed"),
        ]
    );
    let missing = repo
        .run(&["review", "unreviewed", "--crate", "demo"])
        .expect(1);
    assert_eq!(
        missing.stdout.lines().collect::<Vec<_>>(),
        ["demo::added", "demo::changed", "demo::renamed_after"]
    );
    assert!(
        repo.run(&["review", "status", "crate:demo", "--kind", "security"])
            .expect(1)
            .stdout
            .contains("never-reviewed")
    );
    assert_eq!(repo.worktree(), before);
}

fn assert_review_refusal_neighbors(repo: &Repo) {
    let before = repo.worktree();
    for args in [
        &["review", "show", "absent"][..],
        &["review", "show", "evidence:audit"],
        &["review", "status", "crate:absent"],
        &["review", "status", "crate:demo", "--kind", "unknown"],
        &["--format", "json", "--root", "missing", "check"],
    ] {
        let refused = repo.run(args).expect(2);
        assert_eq!(refused.stdout, "");
    }
    assert_eq!(repo.worktree(), before);
    let mut args = vec![
        "review",
        "record",
        "--id",
        "with-findings",
        "--scope",
        "crate:demo",
        "--date",
        "2030-01-02T00:00:00Z",
        "--kind",
        "design",
        "--verdict",
        "findings",
        "--by",
        "human",
        "--copyright",
        support::COPYRIGHT,
        "--licence",
        support::LICENCE,
        "--findings",
        "findings.md",
    ];
    let refused = repo.run(&args).expect(2);
    assert!(refused.stderr.contains("findings.md"), "{refused:#?}");
    assert_eq!(repo.worktree(), before);
    repo.put("findings.md", "<!-- SPDX-FileCopyrightText: 2026 Example -->\n<!-- SPDX-License-Identifier: CC-BY-4.0 -->\n\n# Findings\n\nThe review records an open design finding.\n");
    repo.run(&args).expect(1);
    assert!(
        repo.read("work/reviews/with-findings.md")
            .contains("findings_ref: findings.md")
    );
    let written = repo.worktree();
    args[3] = "invalid-date";
    args[7] = "invalid-date";
    let invalid = repo.run(&args).expect(2);
    assert!(invalid.stderr.contains("date"));
    assert_eq!(repo.worktree(), written);
}
