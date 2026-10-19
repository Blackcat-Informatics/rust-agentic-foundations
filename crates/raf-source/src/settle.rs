// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The scope tables of the module graph, settled to a fixed point.
//!
//! Every module has a scope: for each name and name space, the items it
//! defines and imports by name, which shadow globs, and the items its glob
//! imports bring in. A lookup answers from the shadowing bindings if there
//! are any, and from the globs only in a name space where no explicit import
//! of the module keeps the name from them: an explicit import shadows the
//! globs of its module in each name space where it binds the name, or may
//! still, or reaches what the index cannot see, which is rustc's rule. Until
//! such an import is settled the asker waits. Every answer only grows as the
//! tables settle; that a name space binds nothing is decided only once no
//! import or glob can bring anything in there, so a decision never takes
//! back an answer, and the result does not depend on the order of the
//! declarations.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::cost::{btree_entry, btree_map};
use crate::error::{LoadError, LoadErrorKind};
use crate::facts::Vis;
use crate::glob_scope::{Nobody, Waiters, ambiguity_seen};
use crate::index::{Index, UseDecl, UseKind, binding_bytes};
use crate::item::{ItemId, Namespace};
use crate::meter::{Meter, Steps};
use crate::path::RelPath;
use crate::resolve::{Fail, Heads, Hit, LOAD_WALK, Walked, places};
use crate::work::{self, Work};

/// How a name came to be bound in a module.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Link {
    pub(crate) module: usize,
    pub(crate) name: String,
    pub(crate) source: Option<Source>,
}

/// A place in the scope tables: a name in a module in a name space.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Source {
    pub(crate) module: usize,
    pub(crate) ns: Namespace,
    pub(crate) name: String,
}

/// Where a bound item came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Origin {
    pub(crate) via_glob: bool,
    /// The visibility of the binding, for the globs that import it.
    pub(crate) vis: Vis,
    pub(crate) link: Option<Link>,
    /// The visibilities of the other routes found to the item, when they
    /// differ from `vis`, sorted: a glob imports the item from wherever any
    /// route makes it visible, whichever route is kept. `None` for one route.
    pub(crate) also: Option<Box<[Vis]>>,
}

/// A total order of visibilities, so that a set of them is kept the same
/// way whatever the order the routes were found in.
pub(crate) fn vis_key(vis: &Vis) -> (u8, &str) {
    match vis {
        Vis::Public => (0, ""),
        Vis::Restricted(text) => (1, text.as_str()),
        Vis::Inherited => (2, ""),
    }
}

impl Origin {
    /// Of two routes to one item the lesser wins: a route that uses no glob
    /// beats one that does, and ties go to the smaller link, so the choice
    /// never depends on the order the routes were found in.
    fn rank(&self) -> (bool, &Option<Link>) {
        (self.via_glob, &self.link)
    }

    /// The visibilities of every route found to the item.
    pub(crate) fn visibilities(&self) -> impl Iterator<Item = &Vis> {
        std::iter::once(&self.vis).chain(self.also.iter().flat_map(|also| also.iter()))
    }

    /// The visibilities of `self` and `other` together, as `self` keeps them,
    /// or `None` when `other` adds none. A public route makes the item
    /// public whatever the others are.
    fn widened(&self, other: &Self) -> Option<(Vis, Option<Box<[Vis]>>)> {
        if self.vis == Vis::Public {
            return None;
        }
        if other.visibilities().any(|vis| *vis == Vis::Public) {
            return Some((Vis::Public, None));
        }
        let mut all: Vec<Vis> = self.visibilities().cloned().collect();
        let before = all.len();
        for vis in other.visibilities() {
            if !all.contains(vis) {
                all.push(vis.clone());
            }
        }
        if all.len() == before {
            return None;
        }
        let first = all.remove(0);
        all.sort_by(|left, right| vis_key(left).cmp(&vis_key(right)));
        Some((first, Some(all.into_boxed_slice())))
    }
}

/// The items one name is bound to in one name space, each with the best
/// route found to it, sorted by item.
///
/// Most names are bound to one item, so the bindings are a sorted list that
/// holds exactly what it binds, not a tree whose nodes hold room for many.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Bound {
    sorted: Vec<(ItemId, Origin)>,
}

impl Bound {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.sorted.is_empty()
    }

    pub(crate) fn len(&self) -> usize {
        self.sorted.len()
    }

    pub(crate) fn get(&self, id: ItemId) -> Option<&Origin> {
        self.sorted
            .binary_search_by_key(&id, |(known, _)| *known)
            .ok()
            .map(|at| &self.sorted[at].1)
    }

    pub(crate) fn keys(&self) -> impl Iterator<Item = &ItemId> {
        self.sorted.iter().map(|(id, _)| id)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&ItemId, &Origin)> {
        self.sorted.iter().map(|(id, origin)| (id, origin))
    }

    /// True when [`Bound::offer`] of `id` by `origin` would change the list:
    /// `id` is not bound, `origin` ranks lower than its route, or it makes
    /// the item visible where no route did.
    pub(crate) fn takes(&self, id: ItemId, origin: &Origin) -> bool {
        self.get(id)
            .is_none_or(|known| origin.rank() < known.rank() || known.widened(origin).is_some())
    }

    /// Bind `id` by `origin`. Returns true when `id` was not bound before or
    /// the route makes it visible where none did: a second route to a bound
    /// item replaces the first only when it ranks lower, and the item keeps
    /// the visibilities of both, so what a glob of the module imports does
    /// not depend on which route was found first.
    pub(crate) fn offer(&mut self, id: ItemId, origin: Origin) -> bool {
        match self.sorted.binary_search_by_key(&id, |(known, _)| *known) {
            Ok(at) => {
                let known = &mut self.sorted[at].1;
                let grows = known.widened(&origin).is_some();
                let (kept, union) = if origin.rank() < known.rank() {
                    let union = origin.widened(known);
                    *known = origin;
                    (known, union)
                } else {
                    let union = known.widened(&origin);
                    (known, union)
                };
                if let Some((vis, also)) = union {
                    kept.vis = vis;
                    kept.also = also;
                }
                grows
            }
            Err(at) => {
                if self.sorted.is_empty() {
                    // The first binding takes exactly its own room.
                    self.sorted.reserve_exact(1);
                }
                self.sorted.insert(at, (id, origin));
                true
            }
        }
    }
}

impl<'a> IntoIterator for &'a Bound {
    type Item = (&'a ItemId, &'a Origin);
    type IntoIter = std::iter::Map<
        std::slice::Iter<'a, (ItemId, Origin)>,
        fn(&'a (ItemId, Origin)) -> (&'a ItemId, &'a Origin),
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.sorted.iter().map(|(id, origin)| (id, origin))
    }
}

impl IntoIterator for Bound {
    type Item = (ItemId, Origin);
    type IntoIter = std::vec::IntoIter<(ItemId, Origin)>;

    fn into_iter(self) -> Self::IntoIter {
        self.sorted.into_iter()
    }
}

impl FromIterator<(ItemId, Origin)> for Bound {
    fn from_iter<I: IntoIterator<Item = (ItemId, Origin)>>(pairs: I) -> Self {
        let mut bound = Self::new();
        for (id, origin) in pairs {
            bound.offer(id, origin);
        }
        bound
    }
}

/// The walks of an import's path, one in each name space.
type Walks = Vec<(Namespace, Result<Walked, Fail>)>;

/// The bindings of one name in the three name spaces that shadow globs: the
/// module's items and its explicit imports. What its globs bring in is kept
/// apart, in the glob tables (see [`crate::glob_scope`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Entry {
    pub(crate) defs: [Bound; 3],
}

/// The names of one module. Each entry is boxed, so a node of the table holds
/// eleven names and pointers and not eleven entries of three lists each.
pub(crate) type Scope = BTreeMap<String, Box<Entry>>;

pub(crate) fn slot(ns: Namespace) -> usize {
    match ns {
        Namespace::Type => 0,
        Namespace::Value => 1,
        Namespace::Macro => 2,
    }
}

/// What the work queue of the fixed point holds.
#[derive(Default)]
struct Pending {
    queue: VecDeque<usize>,
    queued: BTreeSet<usize>,
    /// Imports waiting for a name of a module to be bound or settled, by
    /// module and name.
    waiting: BTreeMap<usize, BTreeMap<String, BTreeSet<usize>>>,
    /// The modules where an import waits on each name.
    waiting_by_name: BTreeMap<String, BTreeSet<usize>>,
    /// Globs watching a module for its globs to settle.
    watching: BTreeMap<usize, BTreeSet<usize>>,
}

impl Pending {
    fn push(&mut self, position: usize) {
        if self.queued.insert(position) {
            self.queue.push_back(position);
        }
    }

    /// Queue the import at `position` again whenever one of the places its
    /// path read gains a binding or an import that binds it settles.
    fn wait(&mut self, position: usize, reads: Vec<Source>) {
        for (module, name) in places(reads) {
            self.waiting_by_name
                .entry(name.clone())
                .or_default()
                .insert(module);
            self.waiting
                .entry(module)
                .or_default()
                .entry(name)
                .or_default()
                .insert(position);
        }
    }

    /// The imports waiting on `name` in `module`.
    fn waiters(&self, module: usize, name: &str) -> Option<&BTreeSet<usize>> {
        self.waiting.get(&module)?.get(name)
    }

    /// Wake everything that waits on a name of `module`, and the globs that
    /// watch it.
    fn wake_module(&mut self, module: usize) {
        let mut woken: Vec<usize> = self
            .waiting
            .get(&module)
            .into_iter()
            .flat_map(BTreeMap::values)
            .flat_map(|waiters| waiters.iter().copied())
            .collect();
        if let Some(watchers) = self.watching.get(&module) {
            woken.extend(watchers.iter().copied());
        }
        for position in woken {
            self.push(position);
        }
    }

    /// Wake the globs that watch `module`.
    fn wake_watchers(&mut self, module: usize) {
        let woken: Vec<usize> = self
            .watching
            .get(&module)
            .into_iter()
            .flatten()
            .copied()
            .collect();
        for position in woken {
            self.push(position);
        }
    }
}

impl Waiters for Pending {
    fn wake(&mut self, module: usize, name: &str) {
        let woken: Vec<usize> = self
            .waiters(module, name)
            .into_iter()
            .flatten()
            .copied()
            .collect();
        for position in woken {
            self.push(position);
        }
    }

    fn waiting_on(&self, name: &str) -> Option<&BTreeSet<usize>> {
        self.waiting_by_name.get(name)
    }

    fn waited_in(&self, module: usize) -> Vec<String> {
        self.waiting
            .get(&module)
            .map_or_else(Vec::new, |names| names.keys().cloned().collect())
    }
}

impl Index {
    pub(crate) fn module_depth(&self, mut module: usize) -> usize {
        let mut depth = 0;
        while let Some(parent) = self.modules[module].parent {
            depth += 1;
            module = parent;
        }
        depth
    }

    /// True when `module` is `ancestor` or lies inside it.
    pub(crate) fn within(&self, mut module: usize, ancestor: usize) -> bool {
        loop {
            if module == ancestor {
                return true;
            }
            match self.modules[module].parent {
                Some(parent) => module = parent,
                None => return false,
            }
        }
    }

    /// True when a binding with visibility `vis`, in the scope of `home`, can
    /// be named from `importer`.
    pub(crate) fn is_visible(
        &self,
        vis: &Vis,
        home: usize,
        importer: usize,
        steps: Steps<'_>,
    ) -> bool {
        match vis {
            Vis::Public => true,
            Vis::Inherited => self.within(importer, home),
            Vis::Restricted(text) => match text.as_str() {
                "crate" => self.modules[importer].root == self.modules[home].root,
                "self" => self.within(importer, home),
                "super" => self.modules[home]
                    .parent
                    .is_some_and(|parent| self.within(importer, parent)),
                other => other.strip_prefix("in ").is_some_and(|written| {
                    let segs: Vec<String> = written
                        .split_whitespace()
                        .collect::<String>()
                        .split("::")
                        .map(str::to_owned)
                        .collect();
                    self.walk(home, &segs, false, Namespace::Type, Heads::Local, steps)
                        .is_ok_and(|walked| {
                            let mut modules = walked.modules;
                            modules
                                .extend(walked.items.keys().flat_map(|id| self.modules_for(*id)));
                            modules
                                .iter()
                                .any(|ancestor| self.within(importer, *ancestor))
                        })
                }),
            },
        }
    }

    /// Bind `id` under `name` among the shadowing bindings of `module`.
    /// Returns true when the item was not bound there before; a second route
    /// to a bound item only improves the recorded origin.
    pub(crate) fn insert_def(
        &mut self,
        module: usize,
        ns: Namespace,
        name: &str,
        id: ItemId,
        origin: Origin,
    ) -> bool {
        let entry = self.scopes[module].entry(name.to_owned()).or_default();
        entry.defs[slot(ns)].offer(id, origin)
    }

    /// The names of each module: its definitions. The glob tables are
    /// dropped, to be built again as the tables settle.
    pub(crate) fn build_scopes(&mut self) {
        self.clear_globs();
        self.scopes = vec![Scope::new(); self.modules.len()];
        let defs: Vec<(usize, Vec<ItemId>)> = self
            .modules
            .iter()
            .enumerate()
            .map(|(index, module)| (index, module.defs.clone()))
            .collect();
        for (module, ids) in defs {
            for id in ids {
                let item = &self.items[id.0];
                let name = item.name.clone();
                // An exported macro lives at the crate root and is nameable
                // from everywhere.
                let vis = if item.kind == crate::item::SymbolKind::MacroRules {
                    Vis::Public
                } else {
                    item.visibility.clone()
                };
                for ns in Namespace::ALL {
                    if self.items[id.0].in_namespace(ns) {
                        let origin = Origin {
                            via_glob: false,
                            vis: vis.clone(),
                            link: None,
                            also: None,
                        };
                        self.insert_def(module, ns, &name, id, origin);
                    }
                }
            }
        }
        self.use_resolved = vec![false; self.uses.len()];
        self.use_settled = vec![false; self.uses.len()];
        self.use_bound = vec![[false; 3]; self.uses.len()];
        self.use_free = vec![[false; 3]; self.uses.len()];
        self.use_decided = vec![[false; 3]; self.uses.len()];
        self.use_empty = vec![[false; 3]; self.uses.len()];
        self.glob_closed = vec![false; self.uses.len()];
        self.glob_reach = vec![BTreeSet::new(); self.uses.len()];
        if self.use_ambiguous.len() != self.uses.len() {
            self.use_ambiguous = vec![false; self.uses.len()];
        }
    }

