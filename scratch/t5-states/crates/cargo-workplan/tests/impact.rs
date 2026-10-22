// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `change impact` through the built binary: the text it prints (the
//! library's), the exit 2 of a slug that names no record or a record that does
//! not load, `-q`, the JSON object, and that it writes nothing.

mod support;

use serde_json::json;
use support::{Repo, array, member_names, strings};
use support::{change, run_in};

/// What `change impact grow` prints for the roadmap repository with the
/// binding `demo::missing` left unresolved.
const GROW: &str = "\
change:grow
capabilities:
  cap:beta
  cap:delta
  cap:gamma
sections:
  cap:beta#contract
  cap:delta#contract
  cap:gamma#contract
bindings:
  cap:beta  code  demo::missing  does not resolve: `demo::missing` names nothing in the workspace
  cap:beta  code  demo::real  resolves
dependants:
  none
linking sections:
  none
";

#[test]
fn impact_prints_what_the_change_reaches_and_writes_nothing() {
    let repo = Repo::synced("impact-text", true);
    let before = repo.snapshot();
    let run = repo.both(&["change", "impact", "grow"]).expect(0);
    assert_eq!(run.stdout, GROW);
    assert_eq!(run.stderr, "");
    assert_eq!(repo.snapshot(), before, "impact wrote");
}

#[test]
fn a_change_that_reaches_dependants_and_linking_sections_lists_them_sorted() {
    let repo = Repo::synced("impact-reach", false);
    repo.put(
        "work/changes/watch.md",
        &change("watch", "[cap:alpha]", &["cap:alpha#contract"]),
    );
    // A section of another capability links to the affected section.
    repo.edit(
        "docs/capabilities/gamma.md",
        "Text for Usage of gamma.",
        "Text for Usage of gamma, see [it](alpha.md#contract).",
    );
    repo.sync();
    let before = repo.snapshot();
    let run = repo.both(&["change", "impact", "watch"]).expect(0);
    assert_eq!(
        run.stdout,
        "\
change:watch
capabilities:
  cap:alpha
sections:
  cap:alpha#contract
bindings:
  cap:alpha  code  demo::real  resolves
  cap:alpha  tests  demo::test_crate::it::case  resolves
dependants:
  cap:beta depends on cap:alpha
  cap:gamma depends on cap:alpha
linking sections:
  cap:gamma#usage links to cap:alpha#contract
"
    );
    assert_eq!(repo.snapshot(), before);
}

#[test]
fn a_slug_with_no_record_is_exit_two_and_names_the_open_changes() {
    let repo = Repo::synced("impact-unknown", false);
    let before = repo.snapshot();
    let run = repo.both(&["change", "impact", "nowhere"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(
        run.stderr.contains("no change record `nowhere`") && run.stderr.contains("grow"),
        "{run:#?}"
    );
    assert_eq!(repo.snapshot(), before);
}

#[test]
fn a_record_that_does_not_load_is_exit_two_with_its_findings() {
    let repo = Repo::synced("impact-unloaded", false);
    repo.edit(
        "work/changes/grow.md",
        "items: []",
        "items:\n  - claim: \"x\"\n    outcome: bogus",
    );
    let run = repo.both(&["change", "impact", "grow"]).expect(2);
    assert_eq!(run.stdout, "");
    assert!(run.stderr.contains("did not load"), "{run:#?}");
}

#[test]
fn quiet_prints_nothing_and_the_exit_stays_zero() {
    let repo = Repo::synced("impact-quiet", false);
    let run = repo.both(&["-q", "change", "impact", "grow"]).expect(0);
    assert_eq!(run.stdout, "");
    // A failure is said under `-q`.
    let failed = repo.run(&["-q", "change", "impact", "nowhere"]).expect(2);
    assert!(failed.stderr.contains("no change record"), "{failed:#?}");
}

#[test]
fn json_is_one_impact_object_with_its_lists() {
    let repo = Repo::synced("impact-json", true);
    let run = repo
        .both(&["--format", "json", "change", "impact", "grow"])
        .expect(0);
    let objects = array(&run);
    assert_eq!(objects.len(), 1, "{run:#?}");
    assert_eq!(
        member_names(&run),
        [strings(&[
            "kind",
            "change",
            "capabilities",
            "sections",
            "bindings",
            "dependants",
            "linking"
        ])]
    );
    let object = &objects[0];
    assert_eq!(object["kind"], "impact");
    assert_eq!(object["change"], "grow");
    assert_eq!(
        object["capabilities"],
        json!(["cap:beta", "cap:delta", "cap:gamma"])
    );
    assert_eq!(
        object["sections"],
        json!([
            "cap:beta#contract",
            "cap:delta#contract",
            "cap:gamma#contract"
        ])
    );
    let bindings = object["bindings"].as_array().expect("bindings");
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0]["path"], "demo::missing");
    assert_eq!(bindings[0]["resolves"], false);
    assert!(bindings[0]["reason"].is_string());
    assert_eq!(bindings[0]["list"], "code");
    assert_eq!(bindings[0]["capability"], "cap:beta");
    assert_eq!(bindings[1]["path"], "demo::real");
    assert_eq!(bindings[1]["resolves"], true);
    assert_eq!(bindings[1]["reason"], serde_json::Value::Null);
    assert_eq!(object["dependants"], json!([]));
    assert_eq!(object["linking"], json!([]));

    // The dependants and linking objects have their own fields.
    let repo = Repo::synced("impact-json-reach", false);
    repo.put(
        "work/changes/watch.md",
        &change("watch", "[cap:alpha]", &["cap:alpha#contract"]),
    );
    repo.edit(
        "docs/capabilities/gamma.md",
        "Text for Usage of gamma.",
        "Text for Usage of gamma, see [it](alpha.md#contract).",
    );
    repo.sync();
    let run = repo
        .run(&["--format", "json", "change", "impact", "watch"])
        .expect(0);
    let object = &array(&run)[0];
    assert_eq!(
        object["dependants"],
        json!([
            {"capability": "cap:beta", "depends_on": "cap:alpha"},
            {"capability": "cap:gamma", "depends_on": "cap:alpha"},
        ])
    );
    assert_eq!(
        object["linking"],
        json!([{"from": "cap:gamma#usage", "target": "cap:alpha#contract"}])
    );
}

#[test]
fn impact_finds_the_root_from_below_and_by_root() {
    let repo = Repo::synced("impact-root", false);
    let direct = repo.run(&["change", "impact", "grow"]).expect(0);
    let below = run_in(
        &repo.root().join("docs/capabilities"),
        support::Shape::Cargo,
        &["change", "impact", "grow"],
    )
    .expect(0);
    assert_eq!(below.stdout, direct.stdout);
    let rooted = repo.rooted(&["change", "impact", "grow"]).expect(0);
    assert_eq!(rooted.stdout, direct.stdout);
}
