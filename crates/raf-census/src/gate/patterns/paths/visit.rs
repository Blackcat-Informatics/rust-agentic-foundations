// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Bounded source syntax traversal with explicit lexical scopes.

use super::{
    Compiled, CrossPackagePaths, Found, Role, Scopes, Site, VISIBILITY, cross_package_uses, matches,
};
use crate::detectors::{ReceiverMode, receiver_mode};
use crate::gate::bounds::{BoundRole, Scan, syntax};
use crate::report::bounds::UNCLASSIFIED;
use crate::{
    Error,
    detectors::{ENUM_CATCH_ALL, RawHit},
    syntax::{ImportLeaf, visit_import_paths},
};
use purrdf_jsonschema::ecma::MatchLimits;
use raf_source::{CodeResolver, ImplBlock, ItemId, Namespace, SourceFile, SymbolKind, Workspace};
use std::collections::BTreeSet;
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(in crate::gate) enum Container {
    File,
    Module(usize),
    Item(std::ops::Range<usize>),
    Declaration(usize),
}

/// A policy consumer of the same enclosing lexical frames as path matching.
pub(in crate::gate) trait FunctionObserver {
    fn container(
        &mut self,
        file: &SourceFile,
        container: Container,
        attributes: &[syn::Attribute],
        limits: &mut MatchLimits,
    ) -> Result<(), Error>;
    fn interested(&self, file: &SourceFile, signature: &syn::Signature) -> bool;
    fn observe(
        &mut self,
        file: &SourceFile,
        scopes: &mut Scopes,
        signature: &syn::Signature,
        attributes: &[syn::Attribute],
        body: &syn::Block,
        limits: &mut MatchLimits,
    ) -> Result<(), Error>;
}

pub(in crate::gate) fn functions(
    workspace: &Workspace,
    code: &CodeResolver<'_>,
    file: &SourceFile,
    syntax: &syn::File,
    limits: &mut MatchLimits,
    observer: &mut dyn FunctionObserver,
) -> Result<(), Error> {
    let mut visitor = Paths {
        compiled: &[],
        workspace,
        code,
        file,
        limits,
        scopes: Scopes::default(),
        found: Vec::new(),
        error: None,
        seen: BTreeSet::new(),
        observer: Some(observer),
        numeric: None,
        numeric_receiver: None,
        path_uses: None,
        enum_hits: None,
        impl_blocks: &[],
        owners: super::OwnerCache::default(),
    };
    visitor.visit_file(syntax);
    visitor.error.map_or(Ok(()), Err)
}

// Lookup uses identifier paths; generic syntax is still visited on the original
// borrowed AST. This preserves spans without cloning or reparsing an AST.
fn bare_path(path: &syn::Path) -> syn::Path {
    syn::Path {
        leading_colon: path
            .leading_colon
            .is_some()
            .then(syn::token::PathSep::default),
        segments: path
            .segments
            .iter()
            .map(|segment| syn::PathSegment::from(segment.ident.clone()))
            .collect(),
    }
}

pub(super) fn collect<'p, 'w>(
    compiled: &[Compiled<'p>],
    workspace: &'w Workspace,
    code: &CodeResolver<'_>,
    file: &'w SourceFile,
    syntax: &syn::File,
    limits: &mut MatchLimits,
) -> Result<Vec<Found<'p, 'w>>, Error> {
    let mut visitor = Paths {
        compiled,
        workspace,
        code,
        file,
        limits,
        scopes: Scopes::default(),
        found: Vec::new(),
        error: None,
        seen: BTreeSet::new(),
        observer: None,
        numeric: None,
        numeric_receiver: None,
        path_uses: None,
        enum_hits: None,
        impl_blocks: &[],
        owners: super::OwnerCache::default(),
    };
    visitor.visit_file(syntax);
    if let Some(error) = visitor.error {
        return Err(error);
    }
    Ok(visitor.found)
}

pub(in crate::gate) fn references(
    workspace: &Workspace,
    code: &CodeResolver<'_>,
    file: &SourceFile,
    syntax: &syn::File,
    limits: &mut MatchLimits,
    paths: &mut CrossPackagePaths,
) -> Result<(), Error> {
    let mut visitor = Paths {
        compiled: &[],
        workspace,
        code,
        file,
        limits,
        scopes: Scopes::default(),
        found: Vec::new(),
        error: None,
        seen: BTreeSet::new(),
        observer: None,
        numeric: None,
        numeric_receiver: None,
        path_uses: Some(paths),
        enum_hits: None,
        impl_blocks: &[],
        owners: super::OwnerCache::one_occurrence(),
    };
    visitor.visit_file(syntax);
    visitor.error.map_or(Ok(()), Err)
}

pub(in crate::gate) fn enum_arms(
    workspace: &Workspace,
    code: &CodeResolver<'_>,
    file: &SourceFile,
    syntax: &syn::File,
    limits: &mut MatchLimits,
    retained: &mut usize,
) -> Result<Vec<RawHit>, Error> {
    super::charge(
        ENUM_CATCH_ALL,
        limits,
        file.text.len().try_into().unwrap_or(u64::MAX),
    )?;
    let mut hits = Vec::new();
    let mut visitor = Paths {
        compiled: &[],
        workspace,
        code,
        file,
        limits,
        scopes: Scopes::for_enum_patterns(),
        found: Vec::new(),
        error: None,
        seen: BTreeSet::new(),
        observer: None,
        numeric: None,
        numeric_receiver: None,
        path_uses: None,
        enum_hits: Some(EnumInventory {
            found: &mut hits,
            retained: *retained,
            file_impls: None,
            self_path: None,
        }),
        impl_blocks: &[],
        owners: super::OwnerCache::one_occurrence(),
    };
    visitor.visit_file(syntax);
    if let Some(error) = visitor.error {
        return Err(error);
    }
    *retained += hits.len();
    Ok(hits)
}

struct EnumInventory<'a, 'w> {
    found: &'a mut Vec<RawHit>,
    retained: usize,
    file_impls: Option<&'w [ImplBlock]>,
    self_path: Option<String>,
}

impl<'w> EnumInventory<'_, 'w> {
    fn impls_at(
        &mut self,
        workspace: &'w Workspace,
        file: &SourceFile,
        start: usize,
        limits: &mut MatchLimits,
    ) -> Result<&'w [ImplBlock], Error> {
        let blocks = if let Some(blocks) = self.file_impls {
            blocks
        } else {
            let upper = workspace.impls().size_hint().1.unwrap_or(usize::MAX);
            let probes = 2 * (u64::from(upper.bit_width()) + 1);
            let bytes = u64::try_from(file.path.as_str().len()).unwrap_or(u64::MAX);
            super::charge(
                ENUM_CATCH_ALL,
                limits,
                probes.saturating_mul(bytes.saturating_add(1)),
            )?;
            let blocks = workspace.impls_in(&file.path);
            self.file_impls = Some(blocks);
            blocks
        };
        super::charge(
            ENUM_CATCH_ALL,
            limits,
            2 * (u64::from(blocks.len().bit_width()) + 1),
        )?;
        Ok(impls_at(blocks, start))
    }
}

