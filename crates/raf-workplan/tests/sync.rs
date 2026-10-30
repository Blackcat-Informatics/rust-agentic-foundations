// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Sync and write: idempotence, determinism, agreement with `check`, writing
//! to a disk, the `docs_sync` example, and the catalogue against the roadmap.

#[path = "../examples/component_model.rs"]
mod component_model;
mod support;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Mutex, MutexGuard, PoisonError};

use raf_source::{DiskTree, RelPath, Workspace};
use raf_workplan::diagram::{DiagramKind, DiagramScope, NamedDiagramView};
use raf_workplan::{Finding, LinkContext, SyncError, SyncReport, Workplan};
use support::cases::all_cases;
use support::synced::{
    ALPHA, BETA, DELTA, GAMMA, INDEX, apply, base, check_all, generated_findings, options, sync,
    synced, unsynced, with_plan,
};
use support::{AUXILIARY, Repo, demo_files};

// --- in memory ---------------------------------------------------------------

#[test]
fn syncing_twice_writes_nothing_the_second_time_and_the_bytes_are_equal() {
    let repo = unsynced();
    let first = sync(&repo).expect("syncs");
    assert!(!first.is_empty());
    let once = apply(&repo, &first);
    let second = sync(&once).expect("syncs again");
    assert!(second.is_empty(), "{second:#?}");
    assert_eq!(apply(&once, &second), once);
    // And a third time, from the synced result of the second.
    assert!(sync(&synced(&once)).expect("syncs").is_empty());
    let scratch = Scratch::new("named-diagram-publication");
    let root = scratch.path();
    write_repo(root, &base());
    let original = read(root, ALPHA);
    let declaration = "diagrams:\n  - name: dependencies\n    kind: capability-dependencies\n    scope: workspace\n    grouping: capability\n    max_statements: 2000\n    max_terms: 3000\n";
    let declared = format!(
        "{}\n<!-- raf:generated:diagram-dependencies -->\n<!-- raf:generated:end -->\n",
        original.replacen("bindings:", &format!("{declaration}bindings:"), 1)
    );
    fs::write(root.join(ALPHA), &declared).unwrap();
    fs::create_dir_all(root.join("docs/diagrams")).unwrap();
    fs::write(
        root.join("docs/diagrams/authored.svg"),
        "unrelated authored bytes",
    )
    .unwrap();
    let before = support::disk::snapshot(root);
    let report = on_disk(root, |plan| {
        let view = plan.named_diagram("cap:alpha#dependencies").unwrap();
        assert!(!view.details().is_empty());
        assert!(view.caption().contains("capability-dependencies"));
        assert!(view.text().contains("alpha"));
        assert!(
            plan.check()
                .iter()
                .any(|finding| finding.kind == "diagram-stale")
        );
        plan.sync(&options()).unwrap()
    });
    assert_eq!(
        support::disk::snapshot(root),
        before,
        "preview computes without writes"
    );
    assert!(
        report
            .edits
            .iter()
            .any(|(path, _)| path.as_str().ends_with("diagram.svg"))
    );
    Workplan::write(&report, root).unwrap();
    assert!(on_disk(root, |plan| plan
        .sync(&options())
        .unwrap()
        .is_empty()));
    grouped_detail_names_cannot_alias_declared_views(root);
    assert_eq!(
        read(root, "docs/diagrams/authored.svg"),
        "unrelated authored bytes"
    );
    assert!(read(root, ALPHA).contains("Text alternative"));
    let owned: Vec<_> = report
        .edits
        .iter()
        .filter(|(path, _)| {
            path.as_str().starts_with("docs/diagrams/") && !path.as_str().ends_with("manifest.ttl")
        })
        .map(|(path, _)| path.clone())
        .collect();
    let forged = SyncReport {
        edits: vec![(owned[0].clone(), "forged".into())],
        diagram_assets: None,
    };
    assert!(Workplan::write(&forged, root).is_err());
    let kept = support::disk::snapshot(root);
    fs::write(
        root.join(ALPHA),
        declared.replace("kind: capability-dependencies", "kind: invented"),
    )
    .unwrap();
    assert!(on_disk(root, |plan| plan.sync(&options()).is_err()));
    assert!(
        owned
            .iter()
            .all(|path| fs::read(root.join(path.as_str())).unwrap() == kept[path.as_str()])
    );
    fs::write(root.join(ALPHA), &original).unwrap();
    let removal = on_disk(root, |plan| plan.sync(&options()).unwrap());
    assert_eq!(removal.removals().count(), owned.len());
    Workplan::write(&removal, root).unwrap();
    assert!(owned.iter().all(|path| !root.join(path.as_str()).exists()));
    assert_eq!(
        read(root, "docs/diagrams/authored.svg"),
        "unrelated authored bytes"
    );
    assert!(on_disk(root, |plan| plan
        .sync(&options())
        .unwrap()
        .is_empty()));
}

fn grouped_detail_names_cannot_alias_declared_views(root: &Path) {
    let original = read(root, ALPHA);
    let before = support::disk::snapshot(root);
    let detail = on_disk(root, |plan| {
        plan.named_diagram("cap:alpha#dependencies")
            .unwrap()
            .details()[0]
            .view()
            .name()
            .to_owned()
    });
    let additional = format!(
        "  - name: {detail}\n    kind: capability-dependencies\n    scope: workspace\nbindings:"
    );
    let colliding = format!(
        "{}\n<!-- raf:generated:diagram-{detail} -->\n<!-- raf:generated:end -->\n",
        original.replacen("bindings:", &additional, 1)
    );
    fs::write(root.join(ALPHA), &colliding).unwrap();
    let unchanged = support::disk::snapshot(root);
    on_disk(root, |plan| {
        assert!(
            plan.check()
                .iter()
                .any(|finding| finding.kind == "diagram-invalid"
                    && finding.message.contains("produced by multiple views"))
        );
        assert!(plan.sync(&options()).is_err());
    });
    assert_eq!(support::disk::snapshot(root), unchanged);
    let renamed = format!("{detail}-separate");
    fs::write(root.join(ALPHA), colliding.replace(&detail, &renamed)).unwrap();
    let accepted = on_disk(root, |plan| plan.sync(&options()).unwrap());
    Workplan::write(&accepted, root).unwrap();
    assert!(on_disk(root, |plan| plan
        .sync(&options())
        .unwrap()
        .is_empty()));
    fs::write(root.join(ALPHA), original).unwrap();
    let restored = on_disk(root, |plan| plan.sync(&options()).unwrap());
    Workplan::write(&restored, root).unwrap();
    assert_eq!(support::disk::snapshot(root), before);
}

