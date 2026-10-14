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

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::mem::size_of;
use std::sync::Arc;

use crate::cost::{btree_entry, btree_map, hash_entry, hash_map, shared_text, vec_first, vec_slot};
use crate::error::{LoadError, LoadErrorKind};
use crate::facts::{
    FieldShape, ImplFact, ItemFact, ItemKind, ModBody, ModFact, PathFact, SpanFact, TestRole,
    UseTree, Vis,
};
use crate::intern::Interner;
use crate::item::{
    Item, ItemId, Namespace, SelfType, SharedStr, SymbolKind, TraitKey, UnresolvedImpl,
};
use crate::manifest::DependencyKind;
use crate::meter::Meter;
use crate::modtree::{TargetId, Visit};
use crate::path::RelPath;
use crate::settle::{Entry, Origin, Scope};
use crate::target::{Target, TargetKind};
use crate::workspace::SourceFile;

/// What an item allocates at most in the index beyond its strings: its record
/// in the list of items, its place in the order and among the items of its
/// file, and its source group (the group's entry and first room).
const ITEM_BYTES: usize = vec_slot::<Item>()
    + 2 * vec_slot::<ItemId>()
    + btree_entry::<(RelPath, usize), Vec<ItemId>>()
    + vec_first::<ItemId>()
    + vec_slot::<ItemId>();

/// What a module allocates at most beyond its path: its record, its entry in
/// the table of module paths, its scope table and its slot among the scopes,
/// the first room of its definitions and of the copy the scopes are built from.
pub(crate) const MODULE_BYTES: usize = vec_slot::<Module>()
    + btree_entry::<(usize, SharedStr), usize>()
    + vec_slot::<Scope>()
    + btree_map::<String, Box<Entry>>()
    + 2 * vec_first::<ItemId>()
    + vec_slot::<(usize, Vec<ItemId>)>();

/// What a type's table of members allocates at most, once for the type.
pub(crate) const MEMBERS_BYTES: usize =
    btree_entry::<ItemId, BTreeMap<String, Vec<ItemId>>>() + btree_map::<String, Vec<ItemId>>();

/// What the tables and lists of one load allocate before their first entries:
/// every load-wide table and list at its first room.
pub(crate) const LOAD_BASE_BYTES: usize = btree_map::<&str, (usize, &str)>()
    + btree_map::<String, Vec<usize>>()
    + btree_map::<String, u16>()
    + btree_map::<(usize, SharedStr), usize>()
    + btree_map::<ItemId, usize>()
    + btree_map::<ItemId, Vec<usize>>()
    + btree_map::<ItemId, SharedStr>()
    + btree_map::<usize, ItemId>()
    + btree_map::<(RelPath, usize), Vec<ItemId>>()
    + btree_map::<ItemId, BTreeMap<String, Vec<ItemId>>>()
    + btree_map::<String, Vec<ItemId>>()
    + btree_map::<RelPath, Vec<ItemId>>()
    + btree_map::<ItemId, (usize, Arc<PathFact>)>()
    + btree_map::<RelPath, SourceFile>()
    + btree_map::<(RelPath, TargetId, SharedStr), ()>()
    + btree_map::<(TargetId, &str), Vec<&SourceFile>>()
    + hash_map::<(crate::tree::FileId, usize)>()
    + hash_map::<(RelPath, crate::tree::FileId)>()
    + 2 * hash_map::<SharedStr>()
    + hash_map::<(*const PathFact, Arc<PathFact>)>()
    + vec_first::<Item>()
    + 3 * vec_first::<ItemId>()
    + vec_first::<Module>()
    + vec_first::<Scope>()
    + vec_first::<UseDecl>()
    + 4 * vec_first::<bool>()
    + 4 * vec_first::<[bool; 3]>()
    + vec_first::<BTreeSet<usize>>()
    + vec_first::<(usize, usize)>()
    + vec_first::<Visit>()
    + vec_first::<ImplDecl<'static>>()
    + vec_first::<UnresolvedImpl>()
    + vec_first::<usize>()
    + vec_first::<BTreeMap<String, ExternName>>()
    + vec_first::<u16>()
    + vec_first::<bool>()
    + 2 * vec_first::<usize>()
    + vec_first::<(usize, u64)>();

/// The first eight bytes of `name`, big-endian, padded with zero bytes: two
/// names compare as their prefixes do whenever the prefixes differ, as a name
/// holds no zero byte.
fn name_prefix(name: &str) -> u64 {
    let mut bytes = [0; 8];
    let taken = name.len().min(8);
    bytes[..taken].copy_from_slice(&name.as_bytes()[..taken]);
    u64::from_be_bytes(bytes)
}

/// What a definition allocates at most in the scope tables: its place among its
/// module's definitions and in the copy the scopes are built from, the entry of
/// its name in its module's scope table (the name copied as the key), and in
/// each name space it is bound in, a binding and the visibility it copies.
fn scope_bytes(name: &str, vis: &Vis, namespaces: usize) -> usize {
    2 * vec_slot::<ItemId>()
        + (btree_entry::<String, Box<Entry>>() + size_of::<Entry>())
        + name.len()
        + namespaces * (vec_slot::<(ItemId, Origin)>() + vis_capacity(vis))
}

/// What an import that binds `name` allocates at most for one binding: the
/// scope entry of the name (copied as the key), the binding, the visibility it
/// copies and the route it records (the name and the name it came through).
pub(crate) fn binding_bytes(name: &str, vis: &Vis, source: usize) -> usize {
    (btree_entry::<String, Box<Entry>>() + size_of::<Entry>())
        + 2 * name.len()
        + vec_slot::<(ItemId, Origin)>()
        + vis_capacity(vis)
        + source
}

