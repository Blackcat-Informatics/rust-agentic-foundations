// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Reference-only package collection and cross-boundary evidence.

pub(super) mod acquire;
mod witness;
pub use acquire::ReferenceRoot;

use crate::{
    Census, Error, Fingerprint, IndexUnit, JobReport, Ledger, MatchClass, ReferenceSource, Unit,
};
use purrdf_jsonschema::ecma::MatchLimits;
use purrdf_lex::allocation::{Memory, OrderedTable, Resident};
use raf_source::{
    LockedCatalog, LockedGitReference, LockedPackage, LockedSourceKind, PackageId,
    ReferenceObservation, Workspace,
};
use std::collections::BTreeSet;

pub(super) struct ReferencePackage<'a> {
    workspace: &'a Workspace,
    package: PackageId,
    source: ReferenceSource,
    pub(super) units: Vec<Unit>,
}

pub(super) struct References<'a> {
    packages: Vec<ReferencePackage<'a>>,
    pub(super) consumers: BTreeSet<PackageId>,
}

pub(super) fn refusal(message: impl Into<String>) -> Error {
    Error::Options(format!("reference scan: {}", message.into()))
}

pub(super) fn receipt(row: &LockedPackage, manifest: &str) -> ReferenceSource {
    let source = row.source();
    let source_kind = source.map_or("local", |source| match source.kind() {
        LockedSourceKind::Git => "git",
        LockedSourceKind::Registry => "registry",
        LockedSourceKind::SparseRegistry => "sparse",
        LockedSourceKind::Path => "path",
        LockedSourceKind::LocalRegistry => "local-registry",
        LockedSourceKind::Directory => "directory",
    });
    let selector = match source.and_then(|source| source.reference()) {
        Some(LockedGitReference::Branch(text)) => format!("branch:{text}"),
        Some(LockedGitReference::Tag(text)) => format!("tag:{text}"),
        Some(LockedGitReference::Revision(text)) => format!("rev:{text}"),
        Some(LockedGitReference::DefaultBranch) | None => String::new(),
    };
    ReferenceSource {
        package: row.name().into(),
        version: row.version().into(),
        source_kind: source_kind.into(),
        repository: source.map_or("", |source| source.repository()).into(),
        selector,
        revision: source
            .and_then(|source| source.revision())
            .unwrap_or("")
            .into(),
        manifest: manifest.into(),
        checksum: row
            .checksum()
            .map_or_else(String::new, |bytes| purrdf_hash::hex::encode(bytes)),
        observation: if source_kind == "git" {
            "committed-git"
        } else {
            "contained-tree"
        }
        .into(),
    }
}

fn consumer_graph(
    ledger: &Ledger,
    workspace: &Workspace,
    catalog: &LockedCatalog,
    owners: &[(PackageId, ReferenceObservation)],
    memory: &mut Memory<'_, Resident>,
) -> Result<OrderedTable<raf_source::LockedPackageId, ()>, Error> {
    let storage = |source| Error::SourceIndexStorage { source };
    let mut reachable = OrderedTable::default();
    let mut pending = Vec::new();
    for row in workspace
        .package_ids()
        .filter_map(|package| {
            let package_info = workspace.package(package)?;
            if package_info.manifest.authority() != raf_source::SourceAuthorityId::FIRST_PARTY
                || ledger.reference().contains(&package_info.name)
            {
                return None;
            }
            workspace
                .locked_binding(package)
                .map(raf_source::LockedBinding::row)
        })
        .chain(
            owners
                .iter()
                .filter(|(_, input)| !ledger.reference().iter().any(|name| name == input.name()))
                .map(|(_, input)| input.row()),
        )
    {
        reachable.insert(row, (), memory).map_err(storage)?;
        memory.push(&mut pending, row).map_err(storage)?;
    }
    while let Some(owner) = pending.pop() {
        let row = catalog
            .package(owner)
            .ok_or_else(|| refusal("owning locked row is missing"))?;
        for target in row.dependencies() {
            if reachable
                .insert(*target, (), memory)
                .map_err(storage)?
                .is_none()
            {
                memory.push(&mut pending, *target).map_err(storage)?;
            }
        }
    }
    Ok(reachable)
}

fn prints(unit: &Unit) -> impl Iterator<Item = &Fingerprint> {
    unit.print
        .as_ref()
        .and_then(|print| print.body.as_ref())
        .into_iter()
        .chain(unit.shim_key.as_ref())
}

impl<'a> References<'a> {
    pub(super) fn empty(workspace: &Workspace) -> Self {
        Self {
            packages: Vec::new(),
            consumers: workspace.package_ids().collect(),
        }
    }

