// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Canonical RDF through the built binary retains structured domain results.

mod support;
use raf_okf::Location;
use raf_workplan::Gate;
use support::{COPYRIGHT, LICENCE, Repo, rdf::Rdf};

fn findings_repo(tag: &str) -> Repo {
    let repo = Repo::synced(tag, true);
    let path = "docs/capabilities/alpha.md";
    let text = repo.read(path);
    let (before, rest) = text.split_once("## Usage\n").unwrap();
    let (_, after) = rest.split_once("## Examples\n").unwrap();
    repo.put(path, &format!("{before}## Usage\n\n## Examples\n{after}"));
    repo.put("docs/architecture/notes.md", "# Notes\n");
    repo.edit(
        "docs/capabilities/gamma.md",
        "description: A description of gamma.",
        "description: A description of gamma, reworded.",
    );
    repo.edit(
        "docs/capabilities/alpha.md",
        "Text for Contract of alpha.",
        "Text for Contract of alpha, see [it](docs/capabilities/missing.md).",
    );
    std::fs::remove_file(repo.root().join("docs/capabilities/index.md")).unwrap();
    repo
}

#[test]
fn check_preserves_every_finding_location_in_both_equivalent_formats() {
    let repo = findings_repo("rdf-check");
    let findings = Gate::workspace(repo.root())
        .programs(raf_workplan::exec::Programs::new(repo.root()).discover_git())
        .check();
    let turtle = repo.both(&["--format", "turtle", "check"]).expect(1);
    let nt = repo.both(&["--format", "ntriples", "check"]).expect(1);
    let rdf = Rdf::of(&turtle);
    assert_eq!(
        rdf.graph().snapshot().unwrap(),
        Rdf::of(&nt).graph().snapshot().unwrap()
    );
    let rows = rdf.rows("Finding");
    assert_eq!(rows.len(), findings.len());
    let mut seen = [false; 4];
    for (row, finding) in rows.iter().zip(&findings) {
        assert_eq!(row.required("kind"), finding.kind);
        assert_eq!(row.required("file"), finding.file.as_str());
        assert_eq!(row.required("message"), finding.message);
        assert_eq!(row.required("fix"), finding.fix);
        let (section, line, key, which) = match &finding.location {
            Location::Section(v) => (Some(v.as_str()), None, None, 0),
            Location::Line(v) => (None, Some(v.to_string()), None, 1),
            Location::Key(v) => (None, None, Some(v.as_str()), 2),
            Location::Document => (None, None, None, 3),
            _ => panic!("unhandled location"),
        };
        seen[which] = true;
        assert_eq!(row.text("section"), section);
        assert_eq!(row.text("line"), line.as_deref());
        assert_eq!(row.text("key"), key);
    }
    assert_eq!(seen, [true; 4]);
    assert_eq!(repo.both(&["check"]).expect(1).lines().len(), rows.len());
}

#[test]
fn quiet_keeps_findings_and_an_empty_result_is_still_a_canonical_graph() {
    let repo = findings_repo("rdf-quiet");
    for format in ["turtle", "ntriples"] {
        let loud = repo.both(&["--format", format, "check"]).expect(1);
        let quiet = repo.both(&["-q", "--format", format, "check"]).expect(1);
        assert_eq!(quiet.stdout, loud.stdout);
        let status = repo.both(&["-q", "--format", format, "status"]).expect(0);
        assert_eq!(Rdf::of(&status).result_count(), 0);
        assert_eq!(status.stderr, "");
    }
    let clean = Repo::synced("rdf-clean", false);
    let result = clean.both(&["--format", "turtle", "check"]).expect(0);
    assert_eq!(Rdf::of(&result).result_count(), 0);
}

#[test]
fn status_preserves_ordered_capabilities_binding_gaps_and_open_changes() {
    let repo = Repo::synced("rdf-status", true);
    let run = repo.both(&["--format", "turtle", "status"]).expect(0);
    let rdf = Rdf::of(&run);
    let states = rdf.rows("State");
    assert_eq!(
        states
            .iter()
            .map(|s| s.required("state"))
            .collect::<Vec<_>>(),
        ["delivered", "implementing", "designed", "considered"]
    );
    let beta = states[1].children("capability")[0];
    assert_eq!(states[1].number("count"), 1);
    assert_eq!(beta.required("id"), "cap:beta");
    assert_eq!(beta.required("path"), "docs/capabilities/beta.md");
    assert_eq!(beta.children("openChange")[0].required("slug"), "grow");
    let gaps = beta.children("gap");
    assert_eq!(
        gaps.iter().map(|g| g.required("list")).collect::<Vec<_>>(),
        ["code", "tests", "examples"]
    );
    assert_eq!(gaps[0].required("binding"), "demo::missing");
    assert_eq!(
        gaps[0].required("message"),
        "bindings.code `demo::missing` does not resolve"
    );
    assert!(gaps[1].text("binding").is_none());
    assert!(gaps[2].text("binding").is_none());
    for other in [0, 2, 3] {
        assert!(
            states[other].children("capability")[0]
                .children("gap")
                .is_empty()
        );
    }
    let open = rdf.rows("OpenChanges")[0];
    assert_eq!(open.number("count"), 1);
    let change = open.children("change")[0];
    assert_eq!(change.required("id"), "change:grow");
    assert_eq!(change.required("path"), "work/changes/grow.md");
    assert_eq!(
        change
            .children("capability")
            .iter()
            .map(|c| c.required("id"))
            .collect::<Vec<_>>(),
        ["cap:beta", "cap:delta", "cap:gamma"]
    );
}