/// What a member allocates at most among the members of a type or under the
/// key of a trait: the entry of its name or key (copied) and its slot in the
/// entry's first room.
pub(crate) fn member_bytes(key: &str) -> usize {
    btree_entry::<String, Vec<ItemId>>() + key.len() + vec_first::<ItemId>() + vec_slot::<ItemId>()
}

/// The bytes a visibility holds beyond its size.
fn vis_capacity(vis: &Vis) -> usize {
    match vis {
        Vis::Restricted(text) => text.capacity(),
        Vis::Inherited | Vis::Public => 0,
    }
}

/// The bytes a list of strings owns.
fn strings_bytes(strings: &[String]) -> usize {
    size_of_val(strings) + strings.iter().map(String::capacity).sum::<usize>()
}

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
    /// True for test code: the declaration or a module around it is guarded
    /// by a `test` predicate, or its file is reached only through test code.
    pub(crate) test: bool,
    pub(crate) kind: UseKind,
}

/// What the index knows of a package's manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PackageInfo {
    /// The edition year.
    pub(crate) edition: u16,
    /// Its dependencies, from every table.
    pub(crate) dependencies: Vec<DependencyInfo>,
}

/// A dependency of a package, as the extern prelude of its targets sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DependencyInfo {
    /// The table it is declared in.
    pub(crate) kind: DependencyKind,
    /// The name it is declared under, with `-` as `_`.
    pub(crate) name: String,
    /// True when the entry renames its package (`package = ".."`), so code
    /// names it by `name`.
    pub(crate) renamed: bool,
    /// The package of the workspace its `path` names, if it names one.
    pub(crate) package: Option<String>,
}

/// A crate name in the extern prelude of a target.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ExternName {
    /// True when every code of the target can name it; false when only its
    /// test code can (a dev-dependency of a library or a binary).
    pub(crate) everywhere: bool,
    /// The library crate roots of the workspace that the name stands for,
    /// each with whether every code of the target names it (true) or only
    /// its test code (false); empty for a crate outside the workspace.
    pub(crate) roots: Vec<(usize, bool)>,
}

impl ExternName {
    /// The library crate roots the name stands for in test code (`test`) or
    /// in other code.
    pub(crate) fn roots(&self, test: bool) -> impl Iterator<Item = usize> + '_ {
        self.roots
            .iter()
            .filter(move |(_, everywhere)| *everywhere || test)
            .map(|(root, _)| *root)
    }
}

impl UseDecl {
    /// What the entry allocates at most: its slot in the list of imports, its
    /// four flags, its four sets of flags by name space, its set of modules
    /// reached (each module in it is charged when a settle ends), its place in the
    /// named or the glob imports in order and among the keys of the named
    /// imports, and the strings it owns. What a
    /// named import binds is charged when it binds it.
    fn bytes(&self) -> usize {
        let owned = match &self.kind {
            UseKind::Named { name, segs } => name.capacity() + strings_bytes(segs),
            UseKind::Glob { segs } => strings_bytes(segs),
        };
        vec_slot::<Self>()
            + 4 * vec_slot::<bool>()
            + 4 * vec_slot::<[bool; 3]>()
            + vec_slot::<BTreeSet<usize>>()
            + vec_slot::<usize>()
            + vec_slot::<(usize, u64)>()
            + owned
            + vis_capacity(&self.vis)
    }
}

/// An impl block waiting for its self type to resolve, once for each module
/// path its file is reached under. The fact is the file's own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImplDecl<'f> {
    pub(crate) module: usize,
    pub(crate) file: RelPath,
    pub(crate) target: TargetId,
    pub(crate) reach_test: bool,
    pub(crate) fact: &'f ImplFact,
}

/// A module: all the declarations that define the module at one path of one
/// crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Module {
    pub(crate) path: SharedStr,
    pub(crate) parent: Option<usize>,
    pub(crate) root: usize,
    /// The target whose crate the module is in.
    pub(crate) target: TargetId,
    pub(crate) defs: Vec<ItemId>,
}

