// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The symbol index: every named item of every module tree, the module graph
//! it lives in, and the scope tables the resolver reads.
//!
//! Building runs in four steps. The emit step turns the facts of each file,
//! once per module path the file is reached under, into items, module
//! contents, `use` declarations and impl blocks. It reads the files in the
//! order the module-tree walk reached them, so every module is created by the
//! `mod` declaration that names it, under its parent, before its own file is
//! read; the order of the file names plays no part. The scope step builds the
//! name tables of every module and settles them to a fixed point (see
//! [`crate::resolve`]). The impl step attaches the members of impl blocks to
//! the types they resolve to. The order step sorts the items.

use std::collections::{BTreeMap, BTreeSet};

use crate::error::{LoadError, LoadErrorKind};
use crate::facts::{
    ImplFact, ItemFact, ItemKind, ModBody, PathFact, SpanFact, TestRole, UseTree, Vis,
};
use crate::item::{Item, ItemId, Namespace, SymbolKind, TraitKey};
use crate::manifest::Dependencies;
use crate::modtree::{TargetId, Visit};
use crate::path::RelPath;
use crate::settle::Scope;
use crate::target::{Target, TargetKind};
use crate::workspace::SourceFile;

/// A `use` declaration after its tree is flattened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UseKind {
    /// `use path as name` (or `use path`, which binds the last segment).
    Named { name: String, segs: Vec<String> },
    /// `use path::*`.
    Glob { segs: Vec<String> },
}

/// One flattened `use` entry and where it was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UseDecl {
    pub(crate) module: usize,
    pub(crate) file: RelPath,
    pub(crate) line: usize,
    pub(crate) leading_colon: bool,
    pub(crate) vis: Vis,
    pub(crate) kind: UseKind,
}

/// What the index knows of a package's manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PackageInfo {
    /// The edition year.
    pub(crate) edition: u16,
    pub(crate) dependencies: Dependencies,
}

/// An impl block waiting for its self type to resolve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImplDecl {
    pub(crate) module: usize,
    pub(crate) file: RelPath,
    pub(crate) target: TargetId,
    pub(crate) reach_test: bool,
    pub(crate) fact: ImplFact,
}

/// A module: all the declarations that define the module at one path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Module {
    pub(crate) path: String,
    pub(crate) parent: Option<usize>,
    pub(crate) root: usize,
    pub(crate) defs: Vec<ItemId>,
}

/// The index of a workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Index {
    pub(crate) items: Vec<Item>,
    /// The items in the order of [`crate::Workspace::items`].
    pub(crate) order: Vec<ItemId>,
    pub(crate) modules: Vec<Module>,
    pub(crate) module_by_path: BTreeMap<String, usize>,
    /// The module that a `mod` or `extern crate` item stands for.
    pub(crate) module_of: BTreeMap<ItemId, usize>,
    /// Names of the library crates, for paths that start with a crate name.
    pub(crate) libs: BTreeMap<String, usize>,
    /// The root module of each target crate path.
    pub(crate) crate_roots: BTreeMap<String, usize>,
    pub(crate) targets: Vec<Target>,
    pub(crate) packages: BTreeMap<String, PackageInfo>,
    pub(crate) scopes: Vec<Scope>,
    pub(crate) uses: Vec<UseDecl>,
    pub(crate) use_resolved: Vec<bool>,
    /// True once an import is resolved or has definitively failed.
    pub(crate) use_settled: Vec<bool>,
    /// Named imports not yet settled, by module and bound name.
    pub(crate) unsettled: BTreeMap<usize, BTreeMap<String, usize>>,
    /// The crate an `extern crate` item names, by item.
    pub(crate) extern_names: BTreeMap<ItemId, String>,
    /// Items grouped by file and the byte offset where their span starts.
    /// The item of each target's crate root, by the root module.
    pub(crate) root_items: BTreeMap<usize, ItemId>,
    pub(crate) by_source: BTreeMap<(RelPath, usize), Vec<ItemId>>,
    /// Members of a type or trait, by name.
    pub(crate) assoc: BTreeMap<ItemId, BTreeMap<String, Vec<ItemId>>>,
    /// Members of trait impls on types that are not local, by written path.
    pub(crate) foreign: BTreeMap<String, Vec<ItemId>>,
    pub(crate) by_file: BTreeMap<RelPath, Vec<ItemId>>,
    /// The aliased path of each type alias, with the module it is written in.
    pub(crate) aliases: BTreeMap<ItemId, (usize, PathFact)>,
    impls: Vec<ImplDecl>,
}

