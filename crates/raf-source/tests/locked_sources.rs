// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Source/manifest proof feeds actual package identities into the existing resolver.

#[path = "support/git_objects.rs"]
mod git_objects;
mod support;

use git_objects::{GitObjectId, GitObjectKind, Objects, entry};
use raf_source::{
    Budget, CommittedGitTree, GitPackageRoots, LoadError, LoadErrorKind, LoadOptions,
    LockedCatalog, LockedEdge, LockedManifest, LockedPackageId, MemoryTree, Namespace,
    ReferenceObservation, ReferencePackageRoots, ReferenceRequest, ReferenceTree, RegistryIndex,
    RelPath, ResolutionAttempt, ResolveError, Workspace,
};
use std::fmt::Write as _;

#[test]
fn reference_observations_verify_manifests_without_admitting_provider_routes() {
    let options = LoadOptions::default();
    let tree = MemoryTree::from([
        (
            "Cargo.toml",
            "[workspace]\nmembers=['app','helper']\n[workspace.package]\nversion='1.0.0'\n[workspace.dependencies]\nrenamed={package='helper',path='helper',version='1'}\n",
        ),
        (
            "app/Cargo.toml",
            "[package]\nname='app'\nversion.workspace=true\nedition='2024'\n[dependencies]\nrenamed.workspace=true\n",
        ),
        (
            "app/src/lib.rs",
            "use renamed::missing as unavailable; mod facade {pub use renamed::missing as alias;} use facade::alias as chained; use std::missing as external; pub fn run() {}",
        ),
        (
            "helper/Cargo.toml",
            "[package]\nname='helper'\nversion.workspace=true\nedition='2024'\n",
        ),
        ("helper/src/lib.rs", "pub fn provided() {}"),
        (
            "Cargo.lock",
            "version=4\n[[package]]\nname='app'\nversion='1.0.0'\ndependencies=['helper']\n[[package]]\nname='helper'\nversion='1.0.0'\n",
        ),
    ]);
    let workspace = Workspace::load(&tree, &["."]).unwrap();
    unresolved_routes_preserve_target_identity_and_query_grants(&workspace);
    let app = workspace
        .package_ids()
        .find(|id| workspace.package(*id).unwrap().name == "app")
        .unwrap();
    let declaration = &workspace.dependency_declarations(app).unwrap()[0];
    let catalog =
        LockedCatalog::load(&tree, &RelPath::new("Cargo.lock").unwrap(), &options).unwrap();
    let owner = catalog
        .packages()
        .find(|row| row.name() == "app")
        .unwrap()
        .id();
    let request = ReferenceRequest::new(&catalog, owner, declaration);
    let owning = ReferenceObservation::local_root(
        &catalog,
        &tree,
        &RelPath::new("app/Cargo.toml").unwrap(),
        &options,
    )
    .unwrap();
    assert_eq!(owning.row(), owner);
    let manifest = RelPath::new("helper/Cargo.toml").unwrap();
    let observed = ReferenceObservation::load(&request, &tree, &manifest, &options).unwrap();
    reference_selection_and_source_use_one_work_allowance(&request, &catalog, &tree, &manifest);
    assert_eq!(observed.name(), "helper");
    assert_eq!(observed.version(), "1.0.0");
    assert!(
        observed
            .workspace()
            .locked_binding(observed.package())
            .is_none()
    );
    assert!(observed.workspace().resolve("helper::provided").is_ok());
    let wrong = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='helper'\nversion='2.0.0'\nedition='2024'\n",
        ),
        ("src/lib.rs", "pub fn provided() {}"),
    ]);
    assert!(
        ReferenceObservation::load(
            &request,
            &wrong,
            &RelPath::new("Cargo.toml").unwrap(),
            &options
        )
        .is_err()
    );
    let git_work = committed_reference_neighbors();
    committed_reference_budget_neighbor(git_work);
    #[cfg(unix)]
    parent_path_directory_context_neighbors(&request);
    structural_reference_resolution_uses_actual_directories(&catalog, owner, observed.row());
}

