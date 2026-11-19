// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Bounded source syntax traversal with explicit lexical scopes.

use super::{
    Compiled, CrossPackagePaths, Found, Role, Scopes, Site, VISIBILITY, cross_package_uses, matches,
};
use crate::{Error, syntax::visit_import_paths};
use purrdf_jsonschema::ecma::MatchLimits;
use raf_source::{CodeResolver, ImplBlock, Namespace, SourceFile, Workspace};
use std::collections::BTreeSet;
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
        path_uses: None,
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
        path_uses: None,
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
        path_uses: Some(paths),
        impl_blocks: &[],
        owners: super::OwnerCache::one_occurrence(),
    };
    visitor.visit_file(syntax);
    visitor.error.map_or(Ok(()), Err)
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
    path_uses: Option<&'a mut CrossPackagePaths>,
    impl_blocks: &'w [ImplBlock],
    owners: super::OwnerCache<'w>,
}

impl Paths<'_, '_, '_, '_> {
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
        if let Err(error) = self
            .scopes
            .enter(module, self.path_uses.is_some(), items, self.limits)
        {
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
        visit_import_paths(tree, |parts, alias| {
            let path = syn::Path {
                leading_colon: absolute.then(syn::token::PathSep::default),
                segments: parts.iter().cloned().map(syn::PathSegment::from).collect(),
            };
            let role = if self.path_uses.is_some() && alias.is_none() {
                Role::Glob
            } else {
                Role::Import
            };
            let namespaces = if role == Role::Glob {
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
            if self.path_uses.is_some() {
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
        if self.path_uses.is_some()
            && let syn::Item::Verbatim(tokens) = node
        {
            raf_source::visit_grouped_import(tokens, |attributes, absolute, tree| {
                self.imports(attributes, absolute, tree);
            });
        }
        syn::visit::visit_item(self, node);
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

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if self.ignores_function(&node.sig) {
            return;
        }
        self.generic_scope(&node.sig.generics, false, |this| {
            this.inputs(&node.sig.inputs);
            if !this.observe_function(&node.sig, &node.attrs, &node.block) {
                syn::visit::visit_item_fn(this, node);
            }
        });
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        if self.ignores_function(&node.sig) {
            return;
        }
        self.generic_scope(&node.sig.generics, false, |this| {
            this.inputs(&node.sig.inputs);
            if !this.observe_function(&node.sig, &node.attrs, &node.block) {
                syn::visit::visit_impl_item_fn(this, node);
            }
        });
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        if self.ignores_function(&node.sig) {
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
            if this.path_uses.is_some() {
                let blocks = this.workspace.impls_in(&this.file.path);
                let start = node.span().byte_range().start;
                let first = blocks.partition_point(|block| block.span.start_byte < start);
                let end = blocks.partition_point(|block| block.span.start_byte <= start);
                this.impl_blocks = &blocks[first..end];
                if let Some((path, _)) = &node.trait_ {
                    this.path(path, Role::Type, Namespace::Type);
                    this.type_prefixes(path);
                }
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
) -> Result<Vec<raf_source::ItemId>, Error> {
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
