// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Checked source/manifest association and requested edges in the existing loader.

use purrdf_lex::allocation::{Admission, Memory, StorageError};
use purrdf_lex::walk::{try_equal_range_by, try_sort_unstable_by};

use super::{
    Caches, DependencyDeclaration, Found, ItemAdmission, LoadError, LoadErrorKind, LoadOptions,
    Meter, PackageId, PackageInfo, Reader, RelPath, Workspace, WorkspaceOf, workspace_of,
};
use crate::configuration::{self, ActiveConfiguration, DeclaredConfiguration};
use crate::manifest::{self, VersionDecl};
use crate::{
    CommittedGitTree, LockedCatalog, LockedPackageId, LockedSource, RegistryIndex,
    SourceAuthorityId, SourceTrees, Tree,
};

/// An exact declaration requested for source-qualified cross-package traversal.
#[derive(Debug, Clone, Copy)]
pub struct LockedEdge<'a> {
    key: &'a str,
    registry: Option<RegistryIndex<'a>>,
}

impl<'a> LockedEdge<'a> {
    /// Select a retained manifest key and any explicitly observed registry mapping.
    #[must_use]
    pub const fn new(key: &'a str, registry: Option<RegistryIndex<'a>>) -> Self {
        Self { key, registry }
    }
}

/// A candidate manifest/row association, verified by the locked load before use.
#[derive(Debug)]
pub struct LockedManifest<'a> {
    row: LockedPackageId,
    manifest: RelPath,
    edges: &'a [LockedEdge<'a>],
    configuration: Option<DeclaredConfiguration<'a>>,
}

impl<'a> LockedManifest<'a> {
    /// Associate a local manifest spelling with a row and explicitly requested edges.
    /// This input is a locator, not source or package identity proof.
    #[must_use]
    pub fn new(row: LockedPackageId, manifest: RelPath, edges: &'a [LockedEdge<'a>]) -> Self {
        Self {
            row,
            manifest,
            edges,
            configuration: None,
        }
    }
    /// Select a declared compilation context; every admitted package must supply one.
    #[must_use]
    pub const fn with_configuration(mut self, configuration: DeclaredConfiguration<'a>) -> Self {
        self.configuration = Some(configuration);
        self
    }
}

/// Candidate packages beneath one opaque checked Git repository snapshot.
pub struct GitPackageRoots<'a> {
    tree: &'a CommittedGitTree<'a>,
    packages: &'a [LockedManifest<'a>],
}

impl<'a> GitPackageRoots<'a> {
    /// Keep the repository root intact while selecting package manifests beneath it.
    #[must_use]
    pub const fn new(tree: &'a CommittedGitTree<'a>, packages: &'a [LockedManifest<'a>]) -> Self {
        Self { tree, packages }
    }
}

/// A source/manifest identity computed from a checked load, not from caller labels.
/// Provenance alone does not imply active configuration; the separate optional
/// configuration receipt records declared filtering, never provider policy.
#[derive(Debug, PartialEq, Eq)]
pub struct LockedBinding {
    row: LockedPackageId,
    name: String,
    version: String,
    source: Option<LockedSource>,
    configuration: Option<ActiveConfiguration>,
}

impl LockedBinding {
    /// Validated declared context, present only for a definition-filtered load.
    #[must_use]
    pub const fn configuration(&self) -> Option<&ActiveConfiguration> {
        self.configuration.as_ref()
    }
    /// The row in the catalog supplied to this load.
    #[must_use]
    pub const fn row(&self) -> LockedPackageId {
        self.row
    }
    /// The actual manifest's validated package name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// The actual explicit or inherited manifest version.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }
    /// Exact external source evidence, absent for explicitly associated local roots.
    #[must_use]
    pub const fn source(&self) -> Option<&LockedSource> {
        self.source.as_ref()
    }
}

struct Hint<'a> {
    row: LockedPackageId,
    manifest: RelPath,
    edges: &'a [LockedEdge<'a>],
    configuration: Option<DeclaredConfiguration<'a>>,
}
pub(super) struct BindingLoad<'a> {
    catalog: &'a LockedCatalog,
    hints: Vec<Hint<'a>>,
}