fn impls_at(blocks: &[ImplBlock], start: usize) -> &[ImplBlock] {
    let first = blocks.partition_point(|block| block.span.start_byte < start);
    let end = blocks.partition_point(|block| block.span.start_byte <= start);
    &blocks[first..end]
}

fn enum_lookup<'a, 'w>(
    workspace: &'a Workspace,
    code: &'a CodeResolver<'w>,
    site: &'a Site<'a>,
    owner: &raf_source::Item,
    enum_impl: Option<(&'a str, &'a [ImplBlock])>,
) -> Result<super::Lookup<'a, 'w>, Error> {
    let target = workspace
        .target(owner.target)
        .ok_or_else(|| super::failure(ENUM_CATCH_ALL, "enum pattern has no consuming target"))?;
    Ok(super::Lookup {
        code,
        owner: super::PathOwner::Item(owner.id),
        site,
        label: ENUM_CATCH_ALL,
        visibility: Some((workspace, target.package_id)),
        enum_impl,
    })
}

struct Paths<'a, 'p, 'w, 'c> {
    compiled: &'a [Compiled<'p>],
    workspace: &'w Workspace,
    code: &'c CodeResolver<'c>,
    file: &'w SourceFile,
    limits: &'a mut MatchLimits,
    scopes: Scopes,
    found: Vec<Found<'p, 'w>>,
    error: Option<Error>,
    seen: BTreeSet<(&'p str, usize)>,
    observer: Option<&'a mut dyn FunctionObserver>,
    numeric: Option<&'a mut Scan>,
    numeric_receiver: Option<ReceiverMode>,
    path_uses: Option<&'a mut CrossPackagePaths>,
    enum_hits: Option<EnumInventory<'a, 'w>>,
    impl_blocks: &'w [ImplBlock],
    owners: super::OwnerCache<'w>,
}

impl Paths<'_, '_, '_, '_> {
    fn excludes_from_inventory(&self, attributes: &[syn::Attribute]) -> bool {
        self.enum_hits.is_some() && crate::syntax::definitely_excluded(attributes)
    }

    fn collect_match_arms(&mut self, node: &syn::ExprMatch) -> Result<(), Error> {
        let Some(inventory) = self.enum_hits.as_mut() else {
            return Ok(());
        };
        super::charge(
            ENUM_CATCH_ALL,
            self.limits,
            node.arms.len().try_into().unwrap_or(u64::MAX),
        )?;
        if !super::enum_sites::has_candidate(node) {
            return Ok(());
        }
        let site = Site::at(
            self.file,
            node.match_token.span(),
            Role::Pattern,
            Namespace::Value,
            false,
        );
        let owners = self
            .owners
            .get(self.workspace, self.file, site.byte, self.limits)?;
        let mut qualified: Option<BTreeSet<usize>> = None;
        for owner in owners {
            let enum_impl = inventory
                .self_path
                .as_deref()
                .map(|path| (path, self.impl_blocks));
            let lookup = enum_lookup(self.workspace, self.code, &site, owner, enum_impl)?;
            let found = super::enum_sites::catch_alls(node, &lookup, &self.scopes, self.limits)?;
            if let Some(survivors) = &mut qualified {
                survivors.retain(|arm| found.contains(arm));
            } else {
                qualified = Some(found);
            }
        }
        if let Some(qualified) = qualified {
            let total = inventory
                .retained
                .saturating_add(inventory.found.len())
                .saturating_add(qualified.len());
            if total > self.limits.states {
                return Err(super::failure(
                    ENUM_CATCH_ALL,
                    "catch-all arm state limit exhausted",
                ));
            }
            inventory.found.extend(
                qualified
                    .into_iter()
                    .map(|index| super::enum_sites::hit(&node.arms[index])),
            );
        }
        Ok(())
    }