#[test]
fn the_report_is_a_pure_function_of_the_documents_and_is_sorted_by_path() {
    authored_model_neighbors();
    let repo = unsynced();
    let one = sync(&repo).expect("syncs");
    let two = sync(&repo).expect("syncs");
    assert_eq!(one, two);
    let paths: Vec<&str> = one.edits.iter().map(|(p, _)| p.as_str()).collect();
    assert_eq!(paths, [ALPHA, AUXILIARY, BETA, GAMMA, INDEX]);
    let scratch = Scratch::new("automatic-semantic-views");
    let root = scratch.path();
    write_repo(root, &base());
    let manifest = format!(
        "{}\n[workspace]\nmembers = [\"other\"]\n[dependencies]\nother = {{ path = \"other\", optional = true, default-features = false, features = [\"fine\"] }}\n[build-dependencies]\nbuild_alias = {{package=\"other\",path=\"other\"}}\n[target.'cfg(unix)'.dev-dependencies]\ndev_alias = {{package=\"other\",path=\"other\"}}\n",
        read(root, "Cargo.toml")
    );
    fs::write(root.join("Cargo.toml"), &manifest).unwrap();
    fs::create_dir_all(root.join("other/src")).unwrap();
    fs::write(
        root.join("other/Cargo.toml"),
        "[package]\nname=\"other\"\nversion=\"0.1.0\"\n[features]\nfine=[]\n",
    )
    .unwrap();
    fs::write(root.join("other/src/lib.rs"), "pub fn other() {}\n").unwrap();
    let before = support::disk::snapshot(root);
    for kind in DiagramKind::ALL.into_iter().filter(|kind| {
        !matches!(
            kind,
            DiagramKind::EvidenceClaims
                | DiagramKind::Components
                | DiagramKind::Scenario
                | DiagramKind::Artifacts
                | DiagramKind::ModelProvenance
        )
    }) {
        let view = NamedDiagramView::new(
            "automatic",
            DiagramScope::Automatic {
                kind,
                scope: "workspace".into(),
            },
        )
        .unwrap();
        let (svg, json, text) = on_disk(root, |plan| {
            let one = plan.diagram(&view).unwrap();
            let two = plan.diagram(&view).unwrap();
            assert_eq!(one.svg(), two.svg());
            assert_eq!(one.json(), two.json());
            assert_eq!(one.text(), two.text());
            if kind == DiagramKind::WorkspaceDependencies {
                for label in [
                    "normalDependency",
                    "buildDependency",
                    "devDependency",
                    "targetCondition",
                    "requestedFeature",
                    "unevaluated",
                ] {
                    assert!(one.text().contains(label), "{label}: {}", one.text());
                }
            }
            let invalid = NamedDiagramView::new(
                "invalid",
                DiagramScope::Automatic {
                    kind,
                    scope: "absent".into(),
                },
            )
            .unwrap();
            assert!(plan.diagram(&invalid).is_err());
            (
                one.svg().to_owned(),
                one.json().to_owned(),
                one.text().to_owned(),
            )
        });
        assert!(!svg.is_empty() && !json.is_empty() && !text.is_empty());
        if kind == DiagramKind::WorkspaceDependencies {
            let reordered = manifest.replace(
                "path = \"other\", optional = true, default-features = false",
                "default-features = false, optional = true, path = \"other\"",
            );
            assert_ne!(reordered, manifest);
            fs::write(root.join("Cargo.toml"), reordered).unwrap();
            on_disk(root, |plan| {
                let reordered = plan.diagram(&view).unwrap();
                assert_eq!(reordered.svg(), svg);
                assert_eq!(reordered.json(), json);
                assert_eq!(reordered.text(), text);
            });
            fs::write(root.join("Cargo.toml"), &manifest).unwrap();
        }
    }
    assert_eq!(support::disk::snapshot(root), before);
    coverage_currency_preserves_query_refusals();
}

/// The evidence input is not a capability binding: loading the document succeeds
/// before the diagram queries the globbed input under the caller's allowance.
fn coverage_currency_preserves_query_refusals() {
    use raf_source::{Budget, LoadOptions, MemoryTree, ResolveError};
    use raf_workplan::digest::{Digest, SymbolDigestError};
    use std::error::Error as _;

    let tree = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        ),
        (
            "src/lib.rs",
            "pub mod carrier { pub fn exported() {} }\n\
             pub mod child { pub use crate::carrier::*; }\n",
        ),
    ]);
    let source = Workspace::load(&tree, &["."]).expect("source loads");
    let input = "demo::child::exported";
    let evidence = support::evidence::Evidence::passing("coverage", "[cap:omega]").inputs(vec![(
        input.into(),
        Digest::symbol(&source, input)
            .expect("globbed input resolves")
            .to_string(),
    )]);
    let capability = support::synced::document(
        "omega",
        "Omega.",
        "considered",
        ["[]", "[]", "[]"],
        "[]",
        &[],
    );
    let documents = MemoryTree::from([
        ("docs/capabilities/omega.md", capability.as_str()),
        ("work/evidence/coverage.md", evidence.text().as_str()),
    ]);
    let view = NamedDiagramView::new(
        "coverage",
        DiagramScope::Automatic {
            kind: DiagramKind::Coverage,
            scope: "cap:omega".into(),
        },
    )
    .expect("coverage selection");
    Workplan::load(&documents, &source)
        .diagram(&view)
        .expect("current evidence renders with the normal query allowance");

    let mut options = LoadOptions::default();
    options.max_query_work = 0;
    let limited = Workspace::load_with(&tree, &["."], &options)
        .expect("the query limit does not prevent loading");
    let error = Workplan::load(&documents, &limited)
        .diagram(&view)
        .expect_err("refused evidence evaluation is not stale evidence");
    let cause = error
        .source()
        .and_then(|error| error.downcast_ref::<SymbolDigestError>());
    assert!(
        matches!(
            cause,
            Some(SymbolDigestError::Resolve {
                path,
                error: ResolveError::Budget {
                    budget: Budget::Work,
                    limit: 0,
                    ..
                },
            }) if path == input
        ),
        "{error:?}"
    );
}

