// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Explicit locally available sources; no Cargo metadata, downloads or cache search.

mod namespace;

use super::refusal;
use crate::{Error, Ledger};
use purrdf_lex::allocation::{Memory, OrderedTable, Resident};
use raf_source::{
    CommittedGitTree, DependencyDeclaration, DiskTree, LoadOptions, LockedCatalog, LockedPackageId,
    LockedSourceKind, PackageId, ReferenceObservation, ReferenceRequest, RegistryIndex, RelPath,
    Tree, Workspace,
};
use std::path::{Path, PathBuf};

/// An explicit locally available dependency root, interpreted by its locked source.
/// Git rows use an installed repository and exact committed objects; other rows
/// use a contained disk tree. This locator never supplies package identity.
#[derive(Debug, Clone)]
pub struct ReferenceRoot {
    package: String,
    root: PathBuf,
    manifest: RelPath,
    registry: Option<(Option<String>, String)>,
}

impl ReferenceRoot {
    /// Locate a package manifest beneath an owned disk root or Git repository.
    #[must_use]
    pub fn new(package: impl Into<String>, root: impl Into<PathBuf>, manifest: RelPath) -> Self {
        Self {
            package: package.into(),
            root: root.into(),
            manifest,
            registry: None,
        }
    }

    /// Supply the explicit Cargo registry-name/index mapping used by selection.
    #[must_use]
    pub fn registry(mut self, name: Option<String>, source: impl Into<String>) -> Self {
        self.registry = Some((name, source.into()));
        self
    }
}

enum Origin {
    Tree,
    Disk(PathBuf),
    Git(PathBuf),
    Supplied,
}

pub(crate) struct Acquired {
    pub(crate) catalog: LockedCatalog,
    pub(crate) owners: Vec<(PackageId, ReferenceObservation)>,
    pub(crate) references: Vec<ReferenceObservation>,
    pub(crate) namespace: Option<namespace::Namespace>,
    origins: OrderedTable<LockedPackageId, Origin>,
    edges: Vec<NamespaceEdge>,
    remaining: LoadOptions,
}

struct NamespaceEdge {
    owner: LockedPackageId,
    key: String,
    registry: Option<(Option<String>, String)>,
}

impl NamespaceEdge {
    fn record(
        edges: &mut Vec<Self>,
        owner: LockedPackageId,
        key: &str,
        registry: Option<&(Option<String>, String)>,
        memory: &mut Memory<'_, Resident>,
    ) -> Result<(), Error> {
        let storage = |source| Error::SourceIndexStorage { source };
        let registry = if let Some((name, index)) = registry {
            Some((
                name.as_deref()
                    .map(|name| memory.string(name))
                    .transpose()
                    .map_err(storage)?,
                memory.string(index).map_err(storage)?,
            ))
        } else {
            None
        };
        let key = memory.string(key).map_err(storage)?;
        memory
            .push(
                edges,
                Self {
                    owner,
                    key,
                    registry,
                },
            )
            .map_err(storage)
    }
}

fn effective_path<'a>(
    declaration: &'a DependencyDeclaration,
    manifest: &'a RelPath,
) -> Option<(&'a RelPath, &'a str)> {
    declaration.inherited.as_ref().map_or_else(
        || {
            declaration
                .specification
                .path
                .as_deref()
                .map(|path| (manifest, path))
        },
        |inherited| {
            inherited
                .specification
                .path
                .as_deref()
                .map(|path| (&inherited.manifest, path))
        },
    )
}

fn package_manifest(base: &RelPath, path: &str) -> Result<RelPath, raf_source::PathError> {
    base.parent()
        .expect("Cargo.toml has a source-relative parent")
        .join(path)
        .and_then(|path| path.join("Cargo.toml"))
}

fn charge(options: &mut LoadOptions, input: &ReferenceObservation) -> Result<(), Error> {
    charge_usage(options, input.workspace().usage())
}