fn symbol_kind(kind: ItemKind) -> SymbolKind {
    match kind {
        ItemKind::Fn => SymbolKind::Fn,
        ItemKind::Struct => SymbolKind::Struct,
        ItemKind::Enum => SymbolKind::Enum,
        ItemKind::Union => SymbolKind::Union,
        ItemKind::Trait => SymbolKind::Trait,
        ItemKind::TypeAlias => SymbolKind::TypeAlias,
        ItemKind::Const | ItemKind::UnnamedConst => SymbolKind::Const,
        ItemKind::Static => SymbolKind::Static,
        ItemKind::MacroRules => SymbolKind::MacroRules,
        ItemKind::ExternCrate => SymbolKind::ExternCrate,
        ItemKind::ForeignFn => SymbolKind::ForeignFn,
        ItemKind::ForeignStatic => SymbolKind::ForeignStatic,
        ItemKind::ForeignType => SymbolKind::ForeignType,
    }
}

fn main_namespace(kind: SymbolKind) -> Namespace {
    match kind {
        SymbolKind::Fn
        | SymbolKind::Const
        | SymbolKind::Static
        | SymbolKind::ForeignFn
        | SymbolKind::ForeignStatic
        | SymbolKind::InherentMethod
        | SymbolKind::TraitImplMethod
        | SymbolKind::TraitDefaultMethod
        | SymbolKind::TraitRequiredMethod
        | SymbolKind::AssocConst => Namespace::Value,
        SymbolKind::MacroRules => Namespace::Macro,
        SymbolKind::Struct
        | SymbolKind::Enum
        | SymbolKind::Union
        | SymbolKind::Trait
        | SymbolKind::TypeAlias
        | SymbolKind::Mod
        | SymbolKind::ExternCrate
        | SymbolKind::ForeignType
        | SymbolKind::AssocType => Namespace::Type,
    }
}

/// The role an item has once its module tree's test flag is counted in.
fn combine(own: TestRole, reached_in_tests: bool) -> TestRole {
    if own == TestRole::None && reached_in_tests {
        TestRole::CfgTest
    } else {
        own
    }
}

/// Flatten a `use` tree into entries. A `self` that has no path before it
/// (`use {self};`) names nothing and is an error, as it is in rustc.
fn flatten(tree: &UseTree, prefix: &mut Vec<String>, out: &mut Vec<UseKind>) -> Result<(), String> {
    match tree {
        UseTree::Path { name, rest } => {
            prefix.push(name.clone());
            let result = flatten(rest, prefix, out);
            prefix.pop();
            result
        }
        UseTree::Name(name) if name == "self" => {
            // `use path::{self}` imports the module `path` itself.
            let last = prefix
                .last()
                .ok_or_else(|| "`use {self}` has no path before `self`".to_owned())?;
            out.push(UseKind::Named {
                name: last.clone(),
                segs: prefix.clone(),
            });
            Ok(())
        }
        UseTree::Name(name) => {
            let mut segs = prefix.clone();
            segs.push(name.clone());
            out.push(UseKind::Named {
                name: name.clone(),
                segs,
            });
            Ok(())
        }
        UseTree::Rename { name, alias } if name == "self" => {
            if prefix.is_empty() {
                return Err("`use {self as name}` has no path before `self`".to_owned());
            }
            // `use path as _` brings a trait into scope and binds no name.
            if alias != "_" {
                out.push(UseKind::Named {
                    name: alias.clone(),
                    segs: prefix.clone(),
                });
            }
            Ok(())
        }
        UseTree::Rename { name, alias } => {
            let mut segs = prefix.clone();
            segs.push(name.clone());
            if alias != "_" {
                out.push(UseKind::Named {
                    name: alias.clone(),
                    segs,
                });
            }
            Ok(())
        }
        UseTree::Glob => {
            out.push(UseKind::Glob {
                segs: prefix.clone(),
            });
            Ok(())
        }
        UseTree::Group(items) => {
            for item in items {
                flatten(item, prefix, out)?;
            }
            Ok(())
        }
    }
}

