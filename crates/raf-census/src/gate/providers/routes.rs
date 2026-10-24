// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Provider routes consume actual admitted Source libraries, never written guesses.

use purrdf_jsonschema::ecma::MatchLimits;
use purrdf_lex::allocation::OrderedTable;
use purrdf_lex::walk::WorkList;
use raf_source::{CodeResolver, ExternLibrary, ItemId, Namespace, SymbolKind, Workspace};

use super::{charge, configured, failure, path_error, storage};
use crate::{Error, LedgerJob};

#[derive(Default)]
pub(super) struct Allowed<'a> {
    pub(super) sites: OrderedTable<ItemId, ()>,
    pub(super) aliases: OrderedTable<ItemId, ()>,
    pub(super) apis: WorkList<(&'a str, ItemId), 8>,
}

fn selects(selector: &str, alias: &str, package: &str) -> bool {
    if selector == alias || selector == package || selector.replace('-', "_") == alias {
        return true;
    }
    selector.strip_suffix(".*").is_some_and(|family| {
        package == family
            || package
                .strip_prefix(family)
                .is_some_and(|tail| tail.starts_with('-') || tail.starts_with('_'))
    })
}

fn roots<'a>(
    workspace: &Workspace,
    code: &'a CodeResolver<'a>,
    row: &LedgerJob,
    owner: ItemId,
    limits: &mut MatchLimits,
) -> Result<WorkList<ExternLibrary<'a>, 8>, Error> {
    let provider = row.preferred_provider.as_ref().expect("provider job");
    let item = workspace
        .item(owner)
        .ok_or_else(|| failure(&row.job.id, "missing consumer"))?;
    let byte = item
        .body
        .as_ref()
        .ok_or_else(|| failure(&row.job.id, "consumer has no body"))?
        .start_byte
        + 1;
    let libraries = code
        .extern_libraries_at(owner, byte)
        .map_err(|error| path_error(workspace, owner, &row.job.id, &provider.package, error))?;
    let mut roots = WorkList::new();
    for library in libraries {
        let package = workspace
            .package(library.package())
            .ok_or_else(|| failure(&row.job.id, "admitted library has no package"))?;
        charge(
            &row.job.id,
            limits,
            (provider.package.len() + library.rust_name().len() + package.name.len()) as u64 + 1,
        )?;
        if !selects(&provider.package, library.rust_name(), &package.name) {
            continue;
        }
        configured(workspace, library.root(), &row.job.id)?;
        if roots.len() >= limits.states {
            return Err(failure(
                &row.job.id,
                "provider library-state limit exhausted",
            ));
        }
        roots
            .try_push(library)
            .map_err(|source| Error::PatternStorage {
                pattern: format!("provider:{}", row.job.id),
                source,
            })?;
    }
    if roots.is_empty() {
        return Err(failure(
            &row.job.id,
            &format!(
                "provider {:?} has no admitted dependency route from this consumer; supply its exact locked library source",
                provider.package
            ),
        ));
    }
    Ok(roots)
}

pub(super) fn allowed<'a>(
    workspace: &Workspace,
    code: &CodeResolver<'_>,
    row: &'a LedgerJob,
    owner: ItemId,
    limits: &mut MatchLimits,
) -> Result<Allowed<'a>, Error> {
    let roots = roots(workspace, code, row, owner, limits)?;
    let provider = row.preferred_provider.as_ref().expect("provider job");
    let mut allowed = Allowed::default();
    for api in &provider.entry_points {
        charge(&row.job.id, limits, api.len() as u64 + 1)?;
        let (head, tail) = api
            .trim_start_matches("::")
            .split_once("::")
            .ok_or_else(|| {
                failure(
                    &row.job.id,
                    "provider entry point must be a library-qualified API path",
                )
            })?;
        let head = head.strip_prefix("r#").unwrap_or(head);
        let mut actual = None;
        for library in roots.iter() {
            let target = workspace
                .target(library.target())
                .ok_or_else(|| failure(&row.job.id, "admitted library target is absent"))?;
            charge(
                &row.job.id,
                limits,
                (head.len() + library.rust_name().len() + target.crate_path.len()) as u64 + 1,
            )?;
            if head != library.rust_name() && head != target.crate_path {
                continue;
            }
            let resolution = library
                .resolve(tail, Namespace::Value)
                .map_err(|error| path_error(workspace, owner, &row.job.id, api, error))?;
            for alias in resolution.type_aliases {
                charge(&row.job.id, limits, 1)?;
                configured(workspace, alias, &row.job.id)?;
                if allowed.aliases.get(&alias).is_none() && allowed.aliases.len() >= limits.states {
                    return Err(failure(&row.job.id, "provider alias-state limit exhausted"));
                }
                storage::insert(&mut allowed.aliases, alias, (), &row.job.id)?;
            }
            let site = definition(workspace, code, row, owner, api, &resolution.sites, limits)?;
            if actual.is_some_and(|previous| previous != site) {
                return Err(failure(
                    &row.job.id,
                    &format!("provider API {api:?} has missing or ambiguous definition identity"),
                ));
            }
            actual = Some(site);
        }
        let actual = actual.ok_or_else(|| {
            failure(
                &row.job.id,
                &format!("provider API {api:?} is not rooted in an admitted provider library"),
            )
        })?;
        let site = actual;
        if allowed.apis.len() >= limits.states
            || (allowed.sites.get(&site).is_none() && allowed.sites.len() >= limits.states)
        {
            return Err(failure(&row.job.id, "provider API-state limit exhausted"));
        }
        allowed
            .apis
            .try_push((api.as_str(), site))
            .map_err(|source| Error::PatternStorage {
                pattern: format!("provider:{}", row.job.id),
                source,
            })?;
        storage::insert(&mut allowed.sites, site, (), &row.job.id)?;
    }
    Ok(allowed)
}

fn definition(
    workspace: &Workspace,
    code: &CodeResolver<'_>,
    row: &LedgerJob,
    owner: ItemId,
    api: &str,
    sites: &[ItemId],
    limits: &mut MatchLimits,
) -> Result<ItemId, Error> {
    let mut selected = None;
    for site in sites.iter().copied() {
        charge(&row.job.id, limits, 1)?;
        configured(workspace, site, &row.job.id)?;
        let item = workspace
            .item(site)
            .ok_or_else(|| failure(&row.job.id, "provider API source is absent"))?;
        if !matches!(item.kind, SymbolKind::Fn | SymbolKind::InherentMethod) || item.body.is_none()
        {
            return Err(failure(
                &row.job.id,
                "provider API must name an actual supported function definition, not an unproved callable value",
            ));
        }
        if item.kind == SymbolKind::InherentMethod {
            code.require_unconditional_inherent_method(site)
                .map_err(|error| path_error(workspace, owner, &row.job.id, api, error))?;
        }
        if selected.is_some_and(|previous| previous != site) {
            return Err(failure(
                &row.job.id,
                &format!("provider API {api:?} has missing or ambiguous definition identity"),
            ));
        }
        selected = Some(site);
    }
    selected.ok_or_else(|| {
        failure(
            &row.job.id,
            &format!("provider API {api:?} has missing or ambiguous definition identity"),
        )
    })
}
