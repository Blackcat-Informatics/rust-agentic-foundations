// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Name resolution over the settled scope tables.
//!
//! A path is walked segment by segment through the tables built by
//! [`crate::settle`]. Visibility is not checked on a walk: the index answers
//! whether a path names an item, and a private item is still an item. Glob
//! imports are the one place visibility counts, because rustc imports only the
//! names the importing module can name.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use crate::error::Esc;
use crate::index::{Index, UseKind};
use crate::item::{ItemId, Namespace, SymbolKind};
use crate::modtree::TargetId;
use crate::settle::{Source, slot};

/// The items a walk reached, and the modules a keyword path named.
#[derive(Debug, Default)]
pub(crate) struct Walked {
    pub(crate) modules: BTreeSet<usize>,
    pub(crate) items: BTreeMap<ItemId, Hit>,
}

/// How a walk reached an item.
#[derive(Debug, Clone)]
pub(crate) struct Hit {
    pub(crate) via_glob: bool,
    pub(crate) last: Option<Source>,
}

impl Hit {
    /// Of two routes to one item the lesser wins: one that uses no glob beats
    /// one that does, and ties go to the smaller source.
    fn rank(&self) -> (bool, &Option<Source>) {
        (self.via_glob, &self.last)
    }
}

/// Add a route to an item, keeping the better route if it is already there.
fn merge(found: &mut BTreeMap<ItemId, Hit>, id: ItemId, hit: Hit) {
    match found.entry(id) {
        std::collections::btree_map::Entry::Vacant(vacant) => {
            vacant.insert(hit);
        }
        std::collections::btree_map::Entry::Occupied(mut occupied) => {
            if hit.rank() < occupied.get().rank() {
                occupied.insert(hit);
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum Fail {
    Missing(Vec<Source>),
    SuperAboveRoot,
    /// The path goes through a name that stands for the crate roots of
    /// several crates, as a crate name of the extern prelude that names
    /// several libraries does: `roots` are those crate roots, and the path
    /// would go on in each with its segments from `rest` on. rustc refuses
    /// such a name whatever follows it.
    SeveralCrates {
        roots: Vec<ItemId>,
        rest: usize,
    },
    /// The first segment of a `use` path of edition 2018 or later names both
    /// what a glob of its module brings in and a different crate of the
    /// extern prelude, which rustc refuses as ambiguous: `ids` are the items
    /// the glob brings in and the crate roots of the workspace that the crate
    /// name stands for.
    Ambiguous(Vec<ItemId>),
}

/// The globs of the settled tables whose paths resolve, with the modules
/// they glob, and those that a lookup reading them fails for (see
/// [`ResolveError::UnresolvedGlob`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct GlobFacts {
    reach: BTreeMap<usize, BTreeSet<usize>>,
    broken: BTreeSet<usize>,
}

/// What the walks of a canonical path from its crate roots found: the items
/// they reach, the places they found unbound and the places they read.
#[derive(Default)]
struct Walks {
    found: BTreeMap<ItemId, Hit>,
    missing: Vec<Source>,
    reads: Vec<Source>,
}

/// What the path of a glob import names in the settled tables.
enum GlobPath {
    /// The modules it globs.
    Resolves(BTreeSet<usize>),
    /// Nothing, as rustc refuses it: a path through a name that stands for
    /// several crates, an ambiguous first segment, or `super` above the root.
    Nothing,
    /// What the index cannot see: a first segment that names nothing in the
    /// workspace, as a dependency, or a path that goes on in a crate outside
    /// it.
    Unknown,
    /// Nothing past its first segment: the places where it names nothing.
    Misses(Vec<Source>),
}

/// What the first segment of a path may name besides the items in scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Heads {
    /// Nothing else: a canonical path, which starts at a crate root.
    Local,
    /// A crate of the extern prelude of the module's target, as code written
    /// in the module names it; `test` for test code, which also names the
    /// dev-dependencies of a library or a binary.
    Code {
        /// True for test code.
        test: bool,
    },
    /// The path of a `use` declaration. In edition 2015 a path whose first
    /// segment is not `crate`, `self` or `super` starts at the crate root. In
    /// edition 2018 and later it starts as `Code` does, and a first segment
    /// that a glob of the module binds and that is also a different crate of
    /// the extern prelude is ambiguous. `own` is the named import whose path
    /// it is: an import never resolves through its own binding.
    Use {
        /// True for test code.
        test: bool,
        /// The position of the named import whose path this is.
        own: Option<usize>,
    },
}

impl Heads {
    /// The named import whose own binding the walk does not consult.
    pub(crate) fn own(self) -> Option<usize> {
        match self {
            Self::Use { own, .. } => own,
            Self::Local | Self::Code { .. } => None,
        }
    }
}

/// What `resolve` found.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Resolution {
    /// The canonical path of the symbol: where it is defined.
    pub path: String,
    /// The name space the path was resolved in.
    pub namespace: Namespace,
    /// The definitions of the symbol, never empty and sorted by path, file and
    /// position. More than one means the same symbol is defined more than
    /// once, as `#[cfg(unix)] fn a()` beside `#[cfg(windows)] fn a()`.
    pub sites: Vec<ItemId>,
    /// True when a glob import carried the path to the symbol.
    pub via_glob: bool,
    /// The locations the lookup passed through, from the last segment of the
    /// requested path to the definition.
    pub chain: Vec<String>,
}

/// Why a path did not resolve.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResolveError {
    /// Nothing the workspace defines has this path. A dependency item, a macro
    /// expansion and a misspelling all end here.
    NotFound(String),
    /// The path names more than one symbol: two globs that bring in different
    /// items, two imports of one name, two canonical targets, items of two
    /// targets at one path, a crate path that targets of several packages
    /// share, or a path through a crate name that stands for several
    /// libraries (the ids are then the crate roots).
    Ambiguous(Vec<ItemId>),
    /// The imports that the path goes through depend on each other.
    Cyclic(String),
    /// The text is not a path.
    Invalid(String),
    /// Nothing the workspace defines has this path, and the path goes through
    /// the self type of an inherent impl that the index cannot see (see
    /// [`SelfType::Unresolved`](crate::SelfType::Unresolved)), which may define
    /// it.
    UnresolvedSelf(String),
    /// The package named to qualify the path is not a package of the
    /// workspace.
    UnknownPackage(String),
    /// A lookup on the way of the path reads the names a glob import brings
    /// in, and the glob's path names nothing in the workspace past its first
    /// segment (`use self::missing::*;`, `use crate::a::gone::*;`). rustc
    /// refuses such a glob, so what the lookup would find is not known. The
    /// load does not fail for it: a module that a macro generates is globbed
    /// this way. A glob whose first segment names nothing in the workspace,
    /// as a dependency does, is not one: the index cannot look into it.
    UnresolvedGlob {
        /// The path that was resolved.
        path: String,
        /// The written path of the glob, without its `::*`.
        glob: String,
        /// The path of the module that declares the glob.
        module: String,
    },
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(path) => write!(f, "`{}` names nothing in the workspace", Esc(path)),
            Self::Ambiguous(ids) => write!(f, "the path names {} different symbols", ids.len()),
            Self::Cyclic(path) => {
                write!(f, "`{}` goes through imports that form a cycle", Esc(path))
            }
            Self::Invalid(text) => write!(f, "`{}` is not a path", Esc(text)),
            Self::UnresolvedSelf(path) => write!(
                f,
                "`{}` goes through the self type of an inherent impl that the index cannot see",
                Esc(path)
            ),
            Self::UnknownPackage(package) => {
                write!(f, "`{}` is not a package of the workspace", Esc(package))
            }
            Self::UnresolvedGlob { path, glob, module } => write!(
                f,
                "`{}` reads the glob `{}::*` of module `{}`, whose path names nothing in the \
                 workspace",
                Esc(path),
                Esc(glob),
                Esc(module)
            ),
        }
    }
}

