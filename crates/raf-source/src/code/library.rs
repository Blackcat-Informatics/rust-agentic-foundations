// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Borrowed exact library candidates from the already settled external-name tables.

use super::{CodeResolveError, CodeResolver, Heads};
use crate::meter::{ItemAdmission, Meter};
use crate::{
    ExportOptions, ItemId, LoadError, LoadErrorKind, Namespace, PackageId, RelPath, Resolution,
    ResolveError, SymbolKind, TargetId, TargetKind, TestRole, Usage,
};
use purrdf_lex::allocation::Memory;
use purrdf_lex::walk::{try_dedup_by, try_sort_unstable_by};

#[derive(Clone, Copy)]
enum Group<'a> {
    Prelude(&'a crate::index::ExternName, bool),
    Explicit(&'a [usize]),
}

/// The access route preserved by an admitted library receipt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExternRouteKind {
    /// An absolute external name, with its complete native candidate group.
    Prelude,
    /// A genuine lexical `extern crate` declaration, which can shadow a prelude name.
    Explicit,
}

impl Group<'_> {
    fn scan_len(&self) -> usize {
        match self {
            Self::Prelude(entry, _) => entry.roots.len(),
            Self::Explicit(roots) => roots.len(),
        }
    }
    fn roots(&self) -> impl Iterator<Item = usize> + '_ {
        let (prelude, explicit, test) = match self {
            Self::Prelude(entry, test) => (Some(*entry), None, *test),
            Self::Explicit(roots) => (None, Some(*roots), false),
        };
        prelude
            .into_iter()
            .flat_map(move |entry| entry.roots(test))
            .chain(explicit.into_iter().flat_map(|roots| roots.iter().copied()))
    }
}

/// One actual admitted external library under a Rust name visible in the source model.
///
/// The receipt borrows one code query and keeps its complete original-name candidate
/// group. It is not durable identity or configuration proof. Same-package injected
/// libraries and `extern crate self` roots are excluded.
pub struct ExternLibrary<'a> {
    code: &'a CodeResolver<'a>,
    name: &'a str,
    module: usize,
    group: Group<'a>,
    from: usize,
    test: bool,
}

impl ExternLibrary<'_> {
    /// Whether the receipt represents an absolute name or a lexical declaration.
    #[must_use]
    pub const fn route_kind(&self) -> ExternRouteKind {
        match self.group {
            Group::Prelude(..) => ExternRouteKind::Prelude,
            Group::Explicit(..) => ExternRouteKind::Explicit,
        }
    }
    /// The actual Rust external name, including a custom library or explicit alias.
    #[must_use]
    pub const fn rust_name(&self) -> &str {
        self.name
    }
    /// The actual indexed crate-root definition.
    #[must_use]
    pub fn root(&self) -> ItemId {
        self.code.index.root_items[&self.module]
    }
    /// The actual library target.
    #[must_use]
    pub fn target(&self) -> TargetId {
        self.code.index.modules[self.module].target
    }
    /// The exact load-local package owning that target.
    #[must_use]
    pub fn package(&self) -> PackageId {
        self.code.index.targets[self.target().index()].package_id
    }

    /// Resolve a relative definition that this library actually exports.
    ///
    /// Public reexports retain their actual defining sites. Canonical definition
    /// paths are admitted only when those definitions appear on this library's
    /// public surface. This does not replace lexical classification of a consumer
    /// occurrence. Resolution, surface construction and membership share one query
    /// work grant and retain the reading table's original item-byte allowance,
    /// including hidden but public definitions.
    ///
    /// # Errors
    /// Preserves original external-name ambiguity, native resolution/surface causes,
    /// and an explicit refusal for definitions that are not exported.
    pub fn resolve(
        &self,
        relative: &str,
        namespace: Namespace,
    ) -> Result<Resolution, CodeResolveError> {
        let meter = Meter::resume_query(
            self.code.options,
            Usage {
                item_bytes: self.code.usage.item_bytes,
                ..Usage::default()
            },
        );
        let roots = self.candidate_roots(&meter)?;
        if roots.len() != 1 {
            return Err(CodeResolveError::Resolve(ResolveError::Ambiguous(roots)));
        }
        let (resolved, retained) = self.code.resolve_code_after(
            self.module,
            relative,
            namespace,
            Heads::Local,
            meter.usage(),
        )?;
        let options = ExportOptions {
            include_hidden: true,
        };
        let (surface, usage) = self
            .code
            .workspace
            .exports_after(self.target(), &options, retained)
            .map_err(|error| CodeResolveError::Exports(Box::new(error)))?;
        let meter = Meter::resume_query(self.code.options, usage);
        let target = &self.code.index.targets[self.target().index()];
        for site in resolved.sites.iter().chain(resolved.type_aliases.first()) {
            let mut exported = false;
            for item in &surface.items {
                meter.work(1, &target.root, None).map_err(|error| {
                    CodeResolveError::Exports(Box::new(crate::ExportError::Budget(error)))
                })?;
                if item.item == *site {
                    exported = true;
                    break;
                }
            }
            if !exported {
                for item in &surface.reexports {
                    meter.work(1, &target.root, None).map_err(|error| {
                        CodeResolveError::Exports(Box::new(crate::ExportError::Budget(error)))
                    })?;
                    if item.item == Some(*site) {
                        exported = true;
                        break;
                    }
                }
            }
            if !exported {
                return Err(CodeResolveError::NotExported {
                    root: self.root(),
                    path: relative.into(),
                });
            }
        }
        Ok(resolved)
    }

    fn candidate_roots(&self, meter: &Meter) -> Result<Vec<ItemId>, CodeResolveError> {
        let path = &self.code.index.targets[self.target().index()].root;
        let mut admission = ItemAdmission::new(meter, path);
        let result = (|| {
            self.charge_candidate_scan(meter, path)?;
            let mut memory = Memory::new(&mut admission);
            let mut roots = Vec::new();
            let inherited = (self.route_kind() == ExternRouteKind::Prelude).then(|| {
                self.code.index.extern_root_candidates(
                    self.from,
                    self.name,
                    Heads::Code { test: self.test },
                )
            });
            for root in self.group.roots().chain(inherited.into_iter().flatten()) {
                meter.work(1, path, None)?;
                memory
                    .push(&mut roots, self.code.index.root_items[&root])
                    .map_err(|error| LoadError::new(path, None, LoadErrorKind::Storage(error)))?;
            }
            try_sort_unstable_by(&mut roots, |left, right| {
                let left = self.code.index.sort_key(*left);
                let right = self.code.index.sort_key(*right);
                meter.work(
                    left.0
                        .len()
                        .saturating_add(right.0.len())
                        .saturating_add(left.2.as_str().len())
                        .saturating_add(right.2.as_str().len())
                        .saturating_add(1),
                    path,
                    None,
                )?;
                Ok::<_, LoadError>(left.cmp(&right))
            })?;
            try_dedup_by(&mut roots, |left, right| {
                meter.work(1, path, None)?;
                Ok::<_, LoadError>(left == right)
            })?;
            Ok(roots)
        })();
        result.map_err(|error| CodeResolveError::Query(Box::new(admission.recover(error))))
    }

    fn charge_candidate_scan(&self, meter: &Meter, path: &RelPath) -> Result<(), LoadError> {
        meter.work(self.group.scan_len().saturating_add(1), path, None)?;
        if self.route_kind() == ExternRouteKind::Prelude {
            meter.work(self.name.len().saturating_add(1), path, None)?;
            if let Some(entry) = self.code.index.extern_name(self.from, self.name, self.test) {
                meter.work(entry.roots.len(), path, None)?;
            }
            let root = self.code.index.modules[self.from].root;
            if let Some(entry) = self.code.index.scopes[root].get(self.name) {
                meter.work(
                    entry.defs[crate::settle::slot(Namespace::Type)].len(),
                    path,
                    None,
                )?;
            }
        }
        Ok(())
    }
}

