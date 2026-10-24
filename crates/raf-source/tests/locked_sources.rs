// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Source/manifest proof feeds actual package identities into the existing resolver.

#[path = "support/git_objects.rs"]
mod git_objects;
mod support;

use git_objects::{GitObjectId, GitObjectKind, Objects, entry};
use raf_source::{
    Budget, CommittedGitTree, GitPackageRoots, LoadError, LoadErrorKind, LoadOptions,
    LockedCatalog, LockedEdge, LockedManifest, LockedPackageId, MemoryTree, Namespace, RelPath,
    ResolveError, Workspace,
};
use std::fmt::Write as _;

#[test]
fn explicit_first_party_roots_require_every_discovered_manifest_association() {
    let tree = MemoryTree::from([
        ("Cargo.toml", "[workspace]\nmembers=['one','two']\n"),
        (
            "one/Cargo.toml",
            "[package]\nname='one'\nversion='1.0.0'\nedition='2024'\n",
        ),
        ("one/src/lib.rs", "pub fn first() {}"),
        (
            "two/Cargo.toml",
            "[package]\nname='two'\nversion='1.0.0'\nedition='2024'\n",
        ),
        ("two/src/lib.rs", "pub fn second() {}"),
        (
            "Cargo.lock",
            "version=4\n[[package]]\nname='one'\nversion='1.0.0'\n[[package]]\nname='two'\nversion='1.0.0'\n",
        ),
    ]);
    let options = LoadOptions::default();
    let catalog =
        LockedCatalog::load(&tree, &RelPath::new("Cargo.lock").unwrap(), &options).unwrap();
    let local: Vec<_> = ["one", "two"]
        .into_iter()
        .map(|name| {
            let row = catalog
                .packages()
                .find(|row| row.name() == name)
                .unwrap()
                .id();
            LockedManifest::new(
                row,
                RelPath::new(&format!("{name}/Cargo.toml")).unwrap(),
                &[],
            )
        })
        .collect();
    let roots = [RelPath::root()];
    Workspace::load_locked_sources_under(&catalog, &tree, &roots, &local, &[], &options)
        .expect("complete inventory is admitted");
    let omitted =
        Workspace::load_locked_sources_under(&catalog, &tree, &roots, &local[..1], &[], &options)
            .expect_err("a supplied manifest must not narrow directory discovery");
    assert!(omitted.to_string().contains("association"), "{omitted}");
    let one = [RelPath::new("one").unwrap()];
    Workspace::load_locked_sources_under(&catalog, &tree, &one, &local[..1], &[], &options)
        .expect("explicitly narrower root has its exact association");
    assert!(
        Workspace::load_locked_sources_under(&catalog, &tree, &one, &local, &[], &options).is_err()
    );
    assert!(
        Workspace::load_locked_sources_under(&catalog, &tree, &[], &local, &[], &options).is_err()
    );
}

fn package(objects: &mut Objects, manifest: &str, lib: &str) -> GitObjectId {
    let manifest = objects.insert(GitObjectKind::Blob, manifest.as_bytes().to_vec());
    let lib = objects.insert(GitObjectKind::Blob, lib.as_bytes().to_vec());
    let src = objects.insert(GitObjectKind::Tree, entry("100644", "lib.rs", lib));
    objects.insert(
        GitObjectKind::Tree,
        [
            entry("100644", "Cargo.toml", manifest),
            entry("40000", "src", src),
        ]
        .concat(),
    )
}

struct Fixture {
    objects: Objects,
    catalog: LockedCatalog,
    commit: GitObjectId,
}