    /// The items a module binds to `name` in `ns`, and whether they come from
    /// its globs. `None` when it binds none there, or the answer is not
    /// settled yet: the globs answer for a name only in a name space where
    /// no explicit import of the module keeps it from them (see
    /// [`Index::blocked`]). What the globs answer may still grow, as a glob
    /// brings in more, as in rustc: a path that read the name is read again,
    /// and two items make it ambiguous.
    pub(crate) fn lookup(
        &self,
        module: usize,
        ns: Namespace,
        name: &str,
        steps: Steps<'_>,
    ) -> Option<(Cow<'_, Bound>, bool)> {
        self.lookup_in(module, ns, name, None, steps)
    }

    /// [`Index::lookup`] for the path of the named import at `own`. An import
    /// never resolves through its own binding: where its path reads the name
    /// it binds in its own module, the explicit imports of that name, its own
    /// among them, are not there for it. They are alternatives under `cfg`,
    /// as rustc refuses two imports of one name, so the path reads the
    /// module's definitions, then its globs.
    /// [`Index::lookup_in`] as a `use` path of `viewer` reads `module`, as
    /// rustc resolves an import: a binding `viewer` cannot access does not
    /// count, and a definition or explicit import that it cannot access
    /// hides the globs all the same, so one still to bind keeps the name from
    /// them too. A name two globs bring in as different items is the one
    /// exception to what `viewer` cannot access: it is ambiguous to a viewer
    /// that can access any of its items, with all of them (see
    /// [`Index::seen_globbed`]).
    pub(crate) fn lookup_seen(
        &self,
        module: usize,
        ns: Namespace,
        name: &str,
        own: Option<usize>,
        viewer: Option<usize>,
        steps: Steps<'_>,
    ) -> Option<(Cow<'_, Bound>, bool)> {
        let Some(viewer) = viewer.filter(|viewer| *viewer != module) else {
            return self.lookup_in(module, ns, name, own, steps);
        };
        work::add(Work::Lookups, 1);
        let entry = self.scopes[module].get(name);
        let at = slot(ns);
        if let Some(defs) = entry
            .map(|entry| &entry.defs[at])
            .filter(|defs| !defs.is_empty())
        {
            return self
                .seen_by(Cow::Borrowed(defs), module, viewer, steps)
                .map(|bound| (bound, false));
        }
        // An explicit import that `viewer` cannot name still hides the
        // globs once it binds, so it keeps the name from them meanwhile.
        if self.blocked(module, name, ns, own) {
            return None;
        }
        self.seen_globbed(
            Cow::Owned(self.globbed(module, name, at, steps)),
            module,
            viewer,
            steps,
        )
        .map(|bound| (bound, true))
    }