struct MemberIdentity {
    manifest: RelPath,
    name: String,
    version: Option<String>,
}
struct RepositoryMembers {
    root: RelPath,
    packages: Vec<MemberIdentity>,
}

impl Workspace {
    /// Load checked source/manifest associations and explicitly requested locked edges.
    ///
    /// Ordinary discovery, indexing and resolution are reused. Every discovered
    /// package must match one supplied manifest's actual name/version and source.
    /// A Git package must be the unique matching name/version in its checked
    /// root package or root workspace membership; other layouts refuse.
    /// Only requested, uniquely selected outgoing edges receive cross-package
    /// targets; other dependency declarations remain unresolved syntax.
    /// Target-conditional or optional requested edges need validated configuration
    /// through [`LockedManifest::with_configuration`]. Without it they refuse.
    /// This is not preferred-provider policy acceptance.
    ///
    /// # Errors
    /// Refuses wrong/missing/duplicate manifests or source identities, inconsistent
    /// versions, requested missing/ambiguous edges, unadmitted destinations and
    /// unresolved conditional activation, preserving resource and source causes.
    pub fn load_locked_sources(
        catalog: &LockedCatalog,
        first_party: &dyn Tree,
        local: &[LockedManifest<'_>],
        sources: &[GitPackageRoots<'_>],
        options: &LoadOptions,
    ) -> Result<Self, LoadError> {
        let meter = Meter::new(options);
        let path = RelPath::root();
        meter.items(catalog.usage().item_bytes, &path, None)?;
        let mut admission = ItemAdmission::new(&meter, &path);
        let mut views = Vec::new();
        Memory::new(&mut admission)
            .reserve(&mut views, sources.len())
            .map_err(|error| admission.error(error))?;
        for source in sources {
            views.push(source.tree.under_meter(&meter)?);
        }
        let mut trees: Vec<&dyn Tree> = Vec::new();
        Memory::new(&mut admission)
            .reserve(&mut trees, views.len())
            .map_err(|error| admission.error(error))?;
        for view in &views {
            trees.push(view);
        }
        let tree = SourceTrees::new(first_party, &trees);
        let mut binding = BindingLoad {
            catalog,
            hints: Vec::new(),
        };
        let mut roots = Vec::new();
        add_roots(
            &mut binding,
            &mut roots,
            local,
            SourceAuthorityId::FIRST_PARTY,
            None,
            &meter,
        )?;
        for (index, source) in sources.iter().enumerate() {
            add_roots(
                &mut binding,
                &mut roots,
                source.packages,
                SourceAuthorityId(index + 1),
                Some(source.tree.source()),
                &meter,
            )?;
        }
        try_sort_unstable_by(&mut binding.hints, |left, right| {
            meter.work(
                left.manifest
                    .as_str()
                    .len()
                    .saturating_add(right.manifest.as_str().len())
                    .saturating_add(1),
                &path,
                None,
            )?;
            Ok::<_, LoadError>(left.manifest.cmp(&right.manifest))
        })?;
        for pair in binding.hints.windows(2) {
            meter.work(1, &path, None)?;
            if pair[0].manifest == pair[1].manifest {
                return Err(failure(&path, "duplicate manifest association"));
            }
        }
        Self::load_meter(&tree, &roots, options, &meter, Some(&binding))
    }
}

fn add_roots<'a>(
    binding: &mut BindingLoad<'a>,
    roots: &mut Vec<RelPath>,
    packages: &'a [LockedManifest<'a>],
    authority: SourceAuthorityId,
    source: Option<&LockedSource>,
    meter: &Meter,
) -> Result<(), LoadError> {
    let path = RelPath::root();
    let mut admission = ItemAdmission::new(meter, &path);
    let mut memory = Memory::new(&mut admission);
    let result = (|| {
        for package in packages {
            let row = binding
                .catalog
                .package(package.row)
                .ok_or_else(|| failure(&package.manifest, "unknown lock row"))?;
            if row.source() != source {
                return Err(failure(
                    &package.manifest,
                    "manifest locator's source does not match the checked snapshot or local root",
                ));
            }
            if package.manifest.authority() != SourceAuthorityId::FIRST_PARTY
                || package.manifest.file_name() != Some("Cargo.toml")
            {
                return Err(failure(
                    &package.manifest,
                    "supply an unqualified Cargo.toml path inside the selected source",
                ));
            }
            let manifest = package.manifest.clone().in_source(authority);
            meter.items(
                crate::cost::shared_text(manifest.as_str().len()),
                &manifest,
                None,
            )?;
            let dir = manifest
                .parent()
                .ok_or_else(|| failure(&manifest, "manifest has no package directory"))?;
            memory.push(roots, dir).map_err(native)?;
            memory
                .push(
                    &mut binding.hints,
                    Hint {
                        row: package.row,
                        manifest,
                        edges: package.edges,
                        configuration: package.configuration,
                    },
                )
                .map_err(native)?;
        }
        Ok(())
    })();
    result.map_err(|error| recover(error, &mut admission))
}

