// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! One structural resolver view over the actual acquired source contexts.

use super::{Acquired, Origin, refusal, work};
use crate::Error;
use purrdf_lex::allocation::{Memory, OrderedTable, Resident};
use raf_source::{
    CommittedGitTree, DiskTree, LoadOptions, LockedEdge, LockedManifest, LockedPackageId,
    PackageId, ReferenceObservation, ReferencePackageRoots, ReferenceTree, RegistryIndex, RelPath,
    Tree, Workspace,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(crate) struct Namespace {
    pub(crate) workspace: Workspace,
    pub(crate) consumers: BTreeSet<PackageId>,
}

enum Backend<'a> {
    Disk { root: PathBuf, tree: DiskTree },
    Git(CommittedGitTree<'a>),
}

impl Backend<'_> {
    fn view(&self) -> ReferenceTree<'_> {
        match self {
            Self::Disk { tree, .. } => ReferenceTree::Disk(tree),
            Self::Git(tree) => ReferenceTree::Git(tree),
        }
    }
}

struct Package<'a> {
    row: LockedPackageId,
    manifest: RelPath,
    backend: Option<usize>,
    edges: Vec<LockedEdge<'a>>,
    first_party: bool,
}

fn local_manifests<'a>(
    packages: &'a [Package<'_>],
    roots: &[&str],
    memory: &mut Memory<'_, Resident>,
) -> Result<(Vec<LockedManifest<'a>>, Vec<RelPath>), Error> {
    let mut manifests = Vec::new();
    let mut selected_roots = Vec::new();
    for root in roots {
        let root = RelPath::new(root).map_err(|error| refusal(error.to_string()))?;
        memory.push(&mut selected_roots, root).map_err(storage)?;
    }
    for package in packages.iter().filter(|package| package.backend.is_none()) {
        memory
            .push(
                &mut manifests,
                LockedManifest::new(package.row, package.manifest.clone(), &package.edges),
            )
            .map_err(storage)?;
        let directory = package
            .manifest
            .parent()
            .ok_or_else(|| refusal("reference manifest has no directory"))?;
        if !package.first_party && !selected_roots.contains(&directory) {
            memory
                .push(&mut selected_roots, directory)
                .map_err(storage)?;
        }
    }
    Ok((manifests, selected_roots))
}

fn disk(root: &Path) -> Result<DiskTree, Error> {
    DiskTree::new(root).map_err(|source| Error::Tree {
        file: root.display().to_string(),
        source,
    })
}

impl Acquired {
    pub(crate) fn prepare_namespace(
        &mut self,
        original: &dyn Tree,
        host: Option<&Path>,
        roots: &[&str],
    ) -> Result<(), Error> {
        let namespace = self.load_namespace(original, host, roots)?;
        // Namespace identities and reference bodies now come from the same
        // fresh load. Earlier acquisition parses must not supply stale bodies.
        self.owners.clear();
        self.references.clear();
        self.origins = OrderedTable::default();
        self.edges.clear();
        self.namespace = Some(namespace);
        Ok(())
    }

