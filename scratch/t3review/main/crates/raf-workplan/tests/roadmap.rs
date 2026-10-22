// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The roadmap: the four state groups in the order of the catalogue, the slug
//! order inside a group, the gaps of an implementing capability and the open
//! changes of each, over a fixture and over this repository.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

use raf_okf::State;
use raf_source::{DiskTree, MemoryTree, RelPath, Workspace};
use raf_workplan::{BindingKind, Gap, LinkContext, Roadmap, RoadmapEntry, Workplan};
use support::demo_workspace;

/// A capability document with the given state, bindings (the text after
/// `code:`, `tests:` and `examples:`) and nothing else of interest.
fn capability(slug: &str, state: &str, bindings: [&str; 3]) -> (String, String) {
    let [code, tests, examples] = bindings;
    let sections = [
        "Purpose",
        "Contract",
        "Usage",
        "Examples",
        "Constraints",
        "Explanation",
        "Verification",
    ]
    .map(|name| format!("## {name}\n\nText for {name}.\n"))
    .join("\n");
    (
        format!("docs/capabilities/{slug}.md"),
        format!(
            "---\ntype: Capability\nid: cap:{slug}\ntitle: \"Title of {slug}\"\n\
             description: What {slug} does.\nstate: {state}\n\
             bindings:\n  code: {code}\n  tests: {tests}\n  examples: {examples}\n\
             depends: []\n---\n\n# {slug}\n\n{sections}"
        ),
    )
}

fn change(slug: &str, changes: &str) -> (String, String) {
    (
        format!("work/changes/{slug}.md"),
        format!(
            "---\ntype: Change\nid: change:{slug}\ntitle: Change {slug}\n\
             description: Changes {slug}.\nchanges: {changes}\naffects: []\n\
             contract_effect: none\nitems: []\n---\n\n## Summary\n\nChanges it.\n"
        ),
    )
}

const NONE: [&str; 3] = ["[]", "[]", "[]"];

/// Capabilities given in a deliberately mixed insertion order, one per state
/// plus extras, and two change records.
fn files() -> Vec<(String, String)> {
    vec![
        capability("zeta", "delivered", ["[demo::real]", "[demo::nope]", "[]"]),
        capability(
            "mid",
            "implementing",
            ["[demo::nope, demo::real]", "[]", "[demo::real]"],
        ),
        capability("beta", "considered", NONE),
        capability("alpha", "designed", NONE),
        capability(
            "eta",
            "delivered",
            ["[demo::real]", "[demo::real]", "[demo::real]"],
        ),
        capability("gamma", "considered", NONE),
        capability("delta", "designed", NONE),
        capability("bare", "implementing", NONE),
        change("grow-mid", "[cap:mid, cap:beta]"),
        change("add-alpha", "[cap:alpha]"),
        change("also-mid", "[cap:mid]"),
    ]
}

fn fixture() -> MemoryTree {
    files().into_iter().collect()
}

fn slugs(entries: &[RoadmapEntry]) -> Vec<&str> {
    entries.iter().map(RoadmapEntry::slug).collect()
}

#[test]
fn the_groups_are_in_catalogue_order_and_sorted_by_slug() {
    let tree = fixture();
    let workspace = demo_workspace();
    let roadmap = Workplan::load(&tree, &workspace).status();
    assert_eq!(
        Roadmap::ORDER,
        [
            State::Delivered,
            State::Implementing,
            State::Designed,
            State::Considered
        ]
    );
    let groups = roadmap.groups();
    let states: Vec<State> = groups.iter().map(|(state, _)| *state).collect();
    assert_eq!(states, Roadmap::ORDER);
    assert_eq!(slugs(groups[0].1), ["eta", "zeta"]);
    assert_eq!(slugs(groups[1].1), ["bare", "mid"]);
    assert_eq!(slugs(groups[2].1), ["alpha", "delta"]);
    assert_eq!(slugs(groups[3].1), ["beta", "gamma"]);
    // The flat listing is the groups, concatenated.
    assert_eq!(
        slugs(roadmap.entries()),
        [
            "eta", "zeta", "bare", "mid", "alpha", "delta", "beta", "gamma"
        ]
    );
    assert_eq!(roadmap.len(), 8);
    assert!(!roadmap.is_empty());
    for (state, group) in groups {
        assert!(group.iter().all(|entry| entry.state() == state));
        assert_eq!(roadmap.group(state), group);
    }
}