pub(super) fn bind_found(
    binding: &BindingLoad<'_>,
    reader: &Reader<'_>,
    caches: &mut Caches,
) -> Result<Vec<LockedBinding>, LoadError> {
    let path = RelPath::root();
    let mut admission = ItemAdmission::new(reader.meter, &path);
    let mut memory = Memory::new(&mut admission);
    let result = (|| {
        if reader.found.len() != binding.hints.len() {
            return Err(failure(
                &path,
                "every discovered package needs exactly one explicit manifest association",
            ));
        }
        let mut output = Vec::new();
        let mut repositories = Vec::new();
        memory
            .reserve(&mut output, reader.found.len())
            .map_err(native)?;
        for found in reader.found {
            let hint = hint_for(binding, &found.package.manifest, reader.meter)?;
            let row = binding
                .catalog
                .package(hint.row)
                .ok_or_else(|| failure(&found.package.manifest, "associated row is missing"))?;
            let version_matches = match &found.declared.version {
                VersionDecl::Explicit(version) => version == row.version(),
                VersionDecl::Inherited => match workspace_of(reader, &found.package.dir, caches)? {
                    WorkspaceOf::Member {
                        version: Some(version),
                        ..
                    } => version.as_str() == row.version(),
                    _ => {
                        return Err(failure(
                            &found.package.manifest,
                            "inherited package version has no admitted workspace value",
                        ));
                    }
                },
                VersionDecl::Unset => {
                    return Err(failure(
                        &found.package.manifest,
                        "grounded identity requires an actual explicit or inherited package version",
                    ));
                }
            };
            if found.declared.name != row.name() || !version_matches {
                return Err(failure(
                    &found.package.manifest,
                    "actual manifest name/version does not match its exact lock row",
                ));
            }
            if row.source().is_some() {
                check_locator(
                    reader,
                    found,
                    row.name(),
                    row.version(),
                    caches,
                    &mut repositories,
                    &mut memory,
                )?;
            }
            let source = row
                .source()
                .map(|source| source.retained(&mut memory))
                .transpose()
                .map_err(native)?;
            let name = memory.string(row.name()).map_err(native)?;
            let version = memory.string(row.version()).map_err(native)?;
            output.push(LockedBinding {
                row: hint.row,
                name,
                version,
                source,
                configuration: None,
            });
        }
        Ok(output)
    })();
    result.map_err(|error| recover(error, &mut admission))
}