    /// The bindings of `bound`, in `module`, that `viewer` can access, or
    /// `None` when it can access none.
    fn seen_by<'a>(
        &self,
        bound: Cow<'a, Bound>,
        module: usize,
        viewer: usize,
        steps: Steps<'_>,
    ) -> Option<Cow<'a, Bound>> {
        self.seen_in(bound, module, viewer, false, steps)
    }

    /// [`Index::seen_by`] for what the globs of `module` bring in: a name that
    /// they bring in as different items is ambiguous in the module and is as
    /// visible as the most visible of its items, so a viewer that can access
    /// one of them reads all (see [`ambiguity_seen`]).
    fn seen_globbed<'a>(
        &self,
        bound: Cow<'a, Bound>,
        module: usize,
        viewer: usize,
        steps: Steps<'_>,
    ) -> Option<Cow<'a, Bound>> {
        self.seen_in(bound, module, viewer, true, steps)
    }

    /// The one definition of what `viewer` reads of `bound`; `globs` is
    /// whether `bound` is what globs bring in.
    fn seen_in<'a>(
        &self,
        bound: Cow<'a, Bound>,
        module: usize,
        viewer: usize,
        globs: bool,
        steps: Steps<'_>,
    ) -> Option<Cow<'a, Bound>> {
        let visible = |origin: &Origin| {
            origin
                .visibilities()
                .any(|vis| self.is_visible(vis, module, viewer, steps))
        };
        let seen = bound.iter().filter(|(_, origin)| visible(origin)).count();
        if seen == bound.len() || globs && ambiguity_seen(bound.len(), seen) {
            return (!bound.is_empty()).then_some(bound);
        }
        let kept: Bound = bound
            .iter()
            .filter(|(_, origin)| visible(origin))
            .map(|(id, origin)| (*id, origin.clone()))
            .collect();
        (!kept.is_empty()).then_some(Cow::Owned(kept))
    }

    pub(crate) fn lookup_in(
        &self,
        module: usize,
        ns: Namespace,
        name: &str,
        own: Option<usize>,
        steps: Steps<'_>,
    ) -> Option<(Cow<'_, Bound>, bool)> {
        work::add(Work::Lookups, 1);
        let entry = self.scopes[module].get(name);
        let at = slot(ns);
        if own.is_some_and(|position| self.binds_name(position, module, name)) {
            if let Some(entry) = entry {
                let defs = &entry.defs[at];
                if defs.iter().all(|(_, origin)| origin.link.is_none()) {
                    if !defs.is_empty() {
                        return Some((Cow::Borrowed(defs), false));
                    }
                } else {
                    let local: Bound = defs
                        .iter()
                        .filter(|(_, origin)| origin.link.is_none())
                        .map(|(id, origin)| (*id, origin.clone()))
                        .collect();
                    if !local.is_empty() {
                        return Some((Cow::Owned(local), false));
                    }
                }
            }
        } else {
            if let Some(defs) = entry
                .map(|entry| &entry.defs[at])
                .filter(|defs| !defs.is_empty())
            {
                return Some((Cow::Borrowed(defs), false));
            }
            if self.blocked(module, name, ns, own) {
                return None;
            }
        }
        let globbed = self.globbed(module, name, at, steps);
        (!globbed.is_empty()).then_some((Cow::Owned(globbed), true))
    }

    /// True when an explicit import of `module` that binds `name` keeps the
    /// name from the globs in `ns`: it binds it there, may still bind it
    /// there, or reaches what the index cannot see or what does not resolve.
    /// Such an import shadows the globs of its module and the crates of the
    /// extern prelude under that name in that name space, as in rustc, so
    /// neither answers for it there. An import that binds the name in other
    /// name spaces only, and whose path can bind nothing in this one, leaves
    /// it to the globs here ([`Index::use_free`]). Never for the path of the
    /// named import at `own` where it reads the name it binds itself (see
    /// [`Index::lookup_in`]).
    pub(crate) fn blocked(
        &self,
        module: usize,
        name: &str,
        ns: Namespace,
        own: Option<usize>,
    ) -> bool {
        !own.is_some_and(|position| self.binds_name(position, module, name))
            && self
                .imports_of(module, name)
                .iter()
                .any(|position| !self.use_free[*position][slot(ns)])
    }

    /// [`Index::blocked`] as a glob of `module` in `importer` sees it: an
    /// import that `importer` can name counts, and so does one it cannot name
    /// while the globs of `module` bring the name in, as the import keeps
    /// what they bring in from every importer until it is decided (see
    /// [`Index::blocked_in`]); one that a cycle left to the globs of
    /// `module` still counts until it is decided, as rustc brings a glob
    /// binding into other modules only once no single import may still bind
    /// the name ([`Index::left_to_globs`]).
    fn blocked_for(
        &self,
        module: usize,
        name: &str,
        at: usize,
        importer: usize,
        steps: Steps<'_>,
    ) -> bool {
        self.imports_of(module, name).iter().any(|position| {
            (!self.use_free[*position][at] || self.left_to_globs(*position, at))
                && (self.is_visible(&self.uses[*position].vis, module, importer, steps)
                    || self.globbed_any(module, name, at, steps))
        })
    }

    /// True when a glob of `module`, one that `importer` can name when one is
    /// given, other than the one at `except`, may still bring in `name`, or
    /// more of it, in the name space at `at`: a module its globs reach,
    /// directly or through the globs of modules that bind nothing there,
    /// has an explicit import that keeps the name from the globs. A cycle of
    /// globs brings in nothing of its own. A glob that is not settled may
    /// bring in any name; that is asked apart ([`Index::open_glob`]).
    pub(crate) fn pending_at(
        &self,
        module: usize,
        name: &str,
        at: usize,
        importer: Option<usize>,
        except: Option<usize>,
        steps: Steps<'_>,
    ) -> bool {
        // Only an import keeps a name from the globs: with none that the
        // globs of the module reach, none may still bring it in.
        if !self.reaches_import(module) {
            return false;
        }
        let mut seen = BTreeSet::from([module]);
        let mut work = vec![(module, importer, except)];
        while let Some((from, viewer, skip)) = work.pop() {
            for glob in self.globs_of(from) {
                if skip == Some(*glob)
                    || viewer
                        .is_some_and(|to| !self.is_visible(&self.uses[*glob].vis, from, to, steps))
                {
                    continue;
                }
                // The modules the glob reached when it was last tried: a
                // glob's path is not walked here, as walking it may ask this
                // again.
                for &target in &self.glob_reach[*glob] {
                    if !self.reaches_import(target)
                        || self.scopes[target]
                            .get(name)
                            .is_some_and(|entry| !entry.defs[at].is_empty())
                    {
                        continue;
                    }
                    if self.blocked_for(target, name, at, from, steps) {
                        return true;
                    }
                    if seen.insert(target) {
                        work.push((target, Some(from), None));
                    }
                }
            }
        }
        false
    }

    /// True when a glob of `module` that `importer` can name, when one is
    /// given, is not settled.
    fn open_glob(&self, module: usize, importer: Option<usize>, steps: Steps<'_>) -> bool {
        self.globs_of(module).iter().any(|glob| {
            !self.glob_closed[*glob]
                && importer
                    .is_none_or(|from| self.is_visible(&self.uses[*glob].vis, module, from, steps))
        })
    }

    /// True when what `target` binds to `name` in the name space at `at` is
    /// not settled for `importer`, which globs it: it binds nothing there
    /// yet, and an explicit import or a glob of it may still.
    fn undetermined_for(
        &self,
        target: usize,
        name: &str,
        at: usize,
        importer: usize,
        steps: Steps<'_>,
    ) -> bool {
        self.scopes[target]
            .get(name)
            .is_none_or(|entry| entry.defs[at].is_empty())
            && (self.blocked_for(target, name, at, importer, steps)
                || self.pending_at(target, name, at, Some(importer), None, steps))
    }

    /// True when the walk that found `source` unbound waits on an import that
    /// never settles: one that keeps the name there from the globs, or, when
    /// a glob may still bring it in, one that keeps it from a module the glob
    /// reaches. The settling of the tables leaves such imports only in a
    /// cycle and among those that wait on one.
    pub(crate) fn waits_on_cycle(&self, source: &Source, steps: Steps<'_>) -> bool {
        let mut seen: BTreeSet<(usize, String, usize)> = BTreeSet::new();
        let mut work = vec![(source.module, source.name.clone(), slot(source.ns))];
        while let Some((module, name, at)) = work.pop() {
            if !seen.insert((module, name.clone(), at)) {
                continue;
            }
            if self.open_glob(module, None, steps)
                || self
                    .imports_of(module, &name)
                    .iter()
                    .any(|position| !self.use_decided[*position][at])
            {
                return true;
            }
            work.extend(self.pending_sources(module, &name, at, steps));
        }
        false
    }

    /// The places that keep `name` of `module` open through its globs in the
    /// name space at `at`: the name in each module one of those globs reaches
    /// where it is not settled for `module`. None where `module` defines the
    /// name there: the definition shadows its globs for every reader, a path
    /// that reads the name it imports itself among them.
    fn pending_sources(
        &self,
        module: usize,
        name: &str,
        at: usize,
        steps: Steps<'_>,
    ) -> Vec<(usize, String, usize)> {
        self.pending_sources_for(module, name, at, None, steps)
    }

    /// [`Index::pending_sources`] through the globs of `module` that
    /// `viewer`, when one is given, can name: a glob it cannot access brings
    /// it nothing.
    fn pending_sources_for(
        &self,
        module: usize,
        name: &str,
        at: usize,
        viewer: Option<usize>,
        steps: Steps<'_>,
    ) -> Vec<(usize, String, usize)> {
        let mut sources = Vec::new();
        if self.scopes[module].get(name).is_some_and(|entry| {
            entry.defs[at]
                .iter()
                .any(|(_, origin)| origin.link.is_none())
        }) {
            return sources;
        }
        for glob in self.globs_of(module).iter().filter(|glob| {
            viewer
                .is_none_or(|viewer| self.is_visible(&self.uses[**glob].vis, module, viewer, steps))
        }) {
            for &target in &self.glob_reach[*glob] {
                if self.undetermined_for(target, name, at, module, steps) {
                    sources.push((target, name.to_owned(), at));
                }
            }
        }
        sources
    }

    /// True when the import at `position` is a named import of `module` that
    /// binds `name`.
    pub(crate) fn binds_name(&self, position: usize, module: usize, name: &str) -> bool {
        let decl = &self.uses[position];
        decl.module == module
            && matches!(&decl.kind, UseKind::Named { name: bound, .. } if bound == name)
    }

    /// The items that stand for `module` in a scope: a crate root's item, or
    /// the `mod` items that declare the module in its parent.
    pub(crate) fn module_items(&self, module: usize) -> Vec<ItemId> {
        let Some(parent) = self.modules[module].parent else {
            return self.root_items.get(&module).copied().into_iter().collect();
        };
        let path = &self.modules[module].path;
        let name = path.rsplit("::").next().unwrap_or(path);
        self.scopes[parent]
            .get(name)
            .map_or_else(Vec::new, |entry| {
                entry.defs[slot(Namespace::Type)]
                    .keys()
                    .copied()
                    .filter(|id| self.module_of.get(id) == Some(&module))
                    .collect()
            })
    }

    /// The modules that an item stands for: the module of a `mod` item, or
    /// the crate roots of an `extern crate` item.
    pub(crate) fn modules_for(&self, id: ItemId) -> impl Iterator<Item = usize> + '_ {
        self.module_of
            .get(&id)
            .copied()
            .into_iter()
            .chain(self.crate_of.get(&id).into_iter().flatten().copied())
    }

    pub(crate) fn super_check(&self, decl: &UseDecl, segs: &[String]) -> Result<(), LoadError> {
        let skip = usize::from(segs.first().is_some_and(|s| s == "self"));
        let supers = segs.iter().skip(skip).take_while(|s| *s == "super").count();
        if !decl.leading_colon && supers > self.module_depth(decl.module) {
            return Err(LoadError::new(
                &decl.file,
                Some(decl.line),
                LoadErrorKind::SuperAboveRoot {
                    path: segs.join("::"),
                },
            ));
        }
        Ok(())
    }

    /// The modules the path of the glob import at `position` names.
    pub(crate) fn glob_targets(
        &self,
        position: usize,
        steps: Steps<'_>,
    ) -> Result<BTreeSet<usize>, Fail> {
        self.glob_targets_noting(position, None, steps)
    }

    /// [`Index::glob_targets`], adding to `reads` the places the walk of the
    /// path looked up. The path is read without the glob itself, as rustc
    /// resolves an import.
    fn glob_targets_noting(
        &self,
        position: usize,
        reads: Option<&mut Vec<Source>>,
        steps: Steps<'_>,
    ) -> Result<BTreeSet<usize>, Fail> {
        let decl = &self.uses[position];
        let UseKind::Glob { segs } = &decl.kind else {
            return Err(Fail::Missing(Vec::new()));
        };
        let walked = self.walk_noting(
            decl.module,
            segs,
            decl.leading_colon,
            Namespace::Type,
            Heads::Use {
                test: decl.test,
                own: Some(position),
            },
            reads,
            steps,
        )?;
        let mut targets = walked.modules;
        targets.extend(walked.items.keys().flat_map(|id| self.modules_for(*id)));
        // A glob of a name that stands for several crates brings in nothing.
        self.one_crate(&targets, segs.len())?;
        targets.remove(&decl.module);
        Ok(targets)
    }

    /// The route of a binding that the glob import `decl` brings in from
    /// `name` of `target` in the name space at `ns`, with the visibility `vis`
    /// the glob gives it, the glob's own or the narrower one of the binding.
    pub(crate) fn glob_origin(
        decl: &UseDecl,
        vis: Vis,
        target: usize,
        ns: usize,
        name: &str,
    ) -> Origin {
        Origin {
            via_glob: true,
            vis,
            link: Some(Link {
                module: decl.module,
                name: "*".to_owned(),
                source: Some(Source {
                    module: target,
                    ns: Namespace::ALL[ns],
                    name: name.to_owned(),
                }),
            }),
            also: None,
        }
    }

    /// Mark an import settled: resolved, failed, or held as ambiguous. What
    /// waits on the name it binds is woken, and what its module exports under
    /// the name is passed on (see [`Index::touch`]).
    fn settle_use(
        &mut self,
        position: usize,
        pending: &mut Pending,
        meter: &Meter,
    ) -> Result<(), LoadError> {
        if self.use_settled[position] {
            return Ok(());
        }
        self.use_settled[position] = true;
        self.use_decided[position] = [true; 3];
        let decl = &self.uses[position];
        if let UseKind::Named { name, .. } = &decl.kind {
            let module = decl.module;
            let name = name.clone();
            self.touch(module, &name, pending, meter)?;
        }
        Ok(())
    }

    /// Something about `name` in `module` changed: what binds it there, or an
    /// import of it, which may keep it from the globs. What waits on the name
    /// there is woken, what the module exports under it to globs is
    /// recomputed and the change passed on to the globs that read it, and,
    /// when a visibility of the module reads the tables, the globs of the
    /// module are tried again.
    fn touch(
        &mut self,
        module: usize,
        name: &str,
        pending: &mut Pending,
        meter: &Meter,
    ) -> Result<(), LoadError> {
        pending.wake(module, name);
        if self.exports_by_path(module) {
            pending.wake_watchers(module);
        }
        self.refresh(module, name, pending, meter)
    }

    /// Settle the scope tables.
    ///
    /// Imports are resolved from a work queue. An import is queued again
    /// whenever a place its path read gains a binding, or what may still
    /// bind it there settles, so a resolved import binds what its path
    /// reaches once the tables are settled, a chain of re-exports of any
    /// length resolves, and nothing caps the search. A lookup does not answer
    /// from the globs for a name that an explicit import may still bind in
    /// that name space, and what the globs answer only grows.
    /// When the queue is empty, [`Index::settle_stuck`] decides the imports
    /// it left, and the queue runs again, until nothing is left to decide.
    /// The globs are then rebuilt against the settled definitions.
    ///
    /// Returns how many times the queue ran empty and the imports left were
    /// decided. Each time decides, in one pass, every import that a cycle
    /// does not hold, so the number does not grow with a chain of imports:
    /// two for a chain of imports that bind nothing, one for a cycle.
    fn settle(&mut self, meter: &Meter) -> Result<usize, LoadError> {
        self.lookups.built(false);
        for decl in &self.uses {
            match &decl.kind {
                UseKind::Named { segs, .. } | UseKind::Glob { segs } => {
                    self.super_check(decl, segs)?;
                }
            }
        }
        self.glob_visibilities(meter)?;
        self.reset_globs(meter)?;
        // An import held as ambiguous from the start is settled and binds
        // nothing.
        for position in 0..self.uses.len() {
            if self.use_ambiguous[position] {
                self.use_settled[position] = true;
                self.use_decided[position] = [true; 3];
            }
        }
        let mut pending = Pending::default();
        for position in 0..self.uses.len() {
            pending.push(position);
        }
        // First, a `use` path whose first segment a module with globs does
        // not bind waits: a glob may still bring the name in. Then the globs
        // have brought in what they can, and such a path may start at a
        // crate of the extern prelude; every import of a module with globs
        // is tried again, as one that bound a name in one name space may now
        // bind it in another. The path of an import of a module without
        // globs reads the same whether the crates are open or not.
        self.drain(&mut pending, meter)?;
        self.crates_open = true;
        for position in 0..self.uses.len() {
            if !self.globs_of(self.uses[position].module).is_empty() {
                pending.push(position);
            }
        }
        let mut rounds = 0;
        loop {
            self.drain(&mut pending, meter)?;
            rounds += 1;
            let before = self.progress();
            if !self.settle_stuck(&mut pending, meter)? {
                break;
            }
            self.advanced(rounds, before)?;
        }
        self.rebuild_globs(meter)?;
        // What the sets of modules the globs reached hold.
        let reach: usize = self
            .glob_reach
            .iter()
            .filter(|reached| !reached.is_empty())
            .map(|reached| btree_map::<usize, ()>() + reached.len() * btree_entry::<usize, ()>())
            .sum();
        if let Some(&glob) = self.glob_imports.first()
            && reach > 0
        {
            let decl = &self.uses[glob];
            meter.items(reach, &decl.file, Some(decl.line))?;
        }
        Ok(rounds)
    }

    /// How much of what settling the tables decides is decided: the name
    /// spaces each import is decided in, binds in and leaves to the globs in,
    /// the imports settled and held as ambiguous, and the globs
    /// settled. Each only grows while the tables settle, and it is at most
    /// eleven for each import.
    pub(crate) fn progress(&self) -> usize {
        let flags = |rows: &[[bool; 3]]| rows.iter().flatten().filter(|set| **set).count();
        let count = |rows: &[bool]| rows.iter().filter(|set| **set).count();
        flags(&self.use_decided)
            + flags(&self.use_bound)
            + flags(&self.use_free)
            + count(&self.use_settled)
            + count(&self.use_ambiguous)
            + count(&self.glob_closed)
    }

    /// Check that the round `round` of deciding the imports left, which
    /// changed something, decided for good at least one more of the finitely
    /// many things there are to decide than the `before` decided when it
    /// started ([`Index::progress`]), so that the rounds end, in release
    /// builds too. A round that does not is a bug, reported as `Internal`.
    fn advanced(&self, round: usize, before: usize) -> Result<(), LoadError> {
        if self.progress() > before {
            return Ok(());
        }
        Err(self.internal(format!(
            "round {round} of deciding the imports left decided nothing more"
        )))
    }

    /// Check that the tables, settled `settles` times, were settled at most
    /// once more than there are imports for each count of recorded drops of
    /// companions, from none to those recorded (see
    /// [`Index::settle_agreed`]), in
    /// release builds too. More is a bug, reported as `Internal`.
    fn within_settles(&self, settles: usize) -> Result<(), LoadError> {
        if settles <= (self.uses.len() + 1) * (self.drop_count() + 1) {
            return Ok(());
        }
        Err(self.internal(format!(
            "the scope tables were settled {settles} times for {} imports and {} dropped companions",
            self.uses.len(),
            self.drop_count()
        )))
    }

    /// An `Internal` [`LoadError`] reporting that the settling of the scope
    /// tables broke its own bound, at the first import: it is only made when
    /// there are imports to settle.
    fn internal(&self, what: String) -> LoadError {
        let decl = &self.uses[0];
        LoadError::new(
            &decl.file,
            Some(decl.line),
            LoadErrorKind::Internal { what },
        )
    }

    /// Decide the named imports and the globs that are unsettled when the
    /// queue is empty. Returns whether anything changed, so that the queue
    /// runs again.
    ///
    /// Each such import waits, once for each name space it is undecided in,
    /// on the places its path reads there: a name in a module in a name
    /// space. The name it binds itself it reads as rustc does, without
    /// itself, through what the globs of its module may still bring in. A
    /// glob waits on the places its path reads, its own module's read
    /// without it, or once its path resolves on the globs of its targets
    /// that are unsettled. A place that binds nothing waits on the imports
    /// undecided there that keep it from the globs (see [`Index::blocked`]),
    /// on the unsettled globs of its module, and on the places its globs may
    /// still bring the name in from (see [`Index::pending_at`]). The strongly
    /// connected components of this graph are decided in waves, each once
    /// everything it waits on is decided, and the queue runs after each
    /// wave, so that each decision reads settled tables:
    /// - one import that waits on no cycle is decided in the name spaces of
    ///   the component: an ambiguous path is held, and an import that bound
    ///   something in one name space leaves each name space where its path
    ///   reached a place that can no longer bind anything to the globs
    ///   ([`Index::use_free`]), the import itself left out as rustc leaves
    ///   out the import it resolves; one that waits on itself and reads a
    ///   place something may still bind is not decided there;
    /// - a glob, or globs that wait only on one another and whose paths
    ///   resolved, are settled; places that only a cycle of globs keeps open
    ///   stop waiting;
    /// - a cycle of explicit imports in which exactly one place in a name
    ///   space, read by an import of the cycle as the last segment of its
    ///   path, would be answered by a glob of its module, whose renamed
    ///   imports rustc reads without themselves and whose imports read
    ///   nothing else unsettled, is left to that glob there when that ends
    ///   every loop of imports in the cycle, as rustc reads it while it
    ///   resolves the import that reads it; the cycle then resolves through
    ///   the glob, and an import that keeps the place is decided there only
    ///   once it binds what keeps every item a path that read the place
    ///   found the glob answering, or can bind nothing ([`Index::left_to_globs`]). A cycle where no glob answers, whose
    ///   imports all bound their names in another name space, binds nothing
    ///   in this one;
    /// - any other cycle, and every component that waits on one, never
    ///   settles: the names its imports bind stay kept from the globs and
    ///   the crates, and a path through one resolves `Cyclic`.
    ///
    /// A component whose paths now read a place that waits on an import or
    /// a glob outside it, which the graph did not hold, waits for the next
    /// round.
    fn settle_stuck(&mut self, pending: &mut Pending, meter: &Meter) -> Result<bool, LoadError> {
        let graph = self.blocker_graph(meter)?;
        if graph.imports.is_empty() {
            return Ok(false);
        }
        let found = components(&graph.successors);
        let mut component_of = vec![0; graph.successors.len()];
        for (at, component) in found.iter().enumerate() {
            for node in component {
                component_of[*node] = at;
            }
        }
        let mut after: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); found.len()];
        let mut before: Vec<Vec<usize>> = vec![Vec::new(); found.len()];
        for (at, component) in found.iter().enumerate() {
            for node in component {
                for next in &graph.successors[*node] {
                    let target = component_of[*next];
                    if target != at && after[at].insert(target) {
                        before[target].push(at);
                    }
                }
            }
        }
        let mut waiting: Vec<usize> = after.iter().map(BTreeSet::len).collect();
        let mut state = vec![Decided::Open; found.len()];
        let mut wave: Vec<usize> = (0..found.len()).filter(|at| waiting[*at] == 0).collect();
        let mut changed = false;
        while !wave.is_empty() {
            // Every component of a wave is decided against the tables as the
            // wave found them, then the decisions are made together.
            let mut acts = Vec::new();
            // The import each decision is named by at its checkpoint: its
            // own, or the one its component was reached from.
            let mut named = Vec::new();
            for &at in &wave {
                let origin = graph.origin(found[at][0]);
                state[at] = if after[at].iter().any(|next| state[*next] == Decided::Cyclic) {
                    Decided::Cyclic
                } else if after[at].iter().any(|next| state[*next] == Decided::Again) {
                    Decided::Again
                } else {
                    self.decide(&graph, &found[at], pending, &mut acts)
                };
                self.settle_lookups(meter, origin)?;
                named.extend(
                    acts[named.len()..]
                        .iter()
                        .map(|act| act.import().unwrap_or(origin)),
                );
            }
            changed |= !acts.is_empty();
            for (act, position) in acts.into_iter().zip(named) {
                self.act(act, pending, meter)?;
                self.settle_lookups(meter, position)?;
            }
            self.drain(pending, meter)?;
            let mut next_wave = Vec::new();
            for &at in &wave {
                for &waiter in &before[at] {
                    waiting[waiter] -= 1;
                    if waiting[waiter] == 0 {
                        next_wave.push(waiter);
                    }
                }
            }
            wave = next_wave;
        }
        Ok(changed)
    }

    /// The blocker graph of the undecided named imports, a node for each name
    /// space an import is undecided in, and of the unsettled globs (see
    /// [`Index::settle_stuck`]).
    ///
    /// The lookups of each node are charged at a checkpoint once it is
    /// built (see [`Index::settle_lookups`]), naming its import, or for a
    /// place the import it was first reached from.
    fn blocker_graph(&self, meter: &Meter) -> Result<Blockers, LoadError> {
        let members = self.graph_members();
        let node_of: BTreeMap<(usize, Option<usize>), usize> = members
            .iter()
            .enumerate()
            .map(|(node, member)| (*member, node))
            .collect();
        let mut graph = Blockers {
            places: Vec::new(),
            origins: Vec::new(),
            place_of: BTreeMap::new(),
            successors: vec![Vec::new(); members.len()],
            imports: members,
        };
        let mut work = Vec::new();
        for node in 0..graph.imports.len() {
            let position = graph.imports[node].0;
            self.import_node(&mut graph, node, &node_of, &mut work);
            graph.origins.resize(graph.places.len(), position);
            self.settle_lookups(meter, position)?;
        }
        while let Some(node) = work.pop() {
            let origin = graph.origin(node);
            graph.successors[node] = self.place_successors(&mut graph, node, &node_of, &mut work);
            graph.origins.resize(graph.places.len(), origin);
            self.settle_lookups(meter, origin)?;
        }
        Ok(graph)
    }

    /// The successors of the import at `node` of `graph`: the places its
    /// path reads, or for a glob the globs it waits on (see
    /// [`Index::blocker_graph`]). New places are queued in `work`.
    fn import_node(
        &self,
        graph: &mut Blockers,
        node: usize,
        node_of: &BTreeMap<(usize, Option<usize>), usize>,
        work: &mut Vec<usize>,
    ) {
        let (position, at) = graph.imports[node];
        let decl = &self.uses[position];
        if let Some(at) = at {
            for (module, name, ns, head) in self.places_read_in(position, at) {
                if !self.binds_name(position, module, &name) {
                    let viewer = (module != decl.module).then_some(decl.module);
                    let next = graph.place((module, name, ns, None, head, viewer), work);
                    graph.successors[node].push(next);
                    continue;
                }
                // The name it binds itself, which it reads as rustc
                // does, without itself: what the globs of its module may
                // bring in.
                for glob in self.globs_of(module) {
                    if !head && !self.glob_closed[*glob] {
                        graph.successors[node].push(node_of[&(*glob, None)]);
                    }
                }
                for (target, name, ns) in self.pending_sources(module, &name, ns, Steps::Load) {
                    let next = graph.place((target, name, ns, None, false, Some(module)), work);
                    graph.successors[node].push(next);
                }
            }
            return;
        }
        // A glob waits on the places its path reads, or once it resolves
        // on the globs of its targets that are not settled.
        let mut reads = Vec::new();
        if let Ok(targets) = self.glob_targets_noting(position, Some(&mut reads), Steps::Load) {
            for target in targets {
                for glob in self.globs_of(target) {
                    if !self.glob_closed[*glob]
                        && self.is_visible(&self.uses[*glob].vis, target, decl.module, Steps::Load)
                    {
                        graph.successors[node].push(node_of[&(*glob, None)]);
                    }
                }
            }
        } else {
            // A place of its own module is read without the glob. The
            // first segment read under the name of a crate of the extern
            // prelude waits on no glob of the module, as for a named
            // import (see [`Index::places_read_in`]): two globs of
            // dependencies do not wait on each other.
            let places: BTreeSet<Place> = reads
                .into_iter()
                .map(|source| {
                    let own = source.module == decl.module;
                    let except = own.then_some(position);
                    let viewer = (!own).then_some(decl.module);
                    let head = own && self.prelude_head(position, &source.name);
                    (
                        source.module,
                        source.name,
                        slot(source.ns),
                        except,
                        head,
                        viewer,
                    )
                })
                .collect();
            for place in places {
                let next = graph.place(place, work);
                graph.successors[node].push(next);
            }
        }
    }

    /// The successors of the place at `node` of `graph`: what it waits on
    /// when nothing its reader can access binds it (see
    /// [`Index::blocker_graph`]). New places are queued in `work`.
    fn place_successors(
        &self,
        graph: &mut Blockers,
        node: usize,
        node_of: &BTreeMap<(usize, Option<usize>), usize>,
        work: &mut Vec<usize>,
    ) -> Vec<usize> {
        let (module, name, at, except, head, viewer) =
            graph.places[node - graph.imports.len()].clone();
        // What `viewer`, the module whose path reads the place, cannot
        // access does not count, as rustc resolves an import.
        let seen = |vis: &Vis| {
            viewer.is_none_or(|viewer| self.is_visible(vis, module, viewer, Steps::Load))
        };
        let mut next = Vec::new();
        if self
            .lookup_seen(
                module,
                Namespace::ALL[at],
                &name,
                except,
                viewer,
                Steps::Load,
            )
            .is_none()
        {
            for position in self.imports_of(module, &name) {
                if let Some(member) = node_of.get(&(*position, Some(at)))
                    && !self.use_free[*position][at]
                {
                    next.push(*member);
                }
            }
            for glob in self.globs_of(module) {
                if !head
                    && !self.glob_closed[*glob]
                    && except != Some(*glob)
                    && seen(&self.uses[*glob].vis)
                {
                    next.push(node_of[&(*glob, None)]);
                }
            }
            for (target, name, at) in
                self.pending_sources_for(module, &name, at, viewer, Steps::Load)
            {
                next.push(graph.place((target, name, at, None, false, Some(module)), work));
            }
        }
        next
    }

    /// The members of the blocker graph: each named import that is not
    /// settled, once for each name space it is undecided in, then each glob
    /// that is not settled, without a name space.
    fn graph_members(&self) -> Vec<(usize, Option<usize>)> {
        let mut members: Vec<(usize, Option<usize>)> = Vec::new();
        for &position in &self.named_imports {
            if self.use_settled[position] {
                continue;
            }
            for (at, decided) in self.use_decided[position].iter().enumerate() {
                if !decided {
                    members.push((position, Some(at)));
                }
            }
        }
        members.extend(
            self.glob_imports
                .iter()
                .filter(|position| !self.glob_closed[**position])
                .map(|position| (*position, None)),
        );
        members
    }

    /// The places the path of the named import at `position` reads when it
    /// is walked in the name space at `at`, the name it binds itself among
    /// them when it reads it.
    ///
    /// Each place is marked when it is the first segment of the path read in
    /// its own module under the name of a crate of the extern prelude: a glob
    /// of the module that is not settled does not hold such a path back (see
    /// [`Index::unbound_use_head`]), so the place does not wait on one.
    fn places_read_in(&self, position: usize, at: usize) -> BTreeSet<(usize, String, usize, bool)> {
        let decl = &self.uses[position];
        let UseKind::Named { segs, .. } = &decl.kind else {
            return BTreeSet::new();
        };
        let mut reads = Vec::new();
        let _ = self.walk_noting(
            decl.module,
            segs,
            decl.leading_colon,
            Namespace::ALL[at],
            Heads::Use {
                test: decl.test,
                own: Some(position),
            },
            Some(&mut reads),
            Steps::Load,
        );
        reads
            .into_iter()
            .map(|source| {
                let head =
                    source.module == decl.module && self.prelude_head(position, &source.name);
                (source.module, source.name, slot(source.ns), head)
            })
            .collect()
    }

    /// True when the path of the glob import at `position` resolves, or
    /// names nothing only at its first segment, read in the module where the
    /// path starts.
    fn held_at_head(&self, position: usize) -> bool {
        let decl = &self.uses[position];
        let UseKind::Glob { segs } = &decl.kind else {
            return false;
        };
        let Some(head) = segs.first() else {
            return false;
        };
        let sources = match self.glob_targets(position, Steps::Load) {
            Ok(_) => return true,
            Err(Fail::Missing(sources)) => sources,
            Err(_) => return false,
        };
        let heads = Heads::Use {
            test: decl.test,
            own: Some(position),
        };
        let start = if self.crate_relative(decl.module, head, decl.leading_colon, heads) {
            self.modules[decl.module].root
        } else {
            decl.module
        };
        !decl.leading_colon
            && !matches!(head.as_str(), "crate" | "self" | "super")
            && !sources.is_empty()
            && sources
                .iter()
                .all(|source| source.module == start && source.name == *head)
    }

    /// True when `name`, read in the module of the import at `position`, is
    /// the first segment of its path under the name of a crate of the extern
    /// prelude, which a glob of the module that is not settled does not hold
    /// back (see [`Index::unbound_use_head`]).
    fn prelude_head(&self, position: usize, name: &str) -> bool {
        let decl = &self.uses[position];
        let (UseKind::Named { segs, .. } | UseKind::Glob { segs }) = &decl.kind;
        !decl.leading_colon
            && segs.first().is_some_and(|first| first == name)
            && !matches!(name, "crate" | "self" | "super")
            && self.edition(decl.module) != 2015
            && self.in_use_prelude(decl.module, name, decl.test)
    }

    /// True when the place a path read waits on an import undecided there
    /// outside `inside`, or on a glob that is not settled, directly or
    /// through what its globs may still bring in.
    fn waits_outside(
        &self,
        place: (usize, String, usize, bool),
        reader: usize,
        inside: &BTreeSet<usize>,
    ) -> bool {
        let (module, name, at, head) = place;
        let mut seen = BTreeSet::new();
        // Each place is read by a module, which sees only what it can
        // access: the import's own module first, then the module whose
        // glob reaches the place.
        let mut work = vec![(module, name, at, head, reader)];
        while let Some((module, name, at, head, viewer)) = work.pop() {
            if !seen.insert((module, name.clone(), at, viewer)) {
                continue;
            }
            if self
                .lookup_seen(
                    module,
                    Namespace::ALL[at],
                    &name,
                    None,
                    Some(viewer),
                    Steps::Load,
                )
                .is_some()
            {
                continue;
            }
            let visible =
                |vis: &Vis| module == viewer || self.is_visible(vis, module, viewer, Steps::Load);
            if self.imports_of(module, &name).iter().any(|position| {
                !self.use_decided[*position][at]
                    && !self.use_free[*position][at]
                    && !inside.contains(position)
            }) || !head
                && self.globs_of(module).iter().any(|glob| {
                    !self.glob_closed[*glob]
                        && !inside.contains(glob)
                        && visible(&self.uses[*glob].vis)
                })
            {
                return true;
            }
            work.extend(
                self.pending_sources_for(module, &name, at, Some(viewer), Steps::Load)
                    .into_iter()
                    .map(|(target, name, at)| (target, name, at, false, module)),
            );
        }
        false
    }

    /// Decide one component of the blocker graph, whose successors are all
    /// decided, adding what it does to `acts` (see [`Index::settle_stuck`]).
    fn decide(
        &self,
        graph: &Blockers,
        component: &[usize],
        pending: &Pending,
        acts: &mut Vec<Act>,
    ) -> Decided {
        let mut imports: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        let mut globs: Vec<usize> = Vec::new();
        for node in component.iter().filter(|node| **node < graph.imports.len()) {
            match graph.imports[*node] {
                (position, Some(at)) => {
                    if !self.use_decided[position][at] && !self.use_settled[position] {
                        imports.entry(position).or_default().push(at);
                    }
                }
                (position, None) => {
                    if !self.glob_closed[position] {
                        globs.push(position);
                    }
                }
            }
        }
        let looped = component.len() > 1
            || component
                .first()
                .is_some_and(|node| graph.successors[*node].contains(node));
        if !globs.is_empty() {
            // A glob that waits on nothing unsettled is settled: its path
            // reaches what it reaches. Globs that only wait on one another,
            // each of whose paths resolved, are settled together. A glob in a
            // cycle with imports, or one whose path waits on a cycle, is not.
            let resolved = globs
                .iter()
                .all(|position| self.glob_targets(*position, Steps::Load).is_ok());
            // Globs whose paths are each held at their first segment, which
            // only the globs of the module could bring in, wait on nothing
            // but one another: none can resolve before another does, so none
            // resolves, and each is settled bringing in nothing.
            let at_heads = globs.iter().all(|position| self.held_at_head(*position));
            if !imports.is_empty() || (looped && !resolved && !at_heads) {
                return Decided::Cyclic;
            }
            acts.extend(globs.into_iter().map(Act::CloseGlob));
            return Decided::Done;
        }
        let inside: BTreeSet<usize> = imports.keys().copied().collect();
        for (&position, spaces) in &imports {
            for &at in spaces {
                if self
                    .places_read_in(position, at)
                    .into_iter()
                    .any(|place| self.waits_outside(place, self.uses[position].module, &inside))
                {
                    return Decided::Again;
                }
            }
        }
        match imports.len() {
            0 => Decided::Done,
            1 => {
                let (position, spaces) = imports.iter().next().expect("one import");
                self.decide_one(*position, spaces, &inside, looped, pending, acts)
            }
            _ if self.cycle_falls_back(graph, component, &imports, acts) => Decided::Again,
            // A cycle where no glob answers, whose imports all bound their
            // names in another name space: each binds nothing in this one, as
            // in rustc, and is decided.
            _ if self
                .cycle_fallbacks(graph, component, &imports)
                .is_some_and(|fallbacks| fallbacks.is_empty())
                && imports
                    .keys()
                    .all(|position| self.use_bound[*position].iter().any(|bound| *bound)) =>
            {
                let mut decided = Vec::new();
                for (&position, spaces) in &imports {
                    if self.decide_one(position, spaces, &inside, true, pending, &mut decided)
                        != Decided::Done
                    {
                        return Decided::Cyclic;
                    }
                }
                acts.extend(decided);
                Decided::Done
            }
            _ => Decided::Cyclic,
        }
    }

    /// For a cycle of two or more imports: in each name space where exactly
    /// one place of the cycle would be answered by a glob of its module (see
    /// [`Index::cycle_fallbacks`]) and leaving it to the glob ends every loop
    /// of imports in the cycle, leave that place to the glob, and return
    /// whether there was one. The name spaces are apart: a glob that brings
    /// in a unit struct answers in two.
    fn cycle_falls_back(
        &self,
        graph: &Blockers,
        component: &[usize],
        imports: &BTreeMap<usize, Vec<usize>>,
        acts: &mut Vec<Act>,
    ) -> bool {
        let Some(fallbacks) = self.cycle_fallbacks(graph, component, imports) else {
            return false;
        };
        let mut any = false;
        for at in 0..3 {
            let mut here = fallbacks.iter().filter(|place| place.2 == at);
            let (Some((module, name, _)), None) = (here.next(), here.next()) else {
                continue;
            };
            let keepers: Vec<usize> = self
                .imports_of(*module, name)
                .iter()
                .copied()
                .filter(|position| imports.contains_key(position))
                .collect();
            // rustc reads the place through the glob only while it resolves
            // an import whose speculation ends the cycle: one that leaves
            // another loop of imports waiting is never determined.
            if looped_without(graph, component, &(*module, name.clone(), at)) {
                continue;
            }
            acts.push(Act::Fallback(keepers, (*module, name.clone(), at)));
            any = true;
        }
        any
    }

    /// The places of a cycle of two or more imports, in the name spaces each
    /// is undecided in there, that a glob of their module would answer, each
    /// read by an import of the cycle as the last segment of its path and
    /// kept from the globs only by imports of the cycle; or `None` when an
    /// import of the cycle waits anywhere but at the last segment of its
    /// path, or is renamed and rustc cannot resolve it without itself: it
    /// has bound nothing yet, or a glob of its module brings in its name
    /// there.
    fn cycle_fallbacks(
        &self,
        graph: &Blockers,
        component: &[usize],
        imports: &BTreeMap<usize, Vec<usize>>,
    ) -> Option<Vec<(usize, String, usize)>> {
        if imports.len() < 2 {
            return None;
        }
        let nodes: BTreeSet<usize> = component.iter().copied().collect();
        let mut candidates: BTreeSet<(usize, String, usize)> = BTreeSet::new();
        for (&position, spaces) in imports {
            let decl = &self.uses[position];
            let UseKind::Named { name, segs } = &decl.kind else {
                return None;
            };
            let last = segs.last()?;
            for &at in spaces {
                if name != last {
                    let globbed = self.globbed_any(decl.module, name, at, Steps::Load);
                    if !self.use_bound[position].iter().any(|bound| *bound) || globbed {
                        return None;
                    }
                }
                let walk = self.walk(
                    decl.module,
                    segs,
                    decl.leading_colon,
                    Namespace::ALL[at],
                    Heads::Use {
                        test: decl.test,
                        own: Some(position),
                    },
                    Steps::Load,
                );
                match walk {
                    Err(Fail::Missing(sources)) => {
                        if sources.iter().any(|source| &source.name != last) {
                            return None;
                        }
                        for source in sources {
                            let place = (source.module, source.name, slot(source.ns));
                            // A glob answer the reading module cannot access
                            // answers nothing for it, as rustc reads it.
                            let answered = source.module == decl.module
                                || self
                                    .seen_globbed(
                                        Cow::Owned(self.globbed(
                                            place.0,
                                            &place.1,
                                            place.2,
                                            Steps::Load,
                                        )),
                                        place.0,
                                        decl.module,
                                        Steps::Load,
                                    )
                                    .is_some();
                            if answered && graph.nodes_of(&place).any(|node| nodes.contains(&node))
                            {
                                candidates.insert(place);
                            }
                        }
                    }
                    Err(Fail::Ambiguous(_) | Fail::SeveralCrates { .. } | Fail::SuperAboveRoot) => {
                        return None;
                    }
                    Err(Fail::Budget(_)) => unreachable!("{LOAD_WALK}"),
                    Ok(_) => {}
                }
            }
        }
        let kept_inside = |module: usize, name: &str, at: usize| {
            self.imports_of(module, name)
                .iter()
                .any(|position| imports.contains_key(position) && !self.use_free[*position][at])
        };
        let fallbacks: Vec<(usize, String, usize)> = candidates
            .into_iter()
            .filter(|(module, name, at)| {
                kept_inside(*module, name, *at)
                    && self.scopes[*module]
                        .get(name)
                        .is_none_or(|entry| entry.defs[*at].is_empty())
                    && self.globbed_any(*module, name, *at, Steps::Load)
            })
            .collect();
        Some(fallbacks)
    }

    /// Decide the named import at `position` in the name spaces `spaces`, where
    /// it waits on nothing undecided but the imports `inside` its component
    /// (see [`Index::settle_stuck`]), which is `looped` when they wait on one
    /// another or the import on itself. In a loop, a name space where the
    /// path reads a place that something may still bind without the imports
    /// inside is not decided: rustc, which resolves an import without
    /// itself but cannot read through a glob a name that an import still
    /// keeps from the globs, never determines it.
    fn decide_one(
        &self,
        position: usize,
        spaces: &[usize],
        inside: &BTreeSet<usize>,
        looped: bool,
        pending: &Pending,
        acts: &mut Vec<Act>,
    ) -> Decided {
        let decl = &self.uses[position];
        let UseKind::Named { segs, .. } = &decl.kind else {
            return Decided::Done;
        };
        let (walks, _, _) = self.import_paths(position, decl, segs);
        if walks
            .iter()
            .any(|(_, walk)| matches!(walk, Err(Fail::Ambiguous(_))))
        {
            acts.push(Act::Hold(position));
            return Decided::Done;
        }
        let mut decided = [false; 3];
        let mut empty = [false; 3];
        for (ns, walk) in walks {
            let at = slot(ns);
            if !spaces.contains(&at) {
                continue;
            }
            match walk {
                Err(Fail::Missing(sources)) => {
                    decided[at] = true;
                    empty[at] = self.binds_nothing_at(position, segs, &sources, inside);
                    // Nor is one that a cycle left to the globs, whose path
                    // may still reach something there.
                    if (looped || self.left_to_globs(position, at)) && !empty[at] {
                        return Decided::Cyclic;
                    }
                }
                walk => {
                    // A path that now reaches something, as a glob no longer
                    // may bring in the crate name its first segment reads,
                    // is tried again in the queue before it is decided. One
                    // that a cycle left to the globs and that would take an
                    // item they answer away is never decided there.
                    let items = self.binds(ns, walk, segs.len());
                    if !items.is_empty() {
                        if self.left_to_globs(position, at)
                            && !self.covers_globbed(position, at, &items, pending)
                        {
                            return Decided::Cyclic;
                        }
                        acts.push(Act::Requeue(position));
                        return Decided::Again;
                    }
                    decided[at] = true;
                }
            }
        }
        acts.push(Act::Settle(position, decided, empty));
        Decided::Done
    }

    /// True when a walk of the path of the named import at `position`, which
    /// found the places `sources` unbound, can bind nothing whatever is still
    /// to be settled: no explicit import other than those `inside` its
    /// component keeps any of them from the globs, no glob may still bring
    /// one in, none is the first segment of the path read as a crate the
    /// index cannot see, and no glob of their modules fails to resolve, as
    /// one of a crate outside the index does.
    fn binds_nothing_at(
        &self,
        position: usize,
        segs: &[String],
        sources: &[Source],
        inside: &BTreeSet<usize>,
    ) -> bool {
        let decl = &self.uses[position];
        !sources.is_empty()
            && sources.iter().all(|source| {
                let outside_head = source.module == decl.module
                    && segs.first() == Some(&source.name)
                    && self.in_use_prelude(decl.module, &source.name, decl.test);
                let at = slot(source.ns);
                // The module of the import sees only what it can access in
                // the module it reads, as rustc resolves the import.
                let viewer = decl.module;
                let visible = |vis: &Vis| {
                    source.module == viewer
                        || self.is_visible(vis, source.module, viewer, Steps::Load)
                };
                !outside_head
                    && (self.binds_name(position, source.module, &source.name)
                        || !self
                            .imports_of(source.module, &source.name)
                            .iter()
                            .any(|other| {
                                self.keeps_out(*other, source.module, &source.name, at, inside)
                            }))
                    && !self.open_glob(source.module, Some(viewer), Steps::Load)
                    && self
                        .pending_sources_for(
                            source.module,
                            &source.name,
                            at,
                            Some(viewer),
                            Steps::Load,
                        )
                        .into_iter()
                        .all(|place| !self.may_bind_without(place, source.module, inside))
                    && self
                        .globs_of(source.module)
                        .iter()
                        .filter(|glob| visible(&self.uses[**glob].vis))
                        .all(|glob| self.glob_targets(*glob, Steps::Load).is_ok())
            })
    }

    /// True when the import at `other`, of `name` in `module`, may still
    /// keep the name there in the name space at `at` from the globs or bind
    /// it, with the imports `inside` a component left out: it is not one of
    /// those, its name is not left to the globs there, and it is not decided
    /// to bind nothing there with no glob bringing the name in.
    fn keeps_out(
        &self,
        other: usize,
        module: usize,
        name: &str,
        at: usize,
        inside: &BTreeSet<usize>,
    ) -> bool {
        let left_empty =
            self.use_empty[other][at] && !self.globbed_any(module, name, at, Steps::Load);
        !inside.contains(&other) && !self.use_free[other][at] && !left_empty
    }

    /// True when a glob that reads `place` may bring something in from it,
    /// now or once it is settled, with the imports `inside` a component left
    /// out, as rustc resolves an import without itself: it binds something
    /// there, an import other than those keeps it from the globs, a glob of
    /// its module is not settled or brings something in, or one of the
    /// places its globs may still bring the name in from may.
    fn may_bind_without(
        &self,
        place: (usize, String, usize),
        viewer: usize,
        inside: &BTreeSet<usize>,
    ) -> bool {
        let mut seen = BTreeSet::new();
        // Each place is read by the module whose glob reaches it, which
        // sees only what it can access there.
        let (module, name, at) = place;
        let mut work = vec![(module, name, at, viewer)];
        while let Some((module, name, at, viewer)) = work.pop() {
            if !seen.insert((module, name.clone(), at, viewer)) {
                continue;
            }
            let entry = self.scopes[module].get(&name);
            if entry.is_some_and(|entry| {
                self.seen_by(Cow::Borrowed(&entry.defs[at]), module, viewer, Steps::Load)
                    .is_some()
            }) || self
                .seen_globbed(
                    Cow::Owned(self.globbed(module, &name, at, Steps::Load)),
                    module,
                    viewer,
                    Steps::Load,
                )
                .is_some()
                || self.open_glob(module, Some(viewer), Steps::Load)
                || self.imports_of(module, &name).iter().any(|other| {
                    self.is_visible(&self.uses[*other].vis, module, viewer, Steps::Load)
                        && self.keeps_out(*other, module, &name, at, inside)
                })
            {
                return true;
            }
            work.extend(
                self.pending_sources_without(module, &name, at, viewer, inside)
                    .into_iter()
                    .map(|(target, name, at)| (target, name, at, module)),
            );
        }
        false
    }

    /// [`Index::pending_sources`] through the globs of `module` that
    /// `viewer` can name, with the imports `inside` a component left out: the
    /// places a glob of `module` reads for `name` where the name is kept from
    /// the globs only by those imports count too.
    fn pending_sources_without(
        &self,
        module: usize,
        name: &str,
        at: usize,
        viewer: usize,
        inside: &BTreeSet<usize>,
    ) -> Vec<(usize, String, usize)> {
        let mut sources = Vec::new();
        for glob in self
            .globs_of(module)
            .iter()
            .filter(|glob| self.is_visible(&self.uses[**glob].vis, module, viewer, Steps::Load))
        {
            for &target in &self.glob_reach[*glob] {
                let kept = self.imports_of(target, name).iter().any(|position| {
                    inside.contains(position)
                        && !self.use_free[*position][at]
                        && self.is_visible(&self.uses[*position].vis, target, module, Steps::Load)
                });
                if kept || self.undetermined_for(target, name, at, module, Steps::Load) {
                    sources.push((target, name.to_owned(), at));
                }
            }
        }
        sources
    }

    /// Do what deciding a component decided.
    fn act(&mut self, act: Act, pending: &mut Pending, meter: &Meter) -> Result<(), LoadError> {
        match act {
            Act::Settle(position, decided, empty) => {
                let bound = self.use_bound[position].iter().any(|bound| *bound);
                for at in 0..3 {
                    self.use_decided[position][at] |= decided[at];
                    self.use_empty[position][at] |= empty[at];
                    self.use_free[position][at] |= empty[at] && bound;
                }
                if self.use_decided[position] == [true; 3] {
                    self.settle_use(position, pending, meter)?;
                } else if let UseKind::Named { name, .. } = &self.uses[position].kind {
                    let (module, name) = (self.uses[position].module, name.clone());
                    self.touch(module, &name, pending, meter)?;
                }
            }
            Act::Hold(position) => {
                self.hold(position);
                self.settle_use(position, pending, meter)?;
            }
            Act::Requeue(position) => pending.push(position),
            Act::CloseGlob(position) => {
                self.close_glob(position, pending);
                pending.push(position);
            }
            Act::Fallback(keepers, (module, name, at)) => {
                for position in keepers {
                    self.use_free[position][at] = true;
                }
                self.touch(module, &name, pending, meter)?;
            }
        }
        Ok(())
    }

    /// Settle the scope tables so that the result does not depend on the
    /// order of the declarations.
    ///
    /// While the tables settle, a `use` path may start at a crate before a
    /// glob of its module, whose own path waited on an import that started
    /// at a crate, brings in the same name, which makes it ambiguous. So
    /// once the tables are settled, every import is walked again against
    /// them. If one is ambiguous that was not held from the start, the tables
    /// are rebuilt from the definitions and settled again with every such
    /// import held from the start: a named one binds nothing and keeps its
    /// name from the globs, and a glob brings in nothing. The held imports
    /// only grow, by at least one each time. Once no import is newly held,
    /// the glob reads whose companions the settled tables drop, and that did
    /// not drop them from the start, are recorded (see
    /// [`Index::record_drops`]), and the tables are settled again with them
    /// dropping their companions from the start and with no import held: a
    /// drop changes what the paths read, and an import held for an ambiguity
    /// that a drop removes is held again only if the tables with the drop
    /// show it ambiguous. The recorded drops only grow, by at least one each
    /// time, and between two growths the held imports grow, so the tables
    /// are settled at most once more than there are imports for each count
    /// of recorded drops, which is checked in every build
    /// ([`Index::within_settles`]). Each settle after the first is charged what it builds
    /// again, the scope tables of the definitions and the state of every
    /// import ([`Index::settle_bytes`]), on top of the bindings it makes
    /// again, so the repeated settles are bounded by the budget of index
    /// bytes as well. Returns how many times the tables were settled.
    pub(crate) fn settle_agreed(&mut self, meter: &Meter) -> Result<usize, LoadError> {
        let mut settles = 0;
        loop {
            let held: Vec<bool> = self.use_ambiguous.clone();
            self.settle(meter)?;
            settles += 1;
            self.within_settles(settles)?;
            let mut ambiguous: Vec<bool> = Vec::with_capacity(self.uses.len());
            for position in 0..self.uses.len() {
                ambiguous.push(self.use_ambiguous[position] || self.walks_ambiguous(position));
                self.settle_lookups(meter, position)?;
            }
            // Drops are recorded only from tables that are final for the
            // held imports: tables settled again for a newly held import are
            // thrown away.
            let first = (0..self.uses.len()).find(|at| ambiguous[*at] && !held[*at]);
            let dropped = if first.is_none() {
                let dropped = self.record_drops(meter)?;
                if let Some(&glob) = self.glob_imports.first() {
                    self.settle_lookups(meter, glob)?;
                }
                dropped
            } else {
                0
            };
            let Some(first) = first.or_else(|| (dropped > 0).then(|| self.glob_imports[0])) else {
                #[cfg(debug_assertions)]
                {
                    self.check_settled();
                    // The check is no work of the load: its lookups are not
                    // charged, so a load costs what it does in every build.
                    self.lookups.take();
                }
                return Ok(settles);
            };
            let decl = &self.uses[first];
            meter.items(self.settle_bytes(), &decl.file, Some(decl.line))?;
            self.release_globs(meter);
            self.build_scopes();
            self.crates_open = false;
            // New drops change what every path reads, so the imports are
            // held again only as the tables with the drops show them
            // ambiguous: one held for an ambiguity a drop removes would stay
            // held.
            self.use_ambiguous = if dropped > 0 {
                vec![false; self.uses.len()]
            } else {
                ambiguous
            };
        }
    }

    /// Check that the settled tables agree with every named import. Walked
    /// again against the final tables, with the globs rebuilt: the imports of
    /// a name that resolved reach what they bound, each one that did not
    /// resolve reaches nothing, and each one held as ambiguous walks an
    /// ambiguous path. The queue
    /// binds what a path reads while the tables settle and the globs are
    /// rebuilt once they are settled; this is the invariant that makes the
    /// two agree. Run in debug builds.
    #[cfg(debug_assertions)]
    pub(crate) fn check_settled(&self) {
        let mut at = 0;
        while let Some(&first) = self.named_imports.get(at) {
            let decl = &self.uses[first];
            let UseKind::Named { name, .. } = &decl.kind else {
                unreachable!("the named imports are named");
            };
            let group = self.imports_of(decl.module, name);
            let mut reached: [BTreeSet<ItemId>; 3] = Default::default();
            for &position in group {
                let import = &self.uses[position];
                let UseKind::Named { segs, .. } = &import.kind else {
                    continue;
                };
                let (walks, _, _) = self.import_paths(position, import, segs);
                // A held import binds nothing, and its path is still
                // ambiguous in the settled tables.
                if self.use_ambiguous[position] {
                    assert!(
                        walks
                            .iter()
                            .any(|(_, walk)| matches!(walk, Err(Fail::Ambiguous(_)))),
                        "{}:{}: an import held as ambiguous walks {walks:?} in the settled tables",
                        import.file,
                        import.line
                    );
                    continue;
                }
                for (ns, walk) in walks {
                    // A name space a cycle leaves undecided binds nothing,
                    // and a path through it resolves `Cyclic`.
                    if !self.use_decided[position][slot(ns)] {
                        continue;
                    }
                    let binds = self.binds(ns, walk, segs.len());
                    assert!(
                        self.use_bound[position][slot(ns)] || binds.is_empty(),
                        "{}:{}: an import that bound nothing in {ns:?} reaches {binds:?} there in the settled tables",
                        import.file,
                        import.line
                    );
                    reached[slot(ns)].extend(binds.into_iter().map(|(id, _)| id));
                }
            }
            let entry = self.scopes[decl.module].get(name.as_str());
            for (ns, reached) in reached.iter().enumerate() {
                let defs = entry.map(|entry| &entry.defs[ns]);
                let imported: BTreeSet<ItemId> = defs
                    .into_iter()
                    .flat_map(Bound::iter)
                    .filter(|(_, origin)| origin.link.is_some())
                    .map(|(id, _)| *id)
                    .collect();
                let all: BTreeSet<ItemId> =
                    defs.into_iter().flat_map(Bound::keys).copied().collect();
                assert!(
                    imported.is_subset(reached) && reached.is_subset(&all),
                    "{}:{}: the imports of `{name}` bound {imported:?} and reach {reached:?} in the settled tables",
                    decl.file,
                    decl.line
                );
            }
            at += group.len();
        }
    }

    /// True when the path of the named import at `position`, walked as a
    /// `use` path in the type name space, has an ambiguous first segment.
    fn walks_ambiguous(&self, position: usize) -> bool {
        let decl = &self.uses[position];
        let UseKind::Named { segs, .. } = &decl.kind else {
            return matches!(
                self.glob_targets(position, Steps::Load),
                Err(Fail::Ambiguous(_))
            );
        };
        matches!(
            self.walk(
                decl.module,
                segs,
                decl.leading_colon,
                Namespace::Type,
                Heads::Use {
                    test: decl.test,
                    own: Some(position),
                },
                Steps::Load,
            ),
            Err(Fail::Ambiguous(_))
        )
    }

    /// Run the queue until it is empty.
    /// Check that every lookup step of the load was charged at a checkpoint
    /// once the index is built: none is left on the count, which serves the
    /// load only (a query counts on its own meter). One left is a bug,
    /// reported as `Internal`.
    pub(crate) fn lookups_charged(&self) -> Result<(), LoadError> {
        let left = self.lookups.take();
        if left == 0 {
            self.lookups.built(true);
            return Ok(());
        }
        let file = self
            .uses
            .first()
            .map_or_else(|| self.targets[0].root.clone(), |decl| decl.file.clone());
        Err(LoadError::new(
            &file,
            None,
            LoadErrorKind::Internal {
                what: format!(
                    "{left} lookup steps of the load were made after its last checkpoint"
                ),
            },
        ))
    }

    /// A checkpoint of the lookups of the load: the lookup steps made since
    /// the last one (see [`LookupCount`](crate::meter::LookupCount)) are
    /// charged to the work of the load for the import at `position`, which
    /// made them, and the load is refused there, as at any charge of glob
    /// table work, once the work passes its limit. The lookups answer as
    /// they always do on the way, so what passes the limit is the lookups of
    /// one step: one import tried, one node of the graph of the imports left
    /// undecided, one component of it decided, one decision made, one glob
    /// read again or one import walked again (see
    /// [`Index::settle_impl_lookups`] for the impls).
    pub(crate) fn settle_lookups(&self, meter: &Meter, position: usize) -> Result<(), LoadError> {
        let decl = &self.uses[position];
        self.drain_lookups(meter, &decl.file, Some(decl.line))
    }

    /// Charge the lookup steps made since the last checkpoint to the work
    /// of the load for the declaration at `line` of `file`.
    pub(crate) fn drain_lookups(
        &self,
        meter: &Meter,
        file: &RelPath,
        line: Option<usize>,
    ) -> Result<(), LoadError> {
        let steps = self.lookups.take();
        if steps == 0 {
            return Ok(());
        }
        work::add(Work::LookupCharges, steps);
        meter.work(steps, file, line)
    }

    fn drain(&mut self, pending: &mut Pending, meter: &Meter) -> Result<(), LoadError> {
        while let Some(position) = pending.queue.pop_front() {
            pending.queued.remove(&position);
            let decl = self.uses[position].clone();
            let bound = match &decl.kind {
                // An import held as ambiguous binds nothing.
                UseKind::Named { .. } if self.use_ambiguous[position] => Vec::new(),
                UseKind::Named { name, segs } => {
                    let bound = self.settle_named(position, &decl, name, segs, pending, meter)?;
                    // An import held as ambiguous binds nothing for good and
                    // is settled. One decided in every name space, bound or
                    // not, is settled too (see [`Index::settle_stuck`]).
                    if self.use_ambiguous[position] || self.use_decided[position] == [true; 3] {
                        self.settle_use(position, pending, meter)?;
                    }
                    bound
                }
                UseKind::Glob { .. } => {
                    self.settle_glob(position, &decl, pending, meter)?;
                    Vec::new()
                }
            };
            for (module, name) in bound {
                self.touch(module, &name, pending, meter)?;
            }
            self.settle_lookups(meter, position)?;
        }
        Ok(())
    }

    /// Rebuild every glob table against the final definitions, so that an
    /// explicit import shadows a glob of the same name for every importer,
    /// each glob bringing in from the modules its path reaches in the settled
    /// tables: the tables are cleared and every glob reads its targets again,
    /// which passes on what they export until nothing more is brought in. The
    /// names each glob may still bring in are kept as the queue left them:
    /// each glob recorded them anew whenever a module it reads changed, so
    /// they are those of the settled tables.
    fn rebuild_globs(&mut self, meter: &Meter) -> Result<(), LoadError> {
        // What each glob's path reaches in the settled tables. A path read
        // against tables whose globs are cleared could reach something else,
        // as a crate a glob of the module shadows.
        let mut reached: Vec<(usize, BTreeSet<usize>)> = Vec::new();
        for &position in &self.glob_imports {
            if let Ok(targets) = self.glob_targets(position, Steps::Load) {
                reached.push((position, targets));
            }
            self.settle_lookups(meter, position)?;
        }
        self.reset_globs(meter)?;
        for (position, targets) in reached {
            for target in targets {
                self.read_glob(position, target, &mut Nobody, meter)?;
            }
            self.settle_lookups(meter, position)?;
        }
        Ok(())
    }

    /// The walks of the path of the named import at `position`, written as
    /// `decl` with the segments `segs`, in every name space, read as a `use`
    /// path that does not consult the import's own binding, with the places
    /// they read, those of the walk in the name space at `at` from `ends[at]`
    /// to `ends[at + 1]`.
    fn import_paths(
        &self,
        position: usize,
        decl: &UseDecl,
        segs: &[String],
    ) -> (Walks, Vec<Source>, [usize; 4]) {
        let heads = Heads::Use {
            test: decl.test,
            own: Some(position),
        };
        let mut reads = Vec::new();
        let mut ends = [0; 4];
        let walks = Namespace::ALL
            .into_iter()
            .enumerate()
            .map(|(at, ns)| {
                let walk = self.walk_noting(
                    decl.module,
                    segs,
                    decl.leading_colon,
                    ns,
                    heads,
                    Some(&mut reads),
                    Steps::Load,
                );
                ends[at + 1] = reads.len();
                (ns, walk)
            })
            .collect();
        (walks, reads, ends)
    }

    /// What a walk in `ns` of an import's path of `len` segments binds the
    /// import's name to: the items it reached. A path through a name that
    /// stands for the crate roots of several crates binds the name to them,
    /// so a path through it, or to it, is ambiguous: an import of the crate
    /// name itself names crates, in the type name space; one that goes on
    /// into the crates names something in each name space the path could
    /// end in. A walk that failed otherwise binds nothing.
    fn binds(&self, ns: Namespace, walk: Result<Walked, Fail>, len: usize) -> Vec<(ItemId, Hit)> {
        match walk {
            Ok(walked) => self.walked_items(walked, ns),
            Err(Fail::SeveralCrates { roots, rest }) if ns == Namespace::Type || rest < len => {
                roots
                    .into_iter()
                    .map(|id| {
                        (
                            id,
                            Hit {
                                via_glob: false,
                                last: None,
                            },
                        )
                    })
                    .collect()
            }
            // An ambiguous first segment is refused before a walk is bound.
            Err(
                Fail::Missing(_)
                | Fail::SuperAboveRoot
                | Fail::SeveralCrates { .. }
                | Fail::Ambiguous(_),
            ) => Vec::new(),
            Err(Fail::Budget(_)) => unreachable!("{LOAD_WALK}"),
        }
    }

    /// Try an explicit import. Returns the names it newly bound. The import
    /// waits on every place its path read, so it is tried again when one of
    /// them changes: one that cannot resolve yet waits, and one that resolved
    /// binds what its path reaches as the tables grow.
    fn settle_named(
        &mut self,
        position: usize,
        decl: &UseDecl,
        name: &str,
        segs: &[String],
        pending: &mut Pending,
        meter: &Meter,
    ) -> Result<Vec<(usize, String)>, LoadError> {
        let (walks, reads, ends) = self.import_paths(position, decl, segs);
        // A first segment that is ambiguous binds nothing in any name space,
        // whatever the module binds later, and keeps the name the import
        // binds from the globs.
        if walks
            .iter()
            .any(|(_, walk)| matches!(walk, Err(Fail::Ambiguous(_))))
        {
            self.hold(position);
            return Ok(Vec::new());
        }
        // A name space where the path reached nothing, reading there only
        // places in modules without globs that no other import binds or, for
        // the places it found unbound, that every other import binding the
        // name was decided to bind nothing at, can never bind, whether the
        // crates are open or not (its first segment is read in its own
        // module, which has no globs to wait on): it is decided now, as the
        // stuck phase would decide it (see [`Index::decide_one`]).
        let missing: Vec<usize> = walks
            .iter()
            .enumerate()
            .filter_map(|(at, (ns, walk))| match walk {
                Err(Fail::Missing(sources))
                    if self.reads_final(position, &reads[ends[at]..ends[at + 1]], sources) =>
                {
                    Some(slot(*ns))
                }
                _ => None,
            })
            .collect();
        let decide_now = !missing.is_empty();
        pending.wait(position, reads);
        let mut bound = Vec::new();
        for (ns, walk) in walks {
            if let Err(Fail::SuperAboveRoot) = walk {
                return Err(LoadError::new(
                    &decl.file,
                    Some(decl.line),
                    LoadErrorKind::SuperAboveRoot {
                        path: segs.join("::"),
                    },
                ));
            }
            let items = self.binds(ns, walk, segs.len());
            // An import that a cycle left to the globs binds only what keeps
            // every item that a path which read its name found them
            // answering, so that what a path read there only grows; else
            // rustc never determines it, and neither does the index (see
            // [`Index::left_to_globs`]).
            if self.left_to_globs(position, slot(ns))
                && !items.is_empty()
                && !self.covers_globbed(position, slot(ns), &items, pending)
            {
                continue;
            }
            for (id, hit) in items {
                if !self.use_bound[position].iter().any(|bound| *bound) {
                    // The name spaces where its path can bind nothing are
                    // left to the globs once it binds in one.
                    let empty = self.use_empty[position];
                    for (free, empty) in self.use_free[position].iter_mut().zip(empty) {
                        *free |= empty;
                    }
                }
                self.use_resolved[position] = true;
                self.use_bound[position][slot(ns)] = true;
                self.use_decided[position][slot(ns)] = true;
                let vis = self.import_vis(decl, &hit, id);
                let origin = Origin {
                    via_glob: hit.via_glob,
                    vis,
                    link: Some(Link {
                        module: decl.module,
                        name: name.to_owned(),
                        source: hit.last,
                    }),
                    also: None,
                };
                // A binding is charged when it is made or replaced; one
                // walked again and kept costs nothing more.
                let kept = self.scopes[decl.module]
                    .get(name)
                    .is_some_and(|entry| !entry.defs[slot(ns)].takes(id, &origin));
                if !kept {
                    let source = origin
                        .link
                        .as_ref()
                        .and_then(|link| link.source.as_ref())
                        .map_or(0, |last| last.name.capacity());
                    meter.items(
                        binding_bytes(name, &origin.vis, source),
                        &decl.file,
                        Some(decl.line),
                    )?;
                }
                if self.insert_def(decl.module, ns, name, id, origin) {
                    bound.push((decl.module, name.to_owned()));
                }
            }
        }
        if decide_now {
            let bound_any = self.use_bound[position].iter().any(|bound| *bound);
            let mut changed = false;
            for at in missing {
                if !self.use_decided[position][at] {
                    self.use_decided[position][at] = true;
                    self.use_empty[position][at] = true;
                    self.use_free[position][at] |= bound_any;
                    changed = true;
                }
            }
            if changed {
                self.touch(decl.module, name, pending, meter)?;
            }
        }
        Ok(bound)
    }

    /// The visibility of the binding the named import `decl` makes of `id`,
    /// reached by `hit`: no wider than the binding it re-exports, as in
    /// rustc, which keeps `pub use` of a name its module reads through a
    /// private glob no more visible than the glob made it. The cap is the
    /// widest level at which the importing module can name the binding in the
    /// place the path ended; there is none for a path that ends at an
    /// associated item or at a module (`hit.last` is `None`).
    fn import_vis(&self, decl: &UseDecl, hit: &Hit, id: ItemId) -> Vis {
        let Some(last) = &hit.last else {
            return decl.vis.clone();
        };
        let Some(origin) = self.origin_of(last, id, Steps::Load) else {
            return decl.vis.clone();
        };
        let Some(cap) = origin
            .visibilities()
            .filter(|vis| self.is_visible(vis, last.module, decl.module, Steps::Load))
            .map(|vis| self.level_of(vis, last.module, Steps::Load))
            .reduce(|left, right| self.wider(left, right))
        else {
            return decl.vis.clone();
        };
        let own = self.level_of(&decl.vis, decl.module, Steps::Load);
        let met = self.meet(own, cap);
        if met == own {
            decl.vis.clone()
        } else {
            self.vis_at(met, decl.module)
        }
    }

    /// True when every place in `reads`, read by the path of the named import
    /// at `position` in one name space, is in a module without globs and is
    /// bound by no other import, or, for a place among `missing`, which the
    /// path found unbound, by none that was not decided to bind nothing
    /// there whatever is left to settle: what the path reaches there never
    /// changes.
    fn reads_final(&self, position: usize, reads: &[Source], missing: &[Source]) -> bool {
        reads.iter().all(|source| {
            let unbound = missing.contains(source);
            let at = slot(source.ns);
            self.globs_of(source.module).is_empty()
                && self
                    .imports_of(source.module, &source.name)
                    .iter()
                    .all(|other| {
                        *other == position
                            || unbound && self.use_decided[*other][at] && self.use_empty[*other][at]
                    })
        })
    }

    /// True when the named import at `position` was left out of a cycle in
    /// the name space at `at`, so that the globs of its module answer its
    /// name there while the imports of the cycle resolve (see
    /// [`Index::cycle_falls_back`]), and is not decided there yet. One that
    /// stays so never settles, and a path through its name there resolves
    /// `Cyclic`, as rustc never determines it.
    pub(crate) fn left_to_globs(&self, position: usize, at: usize) -> bool {
        self.use_free[position][at] && !self.use_decided[position][at]
    }

    /// True when `items`, what the named import at `position` would bind in
    /// the name space at `at`, hold every item that each other path that
    /// read its name there, as `pending` records the readers, found the globs
    /// of its module answering there, as that path's module can access them:
    /// binding them then takes nothing away from what a path read there.
    fn covers_globbed(
        &self,
        position: usize,
        at: usize,
        items: &[(ItemId, Hit)],
        pending: &Pending,
    ) -> bool {
        let decl = &self.uses[position];
        let UseKind::Named { name, .. } = &decl.kind else {
            return true;
        };
        let globbed = self.globbed(decl.module, name, at, Steps::Load);
        let readers = pending.waiters(decl.module, name);
        readers.into_iter().flatten().all(|reader| {
            let viewer = self.uses[*reader].module;
            *reader == position
                || self
                    .seen_globbed(Cow::Borrowed(&globbed), decl.module, viewer, Steps::Load)
                    .is_none_or(|seen| {
                        self.is_visible(&decl.vis, decl.module, viewer, Steps::Load)
                            && seen
                                .keys()
                                .all(|id| items.iter().any(|(item, _)| item == id))
                    })
        })
    }

    /// The items a walk in `ns` binds an import's name to: the items it
    /// reached, or for a path that ends at a module and not at a name in it
    /// (`use shared;` of a library of the workspace, `use crate as name;`),
    /// in the type name space, the item of that module.
    fn walked_items(&self, walked: Walked, ns: Namespace) -> Vec<(ItemId, Hit)> {
        let mut items: Vec<(ItemId, Hit)> = walked.items.into_iter().collect();
        if items.is_empty() && ns == Namespace::Type {
            for module in &walked.modules {
                for id in self.module_items(*module) {
                    items.push((
                        id,
                        Hit {
                            via_glob: false,
                            last: None,
                        },
                    ));
                }
            }
        }
        items
    }

    /// Hold the named import at `position` as ambiguous: it binds nothing,
    /// and like every explicit import it keeps its name from the globs and
    /// the crates.
    fn hold(&mut self, position: usize) {
        self.use_ambiguous[position] = true;
    }

    /// Try a glob import: once its path resolves, it reads what each module
    /// the path reaches exports to it (see [`Index::read_glob`]), and what
    /// that brings in is passed on as the targets gain names. The glob is
    /// queued again when a place its path read changes, or a glob of a
    /// target settles. A glob held as ambiguous brings in nothing.
    fn settle_glob(
        &mut self,
        position: usize,
        decl: &UseDecl,
        pending: &mut Pending,
        meter: &Meter,
    ) -> Result<(), LoadError> {
        if self.use_ambiguous[position] {
            self.close_glob(position, pending);
            return Ok(());
        }
        let mut reads = Vec::new();
        let targets = self.glob_targets_noting(position, Some(&mut reads), Steps::Load);
        pending.wait(position, reads);
        let mut settled = false;
        match targets {
            Ok(targets) => {
                self.glob_reach[position].clone_from(&targets);
                // Settled once every glob of its targets that its module can
                // name is.
                settled = targets
                    .iter()
                    .all(|target| !self.open_glob(*target, Some(decl.module), Steps::Load));
                for target in targets {
                    pending.watching.entry(target).or_default().insert(position);
                    self.record_glob_edge(position, decl.module, target, meter)?;
                    self.read_glob(position, target, pending, meter)?;
                }
            }
            // A glob whose path does not resolve yet waits on the places it
            // read, and is decided once the queue is empty.
            Err(Fail::Missing(_)) => {}
            // `super` above the root is refused for every import before the
            // queue runs. A glob of a name that stands for several crates, or
            // whose first segment is ambiguous, brings in nothing: rustc
            // refuses it.
            Err(Fail::SuperAboveRoot | Fail::SeveralCrates { .. } | Fail::Ambiguous(_)) => {
                settled = true;
            }
            Err(Fail::Budget(_)) => unreachable!("{LOAD_WALK}"),
        }
        if settled {
            self.close_glob(position, pending);
        }
        Ok(())
    }

    /// Mark the glob at `position` settled, waking what waits on a name of
    /// its module.
    fn close_glob(&mut self, position: usize, pending: &mut Pending) {
        if !self.glob_closed[position] {
            self.glob_closed[position] = true;
            pending.wake_module(self.uses[position].module);
        }
    }
}