#[test]
fn an_implementing_capability_lists_its_gaps_exactly() {
    let tree = fixture();
    let workspace = demo_workspace();
    let roadmap = Workplan::load(&tree, &workspace).status();
    let mid = roadmap.get("mid").expect("mid is in the roadmap");
    // An unresolved and a resolved code binding, an empty tests list, a
    // resolved example: the unresolved entry and the empty list, in list order.
    assert_eq!(
        mid.gaps(),
        [
            Gap::Unresolved {
                list: BindingKind::Code,
                binding: "demo::nope".to_string()
            },
            Gap::EmptyList(BindingKind::Test),
        ]
    );
    assert_eq!(mid.gaps()[0].binding(), Some("demo::nope"));
    assert_eq!(mid.gaps()[1].binding(), None);
    assert_eq!(mid.gaps()[0].list(), BindingKind::Code);
    assert_eq!(
        mid.gaps()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        [
            "bindings.code `demo::nope` does not resolve",
            "bindings.tests is empty"
        ]
    );
    let bare = roadmap.get("bare").expect("bare is in the roadmap");
    assert_eq!(
        bare.gaps(),
        [
            Gap::EmptyList(BindingKind::Code),
            Gap::EmptyList(BindingKind::Test),
            Gap::EmptyList(BindingKind::Example),
        ]
    );
    assert_eq!(
        bare.gaps()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        [
            "bindings.code is empty",
            "bindings.tests is empty",
            "bindings.examples is empty"
        ]
    );
}

#[test]
fn only_an_implementing_capability_has_gaps() {
    let tree = fixture();
    let workspace = demo_workspace();
    let roadmap = Workplan::load(&tree, &workspace).status();
    // `zeta` (delivered) has an unresolved tests entry and an empty examples
    // list, and a designed or considered capability has every list empty:
    // none of them is a gap outside `implementing`.
    for slug in ["zeta", "eta", "alpha", "delta", "beta", "gamma"] {
        let entry = roadmap.get(slug).expect("in the roadmap");
        assert!(entry.gaps().is_empty(), "{slug}: {:?}", entry.gaps());
    }
    assert!(roadmap.get("nope").is_none());
}

#[test]
fn a_fully_bound_implementing_capability_has_no_gap() {
    let tree: MemoryTree = [capability(
        "full",
        "implementing",
        ["[demo::real]", "[demo::real]", "[demo::real]"],
    )]
    .into_iter()
    .collect();
    let workspace = demo_workspace();
    let roadmap = Workplan::load(&tree, &workspace).status();
    assert_eq!(roadmap.get("full").expect("present").gaps(), []);
}

#[test]
fn open_changes_are_the_changes_that_list_the_capability() {
    let tree = fixture();
    let workspace = demo_workspace();
    let roadmap = Workplan::load(&tree, &workspace).status();
    let open =
        |slug: &str| -> Vec<String> { roadmap.get(slug).expect("present").open_changes().to_vec() };
    assert_eq!(open("mid"), ["change:also-mid", "change:grow-mid"]);
    assert_eq!(open("beta"), ["change:grow-mid"]);
    assert_eq!(open("alpha"), ["change:add-alpha"]);
    for slug in ["zeta", "eta", "gamma", "delta", "bare"] {
        assert!(open(slug).is_empty(), "{slug}");
    }
}

#[test]
fn open_change_slugs_are_the_open_changes_without_their_prefix() {
    let tree = fixture();
    let workspace = demo_workspace();
    let roadmap = Workplan::load(&tree, &workspace).status();
    let slugs = |slug: &str| -> Vec<String> {
        roadmap
            .get(slug)
            .expect("present")
            .open_change_slugs()
            .into_iter()
            .map(ToString::to_string)
            .collect()
    };
    assert_eq!(slugs("mid"), ["also-mid", "grow-mid"]);
    assert_eq!(slugs("alpha"), ["add-alpha"]);
    assert_eq!(slugs("zeta"), Vec::<String>::new());
}

#[test]
fn the_open_changes_are_sorted_by_slug_whatever_the_order_of_the_files() {
    let tree = fixture();
    let workspace = demo_workspace();
    let plan = Workplan::load(&tree, &workspace);
    let open = plan.open_changes();
    let shown: Vec<(&str, &str, &str, Vec<&str>)> = open
        .iter()
        .map(|change| {
            (
                change.slug(),
                change.id(),
                change.path(),
                change.capabilities().iter().map(String::as_str).collect(),
            )
        })
        .collect();
    assert_eq!(
        shown,
        [
            (
                "add-alpha",
                "change:add-alpha",
                "work/changes/add-alpha.md",
                vec!["cap:alpha"]
            ),
            (
                "also-mid",
                "change:also-mid",
                "work/changes/also-mid.md",
                vec!["cap:mid"]
            ),
            (
                "grow-mid",
                "change:grow-mid",
                "work/changes/grow-mid.md",
                // Sorted, not in the order the record lists them.
                vec!["cap:beta", "cap:mid"]
            ),
        ]
    );
    // A record that does not load is not an open change.
    let broken: MemoryTree = files()
        .into_iter()
        .chain([(
            "work/changes/broken.md".to_string(),
            "---\ntype: Change\n".to_string(),
        )])
        .collect();
    let plan = Workplan::load(&broken, &workspace);
    assert_eq!(plan.open_changes().len(), 3);
}

