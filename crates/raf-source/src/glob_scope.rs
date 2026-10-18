// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The glob scope tables: what the glob imports of each module bring in.
//!
//! What a glob import brings in from its target depends on the target and on
//! what the importer can access there, and on nothing else about the
//! importer. A binding is accessible within a module that is the target
//! itself or one of its ancestors (its level), or everywhere; so the
//! importers that lie within the same ancestors of the target see the same of
//! the target: they form a class. A glob import brings a binding in no more
//! visibly than the binding it reads, as rustc does: the level of what a view
//! holds is the narrower of the glob's own and the level the binding has in
//! the module it is read from, and so is the level of an explicit import of a
//! binding reached through a glob. What a module exports to globs (its bindings
//! that shadow globs, and what its own globs bring in where none does) is kept
//! once for each class of its importers, as a shared table, a view, and every
//! glob of the module from an importer of that class reads that view: no
//! importer holds a copy of what its globs bring in. The names of a view are
//! shared strings, each stored once for the load.
//!
//! A name that two globs of a module bring in as different items is ambiguous
//! in the module, as in rustc, and the ambiguity is as visible as the most
//! visible of its items, whichever glob each comes through. So a view holds
//! all the items the globs of its module bring in under such a name, as soon
//! as its class can access one of them, and the others at the widest level of
//! those it can: a reader of the view reads the name ambiguous, and one that
//! can access none reads nothing. This is [`ambiguity_seen`], applied by
//! every table that filters by access. The items a view holds only for that
//! reason are its companions, marked on their level. A module that already
//! holds one of the real items of such a name through a glob of another
//! module that binds the name itself, as when a cycle of globs delivers it
//! back beside that glob, drops the companions, as rustc drops the ambiguity
//! there.
//!
//! A view grows as the scope tables settle. What it gains is passed on only
//! to the globs that read it, only with the names that gained something, and
//! only where it can matter: to the views of the reading module, and to the
//! imports that wait on one of those names in the reading module. A glob
//! whose module has no view passes nothing on, and costs nothing for each
//! name its view gains. So a chain of globs costs what it brings in, once for
//! each link, and a module that a thousand modules glob is read by a thousand
//! globs from one table. A glob that reads a view another glob of its module
//! with the same visibility already reads passes nothing on, and what a
//! module exports is gathered from each view its globs read once, so a module
//! that globs one module many times over costs a read for each glob and not a
//! copy of the view.
//!
//! Every view, each name and item it holds, each glob's read of a view and
//! each gain while it waits to be passed on is charged to the glob-bytes
//! budget ([`Budget::GlobBytes`](crate::Budget::GlobBytes)) as an upper bound
//! of what it allocates, derived from the layout of the containers that hold
//! it (see `crate::cost`). The budget bounds what is held at once: what a gain
//! held is released once it is passed on, and what the tables of one settle
//! hold is released when they are built again. Each charge is made before
//! the allocation it covers.
//!
//! A few temporaries are not charged, each bounded by a constant factor of
//! what another budget already charges and dropped before the next is made:
//! the visibilities of each module, listed once for each item and `use`
//! declaration before their repeats are removed, and the bindings with their
//! routes that a lookup of what the globs of a module bring in under one name
//! returns.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::mem::size_of;
use std::ops::Bound::{Excluded, Unbounded};

use crate::cost::{btree_entry, btree_map, hash_entry, hash_map, shared_text, vec_first, vec_slot};
use crate::error::LoadError;
use crate::facts::Vis;
use crate::index::Index;
use crate::item::{ItemId, Namespace, SharedStr};
use crate::meter::Meter;
use crate::resolve::Heads;
use crate::settle::Bound;
use crate::work::{self as tally, Work};

/// How far a binding is accessible: everywhere, within one module (which is
/// the module the binding is in or one of its ancestors) and so by every
/// module inside it, or nowhere. A level also tells whether the item it is
/// held at is there only as a companion of an ambiguity (see
/// [`Index::exposed`]), which does not change how far it is accessible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Level(usize);

impl Level {
    /// The mark of a companion on the level of a module.
    const COMPANION: usize = 1 << 40;
    /// Accessible everywhere.
    pub(crate) const PUBLIC: Self = Self(usize::MAX);
    /// Accessible nowhere: a restriction that names no module.
    pub(crate) const NOBODY: Self = Self(usize::MAX - 1);

    /// Accessible within `module`.
    pub(crate) const fn within(module: usize) -> Self {
        Self(module)
    }

    /// The level of a companion that is accessible as far as `self` is.
    const fn companion(self) -> Self {
        match self.plain().0 {
            usize::MAX => Self(usize::MAX - 2),
            v if v == usize::MAX - 1 => Self(usize::MAX - 3),
            v => Self(v | Self::COMPANION),
        }
    }

    /// The level without the mark of a companion.
    const fn plain(self) -> Self {
        match self.0 {
            v if v == usize::MAX - 2 => Self::PUBLIC,
            v if v == usize::MAX - 3 => Self::NOBODY,
            v if v < usize::MAX - 3 => Self(v & !Self::COMPANION),
            _ => self,
        }
    }

    /// True when the item is held as a companion only.
    const fn is_companion(self) -> bool {
        self.plain().0 != self.0
    }

    /// The module the level is within, if it is one.
    const fn module(self) -> Option<usize> {
        let plain = self.plain();
        if plain.0 >= Self::NOBODY.0 {
            None
        } else {
            Some(plain.0)
        }
    }
}

/// Whether a reader sees what a name stands for in a module whose globs bring
/// it in as `candidates` different items, of which it can access `seen`: all
/// of them, as the ambiguity rustc keeps for a name two globs bring in as
/// different items, which is as visible as the most visible of its items. One
/// definition for every table that filters by access: the glob views, the
/// reads of a `use` path and the surface of a crate.
pub(crate) const fn ambiguity_seen(candidates: usize, seen: usize) -> bool {
    candidates > 1 && seen > 0
}

/// An item a view holds under a name, with the widest level at which the
/// module the view is of binds it, as far as the class of the view can access
/// it.
type Held = (ItemId, Level);

/// What a module exports under each name, in each name space.
type Exported = Vec<(String, [Vec<Held>; 3])>;

/// The items a name of a view stands for, in each name space, each list
/// sorted by item, one entry for each item.
type Lists = Box<[Vec<Held>; 3]>;