/// The index of a workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Index {
    pub(crate) items: Vec<Item>,
    /// The items in the order of [`crate::Workspace::items`].
    pub(crate) order: Vec<ItemId>,
    pub(crate) modules: Vec<Module>,
    /// The module at each path of each crate, by the crate's root module.
    pub(crate) module_by_path: BTreeMap<(usize, SharedStr), usize>,
    /// The module that a `mod` item stands for.
    pub(crate) module_of: BTreeMap<ItemId, usize>,
    /// The crate roots that an `extern crate` item stands for.
    pub(crate) crate_of: BTreeMap<ItemId, Vec<usize>>,
    /// The root modules of the targets of each crate path, in the order of
    /// the targets. Targets of different packages can share one.
    pub(crate) crate_roots: BTreeMap<String, Vec<usize>>,
    /// The root module of each target.
    pub(crate) target_roots: Vec<usize>,
    /// The crate names each target's code can name, by target: its package's
    /// library (for a target other than the library and the build script) and
    /// the dependencies of its kind.
    pub(crate) preludes: Vec<BTreeMap<String, ExternName>>,
    pub(crate) targets: Vec<Target>,
    /// The edition of each package.
    pub(crate) editions: BTreeMap<String, u16>,
    /// The edition of each target's package, by target.
    pub(crate) target_editions: Vec<u16>,
    /// True for a target whose crate root has `#![no_std]`, by target.
    pub(crate) target_no_std: Vec<bool>,
    /// True once a `use` path whose first segment a module with globs does
    /// not bind may start at a crate of the extern prelude: the globs have
    /// brought in what they can without it.
    pub(crate) crates_open: bool,
    /// The inherent impls whose self type the index cannot see, sorted by
    /// path, file and position.
    pub(crate) unresolved: Vec<UnresolvedImpl>,
    pub(crate) scopes: Vec<Scope>,
    pub(crate) uses: Vec<UseDecl>,
    /// The positions of the named imports, sorted by module, bound name and
    /// position.
    pub(crate) named_imports: Vec<usize>,
    /// The module of each of `named_imports` and the first eight bytes of
    /// the name it binds ([`name_prefix`]), in its order, so that the imports
    /// of a name are found without reading most declarations.
    pub(crate) named_keys: Vec<(usize, u64)>,
    /// The positions of the glob imports, sorted by module and position.
    pub(crate) glob_imports: Vec<usize>,
    /// True for an import that bound something.
    pub(crate) use_resolved: Vec<bool>,
    /// True once an import is resolved, has failed or is held as ambiguous.
    /// The named imports of a cycle, and those that wait on one, never
    /// settle.
    pub(crate) use_settled: Vec<bool>,
    /// True for a named import whose first segment is ambiguous: it binds
    /// nothing, and keeps the name it binds from the globs of its module and
    /// the crates in every name space.
    pub(crate) use_ambiguous: Vec<bool>,
    /// For each import and name space, true once the import bound something
    /// there.
    pub(crate) use_bound: Vec<[bool; 3]>,
    /// For each import and name space, true once the import leaves its name
    /// there to the globs of its module: it binds nothing there, and its path
    /// can bind nothing there whatever is still to be settled.
    pub(crate) use_free: Vec<[bool; 3]>,
    /// For each import and name space, true once the import is decided
    /// there: it bound something, or the settling of the tables decided that
    /// it binds nothing there. An import of a cycle, and one that waits on a
    /// cycle, stays undecided in the name spaces the cycle holds.
    pub(crate) use_decided: Vec<[bool; 3]>,
    /// For each import and name space, true once the settling of the tables
    /// found that the import's path can bind nothing there whatever is left
    /// to settle. Such a name space is left to the globs
    /// ([`Index::use_free`]) once the import binds something in another.
    pub(crate) use_empty: Vec<[bool; 3]>,
    /// True for a glob import whose path resolved and whose target modules'
    /// globs, those its module can name, are settled: what it brings in can
    /// then only grow by what its targets bind (see [`Index::pending_at`]).
    /// Until then no name of its module is decided to bind nothing.
    pub(crate) glob_closed: Vec<bool>,
    /// The modules the path of each glob import reached when the glob was
    /// last tried, by position; empty for a named import.
    pub(crate) glob_reach: Vec<BTreeSet<usize>>,
    /// The crate an `extern crate` item names, by item.
    pub(crate) extern_names: BTreeMap<ItemId, SharedStr>,
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
    /// The path of a file's alias is shared by every module path that reaches
    /// the file.
    pub(crate) aliases: BTreeMap<ItemId, (usize, Arc<PathFact>)>,
}

/// Where the index puts its strings while it is built, and what it charges.
pub(crate) struct Store<'m> {
    /// One copy of each path, name and written type the items hold.
    strings: Interner,
    meter: &'m Meter,
}