fn charge_usage(options: &mut LoadOptions, usage: raf_source::Usage) -> Result<(), Error> {
    for (remaining, used, name) in [
        (
            &mut options.max_total_bytes,
            usage.total_bytes,
            "source bytes",
        ),
        (&mut options.max_item_bytes, usage.item_bytes, "item bytes"),
        (&mut options.max_fact_bytes, usage.fact_bytes, "fact bytes"),
        (
            &mut options.max_glob_bytes,
            usage.glob_bytes,
            "resolver bytes",
        ),
        (&mut options.max_work, usage.work, "work"),
    ] {
        *remaining = remaining.checked_sub(used).ok_or_else(|| {
            refusal(format!(
                "cumulative reference acquisition exceeds its {name} allowance"
            ))
        })?;
    }
    Ok(())
}

fn work(options: &mut LoadOptions, amount: usize) -> Result<(), Error> {
    options.max_work = options.max_work.checked_sub(amount).ok_or_else(|| {
        refusal("reference dependency planning exceeds the cumulative work allowance")
    })?;
    Ok(())
}

fn disk_path(
    request: &ReferenceRequest<'_>,
    directory: PathBuf,
    locator: Option<&ReferenceRoot>,
    options: &LoadOptions,
) -> Result<(ReferenceObservation, Origin), Error> {
    let manifest = RelPath::new("Cargo.toml").expect("literal manifest");
    let (root, manifest) = if let Some(locator) = locator {
        let actual = directory
            .join("Cargo.toml")
            .canonicalize()
            .map_err(|error| {
                refusal(format!(
                    "cannot resolve declared path {}: {error}",
                    directory.display()
                ))
            })?;
        let supplied = locator
            .root
            .join(locator.manifest.as_str())
            .canonicalize()
            .map_err(|error| refusal(format!("cannot resolve supplied path manifest: {error}")))?;
        if actual != supplied {
            return Err(refusal(
                "supplied path manifest differs from the actual owning declaration",
            ));
        }
        (locator.root.clone(), locator.manifest.clone())
    } else {
        (directory, manifest)
    };
    let disk = DiskTree::new(&root).map_err(|source| Error::Tree {
        file: root.display().to_string(),
        source,
    })?;
    Ok((
        ReferenceObservation::load(request, &disk, &manifest, options)?,
        Origin::Disk(root),
    ))
}

fn same_locator(
    tree: &dyn Tree,
    host: Option<&Path>,
    left: (&Origin, &RelPath),
    right: (&Origin, &RelPath),
) -> Result<bool, Error> {
    if matches!((left.0, right.0), (Origin::Tree, Origin::Tree)) {
        // Opaque Trees expose file identities, not directory identities.
        // Hardlinked manifests cannot equate distinct package contexts.
        if left.1 != right.1 {
            return Ok(false);
        }
        let identity = |path: &RelPath| {
            tree.identity(path).map_err(|source| Error::Tree {
                file: path.to_string(),
                source,
            })
        };
        return Ok(identity(left.1)? == identity(right.1)?);
    }
    if let (Origin::Git(left_root), Origin::Git(right_root)) = (left.0, right.0) {
        return Ok(left_root == right_root && left.1 == right.1);
    }
    let physical = |(origin, manifest): (&Origin, &RelPath)| {
        let root = match origin {
            Origin::Tree => host
                .ok_or_else(|| refusal("cannot compare an opaque Tree path with a host locator"))?,
            Origin::Disk(root) => root.as_path(),
            _ => {
                return Err(refusal(
                    "path identity requires an actual owning source locator",
                ));
            }
        };
        root.join(manifest.as_str())
            .canonicalize()
            .map_err(|error| {
                refusal(format!(
                    "cannot resolve previously observed path manifest: {error}"
                ))
            })
    };
    Ok(physical(left)? == physical(right)?)
}

type Rows = OrderedTable<LockedPackageId, ()>;
type Reverse = OrderedTable<LockedPackageId, Vec<LockedPackageId>>;

#[derive(Clone, Copy)]
struct Inputs<'a, 'c> {
    ledger: &'a Ledger,
    catalog: &'c LockedCatalog,
    tree: &'a dyn Tree,
    host: Option<&'a Path>,
    roots: &'a [ReferenceRoot],
    supplied: &'a [ReferenceObservation],
}