impl Error for ResolveError {}

/// The places a walk read, once each by module and name, in order.
pub(crate) fn places(reads: Vec<Source>) -> BTreeSet<(usize, String)> {
    reads
        .into_iter()
        .map(|source| (source.module, source.name))
        .collect()
}

/// Add the place `name` in `module` in `ns` to the places a walk read, when
/// they are noted.
fn note(reads: &mut Option<&mut Vec<Source>>, module: usize, ns: Namespace, name: &str) {
    if let Some(list) = reads.as_deref_mut() {
        list.push(Source {
            module,
            ns,
            name: name.to_owned(),
        });
    }
}

fn strip_raw(segment: &str) -> &str {
    segment.strip_prefix("r#").map_or(segment, |plain| plain)
}

impl Index {
    /// Refuse a step of a walk that would go on in the crate roots of several
    /// crates at once, with the segments from `rest` on.
    pub(crate) fn one_crate(&self, modules: &BTreeSet<usize>, rest: usize) -> Result<(), Fail> {
        let roots: Vec<usize> = modules
            .iter()
            .copied()
            .filter(|module| self.modules[*module].parent.is_none())
            .collect();
        if roots.len() > 1 {
            return Err(Fail::SeveralCrates {
                roots: self.sorted(
                    roots
                        .iter()
                        .filter_map(|root| self.root_items.get(root).copied()),
                ),
                rest,
            });
        }
        Ok(())
    }

    /// Add to `into` the crate roots of the workspace that the crate name
    /// `name` stands for in code of `module` (see [`Heads`]): a library of the
    /// extern prelude of the module's target, or what an `extern crate` item
    /// at the crate root names, which adds its name to that prelude. Returns
    /// whether it added any.
    fn extern_roots(
        &self,
        module: usize,
        name: &str,
        heads: Heads,
        into: &mut BTreeSet<usize>,
    ) -> bool {
        let (Heads::Code { test } | Heads::Use { test, .. }) = heads else {
            return false;
        };
        let before = into.len();
        if let Some(entry) = self.extern_name(module, name, test) {
            into.extend(entry.roots(test));
        }
        let root = self.modules[module].root;
        if let Some(entry) = self.scopes[root].get(name) {
            for id in entry.defs[slot(Namespace::Type)].keys() {
                if self.items[id.0].kind == SymbolKind::ExternCrate {
                    into.extend(self.modules_for(*id));
                }
            }
        }
        into.len() > before
    }

    /// True when a path read under `heads` from `from` starts at the crate
    /// root: in edition 2015, a `use` path whose first segment is not a
    /// keyword, and any path written with a leading `::`.
    pub(crate) fn crate_relative(
        &self,
        from: usize,
        first: &str,
        leading_colon: bool,
        heads: Heads,
    ) -> bool {
        let keyword = matches!(first, "crate" | "self" | "super");
        match heads {
            Heads::Local => false,
            Heads::Code { .. } => leading_colon && self.edition(from) == 2015,
            Heads::Use { .. } => self.edition(from) == 2015 && (leading_colon || !keyword),
        }
    }

    /// True when `name` is a crate of the extern prelude that a `use` path in
    /// `module` can start with: `core`, `std` unless the crate has
    /// `#![no_std]`, a crate of its target's extern prelude for its test code
    /// (`test`) or other code, or an `extern crate` item at the crate root.
    pub(crate) fn in_use_prelude(&self, module: usize, name: &str, test: bool) -> bool {
        name == "core"
            || (name == "std" && !self.no_std(module))
            || self.extern_name(module, name, test).is_some()
            || self.root_extern_crate(module, name)
    }