#[test]
fn an_entry_carries_what_the_catalogue_row_needs() {
    let tree = fixture();
    let workspace = demo_workspace();
    let roadmap = Workplan::load(&tree, &workspace).status();
    let entry = roadmap.get("mid").expect("present");
    assert_eq!(entry.id(), "cap:mid");
    assert_eq!(entry.slug(), "mid");
    assert_eq!(entry.title(), "Title of mid");
    assert_eq!(entry.description(), "What mid does.");
    assert_eq!(entry.state(), State::Implementing);
    assert_eq!(entry.path(), "docs/capabilities/mid.md");
    assert_eq!(entry.link_target(), "mid.md");
}

#[test]
fn planned_is_designed_then_considered_and_not_a_state() {
    let tree = fixture();
    let workspace = demo_workspace();
    let roadmap = Workplan::load(&tree, &workspace).status();
    let planned: Vec<&str> = roadmap.planned().iter().map(|e| e.slug()).collect();
    assert_eq!(planned, ["alpha", "delta", "beta", "gamma"]);
    assert!(
        roadmap
            .planned()
            .iter()
            .all(|e| matches!(e.state(), State::Designed | State::Considered))
    );
    // The groups stay four.
    assert_eq!(roadmap.groups().len(), 4);
}

#[test]
fn the_roadmap_does_not_depend_on_insertion_order_and_repeats() {
    let workspace = demo_workspace();
    let forward = fixture();
    let mut files = files();
    files.reverse();
    let backward: MemoryTree = files.into_iter().collect();
    let a = Workplan::load(&forward, &workspace).status();
    let b = Workplan::load(&backward, &workspace).status();
    assert_eq!(a, b);
    assert_eq!(a, Workplan::load(&forward, &workspace).status());
}

#[test]
fn slug_order_is_not_file_name_order() {
    // `a-b.md` sorts before `a.md`, so the loader yields `a-b` first.
    let tree: MemoryTree = [
        capability("a-b", "designed", NONE),
        capability("a", "designed", NONE),
    ]
    .into_iter()
    .collect();
    let workspace = demo_workspace();
    let plan = Workplan::load(&tree, &workspace);
    let loaded: Vec<&str> = plan
        .capabilities()
        .iter()
        .map(|c| c.value().header.slug.as_str())
        .collect();
    assert_eq!(loaded, ["a-b", "a"], "the loader's order");
    assert_eq!(slugs(plan.status().entries()), ["a", "a-b"]);
}

#[test]
fn a_resolving_binding_of_the_wrong_role_is_not_a_gap() {
    // `demo::bare` resolves but is an undocumented function, and `demo::real`
    // is not a test: `check` reports `binding-wrong-role`, the roadmap no gap.
    let tree: MemoryTree = [capability(
        "role",
        "implementing",
        ["[demo::real]", "[demo::real]", "[demo::bare]"],
    )]
    .into_iter()
    .collect();
    let workspace = demo_workspace();
    let plan = Workplan::load(&tree, &workspace);
    assert!(
        plan.check().iter().any(|f| f.kind == "binding-wrong-role"),
        "{:#?}",
        plan.check()
    );
    assert_eq!(plan.status().get("role").expect("present").gaps(), []);
}

#[test]
fn an_empty_repository_has_an_empty_roadmap() {
    let tree = MemoryTree::from([("README.md", "# Readme\n")]);
    let workspace = demo_workspace();
    let roadmap = Workplan::load(&tree, &workspace).status();
    assert!(roadmap.is_empty());
    assert_eq!(roadmap.len(), 0);
    assert_eq!(roadmap.planned().len(), 0);
    assert!(roadmap.groups().iter().all(|(_, group)| group.is_empty()));
}

// ---------------------------------------------------------------------------
// This repository
// ---------------------------------------------------------------------------