    fn container(&mut self, container: Container, attributes: &[syn::Attribute]) -> bool {
        if let Some(observer) = self.observer.as_deref_mut()
            && let Err(error) = observer.container(self.file, container, attributes, self.limits)
        {
            self.error = Some(error);
            return true;
        }
        false
    }
    fn ignores_function(&self, signature: &syn::Signature) -> bool {
        self.observer
            .as_deref()
            .is_some_and(|observer| !observer.interested(self.file, signature))
    }
    fn observe_function(
        &mut self,
        signature: &syn::Signature,
        attributes: &[syn::Attribute],
        body: &syn::Block,
    ) -> bool {
        if self.observer.is_none() {
            return false;
        }
        self.scope(
            false,
            body.stmts.iter().filter_map(|statement| {
                if let syn::Stmt::Item(item) = statement {
                    Some(item)
                } else {
                    None
                }
            }),
            |this| {
                if let Err(error) = this
                    .observer
                    .as_deref_mut()
                    .expect("function observer")
                    .observe(
                        this.file,
                        &mut this.scopes,
                        signature,
                        attributes,
                        body,
                        this.limits,
                    )
                {
                    this.error = Some(error);
                }
            },
        );
        true
    }
    fn scope<'n>(
        &mut self,
        module: bool,
        items: impl Iterator<Item = &'n syn::Item>,
        walk: impl FnOnce(&mut Self),
    ) {
        if self.error.is_some() {
            return;
        }
        if let Err(error) = self.scopes.enter(
            module,
            self.path_uses.is_some() || self.numeric.is_some() || self.enum_hits.is_some(),
            items,
            self.limits,
        ) {
            self.error = Some(error);
            return;
        }
        walk(self);
        self.scopes.leave();
    }

    fn generic_scope(
        &mut self,
        generics: &syn::Generics,
        self_type: bool,
        walk: impl FnOnce(&mut Self),
    ) {
        self.scope(false, std::iter::empty(), |this| {
            this.scopes.generics(generics, self_type);
            walk(this);
        });
    }

    fn inputs(&mut self, inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>) {
        for input in inputs {
            match input {
                syn::FnArg::Receiver(_) => self.scopes.receiver(),
                syn::FnArg::Typed(typed) => self.bind(&typed.pat),
            }
        }
    }

    fn bind(&mut self, pattern: &syn::Pat) {
        if self.error.is_some() {
            return;
        }
        let id = self.compiled.first().map_or(
            if self.path_uses.is_some() {
                VISIBILITY
            } else {
                "preferred-provider"
            },
            |compiled| compiled.pattern.id.as_str(),
        );
        let result = self.scopes.bind(pattern, |scopes, name| {
            if self.enum_hits.is_some() {
                let site = Site::at(
                    self.file,
                    name.span(),
                    Role::Pattern,
                    Namespace::Value,
                    false,
                );
                let owners = self
                    .owners
                    .get(self.workspace, self.file, site.byte, self.limits)?;
                for owner in owners {
                    let lookup = enum_lookup(self.workspace, self.code, &site, owner, None)?;
                    if !super::enum_sites::bare_binding(name, &lookup, scopes, self.limits)? {
                        return Ok(false);
                    }
                }
                return Ok(!owners.is_empty());
            }
            if self.numeric.is_some() {
                let site = Site::at(
                    self.file,
                    name.span(),
                    Role::Pattern,
                    Namespace::Value,
                    false,
                );
                let owners = self
                    .owners
                    .get(self.workspace, self.file, site.byte, self.limits)?;
                let mut classified = None;
                for owner in owners {
                    let lookup = numeric_lookup(self.workspace, self.code, &site, owner)?;
                    let path = syn::Path::from(name.clone());
                    let actual = super::numeric_definitions(&lookup, scopes, &path, self.limits)?;
                    super::classify_pattern(
                        self.workspace,
                        UNCLASSIFIED,
                        &actual,
                        false,
                        &mut classified,
                    )?;
                }
                return Ok(!classified.unwrap_or_default());
            }
            if let Some(paths) = self.path_uses.as_deref_mut() {
                let site = Site::at(
                    self.file,
                    name.span(),
                    Role::Pattern,
                    Namespace::Value,
                    scopes.in_file_module(),
                );
                return cross_package_uses(
                    self.workspace,
                    self.code,
                    (
                        self.owners
                            .get(self.workspace, self.file, site.byte, self.limits)?,
                        self.impl_blocks,
                    ),
                    scopes,
                    &syn::Path::from(name.clone()),
                    &site,
                    (paths, self.limits),
                )
                .map(|constant| !constant);
            }
            super::constant(
                self.workspace,
                self.code,
                self.file,
                id,
                name,
                &mut self.owners,
                self.limits,
            )
            .map(|constant| !constant)
        });
        if let Err(error) = result {
            self.error = Some(error);
        }
    }

    fn path(&mut self, path: &syn::Path, role: Role, namespace: Namespace) {
        if self.error.is_some() {
            return;
        }
        let Some(first) = path.segments.first() else {
            return;
        };
        if self.path_uses.is_some() && first.ident == "Self" {
            return;
        }
        let site = Site::at(
            self.file,
            first.ident.span(),
            role,
            namespace,
            self.scopes.in_file_module(),
        );
        if self.path_uses.is_none() && !self.compiled.iter().any(|expected| expected.selects(role))
        {
            return;
        }
        let owners = self
            .owners
            .get(self.workspace, self.file, site.byte, self.limits);
        let Ok(owners) = owners else {
            self.error = owners.err();
            return;
        };
        if let Some(paths) = self.path_uses.as_deref_mut() {
            self.error = cross_package_uses(
                self.workspace,
                self.code,
                (owners, self.impl_blocks),
                &self.scopes,
                path,
                &site,
                (paths, self.limits),
            )
            .err();
            return;
        }
        for expected in self.compiled {
            if !expected.selects(role)
                || self
                    .seen
                    .contains(&(expected.pattern.id.as_str(), site.byte))
            {
                continue;
            }
            let matched = matches(
                owners,
                self.code,
                &self.scopes,
                expected,
                path,
                &site,
                self.limits,
            );
            match matched {
                Ok(true) => {
                    self.seen.insert((expected.pattern.id.as_str(), site.byte));
                    let matched_owners = if expected.pattern.scope.is_unscoped() {
                        None
                    } else {
                        match matching_owners(
                            owners,
                            self.code,
                            &self.scopes,
                            expected,
                            path,
                            &site,
                            self.limits,
                        ) {
                            Ok(ids) => Some(ids),
                            Err(error) => {
                                self.error = Some(error);
                                return;
                            }
                        }
                    };
                    if let Err(error) = super::super::retain(
                        &mut self.found,
                        expected.pattern,
                        &self.file.path,
                        site.byte,
                        site.line,
                        matched_owners,
                        self.limits,
                    ) {
                        self.error = Some(error);
                        return;
                    }
                }
                Ok(false) => {}
                Err(error) => {
                    self.error = Some(error);
                    return;
                }
            }
        }
    }

    fn qualified(
        &mut self,
        path: &syn::Path,
        qself: Option<&syn::QSelf>,
        role: Role,
        namespace: Namespace,
    ) {
        let Some(qself) = qself else {
            self.path(path, role, namespace);
            return;
        };
        if qself.position > 0 {
            self.path(path, role, namespace);
        } else if let syn::Type::Path(ty) = qself.ty.as_ref()
            && ty.qself.is_none()
        {
            let mut joined = bare_path(&ty.path);
            joined.segments.extend(
                path.segments
                    .iter()
                    .map(|segment| syn::PathSegment::from(segment.ident.clone())),
            );
            self.path(&joined, role, namespace);
        } else if self.compiled.iter().any(|pattern| {
            pattern.selects(role)
                && pattern.spelling.last().map(String::as_str)
                    == path
                        .segments
                        .last()
                        .map(|segment| segment.ident.to_string())
                        .as_deref()
        }) {
            self.error = Some(super::super::failure(
                &self.compiled[0].pattern.id,
                "qualified self lookup requires a named type or trait path; this source form is not supported",
            ));
        }
    }

    fn type_prefixes(&mut self, path: &syn::Path) {
        if self.path_uses.is_none()
            && !self
                .compiled
                .iter()
                .any(|pattern| pattern.role == Role::Type || pattern.role == Role::Path)
        {
            return;
        }
        self.prefixes(path, Role::Type);
    }

    fn prefixes(&mut self, path: &syn::Path, role: Role) {
        let mut prefix = syn::Path {
            leading_colon: path
                .leading_colon
                .is_some()
                .then(syn::token::PathSep::default),
            segments: syn::punctuated::Punctuated::new(),
        };
        for segment in path
            .segments
            .iter()
            .take(path.segments.len().saturating_sub(1))
        {
            prefix
                .segments
                .push(syn::PathSegment::from(segment.ident.clone()));
            self.path(&prefix, role, Namespace::Type);
        }
    }

    fn imports(&mut self, attributes: &[syn::Attribute], absolute: bool, tree: &syn::UseTree) {
        for attribute in attributes {
            self.visit_attribute(attribute);
        }
        if self.path_uses.is_some() && self.scopes.in_module() {
            return;
        }
        visit_import_paths(tree, |parts, leaf| {
            let path = syn::Path {
                leading_colon: absolute.then(syn::token::PathSep::default),
                segments: parts.iter().cloned().map(syn::PathSegment::from).collect(),
            };
            let role = if self.path_uses.is_some() && matches!(leaf, ImportLeaf::Glob) {
                Role::Glob
            } else {
                Role::Import
            };
            let namespaces = if role == Role::Glob || matches!(leaf, ImportLeaf::SelfType(_)) {
                &[Namespace::Type][..]
            } else {
                &Namespace::ALL[..]
            };
            for &namespace in namespaces {
                self.path(&path, role, namespace);
            }
            if self.path_uses.is_some() {
                self.prefixes(&path, Role::Import);
            }
        });
    }

    fn constructor(&mut self, path: &syn::Path, qself: Option<&syn::QSelf>, namespace: Namespace) {
        if self.path_uses.is_some() {
            self.qualified(path, qself, Role::Path, namespace);
            self.type_prefixes(path);
        }
    }

    fn condition(&mut self, expression: &syn::Expr) {
        match expression {
            syn::Expr::Let(node) => {
                self.visit_expr(&node.expr);
                self.visit_pat(&node.pat);
                self.bind(&node.pat);
            }
            syn::Expr::Binary(node) if matches!(node.op, syn::BinOp::And(_)) => {
                self.condition(&node.left);
                self.condition(&node.right);
            }
            _ => self.visit_expr(expression),
        }
    }

    fn arm_pattern(&mut self, pattern: &syn::Pat) {
        if let syn::Pat::Guard(guard) = pattern {
            self.arm_pattern(&guard.pat);
            if self.path_uses.is_some() || self.numeric.is_some() {
                self.condition(&guard.guard);
            } else {
                self.visit_expr(&guard.guard);
            }
        } else {
            self.visit_pat(pattern);
            self.bind(pattern);
        }
    }
}

