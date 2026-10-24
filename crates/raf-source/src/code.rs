// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Code-head lookup from an actual indexed source owner.

mod aliases;
mod library;
mod syntax;
pub use library::{ExternLibrary, ExternRouteKind};

use std::error::Error;
use std::fmt;

use purrdf_lex::allocation::Memory;

use crate::index::Index;
use crate::meter::{ItemAdmission, Meter};
use crate::reach::{Home, fill_homes};
use crate::resolve::Heads;
use crate::{
    ItemId, LoadError, LoadOptions, Namespace, RelPath, Resolution, ResolveError, SymbolKind,
    TargetId, TestRole, Usage, Workspace,
};

/// Failure to establish a reading context or resolve its module-level code path.
#[derive(Debug)]
#[non_exhaustive]
pub enum CodeResolveError {
    /// The load-local owner is absent from this workspace.
    UnknownItem(ItemId),
    /// The byte is outside the selected item's source span.
    OutsideItem(ItemId, usize),
    /// An indexed owner has no source-owned reading module.
    MissingContext(ItemId),
    /// A definition-scope query names an item that is not a module.
    ExpectedModule(ItemId),
    /// Macro definition tokens require invocation context before name resolution.
    MacroDefinition(ItemId),
    /// The shared resolver refused the path; its original cause is retained.
    Resolve(ResolveError),
    /// The library's public surface refused its original resource or resolution query.
    Exports(Box<crate::ExportError>),
    /// Configured syntax selection preserves its original parser/load refusal.
    ConfiguredSyntax(Box<LoadError>),
    /// The bounded code query preserves its original resource refusal.
    Query(Box<LoadError>),
    /// Generic applicability of this inherent method is not established.
    MethodApplicability(ItemId),
    /// Alias-member applicability is outside the explicitly supported syntax.
    AliasMember {
        /// The actual alias definition needing additional evidence.
        alias: ItemId,
        /// The requested associated member.
        member: String,
        /// The missing applicability evidence.
        reason: &'static str,
    },
    /// A configured syntax query lacks an actual admitted target/file context.
    SyntaxContext {
        /// The requested actual target.
        target: TargetId,
        /// The requested source file.
        file: RelPath,
        /// The missing context that the caller must supply.
        reason: &'static str,
    },
    /// A definition exists but is not exported by the selected library.
    NotExported {
        /// The selected actual library root.
        root: ItemId,
        /// The requested relative API path.
        path: String,
    },
}

impl fmt::Display for CodeResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownItem(id) => write!(f, "source owner {id:?} is absent from this workspace"),
            Self::OutsideItem(id, byte) => write!(f, "byte {byte} is outside source owner {id:?}"),
            Self::MissingContext(id) => write!(f, "source owner {id:?} has no reading module"),
            Self::ExpectedModule(id) => write!(f, "source scope {id:?} is not a module"),
            Self::MacroDefinition(id) => write!(
                f,
                "source owner {id:?} is a macro definition; code lookup requires an actual invocation context"
            ),
            Self::Resolve(error) => error.fmt(f),
            Self::Exports(error) => error.fmt(f),
            Self::ConfiguredSyntax(error) | Self::Query(error) => error.fmt(f),
            Self::MethodApplicability(id) => write!(
                f,
                "inherent method {id:?} needs generic applicability evidence beyond plain nominal types and unconstrained forwarded const parameters"
            ),
            Self::AliasMember {
                alias,
                member,
                reason,
            } => write!(f, "alias {alias:?} member {member:?}: {reason}"),
            Self::SyntaxContext {
                target,
                file,
                reason,
            } => write!(
                f,
                "configured syntax for {file} in target {target:?}: {reason}"
            ),
            Self::NotExported { root, path } => write!(
                f,
                "library root {root:?} does not export the definition of {path:?}"
            ),
        }
    }
}

impl Error for CodeResolveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Resolve(error) => Some(error),
            Self::Exports(error) => Some(error.as_ref()),
            Self::ConfiguredSyntax(error) | Self::Query(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

/// Module-level code lookup with source-owned item/module/target association.
///
/// A lookup uses code-head edition and dev/test rules, rather than `use` rules.
/// The caller must first classify lexical locals, parameters, generic bindings
/// and block imports. This query does not infer those scopes, expand macros,
/// resolve `Self` or authenticate an unindexed dependency from its spelling.
/// Item identities belong to the workspace that constructed this query.
pub struct CodeResolver<'a> {
    workspace: &'a Workspace,
    options: &'a LoadOptions,
    index: &'a Index,
    homes: Vec<Home>,
    usage: Usage,
}

impl<'a> CodeResolver<'a> {
    pub(crate) fn new(
        workspace: &'a Workspace,
        index: &'a Index,
        options: &'a LoadOptions,
        usage: Usage,
    ) -> Result<Self, LoadError> {
        let meter = Meter::new(options);
        let root = RelPath::root();
        meter.items(usage.item_bytes, &root, None)?;
        meter.items(
            size_of::<&Workspace>() + size_of::<&LoadOptions>(),
            &root,
            None,
        )?;
        let mut homes = Vec::new();
        {
            let mut admission = ItemAdmission::new(&meter, &root);
            Memory::new(&mut admission)
                .reserve(&mut homes, index.items.len())
                .map_err(|error| admission.error(error))?;
        }
        homes.resize(index.items.len(), Home::default());
        fill_homes(index, &mut homes, true);
        let mut usage = usage;
        usage.item_bytes = meter.usage().item_bytes;
        Ok(Self {
            workspace,
            options,
            index,
            homes,
            usage,
        })
    }