#[test]
fn nothing_is_in_the_report_for_a_repository_that_is_in_sync() {
    assert!(sync(&base()).expect("syncs").is_empty());
    assert_eq!(SyncReport::default(), sync(&base()).expect("syncs"));
    let scratch = Scratch::new("architecture-diagram-reference");
    let root = scratch.path();
    write_repo(root, &base());
    let page = support::synced::OVERVIEW;
    let original = read(root, page);
    let declaration = "diagrams:\n  - name: structure\n    kind: ownership\n    scope: workspace\n";
    let declared = format!(
        "{}\n<!-- raf:generated:diagram-structure -->\n<!-- raf:generated:end -->\n",
        original.replacen("\n---\n", &format!("\n{declaration}---\n"), 1)
    );
    fs::write(root.join(page), &declared).unwrap();
    on_disk(root, |plan| {
        assert!(plan.named_diagram("arch:overview#structure").is_ok());
        assert!(plan.named_diagram("arch:overview#missing").is_err());
    });
    for invalid in [
        declared.replace("scope: workspace", "scope: workspace\n    invented: true"),
        declared.replace("kind: ownership", "kind: ownership\n    max_terms: 0"),
        declared.replace("scope: workspace", "scope: workspace\n    scope: other"),
    ] {
        fs::write(root.join(page), invalid).unwrap();
        assert!(on_disk(root, |plan| plan.sync(&options()).is_err()));
    }
    fs::write(root.join(page), &declared).unwrap();
    for invalid in ["2026 a--b", "2026 bad\u{1b}"] {
        assert!(on_disk(root, |plan| plan
            .sync(&raf_workplan::SyncOptions::new(invalid, "CC-BY-4.0"))
            .is_err()));
    }
    let report = on_disk(root, |plan| plan.sync(&options()).unwrap());
    let collision = report
        .edits
        .iter()
        .find(|(path, _)| path.as_str().ends_with("diagram.svg"))
        .unwrap()
        .0
        .clone();
    fs::create_dir_all(root.join(collision.as_str()).parent().unwrap()).unwrap();
    fs::write(root.join(collision.as_str()), "authored collision").unwrap();
    assert!(on_disk(root, |plan| plan.sync(&options()).is_err()));
    assert_eq!(read(root, collision.as_str()), "authored collision");
    fs::remove_file(root.join(collision.as_str())).unwrap();
    Workplan::write(&report, root).unwrap();
    assert!(read(root, page).contains("![structure:"));
    assert!(on_disk(root, |plan| plan
        .sync(&options())
        .unwrap()
        .is_empty()));
    let view = NamedDiagramView::new(
        "focused",
        DiagramScope::Automatic {
            kind: DiagramKind::Ownership,
            scope: "workspace".into(),
        },
    )
    .unwrap();
    on_disk(root, |plan| {
        assert!(
            plan.diagram(&view.clone().with_focus(vec!["absent".into()]))
                .is_err()
        );
        assert!(
            plan.diagram(&view.clone().with_relations(vec!["calls".into()]))
                .is_err()
        );
        assert!(
            plan.diagram(
                &view
                    .clone()
                    .with_limits(std::num::NonZeroUsize::MIN, std::num::NonZeroUsize::MIN)
            )
            .is_err()
        );
        assert!(plan.diagram(&view).is_ok());
    });
    let source = read(root, "src/lib.rs");
    fs::write(
        root.join("src/lib.rs"),
        source.replace("pub fn real()", "pub fn renamed()"),
    )
    .unwrap();
    assert!(
        on_disk(root, |plan| plan.diagram(&view).is_err()),
        "a renamed bound input must not yield an apparently complete ownership view"
    );
    fs::write(root.join("src/lib.rs"), source).unwrap();
    assert!(on_disk(root, |plan| plan.diagram(&view).is_ok()));
}

#[test]
fn a_document_that_does_not_load_stops_the_sync_since_the_tables_would_miss_it() {
    for (path, from, to, unloaded) in [
        (BETA, "state: implementing", "state: finished", "cap:beta"),
        (DELTA, "type: Capability", "type: Capabilty", "cap:delta"),
        (
            "work/changes/grow.md",
            "type: Change",
            "type: Chnage",
            "change:grow",
        ),
    ] {
        let mut repo = base();
        let text = repo.get_mut(path).expect("the file");
        *text = text.replacen(from, to, 1);
        match sync(&repo) {
            Err(SyncError::Unloaded(ids)) => assert_eq!(ids, [unloaded]),
            other => panic!("{path}: {other:?}"),
        }
        assert!(
            !check_all(&repo).is_empty(),
            "the document's own findings are in check"
        );
    }
}

/// The kinds that say the generated content of a repository is wrong, and the
/// reader errors of an index that is not one.
fn is_generated_kind(finding: &Finding) -> bool {
    finding.kind.starts_with("okf-parse-index-")
        || [
            "diagram-invalid",
            "diagram-stale",
            "index-missing",
            "generated-stale",
            "generated-unknown-kind",
            "generated-misplaced",
            "index-hand-written",
            "mixed-line-endings",
        ]
        .contains(&finding.kind)
}

/// Whether `check` has no finding about generated content.
fn generated_clean(repo: &Repo) -> bool {
    !check_all(repo).iter().any(is_generated_kind)
}

#[test]
fn generated_checks_are_clean_exactly_when_sync_has_nothing_to_write() {
    let mut repos: Vec<(String, Repo)> = vec![
        ("synced base".to_string(), base()),
        ("unsynced".to_string(), unsynced()),
    ];
    repos.extend(
        support::cases::generated_cases()
            .into_iter()
            .map(|case| (case.name, case.repo)),
    );
    repos.extend(all_cases().into_iter().map(|case| (case.name, case.repo)));
    let mut compared = 0;
    let mut clean = 0;
    for (name, repo) in &repos {
        let outcome = sync(repo);
        if matches!(outcome, Err(SyncError::Unloaded(_))) {
            // A document that does not read is the finding of its own rule.
            continue;
        }
        compared += 1;
        let nothing = matches!(&outcome, Ok(report) if report.is_empty());
        assert_eq!(
            generated_clean(repo),
            nothing,
            "{name}: check says {:#?}, sync says {outcome:?}",
            generated_findings(repo)
        );
        clean += usize::from(nothing);
    }
    assert!(compared > 30, "only {compared} fixtures compared");
    assert!(clean >= 1, "the synced base is among the clean ones");
}

#[test]
fn check_and_sync_agree_after_each_repair() {
    let mut repo = unsynced();
    for _ in 0..2 {
        let report = sync(&repo).expect("syncs");
        repo = apply(&repo, &report);
        assert!(generated_clean(&repo));
    }
}

// --- on a disk ---------------------------------------------------------------

/// A unique directory under the system temporary directory, removed on drop.
struct Scratch(purrdf_testkit::TempDir);

impl Scratch {
    fn new(tag: &str) -> Self {
        Self(
            purrdf_testkit::TempDir::new_in(std::env::temp_dir())
                .unwrap_or_else(|error| panic!("create {tag} fixture: {error}")),
        )
    }

    fn path(&self) -> &Path {
        self.0.path()
    }
}

/// Write `repo` and the demo workspace under `root`.
fn write_repo(root: &Path, repo: &Repo) {
    let put = |path: &str, text: &str| {
        let target = root.join(path);
        fs::create_dir_all(target.parent().expect("a parent"))
            .unwrap_or_else(|e| panic!("cannot create the directory of {path}: {e}"));
        fs::write(&target, text).unwrap_or_else(|e| panic!("cannot write {path}: {e}"));
    };
    for (path, text) in demo_files() {
        put(path, text);
    }
    for (path, text) in repo {
        put(path, text);
    }
}