    /// The crate outside the index that the path of the named import at
    /// `position` starts at, if it starts at one: from edition 2018 on, a
    /// first segment its module does not bind that is a crate of the extern
    /// prelude standing for no library of the workspace, or one with a
    /// leading `::`; in edition 2015, the crate an `extern crate` item at the
    /// crate root names, or the crate the compiler puts there.
    pub(crate) fn outside_crate(&self, position: usize) -> Option<String> {
        let decl = &self.uses[position];
        let UseKind::Named { segs, .. } = &decl.kind else {
            return None;
        };
        let head = segs.first()?;
        if matches!(head.as_str(), "crate" | "self" | "super") {
            return None;
        }
        let heads = Heads::Use {
            test: decl.test,
            own: Some(position),
        };
        if self.crate_relative(decl.module, head, decl.leading_colon, heads) {
            let root = self.modules[decl.module].root;
            return match self.lookup(root, Namespace::Type, head) {
                // An `extern crate` item of a crate outside the index.
                Some((bound, _)) => match bound.keys().collect::<Vec<_>>().as_slice() {
                    [id] if self.modules_for(**id).next().is_none() => {
                        self.extern_names.get(*id).map(ToString::to_string)
                    }
                    _ => None,
                },
                None => (head == self.injected_crate(decl.module)).then(|| head.clone()),
            };
        }
        let unbound = decl.leading_colon
            || (self
                .lookup_in(decl.module, Namespace::Type, head, Some(position))
                .is_none()
                && !self.blocked(decl.module, head, Namespace::Type, Some(position)));
        let mut roots = BTreeSet::new();
        (unbound
            && self.in_use_prelude(decl.module, head, decl.test)
            && !self.extern_roots(decl.module, head, heads, &mut roots))
        .then(|| head.clone())
    }

    /// The aliases of another crate that the globs of `from` bring in under
    /// `name`: named imports whose path is a single segment, a crate outside
    /// the index other than `name` (`pub use std as serde;`), that bind
    /// `name` in a module the globs reach, directly or through the globs of
    /// that module, and are visible from `from`. An import of an item of a
    /// crate outside the index is not one: the index cannot tell its name
    /// space, and only the type name space collides with a crate (`use
    /// quote::quote;` brings in a macro).
    pub(crate) fn globbed_outside(&self, from: usize, name: &str) -> Vec<usize> {
        let mut seen = BTreeSet::from([from]);
        let mut work = vec![from];
        let mut found = Vec::new();
        while let Some(module) = work.pop() {
            for glob in self.globs_of(module) {
                let Ok(targets) = self.glob_targets(*glob) else {
                    continue;
                };
                for target in targets {
                    if !seen.insert(target) {
                        continue;
                    }
                    work.push(target);
                    for position in self.imports_of(target, name) {
                        let import = &self.uses[*position];
                        let crate_alias =
                            matches!(&import.kind, UseKind::Named { segs, .. } if segs.len() == 1);
                        if crate_alias
                            && !self.use_resolved[*position]
                            && self.is_visible(&import.vis, target, from)
                            && self
                                .outside_crate(*position)
                                .is_some_and(|krate| krate != name)
                        {
                            found.push(*position);
                        }
                    }
                }
            }
        }
        found.sort_unstable();
        found
    }

    /// True when the crate root of `module` binds `name` to an `extern crate`
    /// item.
    fn root_extern_crate(&self, module: usize, name: &str) -> bool {
        let root = self.modules[module].root;
        self.scopes[root].get(name).is_some_and(|entry| {
            entry.defs[slot(Namespace::Type)]
                .keys()
                .any(|id| self.items[id.0].kind == SymbolKind::ExternCrate)
        })
    }

    /// The ambiguity of a `use` path whose first segment `name`, written in
    /// `from`, a glob of the module binds to `bound` in the type name space:
    /// the items, with the crate roots that the crate name stands for, when
    /// the name is also a crate of the extern prelude and the glob brings in
    /// something other than that crate.
    pub(crate) fn glob_against_prelude(
        &self,
        from: usize,
        name: &str,
        test: bool,
        bound: &crate::settle::Bound,
    ) -> Option<Vec<ItemId>> {
        if self.edition(from) == 2015 || !self.in_use_prelude(from, name, test) {
            return None;
        }
        let mut roots = BTreeSet::new();
        self.extern_roots(from, name, Heads::Code { test }, &mut roots);
        let names_the_crate = bound.keys().all(|id| {
            let item = &self.items[id.0];
            match item.kind {
                SymbolKind::ExternCrate => {
                    let stands_for: BTreeSet<usize> = self.modules_for(*id).collect();
                    if stands_for.is_empty() {
                        self.extern_names
                            .get(id)
                            .is_some_and(|real| real.as_ref() == name)
                    } else {
                        stands_for == roots
                    }
                }
                SymbolKind::Mod => self
                    .module_of
                    .get(id)
                    .is_some_and(|module| roots.contains(module)),
                _ => false,
            }
        });
        if names_the_crate {
            return None;
        }
        let ids = bound.keys().copied().chain(
            roots
                .iter()
                .filter_map(|root| self.root_items.get(root).copied()),
        );
        Some(self.sorted(ids))
    }