impl<'ast> Visit<'ast> for Paths<'_, '_, '_, '_> {
    fn visit_file(&mut self, node: &'ast syn::File) {
        if self.container(Container::File, &node.attrs) {
            return;
        }
        self.scope(true, node.items.iter(), |this| {
            syn::visit::visit_file(this, node);
        });
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if self.container(
            Container::Module(node.ident.span().byte_range().start),
            &node.attrs,
        ) {
            return;
        }
        // The declaration's attributes belong to the parent reading scope.
        for attribute in &node.attrs {
            self.visit_attribute(attribute);
        }
        if let Some((_, items)) = &node.content {
            self.scope(true, items.iter(), |this| {
                for item in items {
                    this.visit_item(item);
                }
            });
        }
    }

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        self.imports(&node.attrs, node.leading_colon.is_some(), &node.tree);
    }

    fn visit_item(&mut self, node: &'ast syn::Item) {
        if self.excludes_from_inventory(crate::syntax::item_attributes(node)) {
            return;
        }
        if self.path_uses.is_some()
            && let syn::Item::Verbatim(tokens) = node
        {
            raf_source::visit_grouped_import(tokens, |attributes, absolute, tree| {
                self.imports(attributes, absolute, tree);
            });
        }
        let previous = self.impl_blocks;
        // Nested items cannot capture the enclosing implementation's Self.
        let previous_self = self
            .enum_hits
            .as_mut()
            .and_then(|inventory| inventory.self_path.take());
        syn::visit::visit_item(self, node);
        if let Some(inventory) = &mut self.enum_hits {
            inventory.self_path = previous_self;
        }
        self.impl_blocks = previous;
    }

    fn visit_block(&mut self, node: &'ast syn::Block) {
        self.scope(
            false,
            node.stmts.iter().filter_map(|statement| {
                if let syn::Stmt::Item(item) = statement {
                    Some(item)
                } else {
                    None
                }
            }),
            |this| syn::visit::visit_block(this, node),
        );
    }

    fn visit_stmt_macro(&mut self, node: &'ast syn::StmtMacro) {
        if !self.excludes_from_inventory(&node.attrs) {
            syn::visit::visit_stmt_macro(self, node);
        }
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if self.ignores_function(&node.sig) {
            return;
        }
        let previous = self.numeric_receiver.take();
        self.generic_scope(&node.sig.generics, false, |this| {
            this.inputs(&node.sig.inputs);
            if !this.observe_function(&node.sig, &node.attrs, &node.block) {
                syn::visit::visit_item_fn(this, node);
            }
        });
        self.numeric_receiver = previous;
    }

    fn visit_impl_item_const(&mut self, node: &'ast syn::ImplItemConst) {
        if !self.excludes_from_inventory(&node.attrs) {
            syn::visit::visit_impl_item_const(self, node);
        }
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        if self.ignores_function(&node.sig) || self.excludes_from_inventory(&node.attrs) {
            return;
        }
        let previous = self.numeric_receiver;
        if self.numeric.is_some() {
            self.numeric_receiver = receiver_mode(&node.sig);
        }
        self.generic_scope(&node.sig.generics, false, |this| {
            this.inputs(&node.sig.inputs);
            if !this.observe_function(&node.sig, &node.attrs, &node.block) {
                syn::visit::visit_impl_item_fn(this, node);
            }
        });
        self.numeric_receiver = previous;
    }

    fn visit_trait_item_const(&mut self, node: &'ast syn::TraitItemConst) {
        if !self.excludes_from_inventory(&node.attrs) {
            syn::visit::visit_trait_item_const(self, node);
        }
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        if self.ignores_function(&node.sig) || self.excludes_from_inventory(&node.attrs) {
            return;
        }
        self.generic_scope(&node.sig.generics, false, |this| {
            this.inputs(&node.sig.inputs);
            if !node
                .default
                .as_ref()
                .is_some_and(|body| this.observe_function(&node.sig, &node.attrs, body))
            {
                syn::visit::visit_trait_item_fn(this, node);
            }
        });
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        if self.container(Container::Item(node.span().byte_range()), &node.attrs) {
            return;
        }
        self.generic_scope(&node.generics, true, |this| {
            let previous = this.impl_blocks;
            let start = node.span().byte_range().start;
            if let Some(inventory) = &mut this.enum_hits {
                if let syn::Type::Path(path) = node.self_ty.as_ref()
                    && path.qself.is_none()
                    && node.trait_.is_none()
                {
                    inventory.self_path = Some(super::path_text(
                        &super::segments(&path.path),
                        path.path.leading_colon.is_some(),
                    ));
                    match inventory.impls_at(this.workspace, this.file, start, this.limits) {
                        Ok(blocks) => this.impl_blocks = blocks,
                        Err(error) => {
                            this.error = Some(error);
                            return;
                        }
                    }
                }
            } else if this.path_uses.is_some() {
                this.impl_blocks = impls_at(this.workspace.impls_in(&this.file.path), start);
            }
            if this.path_uses.is_some()
                && let Some((path, _)) = &node.trait_
            {
                this.path(path, Role::Type, Namespace::Type);
                this.type_prefixes(path);
            }
            syn::visit::visit_item_impl(this, node);
            this.impl_blocks = previous;
        });
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        if self.container(Container::Item(node.span().byte_range()), &node.attrs) {
            return;
        }
        self.generic_scope(&node.generics, true, |this| {
            syn::visit::visit_item_trait(this, node);
        });
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        if self.container(
            Container::Declaration(node.ident.span().byte_range().start),
            &node.attrs,
        ) {
            return;
        }
        self.generic_scope(&node.generics, false, |this| {
            syn::visit::visit_item_struct(this, node);
        });
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        if self.container(
            Container::Declaration(node.ident.span().byte_range().start),
            &node.attrs,
        ) {
            return;
        }
        self.generic_scope(&node.generics, false, |this| {
            syn::visit::visit_item_enum(this, node);
        });
    }

    fn visit_variant(&mut self, node: &'ast syn::Variant) {
        if !self.excludes_from_inventory(&node.attrs) {
            syn::visit::visit_variant(self, node);
        }
    }

    fn visit_field(&mut self, node: &'ast syn::Field) {
        if !self.excludes_from_inventory(&node.attrs) {
            syn::visit::visit_field(self, node);
        }
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        if self.container(
            Container::Declaration(node.ident.span().byte_range().start),
            &node.attrs,
        ) {
            return;
        }
        self.generic_scope(&node.generics, false, |this| {
            syn::visit::visit_item_type(this, node);
        });
    }

    fn visit_item_union(&mut self, node: &'ast syn::ItemUnion) {
        if self.container(
            Container::Declaration(node.ident.span().byte_range().start),
            &node.attrs,
        ) {
            return;
        }
        self.generic_scope(&node.generics, false, |this| {
            syn::visit::visit_item_union(this, node);
        });
    }

    fn visit_local(&mut self, node: &'ast syn::Local) {
        if self.excludes_from_inventory(&node.attrs) {
            return;
        }
        self.visit_pat(&node.pat);
        if let Some(initializer) = &node.init {
            self.visit_expr(&initializer.expr);
            if let Some((_, otherwise)) = &initializer.diverge {
                self.visit_expr(otherwise);
            }
        }
        self.bind(&node.pat);
    }

    fn visit_expr_closure(&mut self, node: &'ast syn::ExprClosure) {
        self.scope(false, std::iter::empty(), |this| {
            for input in &node.inputs {
                this.bind(input);
            }
            syn::visit::visit_expr_closure(this, node);
        });
    }

    fn visit_arm(&mut self, node: &'ast syn::Arm) {
        if self.enum_hits.is_some() && !crate::syntax::inert_attributes(&node.attrs) {
            return;
        }
        self.scope(false, std::iter::empty(), |this| {
            if this.path_uses.is_some() {
                for attribute in &node.attrs {
                    this.visit_attribute(attribute);
                }
            }
            this.arm_pattern(&node.pat);
            this.visit_expr(&node.body);
        });
    }

    fn visit_expr(&mut self, node: &'ast syn::Expr) {
        if self.enum_hits.is_some()
            && crate::syntax::expression_attributes(node)
                .is_some_and(crate::syntax::definitely_excluded)
        {
            return;
        }
        syn::visit::visit_expr(self, node);
    }

    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        if self.enum_hits.is_some() && !crate::syntax::inert_attributes(&node.attrs) {
            return;
        }
        if self.error.is_none() {
            self.error = self.collect_match_arms(node).err();
        }
        syn::visit::visit_expr_match(self, node);
    }

    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        self.scope(false, std::iter::empty(), |this| {
            this.condition(&node.cond);
            this.visit_block(&node.then_branch);
        });
        if let Some((_, branch)) = &node.else_branch {
            self.visit_expr(branch);
        }
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.scope(false, std::iter::empty(), |this| {
            this.condition(&node.cond);
            this.visit_block(&node.body);
        });
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.visit_expr(&node.expr);
        self.scope(false, std::iter::empty(), |this| {
            this.visit_pat(&node.pat);
            this.bind(&node.pat);
            this.visit_block(&node.body);
        });
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if self.numeric.is_some() {
            self.numeric_uses(BoundRole::Limiter, node.span(), syntax::limiting_call(node));
            self.numeric_call(node);
        }
        let mut function = node.func.as_ref();
        loop {
            match function {
                syn::Expr::Paren(node) => function = &node.expr,
                syn::Expr::Group(node) => function = &node.expr,
                _ => break,
            }
        }
        if let syn::Expr::Path(path) = function {
            self.qualified(
                &path.path,
                path.qself.as_ref(),
                Role::Call,
                Namespace::Value,
            );
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if self.numeric.is_some() {
            self.numeric_uses(
                BoundRole::Limiter,
                node.span(),
                syntax::limiting_method(node),
            );
            self.numeric_method(node);
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        if self.numeric.is_some() {
            self.numeric_uses(
                BoundRole::Comparison,
                node.span(),
                syntax::comparisons(node),
            );
        }
        syn::visit::visit_expr_binary(self, node);
    }

    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        if self.error.is_some() {
            return;
        }
        if self
            .compiled
            .iter()
            .any(|pattern| pattern.role == Role::Path)
        {
            let label = self
                .compiled
                .first()
                .map_or(VISIBILITY, |compiled| compiled.pattern.id.as_str());
            match super::constant(
                self.workspace,
                self.code,
                self.file,
                label,
                &node.ident,
                &mut self.owners,
                self.limits,
            ) {
                Ok(true) => self.path(
                    &syn::Path::from(node.ident.clone()),
                    Role::Path,
                    Namespace::Value,
                ),
                Ok(false) => {}
                Err(error) => {
                    self.error = Some(error);
                    return;
                }
            }
        }
        syn::visit::visit_pat_ident(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        self.qualified(
            &node.path,
            node.qself.as_ref(),
            Role::Path,
            Namespace::Value,
        );
        self.type_prefixes(&node.path);
        syn::visit::visit_expr_path(self, node);
    }

    fn visit_expr_struct(&mut self, node: &'ast syn::ExprStruct) {
        self.constructor(&node.path, node.qself.as_ref(), Namespace::Type);
        syn::visit::visit_expr_struct(self, node);
    }

    fn visit_pat_struct(&mut self, node: &'ast syn::PatStruct) {
        self.constructor(&node.path, node.qself.as_ref(), Namespace::Type);
        syn::visit::visit_pat_struct(self, node);
    }

    fn visit_pat_tuple_struct(&mut self, node: &'ast syn::PatTupleStruct) {
        self.constructor(&node.path, node.qself.as_ref(), Namespace::Value);
        syn::visit::visit_pat_tuple_struct(self, node);
    }

    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        self.qualified(&node.path, node.qself.as_ref(), Role::Type, Namespace::Type);
        self.type_prefixes(&node.path);
        syn::visit::visit_type_path(self, node);
    }

    fn visit_trait_bound(&mut self, node: &'ast syn::TraitBound) {
        self.path(&node.path, Role::Type, Namespace::Type);
        self.type_prefixes(&node.path);
        syn::visit::visit_trait_bound(self, node);
    }

    fn visit_attribute(&mut self, node: &'ast syn::Attribute) {
        self.path(node.path(), Role::Attribute, Namespace::Macro);
        syn::visit::visit_attribute(self, node);
    }

    fn visit_item_macro(&mut self, node: &'ast syn::ItemMacro) {
        if node.ident.is_some() {
            for attribute in &node.attrs {
                self.visit_attribute(attribute);
            }
            if let Some(name) = &node.ident {
                self.scopes.macro_name(name);
            }
        } else {
            syn::visit::visit_item_macro(self, node);
        }
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if self.path_uses.is_some() {
            self.type_prefixes(&node.path);
        }
        self.path(&node.path, Role::Macro, Namespace::Macro);
        // This observes written expression-list arguments, never expansion or
        // the meaning of a macro's non-expression language.
        if let Ok(expressions) = node.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for expression in &expressions {
                self.visit_expr(expression);
            }
        } else if let Ok(statements) = node.parse_body_with(syn::Block::parse_within) {
            // Statement/repetition-shaped arguments retain their original spans.
            // Other macro languages remain token syntax, never expansion proof.
            self.scope(
                false,
                statements.iter().filter_map(|statement| {
                    if let syn::Stmt::Item(item) = statement {
                        Some(item)
                    } else {
                        None
                    }
                }),
                |this| {
                    for statement in &statements {
                        this.visit_stmt(statement);
                    }
                },
            );
        }
        syn::visit::visit_macro(self, node);
    }
}