fn read(root: &Path, path: &str) -> String {
    fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("cannot read {path}: {e}"))
}

/// Load the documents under `root` from the disk and run `f` on the plan.
fn on_disk<R>(root: &Path, f: impl FnOnce(&Workplan<'_>) -> R) -> R {
    let tree =
        DiskTree::new(root).unwrap_or_else(|e| panic!("cannot open {}: {e}", root.display()));
    let source = Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("workspace: {e}"));
    f(&Workplan::load_with(
        &tree,
        &source,
        &LinkContext::of_tree(&tree),
    ))
}

/// The findings of `check` of a plan.
fn checked(plan: &Workplan<'_>) -> Vec<Finding> {
    plan.check()
}

fn rel(path: &str) -> RelPath {
    RelPath::new(path).unwrap_or_else(|e| panic!("`{path}` is not a path: {e}"))
}

#[test]
fn write_puts_every_edit_on_disk_and_a_second_run_writes_nothing() {
    let scratch = Scratch::new("write");
    write_repo(scratch.path(), &unsynced());
    let report = on_disk(scratch.path(), |plan| plan.sync(&options())).expect("syncs");
    assert_eq!(report.edits.len(), 5);
    Workplan::write(&report, scratch.path()).expect("writes");
    let expected = base();
    for path in [ALPHA, AUXILIARY, BETA, GAMMA, INDEX] {
        assert_eq!(read(scratch.path(), path), expected[path], "{path}");
    }
    // The untouched files are as they were, and no temporary file is left.
    assert_eq!(read(scratch.path(), DELTA), unsynced()[DELTA]);
    let leftovers: Vec<String> = fs::read_dir(scratch.path().join("docs/capabilities"))
        .expect("lists")
        .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
        .filter(|name| name.contains("tmp"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
    // Second run: nothing to write, and writing the empty report changes nothing.
    let second = on_disk(scratch.path(), |plan| plan.sync(&options())).expect("syncs");
    assert!(second.is_empty());
    Workplan::write(&second, scratch.path()).expect("writes nothing");
    assert_eq!(read(scratch.path(), INDEX), expected[INDEX]);
    assert!(
        on_disk(scratch.path(), checked)
            .iter()
            .all(|f| !is_generated_kind(f))
    );
}

#[test]
fn write_publishes_the_computed_snapshot_without_comparing_later_authored_edits() {
    for edited in [false, true] {
        let scratch = Scratch::new("snapshot-publication");
        write_repo(scratch.path(), &unsynced());
        let report = on_disk(scratch.path(), |plan| plan.sync(&options())).expect("syncs");
        let snapshot = report
            .edits
            .iter()
            .find(|(path, _)| path.as_str() == ALPHA)
            .expect("the actual report rewrites alpha")
            .1
            .clone();
        if edited {
            let path = scratch.path().join(ALPHA);
            let later = format!(
                "{}\nNewer authored prose after report computation.\n",
                read(scratch.path(), ALPHA)
            );
            fs::write(&path, &later).expect("writes the newer authored text");
            assert_ne!(later, snapshot);
            assert!(read(scratch.path(), ALPHA).contains("Newer authored prose"));
            let newer_report = on_disk(scratch.path(), |plan| plan.sync(&options()))
                .expect("the newer authored document still loads and syncs");
            assert!(newer_report.edits.iter().any(|(path, text)| {
                path.as_str() == ALPHA && text.contains("Newer authored prose")
            }));
        }
        Workplan::write(&report, scratch.path()).expect("publishes the supplied snapshot");
        assert_eq!(read(scratch.path(), ALPHA), snapshot, "edited={edited}");
        assert!(!read(scratch.path(), ALPHA).contains("Newer authored prose"));
        assert!(
            on_disk(scratch.path(), |plan| plan.sync(&options()))
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn write_refuses_a_target_that_is_not_an_existing_regular_file_or_the_new_index() {
    let scratch = Scratch::new("refuse");
    write_repo(scratch.path(), &unsynced());
    let root = scratch.path();
    let before = read(root, ALPHA);
    let good = (rel(ALPHA), "rewritten\n".to_string());
    let refused = |bad: &str| {
        let report = SyncReport {
            diagram_assets: None,
            // A valid edit first: it must not be written when a later one is refused.
            edits: vec![good.clone(), (rel(bad), "x\n".to_string())],
        };
        let error = Workplan::write(&report, root).expect_err(bad);
        assert!(
            error.to_string().contains(bad) || bad.contains("outside"),
            "{bad}: {error}"
        );
        assert_eq!(read(root, ALPHA), before, "{bad}: nothing is written");
    };
    // Missing file other than the index.
    refused("docs/capabilities/ghost.md");
    assert!(!root.join("docs/capabilities/ghost.md").exists());
    // A directory.
    refused("docs/capabilities");
    // The index in a directory that does not exist.
    fs::remove_dir_all(root.join("docs/architecture")).expect("removes");
    refused("docs/architecture/index.md");
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        // A symbolic link to a file inside the root.
        symlink(root.join(BETA), root.join("docs/capabilities/link.md")).expect("links");
        refused("docs/capabilities/link.md");
        assert!(
            read(root, BETA).contains("# beta"),
            "the link target is untouched"
        );
        // A directory link that leaves the root.
        let outside = Scratch::new("outside");
        symlink(outside.path(), root.join("docs/capabilities/outside")).expect("links");
        refused("docs/capabilities/outside/x.md");
        assert_eq!(fs::read_dir(outside.path()).expect("lists").count(), 0);
    }
    // The same path twice.
    let twice = SyncReport {
        diagram_assets: None,
        edits: vec![good.clone(), good],
    };
    assert!(Workplan::write(&twice, root).is_err());
    assert_eq!(read(root, ALPHA), before);
}

/// A directory link inside the root on the way to a target, at `docs` or at
/// `docs/capabilities`, is refused before anything is written, though the
/// file behind it is a regular file; the neighbour with real directories
/// writes the same report.
#[cfg(unix)]
#[test]
fn write_refuses_a_target_behind_a_directory_link_inside_the_root() {
    for link in ["docs", "docs/capabilities"] {
        let scratch = Scratch::new("in-root-link");
        let root = scratch.path();
        write_repo(root, &unsynced());
        let report = on_disk(root, |plan| plan.sync(&options())).expect("syncs");
        assert!(!report.is_empty());
        fs::rename(root.join(link), root.join("moved")).expect("moves");
        let up = "../".repeat(link.matches('/').count());
        std::os::unix::fs::symlink(format!("{up}moved"), root.join(link)).expect("links");
        let before = support::disk::snapshot(root);
        let error = Workplan::write(&report, root).expect_err(link);
        assert!(
            error
                .to_string()
                .contains(&format!("the directory {link} is refused")),
            "{link}: {error}"
        );
        assert_eq!(
            support::disk::snapshot(root),
            before,
            "{link}: nothing is written"
        );
        // The neighbour: the directory is real again.
        fs::remove_file(root.join(link)).expect("unlinks");
        fs::rename(root.join("moved"), root.join(link)).expect("moves back");
        Workplan::write(&report, root).expect("writes");
        assert_eq!(read(root, ALPHA), base()[ALPHA]);
    }
}

#[cfg(unix)]
#[test]
fn write_keeps_the_permissions_of_the_file_it_replaces() {
    use std::os::unix::fs::PermissionsExt;
    let scratch = Scratch::new("mode");
    write_repo(scratch.path(), &unsynced());
    let target = scratch.path().join(ALPHA);
    fs::set_permissions(&target, fs::Permissions::from_mode(0o640)).expect("chmod");
    let report = on_disk(scratch.path(), |plan| plan.sync(&options())).expect("syncs");
    Workplan::write(&report, scratch.path()).expect("writes");
    let mode = fs::metadata(&target).expect("stat").permissions().mode() & 0o777;
    assert_eq!(mode, 0o640);
}

#[test]
fn write_creates_the_index_only_where_the_directory_exists() {
    let scratch = Scratch::new("index");
    write_repo(scratch.path(), &unsynced());
    assert!(!scratch.path().join(INDEX).exists());
    let report = SyncReport {
        diagram_assets: None,
        edits: vec![(rel(INDEX), "created\n".to_string())],
    };
    Workplan::write(&report, scratch.path()).expect("creates the index");
    assert_eq!(read(scratch.path(), INDEX), "created\n");
}

// --- the example ---------------------------------------------------------------

/// The tests that run the `docs_sync` example each start `cargo run`, which may
/// build the example and then starts it, while the others do the same: an
/// executable that one `cargo` is still writing when another starts it makes
/// the start fail with "text file busy". Every test that runs the example holds
/// this lock from its first process to its last, as the other process tests of
/// the crate hold theirs.
static EXAMPLE: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    EXAMPLE.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Run the `docs_sync` example with `args`, from the crate directory.
fn docs_sync(args: &[&str]) -> Output {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut command = Command::new(cargo);
    command.current_dir(env!("CARGO_MANIFEST_DIR")).args([
        "run",
        "--quiet",
        "--locked",
        "-p",
        "raf-workplan",
        "--example",
        "docs_sync",
    ]);
    if !cfg!(debug_assertions) {
        command.arg("--release");
    }
    command.arg("--").args(args);
    command
        .output()
        .unwrap_or_else(|e| panic!("cannot run the docs_sync example: {e}"))
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn the_docs_sync_example_writes_the_expected_files_once_and_nothing_on_a_second_run() {
    let _serial = serial();
    let scratch = Scratch::new("example");
    let root = scratch.path();
    write_repo(root, &unsynced());
    let root_arg = root.to_str().expect("a UTF-8 path");
    let spdx = [
        "--copyright",
        "2026 Example Org <dev@example.org>",
        "--licence",
        "CC-BY-4.0",
    ];
    let with = |extra: &[&'static str]| -> Vec<&str> {
        let mut args = vec![root_arg];
        args.extend_from_slice(extra);
        args.extend_from_slice(&spdx);
        args
    };

    // --check on a stale tree: exit 1, names each file, writes nothing.
    let checked = docs_sync(&with(&["--check"]));
    assert_eq!(checked.status.code(), Some(1), "{}", text(&checked.stderr));
    let listed: BTreeSet<String> = text(&checked.stdout).lines().map(str::to_string).collect();
    assert_eq!(
        listed,
        [ALPHA, AUXILIARY, BETA, GAMMA, INDEX]
            .iter()
            .map(|p| format!("stale {p}"))
            .collect()
    );
    assert_eq!(read(root, ALPHA), unsynced()[ALPHA]);
    assert!(!root.join(INDEX).exists());

    // The first run writes the expected files and says so.
    let first = docs_sync(&with(&[]));
    assert_eq!(first.status.code(), Some(0), "{}", text(&first.stderr));
    let wrote: BTreeSet<String> = text(&first.stdout).lines().map(str::to_string).collect();
    assert_eq!(
        wrote,
        [ALPHA, AUXILIARY, BETA, GAMMA, INDEX]
            .iter()
            .map(|p| format!("rewrote {p}"))
            .collect()
    );
    let expected = base();
    for path in [ALPHA, AUXILIARY, BETA, GAMMA, INDEX] {
        assert_eq!(read(root, path), expected[path], "{path}");
    }

    // The second run writes nothing and prints nothing; --check agrees.
    let second = docs_sync(&with(&[]));
    assert_eq!(second.status.code(), Some(0), "{}", text(&second.stderr));
    assert_eq!(text(&second.stdout), "");
    let again = docs_sync(&with(&["--check"]));
    assert_eq!(again.status.code(), Some(0), "{}", text(&again.stderr));
    assert_eq!(text(&again.stdout), "");
    for path in [ALPHA, AUXILIARY, BETA, GAMMA, INDEX] {
        assert_eq!(read(root, path), expected[path], "{path}");
    }
}

#[test]
fn the_docs_sync_example_in_a_git_checkout_and_with_bad_arguments() {
    let _serial = serial();
    let scratch = Scratch::new("example-git");
    let root = scratch.path();
    write_repo(root, &unsynced());
    let git = |args: &[&str]| {
        let output = support::checkout::git_command(root)
            .args(args)
            .output()
            .unwrap_or_else(|e| panic!("git is needed for this test: {e}"));
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            text(&output.stderr)
        );
    };
    git(&["init", "--quiet"]);
    git(&["add", "-A"]);
    let root_arg = root.to_str().expect("a UTF-8 path");
    let run = docs_sync(&[
        root_arg,
        "--copyright",
        "2026 Example",
        "--licence",
        "CC-BY-4.0",
    ]);
    assert_eq!(run.status.code(), Some(0), "{}", text(&run.stderr));
    assert_eq!(text(&run.stdout).lines().count(), 5);
    assert_eq!(read(root, GAMMA), base()[GAMMA]);

    // Arguments: unknown options are refused; the SPDX flags are not needed once
    // the index exists.
    let again = docs_sync(&[root_arg]);
    assert_eq!(again.status.code(), Some(0), "{}", text(&again.stderr));
    for args in [
        vec![root_arg, "--copyright"],
        vec![
            root_arg,
            "--frobnicate",
            "--copyright",
            "x",
            "--licence",
            "y",
        ],
        vec![root_arg, "extra", "--copyright", "x", "--licence", "y"],
    ] {
        let refused = docs_sync(&args);
        assert_eq!(refused.status.code(), Some(2), "{args:?}");
        assert!(text(&refused.stderr).contains("docs_sync:"), "{args:?}");
        assert_eq!(text(&refused.stdout), "");
    }

    // A tree that sync refuses is a message and exit 1, with nothing written.
    let mut broken = base();
    broken
        .get_mut(INDEX)
        .expect("index")
        .push_str("\nHand written.\n");
    let scratch = Scratch::new("example-refused");
    write_repo(scratch.path(), &broken);
    let refused = docs_sync(&[
        scratch.path().to_str().expect("a UTF-8 path"),
        "--copyright",
        "x",
        "--licence",
        "y",
    ]);
    assert_eq!(refused.status.code(), Some(2));
    assert!(
        text(&refused.stderr).contains("index-hand-written"),
        "{}",
        text(&refused.stderr)
    );
    assert_eq!(read(scratch.path(), INDEX), broken[INDEX]);
}

// --- the catalogue is the roadmap ------------------------------------------------------

/// Undo the escaping of a table cell.
fn unescape(cell: &str) -> String {
    let mut out = String::new();
    let mut chars = cell.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            out.extend(chars.next());
        } else {
            out.push(c);
        }
    }
    out.replace("&lt;", "<")
}

/// The cells of a table row: split on pipes that are not escaped.
fn cells(row: &str) -> Vec<String> {
    let inner = row
        .strip_prefix("| ")
        .and_then(|row| row.strip_suffix(" |"))
        .unwrap_or_else(|| panic!("not a row: {row:?}"));
    let mut cells = vec![String::new()];
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                let last = cells.last_mut().expect("a cell");
                last.push(c);
                last.extend(chars.next());
            }
            '|' => {
                cells.push(String::new());
                // The separator is " | ": drop the space after it.
                if chars.peek() == Some(&' ') {
                    chars.next();
                }
            }
            _ => cells.last_mut().expect("a cell").push(c),
        }
    }
    // The space before each separator belongs to it.
    cells
        .into_iter()
        .map(|cell| cell.strip_suffix(' ').map_or(cell.clone(), str::to_string))
        .collect()
}

