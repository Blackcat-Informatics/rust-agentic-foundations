// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Ledger-designated enum rosters and confirmed shipping value uses.

use std::collections::{BTreeMap, BTreeSet};

use raf_source::{
    CodeResolver, Item, ItemId, Namespace, PackageId, RelPath, SourceAuthorityId, SymbolKind,
    Workspace,
};

use super::{MatchLimits, PathOwner, charge, failure, read_enum_members};
use crate::{Census, Error, Ledger};

pub(super) const RULE: &str = "enum-usage";

pub(in crate::gate) struct EnumUsage<'a> {
    enums: BTreeMap<ItemId, DesignatedEnum>,
    consumers: &'a BTreeSet<PackageId>,
}

struct DesignatedEnum {
    symbol: String,
    file: RelPath,
    line: usize,
    members: BTreeMap<String, VariantUsage>,
}

struct VariantUsage {
    retained: bool,
    observed: bool,
}

impl<'a> EnumUsage<'a> {
    pub(in crate::gate) fn select(
        ledger: &Ledger,
        (workspace, code): (&Workspace, &CodeResolver<'_>),
        (census, consumers): (&Census, &'a BTreeSet<PackageId>),
        limits: &mut MatchLimits,
    ) -> Result<Self, Error> {
        let mut usage = Self {
            enums: BTreeMap::new(),
            consumers,
        };
        let mut capacity = limits.states;
        for (symbol, retained) in &ledger.enum_usage {
            charge(RULE, limits, symbol.len().try_into().unwrap_or(u64::MAX))?;
            let resolution = workspace
                .resolve_in(symbol, Namespace::Type)
                .map_err(|error| failure(RULE, &format!("cannot select {symbol:?}: {error}")))?;
            let [id] = resolution.sites.as_slice() else {
                return Err(failure(
                    RULE,
                    &format!("{symbol:?} must name one indexed enum"),
                ));
            };
            let owner = workspace
                .item(*id)
                .filter(|item| {
                    item.kind == SymbolKind::Enum
                        && usage.shipping(workspace, item)
                        && census.source_context(&item.file).is_some()
                })
                .ok_or_else(|| {
                    failure(
                        RULE,
                        &format!("{symbol:?} must name a selected first-party shipping enum"),
                    )
                })?;
            require_roster_context(owner, code, symbol)?;
            capacity = capacity
                .checked_sub(1)
                .ok_or_else(|| failure(RULE, "enum usage state limit exhausted"))?;
            let members =
                read_enum_members(workspace, owner, RULE, capacity, limits)?.ok_or_else(|| {
                    failure(
                        RULE,
                        &format!("{symbol:?} is excluded by its configuration"),
                    )
                })?;
            capacity -= members.len();
            for (name, member) in &members {
                if member.conditional {
                    return Err(failure(
                        RULE,
                        &format!(
                            "conditional variant {symbol}::{name} leaves the declared roster uncertain"
                        ),
                    ));
                }
            }
            if let Some(name) = retained.iter().find(|name| !members.contains_key(*name)) {
                return Err(failure(
                    RULE,
                    &format!("retention names unknown variant {symbol}::{name}"),
                ));
            }
            let members = members
                .into_keys()
                .map(|name| {
                    let retained = retained.contains(&name);
                    (
                        name,
                        VariantUsage {
                            retained,
                            observed: false,
                        },
                    )
                })
                .collect();
            let declared = DesignatedEnum {
                symbol: owner.path.to_string(),
                file: owner.file.clone(),
                line: owner.name_line,
                members,
            };
            if usage.enums.insert(*id, declared).is_some() {
                return Err(failure(
                    RULE,
                    &format!("{symbol:?} duplicates an enum usage declaration"),
                ));
            }
        }
        Ok(usage)
    }

    pub(super) fn observes(&self) -> bool {
        !self.enums.is_empty()
    }

    pub(super) fn shipping(&self, workspace: &Workspace, item: &Item) -> bool {
        item.file.authority() == SourceAuthorityId::FIRST_PARTY
            && workspace.is_shipping(item)
            && workspace
                .target(item.target)
                .is_some_and(|target| self.consumers.contains(&target.package_id))
    }

    pub(super) fn shipping_context(
        &self,
        workspace: &Workspace,
        file: &raf_source::SourceFile,
        owner: PathOwner<'_>,
    ) -> bool {
        match owner {
            PathOwner::Item(id) => workspace
                .item(id)
                .is_some_and(|item| self.shipping(workspace, item)),
            PathOwner::File(reach) => {
                file.path.authority() == SourceAuthorityId::FIRST_PARTY
                    && !reach.test
                    && workspace.target(reach.target).is_some_and(|target| {
                        target.shipping && self.consumers.contains(&target.package_id)
                    })
            }
            PathOwner::Impl(_) => false,
        }
    }

    pub(super) fn record(
        &mut self,
        enumeration: ItemId,
        member: &str,
        limits: &mut MatchLimits,
    ) -> Result<(), Error> {
        charge(RULE, limits, u64::from(self.enums.len().bit_width()) + 1)?;
        if let Some(declared) = self.enums.get_mut(&enumeration) {
            charge(
                RULE,
                limits,
                u64::from(declared.members.len().bit_width()) + 1,
            )?;
            if let Some(found) = declared.members.get_mut(member) {
                found.observed = true;
            }
        }
        Ok(())
    }

    pub(in crate::gate) fn findings(self) -> Vec<String> {
        let mut findings = Vec::new();
        for declared in self.enums.into_values() {
            for (name, member) in declared.members {
                let message = match (member.observed, member.retained) {
                    (false, false) => "no confirmed shipping value use",
                    (true, true) => "retention is stale because a shipping value use exists",
                    _ => continue,
                };
                findings.push(format!(
                    "{RULE}: {}:{}: {}::{name}: {message}",
                    declared.file, declared.line, declared.symbol
                ));
            }
        }
        findings
    }
}

fn require_roster_context(
    owner: &Item,
    code: &CodeResolver<'_>,
    symbol: &str,
) -> Result<(), Error> {
    if !owner.enum_roster_has_only_written_variants() {
        return Err(failure(
            RULE,
            &format!("{symbol:?} has an attribute that may rewrite its enum roster"),
        ));
    }
    let unconditional = code
        .has_unconditional_reading_module(owner.id, owner.span.start_byte)
        .map_err(|error| {
            failure(
                RULE,
                &format!("cannot inspect declaration {symbol:?}: {error}"),
            )
        })?;
    if !unconditional {
        return Err(failure(
            RULE,
            &format!("{symbol:?} has a conditional module reach"),
        ));
    }
    Ok(())
}