    /// Refuse, or hold back, a path whose first segment `first`, which `from`
    /// does not bind in `ns`, would start at a crate of the extern prelude.
    /// An explicit import of the name binds it before a crate does, in every
    /// name space where it keeps the name from the globs (see
    /// [`Index::blocked`]), whether or not the index can see what it binds:
    /// there the path never starts at a crate and waits on the name. For a `use`
    /// path, a glob of the module may still bring the name in, and a name a
    /// glob brings in that is also a crate is ambiguous, so until the globs
    /// are settled the path waits on the name. An explicit import's first
    /// segment that a glob brings in from an import outside the index, which
    /// the index does not bind, is ambiguous with the crate too (a glob's own
    /// path is not checked: the check reads the globs of the module).
    fn unbound_use_head(
        &self,
        from: usize,
        ns: Namespace,
        first: &str,
        heads: Heads,
    ) -> Result<(), Fail> {
        let wait = || {
            Err(Fail::Missing(vec![Source {
                module: from,
                ns,
                name: first.to_owned(),
            }]))
        };
        // An explicit import shadows a crate in the name spaces where it
        // keeps the name from the globs, and a glob may still bring the name
        // in: the path waits on it.
        if self.blocked(from, first, ns, heads.own()) {
            return wait();
        }
        // A glob of the module that may still bring the name in from a
        // module where an import keeps it holds back a crate as well, as the
        // two would be ambiguous. A glob whose path is not settled does not:
        // until the globs have brought in what they can no crate is read, and
        // a glob that brings the name in later makes the path ambiguous,
        // which the settled tables are checked for (see
        // [`Index::settle_agreed`]).
        let pending = self.pending_at(from, first, slot(ns), None, heads.own());
        let rest = || if pending { wait() } else { Ok(()) };
        let Heads::Use { test, own } = heads else {
            return rest();
        };
        if !self.crates_open && !self.globs_of(from).is_empty() {
            return wait();
        }
        // A glob's own path is not checked: the check reads the globs of the
        // module.
        if !own.is_some_and(|position| matches!(self.uses[position].kind, UseKind::Named { .. }))
            || ns != Namespace::Type
            || self.edition(from) == 2015
            || !self.in_use_prelude(from, first, test)
        {
            return rest();
        }
        // An alias of another crate that a glob carries collides with the
        // crate, though what it binds is never settled in the index.
        let carried = self.globbed_outside(from, first);
        if carried.is_empty() {
            return rest();
        }
        let mut roots = BTreeSet::new();
        self.extern_roots(from, first, heads, &mut roots);
        let ids = carried
            .iter()
            .flat_map(|position| self.module_items(self.uses[*position].module))
            .chain(
                roots
                    .iter()
                    .filter_map(|root| self.root_items.get(root).copied()),
            );
        Err(Fail::Ambiguous(self.sorted(ids)))
    }

    /// The state after the first segment of a path, and the index of the
    /// segment to read next.
    fn start(
        &self,
        from: usize,
        segs: &[String],
        leading_colon: bool,
        last_ns: Namespace,
        heads: Heads,
        reads: &mut Option<&mut Vec<Source>>,
    ) -> Result<(Walked, usize), Fail> {
        let mut state = Walked::default();
        let Some(first) = segs.first() else {
            return Err(Fail::Missing(Vec::new()));
        };
        if self.crate_relative(from, first, leading_colon, heads) {
            // The walk reads the first segment in the crate root's scope,
            // which holds its items, its imports and its `extern crate` items.
            state.modules.insert(self.modules[from].root);
            return Ok((state, 0));
        }
        let mut next_index = 1;
        if leading_colon {
            if !self.extern_roots(from, first, heads, &mut state.modules) {
                return Err(Fail::Missing(Vec::new()));
            }
        } else {
            match first.as_str() {
                "crate" => {
                    state.modules.insert(self.modules[from].root);
                }
                "self" => {
                    state.modules.insert(from);
                }
                "super" => {
                    let mut module = from;
                    let mut at = 0;
                    while segs.get(at).is_some_and(|s| s == "super") {
                        module = self.modules[module].parent.ok_or(Fail::SuperAboveRoot)?;
                        at += 1;
                    }
                    state.modules.insert(module);
                    next_index = at;
                }
                _ => {
                    let ns = if segs.len() == 1 {
                        last_ns
                    } else {
                        Namespace::Type
                    };
                    note(reads, from, ns, first);
                    let lookup = self.lookup_in(from, ns, first, heads.own());
                    if lookup.is_none() {
                        self.unbound_use_head(from, ns, first, heads)?;
                    }
                    if let Some((bound, from_glob)) = lookup {
                        if let (true, Namespace::Type, Heads::Use { test, .. }) =
                            (from_glob, ns, heads)
                            && let Some(ids) = self.glob_against_prelude(from, first, test, &bound)
                        {
                            return Err(Fail::Ambiguous(ids));
                        }
                        for (id, origin) in bound.iter() {
                            merge(
                                &mut state.items,
                                *id,
                                Hit {
                                    via_glob: from_glob || origin.via_glob,
                                    last: Some(Source {
                                        module: from,
                                        ns,
                                        name: first.clone(),
                                    }),
                                },
                            );
                        }
                    } else if !self.extern_roots(from, first, heads, &mut state.modules) {
                        return Err(Fail::Missing(vec![Source {
                            module: from,
                            ns,
                            name: first.clone(),
                        }]));
                    }
                }
            }
        }
        Ok((state, next_index))
    }

    /// Walk `segs` from `from`, in the name space `last_ns` for the last
    /// segment and the type name space before it.
    pub(crate) fn walk(
        &self,
        from: usize,
        segs: &[String],
        leading_colon: bool,
        last_ns: Namespace,
        heads: Heads,
    ) -> Result<Walked, Fail> {
        self.walk_noting(from, segs, leading_colon, last_ns, heads, None)
    }

    /// [`Index::walk`], adding to `reads` every place whose binding the walk
    /// looked up, found or not: what its answer depends on while the tables
    /// settle.
    pub(crate) fn walk_noting(
        &self,
        from: usize,
        segs: &[String],
        leading_colon: bool,
        last_ns: Namespace,
        heads: Heads,
        mut reads: Option<&mut Vec<Source>>,
    ) -> Result<Walked, Fail> {
        let (mut state, next_index) =
            self.start(from, segs, leading_colon, last_ns, heads, &mut reads)?;
        self.one_crate(&state.modules, next_index)?;
        // A `use` path reads each module as rustc resolves an import: only
        // what the importing module can access counts.
        let viewer = matches!(heads, Heads::Use { .. }).then_some(from);
        for (offset, seg) in segs.iter().enumerate().skip(next_index) {
            let last = offset + 1 == segs.len();
            let ns = if last { last_ns } else { Namespace::Type };
            if seg == "super" && state.items.is_empty() {
                let mut parents = BTreeSet::new();
                for module in &state.modules {
                    parents.insert(self.modules[*module].parent.ok_or(Fail::SuperAboveRoot)?);
                }
                state.modules = parents;
                continue;
            }
            let mut containers = state.modules.clone();
            let carried = state.items.values().any(|hit| hit.via_glob);
            for id in state.items.keys() {
                containers.extend(self.modules_for(*id));
            }
            self.one_crate(&containers, offset)?;
            let mut found: BTreeMap<ItemId, Hit> = BTreeMap::new();
            let mut missing = Vec::new();
            for module in containers {
                note(&mut reads, module, ns, seg);
                match self.lookup_seen(module, ns, seg, heads.own(), viewer) {
                    Some((bound, from_glob)) => {
                        for (id, origin) in bound.iter() {
                            merge(
                                &mut found,
                                *id,
                                Hit {
                                    via_glob: carried || from_glob || origin.via_glob,
                                    last: Some(Source {
                                        module,
                                        ns,
                                        name: seg.clone(),
                                    }),
                                },
                            );
                        }
                    }
                    None => missing.push(Source {
                        module,
                        ns,
                        name: seg.clone(),
                    }),
                }
            }
            for (id, hit) in &state.items {
                if let Some(members) = self.assoc.get(id).and_then(|m| m.get(seg.as_str())) {
                    for member in members {
                        if self.items[member.0].in_namespace(ns) {
                            merge(
                                &mut found,
                                *member,
                                Hit {
                                    via_glob: hit.via_glob,
                                    last: None,
                                },
                            );
                        }
                    }
                }
            }
            if found.is_empty() {
                return Err(Fail::Missing(missing));
            }
            state = Walked {
                modules: BTreeSet::new(),
                items: found,
            };
        }
        Ok(state)
    }