#[test]
fn preview_preserves_reconciled_text_and_sync_reports_only_actual_rewrites() {
    let repo = Repo::synced("rdf-preview", false);
    let run = repo
        .both(&["--format", "turtle", "docs", "preview", "grow"])
        .expect(0);
    let rdf = Rdf::of(&run);
    let previews = rdf.rows("Preview");
    assert_eq!(previews.len(), 3);
    let delta = previews
        .iter()
        .find(|p| p.required("capability") == "cap:delta")
        .unwrap();
    assert_eq!(delta.required("path"), "docs/capabilities/delta.md");
    assert!(
        delta
            .required("text")
            .contains("Delta reads streams as they arrive.")
    );
    assert!(!delta.required("text").contains("PROPOSED"));
    let repo = Repo::roadmap("rdf-sync", false);
    let args = [
        "--format",
        "turtle",
        "docs",
        "sync",
        "--copyright",
        COPYRIGHT,
        "--licence",
        LICENCE,
    ];
    let first = repo.both_writing(&args).expect(0);
    let rdf = Rdf::of(&first);
    assert_eq!(
        rdf.rows("Rewritten")
            .iter()
            .map(|row| row.required("path"))
            .collect::<Vec<_>>(),
        ["docs/capabilities/alpha.md", "docs/capabilities/index.md"]
    );
    assert_eq!(Rdf::of(&repo.both(&args).expect(0)).result_count(), 0);
    exported_authoring_documents(&repo);
}

const DOCUMENTS: [(&str, &str); raf_okf::DocType::ALL.len()] = [
    (
        "docs/capabilities/ledger-sync.md",
        include_str!("../../raf-okf/tests/fixtures/capability.md"),
    ),
    (
        "work/changes/ledger-rows.md",
        include_str!("../../raf-okf/tests/fixtures/change.md"),
    ),
    (
        "docs/architecture/row-keys.md",
        include_str!("../../raf-okf/tests/fixtures/architecture.md"),
    ),
    (
        "work/evidence/ledger-sync-bound.md",
        include_str!("../../raf-okf/tests/fixtures/evidence.md"),
    ),
    (
        "work/assessments/ledger-sync-explanation.md",
        include_str!("../../raf-okf/tests/fixtures/assessment.md"),
    ),
    (
        "work/coverage/default.md",
        include_str!("../../raf-okf/tests/fixtures/coverage.md"),
    ),
    (
        "work/debt/default.md",
        include_str!("../../raf-okf/tests/fixtures/debt.md"),
    ),
    (
        "work/reviews/ledger-design.md",
        include_str!("../../raf-okf/tests/fixtures/review.md"),
    ),
];

fn exported_authoring_documents(repo: &Repo) {
    use purrdf_rdf::{ParseOptions, parse_dataset_with};
    use raf_okf::Document;
    for (path, text) in DOCUMENTS {
        repo.put(path, text);
        let document = Document::parse(text).unwrap();
        for (format, media) in [
            ("turtle", "text/turtle"),
            ("ntriples", "application/n-triples"),
        ] {
            let args = ["--format", format, "docs", "export", path];
            let before = repo.snapshot();
            let run = repo.both(&args).expect(0);
            assert_eq!(run.stderr, "");
            let parsed =
                parse_dataset_with(run.stdout.as_bytes(), media, None, &ParseOptions::default())
                    .unwrap();
            assert_eq!(
                Document::from_dataset(&parsed.dataset, path).unwrap(),
                document
            );
            assert_eq!(repo.both(&args).expect(0).stdout, run.stdout);
            assert_eq!(repo.snapshot(), before, "{path}: export changed inputs");
        }
    }
    let (path, text) = DOCUMENTS[0];
    repo.put(
        path,
        &text.replace(
            "code: [crates/ledger/src/sync.rs]",
            "code: ['same', \"same\", same]",
        ),
    );
    let expected = Document::parse(&repo.read(path)).unwrap();
    let before = repo.snapshot();
    let run = repo
        .both(&["--format", "ntriples", "docs", "export", path])
        .expect(0);
    let parsed = parse_dataset_with(
        run.stdout.as_bytes(),
        "application/n-triples",
        None,
        &ParseOptions::default(),
    )
    .unwrap();
    assert_eq!(
        Document::from_dataset(&parsed.dataset, path).unwrap(),
        expected
    );
    assert_eq!(repo.snapshot(), before);
}