fn unresolved_routes_preserve_target_identity_and_query_grants(workspace: &Workspace) {
    for (path, expected) in [
        ("app::unavailable", vec!["app", "helper"]),
        ("app::chained", vec!["app", "helper"]),
        ("app::external", vec!["app"]),
    ] {
        assert!(matches!(
            workspace.resolve_in(path, Namespace::Value),
            Err(ResolveError::NotFound(_))
        ));
        let ResolutionAttempt::NotFound { attempted_targets } = workspace
            .resolve_in_observed(path, Namespace::Value)
            .unwrap()
        else {
            panic!("missing alias acquired a definition");
        };
        let actual: std::collections::BTreeSet<_> = attempted_targets
            .into_iter()
            .map(|id| workspace.target(id).unwrap().package.as_str())
            .collect();
        assert_eq!(actual, expected.into_iter().collect());
    }
    assert_eq!(
        workspace
            .resolve_in_observed("helper::provided", Namespace::Value)
            .unwrap(),
        ResolutionAttempt::Resolved(
            workspace
                .resolve_in("helper::provided", Namespace::Value)
                .unwrap()
        ),
    );
    assert!(matches!(
        workspace.resolve_in_observed("app::self::missing", Namespace::Value),
        Err(ResolveError::Invalid(_))
    ));
    let files = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='demo'\nversion='1.0.0'\nedition='2024'\n",
        ),
        ("src/lib.rs", "pub fn present(){}"),
    ]);
    for budget in [Budget::Work, Budget::GlobBytes] {
        let mut options = LoadOptions::default();
        match budget {
            Budget::Work => options.max_query_work = 0,
            Budget::GlobBytes => options.max_glob_bytes = 0,
            _ => unreachable!(),
        }
        let bounded = Workspace::load_with(&files, &["."], &options).unwrap();
        let error = bounded
            .resolve_in_observed("demo::missing", Namespace::Value)
            .unwrap_err();
        assert!(
            matches!(error, ResolveError::Budget { budget: actual, .. } if actual == budget),
            "{error}"
        );
    }
}

fn structural_reference_resolution_uses_actual_directories(
    catalog: &LockedCatalog,
    owner: LockedPackageId,
    dependency: LockedPackageId,
) {
    let (fixture, app, helper, other) = structural_reference_directories();
    let root = fixture.path();
    let edges = [LockedEdge::new("dependencies.renamed", None)];
    let local = [LockedManifest::new(
        owner,
        RelPath::new("Cargo.toml").unwrap(),
        &edges,
    )];
    let packages = [LockedManifest::new(
        dependency,
        RelPath::new("Cargo.toml").unwrap(),
        &[],
    )];
    let sources = [ReferencePackageRoots::new(
        ReferenceTree::Disk(&helper),
        &packages,
    )];
    for (body, native) in [
        ("pub use renamed::alias as imported;", true),
        (
            "mod renamed {pub fn alias(){}} pub use renamed::alias as imported;",
            false,
        ),
    ] {
        std::fs::write(root.join("app/src/lib.rs"), body).unwrap();
        let workspace = Workspace::load_reference_sources_under(
            catalog,
            ReferenceTree::Disk(&app),
            &[RelPath::root()],
            &local,
            &sources,
            &LoadOptions::default(),
        )
        .unwrap();
        let actual = workspace.resolve("app::imported").unwrap();
        let expected = workspace.resolve("native_helper::provided").unwrap();
        assert_eq!(actual.sites == expected.sites, native);
        assert!(
            workspace
                .package_ids()
                .all(|id| workspace.locked_binding(id).is_none())
        );
    }
    let refused = Workspace::load_reference_sources_under(
        catalog,
        ReferenceTree::Opaque(&app),
        &[RelPath::root()],
        &local,
        &sources,
        &LoadOptions::default(),
    )
    .unwrap_err();
    assert!(
        refused
            .to_string()
            .contains("cross-authority path relationships"),
        "{refused}"
    );
    let wrong = [ReferencePackageRoots::new(
        ReferenceTree::Disk(&other),
        &packages,
    )];
    let refused = Workspace::load_reference_sources_under(
        catalog,
        ReferenceTree::Disk(&app),
        &[RelPath::root()],
        &local,
        &wrong,
        &LoadOptions::default(),
    )
    .unwrap_err();
    assert!(
        refused.to_string().contains("different package directory"),
        "{refused}"
    );
    let configured = [
        LockedManifest::new(owner, RelPath::new("Cargo.toml").unwrap(), &edges)
            .with_configurations(&[]),
    ];
    let refused = Workspace::load_reference_sources_under(
        catalog,
        ReferenceTree::Disk(&app),
        &[RelPath::root()],
        &configured,
        &sources,
        &LoadOptions::default(),
    )
    .unwrap_err();
    assert!(
        refused
            .to_string()
            .contains("must not supply configured target evidence"),
        "{refused}"
    );
}