struct Sources<'a> {
    owners: Vec<(PackageId, ReferenceObservation)>,
    references: Vec<ReferenceObservation>,
    borrowed: OrderedTable<LockedPackageId, &'a ReferenceObservation>,
    origins: OrderedTable<LockedPackageId, Origin>,
    loaded: Rows,
    queue: Vec<LockedPackageId>,
    edges: Vec<NamespaceEdge>,
}

fn reference_owners(
    workspace: &Workspace,
    inputs: &Inputs<'_, '_>,
    options: &mut LoadOptions,
    memory: &mut Memory<'_, Resident>,
) -> Result<Vec<(PackageId, ReferenceObservation)>, Error> {
    let Inputs { catalog, tree, .. } = *inputs;
    let storage = |source| Error::SourceIndexStorage { source };
    let mut owners = Vec::new();
    for package in workspace.package_ids() {
        let info = workspace
            .package(package)
            .ok_or_else(|| refusal("first-party package is absent"))?;
        if info.manifest.authority() != raf_source::SourceAuthorityId::FIRST_PARTY {
            continue;
        }
        let observed = ReferenceObservation::local_root(catalog, tree, &info.manifest, options)?;
        charge(options, &observed)?;
        memory
            .push(&mut owners, (package, observed))
            .map_err(storage)?;
    }
    Ok(owners)
}

fn forward_graph(
    inputs: &Inputs<'_, '_>,
    owners: &[(PackageId, ReferenceObservation)],
    options: &mut LoadOptions,
    memory: &mut Memory<'_, Resident>,
) -> Result<(Rows, Reverse), Error> {
    let Inputs {
        ledger, catalog, ..
    } = *inputs;
    let storage = |source| Error::SourceIndexStorage { source };
    let mut reachable = OrderedTable::default();
    let mut reverse = OrderedTable::<LockedPackageId, Vec<LockedPackageId>>::default();
    let mut pending = Vec::new();
    for (_, input) in owners {
        let mut is_reference = false;
        for name in ledger.reference() {
            work(
                options,
                name.len()
                    .saturating_add(input.name().len())
                    .saturating_add(1),
            )?;
            is_reference |= name == input.name();
        }
        if is_reference {
            continue;
        }
        let row = input.row();
        reachable.insert(row, (), memory).map_err(storage)?;
        memory.push(&mut pending, row).map_err(storage)?;
    }
    while let Some(row) = pending.pop() {
        work(options, 1)?;
        for dependency in catalog
            .package(row)
            .ok_or_else(|| refusal("missing lock row"))?
            .dependencies()
        {
            work(
                options,
                2_usize
                    .saturating_mul(reachable.len().bit_width() as usize)
                    .saturating_add(4),
            )?;
            if reverse.get(dependency).is_none() {
                reverse
                    .insert(*dependency, Vec::new(), memory)
                    .map_err(storage)?;
            }
            memory
                .push(
                    reverse.get_mut(dependency).expect("inserted dependency"),
                    row,
                )
                .map_err(storage)?;
            if reachable
                .insert(*dependency, (), memory)
                .map_err(storage)?
                .is_none()
            {
                memory.push(&mut pending, *dependency).map_err(storage)?;
            }
        }
    }
    Ok((reachable, reverse))
}