/// What one module exports to the globs of one class of importers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct View {
    /// The module the view exports.
    target: usize,
    /// For each ancestor of the target and the target itself, from the crate
    /// root down, whether the importers of the class lie within it, which
    /// is whether they can access a binding at that level.
    class: Box<[bool]>,
    /// The items each name stands for.
    names: BTreeMap<SharedStr, Lists>,
    /// The globs that read the view and whose module has a view that what
    /// they bring in reaches: each glob's module and its place among the
    /// module's reads.
    active: Vec<(usize, usize)>,
    /// The other globs that read the view, by module.
    passive: BTreeMap<usize, Vec<usize>>,
    /// The glob import the view was made for: a refusal by the budget while
    /// the view grows names its file and line.
    site: usize,
    /// Whether the globs of the target have brought in an item the class
    /// cannot access. Only then can a name of the view stand for items the
    /// class cannot access beside those it can (see [`Index::exposed`]), so
    /// a view for which it is false gathers a name from what arrives alone.
    hidden: bool,
}

/// A glob import's read of the view of one of its targets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GlobRead {
    /// The position of the glob import.
    pub(crate) glob: usize,
    /// The module the glob reaches.
    pub(crate) target: usize,
    /// The view it reads.
    view: usize,
}

/// The glob scope tables of a load.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct GlobTables {
    views: Vec<View>,
    /// The view of each module for each class.
    view_of: BTreeMap<(usize, Box<[bool]>), usize>,
    /// The views of each module.
    exports: Vec<Vec<usize>>,
    /// The reads of the globs of each module, in the order they were made.
    reads: Vec<Vec<GlobRead>>,
    /// Each glob import and the view it reads, once read.
    read_by: BTreeSet<(usize, usize)>,
    /// For each module, view and level of a glob of the module, whether a
    /// glob of the module with that level already reads the view: another
    /// such glob brings in nothing the first does not, and passes nothing on.
    passing: BTreeSet<(usize, usize, Level)>,
    /// How many ancestors each module has.
    depth: Vec<usize>,
    /// Whether a visibility of each module is `pub(in path)`, which reads
    /// the tables: what such a module exports to an importer can change as
    /// they settle, so its globs are tried again whenever it changes.
    in_path: Vec<bool>,
    /// One copy of each name the views hold.
    strings: HashSet<SharedStr>,
    /// What the tables were charged, released when they are cleared.
    charged: usize,
}

/// Which modules can reach, through the glob imports of the modules on the
/// way, a module with an explicit import of any name: only such a module's
/// globs may still bring in a name that an explicit import keeps from the
/// globs (see [`Index::pending_at`]), so the search for one starts only
/// there. It is kept over every glob edge recorded in the load, so it covers
/// the edges of the settled tables too.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ImportReach {
    /// For each module, whether it reaches such a module; empty until the
    /// first glob edge is recorded.
    reaches: Vec<bool>,
    /// For each module, the modules whose globs reach it.
    globbed_by: Vec<BTreeSet<usize>>,
}

/// What a view gained, waiting to be passed on: the items of `name` in the
/// name space at `at`.
struct Grown {
    view: usize,
    name: SharedStr,
    at: usize,
    ids: Vec<Held>,
}

impl Grown {
    /// What the change's list of items allocates at most.
    const fn bytes(&self) -> usize {
        vec_first::<Held>() + self.ids.len() * vec_slot::<Held>()
    }
}

/// What a view allocates at most beyond its class and its names: its record,
/// its entry among the views of every module and class (with a copy of the
/// class), its place among its module's views, its table of names, and its
/// lists of readers at their first room.
const VIEW_BYTES: usize = vec_slot::<View>()
    + btree_entry::<(usize, Box<[bool]>), usize>()
    + vec_slot::<usize>()
    + vec_first::<usize>()
    + btree_map::<SharedStr, Lists>()
    + vec_first::<(usize, usize)>()
    + btree_map::<usize, Vec<usize>>();

/// What a name of a view allocates at most beyond its text: its entry and its
/// lists.
const NAME_BYTES: usize = btree_entry::<SharedStr, Lists>() + size_of::<[Vec<Held>; 3]>();

/// What a glob's read of a view allocates at most: its place among its
/// module's reads (with their first room) and among the view's readers, as
/// an active reader or a passive one, and again if it turns active, and its
/// entries in the records of the reads made.
const READ_BYTES: usize = vec_slot::<GlobRead>()
    + btree_entry::<(usize, usize), ()>()
    + btree_entry::<(usize, usize, usize), ()>()
    + vec_first::<GlobRead>()
    + 2 * vec_slot::<(usize, usize)>()
    + btree_entry::<usize, Vec<usize>>()
    + vec_first::<usize>()
    + vec_slot::<usize>();

/// What the tables allocate at most once for a load whose modules are
/// `modules`, before their first view: the lists of each module, the table of
/// views and the table of names.
const fn base_bytes(modules: usize) -> usize {
    modules * (vec_slot::<Vec<usize>>() + vec_slot::<Vec<GlobRead>>())
        + vec_first::<Vec<usize>>()
        + vec_first::<Vec<GlobRead>>()
        + vec_first::<View>()
        + btree_map::<(usize, Box<[bool]>), usize>()
        + btree_map::<(usize, usize), ()>()
        + btree_map::<(usize, usize, usize), ()>()
        + hash_map::<SharedStr>()
}

/// What a waiting import's name is told: the imports that wait on a name of
/// a module are queued again when what the module's globs bring in under it
/// grows.
pub(crate) trait Waiters {
    /// Queue again what waits on `name` in `module`.
    fn wake(&mut self, module: usize, name: &str);
    /// The modules where something waits on `name`, if any.
    fn waiting_on(&self, name: &str) -> Option<&BTreeSet<usize>>;
    /// The names of `module` that something waits on.
    fn waited_in(&self, module: usize) -> Vec<String>;
}

/// No waiters: the tables are built once the imports are settled.
pub(crate) struct Nobody;

impl Waiters for Nobody {
    fn wake(&mut self, _module: usize, _name: &str) {}

    fn waiting_on(&self, _name: &str) -> Option<&BTreeSet<usize>> {
        None
    }

    fn waited_in(&self, _module: usize) -> Vec<String> {
        Vec::new()
    }
}