/// Everything the emit step needs to know about the file it is reading.
struct Cx<'a> {
    file: &'a RelPath,
    target: TargetId,
    reach_test: bool,
}

struct Builder<'a> {
    targets: &'a [Target],
    files_at: BTreeMap<String, Vec<&'a SourceFile>>,
    index: Index,
}

impl Builder<'_> {
    /// The module at `path`, which a `mod` declaration in `parent` names. A
    /// module declared more than once (under exclusive `cfg`s) is one module.
    fn declare(&mut self, path: &str, parent: usize) -> usize {
        if let Some(&known) = self.index.module_by_path.get(path) {
            return known;
        }
        let idx = self.index.modules.len();
        self.index.modules.push(Module {
            path: path.to_owned(),
            parent: Some(parent),
            root: self.index.modules[parent].root,
            defs: Vec::new(),
        });
        self.index.module_by_path.insert(path.to_owned(), idx);
        idx
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "an item is built from the fact, its place and the file context"
    )]
    fn push(
        &mut self,
        fact: &ItemFact,
        kind: SymbolKind,
        path: String,
        module: usize,
        cx: &Cx<'_>,
        documented: bool,
    ) -> ItemId {
        let id = ItemId(self.index.items.len());
        self.index.items.push(Item {
            id,
            path,
            name: fact.name.clone(),
            kind,
            namespace: main_namespace(kind),
            file: cx.file.clone(),
            name_line: fact.name_line,
            span: fact.span,
            body: fact.body,
            visibility: fact.vis.clone(),
            documented,
            doc_hidden: fact.doc_hidden,
            test: combine(fact.test, cx.reach_test),
            target: cx.target,
            self_type: None,
            trait_key: None,
            trait_path: None,
            shape: fact.shape,
        });
        self.index.modules[module].defs.push(id);
        id
    }

    fn item(&mut self, fact: &ItemFact, module: usize, cx: &Cx<'_>) {
        if fact.kind == ItemKind::UnnamedConst {
            // `const _` defines no name: it is in the facts and not in a path.
            return;
        }
        let kind = symbol_kind(fact.kind);
        let (home, base) = if fact.macro_export {
            let root = self.index.modules[module].root;
            (root, self.index.modules[root].path.clone())
        } else {
            (module, self.index.modules[module].path.clone())
        };
        let path = format!("{base}::{}", fact.name);
        let id = self.push(fact, kind, path.clone(), home, cx, fact.documented);
        if let Some(alias) = &fact.alias {
            self.index.aliases.insert(id, (home, alias.clone()));
        }
        if fact.kind == ItemKind::ExternCrate {
            let crate_name = fact.renamed_from.as_deref().unwrap_or(&fact.name);
            self.index.extern_names.insert(id, crate_name.to_owned());
            if let Some(&root) = self.index.libs.get(crate_name) {
                self.index.module_of.insert(id, root);
            }
        }
        for member in &fact.members {
            let member_kind = match (member.kind, member.body.is_some()) {
                (ItemKind::Fn, true) => SymbolKind::TraitDefaultMethod,
                (ItemKind::Fn, false) => SymbolKind::TraitRequiredMethod,
                (ItemKind::TypeAlias, _) => SymbolKind::AssocType,
                _ => SymbolKind::AssocConst,
            };
            let member_path = format!("{path}::{}", member.name);
            let member_id = self.member(member, member_kind, member_path, module, cx);
            self.index
                .assoc
                .entry(id)
                .or_default()
                .entry(member.name.clone())
                .or_default()
                .push(member_id);
        }
    }

    /// A member of a type or trait: an item that no module scope names.
    fn member(
        &mut self,
        fact: &ItemFact,
        kind: SymbolKind,
        path: String,
        module: usize,
        cx: &Cx<'_>,
    ) -> ItemId {
        let id = self.push(fact, kind, path, module, cx, fact.documented);
        self.index.modules[module].defs.pop();
        id
    }

    fn body(&mut self, body: &ModBody, module: usize, cx: &Cx<'_>) -> Result<(), LoadError> {
        for fact in &body.items {
            self.item(fact, module, cx);
        }
        for decl in &body.mods {
            let path = format!("{}::{}", self.index.modules[module].path, decl.name);
            let child = self.declare(&path, module);
            let file_docs = self
                .files_at
                .get(&path)
                .is_some_and(|files| files.iter().any(|file| file.facts.documented));
            let fact = ItemFact {
                name: decl.name.clone(),
                renamed_from: None,
                kind: ItemKind::Fn,
                name_line: decl.name_line,
                span: decl.span,
                body: decl.body_span,
                vis: decl.vis.clone(),
                attrs: Vec::new(),
                documented: decl.documented,
                doc_hidden: decl.doc_hidden,
                test: decl.test,
                macro_export: false,
                members: Vec::new(),
                shape: None,
                alias: None,
            };
            let id = self.push(
                &fact,
                SymbolKind::Mod,
                path,
                module,
                cx,
                decl.documented || file_docs,
            );
            self.index.module_of.insert(id, child);
            if let Some(inline) = &decl.inline {
                self.body(inline, child, cx)?;
            }
        }
        for decl in &body.uses {
            let mut kinds = Vec::new();
            flatten(&decl.tree, &mut Vec::new(), &mut kinds).map_err(|what| {
                LoadError::new(
                    cx.file,
                    Some(decl.line),
                    LoadErrorKind::UnsupportedItem { what },
                )
            })?;
            for kind in kinds {
                self.index.uses.push(UseDecl {
                    module,
                    file: cx.file.clone(),
                    line: decl.line,
                    leading_colon: decl.leading_colon,
                    vis: decl.vis.clone(),
                    kind,
                });
            }
        }
        for fact in &body.impls {
            self.index.impls.push(ImplDecl {
                module,
                file: cx.file.clone(),
                target: cx.target,
                reach_test: cx.reach_test,
                fact: fact.clone(),
            });
        }
        Ok(())
    }
}