fn reference_ancestors(
    inputs: &Inputs<'_, '_>,
    owners: &[(PackageId, ReferenceObservation)],
    options: &mut LoadOptions,
    memory: &mut Memory<'_, Resident>,
) -> Result<Rows, Error> {
    let Inputs {
        ledger, catalog, ..
    } = *inputs;
    let storage = |source| Error::SourceIndexStorage { source };
    let (reachable, reverse) = forward_graph(inputs, owners, options, memory)?;
    let mut pending = Vec::new();
    let mut needed = OrderedTable::default();
    for name in ledger.reference() {
        let mut target = None;
        for row in catalog.packages() {
            work(
                options,
                row.name()
                    .len()
                    .saturating_add(name.len())
                    .saturating_add(2_usize.saturating_mul(reachable.len().bit_width() as usize))
                    .saturating_add(2),
            )?;
            if row.name() == name
                && reachable.get(&row.id()).is_some()
                && target.replace(row.id()).is_some()
            {
                return Err(refusal(format!(
                    "reference {name:?} is ambiguous in the lock graph"
                )));
            }
        }
        let target = target.ok_or_else(|| {
            refusal(format!(
                "reference {name:?} is absent from the owning dependency graph"
            ))
        })?;
        needed.insert(target, (), memory).map_err(storage)?;
        memory.push(&mut pending, target).map_err(storage)?;
    }
    // Only reachable owners enter reverse adjacency. Each selected ancestor
    // enters this worklist once, including in a cyclic dependency graph.
    while let Some(row) = pending.pop() {
        work(
            options,
            2_usize
                .saturating_mul(reverse.len().bit_width() as usize)
                .saturating_add(2),
        )?;
        if let Some(parents) = reverse.get(&row) {
            for parent in parents {
                work(
                    options,
                    2_usize
                        .saturating_mul(needed.len().bit_width() as usize)
                        .saturating_add(2),
                )?;
                if needed
                    .insert(*parent, (), memory)
                    .map_err(storage)?
                    .is_none()
                {
                    memory.push(&mut pending, *parent).map_err(storage)?;
                }
            }
        }
    }
    Ok(needed)
}

impl<'a> Sources<'a> {
    fn declarations(
        &self,
        owner: LockedPackageId,
        memory: &mut Memory<'_, Resident>,
    ) -> Result<(Vec<DependencyDeclaration>, RelPath), Error> {
        let Self {
            owners,
            references,
            borrowed,
            ..
        } = self;
        let storage = |source| Error::SourceIndexStorage { source };
        let input = owners
            .iter()
            .map(|(_, input)| input)
            .chain(references.iter())
            .find(|input| input.row() == owner)
            .or_else(|| borrowed.get(&owner).copied())
            .ok_or_else(|| refusal("acquired dependency owner is absent"))?;
        let declarations = memory
            .collect(
                input
                    .workspace()
                    .dependency_declarations(input.package())
                    .ok_or_else(|| refusal("dependency declarations are absent"))?
                    .iter()
                    .cloned(),
            )
            .map_err(storage)?;
        let owner_manifest = input
            .workspace()
            .package(input.package())
            .ok_or_else(|| refusal("dependency owner has no manifest"))?
            .manifest
            .clone();
        Ok((declarations, owner_manifest))
    }