fn structural_reference_directories() -> (
    purrdf_testkit::TempDir,
    raf_source::DiskTree,
    raf_source::DiskTree,
    raf_source::DiskTree,
) {
    let owned = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    let root = owned.path();
    for dir in ["app/src", "helper/src", "other/src"] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }
    std::fs::write(root.join("app/Cargo.toml"), "[package]\nname='app'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nrenamed={package='helper',path='../helper',version='1',optional=true}\n").unwrap();
    std::fs::write(
        root.join("helper/Cargo.toml"),
        "[package]\nname='helper'\nversion='1.0.0'\nedition='2024'\n[lib]\nname='native_helper'\n",
    )
    .unwrap();
    std::fs::write(
        root.join("helper/src/lib.rs"),
        "pub fn provided() {} pub use provided as alias;",
    )
    .unwrap();
    std::fs::write(root.join("other/src/lib.rs"), "pub fn substituted() {}").unwrap();
    std::fs::hard_link(
        root.join("helper/Cargo.toml"),
        root.join("other/Cargo.toml"),
    )
    .unwrap();
    let app = raf_source::DiskTree::new(root.join("app")).unwrap();
    let helper = raf_source::DiskTree::new(root.join("helper")).unwrap();
    let other = raf_source::DiskTree::new(root.join("other")).unwrap();
    (owned, app, helper, other)
}

#[cfg(unix)]
fn parent_path_directory_context_neighbors(request: &ReferenceRequest<'_>) {
    use raf_source::Tree as _;
    let parent = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("reference-path-context");
    let owned = purrdf_testkit::TempDir::new_in(parent).unwrap();
    let root = owned.path();
    for dir in ["app", "helper/src", "branch/leaf", "branch/helper/src"] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }
    std::fs::write(root.join("Cargo.toml"), "[workspace]\nmembers=['helper']\n").unwrap();
    std::fs::write(
        root.join("helper/Cargo.toml"),
        "[package]\nname='helper'\nversion='1.0.0'\nedition='2024'\n",
    )
    .unwrap();
    std::fs::write(root.join("helper/src/lib.rs"), "pub fn provided() {}\n").unwrap();
    std::fs::write(
        root.join("branch/helper/src/lib.rs"),
        "pub fn substituted() {}\n",
    )
    .unwrap();
    std::fs::hard_link(
        root.join("helper/Cargo.toml"),
        root.join("branch/helper/Cargo.toml"),
    )
    .unwrap();
    std::os::unix::fs::symlink("branch/leaf", root.join("alias")).unwrap();
    let tree = raf_source::DiskTree::new(root).unwrap();
    let original = RelPath::new("alias/../helper/Cargo.toml").unwrap();
    let image = RelPath::new("helper/Cargo.toml").unwrap();
    assert_eq!(
        tree.identity(&original).unwrap(),
        tree.identity(&image).unwrap()
    );
    ReferenceObservation::load(
        request,
        &tree,
        &RelPath::new("app/../helper/Cargo.toml").unwrap(),
        &LoadOptions::default(),
    )
    .unwrap();
    let refused = ReferenceObservation::load(request, &tree, &original, &LoadOptions::default())
        .err()
        .expect("hardlinked manifests cannot authorize a different package directory");
    assert!(
        refused.to_string().contains("directory symlink"),
        "{refused}"
    );
}