impl Index {
    /// Build the index of the module trees held by `files`, reading the files
    /// in the order of `visits`, the arrivals of the module-tree walk.
    ///
    /// # Errors
    ///
    /// Returns a [`LoadError`] for a `super` path that climbs above the crate
    /// root and for an inherent impl whose self type is not a local type, and
    /// an `Internal` one for a visit that names no loaded file or reach, or
    /// that reaches a module before the declaration that names it, which the
    /// walk rules out.
    pub(crate) fn build(
        targets: &[Target],
        files: &BTreeMap<RelPath, SourceFile>,
        visits: &[Visit],
        infos: &BTreeMap<String, PackageInfo>,
    ) -> Result<Self, LoadError> {
        let mut files_at: BTreeMap<String, Vec<&SourceFile>> = BTreeMap::new();
        for file in files.values() {
            for reach in &file.reached {
                files_at
                    .entry(reach.module_path.clone())
                    .or_default()
                    .push(file);
            }
        }
        let mut builder = Builder {
            targets,
            files_at,
            index: Index {
                items: Vec::new(),
                order: Vec::new(),
                modules: Vec::new(),
                module_by_path: BTreeMap::new(),
                module_of: BTreeMap::new(),
                libs: BTreeMap::new(),
                crate_roots: BTreeMap::new(),
                targets: targets.to_vec(),
                packages: infos.clone(),
                scopes: Vec::new(),
                uses: Vec::new(),
                use_resolved: Vec::new(),
                use_settled: Vec::new(),
                unsettled: BTreeMap::new(),
                extern_names: BTreeMap::new(),
                root_items: BTreeMap::new(),
                by_source: BTreeMap::new(),
                assoc: BTreeMap::new(),
                foreign: BTreeMap::new(),
                by_file: BTreeMap::new(),
                aliases: BTreeMap::new(),
                impls: Vec::new(),
            },
        };
        for target in builder.targets {
            let root = builder.ensure_root(&target.crate_path);
            if target.kind == TargetKind::Lib {
                builder.index.libs.insert(target.crate_path.clone(), root);
            }
        }
        let mut seen: BTreeSet<(RelPath, TargetId, String)> = BTreeSet::new();
        for visit in visits {
            let internal =
                |what: String| LoadError::new(&visit.file, None, LoadErrorKind::Internal { what });
            let file = files
                .get(&visit.file)
                .ok_or_else(|| internal("a visited file was not loaded".to_owned()))?;
            let reach = file
                .reached
                .get(visit.reach)
                .ok_or_else(|| internal(format!("the file has no module path {}", visit.reach)))?;
            let key = (file.path.clone(), reach.target, reach.module_path.clone());
            // The same file, target and module path can be reached more than
            // once (a `cfg_attr` path alternative walks an inline body again);
            // the facts are the same, so they are indexed once.
            if !seen.insert(key) {
                continue;
            }
            // A crate root is created above, and every other module by the
            // declaration that names it, which the walk reached first.
            let module = *builder
                .index
                .module_by_path
                .get(&reach.module_path)
                .ok_or_else(|| {
                    internal(format!(
                        "the module `{}` was reached before the declaration that names it",
                        reach.module_path
                    ))
                })?;
            let cx = Cx {
                file: &file.path,
                target: reach.target,
                reach_test: reach.test,
            };
            builder.body(&file.facts.body, module, &cx)?;
        }
        let mut index = builder.index;
        index.add_roots(files)?;
        index.build_scopes();
        index.settle()?;
        index.attach_impls()?;
        index.finish();
        Ok(index)
    }

