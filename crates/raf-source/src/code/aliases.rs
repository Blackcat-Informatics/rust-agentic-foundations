// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Bounded alias-member projection through the existing native symbol queries.

use purrdf_lex::allocation::{Admission, Memory, StorageError};

use super::{CodeResolveError, CodeResolver, Heads, ItemAdmission, Meter};
use crate::facts::PathFact;
use crate::meter::Steps;
use crate::{
    ItemId, Namespace, Resolution, ResolveError, SourceAuthorityId, SymbolKind, TestRole, Usage,
};

enum Failure {
    Code(CodeResolveError),
    Storage(StorageError),
    Outside(CodeResolveError),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AliasPurpose {
    Member,
    Enum,
    FirstParty,
}

enum Destination {
    Alias,
    Nominal(usize),
    Outside(&'static str),
}

const ALIAS_SYNTAX: &str = "supply a zero-parameter plain alias with literal const arguments; general alias substitution is not modeled";
const NOMINAL_ARITY: &str = "alias arguments do not establish the nominal const arity";

fn alias_arity(item: &crate::Item) -> Option<usize> {
    item.detail.as_ref()?.alias_const_arity
}

fn destination(item: &crate::Item, arguments: usize, purpose: AliasPurpose) -> Destination {
    if purpose == AliasPurpose::FirstParty
        && item.file.authority() != SourceAuthorityId::FIRST_PARTY
    {
        return Destination::Outside("target is outside the first-party source");
    }
    match item.kind {
        SymbolKind::TypeAlias if arguments != 0 => Destination::Outside(
            "generic arguments on an alias need explicit substitution evidence",
        ),
        SymbolKind::TypeAlias
            if purpose == AliasPurpose::FirstParty && alias_arity(item).is_none() =>
        {
            Destination::Outside(ALIAS_SYNTAX)
        }
        SymbolKind::TypeAlias => Destination::Alias,
        SymbolKind::Struct | SymbolKind::Enum | SymbolKind::Union => item
            .detail
            .as_ref()
            .and_then(|detail| detail.nominal_const_arity)
            .map_or(Destination::Outside(NOMINAL_ARITY), Destination::Nominal),
        _ => Destination::Outside(NOMINAL_ARITY),
    }
}

fn alias_path<S: Admission + ?Sized>(
    rhs: &PathFact,
    memory: &mut Memory<'_, S>,
) -> Result<String, StorageError> {
    let mut path = String::new();
    if rhs.leading_colon {
        memory.push_str(&mut path, "::")?;
    }
    for (index, segment) in rhs.segments.iter().enumerate() {
        if index > 0 {
            memory.push_str(&mut path, "::")?;
        }
        memory.push_str(&mut path, segment)?;
    }
    Ok(path)
}

impl From<CodeResolveError> for Failure {
    fn from(error: CodeResolveError) -> Self {
        Self::Code(error)
    }
}
impl From<StorageError> for Failure {
    fn from(error: StorageError) -> Self {
        Self::Storage(error)
    }
}

fn unsupported(alias: ItemId, member: &str, reason: &'static str) -> CodeResolveError {
    CodeResolveError::AliasMember {
        alias,
        member: member.into(),
        reason,
    }
}
fn budget(error: crate::LoadError) -> CodeResolveError {
    CodeResolveError::Query(Box::new(error))
}

impl CodeResolver<'_> {
    /// Resolve a plain type alias to its enum through unconditional named paths.
    /// The query admits the same bounded const argument forms as associated
    /// member lookup. Other nominal types supply no enum identity.
    ///
    /// # Errors
    /// Refuses an absent alias, an uncertain or unsupported alias chain and
    /// exhausted query resources.
    pub fn unconditional_enum_alias(
        &self,
        alias: ItemId,
    ) -> Result<Option<ItemId>, CodeResolveError> {
        self.unconditional_alias(alias, AliasPurpose::Enum)
            .map(|(nominal, _)| {
                nominal.filter(|id| self.index.items[id.index()].kind == SymbolKind::Enum)
            })
    }