    fn load_namespace(
        &self,
        original: &dyn Tree,
        host: Option<&Path>,
        roots: &[&str],
    ) -> Result<Namespace, Error> {
        let mut options = self.remaining.clone();
        for (_, origin) in self.origins.ordered() {
            if matches!(origin, Origin::Supplied) {
                return Err(refusal(
                    "reference namespace lookup needs the actual supplied source tree; provide an explicit reference root",
                ));
            }
        }
        let first_root = host
            .map(|path| {
                path.canonicalize().map_err(|error| {
                    refusal(format!("cannot resolve first-party source root: {error}"))
                })
            })
            .transpose()?;
        let first = first_root.as_deref().map(disk).transpose()?;
        let mut resident = Resident;
        let mut memory = Memory::new(&mut resident);
        let mut backends = Vec::new();
        let mut packages = Vec::new();
        let mut owner_rows = OrderedTable::default();
        for (input, first_party) in self
            .owners
            .iter()
            .map(|(_, input)| (input, true))
            .chain(self.references.iter().map(|input| (input, false)))
        {
            let package = self.namespace_package(
                input,
                first_party,
                first_root.as_deref(),
                &mut backends,
                &mut options,
                &mut memory,
            )?;
            if first_party {
                owner_rows
                    .insert(input.row(), (), &mut memory)
                    .map_err(storage)?;
            }
            memory.push(&mut packages, package).map_err(storage)?;
        }
        let (local, selected_roots) = local_manifests(&packages, roots, &mut memory)?;
        let mut manifests = Vec::new();
        for index in 0..backends.len() {
            let mut group = Vec::new();
            for package in packages
                .iter()
                .filter(|package| package.backend == Some(index))
            {
                memory
                    .push(
                        &mut group,
                        LockedManifest::new(package.row, package.manifest.clone(), &package.edges),
                    )
                    .map_err(storage)?;
            }
            memory.push(&mut manifests, group).map_err(storage)?;
        }
        let sources =
            memory
                .collect(backends.iter().zip(&manifests).map(|(backend, packages)| {
                    ReferencePackageRoots::new(backend.view(), packages)
                }))
                .map_err(storage)?;
        let first = first
            .as_ref()
            .map_or(ReferenceTree::Opaque(original), ReferenceTree::Disk);
        let workspace = Workspace::load_reference_sources_under(
            &self.catalog,
            first,
            &selected_roots,
            &local,
            &sources,
            &options,
        )?;
        let mut consumers = BTreeSet::new();
        for package in workspace.package_ids() {
            let binding = workspace
                .reference_binding(package)
                .ok_or_else(|| refusal("structural namespace package has no reference identity"))?;
            if owner_rows.get(&binding.row()).is_some() {
                consumers.insert(package);
            }
        }
        Ok(Namespace {
            workspace,
            consumers,
        })
    }

    fn namespace_package<'a>(
        &'a self,
        input: &ReferenceObservation,
        first_party: bool,
        first: Option<&Path>,
        backends: &mut Vec<Backend<'a>>,
        options: &mut LoadOptions,
        memory: &mut Memory<'_, Resident>,
    ) -> Result<Package<'a>, Error> {
        let origin = self
            .origins
            .get(&input.row())
            .ok_or_else(|| refusal("reference source context is absent"))?;
        let backend = match origin {
            Origin::Tree => None,
            Origin::Disk(root) => {
                let root = root.canonicalize().map_err(|error| {
                    refusal(format!("cannot resolve reference source root: {error}"))
                })?;
                let same_first = first == Some(root.as_path());
                if same_first {
                    None
                } else {
                    let mut existing = None;
                    for (index, source) in backends.iter().enumerate() {
                        work(options, 1)?;
                        if let Backend::Disk {
                            root: existing_root,
                            ..
                        } = source
                            && *existing_root == root
                        {
                            existing = Some(index);
                            break;
                        }
                    }
                    Some(if let Some(index) = existing {
                        index
                    } else {
                        let index = backends.len();
                        let tree = disk(&root)?;
                        memory
                            .push(backends, Backend::Disk { root, tree })
                            .map_err(storage)?;
                        index
                    })
                }
            }
            Origin::Git(root) => {
                let selected = self
                    .catalog
                    .package(input.row())
                    .ok_or_else(|| refusal("reference row is absent"))?;
                let existing = backends.iter().position(|backend| matches!(backend, Backend::Git(tree) if Some(tree.source()) == selected.source()));
                Some(if let Some(index) = existing {
                    index
                } else {
                    let tree = CommittedGitTree::open(&self.catalog, input.row(), root, options)?;
                    let index = backends.len();
                    memory.push(backends, Backend::Git(tree)).map_err(storage)?;
                    index
                })
            }
            Origin::Supplied => {
                return Err(refusal("reference namespace has no actual source context"));
            }
        };
        let manifest = input
            .workspace()
            .package(input.package())
            .ok_or_else(|| refusal("reference has no selected manifest"))?
            .manifest
            .clone();
        let edges = memory
            .collect(
                self.edges
                    .iter()
                    .filter(|edge| edge.owner == input.row())
                    .map(|edge| {
                        LockedEdge::new(
                            &edge.key,
                            edge.registry
                                .as_ref()
                                .map(|(name, index)| RegistryIndex::new(name.as_deref(), index)),
                        )
                    }),
            )
            .map_err(storage)?;
        Ok(Package {
            row: input.row(),
            manifest,
            backend,
            edges,
            first_party,
        })
    }
}

fn storage(source: purrdf_lex::allocation::StorageError) -> Error {
    Error::SourceIndexStorage { source }
}