/// The catalogue rows as `(state, slug, link target, description, gaps)`.
fn catalogue_rows(index: &str) -> Vec<(String, String, String, String, String)> {
    let mut state = String::new();
    let mut rows = Vec::new();
    for line in index.lines() {
        if let Some(name) = line.strip_prefix("## ") {
            state = name.to_lowercase();
        } else if line.starts_with("| [") {
            let cells = cells(line);
            assert_eq!(cells.len(), 3, "{line}");
            let link = &cells[0];
            let (slug, target) = link
                .strip_prefix('[')
                .and_then(|rest| rest.split_once("]("))
                .and_then(|(slug, rest)| rest.strip_suffix(')').map(|t| (slug, t)))
                .unwrap_or_else(|| panic!("not a link: {link}"));
            rows.push((
                state.clone(),
                slug.to_string(),
                target.to_string(),
                unescape(&cells[1]),
                unescape(&cells[2]),
            ));
        }
    }
    rows
}

/// What the roadmap says, in the shape of `catalogue_rows`.
fn roadmap_rows(plan: &Workplan<'_>) -> Vec<(String, String, String, String, String)> {
    plan.status()
        .entries()
        .iter()
        .map(|entry| {
            (
                entry.state().name().to_string(),
                entry.slug().to_string(),
                format!("./{}", entry.link_target()),
                entry.description().to_string(),
                entry
                    .gaps()
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; "),
            )
        })
        .collect()
}