    /// Resolve a written first-party alias to its indexed struct, enum or union.
    ///
    /// Returns no identity for an unindexed target or a form outside the
    /// zero-parameter named alias and literal const argument profile. The
    /// returned usage includes the work of an unsuccessful supported probe.
    ///
    /// # Errors
    /// Refuses absent items, ambiguous, cyclic or conditional indexed routes,
    /// broken explicitly local paths and exhausted query resources.
    pub fn unconditional_nominal_alias(
        &self,
        alias: ItemId,
    ) -> Result<(Option<ItemId>, Usage), CodeResolveError> {
        self.unconditional_alias(alias, AliasPurpose::FirstParty)
    }

    fn unconditional_alias(
        &self,
        alias: ItemId,
        purpose: AliasPurpose,
    ) -> Result<(Option<ItemId>, Usage), CodeResolveError> {
        let item = self
            .index
            .items
            .get(alias.index())
            .ok_or(CodeResolveError::UnknownItem(alias))?;
        if item.kind != SymbolKind::TypeAlias {
            return Ok((None, Usage::default()));
        }
        let meter = Meter::resume_query(
            self.options,
            Usage {
                item_bytes: self.usage.item_bytes,
                ..Usage::default()
            },
        );
        let steps = Steps::QueryItems {
            meter: &meter,
            file: &item.file,
            line: Some(item.name_line),
        };
        let mut admission = ItemAdmission::new(&meter, &item.file);
        let result = self.alias_nominal(
            alias,
            if purpose == AliasPurpose::Enum {
                "enum"
            } else {
                "type"
            },
            &meter,
            &mut Memory::new(&mut admission),
            steps,
            purpose,
        );
        match result {
            Ok((id, _, _)) => Ok((Some(id), meter.usage())),
            Err(Failure::Outside(_)) if purpose == AliasPurpose::FirstParty => {
                Ok((None, meter.usage()))
            }
            Err(Failure::Code(error) | Failure::Outside(error)) => Err(error),
            Err(Failure::Storage(error)) => Err(budget(admission.error(error))),
        }
    }

    /// Require a method whose self type applies without generic inference.
    ///
    /// Admits direct nominal types with no parameters, or the exact forwarding
    /// of all unconstrained nominal const parameters. General bounds, defaults,
    /// type/lifetime substitution and aliased impl headers need more evidence.
    /// Configuration and unexpanded declaration attributes remain caller audits.
    ///
    /// # Errors
    /// Refuses absent items and every method outside that proven header subset.
    pub fn require_unconditional_inherent_method(
        &self,
        site: ItemId,
    ) -> Result<(), CodeResolveError> {
        let meter = Meter::resume_query(
            self.options,
            Usage {
                item_bytes: self.usage.item_bytes,
                ..Usage::default()
            },
        );
        let item = self
            .index
            .items
            .get(site.index())
            .ok_or(CodeResolveError::UnknownItem(site))?;
        let steps = Steps::QueryItems {
            meter: &meter,
            file: &item.file,
            line: Some(item.name_line),
        };
        self.require_unconditional_inherent_method_counted(site, steps)
    }

    fn require_unconditional_inherent_method_counted(
        &self,
        site: ItemId,
        steps: Steps<'_>,
    ) -> Result<(), CodeResolveError> {
        let item = self
            .index
            .items
            .get(site.index())
            .ok_or(CodeResolveError::UnknownItem(site))?;
        if item.kind != SymbolKind::InherentMethod
            || !self
                .index
                .unconditional_method(site, steps)
                .map_err(CodeResolveError::Resolve)?
        {
            return Err(CodeResolveError::MethodApplicability(site));
        }
        Ok(())
    }

    pub(super) fn resolve_code_at(
        &self,
        module: usize,
        path: &str,
        namespace: Namespace,
        heads: Heads,
    ) -> Result<(Resolution, Usage), CodeResolveError> {
        let retained = Usage {
            item_bytes: self.usage.item_bytes,
            ..Usage::default()
        };
        self.resolve_code_after(module, path, namespace, heads, retained)
    }

