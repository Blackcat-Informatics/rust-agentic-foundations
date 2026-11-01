// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Declaration-qualified reference observations, separate from provider admission.

use super::super::{Caches, Reader, WorkspaceOf, package_directory_index, workspace_of};
use super::{BindingLoad, Hint, LockedBinding, failure, native, recover};
use crate::meter::{ItemAdmission, Meter};
use crate::{
    CommittedGitTree, DependencyDeclaration, LoadError, LoadOptions, LockedCatalog,
    LockedPackageId, LockedSource, LockedSourceKind, PackageId, RegistryIndex, RelPath, Tree,
    Workspace,
};
use purrdf_lex::allocation::Memory;

fn iri_failure(manifest: &RelPath, error: purrdf_iri::IriReadError) -> LoadError {
    match error {
        purrdf_iri::IriReadError::Storage(error) => native(error),
        purrdf_iri::IriReadError::Lexical(error) => failure(manifest, &error.to_string()),
    }
}

fn manifest_image(
    tree: &dyn Tree,
    manifest: &RelPath,
    meter: &Meter,
) -> Result<RelPath, LoadError> {
    if !manifest.segments().any(|segment| segment == "..") {
        return Ok(manifest.clone());
    }
    directory_prefixes(tree, manifest, meter)?;
    meter.work(
        manifest
            .as_str()
            .len()
            .saturating_mul(64)
            .saturating_add(128),
        manifest,
        None,
    )?;
    let mut admission = ItemAdmission::new(meter, manifest);
    let result = (|| {
        let mut memory = Memory::new(&mut admission);
        let encoded = purrdf_lex::percent::encode_with_memory(
            manifest.as_str(),
            purrdf_lex::percent::UNRESERVED_SLASH,
            &mut memory,
        )
        .map_err(native)?;
        let base = purrdf_iri::parse_with_memory("https://reference.invalid/", &mut memory)
            .map_err(|error| iri_failure(manifest, error))?;
        let resolved = base
            .resolve_with_memory(&encoded, &mut memory)
            .map_err(|error| iri_failure(manifest, error))?;
        let path = resolved
            .path()
            .strip_prefix('/')
            .ok_or_else(|| failure(manifest, "native manifest image lost its source root"))?;
        // Native decoding and the retained RelPath each fit this admitted
        // upper bound; percent decoding cannot increase the encoded byte count.
        meter.items(
            crate::cost::shared_text(path.len()).saturating_mul(2),
            manifest,
            None,
        )?;
        let decoded = purrdf_lex::percent::decode(path)
            .map_err(|error| failure(manifest, &error.to_string()))?;
        RelPath::new(&decoded)
            .map(|path| path.in_source(manifest.authority()))
            .map_err(|error| failure(manifest, &error.to_string()))
    })();
    let image = result.map_err(|error| recover(error, &mut admission))?;
    let identity = |path: &RelPath| {
        tree.identity(path)
            .map_err(|error| LoadError::tree(path, None, &crate::error::Origin::Discovery, error))
    };
    if identity(manifest)? != identity(&image)? {
        return Err(failure(
            manifest,
            "parent path and native source-relative image name different files; supply an explicit manifest locator",
        ));
    }
    Ok(image)
}

fn directory_prefixes(tree: &dyn Tree, manifest: &RelPath, meter: &Meter) -> Result<(), LoadError> {
    let directory = manifest
        .parent()
        .ok_or_else(|| failure(manifest, "manifest has no source directory"))?;
    let mut prefix = manifest.source_root();
    for segment in directory.segments() {
        let bytes = prefix
            .as_str()
            .len()
            .saturating_add(segment.len())
            .saturating_add(1);
        meter.work(bytes.saturating_mul(2), manifest, None)?;
        meter.items(crate::cost::shared_text(bytes), manifest, None)?;
        prefix = prefix
            .join(segment)
            .map_err(|error| failure(manifest, &error.to_string()))?;
        match tree.entry(&prefix).map_err(|error| {
            LoadError::tree(&prefix, None, &crate::error::Origin::Discovery, error)
        })? {
            Some(crate::EntryKind::Dir) => {}
            Some(crate::EntryKind::Symlink) => {
                return Err(failure(
                    &prefix,
                    "parent traversal crosses a directory symlink; supply an explicit manifest locator",
                ));
            }
            _ => {
                return Err(failure(
                    &prefix,
                    "original manifest directory prefix is not a real directory",
                ));
            }
        }
    }
    Ok(())
}

/// One actual manifest declaration and its exact owning lock row.
pub struct ReferenceRequest<'a> {
    catalog: &'a LockedCatalog,
    owner: LockedPackageId,
    declaration: &'a DependencyDeclaration,
    registry: Option<RegistryIndex<'a>>,
}