    fn dependency(
        &mut self,
        owner: (LockedPackageId, &RelPath),
        declaration: &DependencyDeclaration,
        inputs: &Inputs<'a, '_>,
        needed: &Rows,
        options: &mut LoadOptions,
        memory: &mut Memory<'_, Resident>,
    ) -> Result<(), Error> {
        let (owner, owner_manifest) = owner;
        let Self {
            owners,
            references,
            borrowed,
            origins,
            loaded,
            queue,
            edges,
        } = self;
        let Inputs { tree, host, .. } = *inputs;
        let storage = |source| Error::SourceIndexStorage { source };

        let spec = declaration
            .inherited
            .as_ref()
            .map_or(&declaration.specification, |inherited| {
                &inherited.specification
            });
        let name = spec.package.as_deref().unwrap_or(&declaration.name);
        if !inputs.needs(owner, name, needed)? {
            return Ok(());
        }
        let (request, row, locator) = inputs.select(owner, declaration, name, options)?;
        if needed.get(&row).is_some() {
            NamespaceEdge::record(
                edges,
                owner,
                &declaration.key,
                locator.and_then(|root| root.registry.as_ref()),
                memory,
            )?;
        }
        if needed.get(&row).is_none()
            || (loaded.get(&row).is_some()
                && effective_path(declaration, owner_manifest).is_none()
                && locator.is_none())
        {
            return Ok(());
        }
        let supplied_match = inputs.supplied_observation(row, name)?;
        if locator.is_none()
            && effective_path(declaration, owner_manifest).is_none()
            && let Some(input) = supplied_match
        {
            // Caller-owned observations are borrowed by the final prepare
            // step; they can also supply the target directly without host I/O.
            origins
                .insert(row, Origin::Supplied, memory)
                .map_err(storage)?;
            charge(options, input)?;
            borrowed.insert(row, input, memory).map_err(storage)?;
            loaded.insert(row, (), memory).map_err(storage)?;
            memory.push(queue, row).map_err(storage)?;
            return Ok(());
        }
        let (observation, origin) = inputs.read(
            &request,
            row,
            locator,
            effective_path(declaration, owner_manifest),
            origins
                .get(&owner)
                .ok_or_else(|| refusal("owner source locator is absent"))?,
            options,
        )?;
        let existing = owners
            .iter()
            .map(|(_, input)| input)
            .chain(references.iter())
            .find(|input| input.row() == row)
            .or_else(|| borrowed.get(&row).copied());
        verify_existing(
            (tree, host),
            existing,
            origins.get(&row),
            (&observation, &origin),
        )?;
        loaded.insert(row, (), memory).map_err(storage)?;
        charge(options, &observation)?;
        memory.push(queue, row).map_err(storage)?;
        // A verified equivalent locator must not move an original consumer
        // into a dependency authority or change its original manifest base.
        if owners.iter().any(|(_, input)| input.row() == row) {
            return Ok(());
        }
        origins.insert(row, origin, memory).map_err(storage)?;
        if let Some(existing) = references.iter_mut().find(|input| input.row() == row) {
            *existing = observation;
        } else {
            memory.push(references, observation).map_err(storage)?;
        }
        Ok(())
    }
}

fn verify_existing(
    context: (&dyn Tree, Option<&Path>),
    existing: Option<&ReferenceObservation>,
    previous: Option<&Origin>,
    incoming: (&ReferenceObservation, &Origin),
) -> Result<(), Error> {
    let (tree, host) = context;
    let (observation, origin) = incoming;
    if let Some(existing) = existing {
        let old_manifest = &existing
            .workspace()
            .package(existing.package())
            .ok_or_else(|| refusal("previously observed package has no manifest"))?
            .manifest;
        let new_manifest = &observation
            .workspace()
            .package(observation.package())
            .ok_or_else(|| refusal("selected path package has no manifest"))?
            .manifest;
        if !same_locator(
            tree,
            host,
            (
                previous.ok_or_else(|| refusal("previous source locator is absent"))?,
                old_manifest,
            ),
            (origin, new_manifest),
        )? {
            return Err(refusal(format!(
                "actual dependency declaration for {:?} selects a different manifest {new_manifest} instead of {old_manifest} for an already observed lock row; use one package context for that exact identity",
                observation.name(),
            )));
        }
    }
    Ok(())
}

