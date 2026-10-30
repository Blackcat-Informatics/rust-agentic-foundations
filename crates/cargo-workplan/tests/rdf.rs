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
        .find(|p| p.required("subject") == "cap:delta")
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
    for (format, message) in [
        ("json", "JSON output requires the diagram or review command"),
        ("svg", "SVG output requires the diagram command"),
        ("json-pretty", "invalid value"),
        ("JSON", "invalid value"),
    ] {
        let run = repo.both(&["--format", format, "check"]).expect(2);
        assert_eq!(run.stdout, "");
        assert!(run.stderr.contains(message), "{run:#?}");
        assert!(!run.stderr.contains("cannot open"));
    }
    let run = repo.both(&["--format", "svg", "review", "list"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr
            .contains("SVG output requires the diagram command"),
        "{run:#?}"
    );
    assert!(!run.stderr.contains("cannot open"));
    refused_authoring_documents(&repo);
    mdbook_protocol_refuses_reports_before_loading(&repo);
}

fn mdbook_protocol_refuses_reports_before_loading(repo: &Repo) {
    // The explicit root does not exist and this runner supplies no stdin.
    // Renderer probes must still return only their protocol status.
    for (renderer, code) in [("html", 0), ("latex", 1)] {
        let run = repo
            .both(&[
                "--root",
                "missing-root",
                "docs",
                "mdbook",
                "supports",
                renderer,
            ])
            .expect(code);
        assert_eq!(run.stdout, "");
        assert_eq!(run.stderr, "");
    }
    for format in ["turtle", "ntriples"] {
        let run = repo
            .both(&[
                "--root",
                "missing-root",
                "--format",
                format,
                "docs",
                "mdbook",
            ])
            .expect(2);
        assert_eq!(run.stdout, "");
        assert!(run.stderr.contains("uses its JSON protocol"), "{run:#?}");
    }
    let quiet = repo
        .both(&["--root", "missing-root", "--quiet", "docs", "mdbook"])
        .expect(2);
    assert_eq!(quiet.stdout, "");
    assert!(quiet.stderr.contains("refuses --quiet"), "{quiet:#?}");
    let empty = repo
        .both(&["--root", "missing-root", "docs", "mdbook"])
        .expect(2);
    assert_eq!(empty.stdout, "");
    assert!(
        empty.stderr.contains("invalid mdBook request"),
        "{empty:#?}"
    );
    mdbook_preserves_the_book_and_uses_the_shared_view();
}

fn diagram_repo(tag: &str) -> Repo {
    let repo = Repo::synced(tag, false);
    let path = "docs/capabilities/alpha.md";
    repo.edit(
        path,
        "bindings:",
        "diagrams:\n  - name: dependencies\n    kind: capability-dependencies\n    scope: workspace\nbindings:",
    );
    repo.put(
        path,
        &format!(
            "{}\n<!-- raf:generated:diagram-dependencies -->\n<!-- raf:generated:end -->\n",
            repo.read(path)
        ),
    );
    repo
}

