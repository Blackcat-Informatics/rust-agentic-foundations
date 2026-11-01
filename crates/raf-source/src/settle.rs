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

use purrdf_lex::walk::WorkList;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::cost::{btree_entry, btree_map, vec_first, vec_slot};
use crate::error::{LoadError, LoadErrorKind};
use crate::facts::Vis;
use crate::glob_scope::{Level, Nobody, Waiters, ambiguity_seen, push_work};
use crate::index::{Index, UseDecl, UseKind, binding_bytes};
use crate::item::{ItemId, Namespace};
use crate::meter::{Meter, NativeAdmission, Steps, Transient};
use crate::path::RelPath;
use crate::resolve::{Fail, Heads, Hit, LOAD_WALK, Unsettled, Walked, insert_module};
use crate::work::{self, Work};

mod shadow;

#[cfg(test)]
mod replay;

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

/// One actual named declaration's bound route, before origins are merged.
pub(crate) struct ImportSupply {
    pub(crate) id: ItemId,
    pub(crate) ns: Namespace,
    pub(crate) source: Option<Source>,
    pub(crate) level: Level,
}

/// One unique closed rooted answer certified for final glob delegation.
pub(crate) struct FinalFallback {
    pub(crate) position: usize,
    pub(crate) at: usize,
    pub(crate) id: ItemId,
}

/// A concrete plain local item that an actual outward reader cannot access.
pub(crate) struct ShadowCut {
    pub(crate) source: Source,
    pub(crate) id: ItemId,
}

/// A supplier glob's current targets and the actual reads of its module path.
struct ClosedGlob {
    targets: BTreeSet<usize>,
    reads: Vec<Source>,
}