    /// The named imports that bind `name` in `module` and are not resolved
    /// (`resolved_mode`) or not settled yet.
    pub(crate) fn unresolved_uses(
        &self,
        module: usize,
        name: &str,
        resolved_mode: bool,
    ) -> Vec<usize> {
        self.imports_of(module, name)
            .iter()
            .copied()
            .filter(|position| {
                !(if resolved_mode {
                    self.use_resolved[*position]
                } else {
                    self.use_settled[*position]
                })
            })
            .collect()
    }

    /// The walks of the path of the named import at `position`, in every name
    /// space, read as a `use` path that does not consult its own binding: the
    /// other imports it waits on (unresolved ones in `resolved_mode`, else
    /// unsettled ones), which bind a name the walks looked up, the items of
    /// an ambiguous first segment, and the places the walks read.
    fn import_walks(
        &self,
        position: usize,
        resolved_mode: bool,
    ) -> (Vec<usize>, Vec<ItemId>, Vec<Source>) {
        let decl = &self.uses[position];
        let UseKind::Named { segs, .. } = &decl.kind else {
            return (Vec::new(), Vec::new(), Vec::new());
        };
        let mut reads = Vec::new();
        let mut ambiguous = BTreeSet::new();
        for ns in Namespace::ALL {
            if let Err(Fail::Ambiguous(ids)) = self.walk_noting(
                decl.module,
                segs,
                decl.leading_colon,
                ns,
                Heads::Use {
                    test: decl.test,
                    own: Some(position),
                },
                Some(&mut reads),
            ) {
                ambiguous.extend(ids);
            }
        }
        let mut blockers = BTreeSet::new();
        for (module, name) in places(reads.clone()) {
            // The path never reads the imports of the name it binds.
            if !self.binds_name(position, module, &name) {
                blockers.extend(self.unresolved_uses(module, &name, resolved_mode));
            }
        }
        // The import's own binding is never a way its path goes.
        blockers.remove(&position);
        (
            blockers.into_iter().collect(),
            ambiguous.into_iter().collect(),
            reads,
        )
    }

    /// True when what is bound at `source` was bound by a named import that
    /// never settled: one that bound its name in this name space and is left
    /// undecided in another, by a cycle there, or one that a cycle left to
    /// the globs there and that never decided (see [`Index::left_to_globs`]).
    /// rustc reports such an import as an error, that it cannot determine
    /// its resolution, whatever it bound, so a path through it does not
    /// resolve.
    fn bound_by_undecided(&self, source: &Source) -> bool {
        let at = slot(source.ns);
        let bound = self.scopes[source.module]
            .get(&source.name)
            .is_some_and(|entry| !entry.defs[at].is_empty());
        self.imports_of(source.module, &source.name)
            .iter()
            .any(|position| {
                !self.use_settled[*position]
                    && (bound && self.use_bound[*position][at] || self.left_to_globs(*position, at))
            })
    }

    /// Why nothing is bound at `missing`, the places a walk found unbound:
    /// `Cyclic` when an import that binds one of them never settled, which
    /// the settling of the tables leaves only to the imports of a cycle and
    /// those that wait on one, or a glob may still bring the name in from a
    /// module where such an import keeps it (see [`Index::waits_on_cycle`]);
    /// else
    /// `Ambiguous` naming the items of every ambiguous first segment among
    /// the unresolved imports that bind them, or that those wait on; else
    /// `UnresolvedGlob` when a lookup of the walk, at `reads`, or of the paths
    /// of those imports reads a glob whose path names nothing past its first
    /// segment; else `NotFound`.
    fn unbound(&self, missing: &[Source], reads: &[Source], text: &str) -> ResolveError {
        let cyclic = missing.iter().any(|source| self.waits_on_cycle(source));
        if cyclic {
            return ResolveError::Cyclic(text.to_owned());
        }
        // The imports on the way wait on no cycle, so the search ends; it
        // keeps a list of its own and visits each import once.
        let mut seen: BTreeSet<usize> = BTreeSet::new();
        let mut work: Vec<usize> = Vec::new();
        for source in missing {
            work.extend(self.unresolved_uses(source.module, &source.name, true));
        }
        let mut ambiguous: BTreeSet<ItemId> = BTreeSet::new();
        let mut through: Vec<Source> = reads.to_vec();
        while let Some(position) = work.pop() {
            if !seen.insert(position) {
                continue;
            }
            let (following, items, reads) = self.import_walks(position, true);
            ambiguous.extend(items);
            work.extend(following);
            through.extend(reads);
        }
        if !ambiguous.is_empty() {
            return ResolveError::Ambiguous(self.sorted(ambiguous));
        }
        match self.unresolved_glob_read(&through) {
            Some(glob) => self.glob_error(glob, text),
            None => ResolveError::NotFound(text.to_owned()),
        }
    }