#[test]
fn standalone_publication_preserves_host_files_and_refuses_changed_owned_bytes() {
    let repo = diagram_repo("standalone-publication");
    repo.put("published-docs/demo/index.html", "unrelated Rustdoc output");
    let args = [
        "--format",
        "ntriples",
        "docs",
        "build",
        "standalone",
        "--view",
        "cap:alpha#dependencies",
        "--output-dir",
        "published-docs",
    ];
    let result = repo.both(&args).expect(0);
    let rdf = Rdf::of(&result);
    let rows = rdf.rows("DocumentationBuilt");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].required("view"), "cap:alpha#dependencies");
    let directory = std::path::Path::new(rows[0].required("directory"));
    assert!(std::path::Path::new(rows[0].required("entry")).is_file());
    assert_eq!(
        repo.read("published-docs/demo/index.html"),
        "unrelated Rustdoc output"
    );
    let svg = directory.join("dependencies.svg");
    let expected = std::fs::read(&svg).unwrap();
    std::fs::write(&svg, "changed owned bytes").unwrap();
    let before = repo.snapshot();
    let refused = repo.both(&args).expect(2);
    assert_eq!(refused.stdout, "");
    assert!(refused.stderr.contains("changed"), "{refused:#?}");
    assert_eq!(repo.snapshot(), before);
    std::fs::write(&svg, &expected).unwrap();
    repo.both(&args).expect(0);
    std::fs::remove_file(&svg).unwrap();
    repo.both(&args).expect(0);
    assert_eq!(std::fs::read(&svg).unwrap(), expected);
    #[cfg(unix)]
    {
        std::fs::remove_file(&svg).unwrap();
        std::os::unix::fs::symlink(repo.root().join("published-docs/demo/index.html"), &svg)
            .unwrap();
        let before = repo.snapshot();
        let refused = repo.both(&args).expect(2);
        assert_eq!(refused.stdout, "");
        assert_eq!(repo.snapshot(), before);
        std::fs::remove_file(&svg).unwrap();
        std::fs::write(&svg, &expected).unwrap();
        repo.both(&args).expect(0);
    }
    std::fs::write(directory.join("unrelated.txt"), "preserve this file").unwrap();
    let before = repo.snapshot();
    let refused = repo.both(&args).expect(2);
    assert_eq!(refused.stdout, "");
    assert_eq!(repo.snapshot(), before);
    repo.edit(
        "docs/capabilities/alpha.md",
        "name: dependencies",
        "name: revised-dependencies",
    );
    repo.edit(
        "docs/capabilities/alpha.md",
        "diagram-dependencies",
        "diagram-revised-dependencies",
    );
    let mut revised = args;
    revised[6] = "cap:alpha#revised-dependencies";
    let result = repo.both(&revised).expect(0);
    let rdf = Rdf::of(&result);
    let rows = rdf.rows("DocumentationBuilt");
    assert_ne!(rows[0].required("directory"), directory.to_str().unwrap());
    assert_eq!(std::fs::read(&svg).unwrap(), expected);
    assert_eq!(
        std::fs::read_to_string(directory.join("unrelated.txt")).unwrap(),
        "preserve this file"
    );
    assert_eq!(
        repo.read("published-docs/demo/index.html"),
        "unrelated Rustdoc output"
    );
}

fn mdbook_preserves_the_book_and_uses_the_shared_view() {
    use purrdf_lex::json::{self, Object, Value};
    let repo = diagram_repo("mdbook-shared-view");
    let opened = Gate::workspace(repo.root()).open().unwrap();
    let bundle = opened
        .workplan()
        .named_diagram("cap:alpha#dependencies")
        .unwrap();
    let fragment = raf_workplan::diagram::documentation::static_fragment(&bundle).unwrap();
    let literal = "```text\n{{#workplan-diagram cap:alpha#dependencies}}\n```\n\n";
    let chapter = Object::new()
        .with("name", "Nested page")
        .with("path", "nested/page.md")
        .with("source_path", "nested/page.md")
        .with("sub_items", Vec::<Value>::new())
        .with(
            "content",
            format!("{literal}{{{{#workplan-diagram cap:alpha#dependencies}}}}\n\n# After\n"),
        );
    let mut expected = Value::from(
        Object::new()
            .with(
                "sections",
                vec![
                    Value::from("Separator"),
                    Value::from(Object::new().with("PartTitle", "General")),
                    Value::from(Object::new().with("Chapter", chapter)),
                ],
            )
            .with("opaque_metadata", "preserve this field"),
    );
    let context = Value::from(
        Object::new()
            .with("renderer", "html")
            .with("root", "not-the-source-authority"),
    );
    let request = json::write_compact(&Value::from(vec![context, expected.clone()]));
    repo.put("book-request.json", &request);
    let input = repo.root().join("book-request.json");
    let before = repo.snapshot();
    let result = support::run_both_with_stdin(repo.root(), &["docs", "mdbook"], &input).expect(0);
    let chapter = expected
        .get_mut("sections")
        .unwrap()
        .as_array_mut()
        .unwrap()[2]
        .get_mut("Chapter")
        .unwrap();
    *chapter.get_mut("content").unwrap() = Value::from(format!("{literal}{fragment}\n# After\n"));
    assert_eq!(json::read(&result.stdout).unwrap(), expected);
    assert_eq!(repo.snapshot(), before);
    for (request, reason) in [
        (format!("{request} null"), "invalid mdBook request"),
        (
            request.replace("cap:alpha#dependencies", "cap:alpha#missing"),
            "no declared diagram",
        ),
    ] {
        repo.put("book-request.json", &request);
        let refused =
            support::run_both_with_stdin(repo.root(), &["docs", "mdbook"], &input).expect(2);
        assert_eq!(refused.stdout, "");
        assert!(refused.stderr.contains(reason), "{refused:#?}");
    }
    mdbook_request_limits(&opened.workplan());
    mdbook_output_limit(opened.workspace());
}