/// The owners whose own resolution matches the occurrence; a scope judges the
/// hit by their targets, because a file shared by several targets can resolve
/// one written path to a different definition in each.
fn matching_owners(
    owners: &[&raf_source::Item],
    code: &CodeResolver<'_>,
    scopes: &Scopes,
    expected: &Compiled<'_>,
    path: &syn::Path,
    site: &Site<'_>,
    limits: &mut MatchLimits,
) -> Result<Vec<ItemId>, Error> {
    let mut ids = Vec::new();
    for owner in owners {
        if matches(
            std::slice::from_ref(owner),
            code,
            scopes,
            expected,
            path,
            site,
            limits,
        )? {
            ids.push(owner.id);
        }
    }
    Ok(ids)
}

pub(in crate::gate) fn numeric_calls(
    workspace: &Workspace,
    code: &CodeResolver<'_>,
    file: &SourceFile,
    syntax: &syn::File,
    limits: &mut MatchLimits,
    scan: &mut Scan,
) -> Result<(), Error> {
    let mut visitor = Paths {
        compiled: &[],
        workspace,
        code,
        file,
        limits,
        scopes: Scopes::default(),
        found: Vec::new(),
        error: None,
        seen: BTreeSet::new(),
        observer: None,
        numeric: Some(scan),
        numeric_receiver: None,
        path_uses: None,
        enum_hits: None,
        impl_blocks: &[],
        owners: super::OwnerCache::one_occurrence(),
    };
    visitor.visit_file(syntax);
    visitor.error.map_or(Ok(()), Err)
}