/// The directory the documents are read from: `RAF_GATE_ROOT` when set, the
/// repository root otherwise.
fn repo_root() -> PathBuf {
    let root = std::env::var_os("RAF_GATE_ROOT").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        PathBuf::from,
    );
    root.canonicalize()
        .unwrap_or_else(|e| panic!("cannot resolve the repository root {}: {e}", root.display()))
}

/// The tracked paths of the repository, as `git ls-files` lists them.
fn tracked() -> Vec<RelPath> {
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new("git")
        .arg("-C")
        .arg(&checkout)
        .args(["ls-files", "-z"])
        .output()
        .unwrap_or_else(|e| panic!("this test needs `git` to list tracked files: {e}"));
    assert!(
        output.status.success(),
        "`git ls-files -z` failed in {}: this test needs a git checkout",
        checkout.display()
    );
    let paths: Vec<RelPath> = output
        .stdout
        .split(|b| *b == 0)
        .filter(|name| !name.is_empty())
        .map(|name| {
            let text = std::str::from_utf8(name)
                .unwrap_or_else(|e| panic!("tracked path is not UTF-8: {e}"));
            RelPath::new(text).unwrap_or_else(|e| panic!("`{text}` is not a path: {e}"))
        })
        .collect();
    assert!(!paths.is_empty(), "git lists no tracked file");
    paths
}

/// The capability documents on disk, counted without the loader: the
/// Markdown files of `docs/capabilities` other than `index.md`.
fn capability_files(root: &Path) -> usize {
    let dir = root.join("docs/capabilities");
    std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot list {}: {e}", dir.display()))
        .map(|entry| entry.unwrap_or_else(|e| panic!("cannot read an entry: {e}")))
        .filter(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            name.ends_with(".md") && name != "index.md"
        })
        .count()
}

/// The capabilities this repository has built.
const BUILT: [&str; 12] = [
    "okf-profile",
    "source-index",
    "workplan-cli",
    "workplan-complete-gate",
    "workplan-current-rationale",
    "workplan-document-model",
    "workplan-entity-history",
    "workplan-evidence-claims",
    "workplan-generated-docs",
    "workplan-prose-assessment",
    "workplan-reconcile",
    "workplan-two-way-coverage",
];

/// Whether each of `slugs` is built in `roadmap`: delivered, or implementing
/// with an open change record named after it, which is how a delivered
/// capability is reopened; with no gap in either case. Every slug that does
/// not hold is named with the reason.
fn built_capabilities_hold(roadmap: &Roadmap, slugs: &[&str]) -> Result<(), Vec<String>> {
    let mut problems = Vec::new();
    for slug in slugs {
        let Some(entry) = roadmap.get(slug) else {
            problems.push(format!("{slug} is not in the roadmap"));
            continue;
        };
        match entry.state() {
            State::Delivered => {}
            State::Implementing => {
                if !entry.open_change_slugs().contains(slug) {
                    problems.push(format!(
                        "{slug} is implementing with no open change named after it (open: {:?})",
                        entry.open_changes()
                    ));
                }
            }
            other => problems.push(format!(
                "{slug} is {other:?}, neither delivered nor reopened"
            )),
        }
        if !entry.gaps().is_empty() {
            problems.push(format!("{slug} has gaps: {:?}", entry.gaps()));
        }
        if !roadmap
            .group(entry.state())
            .iter()
            .any(|e| e.slug() == *slug)
        {
            problems.push(format!("{slug} is not in the group of its state"));
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

/// A roadmap of the one fully bound capability `built` in `state`, with the
/// change records `changes` (slug, the capabilities it lists).
fn built_roadmap(state: &str, bindings: [&str; 3], changes: &[(&str, &str)]) -> Roadmap {
    let tree: MemoryTree = std::iter::once(capability("built", state, bindings))
        .chain(changes.iter().map(|(slug, lists)| change(slug, lists)))
        .collect();
    let workspace = demo_workspace();
    Workplan::load(&tree, &workspace).status()
}

const BOUND: [&str; 3] = ["[demo::real]", "[demo::real]", "[demo::real]"];

#[test]
fn a_built_capability_is_delivered_or_reopened_under_its_own_change_and_has_no_gap() {
    // Delivered, with no record: holds.
    let delivered = built_roadmap("delivered", BOUND, &[]);
    assert_eq!(built_capabilities_hold(&delivered, &["built"]), Ok(()));
    // Reopened with no record: refused.
    let unrecorded = built_roadmap("implementing", BOUND, &[]);
    let problems = built_capabilities_hold(&unrecorded, &["built"]).expect_err("no record");
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].starts_with("built is implementing with no open change named after it"),
        "{problems:?}"
    );
    // Reopened under a record of another name only: refused.
    let other = built_roadmap("implementing", BOUND, &[("grow-built", "[cap:built]")]);
    let problems = built_capabilities_hold(&other, &["built"]).expect_err("not its own record");
    assert!(
        problems[0].contains("no open change named after it"),
        "{problems:?}"
    );
    // Reopened under its own record: holds, beside another record.
    let reopened = built_roadmap(
        "implementing",
        BOUND,
        &[("built", "[cap:built]"), ("grow-built", "[cap:built]")],
    );
    assert_eq!(built_capabilities_hold(&reopened, &["built"]), Ok(()));
    // Reopened under its own record but with a gap: refused for the gap.
    let gap = built_roadmap(
        "implementing",
        ["[demo::real]", "[]", "[demo::real]"],
        &[("built", "[cap:built]")],
    );
    let problems = built_capabilities_hold(&gap, &["built"]).expect_err("a gap");
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].starts_with("built has gaps"), "{problems:?}");
    // Designed, and absent: refused.
    let designed = built_roadmap("designed", ["[]", "[]", "[]"], &[("built", "[cap:built]")]);
    let problems = built_capabilities_hold(&designed, &["built"]).expect_err("not built");
    assert_eq!(
        problems,
        ["built is Designed, neither delivered nor reopened"]
    );
    let problems = built_capabilities_hold(&delivered, &["missing"]).expect_err("absent");
    assert_eq!(problems, ["missing is not in the roadmap"]);
}