pub(super) fn install_edges(
    binding: &BindingLoad<'_>,
    found: &[Found],
    declarations: &[Vec<DependencyDeclaration>],
    bound: &[LockedBinding],
    infos: &mut [PackageInfo],
    meter: &Meter,
) -> Result<(), LoadError> {
    let path = RelPath::root();
    let mut admission = ItemAdmission::new(meter, &path);
    let mut rows = Vec::new();
    Memory::new(&mut admission)
        .reserve(&mut rows, bound.len())
        .map_err(|error| admission.error(error))?;
    rows.extend((0..bound.len()).map(PackageId));
    try_sort_unstable_by(&mut rows, |left, right| {
        meter.work(1, &path, None)?;
        Ok::<_, LoadError>(bound[left.index()].row.cmp(&bound[right.index()].row))
    })?;
    for pair in rows.windows(2) {
        meter.work(1, &path, None)?;
        if bound[pair[0].index()].row == bound[pair[1].index()].row {
            return Err(failure(
                &path,
                "one lock row cannot acquire multiple loaded package identities",
            ));
        }
    }
    for (index, info) in infos.iter_mut().enumerate() {
        let manifest = &found[index].package.manifest;
        let hint = hint_for(binding, manifest, meter)?;
        let mut matched = 0;
        for (declaration, dependency) in declarations[index].iter().zip(&mut info.dependencies) {
            let mut requested = None;
            for edge in hint.edges {
                meter.work(
                    edge.key
                        .len()
                        .saturating_add(declaration.key.len())
                        .saturating_add(1),
                    manifest,
                    None,
                )?;
                if edge.key == declaration.key {
                    if requested.is_some() {
                        return Err(failure(manifest, "duplicate requested declaration key"));
                    }
                    requested = Some(edge);
                }
            }
            let Some(edge) = requested else {
                dependency.package = None;
                continue;
            };
            matched += 1;
            eligible(
                declaration,
                bound[index].configuration.as_ref(),
                meter,
                manifest,
            )?;
            let selected = binding.catalog.select_under(
                bound[index].row,
                declaration,
                edge.registry,
                meter,
            )?;
            let matching = try_equal_range_by(&rows, |id| {
                meter.work(1, manifest, None)?;
                Ok::<_, LoadError>(bound[id.index()].row.cmp(&selected))
            })?;
            let target = rows
                .get(matching.start)
                .copied()
                .filter(|_| !matching.is_empty())
                .ok_or_else(|| {
                    failure(
                        manifest,
                        "selected outgoing row has no checked source/manifest package",
                    )
                })?;
            destination_configuration(index, target, declaration, bound, found, meter, manifest)?;
            let spec = declaration
                .inherited
                .as_ref()
                .map_or(&declaration.specification, |entry| &entry.specification);
            if spec.path.is_some() && dependency.package != Some(target) {
                return Err(failure(
                    manifest,
                    "declared path does not name the selected row's actual admitted manifest",
                ));
            }
            dependency.package = Some(target);
        }
        if matched != hint.edges.len() {
            return Err(failure(
                manifest,
                "a requested key is absent; use the exact retained declaration key",
            ));
        }
    }
    Ok(())
}

fn eligible(
    declaration: &DependencyDeclaration,
    active: Option<&ActiveConfiguration>,
    meter: &Meter,
    manifest: &RelPath,
) -> Result<(), LoadError> {
    let Some(active) = active else {
        if declaration.condition.is_some() || declaration.specification.optional == Some(true) {
            return Err(failure(
                manifest,
                "requested conditional/optional dependency needs admitted configuration",
            ));
        }
        return Ok(());
    };
    let allowed = match active.target_kind() {
        crate::TargetKind::BuildScript => declaration.kind == crate::DependencyKind::Build,
        crate::TargetKind::Lib => declaration.kind == crate::DependencyKind::Normal,
        _ => false,
    };
    if !allowed {
        return Err(failure(
            manifest,
            "requested dependency kind is not eligible for the selected compilation target",
        ));
    }
    if let Some(condition) = &declaration.condition
        && !configuration::condition(condition, active, meter, manifest)?
    {
        return Err(failure(
            manifest,
            "requested dependency is inactive under its declared target/host cfg",
        ));
    }
    if declaration.specification.optional == Some(true)
        && !configuration::contains(
            active.optional_dependencies(),
            &declaration.name,
            meter,
            manifest,
        )?
    {
        return Err(failure(
            manifest,
            "requested optional dependency is inactive under the actual feature graph",
        ));
    }
    Ok(())
}
fn destination_configuration(
    index: usize,
    target: PackageId,
    declaration: &DependencyDeclaration,
    bound: &[LockedBinding],
    found: &[Found],
    meter: &Meter,
    manifest: &RelPath,
) -> Result<(), LoadError> {
    let Some(active) = bound[index].configuration.as_ref() else {
        return Ok(());
    };
    let destination = bound[target.index()]
        .configuration
        .as_ref()
        .ok_or_else(|| failure(manifest, "destination has no active compilation context"))?;
    if destination.target_kind() != crate::TargetKind::Lib {
        return Err(failure(
            manifest,
            "dependency destination must select its actual library target",
        ));
    }
    let expected = active
        .selected()
        .ok_or_else(|| failure(manifest, "selected context lacks its host cfg"))?;
    if expected.triple() != destination.target().triple() {
        return Err(failure(
            manifest,
            "dependency destination has a different declared compilation target",
        ));
    }
    configuration::destination(
        active,
        Some(destination),
        declaration,
        &found[index].manifest.features,
        &found[target.index()].manifest.features,
        meter,
        manifest,
    )
}

