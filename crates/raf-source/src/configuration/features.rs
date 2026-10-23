// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Manifest-specific feature implications, never inferred from dependency names.
use super::{
    ActiveConfiguration, BTreeMap, DependencyDeclaration, ItemAdmission, LoadError, Memory, Meter,
    RelPath, contains, failure, native, recover, sort_unique, try_equal_range_by,
};
use purrdf_lex::allocation::Admission;

pub(super) fn admit<S: Admission + ?Sized>(
    enabled: &[&str],
    graph: &BTreeMap<String, Vec<String>>,
    deps: &[DependencyDeclaration],
    meter: &Meter,
    path: &RelPath,
    memory: &mut Memory<'_, S>,
) -> Result<(Vec<String>, Vec<String>), LoadError> {
    let mut features = Vec::new();
    let mut optional = Vec::new();
    for name in enabled {
        valid_name(name, meter, path)?;
        let name = memory.string(name).map_err(native)?;
        memory.push(&mut features, name).map_err(native)?;
    }
    sort_unique(&mut features, meter, path)?;
    // Validate every declared reference, even when its feature is disabled.
    for (name, terms) in graph {
        valid_name(name, meter, path)?;
        for term in terms {
            validate_term(term, graph, deps, meter, path)?;
        }
    }
    for feature in &features {
        if let Some(terms) = graph.get(feature) {
            for term in terms {
                meter.work(term.len().saturating_add(1), path, None)?;
                if let Some(alias) = term.strip_prefix("dep:") {
                    activate(&mut optional, alias, meter, path, memory)?;
                } else if let Some((alias, _)) = term.split_once('/') {
                    if !alias.ends_with('?') && optional_alias(alias, deps, meter, path)? {
                        activate(&mut optional, alias, meter, path, memory)?;
                        if !suppressed(alias, graph, meter, path)?
                            && !contains(&features, alias, meter, path)?
                        {
                            return Err(failure(
                                path,
                                "enabled dependency feature requires its implicit optional feature in the complete selection",
                            ));
                        }
                    }
                } else if !contains(&features, term, meter, path)? {
                    return Err(failure(
                        path,
                        "declared feature selection is missing an implied feature",
                    ));
                }
            }
        } else if optional_alias(feature, deps, meter, path)?
            && !suppressed(feature, graph, meter, path)?
        {
            activate(&mut optional, feature, meter, path, memory)?;
        } else {
            return Err(failure(
                path,
                "unknown enabled feature; use the actual package feature declarations",
            ));
        }
    }
    Ok((features, optional))
}
fn valid_name(name: &str, meter: &Meter, path: &RelPath) -> Result<(), LoadError> {
    let Some(first) = name.chars().next() else {
        return Err(failure(path, "empty feature name"));
    };
    if u64::try_from(name.len()).unwrap_or(u64::MAX) > meter.options().max_file_bytes {
        return Err(failure(
            path,
            "feature name exceeds the original file-byte bound",
        ));
    }
    meter.work(name.len().saturating_mul(2).saturating_add(1), path, None)?;
    let mut admission = ItemAdmission::new(meter, path);
    let mut memory = Memory::new(&mut admission);
    let result = (|| {
        if !first.is_ascii_digit() {
            let start = memory
                .format(&format_args!("{first}__cargo_feature"))
                .map_err(native)?;
            if syn::parse_str::<syn::Ident>(&start).is_err() {
                return Err(failure(
                    path,
                    "feature name has no Unicode XID start or allowed digit/underscore",
                ));
            }
        }
        let mut normalized = memory.string("_").map_err(native)?;
        for ch in name.chars() {
            let ch = if matches!(ch, '-' | '+' | '.') {
                '_'
            } else {
                ch
            };
            let mut bytes = [0; 4];
            memory
                .push_str(&mut normalized, ch.encode_utf8(&mut bytes))
                .map_err(native)?;
        }
        if syn::parse_str::<syn::Ident>(&normalized).is_err() {
            return Err(failure(
                path,
                "feature name contains characters outside Cargo's Unicode XID grammar",
            ));
        }
        Ok(())
    })();
    result.map_err(|error| recover(error, &mut admission))
}
fn validate_term(
    term: &str,
    graph: &BTreeMap<String, Vec<String>>,
    deps: &[DependencyDeclaration],
    meter: &Meter,
    path: &RelPath,
) -> Result<(), LoadError> {
    meter.work(term.len().saturating_add(1), path, None)?;
    if let Some(alias) = term.strip_prefix("dep:") {
        valid_name(alias, meter, path)?;
        if !optional_alias(alias, deps, meter, path)? {
            return Err(failure(
                path,
                "dep: feature reference must name an actual optional dependency alias",
            ));
        }
    } else if let Some((alias, feature)) = term.split_once('/') {
        let alias = alias.strip_suffix('?').unwrap_or(alias);
        valid_name(alias, meter, path)?;
        valid_name(feature, meter, path)?;
        if !dependency_alias(alias, deps, meter, path)? {
            return Err(failure(
                path,
                "feature forwarding names no actual dependency alias",
            ));
        }
        if term.contains("?/") && !optional_alias(alias, deps, meter, path)? {
            return Err(failure(
                path,
                "weak feature forwarding needs an optional dependency",
            ));
        }
    } else {
        valid_name(term, meter, path)?;
        if graph.contains_key(term) {
            return Ok(());
        }
        if !optional_alias(term, deps, meter, path)? || suppressed(term, graph, meter, path)? {
            return Err(failure(
                path,
                "feature graph references an unknown or suppressed feature",
            ));
        }
    }
    Ok(())
}
fn suppressed(
    alias: &str,
    graph: &BTreeMap<String, Vec<String>>,
    meter: &Meter,
    path: &RelPath,
) -> Result<bool, LoadError> {
    for terms in graph.values() {
        for term in terms {
            meter.work(
                term.len().saturating_add(alias.len()).saturating_add(1),
                path,
                None,
            )?;
            if term.strip_prefix("dep:") == Some(alias) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
fn optional_alias(
    alias: &str,
    deps: &[DependencyDeclaration],
    meter: &Meter,
    path: &RelPath,
) -> Result<bool, LoadError> {
    for dependency in deps {
        meter.work(
            dependency
                .name
                .len()
                .saturating_add(alias.len())
                .saturating_add(1),
            path,
            None,
        )?;
        if dependency.name == alias && dependency.specification.optional == Some(true) {
            return Ok(true);
        }
    }
    Ok(false)
}
fn dependency_alias(
    alias: &str,
    deps: &[DependencyDeclaration],
    meter: &Meter,
    path: &RelPath,
) -> Result<bool, LoadError> {
    for dependency in deps {
        meter.work(
            dependency
                .name
                .len()
                .saturating_add(alias.len())
                .saturating_add(1),
            path,
            None,
        )?;
        if dependency.name == alias {
            return Ok(true);
        }
    }
    Ok(false)
}
fn activate<S: Admission + ?Sized>(
    active: &mut Vec<String>,
    alias: &str,
    meter: &Meter,
    path: &RelPath,
    memory: &mut Memory<'_, S>,
) -> Result<(), LoadError> {
    let found = try_equal_range_by(active, |known| {
        meter.work(
            known.len().saturating_add(alias.len()).saturating_add(1),
            path,
            None,
        )?;
        Ok::<_, LoadError>(known.as_str().cmp(alias))
    })?;
    if found.is_empty() {
        let alias = memory.string(alias).map_err(native)?;
        memory.reserve_for_push(active).map_err(native)?;
        meter.work(active.len().saturating_sub(found.start), path, None)?;
        active.insert(found.start, alias);
    }
    Ok(())
}

pub(crate) fn destination(
    owner: &ActiveConfiguration,
    destination: Option<&ActiveConfiguration>,
    declaration: &DependencyDeclaration,
    graph: &BTreeMap<String, Vec<String>>,
    target_graph: &BTreeMap<String, Vec<String>>,
    meter: &Meter,
    path: &RelPath,
) -> Result<(), LoadError> {
    let local = &declaration.specification;
    let base = declaration
        .inherited
        .as_ref()
        .map(|entry| &entry.specification);
    let defaults = base.map_or(local.default_features.unwrap_or(true), |base| {
        base.default_features.unwrap_or(true) || local.default_features.unwrap_or(false)
    });
    let require = |feature: &str| -> Result<(), LoadError> {
        let active = destination.ok_or_else(|| {
            failure(
                path,
                "destination feature requirements need its explicit complete configuration",
            )
        })?;
        if !contains(&active.features, feature, meter, path)? {
            return Err(failure(
                path,
                "destination configuration is missing a requested, default or forwarded feature",
            ));
        }
        Ok(())
    };
    if defaults && target_graph.contains_key("default") {
        require("default")?;
    }
    for feature in base
        .into_iter()
        .flat_map(|spec| spec.features.iter())
        .chain(&local.features)
    {
        require(feature)?;
    }
    for feature in &owner.features {
        if let Some(terms) = graph.get(feature) {
            for term in terms {
                meter.work(
                    term.len()
                        .saturating_add(declaration.name.len())
                        .saturating_add(1),
                    path,
                    None,
                )?;
                if let Some((alias, feature)) = term.split_once('/') {
                    let weak = alias.ends_with('?');
                    let alias = alias.strip_suffix('?').unwrap_or(alias);
                    if alias == declaration.name
                        && (!weak || contains(&owner.optional, alias, meter, path)?)
                    {
                        require(feature)?;
                    }
                }
            }
        }
    }
    Ok(())
}