    /// The error of a path `text` whose lookups read the glob import at
    /// `position`, whose path names nothing past its first segment.
    pub(crate) fn glob_error(&self, position: usize, text: &str) -> ResolveError {
        let decl = &self.uses[position];
        let (UseKind::Glob { segs } | UseKind::Named { segs, .. }) = &decl.kind;
        ResolveError::UnresolvedGlob {
            path: text.to_owned(),
            glob: format!(
                "{}{}",
                if decl.leading_colon { "::" } else { "" },
                segs.join("::")
            ),
            module: self.modules[decl.module].path.to_string(),
        }
    }

    /// True when a lookup of `name` in `ns` in `module` finds an explicit
    /// binding there, or an explicit import keeps the name from the globs
    /// (see [`Index::blocked`]): the globs of the module are not read.
    fn binds_explicitly(&self, module: usize, ns: Namespace, name: &str) -> bool {
        self.scopes[module]
            .get(name)
            .is_some_and(|entry| !entry.defs[slot(ns)].is_empty())
            || self.blocked(module, name, ns, None)
    }

    /// The glob imports that a lookup of `name` in `ns` in `module` reads:
    /// those of the module when nothing explicit binds the name there, and,
    /// through each glob whose path resolves (`reach`), those of the modules
    /// it globs in turn, each module once.
    fn globs_read(
        &self,
        reach: &BTreeMap<usize, BTreeSet<usize>>,
        module: usize,
        ns: Namespace,
        name: &str,
    ) -> BTreeSet<usize> {
        let mut seen = BTreeSet::from([module]);
        let mut work = vec![module];
        let mut out = BTreeSet::new();
        while let Some(at) = work.pop() {
            if self.binds_explicitly(at, ns, name) {
                continue;
            }
            for glob in self.globs_of(at) {
                out.insert(*glob);
                for target in reach.get(glob).into_iter().flatten() {
                    if seen.insert(*target) {
                        work.push(*target);
                    }
                }
            }
        }
        out
    }

    /// What the path of the glob import at `position` names in the settled
    /// tables.
    fn glob_path(&self, position: usize) -> GlobPath {
        let decl = &self.uses[position];
        let UseKind::Glob { segs } = &decl.kind else {
            return GlobPath::Nothing;
        };
        let sources = match self.glob_targets(position) {
            Ok(targets) => return GlobPath::Resolves(targets),
            Err(Fail::Missing(sources)) => sources,
            Err(Fail::SuperAboveRoot | Fail::SeveralCrates { .. } | Fail::Ambiguous(_)) => {
                return GlobPath::Nothing;
            }
        };
        let head = self.walk(
            decl.module,
            &segs[..1],
            decl.leading_colon,
            Namespace::Type,
            Heads::Use {
                test: decl.test,
                own: Some(position),
            },
        );
        // A first segment that names nothing in the workspace, or a path
        // that goes on in a crate outside it, is a dependency.
        if head.is_err() || sources.is_empty() {
            GlobPath::Unknown
        } else {
            GlobPath::Misses(sources)
        }
    }

    /// The globs of the settled tables whose paths resolve, and those whose
    /// paths name nothing past their first segment, computed once for the
    /// index (see [`ResolveError::UnresolvedGlob`]).
    ///
    /// A glob whose path misses is not reported when what its path could
    /// have reached is not known to be missing: a place where the path
    /// names nothing that an import binds, which may be an import the index
    /// cannot see through, or that reads a glob whose path is unknown, a
    /// dependency glob or one that is not reported in turn.
    fn glob_facts(&self) -> &GlobFacts {
        self.glob_facts.get_or_init(|| {
            let mut reach: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
            let mut misses: BTreeMap<usize, Vec<Source>> = BTreeMap::new();
            let mut unknown: Vec<usize> = Vec::new();
            for &glob in &self.glob_imports {
                match self.glob_path(glob) {
                    GlobPath::Resolves(targets) => {
                        reach.insert(glob, targets);
                    }
                    GlobPath::Misses(sources) => {
                        misses.insert(glob, sources);
                    }
                    GlobPath::Unknown => unknown.push(glob),
                    GlobPath::Nothing => {}
                }
            }
            // Which missing globs each glob would make unknown: those that
            // miss at a place that reads it.
            let mut readers: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
            for (glob, sources) in &misses {
                for source in sources {
                    if !self.imports_of(source.module, &source.name).is_empty() {
                        unknown.push(*glob);
                    }
                    for other in self.globs_read(&reach, source.module, source.ns, &source.name) {
                        if other != *glob {
                            readers.entry(other).or_default().push(*glob);
                        }
                    }
                }
            }
            let mut opaque: BTreeSet<usize> = BTreeSet::new();
            while let Some(glob) = unknown.pop() {
                if opaque.insert(glob) {
                    unknown.extend(readers.get(&glob).into_iter().flatten().copied());
                }
            }
            let broken = misses
                .into_keys()
                .filter(|glob| !opaque.contains(glob))
                .collect();
            GlobFacts { reach, broken }
        })
    }

    /// The failure that [`Index::resolve`] reports for a path `text` whose
    /// last segment reads `name` in `module` in `ns`, where the module binds
    /// it to `bound` (what [`Index::lookup_in`] answers there), as far as
    /// that last place decides it: `Cyclic` when an import that never
    /// settled bound it there, `Ambiguous` when it binds several symbols
    /// there, and `UnresolvedGlob` when the lookup reads a glob whose path
    /// names nothing past its first segment. The places before the last are
    /// the caller's to have checked, each as the last place of its own
    /// prefix, so a path is checked once for each of its segments however
    /// long the chain of imports behind each one.
    pub(crate) fn binding_failure(
        &self,
        module: usize,
        ns: Namespace,
        name: &str,
        bound: &[ItemId],
        text: &str,
    ) -> Option<ResolveError> {
        let source = Source {
            module,
            ns,
            name: name.to_owned(),
        };
        if self.bound_by_undecided(&source) {
            return Some(ResolveError::Cyclic(text.to_owned()));
        }
        let groups: BTreeSet<(TargetId, &str, SymbolKind)> = bound
            .iter()
            .map(|id| {
                let item = &self.items[id.0];
                (item.target, item.path.as_str(), item.kind)
            })
            .collect();
        if groups.len() > 1 {
            return Some(ResolveError::Ambiguous(self.sorted(bound.iter().copied())));
        }
        self.unresolved_glob_read(std::slice::from_ref(&source))
            .map(|glob| self.glob_error(glob, text))
    }