impl Paths<'_, '_, '_, '_> {
    fn numeric_uses<'a>(
        &mut self,
        role: BoundRole,
        context: proc_macro2::Span,
        operands: impl Iterator<Item = &'a syn::Expr>,
    ) {
        if self.error.is_some() {
            return;
        }
        let Some(scan) = self.numeric.take() else {
            return;
        };
        for operand in operands {
            let result = (|| {
                if let Some(token) = syntax::integer(operand) {
                    return scan.literal(&self.file.path, token, context, role, self.limits);
                }
                let syn::Expr::Path(path) = syntax::ungroup(operand) else {
                    return Ok(());
                };
                let site = Site::at(self.file, path.span(), Role::Path, Namespace::Value, false);
                let owners = self
                    .owners
                    .get(self.workspace, self.file, site.byte, self.limits)?;
                let actual = numeric_targets(
                    (self.workspace, self.code),
                    &site,
                    owners,
                    self.limits,
                    NumericKind::Bound,
                    |lookup, limits| numeric_path(lookup, &self.scopes, path, limits),
                )?;
                scan.named_use(role, actual, self.limits)
            })();
            if let Err(error) = result {
                self.error = Some(error);
                break;
            }
        }
        self.numeric = Some(scan);
    }

    fn numeric_call(&mut self, node: &syn::ExprCall) {
        if self.error.is_some() {
            return;
        }
        let Some(scan) = self.numeric.take() else {
            return;
        };
        if let syn::Expr::Path(path) = syntax::ungroup(&node.func) {
            self.error = scan
                .call(&self.file.path, node, self.limits, |limits| {
                    let site =
                        Site::at(self.file, path.span(), Role::Call, Namespace::Value, false);
                    let owners = self
                        .owners
                        .get(self.workspace, self.file, site.byte, limits)?;
                    numeric_targets(
                        (self.workspace, self.code),
                        &site,
                        owners,
                        limits,
                        NumericKind::Call,
                        |lookup, limits| numeric_path(lookup, &self.scopes, path, limits),
                    )
                })
                .err();
        }
        self.numeric = Some(scan);
    }

    fn numeric_method(&mut self, node: &syn::ExprMethodCall) {
        if self.error.is_some()
            || !matches!(syntax::ungroup(&node.receiver), syn::Expr::Path(path) if path.qself.is_none() && path.path.is_ident("self"))
        {
            return;
        }
        let (Some(receiver), Some(scan)) = (self.numeric_receiver, self.numeric.as_deref_mut())
        else {
            return;
        };
        self.error = scan
            .method(&self.file.path, node, receiver, self.limits, |limits| {
                let site = Site::at(
                    self.file,
                    node.method.span(),
                    Role::Call,
                    Namespace::Value,
                    false,
                );
                let owners = self
                    .owners
                    .get(self.workspace, self.file, site.byte, limits)?;
                numeric_targets(
                    (self.workspace, self.code),
                    &site,
                    owners,
                    limits,
                    NumericKind::Dot,
                    |lookup, _| numeric_self(lookup, &node.method.unraw().to_string()),
                )
            })
            .err();
    }
}

