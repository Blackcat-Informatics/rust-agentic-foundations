// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Checked source/manifest association and requested edges in the existing loader.

mod observed;
mod reference;
pub use observed::{ReferenceBinding, ReferencePackageRoots, ReferenceTree};
pub use reference::{ReferenceObservation, ReferenceRequest};

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
    configuration: Option<Configurations<'a>>,
}

#[derive(Debug, Clone, Copy)]
enum Configurations<'a> {
    One(DeclaredConfiguration<'a>),
    Many(&'a [DeclaredConfiguration<'a>]),
}

impl<'a> Configurations<'a> {
    fn iter(&self) -> impl Iterator<Item = DeclaredConfiguration<'a>> + '_ {
        let (one, many) = match self {
            Self::One(input) => (Some(*input), &[][..]),
            Self::Many(inputs) => (None, *inputs),
        };
        one.into_iter().chain(many.iter().copied())
    }
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
        self.configuration = Some(Configurations::One(configuration));
        self
    }
    /// Select exact target contexts, including any own-library companion.
    ///
    /// Each target name/kind occurs once. Incompatible modes of the same target
    /// must be loaded separately; no context is inferred from this host.
    #[must_use]
    pub const fn with_configurations(
        mut self,
        configurations: &'a [DeclaredConfiguration<'a>],
    ) -> Self {
        self.configuration = Some(Configurations::Many(configurations));
        self
    }
}

/// Candidate packages beneath one opaque checked Git repository snapshot.
pub struct GitPackageRoots<'a> {
    tree: &'a CommittedGitTree<'a>,
    packages: &'a [LockedManifest<'a>],
}

#[derive(Clone, Copy)]
enum Sources<'a> {
    Configured(&'a [GitPackageRoots<'a>]),
    Observed(&'a [ReferencePackageRoots<'a>]),
}

impl<'a> Sources<'a> {
    fn len(&self) -> usize {
        match self {
            Self::Configured(sources) => sources.len(),
            Self::Observed(sources) => sources.len(),
        }
    }

    fn iter(&self) -> impl Iterator<Item = ReferencePackageRoots<'a>> + Clone {
        let (configured, observed) = match self {
            Self::Configured(sources) => (*sources, &[][..]),
            Self::Observed(sources) => (&[][..], *sources),
        };
        configured
            .iter()
            .map(|source| {
                ReferencePackageRoots::new(ReferenceTree::Git(source.tree), source.packages)
            })
            .chain(observed.iter().copied())
    }
}

#[derive(Clone, Copy)]
enum Purpose<'a> {
    Configured,
    Structural(&'a [ReferenceTree<'a>]),
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
    configurations: Vec<ActiveConfiguration>,
}

impl LockedBinding {
    /// Validated declared context, present only for a definition-filtered load.
    #[must_use]
    pub const fn configuration(&self) -> Option<&ActiveConfiguration> {
        match self.configurations.as_slice() {
            [only] => Some(only),
            _ => None,
        }
    }
    /// Every explicitly selected target context, empty for an unconfigured load.
    #[must_use]
    pub fn configurations(&self) -> &[ActiveConfiguration] {
        &self.configurations
    }
    /// The exact selected context for an actual target kind and Cargo name.
    #[must_use]
    pub fn configuration_for(
        &self,
        kind: crate::TargetKind,
        name: &str,
    ) -> Option<&ActiveConfiguration> {
        self.configurations
            .iter()
            .find(|config| config.target_kind() == kind && config.target_name() == name)
    }
    pub(crate) fn configuration_for_under(
        &self,
        kind: crate::TargetKind,
        name: &str,
        meter: &Meter,
        path: &RelPath,
    ) -> Result<Option<&ActiveConfiguration>, LoadError> {
        for config in &self.configurations {
            meter.work(
                config
                    .target_name()
                    .len()
                    .saturating_add(name.len())
                    .saturating_add(1),
                path,
                None,
            )?;
            if config.target_kind() == kind && config.target_name() == name {
                return Ok(Some(config));
            }
        }
        Ok(None)
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
    configuration: Option<Configurations<'a>>,
}
pub(super) struct BindingLoad<'a> {
    catalog: &'a LockedCatalog,
    hints: Vec<Hint<'a>>,
    purpose: Purpose<'a>,
}