impl<'a> ReferenceRequest<'a> {
    /// Resolve the written alias, inherited constraints and source against this catalog.
    #[must_use]
    pub const fn new(
        catalog: &'a LockedCatalog,
        owner: LockedPackageId,
        declaration: &'a DependencyDeclaration,
    ) -> Self {
        Self {
            catalog,
            owner,
            declaration,
            registry: None,
        }
    }

    /// Supply the explicitly observed index for a named or default registry.
    #[must_use]
    pub const fn registry(mut self, registry: RegistryIndex<'a>) -> Self {
        self.registry = Some(registry);
        self
    }

    /// Select exactly one complete outgoing row, without acquiring its contents.
    ///
    /// # Errors
    /// Preserves missing, ambiguous, incompatible and resource refusals from the catalog.
    pub fn select(&self, options: &LoadOptions) -> Result<LockedPackageId, LoadError> {
        self.select_with_usage(options).map(|(row, _)| row)
    }

    /// Select one outgoing row and report its actual bounded selection usage.
    ///
    /// Work covers declaration/source/version comparisons. Item usage includes
    /// the retained catalog allowance; it is not newly acquired source storage.
    /// No source file is loaded and no provider authority is admitted.
    ///
    /// # Errors
    /// Preserves the same catalog and resource refusals as [`Self::select`].
    pub fn select_with_usage(
        &self,
        options: &LoadOptions,
    ) -> Result<(LockedPackageId, crate::Usage), LoadError> {
        let meter = Meter::new(options);
        let row = self.select_under(&meter, &RelPath::root())?;
        Ok((row, meter.usage()))
    }

    fn select_under(
        &self,
        meter: &Meter,
        manifest: &RelPath,
    ) -> Result<LockedPackageId, LoadError> {
        meter.items(self.catalog.usage().item_bytes, manifest, None)?;
        self.catalog
            .select_under(self.owner, self.declaration, self.registry, meter)
    }
}

/// A read-only reference package whose manifest matches its selected lock identity.
///
/// Local and registry contents are observations of the supplied contained Tree,
/// not archive authentication. Git contents require the sealed committed backend.
/// The loaded workspace deliberately retains no locked bindings, configured
/// dependency edges or preferred-provider admission.
pub struct ReferenceObservation {
    workspace: Workspace,
    identity: LockedBinding,
    checksum: Option<[u8; 32]>,
}

impl ReferenceObservation {
    /// Observe one actual local root package and associate its explicit or
    /// inherited version with a unique source-less lock row. This is the owning
    /// manifest context for subsequent declaration-qualified reference requests.
    ///
    /// # Errors
    /// Refuses missing/ambiguous lock identity, invalid inheritance and bounded
    /// source failures. No provider admission is retained.
    pub fn local_root(
        catalog: &LockedCatalog,
        tree: &dyn Tree,
        manifest: &RelPath,
        options: &LoadOptions,
    ) -> Result<Self, LoadError> {
        let meter = Meter::new(options);
        meter.items(catalog.usage().item_bytes, manifest, None)?;
        let image = manifest_image(tree, manifest, &meter)?;
        let discovered = crate::discovery::single(tree, &image, &meter)?;
        let found = &discovered.packages[0];
        let dirs = package_directory_index(&discovered.packages, &meter)?;
        let reader = Reader {
            tree,
            meter: &meter,
            dirs: &dirs,
            found: &discovered.packages,
        };
        let mut caches = Caches::default();
        let inherited;
        let version = match &found.declared.version {
            crate::manifest::VersionDecl::Explicit(version) => version.as_str(),
            crate::manifest::VersionDecl::Inherited => {
                inherited = match workspace_of(&reader, &found.package.dir, &mut caches)? {
                    WorkspaceOf::Member {
                        version: Some(version),
                        ..
                    } => version,
                    _ => {
                        return Err(failure(
                            manifest,
                            "local package version has no admitted workspace value",
                        ));
                    }
                };
                inherited.as_str()
            }
            crate::manifest::VersionDecl::Unset => {
                return Err(failure(
                    manifest,
                    "local package needs explicit or inherited version evidence",
                ));
            }
        };
        let mut selected = None;
        for row in catalog.packages() {
            let comparison = row
                .name()
                .len()
                .saturating_add(found.declared.name.len())
                .saturating_add(row.version().len())
                .saturating_add(version.len())
                .saturating_add(1);
            meter.work(comparison, manifest, None)?;
            if row.source().is_some()
                || row.name() != found.declared.name
                || row.version() != version
            {
                continue;
            }
            if selected.replace(row.id()).is_some() {
                return Err(failure(
                    manifest,
                    "actual local manifest matches multiple lock rows",
                ));
            }
        }
        let row = selected.ok_or_else(|| {
            failure(
                manifest,
                "actual local manifest name/version has no matching lock row",
            )
        })?;
        Self::observe_discovered(catalog, row, tree, &meter, &discovered)
    }
    /// Observe a local or registry package through the ordinary bounded source loader.
    /// The manifest is relative to the supplied Tree, whose root must retain any
    /// workspace manifest needed for inherited package fields.
    ///
    /// # Errors
    /// Refuses unresolved declarations, Git rows, mismatched manifests, extra
    /// packages, invalid inheritance and all ordinary source/containment limits.
    pub fn load(
        request: &ReferenceRequest<'_>,
        tree: &dyn Tree,
        manifest: &RelPath,
        options: &LoadOptions,
    ) -> Result<Self, LoadError> {
        let meter = Meter::new(options);
        let row = request.select_under(&meter, manifest)?;
        if request
            .catalog
            .package(row)
            .and_then(|row| row.source())
            .is_some_and(|source| source.kind() == LockedSourceKind::Git)
        {
            return Err(failure(
                manifest,
                "a Git reference requires its sealed committed Tree",
            ));
        }
        Self::observe(request.catalog, row, tree, manifest, &meter)
    }

