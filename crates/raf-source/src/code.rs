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

use crate::index::{Index, UseKind};
use crate::meter::{ItemAdmission, Meter};
use crate::reach::{Home, fill_homes};
use crate::resolve::Heads;
use crate::{
    ImplId, ItemId, LoadError, LoadOptions, Namespace, Reach, RelPath, Resolution, ResolveError,
    SymbolKind, TargetId, TestRole, Usage, Workspace,
};

/// Failure to establish a reading context or resolve its module-level code path.
#[derive(Debug)]
#[non_exhaustive]
pub enum CodeResolveError {
    /// The load-local owner is absent from this workspace.
    UnknownItem(ItemId),
    /// The load-local impl is absent from this workspace.
    UnknownImpl(ImplId),
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
            Self::UnknownImpl(id) => write!(f, "source impl {id:?} is absent from this workspace"),
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

/// One import in the module index, with its exact target and reading context.
pub struct IndexedImport<'query, 'workspace> {
    code: &'query CodeResolver<'workspace>,
    position: usize,
}

impl IndexedImport<'_, '_> {
    /// The target and location where this flattened import was written.
    #[must_use]
    pub fn site(&self) -> (TargetId, &RelPath, usize) {
        let declaration = &self.code.index.uses[self.position];
        (
            self.code.index.modules[declaration.module].target,
            &declaration.file,
            declaration.line,
        )
    }

    /// The imported path segments, whether they start with `::`, whether
    /// the declaration imports a glob from that path, and the namespaces
    /// in which the full path is read.
    #[must_use]
    pub fn path(&self) -> (&[String], bool, bool, &'static [Namespace]) {
        let declaration = &self.code.index.uses[self.position];
        let (segments, glob) = match &declaration.kind {
            UseKind::Named { segs, .. } => (segs, false),
            UseKind::Glob { segs } => (segs, true),
        };
        (
            segments,
            declaration.leading_colon,
            glob,
            declaration.kind.terminal_namespaces(),
        )
    }

    /// Resolve the whole import path or a prefix of it in its own use context.
    ///
    /// # Errors
    /// Preserves shared resolver errors, including invalid prefixes.
    pub fn resolve(
        &self,
        path: &str,
        namespace: Namespace,
    ) -> Result<Resolution, CodeResolveError> {
        let declaration = &self.code.index.uses[self.position];
        self.code
            .resolve_code_at(
                declaration.module,
                path,
                namespace,
                declaration.heads(self.position),
            )
            .map(|(resolution, _)| resolution)
    }
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

    /// Every flattened module import in its exact admitted reading context.
    pub fn indexed_imports(&self) -> impl Iterator<Item = IndexedImport<'_, 'a>> {
        (0..self.index.uses.len()).map(|position| IndexedImport {
            code: self,
            position,
        })
    }

    /// Resolve a name at the file's own module, as reached by this workspace.
    /// Use this for file-level syntax that has no named item of its own.
    ///
    /// # Errors
    /// Refuses a reach whose target or module is absent from this index.
    /// Preserves every shared resolver error.
    pub fn resolve_from_file_reach(
        &self,
        reach: &Reach,
        path: &str,
        namespace: Namespace,
    ) -> Result<Resolution, CodeResolveError> {
        let (module, heads) = self.file_context(reach, false)?;
        self.resolve_code_at(module, path, namespace, heads)
            .map(|(resolution, _)| resolution)
    }

    /// Resolve an import spelling in a file's own module and target.
    ///
    /// # Errors
    /// Refuses an absent reach and preserves the shared resolver errors.
    pub fn resolve_import_from_file_reach(
        &self,
        reach: &Reach,
        path: &str,
        namespace: Namespace,
    ) -> Result<Resolution, CodeResolveError> {
        let (module, heads) = self.file_context(reach, true)?;
        self.resolve_code_at(module, path, namespace, heads)
            .map(|(resolution, _)| resolution)
    }