    /// True when the path of the glob import at `position` names nothing in
    /// the workspace past its first segment, so a lookup that reads it fails
    /// with [`ResolveError::UnresolvedGlob`].
    pub(crate) fn glob_broken(&self, position: usize) -> bool {
        self.glob_facts().broken.contains(&position)
    }

    /// The first of the glob imports a walk read, at the places `reads`,
    /// whose path names nothing in the workspace past its first segment.
    fn unresolved_glob_read(&self, reads: &[Source]) -> Option<usize> {
        let facts = self.glob_facts();
        if facts.broken.is_empty() {
            return None;
        }
        let places: BTreeSet<(usize, Namespace, &str)> = reads
            .iter()
            .map(|source| (source.module, source.ns, source.name.as_str()))
            .collect();
        places
            .into_iter()
            .flat_map(|(module, ns, name)| self.globs_read(&facts.reach, module, ns, name))
            .filter(|glob| facts.broken.contains(glob))
            .min()
    }

    fn origin_of(&self, source: &Source, id: ItemId) -> Option<crate::settle::Origin> {
        let ns = slot(source.ns);
        let defs = self.scopes[source.module]
            .get(&source.name)
            .and_then(|entry| entry.defs[ns].get(id));
        match defs {
            Some(origin) => Some(origin.clone()),
            None => self
                .globbed(source.module, &source.name, ns)
                .get(id)
                .cloned(),
        }
    }

    fn chain_for(&self, requested: &str, hit: &Hit, id: ItemId) -> Vec<String> {
        let mut chain = Vec::new();
        let Some(mut at) = hit.last.clone() else {
            chain.push(requested.to_owned());
            return chain;
        };
        // The recorded origins lead back to a definition; the guard is for a
        // route that was replaced by a better one while the tables settled.
        let mut visited: BTreeSet<Source> = BTreeSet::new();
        while visited.insert(at.clone()) {
            chain.push(format!("{}::{}", self.modules[at.module].path, at.name));
            let next = self
                .origin_of(&at, id)
                .and_then(|origin| origin.link)
                .and_then(|link| link.source);
            match next {
                Some(source) => at = source,
                None => break,
            }
        }
        chain
    }

    /// The package of the target whose crate `module` is in.
    fn package_of(&self, module: usize) -> &str {
        &self.targets[self.modules[module].target.index()].package
    }

    /// Walk `rest`, the segments of a canonical path `text` after the crate
    /// path of `root`, in `ns`, adding to `walks` what it reaches, the places
    /// it found unbound and the places it read.
    fn walk_from_root(
        &self,
        root: usize,
        rest: &[String],
        ns: Namespace,
        text: &str,
        walks: &mut Walks,
    ) -> Result<(), ResolveError> {
        let start = walks.reads.len();
        let walk = self.walk_noting(root, rest, false, ns, Heads::Local, Some(&mut walks.reads));
        if walk.is_ok()
            && walks.reads[start..]
                .iter()
                .any(|source| self.bound_by_undecided(source))
        {
            return Err(ResolveError::Cyclic(text.to_owned()));
        }
        match walk {
            Ok(walked) => {
                for (id, hit) in walked.items {
                    merge(&mut walks.found, id, hit);
                }
            }
            Err(Fail::Missing(sources)) => walks.missing.extend(sources),
            Err(Fail::SeveralCrates { roots: ids, .. } | Fail::Ambiguous(ids)) => {
                return Err(ResolveError::Ambiguous(ids));
            }
            // A canonical path holds no `super` (see [`Index::resolve`]), so
            // no walk of one climbs above a crate root; if one did, the path
            // is not a path.
            Err(Fail::SuperAboveRoot) => {
                return Err(ResolveError::Invalid(text.to_owned()));
            }
        }
        Ok(())
    }

    /// Resolve a canonical path in one name space, among the crates of
    /// `package` when one is given.
    fn resolve_in(
        &self,
        segs: &[String],
        ns: Namespace,
        text: &str,
        package: Option<&str>,
    ) -> Result<Resolution, ResolveError> {
        let mut walks = Walks::default();
        // The prefix grows by one segment a step, so the prefixes of a path
        // cost its length, not its length squared.
        let mut prefix = String::with_capacity(text.len());
        for (taken, seg) in (1..=segs.len()).zip(segs) {
            if taken > 1 {
                prefix.push_str("::");
            }
            prefix.push_str(seg);
            let Some(roots) = self.crate_roots.get(&prefix) else {
                continue;
            };
            let roots: Vec<usize> = roots
                .iter()
                .copied()
                .filter(|root| package.is_none_or(|name| self.package_of(*root) == name))
                .collect();
            // A crate path that targets of several packages have (two
            // libraries with one crate name) names each of their crates; the
            // path is told apart only with its package.
            if roots.len() > 1 {
                return Err(ResolveError::Ambiguous(
                    self.sorted(
                        roots
                            .iter()
                            .filter_map(|root| self.root_items.get(root).copied()),
                    ),
                ));
            }
            for root in roots {
                if taken < segs.len() {
                    self.walk_from_root(root, &segs[taken..], ns, text, &mut walks)?;
                } else if ns == Namespace::Type
                    && let Some(item) = self.root_items.get(&root)
                {
                    // A path that is exactly a crate path names the crate
                    // root, a module.
                    merge(
                        &mut walks.found,
                        *item,
                        Hit {
                            via_glob: false,
                            last: None,
                        },
                    );
                }
            }
        }
        let Walks {
            mut found,
            missing,
            reads,
        } = walks;
        let from_keys = found.is_empty();
        if let (true, Some(ids)) = (from_keys, self.foreign.get(text)) {
            for id in ids {
                let item = &self.items[id.0];
                if item.in_namespace(ns)
                    && package.is_none_or(|name| self.targets[item.target.index()].package == name)
                {
                    found.insert(
                        *id,
                        Hit {
                            via_glob: false,
                            last: None,
                        },
                    );
                }
            }
        }
        if found.is_empty() {
            return Err(self.unbound(&missing, &reads, text));
        }
        // One symbol is one path and kind in one crate. Items of two targets
        // at one path are two symbols, whatever order they were found in; the
        // members of trait impls on types that are not local are keyed by
        // the trait, so the impls of every crate share their key.
        let mut groups: BTreeSet<(Option<TargetId>, &str, SymbolKind)> = BTreeSet::new();
        for id in found.keys() {
            let item = &self.items[id.0];
            let target = (!from_keys).then_some(item.target);
            groups.insert((target, item.path.as_str(), item.kind));
        }
        if groups.len() > 1 {
            return Err(ResolveError::Ambiguous(self.sorted(found.keys().copied())));
        }
        // A lookup on the way that read a glob whose path names nothing past
        // its first segment may have missed what the glob would bring in.
        if let Some(glob) = self.unresolved_glob_read(&reads) {
            return Err(self.glob_error(glob, text));
        }
        let sites = self.sorted(found.keys().copied());
        let first = sites[0];
        let hit = &found[&first];
        Ok(Resolution {
            path: self.items[first.0].path.to_string(),
            namespace: ns,
            via_glob: hit.via_glob,
            chain: self.chain_for(text, hit, first),
            sites,
        })
    }