    /// Observe the exact committed source selected by the declaration and lock row.
    ///
    /// The returned workspace usage includes the protocol, listing and blob work
    /// of this observation under its original selection/source allowance. The
    /// caller's prior construction of the supplied Tree precedes this operation.
    ///
    /// # Errors
    /// Refuses a different committed source and all bounded source/identity failures.
    pub fn committed(
        request: &ReferenceRequest<'_>,
        tree: &CommittedGitTree<'_>,
        manifest: &RelPath,
        options: &LoadOptions,
    ) -> Result<Self, LoadError> {
        let meter = Meter::new(options);
        let row = request.select_under(&meter, manifest)?;
        if request.catalog.package(row).and_then(|row| row.source()) != Some(tree.source()) {
            return Err(failure(
                manifest,
                "committed reference source differs from its selected lock row",
            ));
        }
        // Caller-owned construction precedes this observation. Reopen its
        // sealed source under this operation's original selection/load grant.
        let observed = tree.under_meter(&meter)?;
        Self::observe(request.catalog, row, &observed, manifest, &meter)
    }

    fn observe(
        catalog: &LockedCatalog,
        row: LockedPackageId,
        tree: &dyn Tree,
        manifest: &RelPath,
        meter: &Meter,
    ) -> Result<Self, LoadError> {
        let image = manifest_image(tree, manifest, meter)?;
        let discovered = crate::discovery::single(tree, &image, meter)?;
        Self::observe_discovered(catalog, row, tree, meter, &discovered)
    }

    fn observe_discovered(
        catalog: &LockedCatalog,
        row: LockedPackageId,
        tree: &dyn Tree,
        meter: &Meter,
        discovered: &crate::discovery::Discovered,
    ) -> Result<Self, LoadError> {
        let manifest = discovered.packages[0].package.manifest.clone();
        let options = meter.options();
        meter.items(crate::index::LOAD_BASE_BYTES, &manifest, None)?;
        let mut admission = ItemAdmission::new(meter, &manifest);
        let mut hints = Vec::new();
        Memory::new(&mut admission)
            .push(
                &mut hints,
                Hint {
                    row,
                    manifest: manifest.clone(),
                    edges: &[],
                    configuration: None,
                },
            )
            .map_err(|error| admission.error(error))?;
        let binding = BindingLoad {
            catalog,
            hints,
            purpose: super::Purpose::Configured,
        };
        let mut workspace =
            Workspace::build_discovered(tree, options, meter, Some(&binding), discovered)?;
        let identity = workspace.locked.pop().ok_or_else(|| {
            failure(
                &manifest,
                "reference manifest has no validated package identity",
            )
        })?;
        // The one exact association was validated by bind_found. No provider
        // proof or configured edge escapes through the reference workspace.
        if !workspace.locked.is_empty() || workspace.packages.len() != 1 {
            return Err(failure(
                &manifest,
                "a reference observation must select exactly one package",
            ));
        }
        let checksum = catalog.package(row).and_then(|row| row.checksum()).copied();
        Ok(Self {
            workspace,
            identity,
            checksum,
        })
    }

    /// Parsed source and declarations, without provider-admitted locked bindings.
    #[must_use]
    pub const fn workspace(&self) -> &Workspace {
        &self.workspace
    }

    /// The selected package within this observation's own workspace.
    #[must_use]
    pub const fn package(&self) -> PackageId {
        PackageId(0)
    }

    /// Selected identity within the request's catalog.
    #[must_use]
    pub const fn row(&self) -> LockedPackageId {
        self.identity.row
    }

    /// Manifest-verified Cargo package name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.identity.name
    }

    /// Actual explicit or inherited manifest version.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.identity.version
    }

    /// Complete external lock source, absent for a local path package.
    #[must_use]
    pub const fn source(&self) -> Option<&LockedSource> {
        self.identity.source.as_ref()
    }

    /// Recorded registry archive checksum; this does not authenticate a supplied Tree.
    #[must_use]
    pub const fn checksum(&self) -> Option<&[u8; 32]> {
        self.checksum.as_ref()
    }
}