#[test]
fn the_roadmap_rows_equal_the_catalogue_rows() {
    let repo = base();
    with_plan(&repo, |plan| {
        let rows = catalogue_rows(&repo[INDEX]);
        assert_eq!(rows.len(), 5);
        assert_eq!(rows, roadmap_rows(plan));
        // The description with a pipe and quotes came back unescaped.
        assert!(rows.iter().any(|r| r.3 == "Being built | with \"care\"."));
        // The states, in order.
        let mut states: Vec<&str> = rows.iter().map(|r| r.0.as_str()).collect();
        assert_eq!(
            states,
            [
                "delivered",
                "delivered",
                "implementing",
                "designed",
                "considered"
            ]
        );
        let order: Vec<&str> = raf_workplan::Roadmap::ORDER
            .iter()
            .map(|s| s.name())
            .collect();
        states.dedup();
        assert_eq!(states, order);
    });
}

/// The repository root: `RAF_GATE_ROOT` when set, this repository otherwise.
fn repo_root() -> PathBuf {
    let root = std::env::var_os("RAF_GATE_ROOT").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        PathBuf::from,
    );
    root.canonicalize()
        .unwrap_or_else(|e| panic!("cannot resolve the repository root {}: {e}", root.display()))
}

#[test]
fn the_catalogue_of_this_repository_equals_its_roadmap() {
    let root = repo_root();
    let tree = DiskTree::new(&root).unwrap_or_else(|e| panic!("cannot open the tree: {e}"));
    let source = Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("workspace: {e}"));
    let plan = Workplan::load(&tree, &source);
    let report = plan
        .sync(&options())
        .unwrap_or_else(|e| panic!("sync: {e}"));
    // The index is either about to be created or already there.
    let index = report
        .edits
        .iter()
        .find(|(path, _)| path.as_str() == INDEX)
        .map_or_else(|| read(&root, INDEX), |(_, text)| text.clone());
    let rows = catalogue_rows(&index);
    assert!(rows.len() >= 10, "{} capabilities", rows.len());
    assert_eq!(rows, roadmap_rows(&plan));
    // The tree is not written by a sync.
    assert!(sync_is_pure(&plan));
}

/// Whether two syncs of the same plan are equal.
fn sync_is_pure(plan: &Workplan<'_>) -> bool {
    plan.sync(&options()).ok() == plan.sync(&options()).ok()
}

#[test]
fn the_docs_sync_example_asks_for_the_spdx_flags_only_when_the_index_must_be_created() {
    let _serial = serial();
    let scratch = Scratch::new("example-flags");
    write_repo(scratch.path(), &unsynced());
    let root_arg = scratch.path().to_str().expect("a UTF-8 path");
    for (args, flag) in [
        (vec![root_arg], "--copyright"),
        (vec![root_arg, "--copyright", "2026 Example"], "--licence"),
        (vec![root_arg, "--licence", "CC-BY-4.0"], "--copyright"),
    ] {
        let run = docs_sync(&args);
        assert_eq!(run.status.code(), Some(2), "{args:?}");
        assert!(
            text(&run.stderr).contains(flag),
            "{args:?}: {}",
            text(&run.stderr)
        );
        assert!(!scratch.path().join(INDEX).exists());
    }
}