    /// One module item for each target: its crate root, at the crate path.
    /// The root is not named inside any module, so no scope binds it; a path
    /// that is exactly a crate path finds it.
    ///
    /// # Errors
    ///
    /// Returns an `Internal` [`LoadError`] if a target's root file was not
    /// loaded or its crate root has no module, which the walk guarantees.
    fn add_roots(&mut self, files: &BTreeMap<RelPath, SourceFile>) -> Result<(), LoadError> {
        for (position, target) in self.targets.clone().iter().enumerate() {
            let internal =
                |what: String| LoadError::new(&target.root, None, LoadErrorKind::Internal { what });
            let file = files.get(&target.root).ok_or_else(|| {
                internal(format!(
                    "the root file of `{}` was not loaded",
                    target.crate_path
                ))
            })?;
            let module = *self.crate_roots.get(&target.crate_path).ok_or_else(|| {
                internal(format!("`{}` has no crate root module", target.crate_path))
            })?;
            let id = ItemId(self.items.len());
            let lines = file.text.lines().count().max(1);
            let name = target
                .crate_path
                .rsplit("::")
                .next()
                .map_or_else(|| target.crate_path.clone(), str::to_owned);
            self.items.push(Item {
                id,
                path: target.crate_path.clone(),
                name,
                kind: SymbolKind::Mod,
                namespace: Namespace::Type,
                file: file.path.clone(),
                name_line: 1,
                span: SpanFact {
                    start_line: 1,
                    end_line: lines,
                    start_byte: 0,
                    end_byte: file.text.len(),
                },
                body: None,
                visibility: Vis::Public,
                documented: file.facts.documented,
                doc_hidden: file.facts.doc_hidden,
                test: match target.kind {
                    TargetKind::Test | TargetKind::Bench => TestRole::CfgTest,
                    TargetKind::Lib
                    | TargetKind::Bin
                    | TargetKind::Example
                    | TargetKind::BuildScript => TestRole::None,
                },
                target: TargetId(position),
                self_type: None,
                trait_key: None,
                trait_path: None,
                shape: None,
            });
            self.module_of.insert(id, module);
            self.root_items.insert(module, id);
        }
        Ok(())
    }