    /// The original load plus this query's retained reading-table allowance.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }

    fn module(&self, owner: ItemId, byte: usize) -> Result<usize, CodeResolveError> {
        let item = self
            .index
            .items
            .get(owner.index())
            .ok_or(CodeResolveError::UnknownItem(owner))?;
        if !(item.span.start_byte..item.span.end_byte).contains(&byte) {
            return Err(CodeResolveError::OutsideItem(owner, byte));
        }
        if item.kind == SymbolKind::Mod
            && item
                .body
                .as_ref()
                .is_some_and(|body| (body.start_byte..body.end_byte).contains(&byte))
        {
            return self
                .index
                .module_of
                .get(&owner)
                .copied()
                .ok_or(CodeResolveError::MissingContext(owner));
        }
        self.homes[owner.index()]
            .module()
            .ok_or(CodeResolveError::MissingContext(owner))
    }

    /// The actual reading module of a byte within an indexed owner.
    /// Inline module bodies read their own module; their declaration headers
    /// read the containing module. Method and trait ownership comes from Source.
    ///
    /// # Errors
    /// Refuses an absent owner, outside byte or missing reading context.
    pub fn reading_module(&self, owner: ItemId, byte: usize) -> Result<&str, CodeResolveError> {
        Ok(self.index.modules[self.module(owner, byte)?].path.as_str())
    }

    /// Whether an indexed item is defined in this actual module or a descendant.
    /// A relocated export and an impl member keep their lexical defining module.
    /// Distinct targets never share scope merely because their paths agree.
    ///
    /// # Errors
    /// Refuses absent identities, a non-module scope or missing reading context.
    pub fn is_defined_in(&self, owner: ItemId, scope: ItemId) -> Result<bool, CodeResolveError> {
        let item = self
            .index
            .items
            .get(owner.index())
            .ok_or(CodeResolveError::UnknownItem(owner))?;
        let module = self
            .index
            .items
            .get(scope.index())
            .ok_or(CodeResolveError::UnknownItem(scope))?;
        if module.kind != SymbolKind::Mod {
            return Err(CodeResolveError::ExpectedModule(scope));
        }
        if item.target != module.target {
            return Ok(false);
        }
        if owner == scope {
            return Ok(true);
        }
        let wanted = self
            .index
            .module_of
            .get(&scope)
            .copied()
            .or_else(|| {
                self.index
                    .root_items
                    .get(&self.index.target_roots[module.target.index()])
                    .filter(|id| **id == scope)
                    .and_then(|_| self.homes[scope.index()].module())
            })
            .ok_or(CodeResolveError::MissingContext(scope))?;
        let mut actual = self.homes[owner.index()]
            .module()
            .ok_or(CodeResolveError::MissingContext(owner))?;
        loop {
            if actual == wanted {
                return Ok(true);
            }
            let Some(parent) = self.index.modules[actual].parent else {
                return Ok(false);
            };
            actual = parent;
        }
    }

    /// Resolve a module-level code path in one namespace from its actual owner.
    /// Lexical shadow classification is a separate prerequisite; successful
    /// module lookup alone is not proof that an occurrence names that item.
    ///
    /// # Errors
    /// Refuses missing reading context and preserves every shared resolver error.
    pub fn resolve_from_item_at(
        &self,
        owner: ItemId,
        byte: usize,
        path: &str,
        namespace: Namespace,
    ) -> Result<Resolution, CodeResolveError> {
        self.resolve_owned(owner, byte, path, namespace, false)
    }

    /// Resolve an import spelling from an actual source owner's reading module.
    /// This preserves edition-specific `use` roots for block import observations;
    /// it does not register a lexical import or infer a missing external namespace.
    ///
    /// # Errors
    /// The owner, macro-body and shared resolver refusals of code lookup.
    pub fn resolve_import_from_item_at(
        &self,
        owner: ItemId,
        byte: usize,
        path: &str,
        namespace: Namespace,
    ) -> Result<Resolution, CodeResolveError> {
        self.resolve_owned(owner, byte, path, namespace, true)
    }

    fn resolve_owned(
        &self,
        owner: ItemId,
        byte: usize,
        path: &str,
        namespace: Namespace,
        import: bool,
    ) -> Result<Resolution, CodeResolveError> {
        let module = self.module(owner, byte)?;
        let item = &self.index.items[owner.index()];
        if item.kind == SymbolKind::MacroRules
            && item
                .body
                .as_ref()
                .is_some_and(|body| (body.start_byte..body.end_byte).contains(&byte))
        {
            return Err(CodeResolveError::MacroDefinition(owner));
        }
        let test = self.index.items[owner.index()].test != TestRole::None;
        let heads = if import {
            Heads::Use { test, own: None }
        } else {
            Heads::Code { test }
        };
        self.resolve_code_at(module, path, namespace, heads)
            .map(|(resolution, _)| resolution)
    }
}