impl Store<'_> {
    /// The shared copy of `text` and what it costs the first time.
    pub(crate) fn intern(&mut self, text: &str) -> (SharedStr, usize) {
        self.strings.intern(text)
    }

    /// Charge `bytes` of records built for `file` at `line`.
    pub(crate) fn charge(
        &self,
        bytes: usize,
        file: &RelPath,
        line: Option<usize>,
    ) -> Result<(), LoadError> {
        self.meter.items(bytes, file, line)
    }
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
pub(crate) fn combine(own: TestRole, reached_in_tests: bool) -> TestRole {
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
    files_at: BTreeMap<(TargetId, &'a str), Vec<&'a SourceFile>>,
    store: Store<'a>,
    impls: Vec<ImplDecl<'a>>,
    /// The shared aliased path of each alias fact, by the fact's address in
    /// its file's facts, which outlive the build.
    aliases: HashMap<*const PathFact, Arc<PathFact>>,
    index: Index,
}

impl<'a> Builder<'a> {
    /// The module at `path`, which a `mod` declaration in `parent` names, at
    /// `line` of `file`. A module declared more than once (under exclusive
    /// `cfg`s) is one module.
    fn declare(
        &mut self,
        path: &str,
        parent: usize,
        file: &RelPath,
        line: usize,
    ) -> Result<usize, LoadError> {
        let (path, cost) = self.store.intern(path);
        let root = self.index.modules[parent].root;
        let key = (root, path);
        if let Some(&known) = self.index.module_by_path.get(&key) {
            return Ok(known);
        }
        self.store.charge(cost + MODULE_BYTES, file, Some(line))?;
        let idx = self.index.modules.len();
        self.index.modules.push(Module {
            path: key.1.clone(),
            parent: Some(parent),
            root,
            target: self.index.modules[parent].target,
            defs: Vec::new(),
        });
        self.index.module_by_path.insert(key, idx);
        Ok(idx)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "an item is built from the fact, its place and the file context"
    )]
    fn push(
        &mut self,
        fact: &ItemFact,
        kind: SymbolKind,
        path: &str,
        module: usize,
        cx: &Cx<'_>,
        documented: bool,
    ) -> Result<ItemId, LoadError> {
        self.record(fact, kind, path, cx, documented, true)
            .inspect(|id| self.index.modules[module].defs.push(*id))
    }

    /// Record an item, charging it and, when `in_scope`, its definition in
    /// its module's scope table, or else its place among the members of its
    /// type.
    fn record(
        &mut self,
        fact: &ItemFact,
        kind: SymbolKind,
        path: &str,
        cx: &Cx<'_>,
        documented: bool,
        in_scope: bool,
    ) -> Result<ItemId, LoadError> {
        let (path, path_cost) = self.store.intern(path);
        let (name, name_cost) = self.store.intern(&fact.name);
        let namespaces = if kind == SymbolKind::Struct
            && matches!(fact.shape, Some(FieldShape::Unit | FieldShape::Tuple))
        {
            2
        } else {
            1
        };
        let place = if in_scope {
            scope_bytes(&fact.name, &fact.vis, namespaces)
        } else {
            member_bytes(&fact.name)
        };
        self.store.charge(
            ITEM_BYTES + path_cost + name_cost + vis_capacity(&fact.vis) + place,
            cx.file,
            Some(fact.name_line),
        )?;
        let id = ItemId(self.index.items.len());
        self.index.items.push(Item {
            id,
            path,
            name,
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
            self_key: None,
            trait_path: None,
            shape: fact.shape,
        });
        Ok(id)
    }

    /// The shared copy of the aliased path of `alias`, a fact of the file, made
    /// and charged the first time any module path of the file reaches it.
    fn shared_alias(
        &mut self,
        alias: &PathFact,
        cx: &Cx<'_>,
        line: usize,
    ) -> Result<Arc<PathFact>, LoadError> {
        if let Some(shared) = self.aliases.get(&std::ptr::from_ref(alias)) {
            return Ok(Arc::clone(shared));
        }
        self.store.charge(
            shared_text(size_of::<PathFact>())
                + strings_bytes(&alias.segments)
                + hash_entry::<(*const PathFact, Arc<PathFact>)>()
                + btree_entry::<ItemId, (usize, Arc<PathFact>)>(),
            cx.file,
            Some(line),
        )?;
        let shared = Arc::new(alias.clone());
        self.aliases
            .insert(std::ptr::from_ref(alias), Arc::clone(&shared));
        Ok(shared)
    }

    fn item(&mut self, fact: &ItemFact, module: usize, cx: &Cx<'_>) -> Result<(), LoadError> {
        if fact.kind == ItemKind::UnnamedConst {
            // `const _` defines no name: it is in the facts and not in a path.
            return Ok(());
        }
        let kind = symbol_kind(fact.kind);
        let (home, base) = if fact.macro_export {
            let root = self.index.modules[module].root;
            (root, self.index.modules[root].path.clone())
        } else {
            (module, self.index.modules[module].path.clone())
        };
        let path = format!("{base}::{}", fact.name);
        let id = self.push(fact, kind, &path, home, cx, fact.documented)?;
        if let Some(alias) = &fact.alias {
            let shared = self.shared_alias(alias, cx, fact.name_line)?;
            self.store.charge(
                btree_entry::<ItemId, (usize, Arc<PathFact>)>(),
                cx.file,
                Some(fact.name_line),
            )?;
            self.index.aliases.insert(id, (home, shared));
        }
        if fact.kind == ItemKind::ExternCrate {
            let crate_name = fact.renamed_from.as_deref().unwrap_or(&fact.name);
            let (shared, cost) = self.store.intern(crate_name);
            self.store.charge(
                cost + btree_entry::<ItemId, SharedStr>(),
                cx.file,
                Some(fact.name_line),
            )?;
            self.index.extern_names.insert(id, shared);
            // The crates the item names: the crate itself for `self`, or every
            // library of the workspace the target's extern prelude gives the
            // name, so a path through a name of several is told apart by what
            // each of them defines, as a path through the name itself is.
            let test = fact.test != TestRole::None || cx.reach_test;
            let roots: Vec<usize> = if crate_name == "self" {
                vec![self.index.modules[module].root]
            } else {
                self.index
                    .extern_name(module, crate_name, test)
                    .map(|name| name.roots(test).collect())
                    .unwrap_or_default()
            };
            if !roots.is_empty() {
                self.store.charge(
                    btree_entry::<ItemId, Vec<usize>>() + roots.capacity() * size_of::<usize>(),
                    cx.file,
                    Some(fact.name_line),
                )?;
                self.index.crate_of.insert(id, roots);
            }
        }
        if !fact.members.is_empty() {
            self.store
                .charge(MEMBERS_BYTES, cx.file, Some(fact.name_line))?;
        }
        for member in &fact.members {
            let member_kind = match (member.kind, member.body.is_some()) {
                (ItemKind::Fn, true) => SymbolKind::TraitDefaultMethod,
                (ItemKind::Fn, false) => SymbolKind::TraitRequiredMethod,
                (ItemKind::TypeAlias, _) => SymbolKind::AssocType,
                _ => SymbolKind::AssocConst,
            };
            let member_path = format!("{path}::{}", member.name);
            let member_id = self.member(member, member_kind, &member_path, cx)?;
            self.index
                .assoc
                .entry(id)
                .or_default()
                .entry(member.name.clone())
                .or_default()
                .push(member_id);
        }
        Ok(())
    }

    /// A member of a type or trait: an item that no module scope names.
    fn member(
        &mut self,
        fact: &ItemFact,
        kind: SymbolKind,
        path: &str,
        cx: &Cx<'_>,
    ) -> Result<ItemId, LoadError> {
        self.record(fact, kind, path, cx, fact.documented, false)
    }

    /// Emit a module body and the inline modules inside it, depth first, on
    /// an explicit stack: the items, then each module declaration with the
    /// body of an inline one, then the `use` declarations and the impls.
    fn body(&mut self, root: &'a ModBody, module: usize, cx: &Cx<'_>) -> Result<(), LoadError> {
        let mut stack = vec![Emit::Enter(root, module)];
        while let Some(step) = stack.pop() {
            match step {
                Emit::Enter(body, module) => {
                    for fact in &body.items {
                        self.item(fact, module, cx)?;
                    }
                    stack.push(Emit::Rest(body, module));
                    stack.push(Emit::Mods(body, module, 0));
                }
                Emit::Mods(body, module, next) => {
                    let Some(decl) = body.mods.get(next) else {
                        continue;
                    };
                    stack.push(Emit::Mods(body, module, next + 1));
                    let child = self.module_item(decl, module, cx)?;
                    if let Some(inline) = &decl.inline {
                        stack.push(Emit::Enter(inline, child));
                    }
                }
                Emit::Rest(body, module) => self.uses_and_impls(body, module, cx)?,
            }
        }
        Ok(())
    }

    /// The module a declaration names, and its item. Returns the module.
    fn module_item(
        &mut self,
        decl: &ModFact,
        module: usize,
        cx: &Cx<'_>,
    ) -> Result<usize, LoadError> {
        let path = format!("{}::{}", self.index.modules[module].path, decl.name);
        let child = self.declare(&path, module, cx.file, decl.name_line)?;
        let file_docs = self
            .files_at
            .get(&(cx.target, path.as_str()))
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
            &path,
            module,
            cx,
            decl.documented || file_docs,
        )?;
        self.store.charge(
            btree_entry::<ItemId, usize>(),
            cx.file,
            Some(decl.name_line),
        )?;
        self.index.module_of.insert(id, child);
        Ok(child)
    }

    /// The `use` declarations and the impl blocks of one body.
    fn uses_and_impls(
        &mut self,
        body: &'a ModBody,
        module: usize,
        cx: &Cx<'_>,
    ) -> Result<(), LoadError> {
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
                let entry = UseDecl {
                    module,
                    file: cx.file.clone(),
                    line: decl.line,
                    leading_colon: decl.leading_colon,
                    vis: decl.vis.clone(),
                    test: decl.test != TestRole::None || cx.reach_test,
                    kind,
                };
                self.store.charge(entry.bytes(), cx.file, Some(decl.line))?;
                self.index.uses.push(entry);
            }
        }
        for fact in &body.impls {
            self.store
                .charge(vec_slot::<ImplDecl<'_>>(), cx.file, Some(fact.line))?;
            self.impls.push(ImplDecl {
                module,
                file: cx.file.clone(),
                target: cx.target,
                reach_test: cx.reach_test,
                fact,
            });
        }
        Ok(())
    }
}