/// Where a bound item came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Origin {
    pub(crate) via_glob: bool,
    /// Present only because another accessible candidate carries ambiguity.
    pub(crate) companion: bool,
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
    /// The owned payload of a copy, including alternate visibility text.
    fn clone_bytes(&self) -> usize {
        let vis_bytes = |vis: &Vis| match vis {
            Vis::Restricted(text) => text.len(),
            _ => 0,
        };
        vis_bytes(&self.vis)
            + self.link.as_ref().map_or(0, |link| {
                link.name.len() + link.source.as_ref().map_or(0, |source| source.name.len())
            })
            + self.also.as_ref().map_or(0, |also| {
                also.len() * vec_slot::<Vis>() + also.iter().map(vis_bytes).sum::<usize>()
            })
    }
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

    fn adds(&self, other: &Self) -> bool {
        self.vis != Vis::Public
            && other
                .visibilities()
                .any(|vis| !self.visibilities().any(|known| known == vis))
    }

    /// The visibilities of `self` and `other` together, as `self` keeps them,
    /// or `None` when `other` adds none. A public route makes the item
    /// public whatever the others are.
    fn widened(&self, other: &Self) -> Option<(Vis, Option<Box<[Vis]>>)> {
        if !self.adds(other) {
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

/// A lookup's temporary owned result and its pre-construction reservation.
pub(crate) struct LookupRead<'i, 'm> {
    bound: Option<(Cow<'i, Bound>, bool)>,
    _memory: Option<Transient<'m>>,
}

impl LookupRead<'_, '_> {
    pub(crate) fn as_ref(&self) -> Option<(&Bound, bool)> {
        self.bound
            .as_ref()
            .map(|(bound, globbed)| (bound.as_ref(), *globbed))
    }
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
        self.get(id).is_none_or(|known| {
            origin.rank() < known.rank()
                || known.adds(origin)
                || known.companion && !origin.companion
        })
    }

    /// Reserve work for the actual candidate search, shifts and visibility
    /// union before copying or offering an owned route.
    pub(crate) fn offer_work(
        &self,
        id: ItemId,
        origin: &Origin,
        steps: Steps<'_>,
    ) -> Result<(), Fail> {
        let mut units = self.len() + 1;
        if let Some(known) = self.get(id) {
            let left = known.visibilities().count();
            let right = origin.visibilities().count();
            let total = left + right;
            units += 3 * total * total + total * (total.ilog2() as usize + 1);
        }
        steps.storage_work(units).map_err(Fail::Storage)
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
                let companion = known.companion && origin.companion;
                let grows = known.adds(&origin) || known.companion != companion;
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
                kept.companion = companion;
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

/// The finality of an import read, including privacy and delegated answers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ImportReadState {
    #[default]
    Unread,
    Deferred,
    Missing,
    Refused,
    Late,
    Delegated,
    Shadowed,
}

/// Privacy can wait for ordinary propagation before named-cycle fallback.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum PrivacyDecision {
    #[default]
    Unproven,
    Final,
    Deferred,
}

impl ImportReadState {
    /// Admit an ordinary result without reopening an already final refusal.
    fn permits(&mut self, decision: PrivacyDecision, nonempty: bool) -> bool {
        if matches!(self, Self::Delegated | Self::Shadowed) {
            return false;
        }
        if decision == PrivacyDecision::Deferred && *self == Self::Unread {
            *self = Self::Deferred;
        }
        if nonempty && matches!(*self, Self::Missing | Self::Refused | Self::Late) {
            return false;
        }
        if nonempty && *self == Self::Deferred {
            *self = Self::Unread;
        }
        true
    }
}

/// One import's path reads against an unchanged scope generation.
enum ImportRead {
    Named {
        walks: Walks,
        missing: [bool; 3],
        private: [PrivacyDecision; 3],
    },
    Glob {
        targets: Result<BTreeSet<usize>, Fail>,
        settled: bool,
    },
}

impl ImportRead {
    /// The surviving heap payload, excluding the batch's inline enum slots.
    /// All of this storage was admitted before construction on the read lease.
    fn retained_bytes(&self) -> usize {
        fn tree_bytes<K, V>(len: usize) -> usize {
            if len == 0 {
                0
            } else {
                btree_map::<K, V>() + len * btree_entry::<K, V>()
            }
        }
        fn failure_bytes(failure: &Fail) -> usize {
            match failure {
                Fail::Missing(sources) => {
                    sources.capacity() * size_of::<Source>()
                        + sources
                            .iter()
                            .map(|source| source.name.capacity())
                            .sum::<usize>()
                }
                Fail::SeveralCrates { roots, .. } | Fail::Ambiguous(roots) => {
                    roots.capacity() * size_of::<ItemId>()
                }
                Fail::SuperAboveRoot | Fail::Budget(_) => 0,
                Fail::Storage(_) => unreachable!("storage refusals propagate before retention"),
            }
        }
        match self {
            Self::Named { walks, .. } => {
                walks.capacity() * size_of::<(Namespace, Result<Walked, Fail>)>()
                    + walks
                        .iter()
                        .map(|(_, walk)| match walk {
                            Ok(walk) => {
                                tree_bytes::<usize, ()>(walk.modules.len())
                                    + tree_bytes::<ItemId, Hit>(walk.items.len())
                                    + walk
                                        .items
                                        .values()
                                        .filter_map(|hit| hit.last.as_ref())
                                        .map(|source| source.name.capacity())
                                        .sum::<usize>()
                            }
                            Err(failure) => failure_bytes(failure),
                        })
                        .sum::<usize>()
            }
            Self::Glob { targets, .. } => match targets {
                Ok(targets) => tree_bytes::<usize, ()>(targets.len()),
                Err(failure) => failure_bytes(failure),
            },
        }
    }
}

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
struct Pending<'m> {
    memory: Transient<'m>,
    origins: Vec<(RelPath, usize)>,
    queue: VecDeque<usize>,
    queued: Vec<bool>,
    /// Imports waiting for a name of a module to be bound or settled, by
    /// module and name.
    waiting: BTreeMap<usize, BTreeMap<String, BTreeSet<usize>>>,
    /// The modules where an import waits on each name.
    waiting_by_name: BTreeMap<String, BTreeSet<usize>>,
    /// Globs watching a module for its globs to settle.
    watching: BTreeMap<usize, BTreeSet<usize>>,
    /// Module-wide notifications already delivered to this generation.
    notified: Vec<bool>,
    /// Only these flags need resetting before the next generation applies.
    notifications: Vec<(usize, usize)>,
}

impl<'m> Pending<'m> {
    fn new(uses: &[UseDecl], meter: &'m Meter) -> Result<Self, LoadError> {
        let memory = Transient::new(meter);
        let modules = uses
            .iter()
            .map(|decl| decl.module)
            .max()
            .map_or(0, |last| last + 1);
        if let Some(first) = uses.first() {
            memory.charge(
                vec_first::<bool>()
                    + modules * vec_slot::<bool>()
                    + vec_first::<(usize, usize)>()
                    + uses.len() * vec_slot::<(usize, usize)>()
                    + vec_first::<(RelPath, usize)>()
                    + vec_first::<usize>()
                    + vec_first::<bool>()
                    + uses.len()
                        * (vec_slot::<(RelPath, usize)>()
                            + vec_slot::<usize>()
                            + vec_slot::<bool>())
                    + uses
                        .iter()
                        .map(|decl| decl.file.as_str().len())
                        .sum::<usize>(),
                &first.file,
                Some(first.line),
            )?;
        }
        Ok(Self {
            memory,
            origins: uses
                .iter()
                .map(|decl| (decl.file.clone(), decl.line))
                .collect(),
            queue: VecDeque::with_capacity(uses.len()),
            queued: vec![false; uses.len()],
            waiting: BTreeMap::new(),
            waiting_by_name: BTreeMap::new(),
            watching: BTreeMap::new(),
            notified: vec![false; modules],
            notifications: Vec::with_capacity(uses.len()),
        })
    }

    /// Every queued read has observed the preceding generation. Module-wide
    /// changes applied now must be allowed to notify those readers again.
    fn begin_generation(&mut self) -> Result<(), LoadError> {
        for at in 0..self.notifications.len() {
            let (module, position) = self.notifications[at];
            self.step(position, 1)?;
            self.notified[module] = false;
        }
        self.notifications.clear();
        Ok(())
    }

    fn step(&self, position: usize, units: usize) -> Result<(), LoadError> {
        let (file, line) = &self.origins[position];
        self.memory.work(units, file, Some(*line))?;
        work::add(Work::SettleSteps, units);
        Ok(())
    }

    fn charge(&self, position: usize, bytes: usize) -> Result<(), LoadError> {
        let (file, line) = &self.origins[position];
        self.memory.charge(bytes, file, Some(*line))
    }

    fn push(&mut self, position: usize) -> Result<(), LoadError> {
        Self::push_into(
            &self.memory,
            &self.origins,
            &mut self.queue,
            &mut self.queued,
            position,
        )
    }

    fn push_into(
        memory: &Transient<'_>,
        origins: &[(RelPath, usize)],
        queue: &mut VecDeque<usize>,
        queued: &mut [bool],
        position: usize,
    ) -> Result<(), LoadError> {
        let (file, line) = &origins[position];
        memory.work(1, file, Some(*line))?;
        work::add(Work::SettleSteps, 1);
        if !queued[position] {
            queued[position] = true;
            queue.push_back(position);
        }
        Ok(())
    }

    fn pop(&mut self) -> Result<Option<usize>, LoadError> {
        let Some(&position) = self.queue.front() else {
            return Ok(None);
        };
        self.step(position, 1)?;
        self.queue.pop_front();
        self.queued[position] = false;
        Ok(Some(position))
    }

    /// Queue the import at `position` again whenever one of the places its
    /// path read gains a binding or an import that binds it settles.
    fn wait(&mut self, position: usize, reads: Vec<Source>) -> Result<(), LoadError> {
        for source in reads {
            self.step(position, 1)?;
            let (module, name) = (source.module, source.name);
            let by_name = self.waiting_by_name.get(&name);
            let names = self.waiting.get(&module);
            let readers = names.and_then(|names| names.get(&name));
            let mut bytes = 0;
            if by_name.is_none() {
                bytes += btree_entry::<String, BTreeSet<usize>>()
                    + name.len()
                    + btree_map::<usize, ()>();
                if self.waiting_by_name.is_empty() {
                    bytes += btree_map::<String, BTreeSet<usize>>();
                }
            }
            if by_name.is_none_or(|modules| !modules.contains(&module)) {
                bytes += btree_entry::<usize, ()>();
            }
            if names.is_none() {
                bytes += btree_entry::<usize, BTreeMap<String, BTreeSet<usize>>>()
                    + btree_map::<String, BTreeSet<usize>>();
                if self.waiting.is_empty() {
                    bytes += btree_map::<usize, BTreeMap<String, BTreeSet<usize>>>();
                }
            }
            if readers.is_none() {
                bytes += btree_entry::<String, BTreeSet<usize>>()
                    + name.len()
                    + btree_map::<usize, ()>();
            }
            if readers.is_none_or(|readers| !readers.contains(&position)) {
                bytes += btree_entry::<usize, ()>();
            }
            self.charge(position, bytes)?;
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
        Ok(())
    }

    /// The imports waiting on `name` in `module`.
    fn waiters(&self, module: usize, name: &str) -> Option<&BTreeSet<usize>> {
        self.waiting.get(&module)?.get(name)
    }

    fn watch(&mut self, position: usize, module: usize) -> Result<(), LoadError> {
        self.step(position, 1)?;
        let watchers = self.watching.get(&module);
        let mut bytes = 0;
        if watchers.is_none() {
            bytes += btree_entry::<usize, BTreeSet<usize>>() + btree_map::<usize, ()>();
            if self.watching.is_empty() {
                bytes += btree_map::<usize, BTreeSet<usize>>();
            }
        }
        if watchers.is_none_or(|watchers| !watchers.contains(&position)) {
            bytes += btree_entry::<usize, ()>();
        }
        self.charge(position, bytes)?;
        self.watching.entry(module).or_default().insert(position);
        Ok(())
    }

    /// Wake everything that waits on a name of `module`, and the globs that
    /// watch it.
    fn wake_module(&mut self, module: usize) -> Result<(), LoadError> {
        let first = self
            .waiting
            .get(&module)
            .into_iter()
            .flat_map(BTreeMap::values)
            .flat_map(BTreeSet::iter)
            .next()
            .or_else(|| {
                self.watching
                    .get(&module)
                    .and_then(|watchers| watchers.first())
            })
            .copied();
        let Some(first) = first else {
            return Ok(());
        };
        self.step(first, 1)?;
        if self.notified[module] {
            return Ok(());
        }
        self.notified[module] = true;
        self.notifications.push((module, first));
        for &position in self
            .waiting
            .get(&module)
            .into_iter()
            .flat_map(BTreeMap::values)
            .flat_map(BTreeSet::iter)
        {
            Self::push_into(
                &self.memory,
                &self.origins,
                &mut self.queue,
                &mut self.queued,
                position,
            )?;
        }
        if let Some(watchers) = self.watching.get(&module) {
            for &position in watchers {
                Self::push_into(
                    &self.memory,
                    &self.origins,
                    &mut self.queue,
                    &mut self.queued,
                    position,
                )?;
            }
        }
        Ok(())
    }

    /// Wake the globs that watch `module`.
    fn wake_watchers(&mut self, module: usize) -> Result<(), LoadError> {
        for &position in self.watching.get(&module).into_iter().flatten() {
            Self::push_into(
                &self.memory,
                &self.origins,
                &mut self.queue,
                &mut self.queued,
                position,
            )?;
        }
        Ok(())
    }
}

impl Waiters for Pending<'_> {
    fn wake(&mut self, module: usize, name: &str) -> Result<(), LoadError> {
        for &position in self
            .waiting
            .get(&module)
            .and_then(|names| names.get(name))
            .into_iter()
            .flatten()
        {
            Self::push_into(
                &self.memory,
                &self.origins,
                &mut self.queue,
                &mut self.queued,
                position,
            )?;
        }
        Ok(())
    }

    fn waiting_on(&self, name: &str) -> Option<&BTreeSet<usize>> {
        self.waiting_by_name.get(name)
    }

    fn waited_in(
        &self,
        module: usize,
        memory: &Transient<'_>,
        file: &RelPath,
        line: usize,
    ) -> Result<Vec<String>, LoadError> {
        let Some(names) = self.waiting.get(&module) else {
            return Ok(Vec::new());
        };
        memory.work(names.len(), file, Some(line))?;
        work::add(Work::SettleSteps, names.len());
        memory.charge(
            vec_first::<String>()
                + names.len() * vec_slot::<String>()
                + names.keys().map(String::len).sum::<usize>(),
            file,
            Some(line),
        )?;
        Ok(names.keys().cloned().collect())
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
    pub(crate) fn visibility_path(written: &str, steps: Steps<'_>) -> Result<Vec<String>, Fail> {
        steps.storage_work(written.len()).map_err(Fail::Storage)?;
        // Whitespace can separate the colons before normalization.
        let count = 1 + written.bytes().filter(|byte| *byte == b':').count() / 2;
        steps
            .reserve(vec_first::<String>() + count * vec_slot::<String>() + 2 * written.len())
            .map_err(Fail::Storage)?;
        let mut normalized = String::with_capacity(written.len());
        for part in written.split_whitespace() {
            normalized.push_str(part);
        }
        Ok(normalized.split("::").map(str::to_owned).collect())
    }

    pub(crate) fn read_visibility(
        &self,
        vis: &Vis,
        home: usize,
        importer: usize,
        steps: Steps<'_>,
    ) -> Result<bool, Fail> {
        Ok(match vis {
            Vis::Public => true,
            Vis::Inherited => self.within(importer, home),
            Vis::Restricted(text) => match text.as_str() {
                "crate" => self.modules[importer].root == self.modules[home].root,
                "self" => self.within(importer, home),
                "super" => self.modules[home]
                    .parent
                    .is_some_and(|parent| self.within(importer, parent)),
                other => {
                    if let Some(written) = other.strip_prefix("in ") {
                        let segs = Self::visibility_path(written, steps)?;
                        match self.walk(home, &segs, false, Namespace::Type, Heads::Local, steps) {
                            Ok(walked) => walked
                                .modules
                                .iter()
                                .copied()
                                .chain(walked.items.keys().flat_map(|id| self.modules_for(*id)))
                                .any(|ancestor| self.within(importer, ancestor)),
                            Err(Fail::Storage(error)) => return Err(Fail::Storage(error)),
                            Err(_) => false,
                        }
                    } else {
                        false
                    }
                }
            },
        })
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
                            companion: false,
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
        self.use_read_state = vec![[ImportReadState::Unread; 3]; self.uses.len()];
        self.has_late_imports = false;
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
    /// [`Index::read_lookup`] for the path of the named import at `own`. An import
    /// never resolves through its own binding: where its path reads the name
    /// it binds in its own module, the explicit imports of that name, its own
    /// among them, are not there for it. They are alternatives under `cfg`,
    /// as rustc refuses two imports of one name, so the path reads the
    /// module's definitions, then its globs.
    /// [`Index::lookup_in`] as a `use` path of `viewer` reads `module`, as
    /// rustc resolves an import: a binding `viewer` cannot access does not
    /// count. A private forwarded named binding can leave an exported peer
    /// namespace's glob family available; a direct binding still hides its
    /// globs. An import still to bind keeps the name from them too.
    /// A name two globs bring in as different items is the one
    /// exception to what `viewer` cannot access: it is ambiguous to a viewer
    /// that can access any of its items, with all of them (see
    /// [`Index::seen_in`]).
    fn lookup_seen(
        &self,
        module: usize,
        ns: Namespace,
        name: &str,
        own: Option<usize>,
        viewer: Option<usize>,
        steps: Steps<'_>,
    ) -> Result<Option<(Cow<'_, Bound>, bool)>, Fail> {
        let Some(viewer) = viewer.filter(|viewer| *viewer != module) else {
            return self.lookup_in_read(module, ns, name, own, steps);
        };
        work::add(Work::Lookups, 1);
        let entry = self.scopes[module].get(name);
        let at = slot(ns);
        let mut hidden = None;
        if let Some(defs) = entry
            .map(|entry| &entry.defs[at])
            .filter(|defs| !defs.is_empty())
        {
            if let Some(visible) =
                self.seen_in(Cow::Borrowed(defs), module, viewer, false, steps)?
            {
                return Ok(Some((visible, false)));
            }
            if self
                .forwarded_peer_export((module, ns, name), defs, own, viewer, steps)?
                .is_none()
            {
                return Ok(None);
            }
            hidden = Some(defs);
        } else if self.blocked(module, name, ns, own) {
            return Ok(None);
        }
        let globbed = self.read_globbed(module, name, at, steps)?;
        // A named reexport of the same unique item does not replace its
        // private binding. An ordinary glob source can still carry its peer
        // family; preserve every actual route, not just the preferred Origin.
        if let Some(defs) = hidden.filter(|_| globbed.len() == 1) {
            steps.storage_work(defs.len()).map_err(Fail::Storage)?;
            if let Some(id) = defs.keys().find(|id| globbed.get(**id).is_some())
                && !self.unnamed_glob_source(module, name, at, *id, viewer, steps)?
            {
                return Ok(None);
            }
        }
        Ok(self
            .seen_in(Cow::Owned(globbed), module, viewer, true, steps)?
            .map(|bound| (bound, true)))
    }

    /// An external Type/Value import can keep an accessible namespace family
    /// while its private named peer forwards another import. This does not
    /// change direct path lookup, direct private sources or unresolved peers.
    pub(crate) fn forwarded_peer_export(
        &self,
        place: (usize, Namespace, &str),
        defs: &Bound,
        own: Option<usize>,
        viewer: usize,
        steps: Steps<'_>,
    ) -> Result<Option<(Namespace, ItemId)>, Fail> {
        let (module, ns, name) = place;
        let peer = match ns {
            Namespace::Type => Namespace::Value,
            Namespace::Value => Namespace::Type,
            Namespace::Macro => return Ok(None),
        };
        steps
            .storage_work(self.imports_of(module, name).len())
            .map_err(Fail::Storage)?;
        if self.bound_by_undecided(module, peer, name) {
            return Ok(None);
        }
        let read = self.read_lookup(module, peer, name, own, None, steps)?;
        let Some((bound, _)) = read.as_ref().filter(|(bound, _)| bound.len() == 1) else {
            return Ok(None);
        };
        let (id, origin) = bound.iter().next().expect("unique peer");
        if origin.companion || !self.origin_visible(origin, module, viewer, steps)? {
            return Ok(None);
        }
        Ok(self.forwarded_sources(defs, steps)?.then_some((peer, *id)))
    }

    /// Each private definition forwards an already bound, determinate import.
    fn forwarded_sources(&self, defs: &Bound, steps: Steps<'_>) -> Result<bool, Fail> {
        steps.storage_work(defs.len()).map_err(Fail::Storage)?;
        for (id, origin) in defs.iter() {
            let Some(source) = origin.link.as_ref().and_then(|link| link.source.as_ref()) else {
                return Ok(false);
            };
            steps
                .storage_work(self.imports_of(source.module, &source.name).len())
                .map_err(Fail::Storage)?;
            if self.bound_by_undecided(source.module, source.ns, &source.name) {
                return Ok(false);
            }
            let source_read =
                self.read_lookup(source.module, source.ns, &source.name, None, None, steps)?;
            if !source_read
                .as_ref()
                .and_then(|(bound, _)| bound.get(*id))
                .is_some_and(|origin| origin.link.is_some() && !origin.companion)
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Prepare the same lookup with its owned scratch reserved from the
    /// actual source records. The guard remains live while the walker
    /// copies hits into its separately reserved retained result.
    pub(crate) fn read_lookup<'i, 'm>(
        &'i self,
        module: usize,
        ns: Namespace,
        name: &str,
        own: Option<usize>,
        viewer: Option<usize>,
        steps: Steps<'m>,
    ) -> Result<LookupRead<'i, 'm>, Fail> {
        let memory = steps.temporary();
        let bounded = memory.as_ref().map_or(steps, |memory| steps.at(memory));
        if memory.is_some() {
            let defs = self.scopes[module]
                .get(name)
                .map(|entry| &entry.defs[slot(ns)]);
            if let Some(defs) = defs.filter(|defs| !defs.is_empty()) {
                bounded.storage_work(defs.len()).map_err(Fail::Storage)?;
                let exclusion = own.is_some_and(|position| self.binds_name(position, module, name))
                    && defs.iter().any(|(_, origin)| origin.link.is_some());
                let mut partial = false;
                if let Some(viewer) = viewer.filter(|viewer| *viewer != module) {
                    for (_, origin) in defs.iter() {
                        if !self.origin_visible(origin, module, viewer, bounded)? {
                            partial = true;
                            break;
                        }
                    }
                }
                if exclusion || partial {
                    let bytes = vec_first::<(ItemId, Origin)>()
                        + defs
                            .iter()
                            .map(|(_, origin)| {
                                vec_slot::<(ItemId, Origin)>() + origin.clone_bytes()
                            })
                            .sum::<usize>();
                    bounded.reserve(bytes).map_err(Fail::Storage)?;
                }
                // The import's own named bindings do not shadow its path.
                // Removing every linked definition falls through to globs,
                // whose owned buffers must coexist with this local scratch.
                if partial || exclusion && defs.iter().all(|(_, origin)| origin.link.is_some()) {
                    self.reserve_glob_lookup(module, name, slot(ns), bounded)?;
                }
            } else if !self.blocked(module, name, ns, own) {
                self.reserve_glob_lookup(module, name, slot(ns), bounded)?;
            }
        }
        let bound = self.lookup_seen(module, ns, name, own, viewer, bounded)?;
        Ok(LookupRead {
            bound,
            _memory: memory,
        })
    }

    /// The bindings of `bound`, in `module`, that `viewer` can access, or
    /// `None` when it can access none.
    fn origin_visible(
        &self,
        origin: &Origin,
        module: usize,
        viewer: usize,
        steps: Steps<'_>,
    ) -> Result<bool, Fail> {
        for vis in origin.visibilities() {
            steps.storage_work(1).map_err(Fail::Storage)?;
            if self.read_visibility(vis, module, viewer, steps)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn seen_in<'a>(
        &self,
        bound: Cow<'a, Bound>,
        module: usize,
        viewer: usize,
        globs: bool,
        steps: Steps<'_>,
    ) -> Result<Option<Cow<'a, Bound>>, Fail> {
        let mut seen = 0;
        for (_, origin) in bound.iter() {
            if self.origin_visible(origin, module, viewer, steps)? {
                seen += 1;
            }
        }
        if seen == bound.len() || globs && ambiguity_seen(bound.len(), seen) {
            return Ok((!bound.is_empty()).then_some(bound));
        }
        let mut kept = Bound::new();
        for (id, origin) in bound.iter() {
            if self.origin_visible(origin, module, viewer, steps)? {
                kept.offer_work(*id, origin, steps)?;
                kept.offer(*id, origin.clone());
            }
        }
        Ok((!kept.is_empty()).then_some(Cow::Owned(kept)))
    }

    pub(crate) fn lookup_in(
        &self,
        module: usize,
        ns: Namespace,
        name: &str,
        own: Option<usize>,
        steps: Steps<'_>,
    ) -> Option<(Cow<'_, Bound>, bool)> {
        self.lookup_in_read(module, ns, name, own, steps)
            .expect("nonretained lookup reserves no import storage")
    }

    pub(crate) fn lookup_in_read(
        &self,
        module: usize,
        ns: Namespace,
        name: &str,
        own: Option<usize>,
        steps: Steps<'_>,
    ) -> Result<Option<(Cow<'_, Bound>, bool)>, Fail> {
        work::add(Work::Lookups, 1);
        let entry = self.scopes[module].get(name);
        let at = slot(ns);
        if own.is_some_and(|position| self.binds_name(position, module, name)) {
            if let Some(entry) = entry {
                let defs = &entry.defs[at];
                if defs.iter().all(|(_, origin)| origin.link.is_none()) {
                    if !defs.is_empty() {
                        return Ok(Some((Cow::Borrowed(defs), false)));
                    }
                } else {
                    let local: Bound = defs
                        .iter()
                        .filter(|(_, origin)| origin.link.is_none())
                        .map(|(id, origin)| (*id, origin.clone()))
                        .collect();
                    if !local.is_empty() {
                        return Ok(Some((Cow::Owned(local), false)));
                    }
                }
            }
        } else {
            if let Some(defs) = entry
                .map(|entry| &entry.defs[at])
                .filter(|defs| !defs.is_empty())
            {
                return Ok(Some((Cow::Borrowed(defs), false)));
            }
            if self.blocked(module, name, ns, own) {
                return Ok(None);
            }
        }
        let globbed = self.read_globbed(module, name, at, steps)?;
        Ok((!globbed.is_empty()).then_some((Cow::Owned(globbed), true)))
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
    fn read_blocked_for(
        &self,
        module: usize,
        name: &str,
        at: usize,
        importer: usize,
        steps: Steps<'_>,
    ) -> Result<bool, Fail> {
        for &position in self.imports_of(module, name) {
            steps.storage_work(1).map_err(Fail::Storage)?;
            if (!self.use_free[position][at] || self.left_to_globs(position, at))
                && (self.read_visibility(&self.uses[position].vis, module, importer, steps)?
                    || self.read_globbed_any(module, name, at, steps)?)
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// True when a glob of `module`, one that `importer` can name when one is
    /// given, other than the one at `except`, may still bring in `name`, or
    /// more of it, in the name space at `at`: a module its globs reach,
    /// directly or through the globs of modules that bind nothing there,
    /// has an explicit import that keeps the name from the globs. A cycle of
    /// globs brings in nothing of its own. A glob that is not settled may
    /// bring in any name; that is asked apart ([`Index::open_glob`]).
    pub(crate) fn read_pending_at(
        &self,
        module: usize,
        name: &str,
        at: usize,
        importer: Option<usize>,
        except: Option<usize>,
        steps: Steps<'_>,
    ) -> Result<bool, Fail> {
        let memory = steps.temporary();
        let steps = memory.as_ref().map_or(steps, |memory| steps.at(memory));
        // Only an import keeps a name from the globs: with none that the
        // globs of the module reach, none may still bring it in.
        if !self.reaches_import(module) {
            return Ok(false);
        }
        let mut seen = BTreeSet::new();
        insert_module(&mut seen, module, steps)?;
        let stack_memory = steps.query_meter().map(Transient::new);
        let (stack_file, stack_line) = self.module_site(module);
        let stack_steps =
            steps.with_query_storage(stack_memory.as_ref(), (&stack_file, stack_line));
        let mut work = WorkList::<_, 8>::new();
        push_work(&mut work, (module, importer, except), stack_steps).map_err(Fail::Storage)?;
        while let Some((from, viewer, skip)) = work.pop() {
            for glob in self.globs_of(from) {
                steps.storage_work(1).map_err(Fail::Storage)?;
                if skip == Some(*glob) {
                    continue;
                }
                if let Some(to) = viewer
                    && !self.read_visibility(&self.uses[*glob].vis, from, to, steps)?
                {
                    continue;
                }
                // The modules the glob reached when it was last tried: a
                // glob's path is not walked here, as walking it may ask this
                // again.
                for &target in &self.glob_reach[*glob] {
                    steps.storage_work(1).map_err(Fail::Storage)?;
                    if !self.reaches_import(target)
                        || self.scopes[target]
                            .get(name)
                            .is_some_and(|entry| !entry.defs[at].is_empty())
                    {
                        continue;
                    }
                    if self.read_blocked_for(target, name, at, from, steps)? {
                        return Ok(true);
                    }
                    if !seen.contains(&target) {
                        insert_module(&mut seen, target, steps)?;
                        push_work(&mut work, (target, Some(from), None), stack_steps)
                            .map_err(Fail::Storage)?;
                    }
                }
            }
        }
        Ok(false)
    }

    /// True when a glob of `module` that `importer` can name, when one is
    /// given, is not settled.
    fn open_glob(&self, module: usize, importer: Option<usize>, steps: Steps<'_>) -> bool {
        self.read_open_glob(module, importer, steps)
            .expect("nonretained open-glob checks reserve no import storage")
    }

    fn read_open_glob(
        &self,
        module: usize,
        importer: Option<usize>,
        steps: Steps<'_>,
    ) -> Result<bool, Fail> {
        for glob in self.globs_of(module) {
            steps.storage_work(1).map_err(Fail::Storage)?;
            if !self.glob_closed[*glob]
                && (importer.is_none()
                    || self.read_visibility(
                        &self.uses[*glob].vis,
                        module,
                        importer.expect("an importer"),
                        steps,
                    )?)
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// True when what `target` binds to `name` in the name space at `at` is
    /// not settled for `importer`, which globs it: it binds nothing there
    /// yet, and an explicit import or a glob of it may still.
    /// True when the walk that found `source` unbound waits on an import that
    /// never settles: one that keeps the name there from the globs, or, when
    /// a glob may still bring it in, one that keeps it from a module the glob
    /// reaches. The settling of the tables leaves such imports only in a
    /// cycle and among those that wait on one.
    pub(crate) fn unsettled_at(
        &self,
        source: &Source,
        steps: Steps<'_>,
    ) -> Result<Option<Unsettled>, LoadError> {
        let stack_memory = steps.query_meter().map(Transient::new);
        let (stack_file, stack_line) = self.module_site(source.module);
        let stack_steps =
            steps.with_query_storage(stack_memory.as_ref(), (&stack_file, stack_line));
        let mut seen: BTreeSet<NamePlace> = BTreeSet::new();
        let mut work = WorkList::<_, 8>::new();
        stack_steps.reserve(source.name.len())?;
        push_work(
            &mut work,
            (source.module, source.name.clone(), slot(source.ns)),
            stack_steps,
        )?;
        while let Some((module, name, at)) = work.pop() {
            if !seen.insert((module, name.clone(), at)) {
                continue;
            }
            if let Some(position) = self
                .imports_of(module, &name)
                .iter()
                .find(|position| self.use_read_state[**position][at] == ImportReadState::Late)
            {
                return Ok(Some(Unsettled::LateImport(*position)));
            }
            if self.open_glob(module, None, steps)
                || self
                    .imports_of(module, &name)
                    .iter()
                    .any(|position| !self.use_decided[*position][at])
            {
                return Ok(Some(Unsettled::Cycle));
            }
            for place in self.pending_sources(module, &name, at, steps) {
                push_work(&mut work, place, stack_steps)?;
            }
        }
        Ok(None)
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
    ) -> Vec<NamePlace> {
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
    ) -> Vec<NamePlace> {
        self.read_pending_sources_for(module, name, at, viewer, steps)
            .expect("nonretained source traversal reserves no import storage")
    }

    fn read_pending_sources_for(
        &self,
        module: usize,
        name: &str,
        at: usize,
        viewer: Option<usize>,
        steps: Steps<'_>,
    ) -> Result<Vec<NamePlace>, Fail> {
        let mut sources = Vec::new();
        if self.scopes[module].get(name).is_some_and(|entry| {
            entry.defs[at]
                .iter()
                .any(|(_, origin)| origin.link.is_none())
        }) {
            return Ok(sources);
        }
        for glob in self.globs_of(module) {
            steps.storage_work(1).map_err(Fail::Storage)?;
            if let Some(viewer) = viewer
                && !self.read_visibility(&self.uses[*glob].vis, module, viewer, steps)?
            {
                continue;
            }
            for &target in &self.glob_reach[*glob] {
                steps.storage_work(1).map_err(Fail::Storage)?;
                if self.scopes[target]
                    .get(name)
                    .is_none_or(|entry| entry.defs[at].is_empty())
                    && (self.read_blocked_for(target, name, at, module, steps)?
                        || self.read_pending_at(target, name, at, Some(module), None, steps)?)
                {
                    steps
                        .reserve(
                            vec_slot::<NamePlace>()
                                + name.len()
                                + if sources.is_empty() {
                                    vec_first::<NamePlace>()
                                } else {
                                    0
                                },
                        )
                        .map_err(Fail::Storage)?;
                    sources.push((target, name.to_owned(), at));
                }
            }
        }
        Ok(sources)
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
        self.module_item_ids(module).collect()
    }

    pub(crate) fn module_item_ids(&self, module: usize) -> impl Iterator<Item = ItemId> + '_ {
        let path = &self.modules[module].path;
        let name = path.rsplit("::").next().unwrap_or(path);
        let root = self.modules[module]
            .parent
            .is_none()
            .then(|| self.root_items.get(&module))
            .flatten()
            .copied();
        let defs = self.modules[module]
            .parent
            .and_then(|parent| self.scopes[parent].get(name));
        root.into_iter().chain(
            defs.into_iter()
                .flat_map(|entry| entry.defs[slot(Namespace::Type)].keys().copied())
                .filter(move |id| self.module_of.get(id) == Some(&module)),
        )
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
    pub(crate) fn glob_targets_noting(
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
        for module in walked.items.keys().flat_map(|id| self.modules_for(*id)) {
            insert_module(&mut targets, module, steps)?;
        }
        // A glob of a name that stands for several crates brings in nothing.
        self.one_crate(&targets, segs.len(), steps)?;
        targets.remove(&decl.module);
        if targets.is_empty() {
            // Removal may retain an empty tree root. Drop it while the
            // construction lease still owns the original allocation grant.
            drop(std::mem::take(&mut targets));
        }
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
            companion: false,
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
        if matches!(self.uses[position].kind, UseKind::Named { .. }) {
            self.touch_named_import(position, pending, meter)?;
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
        pending.wake(module, name)?;
        if self.exports_by_path(module) {
            pending.wake_watchers(module)?;
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
        let mut pending = Pending::new(&self.uses, meter)?;
        for position in 0..self.uses.len() {
            pending.push(position)?;
        }
        // First, a `use` path whose first segment a module with globs does
        // not bind waits: a glob may still bring the name in. Then the globs
        // have brought in what they can, and such a path may start at a
        // crate of the extern prelude; every import of a module with globs
        // is tried again, as one that bound a name in one name space may now
        // bind it in another. The path of an import of a module without
        // globs reads the same whether the crates are open or not.
        self.drain(&mut pending, meter, true)?;
        self.crates_open = true;
        for position in 0..self.uses.len() {
            if !self.globs_of(self.uses[position].module).is_empty() {
                pending.push(position)?;
            }
        }
        let mut rounds = 0;
        let mut ordinary = true;
        loop {
            self.drain(&mut pending, meter, ordinary)?;
            if ordinary && !self.has_deferred_privacy(meter)? {
                ordinary = false;
            }
            let before = self.progress();
            let changed = self.settle_stuck(&mut pending, meter, ordinary)?;
            if !changed && ordinary {
                ordinary = false;
                self.release_privacy_reads(&mut pending, meter)?;
                continue;
            }
            rounds += 1;
            if !changed {
                break;
            }
            self.advanced(rounds, before)?;
        }
        if !self.fixed_glob_paths(meter)? {
            self.rebuild_globs(meter)?;
        }
        self.finalize_privacy_reads(meter)?;
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
    fn settle_stuck(
        &mut self,
        pending: &mut Pending,
        meter: &Meter,
        ordinary: bool,
    ) -> Result<bool, LoadError> {
        let graph = self.blocker_graph(meter)?;
        if graph.imports.is_empty() {
            return Ok(false);
        }
        let memory = graph.memory.child();
        let decl = &self.uses[graph.origin(0)];
        memory.charge(decl.file.as_str().len(), &decl.file, Some(decl.line))?;
        let (file, line) = (decl.file.clone(), decl.line);
        let steps = Steps::LoadAt {
            memory: &memory,
            file: &file,
            line,
        };
        let found = read_components(&graph.successors, steps)?;
        let nodes = graph.successors.len();
        let edges: usize = graph.successors.iter().map(Vec::len).sum();
        // Every condensation edge comes from an actual graph edge. A node
        // belongs to one component and one wave; old and next waves overlap.
        steps.reserve(
            6 * vec_first::<usize>()
                + vec_first::<Decided>()
                + vec_first::<BTreeSet<usize>>()
                + vec_first::<Vec<usize>>()
                + vec_first::<Act>()
                + nodes
                    * (7 * vec_slot::<usize>()
                        + vec_slot::<Decided>()
                        + vec_slot::<BTreeSet<usize>>()
                        + btree_map::<usize, ()>()
                        + vec_slot::<Vec<usize>>()
                        + vec_first::<usize>()
                        + 4 * vec_slot::<Act>())
                + edges * (btree_entry::<usize, ()>() + vec_slot::<usize>()),
        )?;
        steps.storage_work(8 * nodes + 4 * edges)?;
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
                    self.decide(
                        &graph, &found[at], pending, &mut acts, steps, meter, ordinary,
                    )?
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
            self.drain(pending, meter, ordinary)?;
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
    fn blocker_graph<'m>(&self, meter: &'m Meter) -> Result<Blockers<'m>, LoadError> {
        let memory = Transient::new(meter);
        let members = self.graph_members(&memory)?;
        if let Some((position, _)) = members.first() {
            let decl = &self.uses[*position];
            memory.charge(
                btree_map::<(usize, Option<usize>), usize>()
                    + members.len() * btree_entry::<(usize, Option<usize>), usize>()
                    + vec_first::<Vec<usize>>()
                    + members.len() * vec_slot::<Vec<usize>>(),
                &decl.file,
                Some(decl.line),
            )?;
        }
        let node_of: BTreeMap<(usize, Option<usize>), usize> = members
            .iter()
            .enumerate()
            .map(|(node, member)| (*member, node))
            .collect();
        let mut graph = Blockers {
            memory,
            places: Vec::new(),
            origins: Vec::new(),
            place_of: BTreeMap::new(),
            successors: vec![Vec::new(); members.len()],
            imports: members,
        };
        let mut work = WorkList::<usize, 8>::new();
        for node in 0..graph.imports.len() {
            let position = graph.imports[node].0;
            self.import_node(&mut graph, node, &node_of, &mut work)?;
            self.settle_lookups(meter, position)?;
        }
        while let Some(node) = work.pop() {
            let origin = graph.origin(node);
            self.place_successors(&mut graph, node, &node_of, &mut work)?;
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
        work: &mut WorkList<usize, 8>,
    ) -> Result<(), LoadError> {
        let (position, at) = graph.imports[node];
        let decl = &self.uses[position];
        graph.step(decl, 1)?;
        let memory = graph.memory.child();
        let steps = Steps::LoadAt {
            memory: &memory,
            file: &decl.file,
            line: decl.line,
        };
        if let Some(at) = at {
            for (module, name, ns, head) in self.read_places_in(position, at, steps)? {
                if !self.binds_name(position, module, &name) {
                    let viewer = (module != decl.module).then_some(decl.module);
                    let next = graph.place(
                        (module, name, ns, None, head, viewer),
                        work,
                        position,
                        decl,
                    )?;
                    graph.edge(node, next, decl)?;
                    continue;
                }
                // The name it binds itself, which it reads as rustc
                // does, without itself: what the globs of its module may
                // bring in.
                for glob in self.globs_of(module) {
                    if !head && !self.glob_closed[*glob] {
                        graph.edge(node, node_of[&(*glob, None)], decl)?;
                    }
                }
                for (target, name, ns) in self
                    .read_pending_sources_for(module, &name, ns, None, steps)
                    .map_err(storage_failure)?
                {
                    let next = graph.place(
                        (target, name, ns, None, false, Some(module)),
                        work,
                        position,
                        decl,
                    )?;
                    graph.edge(node, next, decl)?;
                }
            }
            return Ok(());
        }
        // A glob waits on the places its path reads, or once it resolves
        // on the globs of its targets that are not settled.
        let mut reads = Vec::new();
        let targets = self.glob_targets_noting(position, Some(&mut reads), steps);
        if let Err(Fail::Storage(error)) = targets {
            return Err(error);
        }
        if let Ok(targets) = targets {
            for target in targets {
                for glob in self.globs_of(target) {
                    if !self.glob_closed[*glob]
                        && self
                            .read_visibility(&self.uses[*glob].vis, target, decl.module, steps)
                            .map_err(storage_failure)?
                    {
                        graph.edge(node, node_of[&(*glob, None)], decl)?;
                    }
                }
            }
        } else {
            // A place of its own module is read without the glob. The
            // first segment read under the name of a crate of the extern
            // prelude waits on no glob of the module, as for a named
            // import (see [`Index::read_places_in`]): two globs of
            // dependencies do not wait on each other.
            steps.reserve(btree_map::<Place, ()>() + reads.len() * btree_entry::<Place, ()>())?;
            steps.storage_work(reads.len())?;
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
                let next = graph.place(place, work, position, decl)?;
                graph.edge(node, next, decl)?;
            }
        }
        Ok(())
    }

    /// The successors of the place at `node` of `graph`: what it waits on
    /// when nothing its reader can access binds it (see
    /// [`Index::blocker_graph`]). New places are queued in `work`.
    fn place_successors(
        &self,
        graph: &mut Blockers,
        node: usize,
        node_of: &BTreeMap<(usize, Option<usize>), usize>,
        work: &mut WorkList<usize, 8>,
    ) -> Result<(), LoadError> {
        let position = graph.origin(node);
        let site = &self.uses[position];
        graph.step(site, 1)?;
        let temporary = graph.memory.child();
        temporary.charge(
            graph.places[node - graph.imports.len()].1.len(),
            &site.file,
            Some(site.line),
        )?;
        let (module, name, at, except, head, viewer) =
            graph.places[node - graph.imports.len()].clone();
        // What `viewer`, the module whose path reads the place, cannot
        // access does not count, as rustc resolves an import.
        let steps = Steps::LoadAt {
            memory: &temporary,
            file: &site.file,
            line: site.line,
        };
        if self
            .read_lookup(module, Namespace::ALL[at], &name, except, viewer, steps)
            .map_err(storage_failure)?
            .as_ref()
            .is_none()
        {
            for position in self.imports_of(module, &name) {
                if let Some(member) = node_of.get(&(*position, Some(at)))
                    && !self.use_free[*position][at]
                {
                    graph.edge(node, *member, site)?;
                }
            }
            for glob in self.globs_of(module) {
                if !head
                    && !self.glob_closed[*glob]
                    && except != Some(*glob)
                    && (viewer.is_none()
                        || self
                            .read_visibility(
                                &self.uses[*glob].vis,
                                module,
                                viewer.expect("a viewer"),
                                steps,
                            )
                            .map_err(storage_failure)?)
                {
                    graph.edge(node, node_of[&(*glob, None)], site)?;
                }
            }
            for (target, name, at) in self
                .read_pending_sources_for(module, &name, at, viewer, steps)
                .map_err(storage_failure)?
            {
                let next = graph.place(
                    (target, name, at, None, false, Some(module)),
                    work,
                    position,
                    site,
                )?;
                graph.edge(node, next, site)?;
            }
        }
        Ok(())
    }

    /// The members of the blocker graph: each named import that is not
    /// settled, once for each name space it is undecided in, then each glob
    /// that is not settled, without a name space.
    fn graph_members(
        &self,
        memory: &Transient<'_>,
    ) -> Result<Vec<(usize, Option<usize>)>, LoadError> {
        let mut members: Vec<(usize, Option<usize>)> = Vec::new();
        for &position in &self.named_imports {
            if self.use_settled[position] {
                continue;
            }
            for (at, decided) in self.use_decided[position].iter().enumerate() {
                if !decided {
                    let decl = &self.uses[position];
                    memory.work(1, &decl.file, Some(decl.line))?;
                    work::add(Work::SettleSteps, 1);
                    memory.charge(
                        vec_slot::<(usize, Option<usize>)>()
                            + if members.is_empty() {
                                vec_first::<(usize, Option<usize>)>()
                            } else {
                                0
                            },
                        &decl.file,
                        Some(decl.line),
                    )?;
                    members.push((position, Some(at)));
                }
            }
        }
        for &position in self
            .glob_imports
            .iter()
            .filter(|position| !self.glob_closed[**position])
        {
            let decl = &self.uses[position];
            memory.work(1, &decl.file, Some(decl.line))?;
            work::add(Work::SettleSteps, 1);
            memory.charge(
                vec_slot::<(usize, Option<usize>)>()
                    + if members.is_empty() {
                        vec_first::<(usize, Option<usize>)>()
                    } else {
                        0
                    },
                &decl.file,
                Some(decl.line),
            )?;
            members.push((position, None));
        }
        Ok(members)
    }

    /// The places the path of the named import at `position` reads when it
    /// is walked in the name space at `at`, the name it binds itself among
    /// them when it reads it.
    ///
    /// Each place is marked when it is the first segment of the path read in
    /// its own module under the name of a crate of the extern prelude: a glob
    /// of the module that is not settled does not hold such a path back (see
    /// [`Index::unbound_use_head`]), so the place does not wait on one.
    fn read_places_in(
        &self,
        position: usize,
        at: usize,
        steps: Steps<'_>,
    ) -> Result<BTreeSet<(usize, String, usize, bool)>, LoadError> {
        let decl = &self.uses[position];
        let UseKind::Named { segs, .. } = &decl.kind else {
            return Ok(BTreeSet::new());
        };
        let mut reads = Vec::new();
        let walked = self.walk_noting(
            decl.module,
            segs,
            decl.leading_colon,
            Namespace::ALL[at],
            Heads::Use {
                test: decl.test,
                own: Some(position),
            },
            Some(&mut reads),
            steps,
        );
        if let Err(Fail::Storage(error)) = walked {
            return Err(error);
        }
        steps.reserve(
            btree_map::<(usize, String, usize, bool), ()>()
                + reads.len() * btree_entry::<(usize, String, usize, bool), ()>(),
        )?;
        steps.storage_work(reads.len())?;
        Ok(reads
            .into_iter()
            .map(|source| {
                let head =
                    source.module == decl.module && self.prelude_head(position, &source.name);
                (source.module, source.name, slot(source.ns), head)
            })
            .collect())
    }

    /// True when the path of the glob import at `position` resolves, or
    /// names nothing only at its first segment, read in the module where the
    /// path starts.
    fn held_at_head(&self, position: usize, steps: Steps<'_>) -> Result<bool, LoadError> {
        let decl = &self.uses[position];
        let UseKind::Glob { segs } = &decl.kind else {
            return Ok(false);
        };
        let Some(head) = segs.first() else {
            return Ok(false);
        };
        let sources = match self.glob_targets(position, steps) {
            Ok(_) => return Ok(true),
            Err(Fail::Missing(sources)) => sources,
            Err(Fail::Storage(error)) => return Err(error),
            Err(_) => return Ok(false),
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
        Ok(!decl.leading_colon
            && !matches!(head.as_str(), "crate" | "self" | "super")
            && !sources.is_empty()
            && sources
                .iter()
                .all(|source| source.module == start && source.name == *head))
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
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        let memory = steps.temporary();
        let steps = memory.as_ref().map_or(steps, |memory| steps.at(memory));
        let (module, name, at, head) = place;
        steps.reserve(btree_map::<(usize, String, usize, usize), ()>())?;
        let mut seen = BTreeSet::new();
        let stack_memory = steps.query_meter().map(Transient::new);
        let (stack_file, stack_line) = self.module_site(module);
        let stack_steps =
            steps.with_query_storage(stack_memory.as_ref(), (&stack_file, stack_line));
        let mut work = WorkList::<_, 8>::new();
        push_work(&mut work, (module, name, at, head, reader), stack_steps)?;
        while let Some((module, name, at, head, viewer)) = work.pop() {
            steps.storage_work(1 + self.imports_of(module, &name).len())?;
            steps.reserve(btree_entry::<(usize, String, usize, usize), ()>() + name.len())?;
            if !seen.insert((module, name.clone(), at, viewer)) {
                continue;
            }
            if self
                .read_lookup(module, Namespace::ALL[at], &name, None, Some(viewer), steps)
                .map_err(storage_failure)?
                .as_ref()
                .is_some()
            {
                continue;
            }
            if self.imports_of(module, &name).iter().any(|position| {
                !self.use_decided[*position][at]
                    && !self.use_free[*position][at]
                    && !inside.contains(position)
            }) {
                return Ok(true);
            }
            if !head {
                for glob in self.globs_of(module) {
                    steps.storage_work(1)?;
                    if !self.glob_closed[*glob]
                        && !inside.contains(glob)
                        && (module == viewer
                            || self
                                .read_visibility(&self.uses[*glob].vis, module, viewer, steps)
                                .map_err(storage_failure)?)
                    {
                        return Ok(true);
                    }
                }
            }
            let next = self
                .read_pending_sources_for(module, &name, at, Some(viewer), steps)
                .map_err(storage_failure)?;
            for (target, name, at) in next {
                push_work(&mut work, (target, name, at, false, module), stack_steps)?;
            }
        }
        Ok(false)
    }

    /// Decide one component of the blocker graph, whose successors are all
    /// decided, adding what it does to `acts` (see [`Index::settle_stuck`]).
    #[expect(
        clippy::too_many_arguments,
        reason = "one component decision with its pending reads, phase and admitted work"
    )]
    fn decide(
        &self,
        graph: &Blockers,
        component: &[usize],
        pending: &Pending,
        acts: &mut Vec<Act>,
        steps: Steps<'_>,
        meter: &Meter,
        ordinary: bool,
    ) -> Result<Decided, LoadError> {
        let scratch = steps.temporary();
        let reads = scratch.as_ref().map_or(steps, |memory| steps.at(memory));
        reads.reserve(
            btree_map::<usize, Vec<usize>>()
                + btree_map::<usize, ()>()
                + vec_first::<usize>()
                + component.len()
                    * (btree_entry::<usize, Vec<usize>>()
                        + btree_entry::<usize, ()>()
                        + vec_first::<usize>()
                        + 3 * vec_slot::<usize>()),
        )?;
        reads.storage_work(component.len())?;
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
        if self.defer_component(ordinary, looped, &imports, reads)? {
            return Ok(Decided::Cyclic);
        }
        if !globs.is_empty() {
            // A glob that waits on nothing unsettled is settled: its path
            // reaches what it reaches. Globs that only wait on one another,
            // each of whose paths resolved, are settled together. A glob in a
            // cycle with imports, or one whose path waits on a cycle, is not.
            let mut resolved = true;
            let mut at_heads = true;
            for position in &globs {
                match self.glob_targets(*position, reads) {
                    Err(Fail::Storage(error)) => return Err(error),
                    Err(_) => resolved = false,
                    Ok(_) => (),
                }
                at_heads &= self.held_at_head(*position, reads)?;
            }
            if !imports.is_empty() || (looped && !resolved && !at_heads) {
                return Ok(Decided::Cyclic);
            }
            acts.extend(globs.into_iter().map(Act::CloseGlob));
            return Ok(Decided::Done);
        }
        let inside: BTreeSet<usize> = imports.keys().copied().collect();
        for (&position, spaces) in &imports {
            for &at in spaces {
                for place in self.read_places_in(position, at, reads)? {
                    if self.waits_outside(place, self.uses[position].module, &inside, reads)? {
                        return Ok(Decided::Again);
                    }
                }
            }
        }
        if imports.is_empty() {
            return Ok(Decided::Done);
        }
        if imports.len() == 1 {
            let (position, spaces) = imports.iter().next().expect("one import");
            let result =
                self.decide_one((*position, spaces), &inside, looped, pending, acts, reads)?;
            if result == Decided::Cyclic
                && looped
                && let [at] = spaces.as_slice()
                && self.preserved_shadow(*position, *at, meter, reads)?
            {
                acts.push(Act::PreserveShadow(*position, *at));
                return Ok(Decided::Done);
            }
            return Ok(result);
        }
        let fallbacks = self.cycle_fallbacks(graph, component, &imports, reads)?;
        if let Some(fallbacks) = fallbacks {
            if self.cycle_falls_back(graph, component, &imports, &fallbacks, acts, steps)? {
                return Ok(Decided::Again);
            }
            // Each import needs a concrete answer in another namespace,
            // including an independently rooted answer that its free slot
            // leaves to the globs instead of binding privately.
            if fallbacks.is_empty()
                && let Some(certified) = self.grounded_cycle_imports(&imports, meter, reads)?
            {
                let result = self.decide_empty_cycle(&imports, &inside, pending, acts, reads)?;
                if result == Decided::Done {
                    acts.extend(certified.into_iter().map(Act::FinalizeFallback));
                }
                return Ok(result);
            }
        }
        Ok(Decided::Cyclic)
    }

    /// Apply the ordinary per-member absence proof before releasing any
    /// namespace through a concrete other-namespace fallback certificate.
    fn decide_empty_cycle(
        &self,
        imports: &BTreeMap<usize, Vec<usize>>,
        inside: &BTreeSet<usize>,
        pending: &Pending,
        acts: &mut Vec<Act>,
        steps: Steps<'_>,
    ) -> Result<Decided, LoadError> {
        let memory = steps.temporary();
        let steps = memory.as_ref().map_or(steps, |memory| steps.at(memory));
        steps.reserve(vec_first::<Act>() + imports.len() * vec_slot::<Act>())?;
        let mut decided = Vec::new();
        for (&position, spaces) in imports {
            if self.decide_one(
                (position, spaces),
                inside,
                true,
                pending,
                &mut decided,
                steps,
            )? != Decided::Done
            {
                return Ok(Decided::Cyclic);
            }
        }
        for action in &decided {
            if let Act::Settle(position, _, empty) = action
                && !self.use_bound[*position].iter().any(|bound| *bound)
                && (0..3)
                    .filter(|at| !self.use_empty[*position][*at] && !empty[*at])
                    .count()
                    != 1
            {
                return Ok(Decided::Cyclic);
            }
        }
        acts.extend(decided);
        Ok(Decided::Done)
    }

    /// Ordinary propagation cannot decide a named cycle or frozen privacy.
    fn defer_component(
        &self,
        ordinary: bool,
        looped: bool,
        imports: &BTreeMap<usize, Vec<usize>>,
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        if !ordinary {
            return Ok(false);
        }
        steps.storage_work(imports.values().map(Vec::len).sum())?;
        Ok(looped && !imports.is_empty()
            || imports.iter().any(|(position, spaces)| {
                spaces
                    .iter()
                    .any(|at| self.use_read_state[*position][*at] == ImportReadState::Deferred)
            }))
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
        fallbacks: &[NamePlace],
        acts: &mut Vec<Act>,
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        let mut any = false;
        for at in 0..3 {
            let mut here = fallbacks.iter().filter(|place| place.2 == at);
            let (Some((module, name, _)), None) = (here.next(), here.next()) else {
                continue;
            };
            steps.reserve(
                vec_first::<usize>()
                    + self.imports_of(*module, name).len() * vec_slot::<usize>()
                    + name.len(),
            )?;
            steps.storage_work(self.imports_of(*module, name).len())?;
            let keepers: Vec<usize> = self
                .imports_of(*module, name)
                .iter()
                .copied()
                .filter(|position| imports.contains_key(position))
                .collect();
            // rustc reads the place through the glob only while it resolves
            // an import whose speculation ends the cycle: one that leaves
            // another loop of imports waiting is never determined.
            if looped_without(graph, component, &(*module, name.clone(), at), steps)? {
                continue;
            }
            acts.push(Act::Fallback(keepers, (*module, name.clone(), at)));
            any = true;
        }
        Ok(any)
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
        steps: Steps<'_>,
    ) -> Result<Option<Vec<NamePlace>>, LoadError> {
        if imports.len() < 2 {
            return Ok(None);
        }
        steps.reserve(
            btree_map::<usize, ()>()
                + component.len() * btree_entry::<usize, ()>()
                + btree_map::<NamePlace, ()>(),
        )?;
        steps.storage_work(component.len())?;
        let nodes: BTreeSet<usize> = component.iter().copied().collect();
        let mut candidates: BTreeSet<NamePlace> = BTreeSet::new();
        for (&position, spaces) in imports {
            let decl = &self.uses[position];
            let UseKind::Named { name, segs } = &decl.kind else {
                return Ok(None);
            };
            let Some(last) = segs.last() else {
                return Ok(None);
            };
            for &at in spaces {
                let scratch = steps.temporary();
                let read_steps = scratch.as_ref().map_or(steps, |memory| steps.at(memory));
                if name != last {
                    let globbed = self
                        .read_globbed_any(decl.module, name, at, read_steps)
                        .map_err(storage_failure)?;
                    if !self.use_bound[position].iter().any(|bound| *bound) || globbed {
                        return Ok(None);
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
                    read_steps,
                );
                match walk {
                    Err(Fail::Missing(sources)) => {
                        if sources.iter().any(|source| &source.name != last) {
                            return Ok(None);
                        }
                        for source in sources {
                            let place = (source.module, source.name, slot(source.ns));
                            // A glob answer the reading module cannot access
                            // answers nothing for it, as rustc reads it.
                            let answered = if source.module == decl.module {
                                true
                            } else {
                                self.reserve_glob_lookup(place.0, &place.1, place.2, read_steps)
                                    .map_err(storage_failure)?;
                                let bound = self
                                    .read_globbed(place.0, &place.1, place.2, read_steps)
                                    .map_err(storage_failure)?;
                                self.seen_in(
                                    Cow::Owned(bound),
                                    place.0,
                                    decl.module,
                                    true,
                                    read_steps,
                                )
                                .map_err(storage_failure)?
                                .is_some()
                            };
                            if answered && graph.nodes_of(&place).any(|node| nodes.contains(&node))
                            {
                                steps.reserve(btree_entry::<NamePlace, ()>() + place.1.len())?;
                                candidates.insert(place);
                            }
                        }
                    }
                    Err(Fail::Ambiguous(_) | Fail::SeveralCrates { .. } | Fail::SuperAboveRoot) => {
                        return Ok(None);
                    }
                    Err(Fail::Budget(_)) => unreachable!("{LOAD_WALK}"),
                    Err(Fail::Storage(error)) => return Err(error),
                    Ok(_) => {}
                }
            }
        }
        self.kept_fallbacks(candidates, imports, steps).map(Some)
    }

    /// Candidate places still kept from their glob answer only by imports
    /// inside this component.
    fn kept_fallbacks(
        &self,
        candidates: BTreeSet<NamePlace>,
        imports: &BTreeMap<usize, Vec<usize>>,
        steps: Steps<'_>,
    ) -> Result<Vec<NamePlace>, LoadError> {
        let kept_inside = |module: usize, name: &str, at: usize| {
            self.imports_of(module, name)
                .iter()
                .any(|position| imports.contains_key(position) && !self.use_free[*position][at])
        };
        steps.reserve(vec_first::<NamePlace>() + candidates.len() * vec_slot::<NamePlace>())?;
        let mut fallbacks = Vec::new();
        for (module, name, at) in candidates {
            steps.storage_work(1 + self.imports_of(module, &name).len())?;
            if kept_inside(module, &name, at)
                && self.scopes[module]
                    .get(&name)
                    .is_none_or(|entry| entry.defs[at].is_empty())
                && self
                    .read_globbed_any(module, &name, at, steps)
                    .map_err(storage_failure)?
            {
                fallbacks.push((module, name, at));
            }
        }
        Ok(fallbacks)
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
        import: (usize, &[usize]),
        inside: &BTreeSet<usize>,
        looped: bool,
        pending: &Pending,
        acts: &mut Vec<Act>,
        steps: Steps<'_>,
    ) -> Result<Decided, LoadError> {
        let (position, spaces) = import;
        let decl = &self.uses[position];
        let UseKind::Named { segs, .. } = &decl.kind else {
            return Ok(Decided::Done);
        };
        let (walks, _, _) = self.import_paths(position, decl, segs, steps)?;
        if walks
            .iter()
            .any(|(_, walk)| matches!(walk, Err(Fail::Ambiguous(_))))
        {
            acts.push(Act::Hold(position));
            return Ok(Decided::Done);
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
                    empty[at] = self.binds_nothing_at(position, segs, &sources, inside, steps)?;
                    // Nor is one that a cycle left to the globs, whose path
                    // may still reach something there.
                    if (looped || self.left_to_globs(position, at)) && !empty[at] {
                        return Ok(Decided::Cyclic);
                    }
                }
                walk => {
                    // A path that now reaches something, as a glob no longer
                    // may bring in the crate name its first segment reads,
                    // is tried again in the queue before it is decided. One
                    // that a cycle left to the globs and that would take an
                    // item they answer away is never decided there.
                    let items = self.read_binds(ns, walk, segs.len(), steps)?;
                    if !items.is_empty() {
                        if self.left_to_globs(position, at)
                            && !self.covers_globbed(position, at, &items, pending, steps)?
                        {
                            return Ok(Decided::Cyclic);
                        }
                        acts.push(Act::Requeue(position));
                        return Ok(Decided::Again);
                    }
                    decided[at] = true;
                }
            }
        }
        acts.push(Act::Settle(position, decided, empty));
        Ok(Decided::Done)
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
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        let decl = &self.uses[position];
        if sources.is_empty() {
            return Ok(false);
        }
        for source in sources {
            steps.storage_work(1)?;
            let at = slot(source.ns);
            let viewer = decl.module;
            if let Some(defs) = self.scopes[source.module]
                .get(&source.name)
                .map(|entry| &entry.defs[at])
                .filter(|defs| !defs.is_empty())
            {
                let mut visible = false;
                for (_, origin) in defs.iter() {
                    if self
                        .origin_visible(origin, source.module, viewer, steps)
                        .map_err(storage_failure)?
                    {
                        visible = true;
                        break;
                    }
                }
                if !visible {
                    continue;
                }
            }
            if source.module == decl.module
                && segs.first() == Some(&source.name)
                && self.in_use_prelude(decl.module, &source.name, decl.test)
            {
                return Ok(false);
            }
            if !self.binds_name(position, source.module, &source.name) {
                for other in self.imports_of(source.module, &source.name) {
                    steps.storage_work(1)?;
                    if self.keeps_out(*other, source.module, &source.name, at, inside, steps)? {
                        return Ok(false);
                    }
                }
            }
            if self
                .read_open_glob(source.module, Some(viewer), steps)
                .map_err(storage_failure)?
            {
                return Ok(false);
            }
            for place in self
                .read_pending_sources_for(source.module, &source.name, at, Some(viewer), steps)
                .map_err(storage_failure)?
            {
                if self.may_bind_without(place, source.module, inside, steps)? {
                    return Ok(false);
                }
            }
            for glob in self.globs_of(source.module) {
                steps.storage_work(1)?;
                if source.module != viewer
                    && !self
                        .read_visibility(&self.uses[*glob].vis, source.module, viewer, steps)
                        .map_err(storage_failure)?
                {
                    continue;
                }
                match self.glob_targets(*glob, steps) {
                    Err(Fail::Storage(error)) => return Err(error),
                    Err(_) => return Ok(false),
                    Ok(_) => (),
                }
            }
        }
        Ok(true)
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
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        let left_empty = self.use_empty[other][at]
            && !self
                .read_globbed_any(module, name, at, steps)
                .map_err(storage_failure)?;
        Ok(!inside.contains(&other) && !self.use_free[other][at] && !left_empty)
    }

    /// True when a glob that reads `place` may bring something in from it,
    /// now or once it is settled, with the imports `inside` a component left
    /// out, as rustc resolves an import without itself: it binds something
    /// there, an import other than those keeps it from the globs, a glob of
    /// its module is not settled or brings something in, or one of the
    /// places its globs may still bring the name in from may.
    fn may_bind_without(
        &self,
        place: NamePlace,
        viewer: usize,
        inside: &BTreeSet<usize>,
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        let memory = steps.temporary();
        let steps = memory.as_ref().map_or(steps, |memory| steps.at(memory));
        steps.reserve(btree_map::<(usize, String, usize, usize), ()>())?;
        let mut seen = BTreeSet::new();
        let (module, name, at) = place;
        let stack_memory = steps.query_meter().map(Transient::new);
        let (stack_file, stack_line) = self.module_site(module);
        let stack_steps =
            steps.with_query_storage(stack_memory.as_ref(), (&stack_file, stack_line));
        let mut work = WorkList::<_, 8>::new();
        push_work(&mut work, (module, name, at, viewer), stack_steps)?;
        while let Some((module, name, at, viewer)) = work.pop() {
            steps.storage_work(1)?;
            steps.reserve(btree_entry::<(usize, String, usize, usize), ()>() + name.len())?;
            if !seen.insert((module, name.clone(), at, viewer)) {
                continue;
            }
            if let Some(defs) = self.scopes[module]
                .get(&name)
                .map(|entry| &entry.defs[at])
                .filter(|defs| !defs.is_empty())
            {
                for (_, origin) in defs.iter() {
                    if self
                        .origin_visible(origin, module, viewer, steps)
                        .map_err(storage_failure)?
                    {
                        return Ok(true);
                    }
                }
                continue;
            }
            if self.has_preserved_shadow(module, &name, at, inside, steps)? {
                continue;
            }
            if self.glob_visible(module, &name, at, viewer, steps)?
                || self
                    .read_open_glob(module, Some(viewer), steps)
                    .map_err(storage_failure)?
            {
                return Ok(true);
            }
            for &other in self.imports_of(module, &name) {
                steps.storage_work(1)?;
                if self
                    .read_visibility(&self.uses[other].vis, module, viewer, steps)
                    .map_err(storage_failure)?
                    && self.keeps_out(other, module, &name, at, inside, steps)?
                {
                    return Ok(true);
                }
            }
            let next = self.pending_sources_without(module, &name, at, viewer, inside, steps)?;
            for (target, name, at) in next {
                push_work(&mut work, (target, name, at, module), stack_steps)?;
            }
        }
        Ok(false)
    }

    fn glob_visible(
        &self,
        module: usize,
        name: &str,
        at: usize,
        viewer: usize,
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        let memory = steps.temporary();
        let steps = memory.as_ref().map_or(steps, |memory| steps.at(memory));
        self.reserve_glob_lookup(module, name, at, steps)
            .map_err(storage_failure)?;
        let bound = self
            .read_globbed(module, name, at, steps)
            .map_err(storage_failure)?;
        for (_, origin) in bound.iter() {
            if self
                .origin_visible(origin, module, viewer, steps)
                .map_err(storage_failure)?
            {
                return Ok(true);
            }
        }
        Ok(false)
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
        steps: Steps<'_>,
    ) -> Result<Vec<NamePlace>, LoadError> {
        let mut sources = Vec::new();
        for glob in self.globs_of(module) {
            steps.storage_work(1)?;
            if !self
                .read_visibility(&self.uses[*glob].vis, module, viewer, steps)
                .map_err(storage_failure)?
            {
                continue;
            }
            for &target in &self.glob_reach[*glob] {
                let mut kept = false;
                for position in self.imports_of(target, name) {
                    steps.storage_work(1)?;
                    if inside.contains(position)
                        && !self.use_free[*position][at]
                        && self
                            .read_visibility(&self.uses[*position].vis, target, module, steps)
                            .map_err(storage_failure)?
                    {
                        kept = true;
                        break;
                    }
                }
                if kept
                    || self.scopes[target]
                        .get(name)
                        .is_none_or(|entry| entry.defs[at].is_empty())
                        && (self
                            .read_blocked_for(target, name, at, module, steps)
                            .map_err(storage_failure)?
                            || self
                                .read_pending_at(target, name, at, Some(module), None, steps)
                                .map_err(storage_failure)?)
                {
                    steps.reserve(
                        vec_slot::<NamePlace>()
                            + name.len()
                            + if sources.is_empty() {
                                vec_first::<NamePlace>()
                            } else {
                                0
                            },
                    )?;
                    sources.push((target, name.to_owned(), at));
                }
            }
        }
        Ok(sources)
    }

    /// Do what deciding a component decided.
    fn act(&mut self, act: Act, pending: &mut Pending, meter: &Meter) -> Result<(), LoadError> {
        match act {
            Act::PreserveShadow(position, at) => {
                let memory = Transient::new(meter);
                let decl = &self.uses[position];
                if !self.preserved_shadow(
                    position,
                    at,
                    meter,
                    Steps::LoadAt {
                        memory: &memory,
                        file: &decl.file,
                        line: decl.line,
                    },
                )? {
                    return Err(self.internal(
                        "an actual private cut changed before its shadow finality action"
                            .to_owned(),
                    ));
                }
                self.use_decided[position][at] = true;
                self.use_read_state[position][at] = ImportReadState::Shadowed;
                self.settle_use(position, pending, meter)?;
            }
            Act::FinalizeFallback(FinalFallback { position, at, id }) => {
                let memory = Transient::new(meter);
                let decl = &self.uses[position];
                let steps = Steps::LoadAt {
                    memory: &memory,
                    file: &decl.file,
                    line: decl.line,
                };
                let answers = self.fallback_supply(position, &[], steps)?;
                if self.use_bound[position].iter().any(|bound| *bound)
                    || (0..3).any(|other| {
                        other != at
                            && (!self.use_empty[position][other]
                                || !self.use_decided[position][other])
                    })
                    || !answers
                        .iter()
                        .any(|answer| slot(answer.ns) == at && answer.id == id)
                {
                    return Err(self.internal(
                        "a certified glob fallback changed before its finality action".to_owned(),
                    ));
                }
                self.use_decided[position][at] = true;
                self.use_resolved[position] = true;
                self.use_read_state[position][at] = ImportReadState::Delegated;
                for at in 0..3 {
                    self.use_free[position][at] |= self.use_empty[position][at];
                }
                self.settle_use(position, pending, meter)?;
            }
            Act::Settle(position, decided, empty) => {
                let bound = self.use_bound[position].iter().any(|bound| *bound);
                for at in 0..3 {
                    self.use_decided[position][at] |= decided[at];
                    self.use_empty[position][at] |= empty[at];
                    self.use_free[position][at] |= empty[at] && bound;
                }
                if self.use_decided[position] == [true; 3] {
                    self.settle_use(position, pending, meter)?;
                } else {
                    self.touch_named_import(position, pending, meter)?;
                }
            }
            Act::Hold(position) => {
                self.hold(position);
                self.settle_use(position, pending, meter)?;
            }
            Act::Requeue(position) => pending.push(position)?,
            Act::CloseGlob(position) => {
                self.close_glob(position, pending)?;
                pending.push(position)?;
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

    /// Notify readers with the temporary owned name admitted before cloning.
    fn touch_named_import(
        &mut self,
        position: usize,
        pending: &mut Pending,
        meter: &Meter,
    ) -> Result<(), LoadError> {
        let decl = &self.uses[position];
        let UseKind::Named { name, .. } = &decl.kind else {
            unreachable!("only a named import changes a bound name");
        };
        let memory = Transient::new(meter);
        memory.charge(name.len(), &decl.file, Some(decl.line))?;
        Steps::LoadAt {
            memory: &memory,
            file: &decl.file,
            line: decl.line,
        }
        .storage_work(name.len())?;
        let (module, name) = (decl.module, name.clone());
        self.touch(module, &name, pending, meter)
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
            let reconstruct = first.is_none() && self.drops_need_reconstruction(meter)?;
            let Some(first) = first.or_else(|| reconstruct.then(|| self.glob_imports[0])) else {
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
            self.use_ambiguous = if reconstruct {
                vec![false; self.uses.len()]
            } else {
                ambiguous
            };
        }
    }

    /// A certified row batch retains rooted payload only for terminal reads.
    /// There is no drain, arrival or propagation between batches. If any later
    /// proof fails, the caller releases these tables before ordinary settling;
    /// historical hidden/covert/exact hints never seed resumed propagation.
    fn drops_need_reconstruction(&mut self, meter: &Meter) -> Result<bool, LoadError> {
        loop {
            let dropped = self.record_drops(meter)?;
            if let Some(&glob) = self.glob_imports.first() {
                self.settle_lookups(meter, glob)?;
            }
            if dropped == 0 {
                return Ok(false);
            }
            if !self.ranked_glob_tables(meter)? {
                return Ok(true);
            }
            if self.discovery_is_row_independent(meter)? {
                // The completed discovery published every new/wider selection
                // from immutable raw inputs. Ranked validation independently
                // proves the retained table rooted and closed under those rows.
                // No propagation resumes: another discovery can add no row.
                return Ok(false);
            }
        }
    }

    /// Static discovery inputs after successful actual rank/path validation.
    /// This is not independently a rooted-table or closed-path certificate.
    pub(crate) fn discovery_is_row_independent(&self, meter: &Meter) -> Result<bool, LoadError> {
        let Some(&first) = self.glob_imports.first() else {
            return Ok(false);
        };
        let decl = &self.uses[first];
        let charge = |units| {
            meter.work(units, &decl.file, Some(decl.line))?;
            work::add(Work::ProofReads, units);
            Ok::<_, LoadError>(())
        };
        charge(1)?;
        if !self.named_imports.is_empty() {
            return Ok(false);
        }
        charge(1)?;
        if !self.extern_names.is_empty() {
            return Ok(false);
        }
        charge(1)?;
        if !self.aliases.is_empty() {
            return Ok(false);
        }
        for use_ in &self.uses {
            charge(1)?;
            if !matches!(use_.kind, UseKind::Glob { .. })
                || !matches!(use_.vis, Vis::Public | Vis::Inherited)
            {
                return Ok(false);
            }
        }
        // definition_supply walks ALL genuine roots, not just view targets.
        // Public roots have no row-dependent read_level_of path; macro_rules
        // uses the same unconditional PUBLIC normalization as that supplier.
        for module in &self.modules {
            charge(1)?;
            for id in &module.defs {
                charge(1)?;
                let item = &self.items[id.0];
                if item.kind != crate::item::SymbolKind::MacroRules
                    && item.visibility != Vis::Public
                {
                    return Ok(false);
                }
            }
        }
        Ok(true)
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
        self.check_final_glob_fallbacks();
        self.check_preserved_shadows();
        self.check_named_bindings();
    }

    /// Final delegated slots retain the complete certificate after all waves.
    #[cfg(debug_assertions)]
    fn check_final_glob_fallbacks(&self) {
        let delegated: BTreeMap<usize, Vec<usize>> = self
            .named_imports
            .iter()
            .copied()
            .filter(|position| self.use_read_state[*position].contains(&ImportReadState::Delegated))
            .map(|position| {
                let slots: Vec<_> = (0..3)
                    .filter(|at| self.final_glob_fallback(position, *at))
                    .collect();
                assert_eq!(
                    slots.len(),
                    1,
                    "a delegated import needs exactly one final glob slot"
                );
                assert!(self.use_resolved[position] && self.use_settled[position]);
                assert_eq!(self.use_decided[position], [true; 3]);
                assert!(!self.use_bound[position].iter().any(|bound| *bound));
                let at = slots[0];
                assert!(
                    self.use_free[position][at]
                        && self.use_decided[position][at]
                        && !self.use_empty[position][at]
                );
                assert!((0..3).all(|other| other == at || self.use_empty[position][other]));
                (position, (0..3).filter(|at| *at != slots[0]).collect())
            })
            .collect();
        if !delegated.is_empty() {
            let meter = Meter::new(&crate::options::LoadOptions::default());
            let certificates = self
                .grounded_cycle_imports(&delegated, &meter, Steps::Load)
                .expect("a settled fallback proof fits its load budget")
                .expect("a final glob fallback lost its unique closed rooted answer");
            assert_eq!(certificates.len(), delegated.len());
            for answer in certificates {
                assert!(self.final_glob_fallback(answer.position, answer.at));
                let decl = &self.uses[answer.position];
                let UseKind::Named { name, .. } = &decl.kind else {
                    unreachable!("named import")
                };
                let bound = self
                    .read_globbed(decl.module, name, answer.at, Steps::Load)
                    .expect("resident fallback");
                assert_eq!(bound.len(), 1);
                assert!(bound.get(answer.id).is_some());
            }
        }
    }

    /// Named bindings and final delegated slots agree with their actual paths.
    #[cfg(debug_assertions)]
    fn check_named_bindings(&self) {
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
                let (walks, _, _) = self
                    .import_paths(position, import, segs, Steps::Load)
                    .expect("a debug consistency check reserves no import storage");
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
                    // This slot has a final, closed rooted glob answer;
                    // installing a narrower named binding would hide it.
                    if self.final_glob_fallback(position, slot(ns)) {
                        continue;
                    }
                    let binds = self.binds(ns, walk, segs.len());
                    if self.use_read_state[position][slot(ns)] == ImportReadState::Refused {
                        assert!(
                            self.use_bound[position].iter().any(|bound| *bound)
                                && self.use_empty[position][slot(ns)]
                                && !self.use_bound[position][slot(ns)],
                            "{}:{}: a private namespace refusal requires a partially resolved import without a binding in that namespace",
                            import.file,
                            import.line
                        );
                        continue;
                    }
                    if self.use_read_state[position][slot(ns)] == ImportReadState::Late {
                        assert!(
                            self.use_empty[position][slot(ns)]
                                && !self.use_bound[position][slot(ns)]
                                && !binds.is_empty(),
                            "{}:{}: an indeterminate import must retain its initial refusal and final binding",
                            import.file,
                            import.line
                        );
                        continue;
                    }
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

    fn drain(
        &mut self,
        pending: &mut Pending,
        meter: &Meter,
        ordinary: bool,
    ) -> Result<(), LoadError> {
        while !pending.queue.is_empty() {
            // Read one generation and register every waiter before applying
            // any result. Otherwise a same-batch arrival can precede its
            // reader's subscription and hide an initial privacy refusal.
            let memory = pending.memory.child();
            let first = *pending
                .queue
                .front()
                .expect("a generation has queued imports");
            let decl = &self.uses[first];
            let mut batch = Vec::new();
            let mut admission = NativeAdmission::new(Steps::LoadAt {
                memory: &memory,
                file: &decl.file,
                line: decl.line,
            });
            purrdf_lex::allocation::Memory::new(&mut admission)
                .reserve(&mut batch, pending.queue.len())
                .map_err(|error| admission.graph_error(error.into()))?;
            while let Some(position) = pending.pop()? {
                let scratch = memory.child();
                let read = self.read_import(position, pending, &scratch, ordinary)?;
                self.settle_lookups(meter, position)?;
                let decl = &self.uses[position];
                scratch.transfer_to(&memory, read.retained_bytes(), &decl.file, Some(decl.line))?;
                batch.push((position, read));
            }
            pending.begin_generation()?;
            for (position, read) in batch {
                self.apply_import(position, read, pending, meter)?;
            }
        }
        Ok(())
    }

    /// Read an import and subscribe to changes before this generation is applied.
    fn read_import(
        &self,
        position: usize,
        pending: &mut Pending<'_>,
        memory: &Transient<'_>,
        ordinary: bool,
    ) -> Result<ImportRead, LoadError> {
        let decl = &self.uses[position];
        let steps = Steps::LoadAt {
            memory,
            file: &decl.file,
            line: decl.line,
        };
        Ok(match &decl.kind {
            UseKind::Named { segs, .. } => {
                let (walks, reads, ends) = self.import_paths(position, decl, segs, steps)?;
                let mut missing = [false; 3];
                let mut private = [PrivacyDecision::Unproven; 3];
                for (at, (ns, walk)) in walks.iter().enumerate() {
                    if let Err(Fail::Missing(sources)) = walk {
                        private[slot(*ns)] =
                            self.inaccessible_read(position, sources, steps, ordinary)?;
                        missing[slot(*ns)] = private[slot(*ns)] == PrivacyDecision::Final
                            || private[slot(*ns)] != PrivacyDecision::Deferred
                                && self.reads_final(
                                    position,
                                    &reads[ends[at]..ends[at + 1]],
                                    sources,
                                    steps,
                                )?;
                    }
                }
                pending.wait(position, reads)?;
                ImportRead::Named {
                    walks,
                    missing,
                    private,
                }
            }
            UseKind::Glob { .. } => {
                let mut reads = Vec::new();
                let targets = match self.glob_targets_noting(position, Some(&mut reads), steps) {
                    Err(Fail::Storage(error)) => return Err(error),
                    other => other,
                };
                let settled = targets.as_ref().is_ok_and(|targets| {
                    targets
                        .iter()
                        .all(|target| !self.open_glob(*target, Some(decl.module), Steps::Load))
                });
                pending.wait(position, reads)?;
                if let Ok(targets) = &targets {
                    for target in targets {
                        pending.watch(position, *target)?;
                    }
                }
                ImportRead::Glob { targets, settled }
            }
        })
    }

    /// Apply a path read against the preceding generation.
    fn apply_import(
        &mut self,
        position: usize,
        read: ImportRead,
        pending: &mut Pending,
        meter: &Meter,
    ) -> Result<(), LoadError> {
        let memory = Transient::new(meter);
        let source = &self.uses[position];
        memory.charge(source.clone_bytes(), &source.file, Some(source.line))?;
        let decl = self.uses[position].clone();
        let bound = match (&decl.kind, read) {
            // An import held as ambiguous binds nothing.
            (UseKind::Named { .. }, _) if self.use_ambiguous[position] => Vec::new(),
            (
                UseKind::Named { .. },
                ImportRead::Named {
                    walks,
                    missing,
                    private,
                },
            ) => {
                let bound = self.settle_named(
                    position,
                    &decl,
                    (walks, missing, private),
                    pending,
                    meter,
                    &memory,
                )?;
                // An import held as ambiguous binds nothing for good and
                // is settled. One decided in every name space, bound or
                // not, is settled too (see [`Index::settle_stuck`]).
                if self.use_ambiguous[position] || self.use_decided[position] == [true; 3] {
                    self.settle_use(position, pending, meter)?;
                }
                bound
            }
            (UseKind::Glob { .. }, ImportRead::Glob { targets, settled }) => {
                self.settle_glob(position, &decl, (targets, settled), pending, meter)?;
                Vec::new()
            }
            _ => unreachable!("an import's kind does not change during a batch"),
        };
        for (module, name) in bound {
            self.touch(module, &name, pending, meter)?;
        }
        self.settle_lookups(meter, position)
    }

    /// An existing inaccessible binding determines failure before a later
    /// glob route can widen it. Evaluate this with the batch's path reads,
    /// never after another import of that batch changed their visibility.
    fn inaccessible_read(
        &self,
        position: usize,
        sources: &[Source],
        steps: Steps<'_>,
        ordinary: bool,
    ) -> Result<PrivacyDecision, LoadError> {
        let viewer = self.uses[position].module;
        if sources.is_empty() {
            return Ok(PrivacyDecision::Unproven);
        }
        for source in sources {
            steps.storage_work(1)?;
            let lookup = self
                .read_lookup(
                    source.module,
                    source.ns,
                    &source.name,
                    Some(position),
                    None,
                    steps,
                )
                .map_err(storage_failure)?;
            let Some((bound, globbed)) = lookup.as_ref() else {
                return Ok(PrivacyDecision::Unproven);
            };
            for (_, origin) in bound.iter() {
                if self
                    .origin_visible(origin, source.module, viewer, steps)
                    .map_err(storage_failure)?
                {
                    return Ok(PrivacyDecision::Unproven);
                }
            }
            if ordinary
                && source.module != viewer
                && !globbed
                && source.ns != Namespace::Macro
                && self
                    .forwarded_sources(bound, steps)
                    .map_err(storage_failure)?
            {
                return Ok(PrivacyDecision::Deferred);
            }
        }
        Ok(PrivacyDecision::Final)
    }

    /// Freeze ordinary privacy decisions before any named-cycle fallback.
    fn release_privacy_reads(
        &mut self,
        pending: &mut Pending,
        meter: &Meter,
    ) -> Result<(), LoadError> {
        for (position, states) in self.use_read_state.iter_mut().enumerate() {
            let decl = &self.uses[position];
            meter.work(3, &decl.file, Some(decl.line))?;
            work::add(Work::SettleSteps, 3);
            let mut deferred = false;
            for state in states {
                if *state == ImportReadState::Deferred {
                    *state = ImportReadState::Unread;
                    deferred = true;
                }
            }
            if deferred {
                pending.push(position)?;
            }
        }
        Ok(())
    }

    /// Whether ordinary propagation still owns a deferred privacy decision.
    fn has_deferred_privacy(&self, meter: &Meter) -> Result<bool, LoadError> {
        for (position, states) in self.use_read_state.iter().enumerate() {
            let decl = &self.uses[position];
            meter.work(3, &decl.file, Some(decl.line))?;
            work::add(Work::SettleSteps, 3);
            if states.contains(&ImportReadState::Deferred) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Globs are rebuilt without queue notifications. A path refused by an
    /// existing inaccessible binding can therefore acquire its final
    /// visibility only here. Keep that initial decision: a final binding
    /// does not retroactively make the import determinate. Ordinary proved
    /// absence remains subject to `check_settled`'s stronger invariant.
    fn finalize_privacy_reads(&mut self, meter: &Meter) -> Result<(), LoadError> {
        for position in 0..self.uses.len() {
            if self.use_read_state[position]
                .iter()
                .all(|state| *state == ImportReadState::Unread)
            {
                continue;
            }
            let decl = &self.uses[position];
            let UseKind::Named { segs, .. } = &decl.kind else {
                continue;
            };
            let memory = Transient::new(meter);
            let steps = Steps::LoadAt {
                memory: &memory,
                file: &decl.file,
                line: decl.line,
            };
            let (walks, _, _) = self.import_paths(
                position,
                decl,
                segs,
                Steps::LoadAt {
                    memory: &memory,
                    file: &decl.file,
                    line: decl.line,
                },
            )?;
            for (ns, walk) in walks {
                if matches!(
                    self.use_read_state[position][slot(ns)],
                    ImportReadState::Late | ImportReadState::Delegated | ImportReadState::Shadowed
                ) {
                    continue;
                }
                if self.use_read_state[position][slot(ns)] != ImportReadState::Unread {
                    let partial = self.use_read_state[position][slot(ns)]
                        == ImportReadState::Missing
                        && self.use_bound[position].iter().any(|bound| *bound);
                    let binds = self.read_binds(ns, walk, segs.len(), steps)?;
                    let mut kept_shadow = partial && !binds.is_empty();
                    for (_, hit) in &binds {
                        steps.storage_work(1)?;
                        kept_shadow &= self.final_private_shadow(position, hit, steps)?;
                    }
                    self.use_read_state[position][slot(ns)] = if binds.is_empty() || kept_shadow {
                        if partial {
                            ImportReadState::Refused
                        } else {
                            ImportReadState::Missing
                        }
                    } else {
                        ImportReadState::Late
                    };
                    self.has_late_imports |=
                        self.use_read_state[position][slot(ns)] == ImportReadState::Late;
                }
            }
            self.settle_lookups(meter, position)?;
        }
        Ok(())
    }

    /// A peer-export fallback does not widen an effective named binding
    /// that remains private, even when a returning alternative carries its
    /// same item. A partially bound import keeps that namespace absent;
    /// an actual visibility gain still makes the import indeterminate.
    fn final_private_shadow(
        &self,
        position: usize,
        hit: &Hit,
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        let Some(source) = &hit.last else {
            return Ok(false);
        };
        let viewer = self.uses[position].module;
        if viewer == source.module {
            return Ok(false);
        }
        let local = self
            .read_lookup(
                source.module,
                source.ns,
                &source.name,
                Some(position),
                None,
                steps,
            )
            .map_err(storage_failure)?;
        let Some((bound, false)) = local.as_ref() else {
            return Ok(false);
        };
        for (_, origin) in bound.iter() {
            steps.storage_work(1)?;
            if self
                .origin_visible(origin, source.module, viewer, steps)
                .map_err(storage_failure)?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// With fixed genuine paths, definitions and visibility classes, the
    /// drained monotone propagation is already the final fixed point. Check
    /// the actual path reads and resident closed targets before omitting a
    /// redundant replay; dynamic paths and named shadows still rebuild.
    pub(crate) fn fixed_glob_paths(&self, meter: &Meter) -> Result<bool, LoadError> {
        if !self.named_imports.is_empty() {
            return Ok(false);
        }
        let Some(&first) = self.glob_imports.first() else {
            return Ok(true);
        };
        let site = &self.uses[first];
        for module in 0..self.modules.len() {
            meter.work(1, &site.file, Some(site.line))?;
            work::add(Work::SettleSteps, 1);
            if self.exports_by_path(module) {
                return Ok(false);
            }
        }
        for &position in &self.glob_imports {
            let decl = &self.uses[position];
            meter.work(1, &decl.file, Some(decl.line))?;
            work::add(Work::SettleSteps, 1);
            if !self.glob_closed[position] {
                return Ok(false);
            }
            let memory = Transient::new(meter);
            let steps = Steps::LoadAt {
                memory: &memory,
                file: &decl.file,
                line: decl.line,
            };
            let mut reads = Vec::new();
            let targets = self.glob_targets_noting(position, Some(&mut reads), steps);
            self.settle_lookups(meter, position)?;
            let targets = match targets {
                Ok(targets) => targets,
                Err(Fail::Storage(error)) => return Err(error),
                Err(_) => return Ok(false),
            };
            steps.storage_work(targets.len() + self.glob_reach[position].len())?;
            if targets != self.glob_reach[position] {
                return Ok(false);
            }
            for source in reads {
                steps.storage_work(1)?;
                let Some(entry) = self.scopes[source.module].get(&source.name) else {
                    return Ok(false);
                };
                let defs = &entry.defs[slot(source.ns)];
                steps.storage_work(defs.len())?;
                if defs.is_empty() || defs.iter().any(|(_, origin)| origin.link.is_some()) {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    /// Execute the production replay for the operation-count tests.
    #[cfg(test)]
    pub(crate) fn replay_globs_for_measurement(&mut self, meter: &Meter) -> Result<(), LoadError> {
        self.rebuild_globs(meter)
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
        let memory = Transient::new(meter);
        for &position in &self.glob_imports {
            let steps = Steps::LoadAt {
                memory: &memory,
                file: &self.uses[position].file,
                line: self.uses[position].line,
            };
            match self.glob_targets(position, steps) {
                Ok(targets) => reached.push((position, targets)),
                Err(Fail::Storage(error)) => return Err(error),
                Err(_) => {}
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
        steps: Steps<'_>,
    ) -> Result<(Walks, Vec<Source>, [usize; 4]), LoadError> {
        let heads = Heads::Use {
            test: decl.test,
            own: Some(position),
        };
        let mut reads = Vec::new();
        let mut ends = [0; 4];
        steps.reserve(
            vec_first::<(Namespace, Result<Walked, Fail>)>()
                + 3 * vec_slot::<(Namespace, Result<Walked, Fail>)>(),
        )?;
        let mut walks = Vec::with_capacity(3);
        for (at, ns) in Namespace::ALL.into_iter().enumerate() {
            let walk = self.walk_noting(
                decl.module,
                segs,
                decl.leading_colon,
                ns,
                heads,
                Some(&mut reads),
                steps,
            );
            let walk = match walk {
                Err(Fail::Storage(error)) => return Err(error),
                Err(Fail::Budget(refused)) => return Err(self.refused_load(refused)),
                other => other,
            };
            ends[at + 1] = reads.len();
            walks.push((ns, walk));
        }
        Ok((walks, reads, ends))
    }

    /// What a walk in `ns` of an import's path of `len` segments binds the
    /// import's name to: the items it reached. A path through a name that
    /// stands for the crate roots of several crates binds the name to them,
    /// so a path through it, or to it, is ambiguous: an import of the crate
    /// name itself names crates, in the type name space; one that goes on
    /// into the crates names something in each name space the path could
    /// end in. A walk that failed otherwise binds nothing.
    #[cfg(debug_assertions)]
    fn binds(&self, ns: Namespace, walk: Result<Walked, Fail>, len: usize) -> Vec<(ItemId, Hit)> {
        self.read_binds(ns, walk, len, Steps::Load)
            .expect("nonretained binding reads reserve no import storage")
    }

    fn read_binds(
        &self,
        ns: Namespace,
        walk: Result<Walked, Fail>,
        len: usize,
        steps: Steps<'_>,
    ) -> Result<Vec<(ItemId, Hit)>, LoadError> {
        let count = match &walk {
            Ok(walked) if walked.items.is_empty() && ns == Namespace::Type => walked
                .modules
                .iter()
                .map(|module| self.module_item_ids(*module).count())
                .sum(),
            Ok(walked) => walked.items.len(),
            Err(Fail::SeveralCrates { roots, rest }) if ns == Namespace::Type || *rest < len => {
                roots.len()
            }
            _ => 0,
        };
        if count > 0 {
            steps.reserve(vec_first::<(ItemId, Hit)>() + count * vec_slot::<(ItemId, Hit)>())?;
        }
        steps.storage_work(count)?;
        Ok(match walk {
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
            Err(Fail::Storage(error)) => return Err(error),
        })
    }

    /// Try an explicit import. Returns the names it newly bound. The import
    /// waits on every place its path read, so it is tried again when one of
    /// them changes: one that cannot resolve yet waits, and one that resolved
    /// binds what its path reaches as the tables grow.
    fn settle_named(
        &mut self,
        position: usize,
        decl: &UseDecl,
        read: (Walks, [bool; 3], [PrivacyDecision; 3]),
        pending: &mut Pending,
        meter: &Meter,
        memory: &Transient<'_>,
    ) -> Result<Vec<(usize, String)>, LoadError> {
        let UseKind::Named { name, segs } = &decl.kind else {
            unreachable!("only named imports bind names");
        };
        let steps = Steps::LoadAt {
            memory,
            file: &decl.file,
            line: decl.line,
        };
        let (walks, missing, private) = read;
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
        let decide_now = missing.iter().any(|missing| *missing);
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
            let items = self.read_binds(ns, walk, segs.len(), steps)?;
            if !self.use_read_state[position][slot(ns)]
                .permits(private[slot(ns)], !items.is_empty())
            {
                continue;
            }
            // An import that a cycle left to the globs binds only what keeps
            // every item that a path which read its name found them
            // answering, so that what a path read there only grows; else
            // rustc never determines it, and neither does the index (see
            // [`Index::left_to_globs`]).
            if self.left_to_globs(position, slot(ns))
                && !items.is_empty()
                && !self.covers_globbed(position, slot(ns), &items, pending, steps)?
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
                let (vis, companion) = self.import_vis(decl, &hit, id, steps)?;
                steps.reserve(name.len())?;
                let origin = Origin {
                    companion,
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
                    steps.reserve(
                        vec_slot::<(usize, String)>()
                            + name.len()
                            + if bound.is_empty() {
                                vec_first::<(usize, String)>()
                            } else {
                                0
                            },
                    )?;
                    bound.push((decl.module, name.to_owned()));
                }
            }
        }
        if decide_now {
            self.settle_empty_spaces(position, decl, missing, private, pending, meter)?;
        }
        Ok(bound)
    }

    /// Apply the namespaces whose first read determined an empty answer.
    fn settle_empty_spaces(
        &mut self,
        position: usize,
        decl: &UseDecl,
        missing: [bool; 3],
        private: [PrivacyDecision; 3],
        pending: &mut Pending<'_>,
        meter: &Meter,
    ) -> Result<(), LoadError> {
        let UseKind::Named { name, .. } = &decl.kind else {
            unreachable!("only named imports settle empty namespaces")
        };
        let bound_any = self.use_bound[position].iter().any(|bound| *bound);
        let mut changed = false;
        for (at, missing) in missing.into_iter().enumerate() {
            if missing && !self.use_decided[position][at] {
                self.use_decided[position][at] = true;
                self.use_empty[position][at] = true;
                if private[at] == PrivacyDecision::Final {
                    self.use_read_state[position][at] = ImportReadState::Missing;
                }
                self.use_free[position][at] |= bound_any;
                changed = true;
            }
        }
        if changed {
            self.touch(decl.module, name, pending, meter)?;
        }
        Ok(())
    }

    /// Re-read a named declaration's actual routes for the settled holder
    /// proof. Scope origins merge visibility across routes and keep only one
    /// source, so they cannot supply correlated source/visibility edges.
    pub(crate) fn import_supply(
        &self,
        position: usize,
        steps: Steps<'_>,
    ) -> Result<Vec<ImportSupply>, LoadError> {
        let decl = &self.uses[position];
        let UseKind::Named { name, segs } = &decl.kind else {
            return Ok(Vec::new());
        };
        if !self.use_bound[position].iter().any(|bound| *bound) {
            return Ok(Vec::new());
        }
        let (walks, _, _) = self.import_paths(position, decl, segs, steps)?;
        let mut routes = Vec::new();
        for (ns, walk) in walks {
            if !self.use_bound[position][slot(ns)] {
                continue;
            }
            for (id, hit) in self.read_binds(ns, walk, segs.len(), steps)? {
                steps.storage_work(1)?;
                if self.scopes[decl.module]
                    .get(name)
                    .is_none_or(|entry| entry.defs[slot(ns)].get(id).is_none())
                {
                    continue;
                }
                let visibility_memory = steps.temporary();
                let visibility_steps = visibility_memory.as_ref().map_or(steps, |m| steps.at(m));
                let (vis, companion) = self.import_vis(decl, &hit, id, visibility_steps)?;
                let level = self
                    .read_level_of(&vis, decl.module, visibility_steps)
                    .map_err(storage_failure)?;
                drop((vis, visibility_memory));
                crate::glob_scope::reserve_next(&mut routes, steps)?;
                routes.push(ImportSupply {
                    id,
                    ns,
                    source: hit.last,
                    level: if companion { level.companion() } else { level },
                });
            }
        }
        Ok(routes)
    }

    /// Concrete other-namespace answers left to the globs by this import.
    /// A candidate must be the unique plain fallback and the unique item
    /// reached by the actual import path; provenance is checked separately.
    pub(crate) fn fallback_supply(
        &self,
        position: usize,
        spaces: &[usize],
        steps: Steps<'_>,
    ) -> Result<Vec<ImportSupply>, LoadError> {
        let decl = &self.uses[position];
        let UseKind::Named { name, segs } = &decl.kind else {
            return Ok(Vec::new());
        };
        let (walks, _, _) = self.import_paths(position, decl, segs, steps)?;
        let mut answers = Vec::new();
        for (ns, walk) in walks {
            let at = slot(ns);
            if spaces.contains(&at)
                || ns == Namespace::Macro
                || !self.use_free[position][at]
                || self.use_bound[position][at]
                || self.use_empty[position][at]
                || (0..3).any(|other| {
                    other != at && !spaces.contains(&other) && !self.use_decided[position][other]
                })
            {
                continue;
            }
            let items = self.read_binds(ns, walk, segs.len(), steps)?;
            let [(id, hit)] = items.as_slice() else {
                continue;
            };
            self.reserve_glob_lookup(decl.module, name, at, steps)
                .map_err(storage_failure)?;
            let fallback = self
                .read_globbed(decl.module, name, at, steps)
                .map_err(storage_failure)?;
            if fallback.len() != 1 || fallback.get(*id).is_none_or(|origin| origin.companion) {
                continue;
            }
            if !self.fallback_reads_closed(position, at, None, steps)? {
                continue;
            }
            crate::glob_scope::reserve_next(&mut answers, steps)?;
            steps.reserve(hit.last.as_ref().map_or(0, |source| source.name.len()))?;
            answers.push(ImportSupply {
                id: *id,
                ns,
                source: hit.last.clone(),
                level: Level::within(decl.module),
            });
        }
        Ok(answers)
    }

    /// Close the actual path and every effective supplier of its fallback,
    /// including named-source prefixes and private-export peer decisions.
    fn fallback_reads_closed(
        &self,
        position: usize,
        at: usize,
        cut: Option<&Source>,
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        let memory = steps.temporary();
        let steps = memory.as_ref().map_or(steps, |memory| steps.at(memory));
        let decl = &self.uses[position];
        let UseKind::Named { name, .. } = &decl.kind else {
            return Ok(false);
        };
        steps.reserve(btree_map::<(usize, String, usize, usize), ()>())?;
        let mut seen = BTreeSet::new();
        let stack_memory = steps.query_meter().map(Transient::new);
        let (stack_file, stack_line) = self.module_site(decl.module);
        let stack_steps =
            steps.with_query_storage(stack_memory.as_ref(), (&stack_file, stack_line));
        let mut work = WorkList::<_, 8>::new();
        let push = |work: &mut WorkList<_, 8>, module, name: &str, at, viewer| {
            steps.reserve(name.len())?;
            steps.storage_work(name.len())?;
            push_work(work, (module, name.to_owned(), at, viewer), stack_steps)?;
            Ok::<(), LoadError>(())
        };
        if let Some(cut) = cut {
            push(&mut work, cut.module, &cut.name, at, cut.module)?;
        } else {
            push(&mut work, decl.module, name, at, decl.module)?;
        }
        for (module, name, space, _) in self.read_places_in(position, at, steps)? {
            push(&mut work, module, &name, space, decl.module)?;
        }
        while let Some((module, name, space, viewer)) = work.pop() {
            steps.storage_work(1 + self.imports_of(module, &name).len())?;
            steps.reserve(btree_entry::<(usize, String, usize, usize), ()>() + name.len())?;
            steps.storage_work(name.len())?;
            if !seen.insert((module, name.clone(), space, viewer)) {
                continue;
            }
            let mut preserved = false;
            for &other in self.imports_of(module, &name) {
                if other == position && space == at {
                    preserved = cut.is_some();
                    continue;
                }
                if !self.use_decided[other][space] {
                    return Ok(false);
                }
                if self.use_bound[other][space] {
                    for (from, read, ns, _) in self.read_places_in(other, space, steps)? {
                        push(&mut work, from, &read, ns, self.uses[other].module)?;
                    }
                }
            }
            if preserved {
                continue;
            }
            if let Some(defs) = self.scopes[module]
                .get(&name)
                .map(|entry| &entry.defs[space])
                .filter(|defs| !defs.is_empty())
            {
                if let Some(peer) = self.fallback_peer_space(module, space, viewer, defs, steps)? {
                    if !self.fallback_private_reads_closed(module, &name, peer, defs, steps)? {
                        return Ok(false);
                    }
                    push(&mut work, module, &name, peer, viewer)?;
                }
                let lookup = self
                    .read_lookup(
                        module,
                        Namespace::ALL[space],
                        &name,
                        cut.is_none().then_some(position),
                        Some(viewer),
                        steps,
                    )
                    .map_err(storage_failure)?;
                if lookup.as_ref().is_none_or(|(_, globs)| !globs) {
                    continue;
                }
            }
            for &glob in self.globs_of(module) {
                steps.storage_work(1)?;
                if module != viewer
                    && !self
                        .read_visibility(&self.uses[glob].vis, module, viewer, steps)
                        .map_err(storage_failure)?
                {
                    continue;
                }
                let Some(closed) = self.closed_fallback_glob(glob, steps)? else {
                    return Ok(false);
                };
                for read in closed.reads {
                    push(&mut work, read.module, &read.name, slot(read.ns), module)?;
                }
                for target in closed.targets {
                    push(&mut work, target, &name, space, module)?;
                }
            }
        }
        Ok(true)
    }

    /// A supplier's module path must be final as well as its current targets.
    fn closed_fallback_glob(
        &self,
        glob: usize,
        steps: Steps<'_>,
    ) -> Result<Option<ClosedGlob>, LoadError> {
        if !self.glob_closed[glob] {
            return Ok(None);
        }
        let mut reads = Vec::new();
        let targets = match self.glob_targets_noting(glob, Some(&mut reads), steps) {
            Ok(targets) => targets,
            Err(Fail::Storage(error)) => return Err(error),
            Err(_) => return Ok(None),
        };
        for read in &reads {
            steps.storage_work(1 + self.imports_of(read.module, &read.name).len())?;
            if self.bound_by_undecided(read.module, read.ns, &read.name) {
                return Ok(None);
            }
        }
        Ok(Some(ClosedGlob { targets, reads }))
    }

    /// The exact branch that can consult a forwarded private namespace peer.
    fn fallback_peer_space(
        &self,
        module: usize,
        space: usize,
        viewer: usize,
        defs: &Bound,
        steps: Steps<'_>,
    ) -> Result<Option<usize>, LoadError> {
        if viewer == module || Namespace::ALL[space] == Namespace::Macro {
            return Ok(None);
        }
        for (_, origin) in defs.iter() {
            if self
                .origin_visible(origin, module, viewer, steps)
                .map_err(storage_failure)?
            {
                return Ok(None);
            }
        }
        Ok(Some(usize::from(space == 0)))
    }

    /// The private-export branch reads whole-import finality at its peer
    /// and linked sources, even when the selected namespace already bound.
    fn fallback_private_reads_closed(
        &self,
        module: usize,
        name: &str,
        peer: usize,
        defs: &Bound,
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        steps.storage_work(self.imports_of(module, name).len())?;
        if self.bound_by_undecided(module, Namespace::ALL[peer], name) {
            return Ok(false);
        }
        for (_, origin) in defs.iter() {
            steps.storage_work(1)?;
            let Some(source) = origin.link.as_ref().and_then(|link| link.source.as_ref()) else {
                break;
            };
            steps.storage_work(self.imports_of(source.module, &source.name).len())?;
            if self.bound_by_undecided(source.module, source.ns, &source.name) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// The visibility of the binding the named import `decl` makes of `id`,
    /// reached by `hit`: no wider than the binding it re-exports, as in
    /// rustc, which keeps `pub use` of a name its module reads through a
    /// private glob no more visible than the glob made it. The cap is the
    /// widest level at which the importing module can name the binding in the
    /// place the path ended; there is none for a path that ends at an
    /// associated item or at a module (`hit.last` is `None`).
    fn import_vis(
        &self,
        decl: &UseDecl,
        hit: &Hit,
        id: ItemId,
        steps: Steps<'_>,
    ) -> Result<(Vis, bool), LoadError> {
        let text = match &decl.vis {
            Vis::Restricted(text) => text.len(),
            _ => 0,
        };
        steps.reserve(text.max(9 + self.modules[decl.module].path.len()))?;
        let Some(last) = &hit.last else {
            return Ok((decl.vis.clone(), false));
        };
        let memory = steps.temporary();
        let steps = memory.as_ref().map_or(steps, |memory| steps.at(memory));
        let defs = self.scopes[last.module]
            .get(&last.name)
            .and_then(|entry| entry.defs[slot(last.ns)].get(id));
        let globbed;
        let origin = if let Some(origin) = defs {
            Some(origin)
        } else {
            self.reserve_glob_lookup(last.module, &last.name, slot(last.ns), steps)
                .map_err(storage_failure)?;
            globbed = self
                .read_globbed(last.module, &last.name, slot(last.ns), steps)
                .map_err(storage_failure)?;
            globbed.get(id)
        };
        let Some(origin) = origin else {
            return Ok((decl.vis.clone(), false));
        };
        let mut cap = None;
        for vis in origin.visibilities() {
            if self
                .read_visibility(vis, last.module, decl.module, steps)
                .map_err(storage_failure)?
            {
                let level = self
                    .read_level_of(vis, last.module, steps)
                    .map_err(storage_failure)?;
                cap = Some(cap.map_or(level, |previous| self.wider(previous, level)));
            }
        }
        let Some(cap) = cap else {
            return Ok((decl.vis.clone(), true));
        };
        let own = self
            .read_level_of(&decl.vis, decl.module, steps)
            .map_err(storage_failure)?;
        let met = self.meet(own, cap);
        Ok((
            if met == own {
                decl.vis.clone()
            } else {
                self.vis_at(met, decl.module)
            },
            origin.companion,
        ))
    }

    /// True when every place in `reads`, read by the path of the named import
    /// at `position` in one name space, is in a module without globs and is
    /// bound by no other import, or, for a place among `missing`, which the
    /// path found unbound, by none that was not decided to bind nothing
    /// there whatever is left to settle: what the path reaches there never
    /// changes.
    fn reads_final(
        &self,
        position: usize,
        reads: &[Source],
        missing: &[Source],
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        for source in reads {
            steps.storage_work(
                1 + missing.len() + self.imports_of(source.module, &source.name).len(),
            )?;
            let unbound = missing.contains(source);
            let at = slot(source.ns);
            // A successful prefix naming a definition cannot be replaced
            // by a glob. Unrelated globs in its parent do not postpone the
            // absence of the final name in that fixed module. Imported
            // prefixes retain the stronger dependency check below.
            if !unbound && let Some(entry) = self.scopes[source.module].get(&source.name) {
                steps.storage_work(entry.defs[at].len())?;
                if entry.defs[at]
                    .iter()
                    .any(|(_, origin)| origin.link.is_none())
                {
                    continue;
                }
            }
            // A missing name cannot become accessible through a glob whose
            // own visibility excludes the importing module, regardless of
            // what its still-unresolved path eventually supplies.
            let mut open_glob = false;
            for glob in self.globs_of(source.module) {
                steps.storage_work(1)?;
                if !unbound
                    || self
                        .read_visibility(
                            &self.uses[*glob].vis,
                            source.module,
                            self.uses[position].module,
                            steps,
                        )
                        .map_err(storage_failure)?
                {
                    open_glob = true;
                    break;
                }
            }
            if open_glob
                || !self
                    .imports_of(source.module, &source.name)
                    .iter()
                    .all(|other| {
                        *other == position
                            || unbound && self.use_decided[*other][at] && self.use_empty[*other][at]
                    })
            {
                return Ok(false);
            }
        }
        Ok(true)
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

    /// A certified closed rooted answer is final on the globs, without
    /// installing a named binding or claiming the namespace is empty.
    #[cfg(debug_assertions)]
    fn final_glob_fallback(&self, position: usize, at: usize) -> bool {
        self.use_read_state[position][at] == ImportReadState::Delegated
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
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        let decl = &self.uses[position];
        let UseKind::Named { name, .. } = &decl.kind else {
            return Ok(true);
        };
        let memory = steps.temporary();
        let steps = memory.as_ref().map_or(steps, |memory| steps.at(memory));
        self.reserve_glob_lookup(decl.module, name, at, steps)
            .map_err(storage_failure)?;
        let globbed = self
            .read_globbed(decl.module, name, at, steps)
            .map_err(storage_failure)?;
        for reader in pending.waiters(decl.module, name).into_iter().flatten() {
            if *reader == position {
                continue;
            }
            let viewer = self.uses[*reader].module;
            // A glob ambiguity is visible in full whenever any route is visible.
            let mut seen = 0;
            for (_, origin) in globbed.iter() {
                if self
                    .origin_visible(origin, decl.module, viewer, steps)
                    .map_err(storage_failure)?
                {
                    seen += 1;
                }
            }
            if seen == 0 {
                continue;
            }
            if !self
                .read_visibility(&decl.vis, decl.module, viewer, steps)
                .map_err(storage_failure)?
            {
                return Ok(false);
            }
            let all = ambiguity_seen(globbed.len(), seen);
            for (id, origin) in globbed.iter() {
                if all
                    || self
                        .origin_visible(origin, decl.module, viewer, steps)
                        .map_err(storage_failure)?
                {
                    steps.storage_work(items.len())?;
                    if !items.iter().any(|(item, _)| item == id) {
                        return Ok(false);
                    }
                }
            }
        }
        Ok(true)
    }

    /// The items a walk in `ns` binds an import's name to: the items it
    /// reached, or for a path that ends at a module and not at a name in it
    /// (`use shared;` of a library of the workspace, `use crate as name;`),
    /// in the type name space, the item of that module.
    fn walked_items(&self, walked: Walked, ns: Namespace) -> Vec<(ItemId, Hit)> {
        let mut items: Vec<(ItemId, Hit)> = walked.items.into_iter().collect();
        if items.is_empty() && ns == Namespace::Type {
            for module in &walked.modules {
                for id in self.module_item_ids(*module) {
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
        read: (Result<BTreeSet<usize>, Fail>, bool),
        pending: &mut Pending,
        meter: &Meter,
    ) -> Result<(), LoadError> {
        if self.use_ambiguous[position] {
            self.close_glob(position, pending)?;
            return Ok(());
        }
        let (targets, mut settled) = read;
        match targets {
            Ok(targets) => {
                self.glob_reach[position].clone_from(&targets);
                // Settled once every glob of its targets that its module can
                // name is.
                for target in targets {
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
            Err(Fail::Storage(error)) => return Err(error),
        }
        if settled {
            self.close_glob(position, pending)?;
        }
        Ok(())
    }

    /// Mark the glob at `position` settled, waking what waits on a name of
    /// its module.
    fn close_glob(&mut self, position: usize, pending: &mut Pending<'_>) -> Result<(), LoadError> {
        if !self.glob_closed[position] {
            self.glob_closed[position] = true;
            pending.wake_module(self.uses[position].module)?;
        }
        Ok(())
    }
}

/// The blocker graph of the unsettled named imports (see
/// [`Index::settle_stuck`]): node `n` below `imports.len()` is the import at
/// position `imports[n]`, an import or a glob, and node `imports.len() + p`
/// is the place `places[p]`: a name in a module in a name space, read without
/// the glob at the position given, for the reads of a glob's path in its own
/// module.
struct Blockers<'m> {
    memory: Transient<'m>,
    imports: Vec<(usize, Option<usize>)>,
    places: Vec<Place>,
    /// For each place, the import whose node it was first reached from:
    /// the checkpoints of the lookups of the place name it.
    origins: Vec<usize>,
    place_of: BTreeMap<Place, usize>,
    successors: Vec<Vec<usize>>,
}

impl Blockers<'_> {
    /// The import of the node `node`: its own, or for a place the import it
    /// was first reached from.
    fn origin(&self, node: usize) -> usize {
        match node.checked_sub(self.imports.len()) {
            Some(place) => self.origins[place],
            None => self.imports[node].0,
        }
    }

    /// The node of `place`, added and queued in `work` the first time.
    fn place(
        &mut self,
        place: Place,
        work: &mut WorkList<usize, 8>,
        origin: usize,
        site: &UseDecl,
    ) -> Result<usize, LoadError> {
        self.step(site, 1)?;
        if let Some(node) = self.place_of.get(&place) {
            return Ok(*node);
        }
        let mut bytes = vec_slot::<Place>()
            + btree_entry::<Place, usize>()
            + 2 * place.1.len()
            + vec_slot::<usize>()
            + vec_slot::<Vec<usize>>();
        if self.places.is_empty() {
            bytes += vec_first::<Place>() + vec_first::<usize>() + btree_map::<Place, usize>();
        }
        self.memory.charge(bytes, &site.file, Some(site.line))?;
        let node = self.imports.len() + self.places.len();
        self.places.push(place.clone());
        self.origins.push(origin);
        self.place_of.insert(place, node);
        self.successors.push(Vec::new());
        push_work(
            work,
            node,
            Steps::LoadAt {
                memory: &self.memory,
                file: &site.file,
                line: site.line,
            },
        )?;
        Ok(node)
    }

    fn step(&self, site: &UseDecl, units: usize) -> Result<(), LoadError> {
        self.memory.work(units, &site.file, Some(site.line))?;
        work::add(Work::SettleSteps, units);
        Ok(())
    }

    fn edge(&mut self, from: usize, to: usize, site: &UseDecl) -> Result<(), LoadError> {
        self.step(site, 1)?;
        self.memory.charge(
            vec_slot::<usize>()
                + if self.successors[from].is_empty() {
                    vec_first::<usize>()
                } else {
                    0
                },
            &site.file,
            Some(site.line),
        )?;
        self.successors[from].push(to);
        Ok(())
    }

    /// The nodes of the places of `name` in `module` in the name space at
    /// `at` read with no glob left out, by any reader.
    fn nodes_of<'a>(&'a self, place: &'a NamePlace) -> impl Iterator<Item = usize> + 'a {
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
/// A name in one namespace of one module.
type NamePlace = (usize, String, usize);

type Place = (usize, String, usize, Option<usize>, bool, Option<usize>);

/// True when `component` of `graph` still holds a loop through an import
/// once the nodes of `place` are taken out of it.
fn looped_without(
    graph: &Blockers,
    component: &[usize],
    place: &NamePlace,
    steps: Steps<'_>,
) -> Result<bool, LoadError> {
    let memory = steps.temporary();
    let steps = memory.as_ref().map_or(steps, |memory| steps.at(memory));
    let count = component.len();
    let edges: usize = component
        .iter()
        .map(|node| graph.successors[*node].len())
        .sum();
    steps.reserve(
        btree_map::<usize, ()>()
            + btree_map::<usize, usize>()
            + vec_first::<usize>()
            + vec_first::<Vec<usize>>()
            + place.1.len()
            + graph.successors.len() * btree_entry::<usize, ()>()
            + count
                * (vec_slot::<usize>()
                    + btree_entry::<usize, usize>()
                    + vec_slot::<Vec<usize>>()
                    + vec_first::<usize>())
            + edges * vec_slot::<usize>(),
    )?;
    steps.storage_work(count + 3 * edges)?;
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
    Ok(read_components(&successors, steps)?.iter().any(|found| {
        found.iter().any(|node| named(kept[*node]))
            && (found.len() > 1 || found.iter().any(|node| successors[*node].contains(node)))
    }))
}

pub(crate) fn storage_failure(error: Fail) -> LoadError {
    match error {
        Fail::Storage(error) => error,
        _ => unreachable!("storage preparation returns only a storage failure"),
    }
}

/// What deciding a component of the blocker graph does.
enum Act {
    /// Finalize an actual inaccessible cut while retaining its named shadow.
    PreserveShadow(usize, usize),
    /// A proven rooted answer left to globs validates the import's already
    /// empty namespaces without installing a narrower named binding.
    FinalizeFallback(FinalFallback),
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
    Fallback(Vec<usize>, NamePlace),
}

impl Act {
    /// The import the decision is for: the first of the imports a fallback
    /// leaves the place to.
    fn import(&self) -> Option<usize> {
        match self {
            Self::FinalizeFallback(answer) => Some(answer.position),
            Self::PreserveShadow(position, _)
            | Self::Settle(position, ..)
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
/// order: a component comes after every component it leads to. `PurRDF` owns
/// the nonrecursive traversal; its component-index contract supplies the
/// dependency order rather than relying on incidental member emission order.
fn read_components(
    successors: &[Vec<usize>],
    steps: Steps<'_>,
) -> Result<Vec<Vec<usize>>, LoadError> {
    let nodes = successors.len();
    let edges: usize = successors.iter().map(Vec::len).sum();
    // Native Tarjan admits the actual overlapping buffers before allocation.
    // The extra edge pass validates the dense graph before traversal.
    steps.storage_work(12 * nodes + 2 * edges)?;
    let mut admission = NativeAdmission::new(steps);
    let mut memory = purrdf_lex::allocation::Memory::new(&mut admission);
    let result = memory.try_scope(|memory| {
        let index = purrdf_core::graph::scc_component_index_with_memory(successors, memory)?;
        let count = index.iter().max().map_or(0, |last| last + 1);
        let mut found = memory.collect(std::iter::repeat_with(Vec::new).take(count))?;
        for (node, &component) in index.iter().enumerate() {
            memory.push(&mut found[component], node)?;
        }
        memory.release_vec(index)?;
        Ok(found)
    });
    result.map_err(|error| admission.graph_error(error))
}

#[cfg(test)]
fn components(successors: &[Vec<usize>]) -> Vec<Vec<usize>> {
    read_components(successors, Steps::Load).expect("valid resident graph")
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
    fn pending_and_blocker_storage_require_the_original_grants() {
        for (source, has_imports) in [
            (
                "pub mod a { pub use crate::b::X; } pub mod b { pub use crate::a::X; }",
                true,
            ),
            ("pub fn ordinary() {}", false),
        ] {
            let run = |bytes, work| -> Result<crate::Usage, LoadError> {
                let mut index = settled(source);
                index.build_scopes();
                index.lookups.built(false);
                let meter = Meter::new(&LoadOptions {
                    max_item_bytes: 0,
                    max_glob_bytes: bytes,
                    max_work: work,
                    ..LoadOptions::default()
                });
                {
                    let mut pending = Pending::new(&index.uses, &meter)?;
                    for &position in &index.named_imports {
                        pending.push(position)?;
                    }
                    assert_eq!(!pending.queue.is_empty(), has_imports);
                    index.drain(&mut pending, &meter, true)?;
                    assert_eq!(!pending.waiting.is_empty(), has_imports);
                    assert_eq!(!pending.waiting_by_name.is_empty(), has_imports);
                    {
                        let graph = index.blocker_graph(&meter)?;
                        assert_eq!(!graph.imports.is_empty(), has_imports);
                        assert_eq!(!graph.places.is_empty(), has_imports);
                        assert_eq!(
                            graph.successors.iter().any(|row| !row.is_empty()),
                            has_imports
                        );
                    }
                    assert!(!index.settle_stuck(&mut pending, &meter, true)?);
                }
                Ok(meter.usage())
            };
            let usage = run(usize::MAX, usize::MAX).expect("admitted cycle and empty neighbor");
            run(usage.glob_bytes, usage.work).expect("exact original grants");
            if has_imports {
                assert!(usage.glob_bytes > 0 && usage.work > 0);
                assert!(matches!(
                    run(usage.glob_bytes - 1, usize::MAX)
                        .expect_err("short storage")
                        .kind,
                    LoadErrorKind::Budget {
                        budget: crate::Budget::GlobBytes,
                        ..
                    }
                ));
                assert!(matches!(
                    run(usize::MAX, usage.work - 1)
                        .expect_err("short work")
                        .kind,
                    LoadErrorKind::Budget {
                        budget: crate::Budget::Work,
                        ..
                    }
                ));
            } else {
                run(0, 0).expect("no-import neighbor needs neither grant");
            }
        }
    }

    #[test]
    fn repeated_glob_lookups_visit_reads_without_spending_the_settling_work_grant() {
        const GLOBS: usize = 8;
        let mut source = String::from("pub mod d { pub const X:u32=7; }\n");
        for at in 0..GLOBS {
            writeln!(source, "pub mod g{at} {{ pub use crate::d::*; }}").expect("source");
        }
        source.push_str("pub mod t {\n");
        for at in 0..GLOBS {
            writeln!(source, "use crate::g{at}::*; use self::X as Y{at};").expect("source");
        }
        source.push_str("pub const OWN:u32=9; }\n");
        let mut index = settled(&source);
        index.build_scopes();
        index.lookups.built(false);
        index.crates_open = false;
        let meter = Meter::new(&LoadOptions::default());
        index.settle(&meter).expect("settles under original grant");
        index
            .lookups_charged()
            .expect("settling charged every lookup");
        let module = index
            .modules
            .iter()
            .position(|m| m.path == "demo::t")
            .expect("t");
        let at = slot(Namespace::Value);
        let before = meter.usage();
        assert!(before.work > 0);
        let query = Meter::query(&LoadOptions::default());
        let steps = Steps::Query(&query);
        let ((), one) = work::measure_kind(Work::LookupReads, usize::MAX, || {
            assert_eq!(
                index.read_globbed(module, "X", at, steps).expect("X").len(),
                1
            );
        });
        assert_eq!(one.get(Work::LookupReads), GLOBS);
        let ((), positive) = work::measure_kind(Work::LookupReads, 1, || {
            assert!(
                index
                    .read_globbed_any(module, "X", at, steps)
                    .expect("X present")
            );
        });
        assert_eq!(positive.get(Work::LookupReads), 1);
        let ((), repeated) = work::measure_kind(Work::LookupReads, usize::MAX, || {
            for _ in 0..3 {
                assert_eq!(
                    index.read_globbed(module, "X", at, steps).expect("X").len(),
                    1
                );
                assert!(
                    !index
                        .read_globbed_any(module, "missing", at, steps)
                        .expect("absent")
                );
            }
        });
        assert_eq!(repeated.get(Work::LookupReads), 3 * GLOBS);
        assert!(
            query.usage().work > 0,
            "lookups spend the enclosing query allowance"
        );
        assert_eq!(meter.usage(), before);
        let ((), local) = work::measure_kind(Work::LookupReads, 0, || {
            for name in ["OWN", "Y0"] {
                assert!(
                    index
                        .read_lookup(module, Namespace::Value, name, None, None, steps)
                        .expect("local binding")
                        .as_ref()
                        .is_some()
                );
            }
        });
        assert_eq!(local.get(Work::LookupReads), 0);
        assert_eq!(meter.usage(), before);
    }

    #[test]
    fn pending_counts_duplicate_attempts_and_refuses_before_removing_a_queued_import() {
        let index = settled("use shared::inner::other::x as A;");
        for limit in [17, 16] {
            let meter = Meter::new(&LoadOptions {
                max_work: limit,
                ..LoadOptions::default()
            });
            let mut pending = Pending::new(&index.uses[..1], &meter).expect("admitted");
            let (result, tally) = work::measure(usize::MAX, || {
                // Three attempted pushes bill three visits but keep one entry.
                for _ in 0..3 {
                    pending.push(0)?;
                }
                assert_eq!(pending.queue.len(), 1);
                assert_eq!(pending.pop()?, Some(0));
                assert_eq!(pending.pop()?, None);
                // Repeating the same subscriptions still visits each source.
                for _ in 0..2 {
                    pending.wait(
                        0,
                        vec![Source {
                            module: 0,
                            ns: Namespace::Type,
                            name: "X".into(),
                        }],
                    )?;
                    pending.watch(0, 0)?;
                }
                assert_eq!(pending.waiters(0, "X").expect("subscribed").len(), 1);
                assert_eq!(pending.watching[&0].len(), 1);
                // One waiter plus one watcher attempts two pushes, not one.
                pending.wake_module(0)?;
                assert_eq!(pending.queue.len(), 1);
                pending.wake_module(0)?;
                assert_eq!(pending.queue.len(), 1);
                pending.begin_generation()?;
                pending.wake_module(0)?;
                pending.pop()
            });
            if limit == 17 {
                assert_eq!(result.expect("exact budget"), Some(0));
                assert!(pending.queue.is_empty());
                assert_eq!(meter.usage().work, 17);
                assert_eq!(tally.get(Work::SettleSteps), 17);
            } else {
                let error = result.expect_err("the last pop exceeds the budget");
                assert!(
                    matches!(
                        error.kind,
                        LoadErrorKind::Budget {
                            budget: crate::Budget::Work,
                            limit: 16
                        }
                    ),
                    "{error}"
                );
                assert_eq!(pending.queue.front(), Some(&0));
                assert!(pending.queued[0]);
                assert_eq!(tally.get(Work::SettleSteps), 16);
            }
        }
    }

    #[test]
    fn a_glob_prefix_bound_in_type_still_waits_for_its_named_import() {
        let mut index =
            settled("mod C {} mod owner { use crate::C; mod consumer { use super::C::*; } }");
        let owner = index
            .modules
            .iter()
            .position(|m| m.path.as_str() == "demo::owner")
            .expect("owner");
        let named = index.imports_of(owner, "C")[0];
        let glob = index
            .uses
            .iter()
            .position(
                |decl| matches!(&decl.kind, UseKind::Glob { segs } if segs == &["super", "C"]),
            )
            .expect("supplier glob");
        assert!(index.use_bound[named][slot(Namespace::Type)]);
        assert!(
            index
                .closed_fallback_glob(glob, Steps::Load)
                .unwrap()
                .is_some()
        );
        index.use_settled[named] = false;
        index.use_decided[named][slot(Namespace::Macro)] = false;
        assert!(
            index
                .closed_fallback_glob(glob, Steps::Load)
                .unwrap()
                .is_none()
        );
        index.use_decided[named][slot(Namespace::Macro)] = true;
        index.use_settled[named] = true;
        assert!(
            index
                .closed_fallback_glob(glob, Steps::Load)
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn retained_import_batches_reserve_previous_results_and_new_reads_together() {
        let mut index = settled("pub use shared::inner as One;\npub use shared::inner as Two;\n");
        index.build_scopes();
        index.lookups.built(false);
        assert_eq!(index.uses.len(), 2);
        let run = |limit: usize| -> Result<(usize, usize), LoadError> {
            let meter = Meter::new(&LoadOptions {
                max_glob_bytes: limit,
                ..LoadOptions::default()
            });
            let first_peak;
            let final_peak;
            {
                let memory = Transient::new(&meter);
                let mut pending = Pending::new(&index.uses, &meter)?;
                let first = index.read_import(0, &mut pending, &memory, true)?;
                first_peak = meter.usage().glob_bytes;
                let second = index.read_import(1, &mut pending, &memory, true)?;
                final_peak = meter.usage().glob_bytes;
                assert!(matches!(first, ImportRead::Named { .. }));
                assert!(matches!(second, ImportRead::Named { .. }));
            }
            // Both successful results and subscriptions have left scope.
            Transient::new(&meter).charge(limit, &index.uses[0].file, Some(index.uses[0].line))?;
            Ok((first_peak, final_peak))
        };
        let (first, total) = run(usize::MAX).expect("unrestricted retained reads");
        assert!(
            total > first,
            "both generations must remain reserved together"
        );
        assert_eq!(
            run(total).expect("exact simultaneous charge"),
            (first, total)
        );
        let error = run(total - 1).expect_err("the second retained read cannot fit");
        assert!(
            matches!(error.kind, LoadErrorKind::Budget { budget: crate::Budget::GlobBytes, limit } if limit == total - 1)
        );
        assert_eq!(
            (error.file.as_str(), error.line),
            ("demo/src/lib.rs", Some(2))
        );
    }

    #[test]
    fn excluding_ones_own_named_binding_reserves_the_glob_fallback_too() {
        let index = settled(
            "pub mod a { pub struct T; }\npub mod b { pub use crate::a::*; pub use self::T; }\n",
        );
        index.lookups.built(false);
        let position = index.uses.iter().position(|decl| matches!(&decl.kind, UseKind::Named {name, segs} if name == "T" && segs == &["self", "T"])).expect("self-binding import");
        let decl = &index.uses[position];
        let defs = &index.scopes[decl.module]["T"].defs[slot(Namespace::Type)];
        assert!(!defs.is_empty() && defs.iter().all(|(_, origin)| origin.link.is_some()));
        let minimum = {
            let meter = Meter::new(&LoadOptions::default());
            let memory = Transient::new(&meter);
            index
                .reserve_glob_lookup(
                    decl.module,
                    "T",
                    slot(Namespace::Type),
                    Steps::LoadAt {
                        memory: &memory,
                        file: &decl.file,
                        line: decl.line,
                    },
                )
                .expect("the same glob's route reservation");
            meter.usage().glob_bytes
        };
        let run = |limit: usize| -> Result<usize, Fail> {
            let meter = Meter::new(&LoadOptions {
                max_glob_bytes: limit,
                ..LoadOptions::default()
            });
            let memory = Transient::new(&meter);
            let lookup = index.read_lookup(
                decl.module,
                Namespace::Type,
                "T",
                Some(position),
                None,
                Steps::LoadAt {
                    memory: &memory,
                    file: &decl.file,
                    line: decl.line,
                },
            )?;
            let (bound, globbed) = lookup
                .as_ref()
                .expect("the glob answers after own-exclusion");
            assert!(globbed);
            assert_eq!(bound.len(), 1);
            Ok(meter.usage().glob_bytes)
        };
        let total = run(usize::MAX).expect("unrestricted owned fallback");
        assert!(
            minimum > 0 && total > minimum,
            "local exclusion scratch and glob routes coexist"
        );
        assert_eq!(run(total).expect("the exact combined bound"), total);
        let Fail::Storage(error) = run(total - 1).expect_err("one below the combined reservation")
        else {
            panic!("typed storage refusal")
        };
        assert!(
            matches!(error.kind, LoadErrorKind::Budget {budget: crate::Budget::GlobBytes, limit} if limit == total - 1)
        );
        assert_eq!((&error.file, error.line), (&decl.file, Some(decl.line)));
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

    #[test]
    fn native_component_grouping_refuses_one_below_its_live_storage_bound() {
        let graph = [vec![1], vec![0, 2], vec![]];
        let file = RelPath::new("demo/src/lib.rs").expect("path");
        let run = |limit| {
            let meter = Meter::new(&LoadOptions {
                max_glob_bytes: limit,
                ..LoadOptions::default()
            });
            let memory = Transient::new(&meter);
            let result = read_components(
                &graph,
                Steps::LoadAt {
                    memory: &memory,
                    file: &file,
                    line: 9,
                },
            )
            .map(|found| assert_eq!(found, [vec![2], vec![0, 1]]));
            (result, meter.usage().glob_bytes)
        };
        let (found, peak) = run(usize::MAX);
        found.expect("unrestricted graph");
        assert!(peak > 0);
        run(peak).0.expect("exact live bound");
        let error = run(peak - 1)
            .0
            .expect_err("one below the simultaneous buffers");
        assert!(matches!(error.kind, LoadErrorKind::Budget {
            budget: crate::Budget::GlobBytes, limit,
        } if limit == peak - 1));
        assert_eq!(
            (error.file.as_str(), error.line),
            ("demo/src/lib.rs", Some(9))
        );
    }

    fn origin(via_glob: bool) -> Origin {
        Origin {
            companion: false,
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