    /// Sort the items and file them.
    fn finish(&mut self) {
        let mut order: Vec<ItemId> = self.items.iter().map(|item| item.id).collect();
        order.sort_by(|a, b| self.sort_key(*a).cmp(&self.sort_key(*b)));
        self.order = order;
        for item in &self.items {
            self.by_file
                .entry(item.file.clone())
                .or_default()
                .push(item.id);
        }
        let items = &self.items;
        for id in &self.order {
            // A crate root is a whole file, not a source item inside one.
            if self.root_items.values().any(|root| root == id) {
                continue;
            }
            let item = &items[id.0];
            self.by_source
                .entry((item.file.clone(), item.span.start_byte))
                .or_default()
                .push(*id);
        }
        for ids in self.by_file.values_mut() {
            ids.sort_by_key(|id| (items[id.0].span.start_byte, items[id.0].path.clone(), id.0));
        }
    }

    pub(crate) fn sort_key(&self, id: ItemId) -> (&str, Namespace, &RelPath, usize, usize) {
        let item = &self.items[id.0];
        (
            item.path.as_str(),
            item.namespace,
            &item.file,
            item.span.start_byte,
            item.target.index(),
        )
    }

    /// The items of `ids`, sorted and without repeats.
    pub(crate) fn sorted(&self, ids: impl IntoIterator<Item = ItemId>) -> Vec<ItemId> {
        let mut ids: Vec<ItemId> = ids.into_iter().collect();
        ids.sort_by(|a, b| self.sort_key(*a).cmp(&self.sort_key(*b)));
        ids.dedup();
        ids
    }

    pub(crate) fn push_member(
        &mut self,
        decl: &ImplDecl,
        member: &ItemFact,
        kind: SymbolKind,
        path: String,
        self_type: &str,
        trait_key: Option<TraitKey>,
    ) -> ItemId {
        let id = ItemId(self.items.len());
        self.items.push(Item {
            id,
            path,
            name: member.name.clone(),
            kind,
            namespace: main_namespace(kind),
            file: decl.file.clone(),
            name_line: member.name_line,
            span: member.span,
            body: member.body,
            visibility: member.vis.clone(),
            documented: member.documented,
            doc_hidden: member.doc_hidden,
            test: combine(member.test, decl.reach_test),
            target: decl.target,
            self_type: Some(self_type.to_owned()),
            trait_key,
            trait_path: decl
                .fact
                .trait_path
                .as_ref()
                .map(|text| text.split_whitespace().collect()),
            shape: None,
        });
        id
    }

    pub(crate) fn take_impls(&mut self) -> Vec<ImplDecl> {
        std::mem::take(&mut self.impls)
    }

    pub(crate) fn error(decl: &ImplDecl, kind: LoadErrorKind) -> LoadError {
        LoadError::new(&decl.file, Some(decl.fact.line), kind)
    }

    pub(crate) fn path_fact(fact: &ImplFact) -> Option<PathFact> {
        fact.self_path.as_ref().map(|segments| PathFact {
            segments: segments.clone(),
            leading_colon: fact.self_leading_colon,
        })
    }
}