/// The blocker graph of the unsettled named imports (see
/// [`Index::settle_stuck`]): node `n` below `imports.len()` is the import at
/// position `imports[n]`, an import or a glob, and node `imports.len() + p`
/// is the place `places[p]`: a name in a module in a name space, read without
/// the glob at the position given, for the reads of a glob's path in its own
/// module.
struct Blockers {
    imports: Vec<(usize, Option<usize>)>,
    places: Vec<Place>,
    /// For each place, the import whose node it was first reached from:
    /// the checkpoints of the lookups of the place name it.
    origins: Vec<usize>,
    place_of: BTreeMap<Place, usize>,
    successors: Vec<Vec<usize>>,
}

impl Blockers {
    /// The import of the node `node`: its own, or for a place the import it
    /// was first reached from.
    fn origin(&self, node: usize) -> usize {
        match node.checked_sub(self.imports.len()) {
            Some(place) => self.origins[place],
            None => self.imports[node].0,
        }
    }

    /// The node of `place`, added and queued in `work` the first time.
    fn place(&mut self, place: Place, work: &mut Vec<usize>) -> usize {
        if let Some(node) = self.place_of.get(&place) {
            return *node;
        }
        let node = self.imports.len() + self.places.len();
        self.places.push(place.clone());
        self.place_of.insert(place, node);
        self.successors.push(Vec::new());
        work.push(node);
        node
    }