#[test]
fn this_repository_lists_every_capability_once_in_its_state_group() {
    let root = repo_root();
    let tree = DiskTree::new(&root).unwrap_or_else(|e| panic!("cannot open the tree: {e}"));
    let workspace = Workspace::load(&tree, &["crates"])
        .unwrap_or_else(|e| panic!("the workspace does not load: {e}"));
    let links = LinkContext::new(&tree, tracked());
    let plan = Workplan::load_with(&tree, &workspace, &links);
    let roadmap = plan.status();

    let on_disk = capability_files(&root);
    assert!(
        on_disk > 0,
        "no capability document under {}",
        root.display()
    );
    assert_eq!(roadmap.len(), on_disk, "the roadmap lists every document");
    // Each document appears once, in the group of its own state.
    let mut seen: Vec<&str> = roadmap.entries().iter().map(RoadmapEntry::slug).collect();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), on_disk, "a slug is listed twice");
    for loaded in plan.capabilities() {
        let header = &loaded.value().header;
        let entry = roadmap
            .get(&header.slug)
            .unwrap_or_else(|| panic!("{} is not in the roadmap", header.id));
        assert_eq!(entry.state(), loaded.value().state, "{}", header.id);
        assert_eq!(entry.path(), loaded.path());
        assert_eq!(entry.title(), header.title);
        assert_eq!(entry.description(), header.description);
        let group = roadmap.group(entry.state());
        assert_eq!(
            group.iter().filter(|e| e.slug() == header.slug).count(),
            1,
            "{}",
            header.id
        );
        if entry.state() != State::Implementing {
            assert!(entry.gaps().is_empty(), "{}", header.id);
        }
    }
    // The load found the built capabilities: each capability this repository
    // has built is delivered, or reopened under its own open change, and has
    // no gap. The census capabilities are listed too, in whatever group their
    // documents declare (checked above); the rest is planned work.
    if let Err(problems) = built_capabilities_hold(&roadmap, &BUILT) {
        panic!("{}", problems.join("\n"));
    }
    assert_ne!(
        roadmap.group(State::Delivered).len() + roadmap.group(State::Implementing).len(),
        0,
        "nothing is built"
    );
    for slug in [
        "census-cli",
        "census-debt-ratchet",
        "census-detectors",
        "census-ledger",
        "census-migration",
        "census-normal-form",
        "census-patterns",
        "census-reference-scan",
        "census-rule-api",
    ] {
        assert!(roadmap.get(slug).is_some(), "{slug} is not in the roadmap");
    }
    assert_eq!(
        roadmap.planned().len(),
        roadmap.group(State::Designed).len() + roadmap.group(State::Considered).len()
    );
    assert_eq!(
        roadmap.group(State::Delivered).len()
            + roadmap.group(State::Implementing).len()
            + roadmap.planned().len(),
        on_disk
    );
    for (_, group) in roadmap.groups() {
        assert!(group.windows(2).all(|w| w[0].slug() < w[1].slug()));
    }
}