fn book_chapter(
    content: String,
    children: Vec<purrdf_lex::json::Value>,
) -> purrdf_lex::json::Value {
    use purrdf_lex::json::{Object, Value};
    Value::from(
        Object::new().with(
            "Chapter",
            Object::new()
                .with("content", content)
                .with("sub_items", children),
        ),
    )
}

fn book_request(sections: Vec<purrdf_lex::json::Value>) -> purrdf_lex::json::Value {
    use purrdf_lex::json::{Object, Value};
    Value::from(vec![
        Value::from(Object::new().with("renderer", "html")),
        Value::from(Object::new().with("sections", sections)),
    ])
}

fn book_boundary(
    plan: &raf_workplan::Workplan<'_>,
    valid: purrdf_lex::json::Value,
    invalid: purrdf_lex::json::Value,
    reason: &str,
) {
    use raf_workplan::diagram::book::preprocess;
    preprocess(plan, valid).unwrap();
    let error = preprocess(plan, invalid).unwrap_err().to_string();
    assert!(error.contains(reason), "{reason}: {error}");
}

fn mdbook_request_limits(plan: &raf_workplan::Workplan<'_>) {
    let chapters = |count: usize, text: &str| {
        book_request(
            (0..count)
                .map(|_| book_chapter(text.to_owned(), Vec::new()))
                .collect(),
        )
    };
    let nested = |levels: usize| {
        let mut children = Vec::new();
        for _ in 0..levels {
            children = vec![book_chapter(String::new(), children)];
        }
        book_request(children)
    };
    let mebibyte = "x".repeat(raf_okf::MAX_TEXT_BYTES);
    book_boundary(
        plan,
        chapters(1, &mebibyte),
        chapters(1, &format!("{mebibyte}x")),
        "chapter source exceeds 1 MiB",
    );
    let mut excess_source = (0..16)
        .map(|_| book_chapter(mebibyte.clone(), Vec::new()))
        .collect::<Vec<_>>();
    excess_source.push(book_chapter("x".into(), Vec::new()));
    book_boundary(
        plan,
        chapters(16, &mebibyte),
        book_request(excess_source),
        "book source exceeds 16 MiB",
    );
    book_boundary(
        plan,
        chapters(4096, ""),
        chapters(4097, ""),
        "book exceeds 4096 chapters",
    );
    book_boundary(
        plan,
        nested(64),
        nested(65),
        "chapter nesting exceeds 64 levels",
    );
    book_boundary(
        plan,
        chapters(1, &format!("{}text", "> ".repeat(63))),
        chapters(1, &format!("{}text", "> ".repeat(64))),
        "Markdown nesting exceeds 64 containers",
    );
    let reference = "{{#workplan-diagram cap:alpha#dependencies}}\n\n";
    book_boundary(
        plan,
        chapters(1, &reference.repeat(256)),
        chapters(1, &reference.repeat(257)),
        "book exceeds 256 diagram references",
    );
}

fn mdbook_output_limit(source: &raf_source::Workspace) {
    use raf_source::MemoryTree;
    use raf_workplan::{Workplan, diagram::documentation::static_fragment};
    // A real larger semantic view reaches the output bound before the separate
    // source and reference bounds; no private accounting state is constructed.
    let docs = (0..128).map(|index| {
        let slug = format!("node-{index}");
        let mut text = support::capability(&slug, "considered", ["[]"; 3], "[]", &[]);
        if index == 0 {
            text = text.replacen(
                "bindings:",
                "diagrams:\n  - name: dependencies\n    kind: capability-dependencies\n    scope: workspace\nbindings:",
                1,
            );
        }
        (format!("docs/capabilities/{slug}.md"), text)
    });
    let tree = MemoryTree::try_from_iter(docs).unwrap();
    let plan = Workplan::load(&tree, source);
    let bundle = plan.named_diagram("cap:node-0#dependencies").unwrap();
    let fragment = static_fragment(&bundle).unwrap();
    let count = 64 * 1024 * 1024 / (fragment.len() + 1);
    assert!(
        (1..256).contains(&count),
        "fragment bytes: {}",
        fragment.len()
    );
    let reference = "{{#workplan-diagram cap:node-0#dependencies}}\n\n";
    let request = |count| book_request(vec![book_chapter(reference.repeat(count), Vec::new())]);
    book_boundary(
        &plan,
        request(count),
        request(count + 1),
        "transformed book content exceeds 64 MiB",
    );
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
