// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `--format json` through the built binary: one array for the run, one object
//! for each printed record with its `kind` first and the fields its kind has,
//! the same records as the text form, and nothing on standard output when the
//! tool could not run.

mod support;

use raf_okf::Location;
use raf_workplan::{Finding, Gate};
use serde_json::Value;
use support::{COPYRIGHT, LICENCE, Repo, Run};

/// The one array standard output holds.
fn array(run: &Run) -> Vec<Value> {
    let value: Value = serde_json::from_str(&run.stdout)
        .unwrap_or_else(|e| panic!("standard output is not one JSON value: {e}\n{run:#?}"));
    match value {
        Value::Array(items) => items,
        other => panic!("standard output is not an array: {other}"),
    }
}

/// The member names of each object of the array, in the order written.
///
/// The array is printed one object to a line, so the names are read from the
/// text: `serde_json::Value` sorts them.
fn member_names(run: &Run) -> Vec<Vec<String>> {
    run.stdout
        .lines()
        .filter(|line| line.starts_with('{'))
        .map(|line| {
            let line = line.trim_end_matches(',');
            let mut names = Vec::new();
            let mut depth = 0_i32;
            let mut chars = line.char_indices().peekable();
            while let Some((at, c)) = chars.next() {
                match c {
                    '{' | '[' => depth += 1,
                    '}' | ']' => depth -= 1,
                    '"' => {
                        // A string: find its end, honouring escapes.
                        let start = at;
                        let mut end = at;
                        let mut escaped = false;
                        for (i, d) in chars.by_ref() {
                            end = i;
                            if escaped {
                                escaped = false;
                            } else if d == '\\' {
                                escaped = true;
                            } else if d == '"' {
                                break;
                            }
                        }
                        let is_name = depth == 1 && line[end + 1..].starts_with(':');
                        if is_name {
                            names.push(line[start + 1..end].to_string());
                        }
                    }
                    _ => {}
                }
            }
            names
        })
        .collect()
}

fn strings(names: &[&str]) -> Vec<String> {
    names.iter().map(ToString::to_string).collect()
}

const FINDING_FIELDS: [&str; 7] = ["kind", "path", "section", "line", "key", "message", "fix"];

/// A repository with findings at every kind of location: a key, a line, a
/// section and the document as a whole.
fn findings_repo(tag: &str) -> Repo {
    let repo = Repo::synced(tag, true);
    repo.edit("docs/capabilities/alpha.md", "Text for Usage of alpha.", "");
    repo.put("docs/architecture/notes.md", "# Notes\n");
    // A stale block is reported at a line; a missing index is not.
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
    repo
}

/// The repository of [`findings_repo`] with its index removed, which adds a
/// finding about the document as a whole.
fn findings_repo_without_index(tag: &str) -> Repo {
    let repo = findings_repo(tag);
    std::fs::remove_file(repo.root().join("docs/capabilities/index.md")).expect("remove the index");
    repo
}

#[test]
fn a_clean_run_prints_one_empty_array() {
    let repo = Repo::synced("json-clean", false);
    let run = repo.both(&["--format", "json", "check"]).expect(0);
    assert_eq!(run.stdout, "[]\n");
    assert_eq!(run.stderr, "");
}

#[test]
fn check_prints_one_object_for_each_finding_of_the_gate_with_the_fields_of_its_location() {
    let repo = findings_repo_without_index("json-check");
    let gate = Gate::workspace(repo.root()).check();
    let run = repo.both(&["--format", "json", "check"]).expect(1);
    let objects = array(&run);
    assert_eq!(objects.len(), gate.len(), "{run:#?}");
    let mut seen_locations = [false; 4];
    for (object, finding) in objects.iter().zip(&gate) {
        assert_eq!(object["kind"], finding.kind);
        assert_eq!(object["path"], finding.file.as_str());
        assert_eq!(object["message"], finding.message.as_str());
        assert_eq!(object["fix"], finding.fix.as_str());
        let (section, line, key, which) = match &finding.location {
            Location::Section(name) => (Value::from(name.as_str()), Value::Null, Value::Null, 0),
            Location::Line(line) => (Value::Null, Value::from(*line), Value::Null, 1),
            Location::Key(key) => (Value::Null, Value::Null, Value::from(key.as_str()), 2),
            _ => (Value::Null, Value::Null, Value::Null, 3),
        };
        seen_locations[which] = true;
        assert_eq!(object["section"], section, "{finding}");
        assert_eq!(object["line"], line, "{finding}");
        assert_eq!(object["key"], key, "{finding}");
        assert!(Finding::KINDS.contains(&finding.kind));
    }
    assert_eq!(
        seen_locations, [true; 4],
        "the fixture covers each location"
    );
    for names in member_names(&run) {
        assert_eq!(names, strings(&FINDING_FIELDS));
    }
}

#[test]
fn the_json_and_the_text_forms_print_the_same_findings_in_the_same_order() {
    let repo = findings_repo("json-text");
    let text = repo.both(&["check"]).expect(1);
    let json = repo.both(&["--format", "json", "check"]).expect(1);
    let from_json: Vec<String> = array(&json)
        .iter()
        .map(|o| {
            format!(
                "{}|{}",
                o["kind"].as_str().unwrap_or(""),
                o["path"].as_str().unwrap_or("")
            )
        })
        .collect();
    let from_text: Vec<String> = Gate::workspace(repo.root())
        .check()
        .iter()
        .map(|f| format!("{}|{}", f.kind, f.file))
        .collect();
    assert_eq!(from_json, from_text);
    assert_eq!(text.lines().len(), from_json.len());
}