    fn file_context(
        &self,
        reach: &Reach,
        import: bool,
    ) -> Result<(usize, Heads), CodeResolveError> {
        let module = self
            .index
            .target_roots
            .get(reach.target.index())
            .and_then(|&root| {
                self.index
                    .module_by_path
                    .get(&(root, reach.module_path.clone()))
            })
            .copied()
            .ok_or_else(|| {
                CodeResolveError::Resolve(ResolveError::UnknownModule(
                    reach.module_path.to_string(),
                ))
            })?;
        Ok((module, reading_heads(reach.test, import)))
    }

    /// Read a module-level path in the lexical module of an indexed impl.
    /// Use this for the impl header, which has no named item of its own.
    ///
    /// # Errors
    /// Refuses an unknown impl and preserves every shared resolver error.
    pub fn resolve_from_impl(
        &self,
        implementation: ImplId,
        path: &str,
        namespace: Namespace,
        import: bool,
    ) -> Result<Resolution, CodeResolveError> {
        let block = self
            .workspace
            .impl_block(implementation)
            .ok_or(CodeResolveError::UnknownImpl(implementation))?;
        let test = block.test != TestRole::None;
        let heads = if import {
            Heads::Use { test, own: None }
        } else {
            Heads::Code { test }
        };
        self.resolve_code_at(block.module_index, path, namespace, heads)
            .map(|(resolution, _)| resolution)
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

    /// Whether an owner's reading module and its ancestors have no indexed
    /// configuration-dependent declarations or file reaches.
    ///
    /// This classifies the module route. The caller still checks the syntax
    /// of the occurrence and its enclosing items.
    ///
    /// # Errors
    /// Refuses an absent owner, outside byte, missing context or exceeded query grant.
    pub fn has_unconditional_reading_module(
        &self,
        owner: ItemId,
        byte: usize,
    ) -> Result<bool, CodeResolveError> {
        let module = self.module(owner, byte)?;
        self.owner_query(owner, |index, steps| {
            index.module_is_unconditional(module, steps)
        })
    }

    /// Check a file reach's module and ancestor declarations for configuration dependence.
    ///
    /// # Errors
    /// Refuses an absent reach or an exceeded query grant.
    pub fn has_unconditional_file_reach(&self, reach: &Reach) -> Result<bool, CodeResolveError> {
        self.file_query(reach, false, |index, module, _, steps| {
            index.module_is_unconditional(module, steps)
        })
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

    /// Follow visible imports to candidate members of indexed enums.
    ///
    /// Each candidate carries an enum identity and the terminal spelling in
    /// the import. The caller validates the declaration and constructor form
    /// and requires a unique surviving definition. Classify lexical shadowing before
    /// asking this module-level query. Set `import` when the spelling was
    /// written in an import declaration, to preserve its edition rules.
    ///
    /// An empty candidate list means the inspected routes supply no variants.
    /// An existing definition or an inconclusive or conditional route returns
    /// `None`, so the caller cannot infer the name is available to another route.
    ///
    /// # Errors
    /// Refuses missing owner context, invalid paths and native
    /// resource exhaustion.
    pub fn enum_member_imports_from_item_at(
        &self,
        owner: ItemId,
        byte: usize,
        path: &str,
        namespace: Namespace,
        import: bool,
    ) -> Result<Option<Vec<(ItemId, String)>>, CodeResolveError> {
        let (module, heads) = self.owned_context(owner, byte, import)?;
        self.owner_query(owner, |index, steps| {
            index.enum_member_import(module, path, namespace, heads, steps)
        })
    }

    /// Follow visible enum member imports from a file's own module and target.
    ///
    /// As [`Self::enum_member_imports_from_item_at`], validate candidates against the
    /// enum declaration and set `import` for a spelling read in an import.
    ///
    /// # Errors
    /// Refuses an absent reach, invalid path or exhausted query resources.
    pub fn enum_member_imports_from_file_reach(
        &self,
        reach: &Reach,
        path: &str,
        namespace: Namespace,
        import: bool,
    ) -> Result<Option<Vec<(ItemId, String)>>, CodeResolveError> {
        self.file_query(reach, import, |index, module, heads, steps| {
            index.enum_member_import(module, path, namespace, heads, steps)
        })
    }

    /// Find a single named type whose module and import route is unconditional.
    /// Lexical bindings inside blocks and type alias substitution belong to the caller.
    ///
    /// # Errors
    /// Refuses an invalid reading context or path and native resource exhaustion.
    pub fn unconditional_type_from_item_at(
        &self,
        owner: ItemId,
        byte: usize,
        path: &str,
        import: bool,
    ) -> Result<Option<ItemId>, CodeResolveError> {
        let (module, heads) = self.owned_context(owner, byte, import)?;
        self.owner_query(owner, |index, steps| {
            index.unconditional_type_path(module, path, heads, steps)
        })
    }

    /// Find a single named type by an unconditional route from a file's own module.
    ///
    /// # Errors
    /// Refuses an absent reach, invalid path or exhausted query resources.
    pub fn unconditional_type_from_file_reach(
        &self,
        reach: &Reach,
        path: &str,
        import: bool,
    ) -> Result<Option<ItemId>, CodeResolveError> {
        self.file_query(reach, import, |index, module, heads, steps| {
            index.unconditional_type_path(module, path, heads, steps)
        })
    }

    fn file_query<T>(
        &self,
        reach: &Reach,
        import: bool,
        query: impl FnOnce(&'a Index, usize, Heads, crate::meter::Steps<'_>) -> Result<T, ResolveError>,
    ) -> Result<T, CodeResolveError> {
        let (module, heads) = self.file_context(reach, import)?;
        let (file, line) = self.index.module_site(module);
        self.query_at((&file, line), |index, steps| {
            query(index, module, heads, steps)
        })
    }

    fn owner_query<T>(
        &self,
        owner: ItemId,
        query: impl FnOnce(&'a Index, crate::meter::Steps<'_>) -> Result<T, ResolveError>,
    ) -> Result<T, CodeResolveError> {
        let item = &self.index.items[owner.index()];
        self.query_at((&item.file, Some(item.name_line)), query)
    }

    fn query_at<T>(
        &self,
        (file, line): (&RelPath, Option<usize>),
        query: impl FnOnce(&'a Index, crate::meter::Steps<'_>) -> Result<T, ResolveError>,
    ) -> Result<T, CodeResolveError> {
        let meter = Meter::resume_query(
            self.options,
            Usage {
                item_bytes: self.usage.item_bytes,
                ..Usage::default()
            },
        );
        let steps = crate::meter::Steps::QueryItems {
            meter: &meter,
            file,
            line,
        };
        let answer = query(self.index, steps);
        Index::query_refusal(steps)
            .map_err(|refused| CodeResolveError::Resolve(self.index.refused(refused)))?;
        answer.map_err(CodeResolveError::Resolve)
    }

    fn resolve_owned(
        &self,
        owner: ItemId,
        byte: usize,
        path: &str,
        namespace: Namespace,
        import: bool,
    ) -> Result<Resolution, CodeResolveError> {
        let (module, heads) = self.owned_context(owner, byte, import)?;
        self.resolve_code_at(module, path, namespace, heads)
            .map(|(resolution, _)| resolution)
    }

    fn owned_context(
        &self,
        owner: ItemId,
        byte: usize,
        import: bool,
    ) -> Result<(usize, Heads), CodeResolveError> {
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
        Ok((module, reading_heads(test, import)))
    }
}

fn reading_heads(test: bool, import: bool) -> Heads {
    if import {
        Heads::Use { test, own: None }
    } else {
        Heads::Code { test }
    }
}