#[test]
fn the_docs_sync_example_opens_the_repository_as_the_gate_does() {
    let _serial = serial();
    // A root that holds no capability document is not a repository to sync:
    // the example exits 2 with the gate's message and writes nothing.
    let empty = Scratch::new("example-empty");
    fs::write(empty.path().join("note.txt"), "x\n").expect("write");
    let refused = docs_sync(&[
        empty.path().to_str().expect("a UTF-8 path"),
        "--copyright",
        "x",
        "--licence",
        "y",
    ]);
    assert_eq!(refused.status.code(), Some(2));
    assert!(
        text(&refused.stderr).contains("docs/capabilities"),
        "{}",
        text(&refused.stderr)
    );
    assert_eq!(text(&refused.stdout), "");
    assert_eq!(fs::read_dir(empty.path()).expect("lists").count(), 1);

    // A root that does not exist is the gate's "cannot open" message.
    let gone = empty.path().join("gone");
    let refused = docs_sync(&[gone.to_str().expect("a UTF-8 path")]);
    assert_eq!(refused.status.code(), Some(2));
    assert!(text(&refused.stderr).contains("docs_sync:"));
}

#[cfg(unix)]
#[test]
fn write_changes_no_target_when_a_later_file_cannot_be_written() {
    use std::os::unix::fs::PermissionsExt;
    let scratch = Scratch::new("atomic");
    let root = scratch.path();
    write_repo(root, &unsynced());
    // Three edits in path order; the second is in a directory that is read-only.
    fs::create_dir_all(root.join("docs/b")).expect("mkdir");
    fs::write(root.join("docs/b/two.md"), "two\n").expect("write");
    fs::create_dir_all(root.join("docs/a")).expect("mkdir");
    fs::write(root.join("docs/a/one.md"), "one\n").expect("write");
    fs::create_dir_all(root.join("docs/c")).expect("mkdir");
    fs::write(root.join("docs/c/three.md"), "three\n").expect("write");
    fs::set_permissions(root.join("docs/b"), fs::Permissions::from_mode(0o555)).expect("chmod");
    // This assertion needs an actual failed write, including under privileged
    // test runners; never turn an ineffective fixture into a passing test.
    let probe = fs::File::create(root.join("docs/b/probe"));
    if let Ok(probe) = probe {
        drop(probe);
        fs::remove_file(root.join("docs/b/probe")).expect("remove probe");
        fs::set_permissions(root.join("docs/b"), fs::Permissions::from_mode(0o755)).expect("chmod");
        panic!(
            "the write-failure fixture needs directory permissions to refuse creation; run as an unprivileged user"
        );
    }
    let report = SyncReport {
        diagram_assets: None,
        edits: vec![
            (rel("docs/a/one.md"), "ONE\n".to_string()),
            (rel("docs/b/two.md"), "TWO\n".to_string()),
            (rel("docs/c/three.md"), "THREE\n".to_string()),
        ],
    };
    let result = Workplan::write(&report, root);
    fs::set_permissions(root.join("docs/b"), fs::Permissions::from_mode(0o755)).expect("chmod");
    assert!(result.is_err());
    assert_eq!(read(root, "docs/a/one.md"), "one\n");
    assert_eq!(read(root, "docs/b/two.md"), "two\n");
    assert_eq!(read(root, "docs/c/three.md"), "three\n");
    for dir in ["docs/a", "docs/b", "docs/c"] {
        let names: Vec<_> = fs::read_dir(root.join(dir)).expect("lists").collect();
        assert_eq!(names.len(), 1, "{dir}: no temporary file is left");
    }
}

#[test]
fn write_never_touches_a_file_that_is_already_there_under_a_temporary_name() {
    let scratch = Scratch::new("tmpname");
    write_repo(scratch.path(), &unsynced());
    let dir = scratch.path().join("docs/capabilities");
    let decoy = dir.join(format!(".alpha.md.{}.0.sync.tmp", std::process::id()));
    fs::write(&decoy, "decoy").expect("write");
    let plain = dir.join(".alpha.md.sync.tmp");
    fs::write(&plain, "plain").expect("write");
    let report = on_disk(scratch.path(), |plan| plan.sync(&options())).expect("syncs");
    Workplan::write(&report, scratch.path()).expect("writes");
    assert_eq!(fs::read_to_string(&decoy).expect("read"), "decoy");
    assert_eq!(fs::read_to_string(&plain).expect("read"), "plain");
    assert_eq!(read(scratch.path(), ALPHA), base()[ALPHA]);
}

fn authored_model_neighbors() {
    use component_model::{MODEL, SOURCE, document, with_plan};
    use purrdf_rdf::viz::VizTermValue;
    component_model::main().expect("the consuming example renders");
    let original = document(MODEL);
    let view = NamedDiagramView::new(
        "parcel",
        DiagramScope::Automatic {
            kind: DiagramKind::Components,
            scope: "cap:routing".into(),
        },
    )
    .unwrap();
    let model_ids = |export: &purrdf_rdf::viz::VizExport| {
        export
            .model
            .terms
            .iter()
            .filter_map(|term| match &term.value {
                VizTermValue::Iri { value } if value.contains("/model/") => Some(value.clone()),
                _ => None,
            })
            .collect::<BTreeSet<_>>()
    };
    let (identities, original_hash, original_domain) = with_plan(&original, SOURCE, |plan| {
        authored_model_bundle(plan, &view, &model_ids)
    });
    for (document, source) in [
        (
            document(&MODEL.replace("Receive parcel", "Read parcel")),
            SOURCE.to_owned(),
        ),
        (
            original.clone(),
            SOURCE.replace("pub fn store() {}", "pub fn store() { let _changed = 1; }"),
        ),
    ] {
        with_plan(&document, &source, |plan| {
            let bundle = plan.diagram(&view).unwrap();
            let provenance = bundle
                .details()
                .iter()
                .find(|detail| detail.view().kind() == DiagramKind::ModelProvenance)
                .unwrap();
            assert_eq!(
                identities,
                model_ids(provenance.export()),
                "labels and source bodies do not move document-derived identities"
            );
            assert_ne!(
                original_hash,
                bundle.full_model_hash().unwrap(),
                "changed declaration/source inputs invalidate the view"
            );
            assert_eq!(
                original_domain,
                bundle.export().model_hash,
                "selected domain facts did not change"
            );
            assert!(
                bundle.svg().contains(bundle.full_model_hash().unwrap()),
                "SVG carries the current full-model currency separately"
            );
        });
    }
    authored_model_refusals(&original, &view);
}