pub(super) fn configure(
    binding: &BindingLoad<'_>,
    found: &[Found],
    declarations: &[Vec<DependencyDeclaration>],
    bound: &mut [LockedBinding],
    meter: &Meter,
) -> Result<bool, LoadError> {
    let active = binding
        .hints
        .iter()
        .any(|hint| hint.configuration.is_some());
    if !active {
        return Ok(false);
    }
    for (index, package) in found.iter().enumerate() {
        let hint = hint_for(binding, &package.package.manifest, meter)?;
        let input = hint.configuration.ok_or_else(|| {
            failure(
                &package.package.manifest,
                "a configured load needs explicit configuration for every admitted package",
            )
        })?;
        bound[index].configuration = Some(configuration::admit(
            input,
            &package.manifest.features,
            &declarations[index],
            meter,
            &package.package.manifest,
        )?);
    }
    Ok(true)
}

pub(super) fn select_targets(
    bound: &[LockedBinding],
    derived: &mut Vec<(&RelPath, crate::target::Derived)>,
    meter: &Meter,
) -> Result<(), LoadError> {
    let root = RelPath::root();
    for (index, proof) in bound.iter().enumerate() {
        let config = proof.configuration().ok_or_else(|| {
            configuration::failure(&root, "configured package has no checked context")
        })?;
        let mut count = 0;
        for (_, item) in derived.iter() {
            meter.work(
                item.target
                    .name
                    .len()
                    .saturating_add(config.target_name().len())
                    .saturating_add(1),
                &root,
                None,
            )?;
            if item.target.package_id == PackageId(index)
                && item.target.kind == config.target_kind()
                && item.target.name == config.target_name()
            {
                count += 1;
            }
        }
        if count != 1 {
            return Err(configuration::failure(
                &root,
                "configured target name/kind does not select exactly one actual package target",
            ));
        }
    }
    derived.retain(|(_, item)| {
        bound[item.target.package_id.index()]
            .configuration()
            .is_some_and(|config| {
                item.target.kind == config.target_kind() && item.target.name == config.target_name()
            })
    });
    Ok(())
}

fn check_locator<S: Admission + ?Sized>(
    reader: &Reader<'_>,
    found: &Found,
    name: &str,
    version: &str,
    caches: &mut Caches,
    repositories: &mut Vec<RepositoryMembers>,
    memory: &mut Memory<'_, S>,
) -> Result<(), LoadError> {
    let root = found.package.dir.source_root();
    let mut position = None;
    for (index, repository) in repositories.iter().enumerate() {
        reader.meter.work(1, &found.package.manifest, None)?;
        if repository.root == root {
            position = Some(index);
            break;
        }
    }
    let index = if let Some(index) = position {
        index
    } else {
        let packages = repository_members(reader, &root, caches, memory)?;
        let index = repositories.len();
        memory
            .push(repositories, RepositoryMembers { root, packages })
            .map_err(native)?;
        index
    };
    let mut matched = None;
    for package in &repositories[index].packages {
        reader.meter.work(
            package
                .name
                .len()
                .saturating_add(name.len())
                .saturating_add(1),
            &found.package.manifest,
            None,
        )?;
        if package.name != name {
            continue;
        }
        let actual = package.version.as_deref().ok_or_else(|| {
            failure(
                &package.manifest,
                "same-named repository package has no version evidence",
            )
        })?;
        reader.meter.work(
            actual.len().saturating_add(version.len()).saturating_add(1),
            &found.package.manifest,
            None,
        )?;
        if actual == version {
            if matched.is_some() {
                return Err(failure(
                    &found.package.manifest,
                    "ambiguous package name/version in the checked root workspace",
                ));
            }
            matched = Some(&package.manifest);
        }
    }
    if matched != Some(&found.package.manifest) {
        return Err(failure(
            &found.package.manifest,
            "manifest locator is not the unique matching package in the checked root package/workspace",
        ));
    }
    Ok(())
}