impl Index {
    /// Record how deep each module lies and whether a visibility of it reads
    /// the tables, once for the load, and charge them. Nothing is recorded
    /// for a load without glob imports, which builds no glob tables.
    pub(crate) fn glob_visibilities(&mut self, meter: &Meter) -> Result<(), LoadError> {
        let Some(&first) = self.glob_imports.first() else {
            return Ok(());
        };
        if !self.globs.depth.is_empty() {
            return Ok(());
        }
        let in_path_text =
            |vis: &Vis| matches!(vis, Vis::Restricted(text) if text.starts_with("in "));
        let mut in_path = vec![false; self.modules.len()];
        for (module, record) in self.modules.iter().enumerate() {
            in_path[module] = record
                .defs
                .iter()
                .any(|id| in_path_text(&self.items[id.0].visibility));
        }
        for decl in &self.uses {
            in_path[decl.module] |= in_path_text(&decl.vis);
        }
        let depth: Vec<usize> = (0..self.modules.len())
            .map(|module| self.module_depth(module))
            .collect();
        let bytes = vec_first::<usize>()
            + vec_first::<bool>()
            + self.modules.len() * (vec_slot::<usize>() + vec_slot::<bool>());
        let decl = &self.uses[first];
        meter.globs(bytes, &decl.file, Some(decl.line))?;
        self.globs.depth = depth;
        self.globs.in_path = in_path;
        Ok(())
    }

    /// Drop the glob tables, keeping the depths and the `pub(in path)` flags
    /// of the modules. What they were charged is the caller's to release.
    pub(crate) fn clear_globs(&mut self) {
        let depth = std::mem::take(&mut self.globs.depth);
        let in_path = std::mem::take(&mut self.globs.in_path);
        self.globs = GlobTables {
            depth,
            in_path,
            ..GlobTables::default()
        };
    }

    /// Release what the glob tables were charged to `meter` and drop them.
    pub(crate) fn release_globs(&mut self, meter: &Meter) {
        meter.release_globs(self.globs.charged);
        self.clear_globs();
    }

    /// Release and drop the glob tables, then charge the first room of new
    /// ones when the load has glob imports.
    pub(crate) fn reset_globs(&mut self, meter: &Meter) -> Result<(), LoadError> {
        self.release_globs(meter);
        let Some(&first) = self.glob_imports.first() else {
            return Ok(());
        };
        self.charge_tables(base_bytes(self.modules.len()), first, meter)?;
        self.globs.exports = vec![Vec::new(); self.modules.len()];
        self.globs.reads = vec![Vec::new(); self.modules.len()];
        Ok(())
    }

    /// How many ancestors `module` has.
    fn depth_of(&self, module: usize) -> usize {
        self.globs
            .depth
            .get(module)
            .copied()
            .unwrap_or_else(|| self.module_depth(module))
    }

    /// How wide a level is, smaller for a level that more modules can
    /// access: public first, then the shallower modules, and last nobody.
    fn width(&self, level: Level) -> usize {
        match level.module() {
            Some(module) => 1 + self.depth_of(module),
            None if level.plain() == Level::PUBLIC => 0,
            None => usize::MAX,
        }
    }

    /// Of two levels of one chain of ancestors the one more modules can
    /// access.
    pub(crate) fn wider(&self, left: Level, right: Level) -> Level {
        if self.width(right) < self.width(left) {
            right
        } else {
            left
        }
    }

    /// Of two levels of one chain of ancestors the one fewer modules can
    /// access: what a binding is accessible at when it is imported at one
    /// level a binding that is accessible at the other.
    pub(crate) fn meet(&self, left: Level, right: Level) -> Level {
        if self.width(right) > self.width(left) {
            right
        } else {
            left
        }
    }

    /// The level a binding with the visibility `vis` in `home` is accessible
    /// at: `pub(in path)` names an ancestor of `home` or `home` itself.
    pub(crate) fn level_of(&self, vis: &Vis, home: usize) -> Level {
        match vis {
            Vis::Public => Level::PUBLIC,
            Vis::Inherited => Level::within(home),
            Vis::Restricted(text) => match text.as_str() {
                "crate" => Level::within(self.modules[home].root),
                "self" => Level::within(home),
                "super" => self.modules[home]
                    .parent
                    .map_or(Level::NOBODY, Level::within),
                other => other
                    .strip_prefix("in ")
                    .map_or(Level::NOBODY, |written| self.level_in(written, home)),
            },
        }
    }

    /// The level of `pub(in written)` in `home`.
    fn level_in(&self, written: &str, home: usize) -> Level {
        let segs: Vec<String> = written
            .split_whitespace()
            .collect::<String>()
            .split("::")
            .map(str::to_owned)
            .collect();
        let Ok(walked) = self.walk(home, &segs, false, Namespace::Type, Heads::Local) else {
            return Level::NOBODY;
        };
        let mut modules = walked.modules;
        modules.extend(walked.items.keys().flat_map(|id| self.modules_for(*id)));
        modules
            .into_iter()
            .filter(|ancestor| self.within(home, *ancestor))
            .map(Level::within)
            .reduce(|left, right| self.wider(left, right))
            .unwrap_or(Level::NOBODY)
    }

    /// The visibility, written in `home`, of a binding of `home` accessible
    /// at `level`, which is public, nobody, `home` or one of its ancestors.
    pub(crate) fn vis_at(&self, level: Level, home: usize) -> Vis {
        let Some(module) = level.module() else {
            return if level.plain() == Level::PUBLIC {
                Vis::Public
            } else {
                Vis::Restricted("in ".to_owned())
            };
        };
        let root = self.modules[home].root;
        if module == home {
            Vis::Inherited
        } else if module == root {
            Vis::Restricted("crate".to_owned())
        } else {
            let (path, base) = (&self.modules[module].path, &self.modules[root].path);
            Vis::Restricted(format!("in crate{}", &path[base.len()..]))
        }
    }

    /// True when a binding of `module` accessible at `level` can be named by
    /// importers of `class`.
    fn allowed(&self, module: usize, class: &[bool], level: Level) -> bool {
        match level.module() {
            Some(within) => {
                debug_assert!(self.within(module, within), "a level is an ancestor's");
                class.get(self.depth_of(within)).copied().unwrap_or(false)
            }
            None => level.plain() == Level::PUBLIC,
        }
    }

    /// The class of `importer` for globs of `target`: which of the ancestors
    /// of the target, and the target, it lies within.
    fn class_for(&self, target: usize, importer: usize) -> Box<[bool]> {
        let mut chain = vec![target];
        while let Some(parent) = self.modules[*chain.last().expect("one module")].parent {
            chain.push(parent);
        }
        chain
            .iter()
            .rev()
            .map(|ancestor| self.within(importer, *ancestor))
            .collect()
    }