fn authored_model_refusals(original: &str, view: &NamedDiagramView) {
    use component_model::{MODEL, SOURCE, document, with_plan};
    for (from, to, key) in [
        ("parent: delivery", "parent: router", "model.components[1]"),
        (
            "participants: [customer, router, storage]",
            "participants: [customer, router]",
            "model.scenarios[0].steps[1].relation",
        ),
        ("kind: parallel", "kind: sequence", "model.scenarios[0]"),
        (
            "consumers: [storage]",
            "consumers: [customer]",
            "model.relations[1].artifact",
        ),
        (
            "order: 1\n          label: Validation decision",
            "order: 2\n          label: Validation decision",
            "model.scenarios[0]",
        ),
        (
            "code: [parcel::store]",
            "code: [parcel::missing]",
            "model.components[2].bindings.code[0]",
        ),
    ] {
        let changed = original.replacen(from, to, 1);
        assert_ne!(changed, original);
        with_plan(&changed, SOURCE, |plan| {
            let finding = plan
                .findings()
                .iter()
                .find(|finding| finding.kind == "model-invalid")
                .unwrap_or_else(|| panic!("{from}: {:?}", plan.findings()));
            assert!(
                matches!(&finding.location, raf_okf::Location::Key(location) if location.starts_with(key)),
                "{finding:?}"
            );
            assert_ne!(finding.fix, "");
            assert!(plan.diagram(view).is_err());
        });
    }
    with_plan(
        original,
        &SOURCE.replace("pub fn route() {}", "pub fn renamed() {}"),
        |plan| {
            assert!(
                plan.diagram(view).is_err(),
                "a renamed source identity refuses"
            );
        },
    );
    let ambiguous = SOURCE.replace(
        "pub fn route() {}",
        "#[cfg(unix)] pub fn route() {}\n#[cfg(windows)] pub fn route() {}\n",
    );
    with_plan(original, &ambiguous, |plan| {
        assert!(
            plan.diagram(view).is_err(),
            "ambiguous source identities refuse"
        );
    });
    for claim in ["observed", "verified"] {
        with_plan(
            &document(&MODEL.replacen("claim: declared", &format!("claim: {claim}"), 1)),
            SOURCE,
            |plan| {
                assert!(
                    plan.diagram(view).is_err(),
                    "unsupported execution strength refuses"
                );
            },
        );
    }
}

fn authored_model_bundle(
    plan: &Workplan<'_>,
    view: &NamedDiagramView,
    model_ids: &impl Fn(&purrdf_rdf::viz::VizExport) -> BTreeSet<String>,
) -> (BTreeSet<String>, String, String) {
    use purrdf_rdf::viz::VizRegionKind;
    assert!(
        !plan
            .findings()
            .iter()
            .any(|finding| finding.kind == "model-invalid"),
        "{:?}",
        plan.findings()
    );
    let bundle = plan.diagram(view).unwrap();
    assert_eq!(
        bundle.export().scene.nodes.len(),
        4,
        "the primary scene contains the three components and external actor"
    );
    assert!(
        bundle
            .export()
            .model
            .terms
            .iter()
            .all(|term| !matches!(term.value, purrdf_rdf::viz::VizTermValue::Literal { .. })),
        "literal metadata belongs to full provenance"
    );
    let provenance = authored_model_provenance(&bundle);
    let detail = bundle
        .details()
        .iter()
        .find(|detail| detail.view().kind() == DiagramKind::Scenario)
        .unwrap();
    let native = detail.presented_export().unwrap();
    let scenario = native.presentation.scenario.as_ref().unwrap();
    assert_eq!(scenario.interactions.len(), 4);
    assert!(
        scenario
            .regions
            .iter()
            .any(|region| region.kind == VizRegionKind::Error)
    );
    assert!(
        scenario
            .regions
            .iter()
            .any(|region| region.kind == VizRegionKind::Parallel)
    );
    assert!(
        native
            .presentation
            .displays
            .iter()
            .any(|display| display.navigation.is_some())
    );
    for interaction in &native.geometry.interactions {
        assert!(detail.text().contains(&interaction.label.full_text));
        assert!(detail.svg().contains("DECLARED"));
    }
    assert!(detail.json().contains(&native.presentation_hash));
    assert!(detail.svg().contains(&native.presentation_hash));
    for kind in [
        DiagramKind::Scenario,
        DiagramKind::Artifacts,
        DiagramKind::ModelProvenance,
    ] {
        let scope = if kind == DiagramKind::Scenario {
            "cap:routing#deliver"
        } else {
            "cap:routing"
        };
        let selected = NamedDiagramView::new(
            "selected",
            DiagramScope::Automatic {
                kind,
                scope: scope.into(),
            },
        )
        .unwrap();
        assert!(
            plan.diagram(&selected)
                .unwrap()
                .presented_export()
                .is_some()
        );
    }
    (
        model_ids(provenance.export()),
        bundle.full_model_hash().unwrap().to_owned(),
        bundle.export().model_hash.clone(),
    )
}

fn authored_model_provenance(
    bundle: &raf_workplan::diagram::DiagramBundle,
) -> &raf_workplan::diagram::DiagramBundle {
    let provenance = bundle
        .details()
        .iter()
        .find(|detail| detail.view().kind() == DiagramKind::ModelProvenance)
        .unwrap();
    assert_eq!(
        bundle.full_model_hash(),
        Some(provenance.export().model_hash.as_str())
    );
    assert!(
        provenance.json().contains("sourceDigest")
            && provenance.json().contains("claimStrength")
            && provenance.json().contains("regionKind")
    );
    assert!(provenance.export().scene.nodes.len() > bundle.export().scene.nodes.len());
    assert!(bundle.svg().contains(bundle.full_model_hash().unwrap()));
    named_dataset_round_trip(bundle.dataset());
    provenance
}

fn named_dataset_round_trip(dataset: &purrdf_core::RdfDataset) {
    use purrdf_rdf::{
        NativeRdfFormat, ParseOptions, SerializeGraph, SerializeOptions, StatementLayer,
        parse_dataset_with, serialize_dataset_with,
    };
    let serialized = serialize_dataset_with(
        dataset,
        NativeRdfFormat::TriG,
        None,
        &SerializeOptions {
            selection: SerializeGraph::Dataset,
            statement_layer: StatementLayer::Emit,
            jsonld_options: None,
        },
    )
    .unwrap();
    assert_eq!(serialized.named_graph_rows_dropped, 0);
    assert_eq!(serialized.statement_rows_dropped, 0);
    let parsed = parse_dataset_with(
        &serialized.bytes,
        "application/trig",
        None,
        &ParseOptions::default(),
    )
    .unwrap();
    let spec = purrdf_rdf::viz::VizSpec::default();
    assert_eq!(
        purrdf_rdf::viz::project_dataset(dataset, &spec).unwrap(),
        purrdf_rdf::viz::project_dataset(&parsed.dataset, &spec).unwrap(),
        "native TriG preserves every graph and RDF statement/provenance fact"
    );
}