    pub(super) fn resolve_code_after(
        &self,
        module: usize,
        path: &str,
        namespace: Namespace,
        heads: Heads,
        retained: Usage,
    ) -> Result<(Resolution, Usage), CodeResolveError> {
        let (site, line) = self.index.module_site(module);
        let meter = Meter::resume_query(self.options, retained);
        meter.work(0, &site, line).map_err(budget)?;
        let steps = Steps::QueryItems {
            meter: &meter,
            file: &site,
            line,
        };
        let missing = match self
            .index
            .resolve_at(module, path, Some(namespace), heads, steps)
        {
            Ok(resolution) => return Ok((resolution, meter.usage())),
            Err(error @ ResolveError::NotFound(_)) => error,
            Err(error) => return Err(CodeResolveError::Resolve(error)),
        };
        let Some((prefix, member)) = path
            .rsplit_once("::")
            .filter(|(prefix, _)| !prefix.is_empty() && namespace == Namespace::Value)
        else {
            return Err(CodeResolveError::Resolve(missing));
        };
        let prefix =
            match self
                .index
                .resolve_at(module, prefix, Some(Namespace::Type), heads, steps)
            {
                Ok(resolution) => resolution,
                Err(ResolveError::NotFound(_)) => return Err(CodeResolveError::Resolve(missing)),
                Err(error) => return Err(CodeResolveError::Resolve(error)),
            };
        if !prefix
            .sites
            .iter()
            .any(|id| self.index.items[id.index()].kind == SymbolKind::TypeAlias)
        {
            return Err(CodeResolveError::Resolve(missing));
        }
        if prefix.sites.len() != 1 {
            return Err(CodeResolveError::Resolve(ResolveError::Ambiguous(
                prefix.sites,
            )));
        }
        let alias = prefix.sites[0];
        let file = &self.index.items[alias.index()].file;
        meter
            .work(path.len().saturating_mul(2), file, None)
            .map_err(budget)?;
        let mut admission = ItemAdmission::new(&meter, file);
        let result = {
            let mut memory = Memory::new(&mut admission);
            self.alias_member(alias, member, &meter, &mut memory, steps)
        };
        match result {
            Ok(mut resolution) => {
                resolution.via_glob |= prefix.via_glob;
                Ok((resolution, meter.usage()))
            }
            Err(Failure::Code(error) | Failure::Outside(error)) => Err(error),
            Err(Failure::Storage(error)) => Err(budget(admission.error(error))),
        }
    }

    fn alias_member<S: Admission + ?Sized>(
        &self,
        first: ItemId,
        member: &str,
        meter: &Meter,
        memory: &mut Memory<'_, S>,
        steps: Steps<'_>,
    ) -> Result<Resolution, Failure> {
        let (nominal, aliases, carried) =
            self.alias_nominal(first, member, meter, memory, steps, AliasPurpose::Member)?;
        let item = &self.index.items[nominal.index()];
        let home = self.homes[nominal.index()]
            .module()
            .ok_or(CodeResolveError::MissingContext(nominal))?;
        let path = memory.format(&format_args!("{}::{member}", item.name))?;
        meter.work(path.len(), &item.file, None).map_err(budget)?;
        let mut resolution = self
            .index
            .resolve_at(
                home,
                &path,
                Some(Namespace::Value),
                Heads::Code {
                    test: item.test != TestRole::None,
                },
                steps,
            )
            .map_err(CodeResolveError::Resolve)?;
        for id in &resolution.sites {
            match self.require_unconditional_inherent_method_counted(*id, steps) {
                Ok(()) => {}
                Err(CodeResolveError::MethodApplicability(_)) => {
                    return Err(unsupported(
                        first,
                        member,
                        "requires an unconditionally applicable inherent method on the exact nominal type",
                    ).into());
                }
                Err(error) => return Err(error.into()),
            }
        }
        resolution.via_glob |= carried;
        resolution.type_aliases = aliases;
        Ok(resolution)
    }

