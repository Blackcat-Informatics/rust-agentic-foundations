// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Map every source reach to the one physical census owner.

use std::collections::{BTreeMap, BTreeSet};

use raf_source::{SymbolKind, Workspace};

use crate::{Census, Unit};

pub(super) type Owners<'a> = BTreeMap<(String, usize), &'a Unit>;

pub(super) fn index(census: &Census) -> Owners<'_> {
    census
        .units()
        .iter()
        .map(|unit| ((unit.file.clone(), unit.start_byte), unit))
        .collect()
}

pub(super) fn resolve<'a>(
    workspace: &Workspace,
    owners: &Owners<'a>,
    symbol: &str,
) -> Result<Vec<&'a Unit>, String> {
    let resolution = workspace
        .resolve(symbol)
        .map_err(|error| error.to_string())?;
    let mut units = BTreeMap::new();
    for site in resolution.sites {
        let item = workspace
            .item(site)
            .ok_or_else(|| format!("source site {site:?} is absent"))?;
        let key = (item.file.to_string(), item.span.start_byte);
        let unit = owners.get(&key).ok_or_else(|| {
            format!(
                "{symbol:?} does not name a census unit at {}:{}",
                item.file, item.span.start_byte
            )
        })?;
        units.insert(key, *unit);
    }
    if units.is_empty() {
        return Err(format!("{symbol:?} has no census source sites"));
    }
    Ok(units.into_values().collect())
}

pub(super) fn key(unit: &Unit) -> (String, String, usize) {
    (unit.symbol.clone(), unit.file.clone(), unit.start_byte)
}

pub(super) struct Home<'a> {
    pub(super) path: String,
    pub(super) packages: BTreeSet<String>,
    pub(super) units: Vec<&'a Unit>,
}

/// A home may be an implementation item or a module containing implementations.
/// Distinct members and variants continue to require one actual unit path.
pub(super) fn home<'a>(
    workspace: &Workspace,
    owners: &Owners<'a>,
    symbol: &str,
) -> Result<Home<'a>, String> {
    let resolution = workspace
        .resolve(symbol)
        .map_err(|error| error.to_string())?;
    let mut packages = BTreeSet::new();
    let mut units = BTreeMap::new();
    let mut module = false;
    for site in &resolution.sites {
        let item = workspace
            .item(*site)
            .ok_or_else(|| format!("source site {site:?} is absent"))?;
        let target = workspace
            .target(item.target)
            .ok_or_else(|| format!("{symbol:?} has no owning source target"))?;
        packages.insert(target.package.clone());
        if item.kind == SymbolKind::Mod {
            module = true;
            for child in workspace.items_under(item.path.as_str()) {
                if let Some(unit) = owners.get(&(child.file.to_string(), child.span.start_byte)) {
                    units.insert(key(unit), *unit);
                }
            }
        } else {
            let unit = owners
                .get(&(item.file.to_string(), item.span.start_byte))
                .ok_or_else(|| {
                    format!("{symbol:?} does not name an implementation unit or module")
                })?;
            units.insert(key(unit), *unit);
        }
    }
    if packages.is_empty() {
        return Err(format!("{symbol:?} has no source definition"));
    }
    let path = if module {
        resolution.path
    } else {
        units.values().next().expect("resolved unit").symbol.clone()
    };
    Ok(Home {
        path,
        packages,
        units: units.into_values().collect(),
    })
}