impl BindingLoad<'_> {
    pub(super) fn structural(&self) -> bool {
        matches!(self.purpose, Purpose::Structural(_))
    }
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
        Self::load_locked_with(
            catalog,
            ReferenceTree::Opaque(first_party),
            None,
            local,
            Sources::Configured(sources),
            options,
        )
    }

    /// Load all first-party packages discovered beneath explicit directory roots.
    ///
    /// Uses the same checked manifests, sealed Git sources, budgets and index as
    /// [`Self::load_locked_sources`]. Local manifest locators do not narrow the
    /// discovery roots: every discovered package must have exactly one matching
    /// locked association. External Git roots retain their own source authority.
    ///
    /// # Errors
    /// Refuses empty or externally qualified first-party roots, omitted or extra
    /// manifest associations, and every failure of [`Self::load_locked_sources`].
    pub fn load_locked_sources_under(
        catalog: &LockedCatalog,
        first_party: &dyn Tree,
        first_party_roots: &[RelPath],
        local: &[LockedManifest<'_>],
        sources: &[GitPackageRoots<'_>],
        options: &LoadOptions,
    ) -> Result<Self, LoadError> {
        if first_party_roots.is_empty()
            || first_party_roots
                .iter()
                .any(|root| root.authority() != SourceAuthorityId::FIRST_PARTY)
        {
            return Err(failure(
                &RelPath::root(),
                "supply nonempty first-party discovery roots",
            ));
        }
        Self::load_locked_with(
            catalog,
            ReferenceTree::Opaque(first_party),
            Some(first_party_roots),
            local,
            Sources::Configured(sources),
            options,
        )
    }

    fn load_locked_with(
        catalog: &LockedCatalog,
        first_party: ReferenceTree<'_>,
        first_party_roots: Option<&[RelPath]>,
        local: &[LockedManifest<'_>],
        sources: Sources<'_>,
        options: &LoadOptions,
    ) -> Result<Self, LoadError> {
        let meter = Meter::new(options);
        let path = RelPath::root();
        meter.items(catalog.usage().item_bytes, &path, None)?;
        let mut admission = ItemAdmission::new(&meter, &path);
        let result = (|| {
            let mut memory = Memory::new(&mut admission);
            let first_view = match first_party {
                ReferenceTree::Git(tree) => Some(tree.under_meter(&meter)?),
                ReferenceTree::Opaque(_) | ReferenceTree::Disk(_) => None,
            };
            let mut views = Vec::new();
            memory.reserve(&mut views, sources.len()).map_err(native)?;
            for source in sources.iter() {
                views.push(match source.tree {
                    ReferenceTree::Git(tree) => Some(tree.under_meter(&meter)?),
                    ReferenceTree::Opaque(_) | ReferenceTree::Disk(_) => None,
                });
            }
            let mut trees: Vec<&dyn Tree> = Vec::new();
            memory.reserve(&mut trees, views.len()).map_err(native)?;
            for (source, view) in sources.iter().zip(&views) {
                trees.push(
                    view.as_ref()
                        .map_or_else(|| source.tree.tree(), |view| view as &dyn Tree),
                );
            }
            let first = first_view
                .as_ref()
                .map_or_else(|| first_party.tree(), |view| view as &dyn Tree);
            let tree = SourceTrees::new(first, &trees);
            let mut contexts = Vec::new();
            if matches!(sources, Sources::Observed(_)) {
                memory
                    .reserve(&mut contexts, sources.len().saturating_add(1))
                    .map_err(native)?;
                contexts.push(first_party);
                contexts.extend(sources.iter().map(|source| source.tree));
            }
            let mut binding = BindingLoad {
                catalog,
                hints: Vec::new(),
                purpose: match sources {
                    Sources::Configured(_) => Purpose::Configured,
                    Sources::Observed(_) => Purpose::Structural(&contexts),
                },
            };
            let mut roots = Vec::new();
            add_roots(
                &mut binding,
                &mut roots,
                local,
                SourceAuthorityId::FIRST_PARTY,
                first_party.source(),
                &meter,
                &mut memory,
            )?;
            if let Some(selected) = first_party_roots {
                roots.clear();
                for root in selected {
                    meter.items(crate::cost::shared_text(root.as_str().len()), root, None)?;
                }
                for root in selected {
                    memory.push(&mut roots, root.clone()).map_err(native)?;
                }
            }
            for (index, source) in sources.iter().enumerate() {
                add_roots(
                    &mut binding,
                    &mut roots,
                    source.packages,
                    SourceAuthorityId(index + 1),
                    source.tree.source(),
                    &meter,
                    &mut memory,
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
        })();
        result.map_err(|error| recover(error, &mut admission))
    }
}

fn add_roots<'a>(
    binding: &mut BindingLoad<'a>,
    roots: &mut Vec<RelPath>,
    packages: &'a [LockedManifest<'a>],
    authority: SourceAuthorityId,
    source: Option<&LockedSource>,
    meter: &Meter,
    memory: &mut Memory<'_, impl Admission>,
) -> Result<(), LoadError> {
    for package in packages {
        let row = binding
            .catalog
            .package(package.row)
            .ok_or_else(|| failure(&package.manifest, "unknown lock row"))?;
        let observed_contents = binding.structural()
            && source.is_none()
            && row
                .source()
                .is_some_and(|source| source.kind() != crate::LockedSourceKind::Git);
        if row.source() != source && !observed_contents {
            return Err(failure(
                &package.manifest,
                "manifest locator's source does not match the checked snapshot or local root",
            ));
        }
        if binding.structural() && package.configuration.is_some() {
            return Err(failure(
                &package.manifest,
                "structural reference inputs must not supply configured target evidence",
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
                configurations: Vec::new(),
            });
        }
        Ok(output)
    })();
    result.map_err(|error| recover(error, &mut admission))
}

pub(super) fn install_edges(
    binding: &BindingLoad<'_>,
    reader: &Reader<'_>,
    declarations: &[Vec<DependencyDeclaration>],
    bound: &[LockedBinding],
    targets: &[crate::Target],
    infos: &mut [PackageInfo],
) -> Result<(), LoadError> {
    let found = reader.found;
    let meter = reader.meter;
    let rows = bound_rows(bound, meter)?;
    for (index, info) in infos.iter_mut().enumerate() {
        info.configured = !bound[index].configurations.is_empty();
        let manifest = &found[index].package.manifest;
        let hint = hint_for(binding, manifest, meter)?;
        let mut matched = 0;
        for (declaration, dependency) in declarations[index].iter().zip(&mut info.dependencies) {
            let requested = requested_edge(hint.edges, &declaration.key, meter, manifest)?;
            let Some(edge) = requested else {
                if !binding.structural() {
                    dependency.package = None;
                }
                continue;
            };
            matched += 1;
            let active_targets = if binding.structural() {
                None
            } else {
                eligible_targets(
                    PackageId(index),
                    &bound[index],
                    targets,
                    declaration,
                    meter,
                    manifest,
                )?
            };
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
            for source in active_targets.iter().flatten() {
                let selected = &targets[source.index()];
                let active = bound[index]
                    .configuration_for_under(selected.kind, &selected.name, meter, manifest)?
                    .ok_or_else(|| {
                        failure(manifest, "selected target has no admitted configuration")
                    })?;
                destination_configuration(
                    (index, target),
                    active,
                    declaration,
                    bound,
                    found,
                    meter,
                    manifest,
                )?;
            }
            dependency.active_targets = active_targets;
            let spec = declaration
                .inherited
                .as_ref()
                .map_or(&declaration.specification, |entry| &entry.specification);
            if spec.path.is_some() && dependency.package != Some(target) {
                if dependency.package.is_some() {
                    return Err(failure(
                        manifest,
                        "declared path does not name the selected row's actual admitted manifest",
                    ));
                }
                binding.check_observed_path(
                    reader,
                    declaration,
                    manifest,
                    &found[target.index()].package.manifest,
                )?;
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

fn requested_edge<'a>(
    edges: &[LockedEdge<'a>],
    key: &str,
    meter: &Meter,
    manifest: &RelPath,
) -> Result<Option<LockedEdge<'a>>, LoadError> {
    let mut requested = None;
    for edge in edges {
        meter.work(
            edge.key.len().saturating_add(key.len()).saturating_add(1),
            manifest,
            None,
        )?;
        if edge.key == key && requested.replace(*edge).is_some() {
            return Err(failure(manifest, "duplicate requested declaration key"));
        }
    }
    Ok(requested)
}

fn bound_rows(bound: &[LockedBinding], meter: &Meter) -> Result<Vec<PackageId>, LoadError> {
    let path = RelPath::root();
    let mut admission = ItemAdmission::new(meter, &path);
    let mut rows = Vec::new();
    Memory::new(&mut admission)
        .reserve(&mut rows, bound.len())
        .map_err(|error| admission.error(error))?;
    meter.work(bound.len(), &path, None)?;
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
    Ok(rows)
}

fn eligible_targets(
    package: PackageId,
    proof: &LockedBinding,
    targets: &[crate::Target],
    declaration: &DependencyDeclaration,
    meter: &Meter,
    manifest: &RelPath,
) -> Result<Option<Vec<crate::TargetId>>, LoadError> {
    if proof.configurations.is_empty() {
        eligible(declaration, None, meter, manifest)?;
        return Ok(None);
    }
    let mut admission = ItemAdmission::new(meter, manifest);
    let result = (|| {
        let mut memory = Memory::new(&mut admission);
        let mut selected = Vec::new();
        for (position, target) in targets.iter().enumerate() {
            meter.work(1, manifest, None)?;
            if target.package_id != package {
                continue;
            }
            let active = proof
                .configuration_for_under(target.kind, &target.name, meter, manifest)?
                .ok_or_else(|| {
                    failure(manifest, "selected target has no admitted configuration")
                })?;
            if eligible(declaration, Some(active), meter, manifest)? {
                memory
                    .push(&mut selected, crate::TargetId(position))
                    .map_err(native)?;
            }
        }
        if selected.is_empty() {
            return Err(failure(
                manifest,
                "requested dependency is inactive or ineligible in every selected target context",
            ));
        }
        Ok(Some(selected))
    })();
    result.map_err(|error| recover(error, &mut admission))
}

fn eligible(
    declaration: &DependencyDeclaration,
    active: Option<&ActiveConfiguration>,
    meter: &Meter,
    manifest: &RelPath,
) -> Result<bool, LoadError> {
    let Some(active) = active else {
        if declaration.condition.is_some() || declaration.specification.optional == Some(true) {
            return Err(failure(
                manifest,
                "requested conditional/optional dependency needs admitted configuration",
            ));
        }
        return Ok(true);
    };
    let allowed = match active.target_kind() {
        crate::TargetKind::BuildScript => declaration.kind == crate::DependencyKind::Build,
        crate::TargetKind::Lib | crate::TargetKind::Bin => {
            declaration.kind == crate::DependencyKind::Normal
                || (declaration.kind == crate::DependencyKind::Dev
                    && configuration::evaluate(
                        &crate::cfg::CfgExpr::Test,
                        active,
                        false,
                        meter,
                        manifest,
                    )?)
        }
        crate::TargetKind::Test | crate::TargetKind::Bench | crate::TargetKind::Example => {
            matches!(
                declaration.kind,
                crate::DependencyKind::Normal | crate::DependencyKind::Dev
            )
        }
    };
    if !allowed {
        return Ok(false);
    }
    if let Some(condition) = &declaration.condition
        && !configuration::condition(condition, active, meter, manifest)?
    {
        return Ok(false);
    }
    Ok(declaration.specification.optional != Some(true)
        || configuration::contains(
            active.optional_dependencies(),
            &declaration.name,
            meter,
            manifest,
        )?)
}
fn destination_configuration(
    endpoints: (usize, PackageId),
    active: &ActiveConfiguration,
    declaration: &DependencyDeclaration,
    bound: &[LockedBinding],
    found: &[Found],
    meter: &Meter,
    manifest: &RelPath,
) -> Result<(), LoadError> {
    let (index, target) = endpoints;
    meter.work(bound[target.index()].configurations.len(), manifest, None)?;
    let mut libraries = bound[target.index()]
        .configurations
        .iter()
        .filter(|config| config.target_kind() == crate::TargetKind::Lib);
    let destination = libraries.next().ok_or_else(|| {
        failure(
            manifest,
            "dependency destination must select its actual library target",
        )
    })?;
    if libraries.next().is_some() {
        return Err(failure(
            manifest,
            "dependency destination has ambiguous library configurations",
        ));
    }
    let expected = active
        .selected()
        .ok_or_else(|| failure(manifest, "selected context lacks its host cfg"))?;
    meter.work(
        expected
            .triple()
            .len()
            .saturating_add(destination.target().triple().len())
            .saturating_add(1),
        manifest,
        None,
    )?;
    if expected.triple() != destination.target().triple() {
        return Err(failure(
            manifest,
            "dependency destination has a different declared compilation target",
        ));
    }
    if configuration::evaluate(
        &crate::cfg::CfgExpr::Test,
        destination,
        false,
        meter,
        manifest,
    )? {
        return Err(failure(
            manifest,
            "dependency library must use its ordinary context; load unit-test modes separately",
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
        let mut admission = ItemAdmission::new(meter, &package.package.manifest);
        let result = (|| {
            let mut memory = Memory::new(&mut admission);
            for input in input.iter() {
                let active = configuration::admit(
                    input,
                    &package.manifest.features,
                    &declarations[index],
                    meter,
                    &package.package.manifest,
                )?;
                for known in &bound[index].configurations {
                    meter.work(
                        known
                            .target_name()
                            .len()
                            .saturating_add(active.target_name().len())
                            .saturating_add(1),
                        &package.package.manifest,
                        None,
                    )?;
                    if known.target_kind() == active.target_kind()
                        && known.target_name() == active.target_name()
                    {
                        return Err(failure(
                            &package.package.manifest,
                            "duplicate target configuration; load incompatible modes separately",
                        ));
                    }
                }
                memory
                    .push(&mut bound[index].configurations, active)
                    .map_err(native)?;
            }
            Ok(())
        })();
        result.map_err(|error| recover(error, &mut admission))?;
        if bound[index].configurations.is_empty() {
            return Err(failure(
                &package.package.manifest,
                "a configured package needs at least one explicit target context",
            ));
        }
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
        for config in proof.configurations() {
            let mut selected = None;
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
                    if selected.is_some() {
                        return Err(configuration::failure(
                            &root,
                            "configured target name/kind is ambiguous",
                        ));
                    }
                    selected = Some(&item.target);
                }
            }
            let selected = selected.ok_or_else(|| {
                configuration::failure(
                    &root,
                    "configured target name/kind does not select exactly one actual package target",
                )
            })?;
            for feature in &selected.required_features {
                if !configuration::contains(config.features(), feature, meter, &selected.root)? {
                    return Err(configuration::failure(
                        &selected.root,
                        "selected target has an unsatisfied required feature",
                    ));
                }
            }
            if !matches!(
                config.target_kind(),
                crate::TargetKind::Lib | crate::TargetKind::BuildScript
            ) {
                check_companions(PackageId(index), proof, config, derived, meter)?;
            }
        }
    }
    let mut selection = Ok(());
    derived.retain(|(_, item)| {
        if selection.is_err() {
            return false;
        }
        match bound[item.target.package_id.index()].configuration_for_under(
            item.target.kind,
            &item.target.name,
            meter,
            &root,
        ) {
            Ok(context) => context.is_some(),
            Err(error) => {
                selection = Err(error);
                false
            }
        }
    });
    selection
}

fn check_companions(
    package: PackageId,
    proof: &LockedBinding,
    config: &ActiveConfiguration,
    derived: &[(&RelPath, crate::target::Derived)],
    meter: &Meter,
) -> Result<(), LoadError> {
    let root = RelPath::root();
    meter.work(derived.len(), &root, None)?;
    for (_, library) in derived.iter().filter(|(_, item)| {
        item.target.package_id == package && item.target.kind == crate::TargetKind::Lib
    }) {
        meter.work(1, &root, None)?;
        let companion = proof
            .configuration_for_under(crate::TargetKind::Lib, &library.target.name, meter, &root)?
            .ok_or_else(|| {
                failure(
                    &root,
                    "selected target requires an explicit own-library companion configuration",
                )
            })?;
        meter.work(
            config
                .target()
                .triple()
                .len()
                .saturating_add(companion.target().triple().len())
                .saturating_add(1),
            &root,
            None,
        )?;
        if config.target().triple() != companion.target().triple()
            || !config.same_features_under(companion, meter, &root)?
            || configuration::evaluate(&crate::cfg::CfgExpr::Test, companion, false, meter, &root)?
        {
            return Err(failure(
                &root,
                "own-library companion needs the same declared target and unified features in its ordinary mode",
            ));
        }
    }
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
                source: error.source,
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
            source: None,
        },
    )
}
fn native(error: StorageError) -> LoadError {
    LoadError::new(&RelPath::root(), None, LoadErrorKind::Storage(error))
}
fn recover(error: LoadError, admission: &mut ItemAdmission<'_>) -> LoadError {
    admission.recover(error)
}