impl Builder<'_> {
    /// Create the module that is the root of a crate.
    fn ensure_root(&mut self, path: &str) -> usize {
        let idx = self.index.modules.len();
        self.index.modules.push(Module {
            path: path.to_owned(),
            parent: None,
            root: idx,
            defs: Vec::new(),
        });
        self.index.module_by_path.insert(path.to_owned(), idx);
        self.index.crate_roots.insert(path.to_owned(), idx);
        idx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::MemoryTree;
    use crate::workspace::Workspace;

    /// The targets and files of a loaded workspace, as the index takes them.
    fn parts(ws: &Workspace) -> (Vec<Target>, BTreeMap<RelPath, SourceFile>) {
        let targets: Vec<Target> = ws.targets().cloned().collect();
        let files: BTreeMap<RelPath, SourceFile> = ws
            .files()
            .map(|file| {
                (
                    file.path.clone(),
                    SourceFile {
                        path: file.path.clone(),
                        package: file.package.clone(),
                        shipping: file.shipping,
                        test: file.test,
                        text: file.text.clone(),
                        facts: file.facts.clone(),
                        reached: file.reached.clone(),
                    },
                )
            })
            .collect();
        (targets, files)
    }

    fn visit(file: &str) -> Visit {
        Visit {
            file: RelPath::new(file).expect("path"),
            reach: 0,
        }
    }

    #[test]
    fn a_target_whose_root_file_is_missing_is_a_typed_internal_error() {
        let tree = MemoryTree::from([
            ("Cargo.toml", "[package]\nname = \"demo\"\n"),
            ("src/lib.rs", ""),
        ]);
        let ws = Workspace::load(&tree, &["."]).expect("loads");
        let (targets, files) = parts(&ws);
        let infos = BTreeMap::new();
        assert!(Index::build(&targets, &files, &[visit("src/lib.rs")], &infos).is_ok());
        let error =
            Index::build(&targets, &BTreeMap::new(), &[], &infos).expect_err("no root file");
        assert!(
            matches!(error.kind, LoadErrorKind::Internal { .. }),
            "{error}"
        );
        assert_eq!(error.file.as_str(), "src/lib.rs");
    }

    #[test]
    fn a_module_file_visited_before_the_declaration_that_names_it_is_an_internal_error() {
        let tree = MemoryTree::from([
            ("Cargo.toml", "[package]\nname = \"demo\"\n"),
            ("src/lib.rs", "pub mod a;\n"),
            ("src/a/mod.rs", "pub mod b;\n"),
            ("src/a/b.rs", "pub fn in_b() {}\n"),
        ]);
        let ws = Workspace::load(&tree, &["."]).expect("loads");
        let (targets, files) = parts(&ws);
        let infos = BTreeMap::new();
        // The order of the walk builds the index.
        let walked = [
            visit("src/lib.rs"),
            visit("src/a/mod.rs"),
            visit("src/a/b.rs"),
        ];
        let index = Index::build(&targets, &files, &walked, &infos).expect("walk order");
        let paths: Vec<&str> = index.items.iter().map(|item| item.path.as_str()).collect();
        assert!(paths.contains(&"demo::a::b::in_b"), "{paths:?}");
        // The order of the file names reaches `b` before `a` declares it.
        let sorted: Vec<Visit> = files.keys().map(|path| visit(path.as_str())).collect();
        let error = Index::build(&targets, &files, &sorted, &infos).expect_err("name order");
        assert!(
            matches!(&error.kind, LoadErrorKind::Internal { what } if what.contains("`demo::a::b`")),
            "{error}"
        );
        assert_eq!(error.file.as_str(), "src/a/b.rs");
        // A visit must name a loaded file and one of its reaches.
        for stray in [
            visit("src/missing.rs"),
            Visit {
                file: RelPath::new("src/lib.rs").expect("path"),
                reach: 1,
            },
        ] {
            let error = Index::build(&targets, &files, &[stray], &infos).expect_err("stray visit");
            assert!(
                matches!(error.kind, LoadErrorKind::Internal { .. }),
                "{error}"
            );
        }
    }
}