#[test]
fn quiet_keeps_the_findings_in_json_and_prints_an_empty_array_without_any() {
    let repo = findings_repo("json-quiet");
    let loud = repo.both(&["--format", "json", "check"]).expect(1);
    let quiet = repo.both(&["-q", "--format", "json", "check"]).expect(1);
    assert_eq!(quiet.stdout, loud.stdout);
    let status = repo.both(&["-q", "--format", "json", "status"]).expect(0);
    assert_eq!(status.stdout, "[]\n");
}

#[test]
fn status_prints_a_state_object_for_each_state_and_the_open_changes() {
    let repo = Repo::synced("json-status", true);
    let run = repo.both(&["--format", "json", "status"]).expect(0);
    let objects = array(&run);
    let kinds: Vec<&str> = objects
        .iter()
        .map(|o| o["kind"].as_str().expect("a kind"))
        .collect();
    assert_eq!(kinds, ["state", "state", "state", "state", "open-changes"]);
    let states: Vec<&str> = objects[..4]
        .iter()
        .map(|o| o["state"].as_str().expect("a state"))
        .collect();
    assert_eq!(
        states,
        ["delivered", "implementing", "designed", "considered"]
    );
    let names = member_names(&run);
    for state_names in &names[..4] {
        assert_eq!(
            *state_names,
            strings(&["kind", "state", "count", "capabilities"])
        );
    }
    assert_eq!(names[4], strings(&["kind", "count", "changes"]));

    let beta = &objects[1]["capabilities"][0];
    assert_eq!(objects[1]["count"], 1);
    assert_eq!(beta["id"], "cap:beta");
    assert_eq!(beta["slug"], "beta");
    assert_eq!(beta["path"], "docs/capabilities/beta.md");
    assert_eq!(beta["open_changes"], serde_json::json!(["grow"]));
    assert_eq!(
        beta["gaps"],
        serde_json::json!([
            {"list": "code", "binding": "demo::missing", "message": "bindings.code `demo::missing` does not resolve"},
            {"list": "tests", "binding": null, "message": "bindings.tests is empty"},
            {"list": "examples", "binding": null, "message": "bindings.examples is empty"},
        ])
    );
    // Only an implementing capability has gaps.
    for other in [0, 2, 3] {
        assert_eq!(
            objects[other]["capabilities"][0]["gaps"],
            serde_json::json!([])
        );
    }
    let open = &objects[4];
    assert_eq!(open["count"], 1);
    assert_eq!(open["changes"][0]["id"], "change:grow");
    assert_eq!(open["changes"][0]["slug"], "grow");
    assert_eq!(open["changes"][0]["path"], "work/changes/grow.md");
    assert_eq!(
        open["changes"][0]["capabilities"],
        serde_json::json!(["cap:beta", "cap:delta", "cap:gamma"])
    );
}

#[test]
fn preview_prints_a_preview_object_for_each_capability_with_the_reconciled_text() {
    let repo = Repo::synced("json-preview", false);
    let run = repo
        .both(&["--format", "json", "docs", "preview", "grow"])
        .expect(0);
    let objects = array(&run);
    assert_eq!(objects.len(), 3);
    for names in member_names(&run) {
        assert_eq!(names, strings(&["kind", "path", "capability", "text"]));
    }
    let delta = objects
        .iter()
        .find(|o| o["capability"] == "cap:delta")
        .expect("delta");
    assert_eq!(delta["kind"], "preview");
    assert_eq!(delta["path"], "docs/capabilities/delta.md");
    let text = delta["text"].as_str().expect("text");
    assert!(text.contains("Delta reads streams as they arrive."));
    assert!(!text.contains("PROPOSED"));
}

#[test]
fn sync_prints_a_rewrote_object_for_each_file_and_an_empty_array_for_none() {
    let repo = Repo::roadmap("json-sync", false);
    let args = [
        "--format",
        "json",
        "docs",
        "sync",
        "--copyright",
        COPYRIGHT,
        "--licence",
        LICENCE,
    ];
    let run = repo.both_writing(&args).expect(0);
    let objects = array(&run);
    assert_eq!(
        objects,
        [serde_json::json!({"kind": "rewrote", "path": "docs/capabilities/index.md"})]
    );
    assert_eq!(member_names(&run), [strings(&["kind", "path"])]);
    let second = repo.both(&args).expect(0);
    assert_eq!(second.stdout, "[]\n");
}

#[test]
fn a_run_that_could_not_run_prints_nothing_on_standard_output_in_json_too() {
    let repo = Repo::bare("json-failure");
    let run = repo.rooted(&["--format", "json", "check"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.starts_with("cargo-workplan: "), "{run:#?}");
    let usage = repo.both(&["--format", "json", "frobnicate"]).expect(2);
    assert_eq!(usage.stdout, "");
}

#[test]
fn the_kinds_that_are_not_rules_are_not_rule_names() {
    for kind in ["rewrote", "state", "open-changes", "preview"] {
        assert!(!Finding::KINDS.contains(&kind), "{kind} is also a rule");
    }
}

#[test]
fn every_object_is_a_single_line_inside_one_array() {
    let repo = findings_repo("json-lines");
    let run = repo.both(&["--format", "json", "check"]).expect(1);
    let lines = run.lines();
    assert_eq!(lines.first(), Some(&"["));
    assert_eq!(lines.last(), Some(&"]"));
    let inner = &lines[1..lines.len() - 1];
    for (i, line) in inner.iter().enumerate() {
        assert!(line.starts_with("{\"kind\":\""), "{line}");
        assert_eq!(line.ends_with("},"), i + 1 < inner.len(), "{line}");
    }
}