#[derive(Clone, Copy)]
enum NumericKind {
    Call,
    Dot,
    Bound,
}

fn numeric_lookup<'a, 'w>(
    workspace: &'a Workspace,
    code: &'a CodeResolver<'w>,
    site: &'a Site<'a>,
    owner: &raf_source::Item,
) -> Result<super::Lookup<'a, 'w>, Error> {
    let package = workspace
        .target(owner.target)
        .ok_or_else(|| super::failure(UNCLASSIFIED, "path site has no consuming target"))?
        .package_id;
    Ok(super::Lookup {
        code,
        owner: super::PathOwner::Item(owner.id),
        site,
        label: UNCLASSIFIED,
        visibility: Some((workspace, package)),
        enum_impl: None,
    })
}

fn numeric_self(lookup: &super::Lookup<'_, '_>, member: &str) -> Result<Vec<ItemId>, Error> {
    let (super::PathOwner::Item(owner), Some((workspace, _))) = (lookup.owner, lookup.visibility)
    else {
        return Ok(Vec::new());
    };
    let owner = workspace
        .item(owner)
        .ok_or_else(|| super::failure(UNCLASSIFIED, "self path has no indexed owner"))?;
    let Some(name) = super::super::super::rules::self_member(workspace, owner, member) else {
        return Ok(Vec::new());
    };
    lookup.indexed(&name, false).map(Option::unwrap_or_default)
}