/// A step of emitting a module body: its items and module declarations, a
/// declaration from position `next` on, or its imports and impls.
enum Emit<'a> {
    Enter(&'a ModBody, usize),
    Mods(&'a ModBody, usize, usize),
    Rest(&'a ModBody, usize),
}

impl Index {
    /// An index of `targets` that holds nothing yet.
    fn empty(targets: &[Target]) -> Self {
        Self {
            items: Vec::new(),
            order: Vec::new(),
            modules: Vec::new(),
            module_by_path: BTreeMap::new(),
            module_of: BTreeMap::new(),
            crate_of: BTreeMap::new(),
            crate_roots: BTreeMap::new(),
            target_roots: Vec::new(),
            preludes: Vec::new(),
            targets: targets.to_vec(),
            editions: BTreeMap::new(),
            target_editions: Vec::new(),
            target_no_std: Vec::new(),
            crates_open: false,
            unresolved: Vec::new(),
            scopes: Vec::new(),
            uses: Vec::new(),
            named_imports: Vec::new(),
            named_keys: Vec::new(),
            glob_imports: Vec::new(),
            use_resolved: Vec::new(),
            use_settled: Vec::new(),
            use_ambiguous: Vec::new(),
            use_bound: Vec::new(),
            use_free: Vec::new(),
            use_decided: Vec::new(),
            use_empty: Vec::new(),
            glob_closed: Vec::new(),
            glob_reach: Vec::new(),
            extern_names: BTreeMap::new(),
            root_items: BTreeMap::new(),
            by_source: BTreeMap::new(),
            assoc: BTreeMap::new(),
            foreign: BTreeMap::new(),
            by_file: BTreeMap::new(),
            aliases: BTreeMap::new(),
        }
    }

    /// Build the index of the module trees held by `files`, reading the files
    /// in the order of `visits`, the arrivals of the module-tree walk.
    ///
    /// # Errors
    ///
    /// Returns a [`LoadError`] for a `super` path that climbs above the crate
    /// root, for the trait of an impl that cannot be named and for records
    /// that pass the item-bytes budget of `meter`, and an `Internal`
    /// one for a visit that names no loaded file or reach, or that reaches a
    /// module before the declaration that names it, which the walk rules out.
    /// `strings` holds the module paths the walk recorded.
    pub(crate) fn build(
        targets: &[Target],
        files: &BTreeMap<RelPath, SourceFile>,
        visits: &[Visit],
        infos: &BTreeMap<String, PackageInfo>,
        strings: Interner,
        meter: &Meter,
    ) -> Result<Self, LoadError> {
        let mut files_at: BTreeMap<(TargetId, &str), Vec<&SourceFile>> = BTreeMap::new();
        for file in files.values() {
            for reach in &file.reached {
                files_at
                    .entry((reach.target, reach.module_path.as_str()))
                    .or_default()
                    .push(file);
            }
        }
        let mut builder = Builder {
            targets,
            files_at,
            store: Store { strings, meter },
            impls: Vec::new(),
            aliases: HashMap::new(),
            index: Index::empty(targets),
        };
        for (position, target) in builder.targets.iter().enumerate() {
            builder.ensure_root(TargetId(position), target)?;
        }
        builder.preludes(infos)?;
        let mut seen: BTreeSet<(RelPath, TargetId, SharedStr)> = BTreeSet::new();
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
            let root = *builder
                .index
                .target_roots
                .get(reach.target.index())
                .ok_or_else(|| {
                    internal(format!("the target {} has no root", reach.target.index()))
                })?;
            let module = *builder
                .index
                .module_by_path
                .get(&(root, reach.module_path.clone()))
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
        let Builder {
            impls,
            mut index,
            mut store,
            ..
        } = builder;
        index.add_roots(files, &mut store)?;
        index.order_imports();
        index.build_scopes();
        index.settle_agreed(store.meter)?;
        index.attach_impls(impls, &mut store)?;
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
    fn add_roots(
        &mut self,
        files: &BTreeMap<RelPath, SourceFile>,
        store: &mut Store<'_>,
    ) -> Result<(), LoadError> {
        for (position, target) in self.targets.clone().iter().enumerate() {
            let internal =
                |what: String| LoadError::new(&target.root, None, LoadErrorKind::Internal { what });
            let file = files.get(&target.root).ok_or_else(|| {
                internal(format!(
                    "the root file of `{}` was not loaded",
                    target.crate_path
                ))
            })?;
            let module = *self.target_roots.get(position).ok_or_else(|| {
                internal(format!("`{}` has no crate root module", target.crate_path))
            })?;
            let id = ItemId(self.items.len());
            let lines = file.text.lines().count().max(1);
            let name = target
                .crate_path
                .rsplit("::")
                .next()
                .unwrap_or(&target.crate_path);
            let (path, path_cost) = store.intern(&target.crate_path);
            let (name, name_cost) = store.intern(name);
            store.charge(
                ITEM_BYTES
                    + path_cost
                    + name_cost
                    + btree_entry::<ItemId, usize>()
                    + btree_entry::<usize, ItemId>()
                    + vec_slot::<bool>(),
                &target.root,
                None,
            )?;
            // `#![no_std]` takes `std` out of the extern prelude; under a
            // `cfg_attr` it applies only under its predicate, so `std` stays.
            self.target_no_std.push(
                file.facts
                    .attrs
                    .iter()
                    .any(|attr| attr.path == "no_std" && attr.args.is_empty()),
            );
            self.items.push(Item {
                id,
                path,
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
                self_key: None,
                trait_path: None,
                shape: None,
            });
            self.module_of.insert(id, module);
            self.root_items.insert(module, id);
        }
        Ok(())
    }

    /// Sort the items and the unresolved impls, and file the items.
    fn finish(&mut self) {
        self.unresolved.sort_unstable_by(|a, b| {
            (&a.path, &a.file, a.span.start_byte, a.target).cmp(&(
                &b.path,
                &b.file,
                b.span.start_byte,
                b.target,
            ))
        });
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
            ids.sort_by(|a, b| {
                let (a, b) = (&items[a.0], &items[b.0]);
                (a.span.start_byte, &a.path, a.id).cmp(&(b.span.start_byte, &b.path, b.id))
            });
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

    /// Record a member of an impl block.
    ///
    /// # Errors
    ///
    /// Returns a [`LoadError`] naming the member's file and line when the
    /// record passes the item-bytes budget.
    #[allow(
        clippy::too_many_arguments,
        reason = "a member is built from its impl, its fact and what the impl resolved to"
    )]
    pub(crate) fn push_member(
        &mut self,
        decl: &ImplDecl<'_>,
        member: &ItemFact,
        kind: SymbolKind,
        path: &str,
        self_type: &str,
        self_key: SelfType,
        trait_key: Option<Arc<TraitKey>>,
        store: &mut Store<'_>,
    ) -> Result<ItemId, LoadError> {
        let (path, path_cost) = store.intern(path);
        let (name, name_cost) = store.intern(&member.name);
        let (self_type, self_cost) = store.intern(self_type);
        let written = decl
            .fact
            .trait_path
            .as_ref()
            .map(|text| text.split_whitespace().collect::<String>());
        let (trait_path, trait_cost) = match written {
            Some(text) => {
                let (shared, cost) = store.intern(&text);
                (Some(shared), cost)
            }
            None => (None, 0),
        };
        store.charge(
            ITEM_BYTES + path_cost + name_cost + self_cost + trait_cost + vis_capacity(&member.vis),
            &decl.file,
            Some(member.name_line),
        )?;
        let id = ItemId(self.items.len());
        self.items.push(Item {
            id,
            path,
            name,
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
            self_type: Some(self_type),
            trait_key,
            self_key: Some(self_key),
            trait_path,
            shape: None,
        });
        Ok(id)
    }

    /// The crate name `name` in the extern prelude of the target of `module`,
    /// for its test code when `test`; `None` when that code cannot name it.
    pub(crate) fn extern_name(&self, module: usize, name: &str, test: bool) -> Option<&ExternName> {
        let target = self.modules[module].target;
        self.preludes
            .get(target.index())?
            .get(name)
            .filter(|entry| entry.everywhere || test)
    }

    /// The edition of the package whose crate `module` is in.
    pub(crate) fn edition(&self, module: usize) -> u16 {
        self.target_editions[self.modules[module].target.index()]
    }

    /// True when the crate `module` is in has `#![no_std]`.
    pub(crate) fn no_std(&self, module: usize) -> bool {
        self.target_no_std[self.modules[module].target.index()]
    }

    /// The sysroot crate the compiler puts at the crate root of `module`'s
    /// crate: `core` under `#![no_std]`, else `std`.
    pub(crate) fn injected_crate(&self, module: usize) -> &'static str {
        if self.no_std(module) { "core" } else { "std" }
    }

    /// Sort the positions of the named imports by module, bound name and
    /// position, and those of the glob imports by module and position. Each
    /// list takes exactly its room, which every import's charge covers.
    fn order_imports(&mut self) {
        let named = self
            .uses
            .iter()
            .filter(|decl| matches!(decl.kind, UseKind::Named { .. }))
            .count();
        let mut named_imports = Vec::with_capacity(named);
        let mut glob_imports = Vec::with_capacity(self.uses.len() - named);
        for (position, decl) in self.uses.iter().enumerate() {
            match decl.kind {
                UseKind::Named { .. } => named_imports.push(position),
                UseKind::Glob { .. } => glob_imports.push(position),
            }
        }
        named_imports.sort_by(|a, b| self.named_key(*a).cmp(&self.named_key(*b)));
        glob_imports.sort_by_key(|position| (self.uses[*position].module, *position));
        let mut named_keys = Vec::with_capacity(named);
        named_keys.extend(named_imports.iter().map(|position| {
            let (module, name, _) = self.named_key(*position);
            (module, name_prefix(name))
        }));
        self.named_keys = named_keys;
        self.named_imports = named_imports;
        self.glob_imports = glob_imports;
    }

    /// The module and the name a named import binds, and its position.
    fn named_key(&self, position: usize) -> (usize, &str, usize) {
        let decl = &self.uses[position];
        let name = match &decl.kind {
            UseKind::Named { name, .. } => name.as_str(),
            UseKind::Glob { .. } => "",
        };
        (decl.module, name, position)
    }

    /// The named imports of `module` that bind `name`, in order of position.
    pub(crate) fn imports_of(&self, module: usize, name: &str) -> &[usize] {
        // The names are sorted, so their prefixes are too: the imports of
        // `name` are among those of its module and prefix.
        let key = (module, name_prefix(name));
        let from = self.named_keys.partition_point(|at| *at < key);
        let to = from + self.named_keys[from..].partition_point(|at| *at == key);
        let here = &self.named_imports[from..to];
        let start = here.partition_point(|position| self.named_key(*position).1 < name);
        let end = here[start..].partition_point(|position| self.named_key(*position).1 == name);
        &here[start..start + end]
    }

    /// What settling the tables again allocates at most beyond the bindings
    /// its imports make: the scope tables built from the definitions, with
    /// the copy of each module's definitions they are built from, and the
    /// state of every import.
    pub(crate) fn settle_bytes(&self) -> usize {
        let definitions: usize = self
            .modules
            .iter()
            .flat_map(|module| &module.defs)
            .map(|id| {
                let item = &self.items[id.0];
                let namespaces = Namespace::ALL
                    .into_iter()
                    .filter(|ns| item.in_namespace(*ns))
                    .count();
                scope_bytes(&item.name, &item.visibility, namespaces)
            })
            .sum();
        definitions
            + self.modules.len()
                * (vec_slot::<Scope>()
                    + btree_map::<String, Box<Entry>>()
                    + 2 * vec_first::<ItemId>()
                    + vec_slot::<(usize, Vec<ItemId>)>())
            + vec_first::<Scope>()
            + self.uses.len()
                * (4 * vec_slot::<bool>()
                    + 4 * vec_slot::<[bool; 3]>()
                    + vec_slot::<BTreeSet<usize>>())
            + 4 * vec_first::<bool>()
            + 4 * vec_first::<[bool; 3]>()
            + vec_first::<BTreeSet<usize>>()
    }

    /// The glob imports of `module`, in order of position.
    pub(crate) fn globs_of(&self, module: usize) -> &[usize] {
        let start = self
            .glob_imports
            .partition_point(|position| self.uses[*position].module < module);
        let end = self.glob_imports[start..]
            .partition_point(|position| self.uses[*position].module == module);
        &self.glob_imports[start..start + end]
    }

    pub(crate) fn error(decl: &ImplDecl<'_>, kind: LoadErrorKind) -> LoadError {
        LoadError::new(&decl.file, Some(decl.fact.line), kind)
    }

    pub(crate) fn path_fact(fact: &ImplFact) -> Option<PathFact> {
        fact.self_path.as_ref().map(|segments| PathFact {
            segments: segments.clone(),
            leading_colon: fact.self_leading_colon,
        })
    }
}

/// The edition of the package of `target`. A package without facts is an
/// internal error, which discovery rules out.
fn edition_of(target: &Target, infos: &BTreeMap<String, PackageInfo>) -> Result<u16, LoadError> {
    infos
        .get(&target.package)
        .map(|info| info.edition)
        .ok_or_else(|| {
            LoadError::new(
                &target.root,
                None,
                LoadErrorKind::Internal {
                    what: format!("the package `{}` has no manifest facts", target.package),
                },
            )
        })
}

impl Builder<'_> {
    /// Create the module that is the root of a target's crate.
    fn ensure_root(&mut self, id: TargetId, target: &Target) -> Result<usize, LoadError> {
        let idx = self.index.modules.len();
        let path = target.crate_path.as_str();
        let (shared, cost) = self.store.intern(path);
        // Its module, its entry among the crate roots and its slot among the
        // roots of the targets.
        self.store.charge(
            cost + MODULE_BYTES
                + btree_entry::<String, Vec<usize>>()
                + path.len()
                + vec_first::<usize>()
                + 2 * vec_slot::<usize>(),
            &target.root,
            None,
        )?;
        self.index.modules.push(Module {
            path: shared.clone(),
            parent: None,
            root: idx,
            target: id,
            defs: Vec::new(),
        });
        self.index.module_by_path.insert((idx, shared), idx);
        self.index
            .crate_roots
            .entry(path.to_owned())
            .or_default()
            .push(idx);
        self.index.target_roots.push(idx);
        Ok(idx)
    }

    /// The extern prelude and the edition of every target, and the edition of
    /// every package. A target whose package has no facts is an internal
    /// error, which discovery rules out.
    ///
    /// A target other than the library and the build script names its
    /// package's library; each target names the dependencies of its kind: the
    /// build dependencies for the build script, the normal and dev
    /// dependencies for a test, bench or example, and the normal dependencies
    /// for a library or binary, whose test code also names the dev
    /// dependencies. A dependency on a package of the workspace stands for
    /// that package's library, under its library's crate name, or under the
    /// name it is declared under when it renames its package.
    fn preludes(&mut self, infos: &BTreeMap<String, PackageInfo>) -> Result<(), LoadError> {
        let mut libraries: BTreeMap<&str, (usize, &str)> = BTreeMap::new();
        for (position, target) in self.targets.iter().enumerate() {
            if target.kind == TargetKind::Lib {
                self.store
                    .charge(btree_entry::<&str, (usize, &str)>(), &target.root, None)?;
                libraries.insert(
                    &target.package,
                    (self.index.target_roots[position], &target.crate_path),
                );
            }
        }
        for (package, info) in infos {
            self.store.charge(
                btree_entry::<String, u16>() + package.capacity(),
                &RelPath::root(),
                None,
            )?;
            self.index.editions.insert(package.clone(), info.edition);
        }
        for target in self.targets {
            self.store.charge(vec_slot::<u16>(), &target.root, None)?;
            self.index.target_editions.push(edition_of(target, infos)?);
            let mut names: BTreeMap<String, ExternName> = BTreeMap::new();
            let mut offer =
                |store: &Store<'_>, name: &str, root: Option<usize>, everywhere: bool| {
                    let fresh = !names.contains_key(name);
                    let pushed = root.is_some();
                    store.charge(
                        if fresh {
                            btree_entry::<String, ExternName>() + name.len()
                        } else {
                            0
                        } + if pushed {
                            vec_first::<(usize, bool)>() + vec_slot::<(usize, bool)>()
                        } else {
                            0
                        },
                        &target.root,
                        None,
                    )?;
                    let entry = names.entry(name.to_owned()).or_default();
                    entry.everywhere |= everywhere;
                    if let Some(root) = root {
                        match entry.roots.iter_mut().find(|(known, _)| *known == root) {
                            Some((_, known)) => *known |= everywhere,
                            None => entry.roots.push((root, everywhere)),
                        }
                    }
                    Ok::<(), LoadError>(())
                };
            if !matches!(target.kind, TargetKind::Lib | TargetKind::BuildScript)
                && let Some((root, name)) = libraries.get(target.package.as_str())
            {
                offer(&self.store, name, Some(*root), true)?;
            }
            let dependencies = infos
                .get(&target.package)
                .map_or(&[][..], |info| info.dependencies.as_slice());
            for dependency in dependencies {
                let everywhere = match (target.kind, dependency.kind) {
                    (TargetKind::BuildScript, DependencyKind::Build)
                    | (
                        TargetKind::Test | TargetKind::Bench | TargetKind::Example,
                        DependencyKind::Normal | DependencyKind::Dev,
                    )
                    | (TargetKind::Lib | TargetKind::Bin, DependencyKind::Normal) => true,
                    (TargetKind::Lib | TargetKind::Bin, DependencyKind::Dev) => false,
                    (TargetKind::BuildScript, DependencyKind::Normal | DependencyKind::Dev)
                    | (_, DependencyKind::Build) => continue,
                };
                let library = dependency
                    .package
                    .as_deref()
                    .and_then(|package| libraries.get(package));
                let name = match library {
                    Some((_, crate_name)) if !dependency.renamed => crate_name,
                    _ => dependency.name.as_str(),
                };
                offer(
                    &self.store,
                    name,
                    library.map(|(root, _)| *root),
                    everywhere,
                )?;
            }
            self.store.charge(
                vec_slot::<BTreeMap<String, ExternName>>() + btree_map::<String, ExternName>(),
                &target.root,
                None,
            )?;
            self.index.preludes.push(names);
        }
        Ok(())
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

    /// Build an index with the default budgets.
    fn build_index(
        targets: &[Target],
        files: &BTreeMap<RelPath, SourceFile>,
        visits: &[Visit],
        infos: &BTreeMap<String, PackageInfo>,
    ) -> Result<Index, LoadError> {
        let meter = Meter::new(&crate::options::LoadOptions::default());
        Index::build(targets, files, visits, infos, Interner::default(), &meter)
    }

    /// The facts of the one package `demo`, as the index takes them.
    fn demo_infos() -> BTreeMap<String, PackageInfo> {
        BTreeMap::from([(
            "demo".to_owned(),
            PackageInfo {
                edition: 2021,
                dependencies: Vec::new(),
            },
        )])
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
        let infos = demo_infos();
        assert!(build_index(&targets, &files, &[visit("src/lib.rs")], &infos).is_ok());
        // A target whose package has no facts is an internal error too.
        let error = build_index(&targets, &files, &[visit("src/lib.rs")], &BTreeMap::new())
            .expect_err("no package facts");
        assert!(
            matches!(&error.kind, LoadErrorKind::Internal { what } if what.contains("`demo`")),
            "{error}"
        );
        let error = build_index(&targets, &BTreeMap::new(), &[], &infos).expect_err("no root file");
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
        let infos = demo_infos();
        // The order of the walk builds the index.
        let walked = [
            visit("src/lib.rs"),
            visit("src/a/mod.rs"),
            visit("src/a/b.rs"),
        ];
        let index = build_index(&targets, &files, &walked, &infos).expect("walk order");
        let paths: Vec<&str> = index.items.iter().map(|item| item.path.as_str()).collect();
        assert!(paths.contains(&"demo::a::b::in_b"), "{paths:?}");
        // The order of the file names reaches `b` before `a` declares it.
        let sorted: Vec<Visit> = files.keys().map(|path| visit(path.as_str())).collect();
        let error = build_index(&targets, &files, &sorted, &infos).expect_err("name order");
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
            let error = build_index(&targets, &files, &[stray], &infos).expect_err("stray visit");
            assert!(
                matches!(error.kind, LoadErrorKind::Internal { .. }),
                "{error}"
            );
        }
    }
}
