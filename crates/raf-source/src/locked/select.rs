// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Declaration-qualified outgoing observations, independently of source binding.

use purrdf_lex::allocation::{Admission, Memory};

use super::{
    BuildError, LockedCatalog, LockedGitReference, LockedPackage, LockedPackageId, LockedSource,
    LockedSourceKind, RegistryIndex, invalid,
};
use crate::manifest::validate_dependency_spec;
use crate::meter::Meter;
use crate::{DependencyDeclaration, DependencySpec};

enum SourceConstraint<'a> {
    Path(Option<&'a LockedSource>),
    Git {
        source: LockedSource,
        spec: &'a DependencySpec,
    },
    Registry(LockedSource),
}

impl SourceConstraint<'_> {
    fn matches(&self, package: &LockedPackage) -> bool {
        match self {
            Self::Path(expected) => package.source() == *expected,
            Self::Registry(expected) => package.source() == Some(expected),
            Self::Git { source, spec } => package.source().is_some_and(|actual| {
                actual.kind() == LockedSourceKind::Git
                    && actual.repository() == source.repository()
                    && actual.revision().is_some()
                    && match actual.reference() {
                        Some(LockedGitReference::DefaultBranch) => {
                            spec.rev.is_none() && spec.branch.is_none() && spec.tag.is_none()
                        }
                        Some(LockedGitReference::Revision(value)) => {
                            spec.rev.as_ref() == Some(value)
                        }
                        Some(LockedGitReference::Branch(value)) => {
                            spec.branch.as_ref() == Some(value)
                        }
                        Some(LockedGitReference::Tag(value)) => spec.tag.as_ref() == Some(value),
                        None => false,
                    }
            }),
        }
    }
}

pub(super) fn dependency<S: Admission + ?Sized>(
    catalog: &LockedCatalog,
    owner: LockedPackageId,
    declaration: &DependencyDeclaration,
    registry: Option<RegistryIndex<'_>>,
    memory: &mut Memory<'_, S>,
    meter: &Meter,
) -> Result<LockedPackageId, BuildError> {
    let owner = catalog.package(owner).ok_or_else(|| {
        invalid(
            memory,
            "declaration",
            format_args!(
                "unknown owning lock row {}; select an owner from this catalog",
                owner.index()
            ),
        )
    })?;
    meter.work(
        declaration
            .name
            .len()
            .saturating_add(declaration.specification.heap_bytes())
            .saturating_add(
                declaration
                    .inherited
                    .as_ref()
                    .map_or(0, |entry| entry.specification.heap_bytes()),
            )
            .saturating_add(registry.map_or(0, |entry| entry.source.len())),
        &catalog.path,
        None,
    )?;
    let spec = effective_spec(declaration, memory)?;
    let name = spec.package.as_deref().unwrap_or(&declaration.name);
    let requirement = spec
        .version
        .as_ref()
        .map(|value| {
            semver::VersionReq::parse(value).map_err(|error| {
                invalid(
                    memory,
                    "declaration",
                    format_args!("invalid version requirement: {error}"),
                )
            })
        })
        .transpose()?;
    let source = source_constraint(spec, owner.source(), registry, memory)?;
    let mut selected: Option<LockedPackageId> = None;
    for id in owner.dependencies() {
        let package = catalog.package(*id).ok_or_else(|| {
            invalid(
                memory,
                "declaration",
                format_args!("owning row contains an invalid outgoing row {}", id.index()),
            )
        })?;
        meter.work(comparison_work(package, name), &catalog.path, None)?;
        if package.name() != name || !source.matches(package) {
            continue;
        }
        let version = semver::Version::parse(package.version()).map_err(|error| {
            invalid(
                memory,
                "declaration",
                format_args!("invalid catalog version: {error}"),
            )
        })?;
        if requirement
            .as_ref()
            .is_some_and(|requirement| !requirement.matches(&version))
        {
            continue;
        }
        if let Some(previous) = selected {
            return Err(invalid(
                memory,
                "declaration",
                format_args!(
                    "{} is ambiguous between outgoing rows {} and {}; supply precise version/source constraints",
                    declaration.key,
                    previous.index(),
                    id.index()
                ),
            ));
        }
        selected = Some(*id);
    }
    selected.ok_or_else(|| invalid(memory, "declaration", format_args!(
        "{} selects no compatible outgoing row of {} {}; check the declaration, owner and lock file",
        declaration.key, owner.name(), owner.version())))
}