#[test]
fn failures_have_no_machine_output_and_json_is_rejected_before_root_loading() {
    let repo = Repo::bare("rdf-failure");
    for format in ["turtle", "ntriples"] {
        let before = repo.snapshot();
        let quiet = repo
            .both(&["--quiet", "--format", format, "evidence", "claims", "peak"])
            .expect(2);
        assert_eq!(quiet.stdout, "");
        assert!(quiet.stderr.contains("refuses --quiet"), "{quiet:#?}");
        assert!(!quiet.stderr.contains("cannot open"));
        assert_eq!(repo.snapshot(), before);
        let run = repo.rooted(&["--format", format, "check"]).expect(2);
        assert_eq!(run.stdout, "");
        assert!(run.stderr.starts_with("cargo-workplan: "));
        assert_eq!(
            repo.both(&["--format", format, "frobnicate"])
                .expect(2)
                .stdout,
            ""
        );
    }
    for format in ["json", "json-pretty", "JSON"] {
        let run = repo.both(&["--format", format, "check"]).expect(2);
        assert_eq!(run.stdout, "");
        assert!(run.stderr.contains("invalid value"));
        assert!(!run.stderr.contains("cannot open"));
    }
    refused_authoring_documents(&repo);
}

fn refused_authoring_documents(repo: &Repo) {
    let (path, valid) = DOCUMENTS[2];
    for (selected, text, expected) in [
        ("docs/architecture/missing.md", None, "missing.md"),
        ("docs/capabilities/index.md", None, "navigation Index"),
        ("../outside.md", None, "outside"),
        ("/outside.md", None, "absolute"),
        (path, Some("not frontmatter".into()), "fence line"),
        (
            path,
            Some(valid.replace("id: arch:row-keys", "id: arch:wrong")),
            "id",
        ),
        (
            path,
            Some("x".repeat(raf_okf::MAX_TEXT_BYTES + 1)),
            "1048576",
        ),
        (
            path,
            Some("x".repeat(raf_okf::MAX_TEXT_BYTES + 2)),
            "1048577",
        ),
    ] {
        if let Some(text) = text {
            repo.put(selected, &text);
        }
        let before = repo.snapshot();
        let run = repo
            .both(&["--format", "ntriples", "docs", "export", selected])
            .expect(2);
        assert_eq!(run.stdout, "");
        assert!(run.stderr.contains(expected), "{selected}: {run:#?}");
        assert_eq!(
            repo.snapshot(),
            before,
            "{selected}: refusal changed inputs"
        );
    }
    let (cap, text) = DOCUMENTS[0];
    let many = format!("code: [{}x]", "x, ".repeat(65_536));
    repo.put(
        cap,
        &text.replace("code: [crates/ledger/src/sync.rs]", &many),
    );
    let before = repo.snapshot();
    let run = repo
        .both(&["--format", "turtle", "docs", "export", cap])
        .expect(2);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.contains("131072 structural nodes"), "{run:#?}");
    assert_eq!(repo.snapshot(), before);
    repo.put(path, valid);
    let before = repo.snapshot();
    repo.both(&["--format", "ntriples", "docs", "export", path])
        .expect(0);
    assert_eq!(repo.snapshot(), before);
    #[cfg(unix)]
    {
        let outside = support::Scratch::new("export-outside");
        let file = outside.path().join("escape.md");
        std::fs::write(&file, valid.replace("arch:row-keys", "arch:escape")).unwrap();
        std::os::unix::fs::symlink(&file, repo.root().join("docs/architecture/escape.md")).unwrap();
        let before = repo.snapshot();
        let outside_before = std::fs::read(&file).unwrap();
        let run = repo
            .both(&[
                "--format",
                "ntriples",
                "docs",
                "export",
                "docs/architecture/escape.md",
            ])
            .expect(2);
        assert_eq!(run.stdout, "");
        assert!(run.stderr.contains("outside"), "{run:#?}");
        assert_eq!(repo.snapshot(), before);
        assert_eq!(std::fs::read(&file).unwrap(), outside_before);
    }
}
