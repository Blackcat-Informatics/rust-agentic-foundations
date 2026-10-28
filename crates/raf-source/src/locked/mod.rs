// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Exact package and outgoing-edge observations from an explicit lock profile.

mod read;
mod select;
mod source;

use std::fmt;

use purrdf_lex::allocation::{Admission, Memory, StorageError};

use crate::cost::shared_text;
use crate::meter::{ItemAdmission, Meter};
use crate::{LoadError, LoadErrorKind, LoadOptions, Origin, RelPath, Tree, Usage};

pub use source::{LockedGitReference, LockedSource, LockedSourceKind};

/// Explicit Cargo registry configuration supplied for one declaration selection.
///
/// This is an observation of configuration, not authentication of registry content.
/// The source must include its `registry+` or `sparse+` family; no default index
/// or registry-name mapping is inferred.
#[derive(Debug, Clone, Copy)]
pub struct RegistryIndex<'a> {
    pub(crate) name: Option<&'a str>,
    pub(crate) source: &'a str,
}

impl<'a> RegistryIndex<'a> {
    /// Associate a registry name (`None` for the default) with an explicit index.
    /// The source is validated when selecting an outgoing dependency.
    #[must_use]
    pub const fn new(name: Option<&'a str>, source: &'a str) -> Self {
        Self { name, source }
    }
}

/// A package row's identity within one [`LockedCatalog`], not a persistent key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LockedPackageId(pub(crate) usize);

impl LockedPackageId {
    /// The row position in [`LockedCatalog::packages`].
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// One complete locked package identity and its uniquely selected outgoing edges.
///
/// An absent source denotes a local lock row. Its actual manifest location still
/// has to be supplied; absence is not a universal local source identity.
#[derive(Debug, PartialEq, Eq)]
pub struct LockedPackage {
    id: LockedPackageId,
    name: String,
    version: String,
    source: Option<LockedSource>,
    checksum: Option<[u8; 32]>,
    dependencies: Vec<LockedPackageId>,
}

impl LockedPackage {
    /// The load-local row identity.
    #[must_use]
    pub const fn id(&self) -> LockedPackageId {
        self.id
    }

    /// The exact package name, distinct from any Rust dependency alias.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The validated canonical Cargo version, including prerelease and build text.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// The complete external source identity, or a local row awaiting binding.
    #[must_use]
    pub const fn source(&self) -> Option<&LockedSource> {
        self.source.as_ref()
    }

    /// The recorded archive checksum, not authentication of a supplied tree.
    #[must_use]
    pub const fn checksum(&self) -> Option<&[u8; 32]> {
        self.checksum.as_ref()
    }

    /// Actual outgoing edges after unique full-identity selection.
    #[must_use]
    pub fn dependencies(&self) -> &[LockedPackageId] {
        &self.dependencies
    }

    fn key(&self) -> (&str, &str, Option<&LockedSource>) {
        (&self.name, &self.version, self.source.as_ref())
    }
}

/// A strict Cargo.lock version 3 or 4 package/edge profile.
///
/// TOML, repository IRIs, reference percent escapes, checksums and retained storage
/// use `PurRDF`. Cargo versions use the private borrowed version adapter. Source
/// repository spellings remain lexical identities, not guessed URL equivalences.
/// Abbreviated edges must select exactly one complete package. Replacements and
/// unsupported fields refuse instead of publishing partial dependency evidence.
/// A catalog proves locked relationships, not source contents or active features.
///
/// Native retained conversion uses the original item allowance. Version parsing
/// allocates nothing, and matching spends the original work grant before scanning
/// qualifiers. The TOML decoder's temporary internal allocations have no admission
/// callback; the original file/total-byte bounds apply to its input.
#[derive(Debug)]
pub struct LockedCatalog {
    path: RelPath,
    packages: Vec<LockedPackage>,
    order: Vec<LockedPackageId>,
    usage: Usage,
}

impl PartialEq for LockedCatalog {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.packages == other.packages && self.order == other.order
    }
}

impl Eq for LockedCatalog {}