fn repository_members<S: Admission + ?Sized>(
    reader: &Reader<'_>,
    root: &RelPath,
    caches: &mut Caches,
    memory: &mut Memory<'_, S>,
) -> Result<Vec<MemberIdentity>, LoadError> {
    let root_manifest = root
        .join("Cargo.toml")
        .map_err(|error| failure(root, &error.to_string()))?;
    let parsed = read_manifest(reader, &root_manifest)?;
    let mut dirs = Vec::new();
    if parsed.has_workspace {
        let members = match caches.members.entry(root.clone()) {
            std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::btree_map::Entry::Vacant(entry) => entry.insert(super::members(
                reader.tree,
                reader.meter,
                root,
                &root_manifest,
                &parsed,
            )?),
        };
        memory.reserve(&mut dirs, members.len()).map_err(native)?;
        dirs.extend(members.iter().cloned());
    } else if parsed.package.is_some() {
        memory.push(&mut dirs, root.clone()).map_err(native)?;
    } else {
        return Err(failure(
            &root_manifest,
            "Git package binding needs a checked root package or root workspace",
        ));
    }
    let mut packages = Vec::new();
    memory.reserve(&mut packages, dirs.len()).map_err(native)?;
    for dir in dirs {
        reader.meter.items(
            crate::cost::shared_text(dir.as_str().len().saturating_add(11)),
            &root_manifest,
            None,
        )?;
        let manifest = dir
            .join("Cargo.toml")
            .map_err(|error| failure(&dir, &error.to_string()))?;
        let member = read_manifest(reader, &manifest)?;
        let package = member
            .package
            .as_ref()
            .ok_or_else(|| failure(&manifest, "workspace member has no package identity"))?;
        let version = match &package.version {
            VersionDecl::Explicit(version) => Some(version.as_str()),
            VersionDecl::Inherited => parsed.workspace_version.as_deref(),
            VersionDecl::Unset => None,
        }
        .map(|version| memory.string(version))
        .transpose()
        .map_err(native)?;
        let name = memory.string(&package.name).map_err(native)?;
        packages.push(MemberIdentity {
            manifest,
            name,
            version,
        });
    }
    Ok(packages)
}

fn read_manifest(reader: &Reader<'_>, path: &RelPath) -> Result<manifest::Manifest, LoadError> {
    let text = reader
        .tree
        .read(path, reader.meter.options().max_file_bytes)
        .map_err(|error| LoadError::tree(path, None, &crate::Origin::Discovery, error))?;
    reader.meter.read(path, &text)?;
    manifest::parse(&text).map_err(|error| {
        let origin = error
            .key
            .map_or(crate::Origin::Discovery, |key| crate::Origin::ManifestKey {
                key,
            });
        LoadError::new(
            path,
            None,
            LoadErrorKind::Manifest {
                message: error.message,
            },
        )
        .reached_by(&origin)
    })
}

fn hint_for<'a, 'b>(
    binding: &'a BindingLoad<'b>,
    path: &RelPath,
    meter: &Meter,
) -> Result<&'a Hint<'b>, LoadError> {
    let matching = try_equal_range_by(&binding.hints, |hint| {
        meter.work(
            hint.manifest
                .as_str()
                .len()
                .saturating_add(path.as_str().len())
                .saturating_add(1),
            path,
            None,
        )?;
        Ok::<_, LoadError>(hint.manifest.cmp(path))
    })?;
    binding
        .hints
        .get(matching.start)
        .filter(|_| !matching.is_empty())
        .ok_or_else(|| failure(path, "discovered manifest was not explicitly associated"))
}

fn failure(path: &RelPath, message: &str) -> LoadError {
    LoadError::new(
        path,
        None,
        LoadErrorKind::Locked {
            field: "source-binding",
            message: message.to_owned(),
        },
    )
}
fn native(error: StorageError) -> LoadError {
    LoadError::new(&RelPath::root(), None, LoadErrorKind::Storage(error))
}
fn recover(error: LoadError, admission: &mut ItemAdmission<'_>) -> LoadError {
    match error.kind {
        LoadErrorKind::Storage(error) => admission.error(error),
        _ => error,
    }
}