    pub(super) fn prepare(
        ledger: &Ledger,
        workspace: &'a Workspace,
        catalog: &LockedCatalog,
        observed: &'a [ReferenceObservation],
        owners: &'a [(PackageId, ReferenceObservation)],
    ) -> Result<Self, Error> {
        let mut resident = Resident;
        let borrowed = Memory::new(&mut resident)
            .collect(observed.iter())
            .map_err(|source| Error::SourceIndexStorage { source })?;
        Self::prepare_borrowed(ledger, workspace, catalog, &borrowed, owners)
    }

    pub(super) fn prepare_borrowed(
        ledger: &Ledger,
        workspace: &'a Workspace,
        catalog: &LockedCatalog,
        observed: &[&'a ReferenceObservation],
        owners: &'a [(PackageId, ReferenceObservation)],
    ) -> Result<Self, Error> {
        let mut resident = Resident;
        let mut memory = Memory::new(&mut resident);
        let storage = |source| Error::SourceIndexStorage { source };
        let reachable = consumer_graph(ledger, workspace, catalog, owners, &mut memory)?;
        let mut result = Self::empty(workspace);
        result.consumers.retain(|package| {
            workspace.package(*package).is_some_and(|package| {
                package.manifest.authority() == raf_source::SourceAuthorityId::FIRST_PARTY
            })
        });
        for name in ledger.reference() {
            let mut rows = catalog
                .packages()
                .filter(|row| row.name() == name && reachable.get(&row.id()).is_some());
            let row = rows.next().ok_or_else(|| {
                refusal(format!(
                    "package {name:?} is not reachable from the admitted workspace lock roots"
                ))
            })?;
            if rows.next().is_some() {
                return Err(refusal(format!(
                    "package {name:?} is ambiguous in the locked dependency graph"
                )));
            }
            let internal = memory
                .collect(workspace.package_ids().filter(|package| {
                    workspace
                        .locked_binding(*package)
                        .is_some_and(|binding| binding.row() == row.id())
                }))
                .map_err(storage)?;
            let (reference_workspace, package) = if let [package] = internal.as_slice() {
                result.consumers.remove(package);
                (workspace, *package)
            } else if internal.is_empty()
                && let Some((package, input)) = owners.iter().find(|(_, input)| {
                    input.row() == row.id()
                        && input.name() == row.name()
                        && input.version() == row.version()
                        && input.source() == row.source()
                })
            {
                result.consumers.remove(package);
                (input.workspace(), input.package())
            } else if internal.is_empty() {
                let mut supplied = observed.iter().copied().filter(|input| {
                    input.name() == row.name()
                        && input.version() == row.version()
                        && input.source() == row.source()
                });
                let selected = supplied.next().ok_or_else(|| refusal(format!("package {name:?} has no admitted local reference source; supply its contained root or exact committed repository")))?;
                if supplied.next().is_some() {
                    return Err(refusal(format!(
                        "package {name:?} has multiple reference observations"
                    )));
                }
                (selected.workspace(), selected.package())
            } else {
                return Err(refusal(format!(
                    "package {name:?} has multiple loaded manifests"
                )));
            };
            let manifest = &reference_workspace
                .package(package)
                .ok_or_else(|| refusal("reference package is absent from its observation"))?
                .manifest;
            let source = receipt(row, manifest.as_str());
            let identity = source.identity()?;
            let mut units = crate::units::collect_packages(
                reference_workspace,
                ledger.options(),
                &BTreeSet::from([package]),
            )?;
            for unit in &mut units {
                unit.file = format!(
                    "reference:{}/{}",
                    crate::report::mint::part(&identity),
                    unit.file
                );
            }
            memory
                .push(
                    &mut result.packages,
                    ReferencePackage {
                        workspace: reference_workspace,
                        package,
                        source,
                        units,
                    },
                )
                .map_err(storage)?;
        }
        Ok(result)
    }

    pub(super) fn attach(
        &self,
        census: &mut Census,
        limits: &mut MatchLimits,
    ) -> Result<(), Error> {
        let mut resident = Resident;
        let mut memory = Memory::new(&mut resident);
        let storage = |source| Error::SourceIndexStorage { source };
        let mut by_print = OrderedTable::<&Fingerprint, Vec<&Unit>>::default();
        for package in &self.packages {
            if package.units.is_empty() {
                continue;
            }
            memory
                .push(&mut census.reference_sources, package.source.clone())
                .map_err(storage)?;
            let identity = package.source.identity()?;
            for unit in &package.units {
                memory
                    .push(&mut census.reference_raw_units, unit.clone())
                    .map_err(storage)?;
                memory
                    .push(
                        &mut census.reference_units,
                        IndexUnit {
                            reference_source: Some(identity.clone()),
                            symbol: unit.symbol.clone(),
                            package: unit.package.clone(),
                            file: unit.file.clone(),
                            line: unit.line,
                            start_byte: unit.start_byte,
                            shipping: unit.shipping,
                            tokens: unit.print.as_ref().map_or(0, |print| print.tokens),
                            body: unit.print.as_ref().and_then(|print| print.body.clone()),
                            shim: unit.shim_key.clone(),
                        },
                    )
                    .map_err(storage)?;
                for fingerprint in prints(unit) {
                    if by_print.get(&fingerprint).is_none() {
                        by_print
                            .insert(fingerprint, Vec::new(), &mut memory)
                            .map_err(storage)?;
                    }
                    memory
                        .push(
                            by_print
                                .get_mut(&fingerprint)
                                .expect("inserted fingerprint"),
                            unit,
                        )
                        .map_err(storage)?;
                }
            }
        }
        census.reference_witnesses = witness::collect(census.units(), &by_print, limits)?;
        Ok(())
    }

    pub(super) fn home(&self, symbol: &str) -> Result<Option<super::owners::Home<'_>>, Error> {
        let mut homes = OrderedTable::default();
        let mut resident = Resident;
        let mut memory = Memory::new(&mut resident);
        for package in &self.packages {
            let Some((head, tail)) = symbol.split_once("::") else {
                continue;
            };
            if !package
                .workspace
                .targets()
                .any(|target| target.package_id == package.package && target.crate_path == head)
            {
                continue;
            }
            match package.workspace.resolve_from_package_id(
                package.package,
                head,
                &format!("crate::{tail}"),
            ) {
                Ok(resolved) => {
                    for site in resolved.sites {
                        let item = package
                            .workspace
                            .item(site)
                            .ok_or_else(|| refusal("reference home definition is absent"))?;
                        for unit in &package.units {
                            let direct = unit.source_file == item.file
                                && unit.start_byte == item.span.start_byte;
                            let mut inside = false;
                            if item.kind == raf_source::SymbolKind::Mod {
                                let code = package.workspace.code_resolver()?;
                                for child in package.workspace.items().filter(|child| {
                                    child.file == unit.source_file
                                        && child.span.start_byte == unit.start_byte
                                }) {
                                    if code
                                        .is_defined_in(child.id, site)
                                        .map_err(|error| refusal(error.to_string()))?
                                    {
                                        inside = true;
                                        break;
                                    }
                                }
                            }
                            if direct || inside {
                                homes
                                    .insert(
                                        (unit.file.as_str(), unit.start_byte),
                                        unit,
                                        &mut memory,
                                    )
                                    .map_err(|source| Error::SourceIndexStorage { source })?;
                            }
                        }
                    }
                }
                Err(raf_source::ResolveError::NotFound(_)) => {}
                Err(error) => return Err(refusal(format!("reference home {symbol:?}: {error}"))),
            }
        }
        if homes.is_empty() {
            return Ok(None);
        }
        Ok(Some(super::owners::Home {
            source_items: BTreeSet::new(),
            path: symbol.into(),
            packages: BTreeSet::new(),
            units: memory
                .collect(homes.ordered().map(|(_, unit)| *unit))
                .map_err(|source| Error::SourceIndexStorage { source })?,
        }))
    }

    pub(super) fn unclassified(
        census: &Census,
        reports: &[JobReport],
    ) -> Result<Vec<String>, Error> {
        let mut resident = Resident;
        Memory::new(&mut resident)
            .collect(
                census
                    .reference_witnesses
                    .iter()
                    .filter(|witness| {
                        !reports.iter().any(|job| {
                            job.matches.iter().any(|row| {
                                row.class == MatchClass::Home
                                    && row.symbol == witness.reference.symbol
                                    && row.file == witness.reference.file
                                    && row.start_byte == witness.reference.start_byte
                            })
                        })
                    })
                    .map(|witness| {
                        format!(
                            "REFERENCE-COPY {}:{} {} matches {} {}",
                            witness.consumer.file,
                            witness.consumer.line,
                            witness.consumer.symbol,
                            witness.reference.symbol,
                            witness.fingerprint
                        )
                    }),
            )
            .map_err(|source| Error::SourceIndexStorage { source })
    }
}
