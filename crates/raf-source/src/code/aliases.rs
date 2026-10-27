// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Bounded alias-member projection through the existing native symbol queries.

use purrdf_lex::allocation::{Admission, Memory, StorageError};

use super::{CodeResolveError, CodeResolver, Heads, ItemAdmission, Meter};
use crate::meter::Steps;
use crate::{ItemId, Namespace, Resolution, ResolveError, SymbolKind, TestRole, Usage};

enum Failure {
    Code(CodeResolveError),
    Storage(StorageError),
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
            Err(Failure::Code(error)) => Err(error),
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
            self.alias_nominal(first, member, meter, memory, steps)?;
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
            if aliases.contains(&current) {
                return Err(
                    CodeResolveError::Resolve(ResolveError::Cyclic(item.path.to_string())).into(),
                );
            }
            let arguments = item.detail.as_ref().and_then(|detail| detail.alias_const_arity).ok_or_else(|| unsupported(current, member, "supply a zero-parameter plain alias with literal const arguments; general alias substitution is not modeled"))?;
            let (home, rhs) = self
                .index
                .aliases
                .get(&current)
                .ok_or_else(|| unsupported(current, member, "plain alias target is absent"))?;
            memory.push(&mut aliases, current)?;
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
            meter.work(path.len(), &item.file, None).map_err(budget)?;
            let resolution = self
                .index
                .resolve_at(
                    *home,
                    &path,
                    Some(Namespace::Type),
                    Heads::Code {
                        test: item.test != TestRole::None,
                    },
                    steps,
                )
                .map_err(CodeResolveError::Resolve)?;
            if resolution.sites.len() != 1 {
                return Err(
                    CodeResolveError::Resolve(ResolveError::Ambiguous(resolution.sites)).into(),
                );
            }
            carried |= resolution.via_glob;
            current = resolution.sites[0];
            let target = &self.index.items[current.index()];
            if target.kind == SymbolKind::TypeAlias {
                if arguments != 0 {
                    return Err(unsupported(
                        declared,
                        member,
                        "generic arguments on an alias need explicit substitution evidence",
                    )
                    .into());
                }
                continue;
            }
            if !matches!(
                target.kind,
                SymbolKind::Struct | SymbolKind::Enum | SymbolKind::Union
            ) || target
                .detail
                .as_ref()
                .and_then(|detail| detail.nominal_const_arity)
                != Some(arguments)
            {
                return Err(unsupported(
                    declared,
                    member,
                    "alias arguments do not establish the nominal const arity",
                )
                .into());
            }
            return Ok((current, aliases, carried));
        }
    }
}