impl<'a> Inputs<'a, '_> {
    fn supplied_observation(
        &self,
        row: LockedPackageId,
        name: &str,
    ) -> Result<Option<&'a ReferenceObservation>, Error> {
        let locked = self
            .catalog
            .package(row)
            .ok_or_else(|| refusal("selected reference row is absent"))?;
        let mut matches = self.supplied.iter().filter(|input| {
            input.name() == locked.name()
                && input.version() == locked.version()
                && input.source() == locked.source()
        });
        let selected = matches.next();
        if matches.next().is_some() {
            return Err(refusal(format!(
                "multiple supplied observations of {name:?}"
            )));
        }
        Ok(selected)
    }

    fn needs(&self, owner: LockedPackageId, name: &str, needed: &Rows) -> Result<bool, Error> {
        Ok(self
            .catalog
            .package(owner)
            .ok_or_else(|| refusal("owner row is absent"))?
            .dependencies()
            .iter()
            .any(|id| {
                needed.get(id).is_some()
                    && self
                        .catalog
                        .package(*id)
                        .is_some_and(|row| row.name() == name)
            }))
    }

    fn select<'r>(
        &'r self,
        owner: LockedPackageId,
        declaration: &'r DependencyDeclaration,
        name: &str,
        options: &mut LoadOptions,
    ) -> Result<
        (
            ReferenceRequest<'r>,
            LockedPackageId,
            Option<&'r ReferenceRoot>,
        ),
        Error,
    > {
        let Self { roots, catalog, .. } = *self;
        let mut candidates = roots.iter().filter(|root| root.package == name);
        let locator = candidates.next();
        if candidates.next().is_some() {
            return Err(refusal(format!(
                "multiple explicit source locators for {name:?}"
            )));
        }
        let mut request = ReferenceRequest::new(catalog, owner, declaration);
        if let Some((registry, index)) = locator.and_then(|root| root.registry.as_ref()) {
            request = request.registry(RegistryIndex::new(registry.as_deref(), index));
        }
        let (row, selection) = request.select_with_usage(options)?;
        options.max_work = options
            .max_work
            .checked_sub(selection.work)
            .ok_or_else(|| {
                refusal("reference lock-row preselection exceeds the cumulative work allowance")
            })?;
        Ok((request, row, locator))
    }

    fn read(
        &self,
        request: &ReferenceRequest<'_>,
        row: LockedPackageId,
        locator: Option<&ReferenceRoot>,
        path: Option<(&RelPath, &str)>,
        origin: &Origin,
        options: &mut LoadOptions,
    ) -> Result<(ReferenceObservation, Origin), Error> {
        if let Some(locator) = locator
            && path.is_none()
        {
            return self.explicit(request, row, locator, options);
        }
        if let Some((base, path)) = path {
            return self.path(request, row, locator, (base, path), origin, options);
        }
        let name = self
            .catalog
            .package(row)
            .ok_or_else(|| refusal("selected reference row is absent"))?
            .name();
        Err(refusal(format!(
            "reference {name:?} is not available locally; supply its explicit source root, manifest and registry mapping"
        )))
    }

    fn explicit(
        &self,
        request: &ReferenceRequest<'_>,
        row: LockedPackageId,
        locator: &ReferenceRoot,
        options: &mut LoadOptions,
    ) -> Result<(ReferenceObservation, Origin), Error> {
        let locked = self
            .catalog
            .package(row)
            .ok_or_else(|| refusal("selected reference row is absent"))?;
        if locked
            .source()
            .is_some_and(|source| source.kind() == LockedSourceKind::Git)
        {
            let committed = CommittedGitTree::open(self.catalog, row, &locator.root, options)?;
            charge_usage(options, committed.usage())?;
            Ok((
                ReferenceObservation::committed(request, &committed, &locator.manifest, options)?,
                Origin::Git(locator.root.clone()),
            ))
        } else {
            let disk = DiskTree::new(&locator.root).map_err(|source| Error::Tree {
                file: locator.root.display().to_string(),
                source,
            })?;
            Ok((
                ReferenceObservation::load(request, &disk, &locator.manifest, options)?,
                Origin::Disk(locator.root.clone()),
            ))
        }
    }

    fn path(
        &self,
        request: &ReferenceRequest<'_>,
        row: LockedPackageId,
        locator: Option<&ReferenceRoot>,
        path: (&RelPath, &str),
        origin: &Origin,
        options: &mut LoadOptions,
    ) -> Result<(ReferenceObservation, Origin), Error> {
        let Self {
            catalog,
            tree,
            host,
            ..
        } = *self;
        let (base, path) = path;
        let name = catalog
            .package(row)
            .ok_or_else(|| refusal("selected reference row is absent"))?
            .name();
        Ok(match origin {
            Origin::Tree => match package_manifest(base, path) {
                Ok(manifest) => {
                    if locator.is_some() {
                        let root = host.ok_or_else(|| refusal("an explicit path locator cannot be verified against an opaque Tree; use the host gate"))?;
                        let directory = root
                            .join(manifest.as_str())
                            .parent()
                            .ok_or_else(|| refusal("declared path manifest has no directory"))?
                            .to_path_buf();
                        disk_path(request, directory, locator, options)?
                    } else {
                        (
                            ReferenceObservation::load(request, tree, &manifest, options)?,
                            Origin::Tree,
                        )
                    }
                }
                Err(
                    raf_source::PathError::AboveRoot { .. }
                    | raf_source::PathError::Absolute { .. },
                ) => {
                    let root = host.ok_or_else(|| refusal("path dependency leaves the supplied Tree; use the host gate with an explicit contained source root"))?;
                    let base = root
                        .join(base.as_str())
                        .parent()
                        .ok_or_else(|| refusal("host manifest has no parent"))?
                        .to_path_buf();
                    let directory = base.join(path);
                    disk_path(request, directory, locator, options)?
                }
                Err(error) => return Err(refusal(error.to_string())),
            },
            Origin::Disk(root) => {
                let directory = root
                    .join(base.as_str())
                    .parent()
                    .ok_or_else(|| refusal("disk manifest has no parent"))?
                    .join(path);
                disk_path(request, directory, locator, options)?
            }
            Origin::Git(root) => {
                let committed = CommittedGitTree::open(catalog, row, root, options)?;
                charge_usage(options, committed.usage())?;
                let manifest =
                    package_manifest(base, path).map_err(|error| refusal(error.to_string()))?;
                (
                    ReferenceObservation::committed(request, &committed, &manifest, options)?,
                    Origin::Git(root.clone()),
                )
            }
            Origin::Supplied => {
                return Err(refusal(format!(
                    "reference ancestor {name:?} needs an explicit source locator"
                )));
            }
        })
    }
}

