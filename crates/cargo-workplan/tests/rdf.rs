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
    let findings = Gate::workspace(repo.root()).check();
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
}

#[test]
fn failures_have_no_machine_output_and_json_is_rejected_before_root_loading() {
    let repo = Repo::bare("rdf-failure");
    for format in ["turtle", "ntriples"] {
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
}