    /// True when a proper prefix of `segs` is the self path of an inherent
    /// impl that the index cannot see, of `package` when one is given.
    fn through_unresolved(&self, segs: &[String], package: Option<&str>) -> bool {
        // The prefix grows by one segment a step, as in [`Index::resolve_in`].
        let mut prefix = String::new();
        segs.iter().take(segs.len().saturating_sub(1)).any(|seg| {
            if !prefix.is_empty() {
                prefix.push_str("::");
            }
            prefix.push_str(seg);
            let start = self
                .unresolved
                .partition_point(|block| block.path.as_str() < prefix.as_str());
            self.unresolved[start..]
                .iter()
                .take_while(|block| block.path == prefix)
                .any(|block| {
                    package.is_none_or(|name| self.targets[block.target.index()].package == name)
                })
        })
    }

    /// Resolve `path`, trying the type, value and macro name spaces in turn,
    /// among the crates of `package` when one is given.
    pub(crate) fn resolve(
        &self,
        path: &str,
        ns: Option<Namespace>,
        package: Option<&str>,
    ) -> Result<Resolution, ResolveError> {
        if let Some(name) = package
            && !self.editions.contains_key(name)
        {
            return Err(ResolveError::UnknownPackage(name.to_owned()));
        }
        // Each segment is read once. A canonical path starts at a crate root
        // and names items by their names, so a path keyword is no segment of
        // one: `demo::crate::a::x` and `demo::super::x` are not canonical
        // paths, and neither is `r#crate`, which is no raw identifier.
        let segs: Vec<String> = path.split("::").map(|s| strip_raw(s).to_owned()).collect();
        let valid = |s: &String| {
            !s.is_empty()
                && !matches!(s.as_str(), "crate" | "self" | "super" | "Self")
                && s.chars().all(|c| c.is_alphanumeric() || c == '_')
        };
        if !segs.iter().all(valid) {
            return Err(ResolveError::Invalid(path.to_owned()));
        }
        let text = segs.join("::");
        let spaces: Vec<Namespace> = ns.map_or_else(|| Namespace::ALL.to_vec(), |one| vec![one]);
        for space in spaces {
            match self.resolve_in(&segs, space, &text, package) {
                Ok(resolution) => return Ok(resolution),
                Err(ResolveError::NotFound(_)) => {}
                Err(other) => return Err(other),
            }
        }
        if self.through_unresolved(&segs, package) {
            return Err(ResolveError::UnresolvedSelf(text));
        }
        Err(ResolveError::NotFound(text))
    }
}

#[cfg(test)]
mod tests {
    use super::ResolveError;
    use crate::item::ItemId;

    /// An escape sequence, a bidirectional override and a line break.
    const HOSTILE: &str = "a\u{1b}[2J\u{202e}\r\nb";

    /// Every variant, built so that a new one cannot be left out: the match
    /// names each.
    fn every_variant() -> Vec<ResolveError> {
        let hostile = || HOSTILE.to_owned();
        let all = vec![
            ResolveError::NotFound(hostile()),
            ResolveError::Ambiguous(vec![ItemId(0), ItemId(1)]),
            ResolveError::Cyclic(hostile()),
            ResolveError::Invalid(hostile()),
            ResolveError::UnresolvedSelf(hostile()),
            ResolveError::UnknownPackage(hostile()),
            ResolveError::UnresolvedGlob {
                path: hostile(),
                glob: hostile(),
                module: hostile(),
            },
        ];
        for error in &all {
            match error {
                ResolveError::NotFound(_)
                | ResolveError::Ambiguous(_)
                | ResolveError::Cyclic(_)
                | ResolveError::Invalid(_)
                | ResolveError::UnresolvedSelf(_)
                | ResolveError::UnknownPackage(_)
                | ResolveError::UnresolvedGlob { .. } => {}
            }
        }
        all
    }

    #[test]
    fn every_user_derived_field_of_every_resolve_error_is_escaped() {
        for error in every_variant() {
            let text = error.to_string();
            assert!(
                !text.chars().any(|c| c.is_control() || c == '\u{202e}'),
                "a control character got through: {text:?}"
            );
            let fields = match &error {
                ResolveError::Ambiguous(_) => 0,
                ResolveError::UnresolvedGlob { .. } => 3,
                _ => 1,
            };
            assert_eq!(
                text.matches(r"a\u{1b}[2J\u{202e}\r\nb").count(),
                fields,
                "{text}"
            );
        }
    }
}
