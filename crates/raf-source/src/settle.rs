// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The scope tables of the module graph, settled to a fixed point.
//!
//! Every module has a scope: for each name and name space, the items it
//! defines and imports by name, which shadow globs, and the items its glob
//! imports bring in. A lookup answers from the shadowing bindings if there
//! are any. It answers from the globs only when no explicit import of the
//! module that could bind the same name is still unsettled: until then the
//! answer is indeterminate and the asker waits, which is rustc's rule and
//! makes the result independent of the order of the declarations.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::error::{LoadError, LoadErrorKind};
use crate::facts::Vis;
use crate::index::{Index, UseDecl, UseKind, binding_bytes};
use crate::item::{ItemId, Namespace};
use crate::meter::Meter;
use crate::resolve::{Fail, Heads, Hit, Walked};

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
}

impl Origin {
    /// Of two routes to one item the lesser wins: a route that uses no glob
    /// beats one that does, and ties go to the smaller link, so the choice
    /// never depends on the order the routes were found in.
    fn rank(&self) -> (bool, &Option<Link>) {
        (self.via_glob, &self.link)
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

    /// Bind `id` by `origin`. Returns true when `id` was not bound before; a
    /// second route to a bound item replaces the first only when it ranks
    /// lower.
    pub(crate) fn offer(&mut self, id: ItemId, origin: Origin) -> bool {
        match self.sorted.binary_search_by_key(&id, |(known, _)| *known) {
            Ok(at) => {
                if origin.rank() < self.sorted[at].1.rank() {
                    self.sorted[at].1 = origin;
                }
                false
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

/// The bindings of one name in the three name spaces.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Entry {
    pub(crate) defs: [Bound; 3],
    pub(crate) globbed: [Bound; 3],
}

/// The names of one module. Each entry is boxed, so a node of the table holds
/// eleven names and pointers and not eleven entries of six lists each.
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
    /// Imports waiting for a name of a module to be bound or settled.
    waiting: BTreeMap<(usize, String), BTreeSet<usize>>,
    /// Globs watching a module for new names.
    watching: BTreeMap<usize, BTreeSet<usize>>,
}

impl Pending {
    fn push(&mut self, position: usize) {
        if self.queued.insert(position) {
            self.queue.push_back(position);
        }
    }

    fn wait(&mut self, position: usize, sources: Vec<Source>) {
        for source in sources {
            self.waiting
                .entry((source.module, source.name))
                .or_default()
                .insert(position);
        }
    }

    fn wake(&mut self, module: usize, name: &str) {
        let mut woken: Vec<usize> = Vec::new();
        if let Some(waiters) = self.waiting.get(&(module, name.to_owned())) {
            woken.extend(waiters.iter().copied());
        }
        if let Some(watchers) = self.watching.get(&module) {
            woken.extend(watchers.iter().copied());
        }
        for position in woken {
            self.push(position);
        }
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
    fn within(&self, mut module: usize, ancestor: usize) -> bool {
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
    pub(crate) fn is_visible(&self, vis: &Vis, home: usize, importer: usize) -> bool {
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
                    self.walk(home, &segs, false, Namespace::Type, Heads::Local)
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

    /// Bind `id` under `name` among the glob bindings of `module`; see
    /// [`Index::insert_def`].
    pub(crate) fn insert_globbed(
        &mut self,
        module: usize,
        ns: usize,
        name: &str,
        id: ItemId,
        origin: Origin,
    ) -> bool {
        let entry = self.scopes[module].entry(name.to_owned()).or_default();
        entry.globbed[ns].offer(id, origin)
    }

    /// The names of each module: its definitions.
    pub(crate) fn build_scopes(&mut self) {
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
                        };
                        self.insert_def(module, ns, &name, id, origin);
                    }
                }
            }
        }
        self.use_resolved = vec![false; self.uses.len()];
        self.use_settled = vec![false; self.uses.len()];
        if self.use_ambiguous.len() != self.uses.len() {
            self.use_ambiguous = vec![false; self.uses.len()];
        }
    }

    /// The items a module binds to `name` in `ns`, and whether they come from
    /// its globs. `None` when it binds none, or when the answer is not
    /// determined yet because an explicit import of the module that could
    /// bind the name is unsettled.
    pub(crate) fn lookup(
        &self,
        module: usize,
        ns: Namespace,
        name: &str,
    ) -> Option<(&Bound, bool)> {
        self.lookup_in(module, ns, name, None)
    }

    /// [`Index::lookup`] for the path of the named import at `own`, which
    /// does not wait on itself: an import never resolves through its own
    /// binding.
    pub(crate) fn lookup_in(
        &self,
        module: usize,
        ns: Namespace,
        name: &str,
        own: Option<usize>,
    ) -> Option<(&Bound, bool)> {
        let entry = self.scopes[module].get(name)?;
        let defs = &entry.defs[slot(ns)];
        if !defs.is_empty() {
            return Some((defs, false));
        }
        if self.is_unsettled(module, name, own) {
            return None;
        }
        let globbed = &entry.globbed[slot(ns)];
        if globbed.is_empty() {
            None
        } else {
            Some((globbed, true))
        }
    }

    /// True when an explicit import of `module` that binds `name`, other than
    /// the one at `own`, is unsettled.
    pub(crate) fn is_unsettled(&self, module: usize, name: &str, own: Option<usize>) -> bool {
        let count = self
            .unsettled
            .get(&module)
            .and_then(|names| names.get(name))
            .copied()
            .unwrap_or(0);
        let itself = own.is_some_and(|position| {
            let decl = &self.uses[position];
            (!self.use_settled[position] || self.use_ambiguous[position])
                && decl.module == module
                && matches!(&decl.kind, UseKind::Named { name: bound, .. } if bound == name)
        });
        count > usize::from(itself)
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

    /// The modules a glob path names.
    pub(crate) fn glob_targets(
        &self,
        decl: &UseDecl,
        segs: &[String],
    ) -> Result<BTreeSet<usize>, Fail> {
        let walked = self.walk(
            decl.module,
            segs,
            decl.leading_colon,
            Namespace::Type,
            Heads::Use {
                test: decl.test,
                own: None,
            },
        )?;
        let mut targets = walked.modules;
        targets.extend(walked.items.keys().flat_map(|id| self.modules_for(*id)));
        // A glob of a name that stands for several crates brings in nothing.
        self.one_crate(&targets, segs.len())?;
        targets.remove(&decl.module);
        Ok(targets)
    }

    /// The names a glob of `target` brings in to `importer`: the bindings that
    /// shadow globs first, the target's own glob names where it has neither,
    /// and only those the importer can name. A name that an unsettled import
    /// of the target could still bind is left out until it is settled.
    fn visible(&self, target: usize, importer: usize) -> Vec<(String, usize, Bound)> {
        let mut names = Vec::new();
        for (name, entry) in &self.scopes[target] {
            for (ns, (defs, globbed)) in entry.defs.iter().zip(&entry.globbed).enumerate() {
                let bound = if !defs.is_empty() {
                    defs
                } else if self.is_unsettled(target, name, None) {
                    continue;
                } else {
                    globbed
                };
                let seen: Bound = bound
                    .iter()
                    .filter(|(_, origin)| self.is_visible(&origin.vis, target, importer))
                    .map(|(id, origin)| (*id, origin.clone()))
                    .collect();
                if !seen.is_empty() {
                    names.push((name.clone(), ns, seen));
                }
            }
        }
        names
    }

    fn glob_origin(decl: &UseDecl, target: usize, ns: usize, name: &str) -> Origin {
        Origin {
            via_glob: true,
            vis: decl.vis.clone(),
            link: Some(Link {
                module: decl.module,
                name: "*".to_owned(),
                source: Some(Source {
                    module: target,
                    ns: Namespace::ALL[ns],
                    name: name.to_owned(),
                }),
            }),
        }
    }

    fn glob_pass(&self) -> Vec<Scope> {
        let mut fresh = vec![Scope::new(); self.modules.len()];
        for decl in &self.uses {
            let UseKind::Glob { segs } = &decl.kind else {
                continue;
            };
            // A glob whose path does not resolve brings in nothing.
            let Ok(targets) = self.glob_targets(decl, segs) else {
                continue;
            };
            for target in targets {
                for (name, ns, bound) in self.visible(target, decl.module) {
                    for id in bound.keys() {
                        let origin = Self::glob_origin(decl, target, ns, &name);
                        fresh[decl.module].entry(name.clone()).or_default().globbed[ns]
                            .offer(*id, origin);
                    }
                }
            }
        }
        fresh
    }

    /// Mark an import settled: resolved, or definitively failed. Everything
    /// that waited on the name it binds, or that globs its module, is woken.
    /// An import whose first segment is ambiguous keeps its name
    /// indeterminate.
    fn settle_use(&mut self, position: usize, pending: &mut Pending) {
        if self.use_settled[position] {
            return;
        }
        self.use_settled[position] = true;
        let decl = &self.uses[position];
        if self.use_ambiguous[position] {
            return;
        }
        if let UseKind::Named { name, .. } = &decl.kind {
            let module = decl.module;
            let name = name.clone();
            if let Some(count) = self
                .unsettled
                .get_mut(&module)
                .and_then(|names| names.get_mut(&name))
            {
                *count -= 1;
            }
            pending.wake(module, &name);
        }
    }

    /// Settle the scope tables.
    ///
    /// Imports are resolved from a work queue: an import that cannot resolve
    /// yet waits on the names it needed, and is queued again when one of them
    /// is bound or settled. The work is linear in the length of a chain of
    /// re-exports, and nothing caps it. When the queue is empty, the imports
    /// that wait on nothing that can still change are settled as failed, one
    /// round at a time, and the queue runs again. The globs are then rebuilt
    /// against the settled definitions.
    fn settle(&mut self, meter: &Meter) -> Result<(), LoadError> {
        for decl in &self.uses {
            match &decl.kind {
                UseKind::Named { segs, .. } | UseKind::Glob { segs } => {
                    self.super_check(decl, segs)?;
                }
            }
        }
        for decl in &self.uses {
            if let UseKind::Named { name, .. } = &decl.kind {
                *self
                    .unsettled
                    .entry(decl.module)
                    .or_default()
                    .entry(name.clone())
                    .or_default() += 1;
            }
        }
        // An import held as ambiguous from the start is settled and keeps its
        // name unsettled.
        for (settled, held) in self.use_settled.iter_mut().zip(&self.use_ambiguous) {
            *settled |= *held;
        }
        let mut pending = Pending::default();
        for position in 0..self.uses.len() {
            pending.push(position);
        }
        // First, a `use` path whose first segment a module with globs does
        // not bind waits: a glob may still bring the name in. Then the globs
        // have brought in what they can, and such a path may start at a
        // crate of the extern prelude; the globs are tried again first, then
        // every import still unsettled.
        self.drain(&mut pending, meter)?;
        self.crates_open = true;
        for position in self.glob_imports.clone() {
            pending.push(position);
        }
        for (position, settled) in self.use_settled.iter().enumerate() {
            if !settled {
                pending.push(position);
            }
        }
        loop {
            self.drain(&mut pending, meter)?;
            let stuck: Vec<usize> = (0..self.uses.len())
                .filter(|p| {
                    matches!(self.uses[*p].kind, UseKind::Named { .. }) && !self.use_settled[*p]
                })
                .collect();
            let Some(&first) = stuck.first() else {
                break;
            };
            // Failures first that wait on no other unsettled import.
            let leaves: Vec<usize> = stuck
                .iter()
                .copied()
                .filter(|p| self.blockers(*p, false).is_empty())
                .collect();
            let chosen = if leaves.is_empty() {
                vec![first]
            } else {
                leaves
            };
            for position in chosen {
                self.settle_use(position, &mut pending);
            }
        }
        self.rebuild_globs();
        Ok(())
    }

    /// Settle the scope tables so that the result does not depend on the
    /// order of the declarations.
    ///
    /// While the tables settle, a `use` path may start at a crate before a
    /// glob of its module, whose own path waited on an import that started
    /// at a crate, brings in the same name, which makes it ambiguous. So
    /// once the tables are settled, every named import is walked again
    /// against them. If one is ambiguous that was not held from the start,
    /// the tables are rebuilt from the definitions and settled again with
    /// every such import held from the start: it binds nothing and keeps its
    /// name indeterminate. Likewise, a `use` path may read its first segment
    /// as a crate after an explicit import of that name failed for now and
    /// before it bound the name: an explicit import shadows a crate, so the
    /// tables are settled again with that name of that module never read as
    /// a crate. The held imports and the names kept from crates only grow,
    /// so this ends. Bindings made again are charged again, so the charge
    /// stays at least what is held; the records of crate reads live only
    /// while the tables settle.
    pub(crate) fn settle_agreed(&mut self, meter: &Meter) -> Result<(), LoadError> {
        loop {
            let held: Vec<bool> = self.use_ambiguous.clone();
            self.settle(meter)?;
            let ambiguous: Vec<bool> = (0..self.uses.len())
                .map(|position| self.use_ambiguous[position] || self.walks_ambiguous(position))
                .collect();
            let shadowed: Vec<(usize, String)> = self
                .shadowed_crate_reads()
                .into_iter()
                .filter(|key| !self.no_crate.contains(key))
                .collect();
            if shadowed.is_empty()
                && ambiguous
                    .iter()
                    .zip(&held)
                    .all(|(now, before)| !now || *before)
            {
                self.crate_reads = BTreeMap::new();
                self.no_crate = BTreeSet::new();
                return Ok(());
            }
            self.no_crate.extend(shadowed);
            self.crate_reads.clear();
            self.build_scopes();
            self.unsettled.clear();
            self.crates_open = false;
            self.use_ambiguous = ambiguous;
        }
    }

    /// Record that the `use` path of the import at `position`, which just
    /// resolved, read its first segment as a crate of the extern prelude: a
    /// plain first segment that its module does not bind (`own` is the
    /// named import whose path it is).
    fn note_crate_read(
        &mut self,
        position: usize,
        decl: &UseDecl,
        segs: &[String],
        own: Option<usize>,
    ) {
        let Some(head) = segs.first() else {
            return;
        };
        let heads = Heads::Use {
            test: decl.test,
            own,
        };
        if decl.leading_colon
            || matches!(head.as_str(), "crate" | "self" | "super")
            || self.crate_relative(decl.module, head, false, heads)
            || self
                .lookup_in(decl.module, Namespace::Type, head, own)
                .is_some()
        {
            return;
        }
        self.crate_reads
            .entry((decl.module, head.clone()))
            .or_default()
            .insert(position);
    }

    /// The names that a `use` path read as a crate and that an explicit
    /// import of the module, other than those paths' own, binds in the
    /// settled tables: such a path went to the crate before the import that
    /// shadows it was settled.
    fn shadowed_crate_reads(&self) -> Vec<(usize, String)> {
        self.crate_reads
            .iter()
            .filter(|((module, name), readers)| {
                self.scopes[*module]
                    .get(name)
                    .is_some_and(|entry| !entry.defs[slot(Namespace::Type)].is_empty())
                    && self
                        .imports_of(*module, name)
                        .iter()
                        .any(|import| self.use_resolved[*import] && !readers.contains(import))
            })
            .map(|(key, _)| key.clone())
            .collect()
    }

    /// True when the path of the named import at `position`, walked as a
    /// `use` path in the type name space, has an ambiguous first segment.
    fn walks_ambiguous(&self, position: usize) -> bool {
        let decl = &self.uses[position];
        let UseKind::Named { segs, .. } = &decl.kind else {
            return false;
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
            ),
            Err(Fail::Ambiguous(_))
        )
    }

    /// Run the queue until it is empty.
    fn drain(&mut self, pending: &mut Pending, meter: &Meter) -> Result<(), LoadError> {
        while let Some(position) = pending.queue.pop_front() {
            pending.queued.remove(&position);
            let decl = self.uses[position].clone();
            let bound = match &decl.kind {
                // An import held as ambiguous binds nothing.
                UseKind::Named { .. } if self.use_ambiguous[position] => Vec::new(),
                UseKind::Named { name, segs } => {
                    let bound = self.settle_named(position, &decl, name, segs, pending, meter)?;
                    if self.use_resolved[position] {
                        self.settle_use(position, pending);
                    }
                    bound
                }
                UseKind::Glob { segs } => self.settle_glob(position, &decl, segs, pending),
            };
            for (module, name) in bound {
                pending.wake(module, &name);
            }
        }
        Ok(())
    }

    /// Rebuild every glob table against the final definitions, so that an
    /// explicit import shadows a glob of the same name for every importer.
    fn rebuild_globs(&mut self) {
        for scope in &mut self.scopes {
            for entry in scope.values_mut() {
                entry.globbed = [Bound::new(), Bound::new(), Bound::new()];
            }
        }
        loop {
            let fresh = self.glob_pass();
            let mut changed = false;
            for (module, scope) in fresh.into_iter().enumerate() {
                for (name, entry) in scope {
                    for (ns, bound) in entry.globbed.into_iter().enumerate() {
                        for (id, origin) in bound {
                            changed |= self.insert_globbed(module, ns, &name, id, origin);
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
    }

    /// Try an explicit import. Returns the names it newly bound; an import
    /// that cannot resolve yet waits on the names it needed.
    fn settle_named(
        &mut self,
        position: usize,
        decl: &UseDecl,
        name: &str,
        segs: &[String],
        pending: &mut Pending,
        meter: &Meter,
    ) -> Result<Vec<(usize, String)>, LoadError> {
        let heads = Heads::Use {
            test: decl.test,
            own: Some(position),
        };
        let walks: Vec<(Namespace, Result<Walked, Fail>)> = Namespace::ALL
            .into_iter()
            .map(|ns| {
                (
                    ns,
                    self.walk(decl.module, segs, decl.leading_colon, ns, heads),
                )
            })
            .collect();
        // A first segment that is ambiguous binds nothing in any name space,
        // whatever the module binds later, and keeps the name the import
        // binds indeterminate, so no glob of the module answers for it.
        if walks
            .iter()
            .any(|(_, walk)| matches!(walk, Err(Fail::Ambiguous(_))))
        {
            self.hold(position, decl, name);
            return Ok(Vec::new());
        }
        if walks.iter().any(|(_, walk)| walk.is_ok()) {
            self.note_crate_read(position, decl, segs, Some(position));
        }
        let mut bound = Vec::new();
        for (ns, walk) in walks {
            match walk {
                Ok(walked) => {
                    for (id, hit) in self.walked_items(walked, ns) {
                        self.use_resolved[position] = true;
                        let source = hit.last.as_ref().map_or(0, |last| last.name.capacity());
                        meter.items(
                            binding_bytes(name, &decl.vis, source),
                            &decl.file,
                            Some(decl.line),
                        )?;
                        let origin = Origin {
                            via_glob: hit.via_glob,
                            vis: decl.vis.clone(),
                            link: Some(Link {
                                module: decl.module,
                                name: name.to_owned(),
                                source: hit.last,
                            }),
                        };
                        if self.insert_def(decl.module, ns, name, id, origin) {
                            bound.push((decl.module, name.to_owned()));
                        }
                    }
                }
                Err(Fail::Missing(sources)) => pending.wait(position, sources),
                // The import goes through a name that stands for the crate
                // roots of several crates; it binds its name to them, so a
                // path through it, or to it, is ambiguous. An import of the
                // crate name itself names crates, in the type name space; one
                // that goes on into the crates names something in each name
                // space the path could end in.
                Err(Fail::SeveralCrates { roots, rest }) => {
                    if ns == Namespace::Type || rest < segs.len() {
                        for id in roots {
                            self.use_resolved[position] = true;
                            meter.items(
                                binding_bytes(name, &decl.vis, 0),
                                &decl.file,
                                Some(decl.line),
                            )?;
                            let origin = Origin {
                                via_glob: false,
                                vis: decl.vis.clone(),
                                link: Some(Link {
                                    module: decl.module,
                                    name: name.to_owned(),
                                    source: None,
                                }),
                            };
                            if self.insert_def(decl.module, ns, name, id, origin) {
                                bound.push((decl.module, name.to_owned()));
                            }
                        }
                    }
                }
                // Refused above.
                Err(Fail::Ambiguous(_)) => {}
                Err(Fail::SuperAboveRoot) => {
                    return Err(LoadError::new(
                        &decl.file,
                        Some(decl.line),
                        LoadErrorKind::SuperAboveRoot {
                            path: segs.join("::"),
                        },
                    ));
                }
            }
        }
        Ok(bound)
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

    /// Mark the named import at `position`, which binds `name`, ambiguous.
    /// Its name stays unsettled for good: counted again if the import had
    /// already settled.
    fn hold(&mut self, position: usize, decl: &UseDecl, name: &str) {
        if self.use_ambiguous[position] {
            return;
        }
        self.use_ambiguous[position] = true;
        if self.use_settled[position]
            && let Some(count) = self
                .unsettled
                .get_mut(&decl.module)
                .and_then(|names| names.get_mut(name))
        {
            *count += 1;
        }
    }

    /// Try a glob import. Returns the names it newly bound; the glob is
    /// queued again when its target module changes.
    fn settle_glob(
        &mut self,
        position: usize,
        decl: &UseDecl,
        segs: &[String],
        pending: &mut Pending,
    ) -> Vec<(usize, String)> {
        let mut bound = Vec::new();
        match self.glob_targets(decl, segs) {
            Ok(targets) => {
                self.note_crate_read(position, decl, segs, None);
                for target in targets {
                    pending.watching.entry(target).or_default().insert(position);
                    for (name, ns, names) in self.visible(target, decl.module) {
                        for id in names.keys() {
                            let origin = Self::glob_origin(decl, target, ns, &name);
                            if self.insert_globbed(decl.module, ns, &name, *id, origin) {
                                bound.push((decl.module, name.clone()));
                            }
                        }
                    }
                }
            }
            Err(Fail::Missing(sources)) => pending.wait(position, sources),
            // `super` above the root is refused for every import before the
            // queue runs. A glob of a name that stands for several crates, or
            // whose first segment is ambiguous, brings in nothing: rustc
            // refuses it.
            Err(Fail::SuperAboveRoot | Fail::SeveralCrates { .. } | Fail::Ambiguous(_)) => {}
        }
        bound
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn origin(via_glob: bool) -> Origin {
        Origin {
            via_glob,
            vis: Vis::Public,
            link: None,
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
