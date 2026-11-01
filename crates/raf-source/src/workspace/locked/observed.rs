// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Structural source contexts, kept separate from configured provider admission.

use super::super::Reader;
use super::{BindingLoad, LockedBinding, Purpose};
use super::{LockedManifest, Sources, failure};
use crate::{
    CommittedGitTree, DiskTree, LoadError, LoadOptions, LockedCatalog, LockedPackageId,
    LockedSource, PackageId, RelPath, SourceAuthorityId, Tree, Workspace,
};

/// Manifest/lock identity retained for structural reference navigation.
///
/// This view has no configured target context and cannot be converted into a
/// provider-admitted [`LockedBinding`]. Its package/source contents retain the
/// observation contract of the actual selected backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReferenceBinding<'a> {
    binding: &'a LockedBinding,
}

impl ReferenceBinding<'_> {
    /// The selected row in the catalog supplied to this load.
    #[must_use]
    pub const fn row(&self) -> LockedPackageId {
        self.binding.row()
    }
    /// The actual manifest's validated package name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.binding.name()
    }
    /// The actual explicit or inherited package version.
    #[must_use]
    pub fn version(&self) -> &str {
        self.binding.version()
    }
    /// Declared source identity; contents may be an observation, not an archive.
    #[must_use]
    pub fn source(&self) -> Option<&LockedSource> {
        self.binding.source()
    }
}

impl BindingLoad<'_> {
    pub(super) fn check_observed_path(
        &self,
        reader: &Reader<'_>,
        declaration: &crate::DependencyDeclaration,
        manifest: &RelPath,
        destination: &RelPath,
    ) -> Result<(), LoadError> {
        let Purpose::Structural(contexts) = self.purpose else {
            return Err(failure(
                manifest,
                "declared path does not name the selected row's actual admitted manifest",
            ));
        };
        let (base, specification) = declaration
            .inherited
            .as_ref()
            .map_or((manifest, &declaration.specification), |inherited| {
                (&inherited.manifest, &inherited.specification)
            });
        let relative = specification.path.as_deref().ok_or_else(|| {
            failure(
                manifest,
                "path relationship has no written path declaration",
            )
        })?;
        let owner = contexts
            .get(base.authority().index())
            .ok_or_else(|| failure(manifest, "unknown owning source authority"))?;
        let target = contexts
            .get(destination.authority().index())
            .ok_or_else(|| failure(destination, "unknown destination source authority"))?;
        let bytes = base
            .as_str()
            .len()
            .saturating_add(relative.len())
            .saturating_add(destination.as_str().len());
        reader.meter.work(bytes.saturating_add(1), manifest, None)?;
        let matches = match (owner, target) {
            (ReferenceTree::Disk(owner), ReferenceTree::Disk(target)) => {
                reader.meter.items(
                    bytes
                        .saturating_add(owner.root_path_bytes())
                        .saturating_add(target.root_path_bytes())
                        .saturating_mul(4),
                    manifest,
                    None,
                )?;
                owner
                    .dependency_directory_matches(
                        &base.in_source(SourceAuthorityId::FIRST_PARTY),
                        relative,
                        target,
                        &destination.in_source(SourceAuthorityId::FIRST_PARTY),
                    )
                    .map_err(|error| {
                        LoadError::tree(manifest, None, &crate::error::Origin::Discovery, error)
                    })?
            }
            (ReferenceTree::Git(owner), ReferenceTree::Git(target))
                if owner.source() == target.source() =>
            {
                let directory = base
                    .parent()
                    .ok_or_else(|| failure(base, "manifest has no owning directory"))?;
                let expected = directory
                    .join(relative)
                    .and_then(|path| path.join("Cargo.toml"))
                    .map_err(|error| failure(manifest, &error.to_string()))?;
                let expected = crate::modtree::tidy(reader.tree, &expected).map_err(|error| {
                    LoadError::tree(manifest, None, &crate::error::Origin::Discovery, error)
                })?;
                expected.in_source(destination.authority()) == *destination
            }
            _ => {
                return Err(failure(
                    manifest,
                    "cross-authority path relationships require actual disk roots or one exact committed Git source",
                ));
            }
        };
        if !matches {
            return Err(failure(
                manifest,
                "declared path names a different package directory from the supplied destination",
            ));
        }
        Ok(())
    }
}

/// The actual backend of a structural reference source.
///
/// Disk contexts retain the root needed to verify cross-root path declarations.
/// Opaque trees permit path edges only inside their own authority. Git contexts
/// retain their exact committed source. None supplies configured provider proof.
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum ReferenceTree<'a> {
    /// A caller-owned tree with no host-directory relationship to other trees.
    Opaque(&'a dyn Tree),
    /// A contained disk root whose actual directory relationship can be checked.
    Disk(&'a DiskTree),
    /// An opaque exact committed Git source.
    Git(&'a CommittedGitTree<'a>),
}

impl<'a> ReferenceTree<'a> {
    pub(super) fn tree(self) -> &'a dyn Tree {
        match self {
            Self::Opaque(tree) => tree,
            Self::Disk(tree) => tree,
            Self::Git(tree) => tree,
        }
    }

    pub(super) fn source(self) -> Option<&'a LockedSource> {
        match self {
            Self::Git(tree) => Some(tree.source()),
            Self::Opaque(_) | Self::Disk(_) => None,
        }
    }
}

/// Explicit manifest associations in one actual structural reference context.
///
/// Package names and lock rows are checked against the loaded manifests. Disk
/// and opaque contents remain observations, not archive authentication.
#[derive(Clone, Copy)]
pub struct ReferencePackageRoots<'a> {
    pub(super) tree: ReferenceTree<'a>,
    pub(super) packages: &'a [LockedManifest<'a>],
}

impl<'a> ReferencePackageRoots<'a> {
    /// Select the actual source backend and exact retained declaration keys.
    #[must_use]
    pub const fn new(tree: ReferenceTree<'a>, packages: &'a [LockedManifest<'a>]) -> Self {
        Self { tree, packages }
    }
}

impl Workspace {
    /// Structural reference identity without configured provider admission.
    #[must_use]
    pub fn reference_binding(&self, package: PackageId) -> Option<ReferenceBinding<'_>> {
        self.references
            .get(package.index())
            .map(|binding| ReferenceBinding { binding })
    }

    /// Load one structural identity domain for first-party and reference code.
    ///
    /// Every discovered package needs an exact manifest/lock association. Edges
    /// use retained declaration keys, aliases and dependency kinds. Path edges
    /// must name the actual destination directory, including across disk roots.
    /// Optional and conditional declarations remain structural observations;
    /// no target activation or configured provider proof is asserted.
    /// Unrequested local paths retain ordinary structural lookup; unselected
    /// external dependencies remain unresolved.
    ///
    /// The ordinary index is built once over all selected sources. Returned
    /// packages have no [`Self::locked_binding`], even for sealed Git sources.
    /// All source reads and indexing share the original supplied limits.
    ///
    /// # Errors
    /// Refuses missing or ambiguous manifests, wrong rows or path destinations,
    /// configured inputs, unproved opaque cross-authority paths and exhausted
    /// source/index limits. No partial identity domain is returned.
    pub fn load_reference_sources_under(
        catalog: &LockedCatalog,
        first_party: ReferenceTree<'_>,
        first_party_roots: &[RelPath],
        local: &[LockedManifest<'_>],
        sources: &[ReferencePackageRoots<'_>],
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
            first_party,
            Some(first_party_roots),
            local,
            Sources::Observed(sources),
            options,
        )
    }
}