impl CodeResolver<'_> {
    /// Borrow every actual external library candidate from the owner's admitted tables.
    ///
    /// Uses existing target/dev/test filtering and settled Cargo names; custom library
    /// names, renames and explicit `extern crate` aliases are retained. Unindexed
    /// external names yield no library receipt. Every original-name candidate is
    /// preserved; choosing a receipt does not remove its group's ambiguity.
    ///
    /// # Errors
    /// Refuses absent/outside owners, missing context and macro definition bodies.
    pub fn extern_libraries_at(
        &self,
        owner: ItemId,
        byte: usize,
    ) -> Result<impl Iterator<Item = ExternLibrary<'_>> + '_, CodeResolveError> {
        let module = self.module(owner, byte)?;
        let reading = module;
        let item = &self.index.items[owner.index()];
        if item.kind == SymbolKind::MacroRules
            && item
                .body
                .as_ref()
                .is_some_and(|body| (body.start_byte..body.end_byte).contains(&byte))
        {
            return Err(CodeResolveError::MacroDefinition(owner));
        }
        let test = item.test != TestRole::None;
        let target = self.index.modules[module].target;
        let own_package = self.index.targets[target.index()].package_id;
        let root = self.index.modules[module].root;
        let preludes = self.index.preludes[target.index()]
            .iter()
            .filter(move |(_, entry)| entry.everywhere || test)
            .flat_map(move |(name, entry)| {
                entry.roots(test).map(move |module| ExternLibrary {
                    code: self,
                    name,
                    module,
                    group: Group::Prelude(entry, test),
                    from: reading,
                    test,
                })
            });
        let explicit = self
            .index
            .crate_of
            .iter()
            .filter(move |(id, _)| {
                let declaration = &self.index.items[id.index()];
                let home = self.homes[id.index()].module();
                declaration.target == target
                    && (test || declaration.test == TestRole::None)
                    && (home == Some(module) || home == Some(root))
            })
            .flat_map(move |(id, roots)| {
                roots.iter().copied().map(move |module| ExternLibrary {
                    code: self,
                    name: self.index.items[id.index()].name.as_str(),
                    module,
                    group: Group::Explicit(roots),
                    from: reading,
                    test,
                })
            });
        Ok(preludes.chain(explicit).filter(move |library| {
            library.package() != own_package
                && self.index.targets[library.target().index()].kind == TargetKind::Lib
        }))
    }
}
