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
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(path) => write!(f, "`{path}` names nothing in the workspace"),
            Self::Ambiguous(ids) => write!(f, "the path names {} different symbols", ids.len()),
            Self::Cyclic(path) => write!(f, "`{path}` goes through imports that form a cycle"),
            Self::Invalid(text) => write!(f, "`{text}` is not a path"),
            Self::UnresolvedSelf(path) => write!(
                f,
                "`{path}` goes through the self type of an inherent impl that the index cannot see"
            ),
            Self::UnknownPackage(package) => {
                write!(f, "`{package}` is not a package of the workspace")
            }
        }
    }
}

impl Error for ResolveError {}

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
            || self
                .lookup_in(decl.module, Namespace::Type, head, Some(position))
                .is_none();
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
                let decl = &self.uses[*glob];
                let UseKind::Glob { segs } = &decl.kind else {
                    continue;
                };
                let Ok(targets) = self.glob_targets(decl, segs) else {
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
    /// An explicit import of the name that is still unsettled may bind it,
    /// and binds the name before a crate does. For a `use` path, a name an
    /// explicit import of the module binds once the tables are settled is
    /// never read as a crate, a glob of the module may still bring the name
    /// in, and a name
    /// a glob brings in that is also a crate is ambiguous, so until the
    /// globs are settled the path waits on the name. An explicit import's
    /// first segment that a glob brings in from an import outside the index,
    /// which the index does not bind, is ambiguous with the crate too (a
    /// glob's own path is not checked: the check reads the globs of the
    /// module).
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
        // An explicit import of the name that is still unsettled may bind it,
        // and an explicit import shadows a crate: the path waits on it.
        if self.is_unsettled(from, first, heads.own()) {
            return wait();
        }
        let Heads::Use { test, own } = heads else {
            return Ok(());
        };
        if (!self.crates_open && !self.globs_of(from).is_empty())
            || (!self.no_crate.is_empty() && self.no_crate.contains(&(from, first.to_owned())))
        {
            return wait();
        }
        if own.is_none()
            || ns != Namespace::Type
            || self.edition(from) == 2015
            || !self.in_use_prelude(from, first, test)
        {
            return Ok(());
        }
        let carried = self.globbed_outside(from, first);
        if carried.is_empty() {
            return Ok(());
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
                    let lookup = self.lookup_in(from, ns, first, heads.own());
                    if lookup.is_none() {
                        self.unbound_use_head(from, ns, first, heads)?;
                    }
                    if let Some((bound, from_glob)) = lookup {
                        if let (true, Namespace::Type, Heads::Use { test, .. }) =
                            (from_glob, ns, heads)
                            && let Some(ids) = self.glob_against_prelude(from, first, test, bound)
                        {
                            return Err(Fail::Ambiguous(ids));
                        }
                        for (id, origin) in bound {
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
        let (mut state, next_index) = self.start(from, segs, leading_colon, last_ns, heads)?;
        self.one_crate(&state.modules, next_index)?;
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
                match self.lookup_in(module, ns, seg, heads.own()) {
                    Some((bound, from_glob)) => {
                        for (id, origin) in bound {
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

    /// The named imports that bind `source`'s name in its module and are not
    /// resolved (`resolved_mode`) or not settled yet.
    fn unresolved_uses(&self, source: &Source, resolved_mode: bool) -> Vec<usize> {
        self.imports_of(source.module, &source.name)
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
    /// unsettled ones), and the items of an ambiguous first segment.
    fn import_walks(&self, position: usize, resolved_mode: bool) -> (Vec<usize>, Vec<ItemId>) {
        let decl = &self.uses[position];
        let UseKind::Named { segs, .. } = &decl.kind else {
            return (Vec::new(), Vec::new());
        };
        let mut blockers = BTreeSet::new();
        let mut ambiguous = BTreeSet::new();
        for ns in Namespace::ALL {
            match self.walk(
                decl.module,
                segs,
                decl.leading_colon,
                ns,
                Heads::Use {
                    test: decl.test,
                    own: Some(position),
                },
            ) {
                Err(Fail::Missing(sources)) => {
                    for source in sources {
                        blockers.extend(self.unresolved_uses(&source, resolved_mode));
                    }
                }
                Err(Fail::Ambiguous(ids)) => ambiguous.extend(ids),
                Ok(_) | Err(Fail::SuperAboveRoot | Fail::SeveralCrates { .. }) => {}
            }
        }
        // The import's own binding is never a way its path goes.
        blockers.remove(&position);
        (
            blockers.into_iter().collect(),
            ambiguous.into_iter().collect(),
        )
    }

    /// The imports that the import at `position` waits on.
    pub(crate) fn blockers(&self, position: usize, resolved_mode: bool) -> Vec<usize> {
        self.import_walks(position, resolved_mode).0
    }

    /// Why nothing is bound at `missing`, the places a walk found unbound:
    /// `Cyclic` when the unresolved imports that bind them, or that those wait
    /// on, contain a cycle; else `Ambiguous` naming the items of every
    /// ambiguous first segment among them; else `NotFound`.
    fn unbound(&self, missing: &[Source], text: &str) -> ResolveError {
        // Each import is on the search stack (true) or done (false).
        let mut state: BTreeMap<usize, bool> = BTreeMap::new();
        let mut ambiguous: BTreeSet<ItemId> = BTreeSet::new();
        for source in missing {
            for start in self.unresolved_uses(source, true) {
                if state.contains_key(&start) {
                    continue;
                }
                let (following, items) = self.import_walks(start, true);
                ambiguous.extend(items);
                state.insert(start, true);
                let mut stack: Vec<(usize, Vec<usize>, usize)> = vec![(start, following, 0)];
                while let Some((node, successors, at)) = stack.pop() {
                    if at < successors.len() {
                        let next = successors[at];
                        stack.push((node, successors, at + 1));
                        match state.get(&next) {
                            Some(true) => return ResolveError::Cyclic(text.to_owned()),
                            Some(false) => {}
                            None => {
                                state.insert(next, true);
                                let (following, items) = self.import_walks(next, true);
                                ambiguous.extend(items);
                                stack.push((next, following, 0));
                            }
                        }
                    } else {
                        state.insert(node, false);
                    }
                }
            }
        }
        if ambiguous.is_empty() {
            ResolveError::NotFound(text.to_owned())
        } else {
            ResolveError::Ambiguous(self.sorted(ambiguous))
        }
    }

    fn origin_of(&self, source: &Source, id: ItemId) -> Option<&crate::settle::Origin> {
        let entry = self.scopes[source.module].get(&source.name)?;
        let ns = slot(source.ns);
        entry.defs[ns].get(id).or_else(|| entry.globbed[ns].get(id))
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
                .and_then(|origin| origin.link.as_ref())
                .and_then(|link| link.source.clone());
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

    /// Resolve a canonical path in one name space, among the crates of
    /// `package` when one is given.
    fn resolve_in(
        &self,
        segs: &[String],
        ns: Namespace,
        text: &str,
        package: Option<&str>,
    ) -> Result<Resolution, ResolveError> {
        let mut found: BTreeMap<ItemId, Hit> = BTreeMap::new();
        let mut missing: Vec<Source> = Vec::new();
        for taken in 1..=segs.len() {
            let prefix = segs[..taken].join("::");
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
                    match self.walk(root, &segs[taken..], false, ns, Heads::Local) {
                        Ok(walked) => {
                            for (id, hit) in walked.items {
                                merge(&mut found, id, hit);
                            }
                        }
                        Err(Fail::Missing(sources)) => missing.extend(sources),
                        Err(Fail::SeveralCrates { roots: ids, .. } | Fail::Ambiguous(ids)) => {
                            return Err(ResolveError::Ambiguous(ids));
                        }
                        Err(Fail::SuperAboveRoot) => {}
                    }
                } else if ns == Namespace::Type
                    && let Some(item) = self.root_items.get(&root)
                {
                    // A path that is exactly a crate path names the crate
                    // root, a module.
                    merge(
                        &mut found,
                        *item,
                        Hit {
                            via_glob: false,
                            last: None,
                        },
                    );
                }
            }
        }
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
            return Err(self.unbound(&missing, text));
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
        (1..segs.len()).any(|taken| {
            let prefix = segs[..taken].join("::");
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
        let segs: Vec<String> = path.split("::").map(|s| strip_raw(s).to_owned()).collect();
        let valid =
            |s: &String| !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_');
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