fn reference_selection_and_source_use_one_work_allowance(
    request: &ReferenceRequest<'_>,
    catalog: &LockedCatalog,
    tree: &MemoryTree,
    manifest: &RelPath,
) {
    let mut bounded = LoadOptions::default();
    let source = ReferenceObservation::local_root(catalog, tree, manifest, &bounded).unwrap();
    bounded.max_work = source.workspace().usage().work;
    ReferenceObservation::local_root(catalog, tree, manifest, &bounded).unwrap();
    request.select(&bounded).unwrap();
    let refused = ReferenceObservation::load(request, tree, manifest, &bounded)
        .err()
        .expect("selection and source cannot each spend the original grant");
    assert!(
        matches!(
            refused.kind,
            LoadErrorKind::Budget {
                budget: Budget::Work,
                ..
            }
        ),
        "{refused}"
    );
}

#[test]
fn local_reference_owner_selection_spends_its_original_work_allowance() {
    let tree = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='app'\nversion='1.0.0'\nedition='2024'\n",
        ),
        ("src/lib.rs", "pub fn run() {}"),
        (
            "Cargo.lock",
            "version=4\n[[package]]\nname='app'\nversion='1.0.0'\n[[package]]\nname='unrelated'\nversion='2.0.0'\n",
        ),
    ]);
    let mut options = LoadOptions::default();
    let manifest = RelPath::new("Cargo.toml").unwrap();
    let catalog =
        LockedCatalog::load(&tree, &RelPath::new("Cargo.lock").unwrap(), &options).unwrap();
    let small = MemoryTree::from([(
        "Cargo.lock",
        "version=4\n[[package]]\nname='app'\nversion='1.0.0'\n",
    )]);
    let minimal =
        LockedCatalog::load(&small, &RelPath::new("Cargo.lock").unwrap(), &options).unwrap();
    let admitted = ReferenceObservation::local_root(&minimal, &tree, &manifest, &options).unwrap();
    options.max_work = admitted.workspace().usage().work;
    ReferenceObservation::local_root(&minimal, &tree, &manifest, &options).unwrap();
    let refused = ReferenceObservation::local_root(&catalog, &tree, &manifest, &options)
        .err()
        .expect("additional catalog comparisons exceed the original grant");
    assert!(
        matches!(
            refused.kind,
            LoadErrorKind::Budget {
                budget: Budget::Work,
                ..
            }
        ),
        "{refused}"
    );
}

#[test]
fn registry_reference_observations_match_selected_manifest_identity() {
    let source = "registry+https://example.test/index";
    let lock = format!(
        "version=4\n[[package]]\nname='app'\nversion='1.0.0'\ndependencies=['helper']\n[[package]]\nname='helper'\nversion='1.2.0'\nsource='{source}'\nchecksum='{}'\n",
        "a".repeat(64)
    );
    let tree = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='app'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nalias={package='helper',version='1',registry='owned'}\n",
        ),
        ("src/lib.rs", "pub fn run() {}"),
        ("Cargo.lock", lock.as_str()),
    ]);
    let options = LoadOptions::default();
    let catalog =
        LockedCatalog::load(&tree, &RelPath::new("Cargo.lock").unwrap(), &options).unwrap();
    let workspace = Workspace::load(&tree, &["."]).unwrap();
    let declaration = &workspace
        .dependency_declarations(workspace.package_ids().next().unwrap())
        .unwrap()[0];
    let owner = catalog
        .packages()
        .find(|row| row.name() == "app")
        .unwrap()
        .id();
    let request = ReferenceRequest::new(&catalog, owner, declaration)
        .registry(RegistryIndex::new(Some("owned"), source));
    let package = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='helper'\nversion='1.2.0'\nedition='2024'\n",
        ),
        ("src/lib.rs", "pub fn provided() {}"),
    ]);
    let manifest = RelPath::new("Cargo.toml").unwrap();
    let observed = ReferenceObservation::load(&request, &package, &manifest, &options).unwrap();
    assert_eq!(observed.version(), "1.2.0");
    assert!(observed.checksum().is_some());
    assert!(
        observed
            .workspace()
            .locked_binding(observed.package())
            .is_none()
    );
    let wrong = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='other'\nversion='1.2.0'\nedition='2024'\n",
        ),
        ("src/lib.rs", "pub fn provided() {}"),
    ]);
    assert!(ReferenceObservation::load(&request, &wrong, &manifest, &options).is_err());
}