impl Fixture {
    fn new() -> Self {
        let mut objects = Objects::default();
        let workspace = objects.insert(GitObjectKind::Blob, b"[workspace]\nmembers=['kit','core']\n[workspace.package]\nversion='3.0.1'\nedition='2024'\n[workspace.dependencies]\ncorekit={path='core',version='3.0.1'}\n".to_vec());
        let kit = package(
            &mut objects,
            "[package]\nname='kit'\nversion.workspace=true\nedition.workspace=true\n[dependencies]\ncorekit.workspace=true",
            "pub use corekit::Core as Routed;",
        );
        let core = package(
            &mut objects,
            "[package]\nname='corekit'\nversion.workspace=true\nedition.workspace=true",
            "pub struct Core;",
        );
        let impostor = package(
            &mut objects,
            "[package]\nname='impostor'\nversion='3.0.1'\nedition='2024'\n[workspace]",
            "pub struct Forged;",
        );
        let wrong_version = package(
            &mut objects,
            "[package]\nname='kit'\nversion='4.0.0'\nedition='2024'\n[workspace]",
            "pub struct Forged;",
        );
        let fixture_kit = package(
            &mut objects,
            "[package]\nname='kit'\nversion='3.0.1'\nedition='2024'\n[workspace]",
            "pub struct Forged;",
        );
        let root = objects.insert(
            GitObjectKind::Tree,
            [
                entry("100644", "Cargo.toml", workspace),
                entry("40000", "core", core),
                entry("40000", "fixture_kit", fixture_kit),
                entry("40000", "impostor", impostor),
                entry("40000", "kit", kit),
                entry("40000", "wrong_version", wrong_version),
            ]
            .concat(),
        );
        let commit = objects.commit(root);
        let mut text = String::from("version=4\n");
        for source in ["first", "other"] {
            let source = format!("git+https://example.test/{source}?rev={commit}#{commit}");
            write!(text, "[[package]]\nname='kit'\nversion='3.0.1'\nsource={source:?}\ndependencies=[{:?}]\n[[package]]\nname='corekit'\nversion='3.0.1'\nsource={source:?}\n", format!("corekit 3.0.1 ({source})")).expect("write fixture");
        }
        write!(
            text,
            "[[package]]\nname='consumer'\nversion='1.0.0'\ndependencies=[{:?}]\n",
            format!("kit 3.0.1 (git+https://example.test/first?rev={commit}#{commit})")
        )
        .expect("write fixture");
        let catalog = LockedCatalog::load(
            &MemoryTree::from([("Cargo.lock", text.as_str())]),
            &RelPath::new("Cargo.lock").expect("path"),
            &LoadOptions::default(),
        )
        .expect("actual outgoing identities");
        Self {
            objects,
            catalog,
            commit,
        }
    }

    fn row(&self, name: &str, source: Option<&str>) -> LockedPackageId {
        self.catalog
            .packages()
            .find(|row| {
                row.name() == name
                    && row.source().map(raf_source::LockedSource::repository) == source
            })
            .expect("exact row")
            .id()
    }

    fn consumer(&self, dependency_suffix: &str) -> MemoryTree {
        let manifest = format!(
            "[package]\nname='consumer'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nrenamed={{package='kit',version='3.0.1',git='https://example.test/first',rev='{}'{dependency_suffix}}}",
            self.commit
        );
        MemoryTree::from([
            ("Cargo.toml", manifest.as_str()),
            (
                "src/lib.rs",
                "pub mod reader { pub use renamed::Routed as Alias; }",
            ),
        ])
    }