    /// True when `module` has a `pub(in path)` visibility, so what it exports
    /// to an importer can change as the tables settle.
    pub(crate) fn exports_by_path(&self, module: usize) -> bool {
        self.globs.in_path.get(module).copied().unwrap_or(false)
    }

    /// The reads of the globs of `module`.
    fn reads_of(&self, module: usize) -> &[GlobRead] {
        self.globs.reads.get(module).map_or(&[], Vec::as_slice)
    }

    /// The views of `module`.
    fn exports_of(&self, module: usize) -> &[usize] {
        self.globs.exports.get(module).map_or(&[], Vec::as_slice)
    }

    /// Each read of the globs of `module` in the settled tables: the glob
    /// import, the module it reaches and the view of that module it reads.
    pub(crate) fn glob_reads(
        &self,
        module: usize,
    ) -> impl Iterator<Item = (usize, usize, usize)> + '_ {
        self.reads_of(module).iter().map(|read| {
            tally::add(Work::ViewReads, 1);
            (read.glob, read.target, read.view)
        })
    }

    /// True when `view` holds `id` under `name` in the name space at `at`.
    pub(crate) fn view_holds(&self, view: usize, name: &str, at: usize, id: ItemId) -> bool {
        tally::add(Work::ViewReads, 1);
        self.globs.views[view]
            .names
            .get(name)
            .is_some_and(|lists| lists[at].binary_search_by_key(&id, |held| held.0).is_ok())
    }

    /// The items `view` holds under `name` in the name space at `at`, sorted.
    pub(crate) fn view_ids(
        &self,
        view: usize,
        name: &str,
        at: usize,
    ) -> impl Iterator<Item = ItemId> + '_ {
        tally::add(Work::ViewReads, 1);
        self.globs.views[view]
            .names
            .get(name)
            .map_or(&[][..], |lists| lists[at].as_slice())
            .iter()
            .map(|held| held.0)
    }

    /// The names `view` holds, in order.
    pub(crate) fn view_names(&self, view: usize) -> impl Iterator<Item = &str> + '_ {
        self.globs.views[view].names.keys().map(|name| {
            tally::add(Work::ViewReads, 1);
            name.as_str()
        })
    }

    /// The items the globs of `module` bring in under `name` in the name
    /// space at `at`, each with the route of the glob and target that rank
    /// first and the visibilities of every glob that brings it in: no wider
    /// than the glob, nor than the binding it reads.
    pub(crate) fn globbed(&self, module: usize, name: &str, at: usize) -> Bound {
        let mut bound = Bound::new();
        for (read, held) in self.delivered(module, name, at) {
            let decl = &self.uses[read.glob];
            let own = self.level_of(&decl.vis, module);
            for (id, level) in &held {
                let met = self.meet(own, *level);
                let vis = if met == own {
                    decl.vis.clone()
                } else {
                    self.vis_at(met, module)
                };
                bound.offer(*id, Self::glob_origin(decl, vis, read.target, at, name));
            }
        }
        bound
    }

    /// What each glob read of `module` delivers under `name` in the name
    /// space at `at`: the items of the view it reads, less the companions of
    /// an ambiguity when another glob of the module delivers an item of it
    /// as itself, from a module that binds the name itself. A view holds the
    /// items of a name that its target brings in through globs the class
    /// cannot access as companions of the ones it can (see
    /// [`Index::exposed`]). rustc drops such an ambiguity from a module that
    /// already holds the item it was carried with through a glob of another
    /// module, as when a cycle of globs delivers that item back, so the
    /// companions do not count there. Only a module that binds the name
    /// itself counts as that other module: what it binds there no glob of
    /// `module` passed on to it. An item a cycle of globs carries back
    /// through `module` itself would otherwise drop the very companions it
    /// was passed on with, or not, by whether it came back before or after
    /// them, and a view that already passed the companions on keeps them.
    fn delivered(&self, module: usize, name: &str, at: usize) -> Vec<(&GlobRead, Vec<Held>)> {
        let mut found: Vec<(&GlobRead, Vec<Held>)> = Vec::new();
        for read in self.reads_of(module) {
            let Some(lists) = self.globs.views[read.view].names.get(name) else {
                continue;
            };
            if !lists[at].is_empty() {
                found.push((read, lists[at].clone()));
            }
        }
        let firm: Vec<Vec<ItemId>> = found
            .iter()
            .map(|(_, held)| {
                held.iter()
                    .filter(|entry| !entry.1.is_companion())
                    .map(|entry| entry.0)
                    .collect()
            })
            .collect();
        // Whether each read's target binds the name itself, as a definition
        // or an explicit import: what such a read delivers no glob of
        // `module` passed on to it.
        let own: Vec<bool> = found
            .iter()
            .map(|(read, _)| {
                self.scopes[read.target]
                    .get(name)
                    .is_some_and(|entry| !entry.defs[at].is_empty())
            })
            .collect();
        for this in 0..found.len() {
            let redelivered = (0..found.len()).any(|other| {
                own[other]
                    && found[other].0.view != found[this].0.view
                    && firm[other].iter().any(|id| firm[this].contains(id))
            });
            if redelivered {
                found[this].1.retain(|entry| !entry.1.is_companion());
            }
        }
        found
    }

    /// True when a glob of `module` brings in something under `name` in the
    /// name space at `at`.
    pub(crate) fn globbed_any(&self, module: usize, name: &str, at: usize) -> bool {
        self.reads_of(module).iter().any(|read| {
            self.globs.views[read.view]
                .names
                .get(name)
                .is_some_and(|lists| !lists[at].is_empty())
        })
    }

    /// True when an explicit import of `module` that binds `name` keeps the
    /// name from what its globs export in the name space at `at`: one that
    /// binds the name there or may still, or that a cycle left to the globs
    /// and that is not decided (see [`Index::left_to_globs`]), as rustc hides
    /// a glob binding of a name that a single import may still bind. It does
    /// so for every importer, also for one that cannot name the import: an
    /// import that binds the name hides the glob's binding from all of them.
    fn blocked_in(&self, module: usize, name: &str, at: usize) -> bool {
        self.imports_of(module, name)
            .iter()
            .any(|position| !self.use_free[*position][at] || self.left_to_globs(*position, at))
    }

    /// The items of a binding list of `module` that importers of `class` can
    /// access, each at the widest level of its routes they can access.
    fn defs_held(&self, module: usize, class: &[bool], defs: &Bound) -> Vec<Held> {
        defs.iter()
            .filter_map(|(id, origin)| {
                origin
                    .visibilities()
                    .map(|vis| self.level_of(vis, module))
                    .filter(|level| self.allowed(module, class, *level))
                    .reduce(|left, right| self.wider(left, right))
                    .map(|level| (*id, level))
            })
            .collect()
    }

    /// Of the items `held`, sorted by item, with repeats, keep each item once
    /// at its widest level.
    fn widest(&self, held: &mut Vec<Held>) {
        held.sort_unstable_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then(self.width(left.1).cmp(&self.width(right.1)))
        });
        let mut kept: Vec<Held> = Vec::with_capacity(held.len());
        for entry in held.drain(..) {
            match kept.last_mut() {
                Some(last) if last.0 == entry.0 => {
                    if !entry.1.is_companion() {
                        last.1 = last.1.plain();
                    }
                }
                _ => kept.push(entry),
            }
        }
        *held = kept;
    }

    /// What the glob at `glob` of `module` brings in of the items `held` of
    /// a view it reads, for importers of `class`: no wider than the glob.
    /// The items the importers can access go to `seen`, the others to
    /// `hidden`.
    fn glob_held(
        &self,
        module: usize,
        class: &[bool],
        glob: Level,
        held: &[Held],
        into: (&mut Vec<Held>, &mut Vec<Held>),
    ) {
        let (seen, hidden) = into;
        for (id, level) in held {
            let mut met = self.meet(glob, *level).plain();
            if level.is_companion() {
                met = met.companion();
            }
            if self.allowed(module, class, met) {
                seen.push((*id, met));
            } else {
                hidden.push((*id, met));
            }
        }
    }

    /// What a name stands for to the importers of a class, given the items
    /// the globs of a module bring in under it that they can access (`seen`)
    /// and those they cannot (`hidden`), as rustc keeps them: a name that
    /// two globs bring in as different items is ambiguous in the module,
    /// however visible each glob is, and the ambiguity is as visible as the
    /// most visible of its items. So an importer that can access one of them
    /// reads them all as the name, and one that can access none reads
    /// nothing. The items they cannot access are held at the widest level of
    /// those they can, and the result only grows with either list. Sorted,
    /// one entry for each item.
    fn exposed(&self, mut seen: Vec<Held>, mut hidden: Vec<Held>) -> Vec<Held> {
        self.widest(&mut seen);
        let Some(widest) = seen
            .iter()
            .map(|held| held.1)
            .reduce(|left, right| self.wider(left, right))
        else {
            return seen;
        };
        hidden.retain(|(id, _)| seen.binary_search_by_key(id, |held| held.0).is_err());
        hidden.sort_unstable_by_key(|held| held.0);
        hidden.dedup_by_key(|held| held.0);
        if !ambiguity_seen(seen.len() + hidden.len(), seen.len()) {
            return seen;
        }
        seen.extend(hidden.into_iter().map(|(id, _)| (id, widest.companion())));
        seen.sort_unstable_by_key(|held| held.0);
        seen
    }

    /// What `target` exports under `name` in the name space at `at` to the
    /// importers of `class`: the bindings that shadow globs, when there are
    /// any, and those importers can access; else, when no explicit import
    /// they can name keeps the name from the globs, what the globs bring in
    /// (see [`Index::exposed`]). Sorted.
    fn export_ids(&self, target: usize, class: &[bool], name: &str, at: usize) -> Vec<Held> {
        if let Some(defs) = self.scopes[target]
            .get(name)
            .map(|entry| &entry.defs[at])
            .filter(|defs| !defs.is_empty())
        {
            return self.defs_held(target, class, defs);
        }
        if self.blocked_in(target, name, at) {
            return Vec::new();
        }
        let (seen, hidden) = self.split_globs_delivered(target, class, name, at);
        self.exposed(seen, hidden)
    }

    /// What the globs of `target` bring in under `name`, as the items the
    /// importers of `class` can access and those they cannot, from what each
    /// glob delivers (see [`Index::delivered`]).
    fn split_globs_delivered(
        &self,
        target: usize,
        class: &[bool],
        name: &str,
        at: usize,
    ) -> (Vec<Held>, Vec<Held>) {
        let (mut seen, mut hidden) = (Vec::new(), Vec::new());
        for (read, held) in self.delivered(target, name, at) {
            let glob = self.level_of(&self.uses[read.glob].vis, target);
            self.glob_held(target, class, glob, &held, (&mut seen, &mut hidden));
        }
        (seen, hidden)
    }

    /// Add `ids`, sorted by item with one entry for each, to the items of
    /// `name` in the name space at `at` of `view`, charging what that
    /// allocates. An item already held is taken again only at a wider level.
    /// Returns the entries that were new or wider.
    fn add_to_view(
        &mut self,
        view: usize,
        name: &str,
        at: usize,
        ids: &[Held],
        meter: &Meter,
    ) -> Result<Vec<Held>, LoadError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let held = self.globs.views[view].names.get(name);
        let fresh: Vec<Held> = ids
            .iter()
            .copied()
            .filter(|(id, level)| {
                held.is_none_or(|lists| {
                    lists[at]
                        .binary_search_by_key(id, |known| known.0)
                        .map_or(true, |place| {
                            let known = lists[at][place].1;
                            self.width(*level) < self.width(known)
                                || known.is_companion() && !level.is_companion()
                        })
                })
            })
            .collect();
        if fresh.is_empty() {
            return Ok(fresh);
        }
        let new = fresh
            .iter()
            .filter(|(id, _)| {
                held.is_none_or(|lists| {
                    lists[at].binary_search_by_key(id, |known| known.0).is_err()
                })
            })
            .count();
        let mut bytes = new * vec_slot::<Held>();
        if held.is_none_or(|lists| lists[at].is_empty()) {
            bytes += vec_first::<Held>();
        }
        // A name new to the view takes an entry, and its text the first time
        // any view holds it.
        let key = if held.is_some() {
            None
        } else {
            bytes += NAME_BYTES;
            let known = self.globs.strings.get(name).cloned();
            if known.is_none() {
                bytes += shared_text(name.len()) + hash_entry::<SharedStr>();
            }
            Some(known.unwrap_or_else(|| SharedStr::from(name)))
        };
        self.charge_tables(bytes, self.globs.views[view].site, meter)?;
        // Each entry as the view holds it once added: as wide as the wider
        // of the two routes, a companion only if both are.
        let finals: Vec<Held> = fresh
            .iter()
            .map(|entry| {
                let known = self.globs.views[view].names.get(name).and_then(|lists| {
                    lists[at]
                        .binary_search_by_key(&entry.0, |known| known.0)
                        .ok()
                        .map(|place| lists[at][place].1)
                });
                let Some(known) = known else { return *entry };
                let mut level = if self.width(entry.1) < self.width(known) {
                    entry.1.plain()
                } else {
                    known.plain()
                };
                if known.is_companion() && entry.1.is_companion() {
                    level = level.companion();
                }
                (entry.0, level)
            })
            .collect();
        let lists = match key {
            Some(key) => {
                self.globs.strings.insert(key.clone());
                self.globs.views[view].names.entry(key).or_default()
            }
            None => self.globs.views[view]
                .names
                .get_mut(name)
                .expect("the name is held"),
        };
        for entry in &finals {
            match lists[at].binary_search_by_key(&entry.0, |known| known.0) {
                Ok(place) => lists[at][place] = *entry,
                Err(place) => lists[at].insert(place, *entry),
            }
        }
        Ok(fresh)
    }

    /// Charge `bytes` of the tables, for the glob import at `site`.
    fn charge_tables(&mut self, bytes: usize, site: usize, meter: &Meter) -> Result<(), LoadError> {
        let decl = &self.uses[site];
        meter.globs(bytes, &decl.file, Some(decl.line))?;
        self.globs.charged += bytes;
        Ok(())
    }

    /// Charge `bytes` held while changes wait to be passed on, for the glob
    /// import at `site`; they are released once passed on.
    fn charge_work(&self, bytes: usize, site: usize, meter: &Meter) -> Result<(), LoadError> {
        let decl = &self.uses[site];
        meter.globs(bytes, &decl.file, Some(decl.line))
    }

    /// Make the view of `target` for `class`, for the glob import at `site`:
    /// what the target exports to the class as the tables stand. The
    /// target's globs that reach the view pass on to it from now on.
    fn make_view(
        &mut self,
        target: usize,
        class: &[bool],
        site: usize,
        meter: &Meter,
    ) -> Result<usize, LoadError> {
        self.charge_tables(VIEW_BYTES + 2 * class.len(), site, meter)?;
        let view = self.globs.views.len();
        self.globs.views.push(View {
            target,
            class: class.into(),
            names: BTreeMap::new(),
            active: Vec::new(),
            passive: BTreeMap::new(),
            site,
            hidden: false,
        });
        self.globs.view_of.insert((target, class.into()), view);
        self.globs.exports[target].push(view);
        // The target's reads whose globs reach the view now pass on what
        // they bring in.
        for read in 0..self.globs.reads[target].len() {
            let read_view = self.globs.reads[target][read].view;
            let readers = &mut self.globs.views[read_view];
            if let Some(list) = readers.passive.get_mut(&target)
                && let Some(place) = list.iter().position(|known| *known == read)
            {
                list.remove(place);
                if list.is_empty() {
                    readers.passive.remove(&target);
                }
                readers.active.push((target, read));
            }
        }
        // What the target exports to the class, gathered in one pass over
        // its bindings and over the views its globs that reach the class
        // read, then added to the view. Every read of one view brings in the
        // same lists, so each view is gathered once, however many globs of
        // the target read it. What the gathering allocates is charged before
        // it is allocated, as a change waiting to be passed on, and released
        // once it is added.
        let reads = self.reads_of(target).len();
        let listed = vec_first::<(usize, Level)>() + reads * vec_slot::<(usize, Level)>();
        self.charge_work(listed, site, meter)?;
        let mut sources: Vec<(usize, Level)> = self
            .reads_of(target)
            .iter()
            .map(|read| (read.view, self.level_of(&self.uses[read.glob].vis, target)))
            .collect();
        sources.sort_unstable();
        sources.dedup();
        let gathered = self.gather_bytes(target, &sources);
        self.charge_work(gathered, site, meter)?;
        let (content, any_hidden) = self.gather(target, class, &sources);
        drop(sources);
        meter.release_globs(listed);
        self.globs.views[view].hidden = any_hidden;
        for (name, lists) in content {
            for (at, ids) in lists.into_iter().enumerate() {
                self.add_to_view(view, &name, at, &ids, meter)?;
            }
        }
        meter.release_globs(gathered);
        Ok(view)
    }

    /// What `target` exports to `class` under each name, as the tables
    /// stand: its own bindings that shadow globs, and what the views
    /// `sources` hold, by [`Index::exposed`], and whether any of the items
    /// the globs bring in is one the class cannot access.
    fn gather(
        &self,
        target: usize,
        class: &[bool],
        sources: &[(usize, Level)],
    ) -> (Exported, bool) {
        let mut content: BTreeMap<&str, [Vec<Held>; 3]> = BTreeMap::new();
        let mut outside: BTreeMap<(&str, usize), Vec<Held>> = BTreeMap::new();
        let mut blocked: BTreeMap<(&str, usize), bool> = BTreeMap::new();
        for (name, entry) in &self.scopes[target] {
            for (at, defs) in entry.defs.iter().enumerate() {
                let held = self.defs_held(target, class, defs);
                if !held.is_empty() {
                    content.entry(name.as_str()).or_default()[at] = held;
                }
            }
        }
        for (source, glob) in sources {
            for (name, lists) in &self.globs.views[*source].names {
                for (at, held) in lists.iter().enumerate() {
                    if held.is_empty()
                        || self.scopes[target]
                            .get(name.as_str())
                            .is_some_and(|entry| !entry.defs[at].is_empty())
                        || *blocked
                            .entry((name.as_str(), at))
                            .or_insert_with(|| self.blocked_in(target, name, at))
                    {
                        continue;
                    }
                    let mut hidden = Vec::new();
                    self.glob_held(
                        target,
                        class,
                        *glob,
                        held,
                        (
                            &mut content.entry(name.as_str()).or_default()[at],
                            &mut hidden,
                        ),
                    );
                    if !hidden.is_empty() {
                        outside
                            .entry((name.as_str(), at))
                            .or_default()
                            .extend(hidden);
                    }
                }
            }
        }
        let mut any_hidden = false;
        let content: Exported = content
            .into_iter()
            .map(|(name, mut lists)| {
                for (at, ids) in lists.iter_mut().enumerate() {
                    let hidden = outside.remove(&(name, at)).unwrap_or_default();
                    any_hidden |= !hidden.is_empty();
                    // Items the class cannot access, and companions, are
                    // read again from the globs, as each delivers them (see
                    // [`Index::delivered`]).
                    *ids = if hidden.is_empty() && !ids.iter().any(|held| held.1.is_companion()) {
                        self.exposed(std::mem::take(ids), hidden)
                    } else {
                        self.export_ids(target, class, name, at)
                    };
                }
                (name.to_owned(), lists)
            })
            .collect();
        (content, any_hidden)
    }

    /// What gathering what `target` exports from its own bindings and from
    /// the views `sources` allocates at most: for each name of each, an entry
    /// of the gathered table and of the list it is turned into, with a copy
    /// of its text, the first room of its three lists and a slot for each
    /// item, a second for the items the class cannot access, and entries for
    /// whether an import keeps it from the globs and for the items the class
    /// cannot access, in each name space.
    fn gather_bytes(&self, target: usize, sources: &[(usize, Level)]) -> usize {
        const NAME: usize = btree_entry::<&str, [Vec<Held>; 3]>()
            + vec_slot::<(String, [Vec<Held>; 3])>()
            + 3 * vec_first::<Held>();
        let own: usize = self.scopes[target]
            .iter()
            .map(|(name, entry)| {
                NAME + name.len()
                    + entry
                        .defs
                        .iter()
                        .map(|defs| defs.iter().count() * vec_slot::<Held>())
                        .sum::<usize>()
            })
            .sum();
        let globbed: usize = sources
            .iter()
            .flat_map(|(source, _)| &self.globs.views[*source].names)
            .map(|(name, lists)| {
                NAME + name.len()
                    + 3 * (btree_entry::<(&str, usize), bool>()
                        + btree_entry::<(&str, usize), Vec<Held>>()
                        + vec_first::<Held>())
                    + lists
                        .iter()
                        .map(|ids| 2 * ids.len() * vec_slot::<Held>())
                        .sum::<usize>()
            })
            .sum();
        btree_map::<&str, [Vec<Held>; 3]>()
            + btree_map::<(&str, usize), bool>()
            + vec_first::<(String, [Vec<Held>; 3])>()
            + own
            + globbed
    }

    /// Record that the glob import at `glob` reaches `target`: it reads the
    /// view of the target for its module's class, made if there is none, and
    /// what the view holds is passed on to its module. Reading the same view
    /// again does nothing.
    pub(crate) fn read_glob(
        &mut self,
        glob: usize,
        target: usize,
        waiters: &mut dyn Waiters,
        meter: &Meter,
    ) -> Result<(), LoadError> {
        let module = self.uses[glob].module;
        let class = self.class_for(target, module);
        if let Some(view) = self.globs.view_of.get(&(target, class.clone()))
            && self.globs.read_by.contains(&(glob, *view))
        {
            return Ok(());
        }
        let view = match self.globs.view_of.get(&(target, class.clone())) {
            Some(view) => *view,
            None => self.make_view(target, &class, glob, meter)?,
        };
        self.charge_tables(READ_BYTES, glob, meter)?;
        let read = self.globs.reads[module].len();
        self.globs.reads[module].push(GlobRead { glob, target, view });
        self.globs.read_by.insert((glob, view));
        let level = self.level_of(&self.uses[glob].vis, module);
        // A glob of the module with the same level already reads the view:
        // this one passes on nothing more, so it is neither an active nor a
        // passive reader, and a module that reads one table through many
        // such globs passes on what the table holds once.
        let first = self.globs.passing.insert((module, view, level));
        let passes = first && !self.exports_of(module).is_empty();
        if !passes {
            if first {
                self.globs.views[view]
                    .passive
                    .entry(module)
                    .or_default()
                    .push(read);
            }
            for name in waiters.waited_in(module) {
                if self.globs.views[view].names.contains_key(name.as_str()) {
                    waiters.wake(module, &name);
                }
            }
            return Ok(());
        }
        self.globs.views[view].active.push((module, read));
        // Each name of the view, one at a time: passing one on can add to
        // this view too, through a cycle of globs, and what it adds is passed
        // on then.
        let mut after: Option<SharedStr> = None;
        loop {
            let next = match &after {
                None => self.globs.views[view].names.iter().next(),
                Some(name) => self.globs.views[view]
                    .names
                    .range::<SharedStr, _>((Excluded(name), Unbounded))
                    .next(),
            };
            let Some((name, lists)) = next.map(|(name, lists)| (name.clone(), lists.clone()))
            else {
                return Ok(());
            };
            for (at, ids) in lists.iter().enumerate() {
                if !ids.is_empty() {
                    let mut work = Vec::new();
                    self.arrive(module, read, &name, at, ids, &mut work, waiters, meter)?;
                    self.pass_on(work, glob, waiters, meter)?;
                }
            }
            after = Some(name);
        }
    }

    /// Recompute what `module` exports under `name` to each of its classes,
    /// after its bindings of the name or an import of the name changed, and
    /// pass on what that adds.
    pub(crate) fn refresh(
        &mut self,
        module: usize,
        name: &str,
        waiters: &mut dyn Waiters,
        meter: &Meter,
    ) -> Result<(), LoadError> {
        for view in self.exports_of(module).to_vec() {
            let class = self.globs.views[view].class.clone();
            for at in 0..3 {
                let ids = self.export_ids(module, &class, name, at);
                let fresh = self.add_to_view(view, name, at, &ids, meter)?;
                if !fresh.is_empty() {
                    let grown = Grown {
                        view,
                        name: self.globs.strings.get(name).expect("interned").clone(),
                        at,
                        ids: fresh,
                    };
                    self.pass_on(vec![grown], self.globs.views[view].site, waiters, meter)?;
                }
            }
        }
        Ok(())
    }

    /// Pass on every change in `work`, and every change that passing one on
    /// makes, until none is left, charging what the list holds while it holds
    /// it, to the glob import at `site` for the changes it starts with and to
    /// the glob that passes each change on for those it makes.
    fn pass_on(
        &mut self,
        mut work: Vec<Grown>,
        site: usize,
        waiters: &mut dyn Waiters,
        meter: &Meter,
    ) -> Result<(), LoadError> {
        // The room of the list is charged as it grows and released at the end;
        // each change's items while the change is held.
        let mut room = work.capacity() * size_of::<Grown>();
        self.charge_work(
            room + work.iter().map(Grown::bytes).sum::<usize>(),
            site,
            meter,
        )?;
        while let Some(grown) = work.pop() {
            // The globs that pass nothing on: only what waits on the name in
            // their modules is told.
            let passive = &self.globs.views[grown.view].passive;
            if !passive.is_empty()
                && let Some(modules) = waiters.waiting_on(&grown.name)
            {
                let woken: Vec<usize> = if modules.len() <= passive.len() {
                    modules
                        .iter()
                        .copied()
                        .filter(|module| passive.contains_key(module))
                        .collect()
                } else {
                    passive
                        .keys()
                        .copied()
                        .filter(|module| modules.contains(module))
                        .collect()
                };
                for module in woken {
                    waiters.wake(module, &grown.name);
                }
            }
            // Passing on adds names to views, never readers, so the list of
            // readers stays as it is meanwhile.
            for reader in 0..self.globs.views[grown.view].active.len() {
                let (module, read) = self.globs.views[grown.view].active[reader];
                let before = work.len();
                self.arrive(
                    module,
                    read,
                    &grown.name,
                    grown.at,
                    &grown.ids,
                    &mut work,
                    waiters,
                    meter,
                )?;
                let glob = self.globs.reads[module][read].glob;
                let grew = (work.capacity() * size_of::<Grown>()).saturating_sub(room);
                room += grew;
                self.charge_work(
                    grew + work[before..].iter().map(Grown::bytes).sum::<usize>(),
                    glob,
                    meter,
                )?;
            }
            meter.release_globs(grown.bytes());
        }
        meter.release_globs(room);
        Ok(())
    }

    /// Pass on what the read at `read` of the globs of `module` brought in,
    /// the items `ids` of `name` in the name space at `at`: tell what waits
    /// on the name there, and add the items to each view of the module that
    /// the glob reaches where nothing shadows the globs, queueing what that
    /// adds in `work`.
    #[expect(
        clippy::too_many_arguments,
        reason = "one change passed on, the place it arrives at and the two sinks"
    )]
    fn arrive(
        &mut self,
        module: usize,
        read: usize,
        name: &SharedStr,
        at: usize,
        ids: &[Held],
        work: &mut Vec<Grown>,
        waiters: &mut dyn Waiters,
        meter: &Meter,
    ) -> Result<(), LoadError> {
        waiters.wake(module, name);
        if self.scopes[module]
            .get(name.as_str())
            .is_some_and(|entry| !entry.defs[at].is_empty())
        {
            return Ok(());
        }
        let glob = self.globs.reads[module][read].glob;
        let level = self.level_of(&self.uses[glob].vis, module);
        if self.blocked_in(module, name, at) {
            return Ok(());
        }
        for view in self.exports_of(module).to_vec() {
            let class = self.globs.views[view].class.clone();
            let (mut seen, mut hidden) = (Vec::new(), Vec::new());
            self.glob_held(module, &class, level, ids, (&mut seen, &mut hidden));
            let known = self.globs.views[view].names.get(name.as_str());
            let held = known.map_or(&[][..], |lists| lists[at].as_slice());
            let passed = if !seen.is_empty() {
                // Items the class can access arrive. Where the globs of the
                // module bring in items it cannot access, those join them,
                // whichever glob they came through, so the table of the
                // module is read again for the name. So it is where a
                // companion arrives: another glob of the module may deliver
                // the item it was carried with, which drops it (see
                // [`Index::delivered`]).
                if self.globs.views[view].hidden || !hidden.is_empty() {
                    self.globs.views[view].hidden = true;
                    self.export_ids(module, &class, name, at)
                } else if seen.iter().any(|held| held.1.is_companion()) {
                    self.export_ids(module, &class, name, at)
                } else {
                    self.widest(&mut seen);
                    seen
                }
            } else if let Some(widest) = held
                .iter()
                .map(|held| held.1)
                .reduce(|left, right| self.wider(left, right))
            {
                // Nothing the class can access arrives, and what it already
                // reads under the name is now ambiguous with the items that
                // arrive.
                self.globs.views[view].hidden = true;
                hidden.sort_unstable_by_key(|held| held.0);
                hidden.dedup_by_key(|held| held.0);
                hidden
                    .into_iter()
                    .map(|(id, _)| (id, widest.companion()))
                    .collect()
            } else {
                self.globs.views[view].hidden |= !hidden.is_empty();
                continue;
            };
            let fresh = self.add_to_view(view, name, at, &passed, meter)?;
            if !fresh.is_empty() {
                work.push(Grown {
                    view,
                    name: name.clone(),
                    at,
                    ids: fresh,
                });
            }
        }
        Ok(())
    }

    /// Record that a glob of `module` reaches `target`, charging the edge
    /// the first time, at the glob import at `glob`.
    pub(crate) fn record_glob_edge(
        &mut self,
        glob: usize,
        module: usize,
        target: usize,
        meter: &Meter,
    ) -> Result<(), LoadError> {
        let decl = &self.uses[glob];
        if self.import_reach.reaches.is_empty() {
            meter.globs(
                self.modules.len() * (vec_slot::<bool>() + vec_slot::<BTreeSet<usize>>())
                    + vec_first::<bool>()
                    + vec_first::<BTreeSet<usize>>(),
                &decl.file,
                Some(decl.line),
            )?;
            let mut reaches = vec![false; self.modules.len()];
            for position in &self.named_imports {
                reaches[self.uses[*position].module] = true;
            }
            self.import_reach.reaches = reaches;
            self.import_reach.globbed_by = vec![BTreeSet::new(); self.modules.len()];
        }
        if self.import_reach.globbed_by[target].contains(&module) {
            return Ok(());
        }
        meter.globs(
            btree_map::<usize, ()>() + btree_entry::<usize, ()>(),
            &decl.file,
            Some(decl.line),
        )?;
        self.import_reach.globbed_by[target].insert(module);
        if !self.import_reach.reaches[target] {
            return Ok(());
        }
        let mut work = vec![module];
        while let Some(next) = work.pop() {
            if !std::mem::replace(&mut self.import_reach.reaches[next], true) {
                work.extend(self.import_reach.globbed_by[next].iter().copied());
            }
        }
        Ok(())
    }

    /// False when no module that `module` reaches through globs, itself
    /// among them, has an explicit import: then no glob of the module may
    /// still bring in a name an import keeps from the globs.
    pub(crate) fn reaches_import(&self, module: usize) -> bool {
        self.import_reach
            .reaches
            .get(module)
            .is_none_or(|reaches| *reaches)
    }
}