    /// The nodes of the places of `name` in `module` in the name space at
    /// `at` read with no glob left out, by any reader.
    fn nodes_of<'a>(
        &'a self,
        place: &'a (usize, String, usize),
    ) -> impl Iterator<Item = usize> + 'a {
        let (module, name, at) = place;
        self.place_of
            .range((*module, name.clone(), *at, None, false, None)..)
            .take_while(move |((m, n, a, _, _, _), _)| m == module && n == name && a == at)
            .filter(|((_, _, _, except, _, _), _)| except.is_none())
            .map(|(_, node)| *node)
    }
}

/// A place of the blocker graph: a name in a module in a name space, the
/// glob whose own path reads it without the glob, whether an import reads it
/// as the first segment of its path under a crate's name, and the module
/// that reads it from outside, which sees only what it can access.
type Place = (usize, String, usize, Option<usize>, bool, Option<usize>);

/// True when `component` of `graph` still holds a loop through an import
/// once the nodes of `place` are taken out of it.
fn looped_without(graph: &Blockers, component: &[usize], place: &(usize, String, usize)) -> bool {
    let cut: BTreeSet<usize> = graph.nodes_of(place).collect();
    let kept: Vec<usize> = component
        .iter()
        .copied()
        .filter(|node| !cut.contains(node))
        .collect();
    let local: BTreeMap<usize, usize> = kept
        .iter()
        .enumerate()
        .map(|(at, node)| (*node, at))
        .collect();
    // A place that an import keeps waits on the import alone: once it
    // binds, what the globs of the place's module may bring in is shadowed.
    let named = |node: usize| graph.imports.get(node).is_some_and(|(_, at)| at.is_some());
    let successors: Vec<Vec<usize>> = kept
        .iter()
        .map(|node| {
            let next = &graph.successors[*node];
            let kept_by_import = *node >= graph.imports.len() && next.iter().any(|to| named(*to));
            next.iter()
                .filter(|to| !kept_by_import || named(**to))
                .filter_map(|to| local.get(to).copied())
                .collect()
        })
        .collect();
    // A loop of places alone is globs that glob one another, which adds
    // nothing; a loop through an import is one the place does not end.
    components(&successors).iter().any(|found| {
        found.iter().any(|node| named(kept[*node]))
            && (found.len() > 1 || found.iter().any(|node| successors[*node].contains(node)))
    })
}