fn committed_reference_neighbors() -> usize {
    let fixture = Fixture::new();
    let tree = fixture.consumer("");
    let workspace = Workspace::load(&tree, &["."]).unwrap();
    let declaration = &workspace
        .dependency_declarations(workspace.package_ids().next().unwrap())
        .unwrap()[0];
    let owner = fixture.row("consumer", None);
    let request = ReferenceRequest::new(&fixture.catalog, owner, declaration);
    let manifest = RelPath::new("kit/Cargo.toml").unwrap();
    let committed = CommittedGitTree::open(
        &fixture.catalog,
        request.select(&LoadOptions::default()).unwrap(),
        fixture.objects.path(),
        &LoadOptions::default(),
    )
    .unwrap();
    let refused =
        ReferenceObservation::load(&request, &committed, &manifest, &LoadOptions::default())
            .err()
            .expect("generic Tree cannot admit otherwise valid committed bytes");
    assert!(
        refused.to_string().contains("sealed committed Tree"),
        "{refused}"
    );
    let caller_usage = committed.usage();
    let observed =
        ReferenceObservation::committed(&request, &committed, &manifest, &LoadOptions::default())
            .unwrap();
    assert_eq!(committed.usage(), caller_usage);
    let mut exact = LoadOptions::default();
    exact.max_work = observed.workspace().usage().work;
    ReferenceObservation::committed(&request, &committed, &manifest, &exact).unwrap();
    assert_eq!(observed.name(), "kit");
    assert_eq!(observed.version(), "3.0.1");
    assert!(
        observed
            .workspace()
            .locked_binding(observed.package())
            .is_none()
    );
    let other = CommittedGitTree::open(
        &fixture.catalog,
        fixture.row("kit", Some("https://example.test/other")),
        fixture.objects.path(),
        &LoadOptions::default(),
    )
    .unwrap();
    assert!(
        ReferenceObservation::committed(&request, &other, &manifest, &LoadOptions::default())
            .is_err()
    );
    observed.workspace().usage().work
}

fn committed_reference_budget_neighbor(work: usize) {
    let fixture = Fixture::with_padding(512);
    let consumer = fixture.consumer("");
    let workspace = Workspace::load(&consumer, &["."]).unwrap();
    let declaration = &workspace
        .dependency_declarations(workspace.package_ids().next().unwrap())
        .unwrap()[0];
    let request =
        ReferenceRequest::new(&fixture.catalog, fixture.row("consumer", None), declaration);
    let mut options = LoadOptions::default();
    options.max_work = work;
    let selected = request.select(&options).unwrap();
    let committed = CommittedGitTree::open(
        &fixture.catalog,
        selected,
        fixture.objects.path(),
        &LoadOptions::default(),
    )
    .unwrap();
    let refused = ReferenceObservation::committed(
        &request,
        &committed,
        &RelPath::new("kit/Cargo.toml").unwrap(),
        &options,
    )
    .err()
    .expect("extra native Git listing work exceeds the original observation allowance");
    assert!(
        matches!(
            refused.kind,
            LoadErrorKind::Budget {
                budget: Budget::Work,
                ..
            }
        ),
        "{refused}"
    );
}

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
        Self::with_padding(0)
    }

    fn with_padding(padding: usize) -> Self {
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
        let mut root_entries = [
            entry("100644", "Cargo.toml", workspace),
            entry("40000", "core", core),
            entry("40000", "fixture_kit", fixture_kit),
            entry("40000", "impostor", impostor),
            entry("40000", "kit", kit),
        ]
        .concat();
        let extra = objects.insert(GitObjectKind::Blob, b"unrelated committed input".to_vec());
        for index in 0..padding {
            root_entries.extend(entry("100644", &format!("unused-{index:04}"), extra));
        }
        root_entries.extend(entry("40000", "wrong_version", wrong_version));
        let root = objects.insert(GitObjectKind::Tree, root_entries);
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