fn comparison_work(package: &LockedPackage, name: &str) -> usize {
    let source_bytes = package.source().map_or(0, |source| {
        source
            .repository()
            .len()
            .saturating_add(source.revision().map_or(0, str::len))
            .saturating_add(match source.reference() {
                Some(
                    LockedGitReference::Branch(text)
                    | LockedGitReference::Tag(text)
                    | LockedGitReference::Revision(text),
                ) => text.len(),
                _ => 0,
            })
    });
    1_usize
        .saturating_add(name.len())
        .saturating_add(package.name().len())
        .saturating_add(package.version().len())
        .saturating_add(source_bytes)
}

fn effective_spec<'a, S: Admission + ?Sized>(
    declaration: &'a DependencyDeclaration,
    memory: &mut Memory<'_, S>,
) -> Result<&'a DependencySpec, BuildError> {
    validate_dependency_spec(&declaration.specification, &declaration.key).map_err(|error| {
        invalid(
            memory,
            "declaration",
            format_args!("{}: {}", declaration.key, error.message),
        )
    })?;
    match (&declaration.inherited, declaration.specification.workspace) {
        (None, false) => Ok(&declaration.specification),
        (Some(inherited), true) if !inherited.specification.workspace => {
            validate_dependency_spec(&inherited.specification, &declaration.key).map_err(
                |error| {
                    invalid(
                        memory,
                        "declaration",
                        format_args!("invalid inherited entry: {}", error.message),
                    )
                },
            )?;
            Ok(&inherited.specification)
        }
        _ => Err(invalid(
            memory,
            "declaration",
            format_args!(
                "{} has inconsistent workspace inheritance; supply the observed workspace entry",
                declaration.key
            ),
        )),
    }
}

fn source_constraint<'a, S: Admission + ?Sized>(
    spec: &'a DependencySpec,
    owner: Option<&'a LockedSource>,
    registry: Option<RegistryIndex<'_>>,
    memory: &mut Memory<'_, S>,
) -> Result<SourceConstraint<'a>, BuildError> {
    if let Some(git) = &spec.git {
        if registry.is_some() {
            return Err(invalid(
                memory,
                "declaration",
                format_args!("registry configuration was supplied for a Git declaration"),
            ));
        }
        let text = memory.format(&format_args!("git+{git}"))?;
        let source = LockedSource::parse(&text, memory)?;
        memory.release_string(text)?;
        if source.revision().is_some()
            || source.reference() != Some(&LockedGitReference::DefaultBranch)
        {
            return Err(invalid(
                memory,
                "declaration",
                format_args!(
                    "Git repository must not embed a selector or revision; use rev, branch or tag"
                ),
            ));
        }
        return Ok(SourceConstraint::Git { source, spec });
    }
    if spec.path.is_some() {
        if registry.is_some() {
            return Err(invalid(
                memory,
                "declaration",
                format_args!(
                    "registry configuration cannot establish a path declaration's local lock identity"
                ),
            ));
        }
        if owner.is_some_and(|source| source.kind() != LockedSourceKind::Git) {
            return Err(invalid(
                memory,
                "declaration",
                format_args!(
                    "a path declaration requires a local owner or an exact Git workspace source"
                ),
            ));
        }
        return Ok(SourceConstraint::Path(owner));
    }
    registry_source(spec, registry, memory).map(SourceConstraint::Registry)
}

fn registry_source<S: Admission + ?Sized>(
    spec: &DependencySpec,
    registry: Option<RegistryIndex<'_>>,
    memory: &mut Memory<'_, S>,
) -> Result<LockedSource, BuildError> {
    let text = if let Some(index) = &spec.registry_index {
        if spec.registry.is_some() || registry.is_some() {
            return Err(invalid(
                memory,
                "declaration",
                format_args!("explicit registry-index conflicts with registry-name configuration"),
            ));
        }
        if index.starts_with("sparse+") {
            memory.string(index)?
        } else {
            memory.format(&format_args!("registry+{index}"))?
        }
    } else {
        let config = registry.ok_or_else(|| invalid(memory, "declaration", format_args!(
            "supply the explicit registry index for {:?}; no default or custom index is inferred", spec.registry)))?;
        if config.name != spec.registry.as_deref() {
            return Err(invalid(
                memory,
                "declaration",
                format_args!(
                    "registry configuration name {:?} does not match declaration {:?}",
                    config.name, spec.registry
                ),
            ));
        }
        memory.string(config.source)?
    };
    let source = LockedSource::parse(&text, memory)?;
    memory.release_string(text)?;
    if !matches!(
        source.kind(),
        LockedSourceKind::Registry | LockedSourceKind::SparseRegistry
    ) {
        return Err(invalid(
            memory,
            "declaration",
            format_args!("registry configuration must identify a registry or sparse index"),
        ));
    }
    Ok(source)
}
