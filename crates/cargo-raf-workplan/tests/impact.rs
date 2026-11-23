// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `change impact` through the built binary: the text it prints (the
//! library's), the exit 2 of a slug that names no record or a record that does
//! not load, `-q`, the typed RDF result, and that it writes nothing.

mod support;
use support::rdf::Rdf;

use support::Repo;
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
  cap:alpha  examples  demo::real  resolves
dependants:
  cap:beta depends on cap:alpha
  cap:gamma depends on cap:alpha
linking sections:
  cap:alpha#verification links to cap:alpha#contract
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
fn rdf_preserves_impact_bindings_dependencies_links_and_list_order() {
    let repo = Repo::synced("impact-rdf", true);
    let run = repo
        .both(&["--format", "turtle", "change", "impact", "grow"])
        .expect(0);
    let rdf = Rdf::of(&run);
    let rows = rdf.rows("Impact");
    assert_eq!(rows.len(), 1);
    let impact = rows[0];
    assert_eq!(impact.required("change"), "grow");
    assert_eq!(
        impact
            .children("capability")
            .iter()
            .map(|r| r.required("id"))
            .collect::<Vec<_>>(),
        ["cap:beta", "cap:delta", "cap:gamma"]
    );
    assert_eq!(
        impact
            .children("section")
            .iter()
            .map(|r| r.required("subject"))
            .collect::<Vec<_>>(),
        [
            "cap:beta#contract",
            "cap:delta#contract",
            "cap:gamma#contract"
        ]
    );
    let bindings = impact.children("binding");
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].required("path"), "demo::missing");
    assert!(!bindings[0].boolean("resolves"));
    assert!(bindings[0].text("reason").is_some());
    assert_eq!(bindings[0].required("list"), "code");
    assert_eq!(bindings[0].required("capability"), "cap:beta");
    assert_eq!(bindings[1].required("path"), "demo::real");
    assert!(bindings[1].boolean("resolves"));
    assert!(bindings[1].text("reason").is_none());
    assert!(impact.children("dependant").is_empty());
    assert!(impact.children("link").is_empty());
    let repo = Repo::synced("impact-rdf-reach", false);
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
        .run(&["--format", "turtle", "change", "impact", "watch"])
        .expect(0);
    let rdf = Rdf::of(&run);
    let impact = rdf.rows("Impact")[0];
    let dependants = impact.children("dependant");
    assert_eq!(dependants.len(), 2);
    assert_eq!(
        dependants
            .iter()
            .map(|d| (d.required("capability"), d.required("dependsOn")))
            .collect::<Vec<_>>(),
        [("cap:beta", "cap:alpha"), ("cap:gamma", "cap:alpha")]
    );
    let links = impact.children("link");
    assert_eq!(links.len(), 2);
    assert_eq!(
        (links[0].required("from"), links[0].required("target")),
        ("cap:alpha#verification", "cap:alpha#contract")
    );
    assert_eq!(
        (links[1].required("from"), links[1].required("target")),
        ("cap:gamma#usage", "cap:alpha#contract")
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

/// A committed capability with a shared source file, a test, an example and an assessment.
fn source_impact_fixture() -> (Repo, String) {
    let repo = Repo::bare("source-impact");
    let cap = support::capability(
        "reader",
        "delivered",
        [
            "[demo::real, demo::other]",
            "[demo::test_crate::it::case]",
            "[demo::real]",
        ],
        "[]",
        &[(
            "Contract",
            "The guarantee covers `demo::real`.\n<!-- raf:tested-by demo::real -->\n",
        )],
    );
    repo.put("docs/capabilities/reader.md", &cap);
    let source = format!(
        "{}pub fn other() {{}}\n#[doc(hidden)]\npub fn orphan() {{}}\n\
         pub struct C; trait LocalOnly {{ fn hidden(&self); }}\n\
         impl LocalOnly for C {{ fn hidden(&self) {{}} }}\n",
        repo.read("src/lib.rs")
    );
    repo.put("src/lib.rs", &source);
    repo.run(&[
        "assess",
        "cap:reader#contract",
        "--verdict",
        "accepted",
        "--by",
        "human",
    ])
    .expect(0);
    repo.checkout();
    (repo, source)
}

/// An impact trace starts at the edited definition, even when several
/// documented and undocumented definitions share its file and Git revision.
#[test]
#[expect(
    clippy::expect_used,
    reason = "fixture prerequisites and required observations carry their assertion messages"
)]
fn source_impact_traces_bound_guarantees_and_marks_public_items_without_owners() {
    let (repo, source) = source_impact_fixture();
    repo.edit("src/lib.rs", "other() {}", "other() { let _x = 1; }");
    let neighbour = repo.run(&["impact", "--base", "HEAD"]).expect(0);
    assert!(
        neighbour.stdout.contains("changed demo::other"),
        "{neighbour:#?}"
    );
    assert!(
        !neighbour.stdout.contains("cap:reader#contract"),
        "{neighbour:#?}"
    );

    repo.put(
        "src/lib.rs",
        &source
            .replace("real() {}", "real() { let _y = 2; }")
            .replace("orphan() {}", "orphan() { let _z = 3; }")
            .replace("hidden(&self) {}", "hidden(&self) { let _w = 4; }"),
    );
    let before = repo.snapshot();
    let run = repo
        .both(&["impact", "--base", "refs/heads/main"])
        .expect(0);
    for expected in [
        "changed demo::real\n  public item with a documented owner",
        "cap:reader via code demo::real",
        "cap:reader#contract",
        "guarantee cap:reader#contract/result (named)",
        "guarantee cap:reader#contract/result (tested-by)",
        "cap:reader#purpose (all-code-bindings)",
        "test demo::test_crate::it::case",
        "example demo::real",
        "assessment assessment:reader-contract",
        "changed demo::orphan\n  public item without a documented owner",
    ] {
        assert!(
            run.stdout.contains(expected),
            "missing {expected}: {run:#?}"
        );
    }
    assert!(!run.stdout.contains("test demo::real\n"), "{run:#?}");
    assert!(!run.stdout.contains("demo::C::hidden"), "{run:#?}");
    let rdf = Rdf::of(
        &repo
            .run(&["--format", "turtle", "impact", "--base", "HEAD"])
            .expect(0),
    );
    let symbols = rdf.rows("SourceImpact")[0].children("symbol");
    let real = symbols
        .iter()
        .find(|s| s.required("path") == "demo::real")
        .expect("changed bound item");
    assert!(real.children("binding").iter().any(|b| {
        b.required("list") == "code"
            && b.required("path") == "demo::real"
            && b.children("section").iter().any(|s| {
                s.required("subject") == "cap:reader#contract"
                    && !s.children("assessment").is_empty()
            })
    }));
    let missing = repo
        .run(&["impact", "--base", "refs/heads/absent"])
        .expect(2);
    assert_eq!(missing.stdout, "");
    assert_eq!(repo.snapshot(), before);

    repo.edit(
        "docs/capabilities/reader.md",
        "[demo::real, demo::other]",
        "[demo::other]",
    );
    let withdrawn = repo.run(&["impact", "--base", "HEAD"]).expect(0);
    assert!(
        withdrawn
            .stdout
            .contains("changed demo::real\n  public item without a documented owner")
    );
    assert!(
        withdrawn
            .stdout
            .contains("cap:reader via code demo::real (base)")
    );
    assert!(
        withdrawn
            .stdout
            .contains("cap:reader via examples demo::real (checkout)")
    );
}