impl LockedCatalog {
    /// Read and validate all package rows and outgoing references under bounds.
    ///
    /// # Errors
    /// Returns the original tree, byte, native storage or work refusal, or an
    /// actionable lock-profile error for malformed, unsupported, duplicate,
    /// missing or ambiguous evidence. No first-match edge is accepted.
    pub fn load(tree: &dyn Tree, path: &RelPath, options: &LoadOptions) -> Result<Self, LoadError> {
        let meter = Meter::new(options);
        let text = tree
            .read(path, options.max_file_bytes)
            .map_err(|error| LoadError::tree(path, None, &Origin::Discovery, error))?;
        meter.read(path, &text)?;
        meter.items(
            size_of::<Self>() + shared_text(path.as_str().len()),
            path,
            None,
        )?;
        let mut admission = ItemAdmission::new(&meter, path);
        let result = {
            let mut memory = Memory::new(&mut admission);
            read::catalog(&text, &mut memory, &meter, path)
        };
        let (packages, order) = result.map_err(|error| match error {
            BuildError::Storage(error) => admission.error(error),
            BuildError::Load(error) => error,
            BuildError::Invalid {
                field,
                message,
                source,
            } => LoadError::new(
                path,
                None,
                LoadErrorKind::Locked {
                    field,
                    message,
                    source,
                },
            ),
        })?;
        Ok(Self {
            path: path.clone(),
            packages,
            order,
            usage: meter.usage(),
        })
    }

    /// Package rows in lock-file order, with load-local IDs.
    pub fn packages(&self) -> impl Iterator<Item = &LockedPackage> {
        self.packages.iter()
    }

    /// The package of a valid row identity.
    #[must_use]
    pub fn package(&self, id: LockedPackageId) -> Option<&LockedPackage> {
        self.packages.get(id.index())
    }

    /// Select one owning row's outgoing dependency by its written declaration.
    ///
    /// Uses the package rename, version and declared source/reference constraints.
    /// Registry declarations need explicit matching configuration unless their
    /// index is written directly. A path declaration retains its owner's local
    /// or exact Git-workspace source; selection does not prove that row
    /// corresponds to the declared directory.
    /// Optional features and target conditions are not evaluated. Selection alone
    /// neither binds a source tree nor establishes a loaded package's identity.
    ///
    /// # Errors
    /// Refuses unknown owners, malformed declarations/configuration, missing or
    /// ambiguous outgoing matches, incompatible constraints and budget failures.
    /// Query storage shares the original retained catalog's item allowance; work
    /// bounds this selection's comparisons, including compared text lengths.
    pub fn select_dependency(
        &self,
        owner: LockedPackageId,
        declaration: &crate::DependencyDeclaration,
        registry: Option<RegistryIndex<'_>>,
        options: &LoadOptions,
    ) -> Result<LockedPackageId, LoadError> {
        let meter = Meter::new(options);
        meter.items(self.usage.item_bytes, &self.path, None)?;
        self.select_under(owner, declaration, registry, &meter)
    }

    pub(crate) fn select_under(
        &self,
        owner: LockedPackageId,
        declaration: &crate::DependencyDeclaration,
        registry: Option<RegistryIndex<'_>>,
        meter: &Meter,
    ) -> Result<LockedPackageId, LoadError> {
        let mut admission = ItemAdmission::new(meter, &self.path);
        let result = select::dependency(
            self,
            owner,
            declaration,
            registry,
            &mut Memory::new(&mut admission),
            meter,
        );
        result.map_err(|error| match error {
            BuildError::Storage(error) => admission.error(error),
            BuildError::Load(error) => error,
            BuildError::Invalid {
                field,
                message,
                source,
            } => LoadError::new(
                &self.path,
                None,
                LoadErrorKind::Locked {
                    field,
                    message,
                    source,
                },
            )
            .reached_by(&Origin::ManifestKey {
                key: declaration.key.clone(),
            }),
        })
    }

    /// The byte, retained conversion and comparison work charged by this load.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }
}

enum BuildError {
    Storage(StorageError),
    Load(LoadError),
    Invalid {
        field: &'static str,
        message: String,
        source: Option<Box<purrdf_toml::TomlError>>,
    },
}

impl From<StorageError> for BuildError {
    fn from(error: StorageError) -> Self {
        Self::Storage(error)
    }
}

impl From<LoadError> for BuildError {
    fn from(error: LoadError) -> Self {
        Self::Load(error)
    }
}

fn invalid<S: Admission + ?Sized>(
    memory: &mut Memory<'_, S>,
    field: &'static str,
    message: fmt::Arguments<'_>,
) -> BuildError {
    match memory.format(&message) {
        Ok(message) => BuildError::Invalid {
            field,
            message,
            source: None,
        },
        Err(error) => BuildError::Storage(error),
    }
}