/// What deciding a component of the blocker graph does.
enum Act {
    /// Decide the import at the position in the name spaces marked first,
    /// binding nothing in those marked second: there its name is left to the
    /// globs once it binds in another.
    Settle(usize, [bool; 3], [bool; 3]),
    /// Hold the import at the position as ambiguous.
    Hold(usize),
    /// Queue the import at the position again.
    Requeue(usize),
    /// Settle the glob at the position.
    CloseGlob(usize),
    /// Leave the place to the globs of its module for the imports of a cycle
    /// that keep it from them.
    Fallback(Vec<usize>, (usize, String, usize)),
}

impl Act {
    /// The import the decision is for: the first of the imports a fallback
    /// leaves the place to.
    fn import(&self) -> Option<usize> {
        match self {
            Self::Settle(position, ..)
            | Self::Hold(position)
            | Self::Requeue(position)
            | Self::CloseGlob(position) => Some(*position),
            Self::Fallback(keepers, _) => keepers.first().copied(),
        }
    }
}

/// How a component of the blocker graph was decided in one round.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Decided {
    /// Not yet.
    Open,
    /// Decided.
    Done,
    /// A cycle, or a component that waits on one: it never settles.
    Cyclic,
    /// Left for the next round.
    Again,
}

/// The strongly connected components of the graph whose node `n` leads to
/// the nodes `successors[n]`, each component once, in reverse topological
/// order: a component comes after every component it leads to. Tarjan's
/// algorithm, with an explicit stack in place of recursion, so a long chain
/// does not exhaust the caller's stack.
fn components(successors: &[Vec<usize>]) -> Vec<Vec<usize>> {
    const UNSEEN: usize = usize::MAX;
    let count = successors.len();
    let mut index = vec![UNSEEN; count];
    let mut low = vec![0; count];
    let mut on_stack = vec![false; count];
    let mut stack: Vec<usize> = Vec::new();
    let mut found: Vec<Vec<usize>> = Vec::new();
    let mut next_index = 0;
    // Each frame is a node and how many of its successors it has visited.
    let mut frames: Vec<(usize, usize)> = Vec::new();
    for root in 0..count {
        if index[root] != UNSEEN {
            continue;
        }
        index[root] = next_index;
        low[root] = next_index;
        next_index += 1;
        stack.push(root);
        on_stack[root] = true;
        frames.push((root, 0));
        while let Some(frame) = frames.last_mut() {
            let (node, visited) = *frame;
            if let Some(&next) = successors[node].get(visited) {
                frame.1 += 1;
                if index[next] == UNSEEN {
                    index[next] = next_index;
                    low[next] = next_index;
                    next_index += 1;
                    stack.push(next);
                    on_stack[next] = true;
                    frames.push((next, 0));
                } else if on_stack[next] {
                    low[node] = low[node].min(index[next]);
                }
                continue;
            }
            frames.pop();
            if let Some(&(parent, _)) = frames.last() {
                low[parent] = low[parent].min(low[node]);
            }
            if low[node] == index[node] {
                let mut component = Vec::new();
                while let Some(member) = stack.pop() {
                    on_stack[member] = false;
                    component.push(member);
                    if member == node {
                        break;
                    }
                }
                found.push(component);
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use super::*;
    use crate::options::LoadOptions;
    use crate::tree::MemoryTree;
    use crate::workspace::Workspace;

    /// The settled index of a workspace of the libraries `shared` and `other`
    /// and the package `demo`, which depends on both, with `text` as
    /// `demo`'s library.
    fn settled(text: &str) -> Index {
        let tree = MemoryTree::from([
            (
                "Cargo.toml",
                "[workspace]\nmembers = [\"shared\", \"other\", \"demo\"]\n",
            ),
            (
                "shared/Cargo.toml",
                "[package]\nname = \"shared\"\nedition = \"2021\"\n",
            ),
            (
                "shared/src/lib.rs",
                "pub mod inner { pub mod other { pub fn x() {} } }\n",
            ),
            (
                "other/Cargo.toml",
                "[package]\nname = \"other\"\nedition = \"2021\"\n",
            ),
            ("other/src/lib.rs", "pub fn x() {}\n"),
            (
                "demo/Cargo.toml",
                "[package]\nname = \"demo\"\nedition = \"2021\"\n\n[dependencies]\n\
                 shared = { path = \"../shared\" }\nother = { path = \"../other\" }\n",
            ),
            ("demo/src/lib.rs", text),
        ]);
        Workspace::load(&tree, &["."])
            .expect("loads")
            .index()
            .clone()
    }

    /// Settle `index` once from its definitions with `held` held from the
    /// start, and return what it charged.
    fn settle_once(index: &mut Index, held: &[bool]) -> usize {
        index.build_scopes();
        index.crates_open = false;
        index.use_ambiguous = held.to_vec();
        let meter = Meter::new(&LoadOptions::default());
        index.settle(&meter).expect("settles");
        meter.usage().item_bytes
    }

    #[test]
    fn each_settle_after_the_first_is_charged_what_it_builds_again() {
        // `use inn::*` waits on `inn`, which starts at the crate `shared`, so
        // `pub use other::x` reads the crate `other` before the glob brings
        // in `inner::other`: the walk against the settled tables finds the
        // ambiguity, and the tables are settled again with it held.
        let mut index = settled("pub use other::x;\nuse shared::inner as inn;\nuse inn::*;\n");
        let held = index.use_ambiguous.clone();
        assert_eq!(held.iter().filter(|held| **held).count(), 1);
        let none = vec![false; held.len()];
        index.build_scopes();
        index.crates_open = false;
        index.use_ambiguous.clone_from(&none);
        let meter = Meter::new(&LoadOptions::default());
        assert_eq!(index.settle_agreed(&meter).expect("settles"), 2);
        assert!(2 <= index.uses.len() + 1);
        let both = meter.usage().item_bytes;
        let first = settle_once(&mut index, &none);
        let second = settle_once(&mut index, &held);
        assert_eq!(both, first + index.settle_bytes() + second);
        // Settled with the ambiguous import held from the start, it settles
        // once and charges no more than the bindings.
        index.build_scopes();
        index.crates_open = false;
        index.use_ambiguous.clone_from(&held);
        let meter = Meter::new(&LoadOptions::default());
        assert_eq!(index.settle_agreed(&meter).expect("settles"), 1);
        assert_eq!(meter.usage().item_bytes, second);
    }

    /// Settle `text` once more from its definitions, with no import held
    /// from the start, and return how many times the imports left were
    /// decided and how many imports were held as ambiguous.
    fn rounds(text: &str) -> (usize, usize) {
        let mut index = settled(text);
        index.build_scopes();
        index.crates_open = false;
        index.use_ambiguous = vec![false; index.uses.len()];
        let meter = Meter::new(&LoadOptions::default());
        let rounds = index.settle(&meter).expect("settles");
        // Each round but the last decided at least one more thing, of at
        // most eleven for each import.
        assert!(rounds <= index.progress() + 1, "{rounds} rounds");
        assert!(index.progress() <= 11 * index.uses.len());
        let held = index.use_ambiguous.iter().filter(|held| **held).count();
        (rounds, held)
    }

    #[test]
    fn the_imports_of_a_name_are_found_among_names_that_share_its_first_eight_bytes() {
        // Names that share their first eight bytes, a name that is a prefix
        // of another, and the same names in two modules.
        let index = settled(
            "mod a { pub use crate::z::abcdefgh2; pub use crate::z::abcdefgh1; \
             pub use crate::z::abcdefgh; pub use crate::z::abc; pub use crate::z::abcdefgh1 as x; }\n\
             mod b { pub use crate::z::abcdefgh1; pub use crate::z::abc; }\n\
             mod z { pub fn abcdefgh2() {} pub fn abcdefgh1() {} pub fn abcdefgh() {} pub fn abc() {} }\n",
        );
        let module = |path: &str| {
            index
                .modules
                .iter()
                .position(|module| module.path.as_str() == path)
                .expect("module")
        };
        let (a, b) = (module("demo::a"), module("demo::b"));
        let names = |module: usize, name: &str| -> Vec<String> {
            index
                .imports_of(module, name)
                .iter()
                .map(|position| match &index.uses[*position].kind {
                    UseKind::Named { name, segs } => format!("{name}={}", segs.join("::")),
                    UseKind::Glob { .. } => "glob".to_owned(),
                })
                .collect()
        };
        assert_eq!(names(a, "abcdefgh1"), ["abcdefgh1=crate::z::abcdefgh1"]);
        assert_eq!(names(a, "abcdefgh2"), ["abcdefgh2=crate::z::abcdefgh2"]);
        assert_eq!(names(a, "abcdefgh"), ["abcdefgh=crate::z::abcdefgh"]);
        assert_eq!(names(a, "abc"), ["abc=crate::z::abc"]);
        assert_eq!(names(a, "x"), ["x=crate::z::abcdefgh1"]);
        assert_eq!(names(a, "abcdefgh3"), Vec::<String>::new());
        assert_eq!(names(a, "ab"), Vec::<String>::new());
        assert_eq!(names(b, "abcdefgh1"), ["abcdefgh1=crate::z::abcdefgh1"]);
        assert_eq!(names(b, "abcdefgh2"), Vec::<String>::new());
    }

    #[test]
    fn a_round_that_decides_nothing_more_or_a_settle_too_many_is_reported_in_every_build() {
        let index = settled("mod a { pub struct T; }\nmod b { pub use super::a::T; }\n");
        let done = index.progress();
        assert!(done > 0);
        assert!(index.advanced(1, done - 1).is_ok());
        let error = index.advanced(3, done).expect_err("no progress");
        assert!(
            matches!(&error.kind, LoadErrorKind::Internal { what }
                if what == "round 3 of deciding the imports left decided nothing more"),
            "{error:?}"
        );
        let error = index.internal("bound".to_owned());
        assert_eq!(error.line, Some(2));
        // One import: the tables are settled at most twice.
        assert!(index.within_settles(2).is_ok());
        let error = index.within_settles(3).expect_err("one settle too many");
        assert!(
            matches!(&error.kind, LoadErrorKind::Internal { what }
                if what == "the scope tables were settled 3 times for 1 imports and 0 dropped companions"),
            "{error:?}"
        );
    }

    #[test]
    fn the_imports_left_are_decided_in_rounds_that_do_not_grow_with_them() {
        // A chain of imports that bind nothing: each waits on the next, and
        // the last names an item that does not exist. No module has globs,
        // so the queue decides each import once the next is decided, and one
        // round confirms that nothing is left.
        for length in [2, 20_000] {
            let mut text = String::new();
            for at in 0..length {
                writeln!(text, "mod m{at} {{ pub use super::m{}::T; }}", at + 1)
                    .expect("write to a string");
            }
            writeln!(text, "mod m{length} {{ pub use crate::nothing::T; }}")
                .expect("write to a string");
            assert_eq!(rounds(&text), (1, 0), "a chain of {length}");
        }
        // The same chain where each module globs an empty module: a glob may
        // bring a name in, so the imports are left to the rounds. Every one
        // fails in one round, and one more confirms that nothing is left.
        for length in [2, 20_000] {
            let mut text = String::from("mod e {}\n");
            for at in 0..length {
                writeln!(
                    text,
                    "mod m{at} {{ pub use super::m{}::T; pub use super::e::*; }}",
                    at + 1
                )
                .expect("write to a string");
            }
            writeln!(
                text,
                "mod m{length} {{ pub use crate::nothing::T; pub use super::e::*; }}"
            )
            .expect("write to a string");
            assert_eq!(rounds(&text), (2, 0), "a chain of {length} beside globs");
        }
        // A cycle, with a chain that waits on it: nothing fails, so one round
        // decides everything.
        for length in [2, 20_000] {
            let mut text = String::new();
            for at in 0..length {
                writeln!(
                    text,
                    "mod m{at} {{ pub use super::m{}::T; }}\nmod w{at} {{ pub use super::m{at}::T; }}",
                    (at + 1) % length
                )
                .expect("write to a string");
            }
            assert_eq!(rounds(&text), (1, 0), "a cycle of {length}");
        }
        // An import that bound its name before the crates opened is walked
        // again in the queue once they are, and binds the crate there: one
        // round decides its macro name space, and one more confirms.
        assert_eq!(
            rounds(
                "pub mod a { pub use other as inn; pub use crate::e::*; }\npub mod e { pub fn other() {} }\n"
            ),
            (2, 0)
        );
        // An import whose first segment a glob and a crate both bring in is
        // held as ambiguous while the queue runs, and adds no round to the
        // chain beside it.
        assert_eq!(
            rounds(
                "pub mod k { pub mod shared { pub fn x() {} } }\nuse k::*;\n\
                 pub use shared::x;\nmod a { pub use super::b::T; }\n\
                 mod b { pub use crate::nothing::T; }\n"
            ),
            (2, 1)
        );
    }

    #[test]
    fn components_come_out_once_each_after_every_component_they_lead_to() {
        // 0 -> 1 -> 2 -> 1 (a cycle), 2 -> 3, 4 -> 0, 5 alone, 6 <-> 7 -> 3.
        let successors = vec![
            vec![1],
            vec![2],
            vec![1, 3],
            vec![],
            vec![0],
            vec![],
            vec![7],
            vec![6, 3],
        ];
        let found = components(&successors);
        let mut sorted: Vec<Vec<usize>> = found
            .iter()
            .map(|component| {
                let mut nodes = component.clone();
                nodes.sort_unstable();
                nodes
            })
            .collect();
        sorted.sort();
        assert_eq!(
            sorted,
            [vec![0], vec![1, 2], vec![3], vec![4], vec![5], vec![6, 7]]
        );
        let at = |node: usize| {
            found
                .iter()
                .position(|component| component.contains(&node))
                .expect("every node is in a component")
        };
        for (node, next) in successors.iter().enumerate() {
            for target in next {
                assert!(at(*target) <= at(node), "{node} -> {target}");
            }
        }
        // A chain far longer than a recursive search could follow on a small
        // stack.
        let chain: Vec<Vec<usize>> = (0..200_000)
            .map(|node| vec![node + 1])
            .chain([vec![0]])
            .collect();
        assert_eq!(components(&chain).len(), 1);
    }

    fn origin(via_glob: bool) -> Origin {
        Origin {
            via_glob,
            vis: Vis::Public,
            link: None,
            also: None,
        }
    }

    #[test]
    fn a_binding_list_stays_sorted_keeps_the_better_route_and_takes_only_its_room() {
        let mut bound = Bound::new();
        assert!(bound.is_empty());
        assert!(bound.offer(ItemId(7), origin(true)));
        // The first binding takes exactly its own room.
        assert_eq!(bound.sorted.capacity(), 1);
        assert!(bound.offer(ItemId(2), origin(true)));
        assert!(bound.offer(ItemId(5), origin(false)));
        let ids: Vec<usize> = bound.keys().map(|id| id.0).collect();
        assert_eq!(ids, [2, 5, 7]);
        // A second route to a bound item is not a new binding; one without a
        // glob replaces one through a glob, and not the other way round.
        assert!(!bound.offer(ItemId(7), origin(false)));
        assert_eq!(bound.get(ItemId(7)), Some(&origin(false)));
        assert!(!bound.offer(ItemId(5), origin(true)));
        assert_eq!(bound.get(ItemId(5)), Some(&origin(false)));
        assert_eq!(bound.get(ItemId(3)), None);
        let pairs: Vec<(usize, bool)> = (&bound)
            .into_iter()
            .map(|(id, origin)| (id.0, origin.via_glob))
            .collect();
        assert_eq!(pairs, [(2, true), (5, false), (7, false)]);
        // Collecting keeps the better route of a repeated item, in any order.
        let collected: Bound = [
            (ItemId(4), origin(true)),
            (ItemId(1), origin(true)),
            (ItemId(4), origin(false)),
        ]
        .into_iter()
        .collect();
        let pairs: Vec<(usize, bool)> = collected
            .into_iter()
            .map(|(id, origin)| (id.0, origin.via_glob))
            .collect();
        assert_eq!(pairs, [(1, true), (4, false)]);
    }
}