    fn alias_nominal<S: Admission + ?Sized>(
        &self,
        first: ItemId,
        member: &str,
        meter: &Meter,
        memory: &mut Memory<'_, S>,
        steps: Steps<'_>,
        purpose: AliasPurpose,
    ) -> Result<(ItemId, Vec<ItemId>, bool), Failure> {
        let mut current = first;
        let mut aliases = Vec::new();
        let mut carried = false;
        loop {
            let item = &self.index.items[current.index()];
            let declared = current;
            meter
                .work(aliases.len().saturating_add(1), &item.file, None)
                .map_err(budget)?;
            if purpose == AliasPurpose::FirstParty
                && item.file.authority() != SourceAuthorityId::FIRST_PARTY
            {
                return Err(Failure::Outside(unsupported(
                    current,
                    member,
                    "alias is outside the first-party source",
                )));
            }
            if aliases.contains(&current) {
                return Err(
                    CodeResolveError::Resolve(ResolveError::Cyclic(item.path.to_string())).into(),
                );
            }
            let arguments = alias_arity(item)
                .ok_or_else(|| Failure::Outside(unsupported(current, member, ALIAS_SYNTAX)))?;
            let (home, rhs, configuration) = self
                .index
                .aliases
                .get(&current)
                .ok_or_else(|| unsupported(current, member, "plain alias target is absent"))?;
            memory.push(&mut aliases, current)?;
            let path = alias_path(rhs, memory)?;
            meter.work(path.len(), &item.file, None).map_err(budget)?;
            let heads = Heads::Code {
                test: item.test != TestRole::None,
            };
            let resolution =
                match self
                    .index
                    .resolve_at(*home, &path, Some(Namespace::Type), heads, steps)
                {
                    Err(error @ ResolveError::NotFound(_))
                        if purpose == AliasPurpose::FirstParty
                            && !self.indexed_alias_head(*home, &path, heads, steps)? =>
                    {
                        return Err(Failure::Outside(CodeResolveError::Resolve(error)));
                    }
                    result => result.map_err(CodeResolveError::Resolve)?,
                };
            if resolution.sites.len() != 1 {
                return Err(
                    CodeResolveError::Resolve(ResolveError::Ambiguous(resolution.sites)).into(),
                );
            }
            carried |= resolution.via_glob;
            current = resolution.sites[0];
            let target = &self.index.items[current.index()];
            let next = destination(target, arguments, purpose);
            if purpose == AliasPurpose::FirstParty
                && let Destination::Outside(reason) = next
            {
                return Err(Failure::Outside(unsupported(declared, member, reason)));
            }
            if purpose != AliasPurpose::Member
                && (*configuration == crate::facts::ConfigurationDependence::Conditional
                    || !self
                        .index
                        .module_is_unconditional(*home, steps)
                        .map_err(CodeResolveError::Resolve)?
                    || self
                        .index
                        .unconditional_type_path(*home, &path, heads, steps)
                        .map_err(CodeResolveError::Resolve)?
                        != Some(current))
            {
                return Err(unsupported(
                    declared,
                    member,
                    "alias route is not unconditionally selected",
                )
                .into());
            }
            match next {
                Destination::Alias => {}
                Destination::Nominal(arity) if arity == arguments => {
                    return Ok((current, aliases, carried));
                }
                Destination::Nominal(_) => {
                    return Err(unsupported(declared, member, NOMINAL_ARITY).into());
                }
                Destination::Outside(reason) => {
                    return Err(Failure::Outside(unsupported(declared, member, reason)));
                }
            }
        }
    }

    fn indexed_alias_head(
        &self,
        home: usize,
        path: &str,
        heads: Heads,
        steps: Steps<'_>,
    ) -> Result<bool, CodeResolveError> {
        let written = path.strip_prefix("::").unwrap_or(path);
        let Some((head, _)) = written.split_once("::") else {
            return Ok(false);
        };
        if matches!(head, "crate" | "self" | "super") {
            return Ok(true);
        }
        let head = &path[..path.len() - written.len() + head.len()];
        match self
            .index
            .resolve_at(home, head, Some(Namespace::Type), heads, steps)
        {
            Ok(resolution) => Ok(resolution.sites.iter().any(|id| {
                self.index.items[id.index()].file.authority() == SourceAuthorityId::FIRST_PARTY
            })),
            Err(ResolveError::NotFound(_)) => Ok(false),
            Err(error) => Err(CodeResolveError::Resolve(error)),
        }
    }
}