pub(crate) fn acquire(
    ledger: &Ledger,
    workspace: &Workspace,
    tree: &dyn Tree,
    host: Option<&Path>,
    roots: &[ReferenceRoot],
    supplied: &[ReferenceObservation],
) -> Result<Acquired, Error> {
    let mut options = LoadOptions::default();
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    let storage = |source| Error::SourceIndexStorage { source };
    options.max_file_bytes = ledger.options().max_file_bytes;
    let catalog = LockedCatalog::load(
        tree,
        &RelPath::new("Cargo.lock").expect("literal lock path"),
        &options,
    )?;
    work(&mut options, catalog.usage().work)?;
    let inputs = Inputs {
        ledger,
        catalog: &catalog,
        tree,
        host,
        roots,
        supplied,
    };
    let owners = reference_owners(workspace, &inputs, &mut options, &mut memory)?;
    let needed = reference_ancestors(&inputs, &owners, &mut options, &mut memory)?;
    let mut loaded = OrderedTable::default();
    for (_, input) in &owners {
        loaded
            .insert(input.row(), (), &mut memory)
            .map_err(storage)?;
    }
    let references = Vec::new();
    let borrowed = OrderedTable::default();
    let mut origins = OrderedTable::default();
    let mut queue = Vec::new();
    for (_, input) in &owners {
        origins
            .insert(input.row(), Origin::Tree, &mut memory)
            .map_err(storage)?;
        memory.push(&mut queue, input.row()).map_err(storage)?;
    }
    let mut processed = OrderedTable::default();

    let mut sources = Sources {
        owners,
        references,
        borrowed,
        origins,
        loaded,
        queue,
        edges: Vec::new(),
    };
    while let Some(owner) = sources.queue.pop() {
        if processed
            .insert(owner, (), &mut memory)
            .map_err(storage)?
            .is_some()
        {
            continue;
        }
        let (declarations, manifest) = sources.declarations(owner, &mut memory)?;
        for declaration in declarations {
            sources.dependency(
                (owner, &manifest),
                &declaration,
                &inputs,
                &needed,
                &mut options,
                &mut memory,
            )?;
        }
    }
    Ok(Acquired {
        catalog,
        owners: sources.owners,
        references: sources.references,
        namespace: None,
        origins: sources.origins,
        edges: sources.edges,
        remaining: options,
    })
}