fn numeric_path(
    lookup: &super::Lookup<'_, '_>,
    scopes: &Scopes,
    path: &syn::ExprPath,
    limits: &mut MatchLimits,
) -> Result<Vec<ItemId>, Error> {
    let joined;
    let name = match &path.qself {
        Some(qself) if qself.position == 0 => {
            let syn::Type::Path(nominal) = qself.ty.as_ref() else {
                return Ok(Vec::new());
            };
            if nominal.qself.is_some() {
                return Ok(Vec::new());
            }
            let mut name = bare_path(&nominal.path);
            name.segments.extend(bare_path(&path.path).segments);
            joined = name;
            &joined
        }
        _ => &path.path,
    };
    if name.segments.len() == 2
        && name
            .segments
            .first()
            .is_some_and(|part| part.ident == "Self")
        && let Some(member) = name.segments.last()
    {
        return numeric_self(lookup, &member.ident.unraw().to_string());
    }
    super::numeric_definitions(lookup, scopes, name, limits)
}

fn numeric_targets(
    (workspace, code): (&Workspace, &CodeResolver<'_>),
    site: &Site<'_>,
    owners: &[&raf_source::Item],
    limits: &mut MatchLimits,
    kind: NumericKind,
    mut resolve: impl FnMut(&super::Lookup<'_, '_>, &mut MatchLimits) -> Result<Vec<ItemId>, Error>,
) -> Result<Vec<Option<ItemId>>, Error> {
    let mut definitions = BTreeSet::new();
    let mut retain = |definition, limits: &mut MatchLimits| -> Result<(), Error> {
        super::charge(UNCLASSIFIED, limits, 1)?;
        if definitions.len() >= limits.states && !definitions.contains(&definition) {
            return Err(super::failure(
                UNCLASSIFIED,
                "numeric definition state limit exhausted",
            ));
        }
        definitions.insert(definition);
        Ok(())
    };
    for owner in owners {
        let lookup = numeric_lookup(workspace, code, site, owner)?;
        let actual = resolve(&lookup, limits)?;
        if actual.is_empty() {
            retain(None, limits)?;
        }
        for id in actual {
            let item = workspace.item(id).ok_or_else(|| {
                super::failure(UNCLASSIFIED, "resolved path has no indexed source item")
            })?;
            let admitted = match (kind, item.kind) {
                (
                    NumericKind::Bound,
                    SymbolKind::Const | SymbolKind::Static | SymbolKind::AssocConst,
                )
                | (
                    NumericKind::Call,
                    SymbolKind::Fn
                    | SymbolKind::ForeignFn
                    | SymbolKind::TraitDefaultMethod
                    | SymbolKind::TraitRequiredMethod,
                ) => true,
                (NumericKind::Call | NumericKind::Dot, SymbolKind::InherentMethod) => {
                    match code.require_unconditional_inherent_method(id) {
                        Ok(()) => true,
                        Err(raf_source::CodeResolveError::MethodApplicability(_)) => false,
                        Err(error) => return Err(lookup.error(item.path.as_str(), error)),
                    }
                }
                _ => false,
            }
            .then_some(id);
            retain(admitted, limits)?;
        }
    }
    Ok(definitions.into_iter().collect())
}