    fn load(
        &self,
        consumer: &MemoryTree,
        kit_manifest: &str,
        edges: &[LockedEdge<'_>],
        options: &LoadOptions,
    ) -> Result<Workspace, LoadError> {
        let first_kit = self.row("kit", Some("https://example.test/first"));
        let other_kit = self.row("kit", Some("https://example.test/other"));
        let first = CommittedGitTree::open(
            &self.catalog,
            first_kit,
            self.objects.path(),
            &LoadOptions::default(),
        )?;
        let other = CommittedGitTree::open(
            &self.catalog,
            other_kit,
            self.objects.path(),
            &LoadOptions::default(),
        )?;
        let local = [LockedManifest::new(
            self.row("consumer", None),
            RelPath::new("Cargo.toml").expect("path"),
            edges,
        )];
        let kit_edges = [LockedEdge::new("dependencies.corekit", None)];
        let first_roots = [
            LockedManifest::new(
                first_kit,
                RelPath::new(kit_manifest).expect("path"),
                &kit_edges,
            ),
            LockedManifest::new(
                self.row("corekit", Some("https://example.test/first")),
                RelPath::new("core/Cargo.toml").expect("path"),
                &[],
            ),
        ];
        let other_roots = [
            LockedManifest::new(
                other_kit,
                RelPath::new("kit/Cargo.toml").expect("path"),
                &kit_edges,
            ),
            LockedManifest::new(
                self.row("corekit", Some("https://example.test/other")),
                RelPath::new("core/Cargo.toml").expect("path"),
                &[],
            ),
        ];
        Workspace::load_locked_sources(
            &self.catalog,
            consumer,
            &local,
            &[
                GitPackageRoots::new(&first, &first_roots),
                GitPackageRoots::new(&other, &other_roots),
            ],
            options,
        )
    }
}

#[test]
fn exact_sources_versions_and_path_edges_resolve_aliases_to_the_actual_package() {
    let fixture = Fixture::new();
    let tree = fixture.consumer("");
    let edges = [LockedEdge::new("dependencies.renamed", None)];
    let ws = fixture
        .load(&tree, "kit/Cargo.toml", &edges, &LoadOptions::default())
        .expect("checked production join");
    let consumer = ws
        .package_ids()
        .find(|id| ws.package(*id).expect("package").name == "consumer")
        .expect("consumer");
    assert!(matches!(
        ws.resolve("corekit::Core"),
        Err(ResolveError::Ambiguous(_))
    ));
    for namespace in [Namespace::Type, Namespace::Value] {
        let resolution = ws
            .resolve_from_package_id_in(consumer, "consumer::reader", "Alias", namespace)
            .expect("alias through exact source");
        assert_eq!(resolution.path, "corekit::Core");
        for site in resolution.sites {
            let item = ws.item(site).expect("actual definition");
            assert_eq!(item.file.authority().index(), 1);
            let package = ws.target(item.target).expect("target").package_id;
            let binding = ws.locked_binding(package).expect("computed binding");
            assert_eq!(binding.name(), "corekit");
            assert_eq!(binding.version(), "3.0.1");
            assert_eq!(
                binding.row(),
                fixture.row("corekit", Some("https://example.test/first"))
            );
            assert_eq!(
                binding.source().expect("Git").revision(),
                Some(fixture.commit.to_string().as_str())
            );
        }
    }
}

#[test]
fn checked_contents_do_not_let_wrong_manifest_names_or_versions_acquire_a_row() {
    let fixture = Fixture::new();
    let tree = fixture.consumer("");
    let edges = [LockedEdge::new("dependencies.renamed", None)];
    for path in ["impostor/Cargo.toml", "wrong_version/Cargo.toml"] {
        let error = fixture
            .load(&tree, path, &edges, &LoadOptions::default())
            .expect_err("actual metadata mismatch");
        assert!(
            matches!(error.kind, LoadErrorKind::Locked { .. }),
            "{error}"
        );
        assert!(
            error.to_string().contains("actual manifest name/version"),
            "{error}"
        );
    }
    fixture
        .load(&tree, "kit/Cargo.toml", &edges, &LoadOptions::default())
        .expect("actual metadata neighbor");
}

#[test]
fn a_nested_same_name_and_version_cannot_replace_the_root_workspace_package() {
    let fixture = Fixture::new();
    let tree = fixture.consumer("");
    let edges = [LockedEdge::new("dependencies.renamed", None)];
    let error = fixture
        .load(
            &tree,
            "fixture_kit/Cargo.toml",
            &edges,
            &LoadOptions::default(),
        )
        .expect_err("a matching label inside a nested fixture is not a package locator");
    assert!(
        error.to_string().contains("unique matching package"),
        "{error}"
    );
    fixture
        .load(&tree, "kit/Cargo.toml", &edges, &LoadOptions::default())
        .expect("the checked root workspace member remains valid");
}

#[test]
fn requested_missing_and_optional_edges_refuse_and_unrequested_edges_never_borrow_a_name() {
    let fixture = Fixture::new();
    let edges = [LockedEdge::new("dependencies.absent", None)];
    let error = fixture
        .load(
            &fixture.consumer(""),
            "kit/Cargo.toml",
            &edges,
            &LoadOptions::default(),
        )
        .expect_err("missing requested key");
    assert!(
        error.to_string().contains("requested key is absent"),
        "{error}"
    );
    let edges = [LockedEdge::new("dependencies.renamed", None)];
    let error = fixture
        .load(
            &fixture.consumer(",optional=true"),
            "kit/Cargo.toml",
            &edges,
            &LoadOptions::default(),
        )
        .expect_err("no activation proof");
    assert!(
        error.to_string().contains("needs admitted configuration"),
        "{error}"
    );
    let ws = fixture
        .load(
            &fixture.consumer(""),
            "kit/Cargo.toml",
            &[],
            &LoadOptions::default(),
        )
        .expect("explicitly unrequested syntax");
    let consumer = ws
        .package_ids()
        .find(|id| ws.package(*id).expect("package").name == "consumer")
        .expect("consumer");
    assert!(
        ws.resolve_from_package_id(consumer, "consumer::reader", "Alias")
            .is_err()
    );
    fixture
        .load(
            &fixture.consumer(""),
            "kit/Cargo.toml",
            &edges,
            &LoadOptions::default(),
        )
        .expect("requested valid neighbor");
}

#[test]
fn all_snapshots_catalog_and_walker_use_one_original_resource_grant() {
    let fixture = Fixture::new();
    let tree = fixture.consumer("");
    let edges = [LockedEdge::new("dependencies.renamed", None)];
    let baseline = fixture
        .load(&tree, "kit/Cargo.toml", &edges, &LoadOptions::default())
        .expect("baseline");
    let usage = baseline.usage();
    let mut options = LoadOptions::default();
    options.max_work = usage.work;
    options.max_item_bytes = usage.item_bytes;
    options.max_total_bytes = usage.total_bytes;
    fixture
        .load(&tree, "kit/Cargo.toml", &edges, &options)
        .expect("exact combined allowance");
    for budget in [Budget::Work, Budget::ItemBytes, Budget::TotalBytes] {
        let mut short = options.clone();
        match budget {
            Budget::Work => short.max_work -= 1,
            Budget::ItemBytes => short.max_item_bytes -= 1,
            Budget::TotalBytes => short.max_total_bytes -= 1,
            _ => unreachable!("three varied grants"),
        }
        let error = fixture
            .load(&tree, "kit/Cargo.toml", &edges, &short)
            .expect_err("one short combined grant");
        assert!(
            matches!(error.kind, LoadErrorKind::Budget { budget: actual, .. } if actual == budget),
            "{error}"
        );
    }
}

#[test]
fn actual_version_evidence_is_required_without_changing_ordinary_missing_version_loading() {
    let fixture = Fixture::new();
    for version in [
        "",
        "version.workspace=true\n[workspace]",
        "version='not-a-version'",
    ] {
        let manifest = format!("[package]\nname='consumer'\n{version}\n");
        let tree = MemoryTree::from([("Cargo.toml", manifest.as_str()), ("src/lib.rs", "")]);
        let error = fixture
            .load(&tree, "kit/Cargo.toml", &[], &LoadOptions::default())
            .expect_err("no synthesized identity");
        assert!(
            matches!(
                error.kind,
                LoadErrorKind::Manifest { .. } | LoadErrorKind::Locked { .. }
            ),
            "{error}"
        );
        if version.is_empty() {
            Workspace::load(&tree, &["."]).expect("ordinary syntax loading remains accepted");
        }
    }
    fixture
        .load(
            &fixture.consumer(""),
            "kit/Cargo.toml",
            &[LockedEdge::new("dependencies.renamed", None)],
            &LoadOptions::default(),
        )
        .expect("actual version neighbor");
}

#[test]
fn another_checked_repository_cannot_supply_an_expected_source_by_name_or_authority() {
    let fixture = Fixture::new();
    let other = CommittedGitTree::open(
        &fixture.catalog,
        fixture.row("kit", Some("https://example.test/other")),
        fixture.objects.path(),
        &LoadOptions::default(),
    )
    .expect("other snapshot is individually checked");
    let edges = [LockedEdge::new("dependencies.renamed", None)];
    let local = [LockedManifest::new(
        fixture.row("consumer", None),
        RelPath::new("Cargo.toml").expect("path"),
        &edges,
    )];
    let expected = [LockedManifest::new(
        fixture.row("kit", Some("https://example.test/first")),
        RelPath::new("kit/Cargo.toml").expect("path"),
        &[],
    )];
    let tree = fixture.consumer("");
    let error = Workspace::load_locked_sources(
        &fixture.catalog,
        &tree,
        &local,
        &[GitPackageRoots::new(&other, &expected)],
        &LoadOptions::default(),
    )
    .expect_err("opaque source identity mismatch");
    assert!(
        error.to_string().contains("source does not match"),
        "{error}"
    );
    fixture
        .load(&tree, "kit/Cargo.toml", &edges, &LoadOptions::default())
        .expect("actual source neighbor");
}
