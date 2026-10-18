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
//! there. What reads and views hold only grows, so a drop that the settled
//! tables show is made from the start when the tables settle again: then it
//! does not depend on whether the item that justifies it came in before or
//! after the companions.
//!
//! A view grows as the scope tables settle. What it gains is passed on only
//! to the globs that read it, only with the names that gained something, and
//! only where it can matter: to the views of the reading module, and to the
//! imports that wait on one of those names in the reading module. A glob
//! whose module has no view passes nothing on, and costs nothing for each
//! name its view gains. So a chain of globs costs what it brings in, once for
//! each link, and a module that a thousand modules glob is read by a thousand
//! globs from one table. Three shapes cost more than that, because each
//! companion that arrives at a module reads and copies the lists of all its
//! reads again: a fan-in of private globs onto one module (a quadratic term),
//! the prelude pattern, where a root globs many children and each child globs
//! its parent (quadratic), and a hub that many siblings glob and that globs
//! them back, partly privately (about cubic). A glob that reads a view another glob of its module
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
//! declaration before their repeats are removed, the bindings with their
//! routes that a lookup of what the globs of a module bring in under one name
//! returns, the names that hold companions, listed while the drops the
//! settled tables show are recorded. The search for the modules that hold an
//! item apart from a reading module is charged for each step it makes (see
//! `SUPPLY_STEP_BYTES`); its tree of dominators, one entry for each module of
//! the holder graph, is built once for an item and name and dropped when the
//! drops are recorded.

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

/// The glob reads whose companions are dropped: for each glob import and
/// module it reaches, the names whose companions the read does not deliver,
/// in each name space, to the importers that can access the widest level
/// given (see [`Index::delivered`]).
type Drops = BTreeMap<(usize, usize), BTreeMap<SharedStr, [Option<Level>; 3]>>;

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
    /// The glob reads whose companions are dropped, decided from the
    /// settled tables of an earlier settle of the load and kept, with the
    /// depths, when the tables are cleared (see [`Index::record_drops`]).
    drops: Drops,
    /// How many names of glob reads, each in one name space, `drops` holds.
    dropped: usize,
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

/// The modules that hold an item under a name apart from a reading module,
/// found once for each item, name and name space over the settled tables
/// while their drops are recorded (see [`Index::record_drops`]).
#[derive(Default)]
struct Supply {
    /// For each module, the glob reads of other modules that read one of
    /// its views: the reading module and the place of the read among its
    /// reads.
    readers: Vec<Vec<(usize, usize)>>,
    /// For each name and name space, the modules that bind the name there
    /// themselves.
    binders: BTreeMap<(SharedStr, usize), Vec<usize>>,
    /// For each item, name and name space, the modules that hold the item as
    /// itself under the name and which of them every way to each passes
    /// through (see [`Holding`]).
    holdings: BTreeMap<(ItemId, SharedStr, usize), Holding>,
    /// A refusal by the budget during a search, returned once it ends.
    refused: Option<LoadError>,
}

/// The modules that hold an item as itself under a name, and their
/// dominators. The holders are the modules that bind the name themselves
/// with the item, and those a glob read brings it in to from a holder,
/// through a view that holds it as itself, where the reading module does not
/// bind the name itself. From a root above the modules that bind it, a
/// holder dominates another when every way from the root to the other passes
/// through it: a holder keeps the item apart from a module exactly when the
/// module does not dominate it, as leaving the module out leaves a way to it.
#[derive(Default)]
struct Holding {
    /// The place of each holder in the tree of dominators, as the entry and
    /// the exit of a walk of the tree: one dominates another when its span
    /// holds the other's.
    span: BTreeMap<usize, (usize, usize)>,
}

impl Holding {
    /// True when `module` holds the item.
    fn holds(&self, module: usize) -> bool {
        self.span.contains_key(&module)
    }

    /// True when every way to the holder `held` passes through `module`.
    fn dominates(&self, module: usize, held: usize) -> bool {
        match (self.span.get(&module), self.span.get(&held)) {
            (Some(outer), Some(inner)) => outer.0 <= inner.0 && inner.1 <= outer.1,
            _ => false,
        }
    }
}

/// What the search for the holders of an item charges for each glob read it
/// looks at, each holder it finds and each step of finding their dominators:
/// an entry of a set of modules.
const SUPPLY_STEP_BYTES: usize = btree_entry::<usize, ()>();

/// No node: the parent or ancestor of the root of the holder graph.
const NONE: usize = usize::MAX;

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
        let drops = std::mem::take(&mut self.globs.drops);
        let dropped = self.globs.dropped;
        self.globs = GlobTables {
            depth,
            in_path,
            drops,
            dropped,
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
        for (read, held) in self.delivered(module, name, at, None) {
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
    /// space at `at`: the items of the view it reads, less its companions
    /// where the read drops them. A view holds the items of a name that its
    /// target brings in through globs the class cannot access as companions
    /// of the ones it can (see [`Index::exposed`]). rustc drops such an
    /// ambiguity from a module that already holds the item it was carried
    /// with through a glob of another module, as when a cycle of globs
    /// delivers that item back, so the companions do not count there (see
    /// [`Index::redelivered`]). A read drops them once the tables show that,
    /// and from the start of every later settle of the load when the settled
    /// tables of an earlier one showed it (see [`Index::record_drops`]): the
    /// item that delivers the companions' item can come in after a path or a
    /// view read them, and what a path or a view read under a name only
    /// grows, so a drop that only came then would leave the answer to the
    /// order of the declarations. With `class`, what the importers of that
    /// class read of the module's globs: a read drops its companions to them
    /// only when they can access the item that justifies the drop as the
    /// module holds it. Without, what the module itself reads. While the
    /// tables settle, only a read from a module that binds the name itself
    /// justifies a drop; one whose globs bring the item in is asked only of
    /// the settled tables (see [`Index::record_drops`]).
    fn delivered(
        &self,
        module: usize,
        name: &str,
        at: usize,
        class: Option<&[bool]>,
    ) -> Vec<(&GlobRead, Vec<Held>)> {
        let mut found = self.read_lists(module, name, at);
        let redelivered = self.redelivered(module, &found, name, at, &mut |_, _| false);
        let seen = |level: Option<Level>| {
            level.is_some_and(|level| class.is_none_or(|class| self.allowed(module, class, level)))
        };
        for ((read, held), redelivered) in found.iter_mut().zip(redelivered) {
            if seen(redelivered) || seen(self.drop_level(read, name, at)) {
                held.retain(|entry| !entry.1.is_companion());
            }
        }
        found
    }

    /// The items of the view each glob read of `module` reads under `name`
    /// in the name space at `at`, for the reads whose view holds any.
    fn read_lists(&self, module: usize, name: &str, at: usize) -> Vec<(&GlobRead, Vec<Held>)> {
        let mut found: Vec<(&GlobRead, Vec<Held>)> = Vec::new();
        for read in self.reads_of(module) {
            let Some(lists) = self.globs.views[read.view].names.get(name) else {
                continue;
            };
            if !lists[at].is_empty() {
                found.push((read, lists[at].clone()));
            }
        }
        found
    }

    /// For each glob read of `found`, the reads of `module` and the items
    /// their views hold under `name` in the name space at `at`, the widest
    /// level at which `module` holds, through another of them, an item that
    /// the read delivers as itself beside companions, if it does: the other
    /// read delivers the item as itself and none of the read's companions,
    /// from a module that holds it apart from `module`, one that binds the
    /// name itself, as a definition or an explicit import, or, as `supplied`
    /// tells for that module and the item, one whose globs bring the item in
    /// from such a module through modules other than `module` (see
    /// [`Index::record_drops`]). That module already holds the item when a
    /// cycle of globs carries it back to `module` with the companions, so
    /// rustc drops them, for the importers that can access the item as it
    /// holds it: to the others, which see only the ambiguity, it stays (see
    /// [`Index::delivered`]). An item a cycle of globs carries back through
    /// `module` itself would otherwise drop the very companions it was
    /// passed on with. A binding of the name that reads it through the
    /// module's own globs, and so through the read itself, holds the
    /// companions as well while the read delivers them, so it never
    /// justifies their drop: rustc reads that binding as the ambiguity.
    ///
    /// The reads are answered together: for each item a read with
    /// companions delivers as itself, the reads that deliver it are listed
    /// once, the widest first, and the reads with the same companions, which
    /// leave out the same reads, take for each item the widest read left and
    /// the widest of another view, asking `supplied` only of those, in that
    /// order. So the answer costs a step for each item each read holds and
    /// each read left out for each set of companions, and not one for each
    /// pair of reads.
    fn redelivered(
        &self,
        module: usize,
        found: &[(&GlobRead, Vec<Held>)],
        name: &str,
        at: usize,
        supplied: &mut dyn FnMut(usize, ItemId) -> bool,
    ) -> Vec<Option<Level>> {
        let companions: Vec<Vec<ItemId>> = found
            .iter()
            .map(|(_, held)| {
                held.iter()
                    .filter(|entry| entry.1.is_companion())
                    .map(|entry| entry.0)
                    .collect()
            })
            .collect();
        let mut answer = vec![None; found.len()];
        if companions.iter().all(Vec::is_empty) {
            return answer;
        }
        // The items asked: those a read with companions delivers as itself.
        let asked: BTreeSet<ItemId> = found
            .iter()
            .zip(&companions)
            .filter(|(_, companions)| !companions.is_empty())
            .flat_map(|((_, held), _)| held.iter())
            .filter(|entry| !entry.1.is_companion())
            .map(|entry| entry.0)
            .collect();
        let hidden: BTreeSet<ItemId> = companions.iter().flatten().copied().collect();
        // For each item asked, the reads that deliver it as itself, at the
        // level `module` holds it through each, the widest first, with
        // whether the read's module binds the name itself; and for each
        // companion, the reads that deliver it in any way. Whether a module
        // holds the item apart from `module` is asked only of a read that
        // is not left out, in that order, until the answer is known.
        let mut supporters: BTreeMap<ItemId, Vec<(Level, usize, bool)>> = BTreeMap::new();
        let mut carriers: BTreeMap<ItemId, Vec<usize>> = BTreeMap::new();
        for (other, (read, held)) in found.iter().enumerate() {
            let own = self.binds_itself(read.target, name, at);
            let glob = self.level_of(&self.uses[read.glob].vis, module);
            for (id, level) in held {
                if hidden.contains(id) {
                    carriers.entry(*id).or_default().push(other);
                }
                if !level.is_companion() && asked.contains(id) {
                    supporters
                        .entry(*id)
                        .or_default()
                        .push((self.meet(glob, *level), other, own));
                }
            }
        }
        for list in supporters.values_mut() {
            list.sort_by_key(|(level, other, _)| (self.width(*level), *other));
        }
        // The reads with the same companions leave out the same supporters,
        // so they are answered together: for each item, the widest
        // supporter left, and the widest of another view than that one's.
        let mut groups: BTreeMap<&[ItemId], Vec<usize>> = BTreeMap::new();
        for (this, companions) in companions.iter().enumerate() {
            if !companions.is_empty() {
                groups.entry(companions.as_slice()).or_default().push(this);
            }
        }
        for (companions, members) in groups {
            let left_out: BTreeSet<usize> = companions
                .iter()
                .flat_map(|id| carriers.get(id).into_iter().flatten())
                .copied()
                .collect();
            let mut best: BTreeMap<ItemId, [Option<(Level, usize)>; 2]> = BTreeMap::new();
            for &this in &members {
                let view = found[this].0.view;
                for (id, _) in found[this].1.iter().filter(|entry| !entry.1.is_companion()) {
                    let pair = best.entry(*id).or_insert_with(|| {
                        let mut pair = [None, None];
                        for &(level, other, own) in supporters.get(id).into_iter().flatten() {
                            if left_out.contains(&other)
                                || !(own || supplied(found[other].0.target, *id))
                            {
                                continue;
                            }
                            let seen = found[other].0.view;
                            match pair {
                                [None, _] => pair[0] = Some((level, seen)),
                                [Some((_, first)), None] if first != seen => {
                                    pair[1] = Some((level, seen));
                                    break;
                                }
                                _ => {}
                            }
                        }
                        pair
                    });
                    let chosen = match *pair {
                        [Some((level, seen)), _] if seen != view => Some(level),
                        [_, Some((level, _))] => Some(level),
                        _ => None,
                    };
                    if let Some(level) = chosen {
                        answer[this] = Some(
                            answer[this].map_or(level, |known: Level| self.wider(known, level)),
                        );
                    }
                }
            }
        }
        answer
    }

    /// True when `module` binds `name` itself in the name space at `at`, as
    /// a definition or an explicit import.
    fn binds_itself(&self, module: usize, name: &str, at: usize) -> bool {
        self.scopes[module]
            .get(name)
            .is_some_and(|entry| !entry.defs[at].is_empty())
    }

    /// True when the view `view` holds `id` as itself, not as a companion,
    /// under `name` in the name space at `at`.
    fn holds_firm(&self, view: usize, name: &str, at: usize, id: ItemId) -> bool {
        self.globs.views[view].names.get(name).is_some_and(|lists| {
            lists[at]
                .binary_search_by_key(&id, |held| held.0)
                .is_ok_and(|place| !lists[at][place].1.is_companion())
        })
    }

    /// The widest level of the importers to which the glob read `read`
    /// drops its companions under `name` in the name space at `at` from the
    /// start of a settle, if it drops them.
    fn drop_level(&self, read: &GlobRead, name: &str, at: usize) -> Option<Level> {
        self.globs
            .drops
            .get(&(read.glob, read.target))
            .and_then(|names| names.get(name))
            .and_then(|spaces| spaces[at])
    }

    /// How many names of glob reads, each in one name space, drop their
    /// companions.
    pub(crate) fn drop_count(&self) -> usize {
        self.globs.dropped
    }

    /// Record, from the settled tables, every glob read that delivers an
    /// item beside companions under a name that another read of its module
    /// delivers from a module that holds it apart from the reading module
    /// (see [`Index::redelivered`]), and that does not drop its companions
    /// there from the start yet, charging what the record of each takes. The
    /// reads recorded drop their companions from the start of every later
    /// settle of the load, and the record only grows. Returns how many were
    /// recorded.
    ///
    /// A module holds the item apart from the reading module when it binds
    /// the name itself, which the settling tells too, or when its globs bring
    /// the item in from such a module through modules other than the reading
    /// one. That is only asked here, of tables that no longer change, and
    /// only for a read that holds companions: the modules that hold an item
    /// under a name are found once, from the modules that bind it, along the
    /// glob reads whose views hold it, with their dominators, which answer
    /// for every reading module at once (see [`Holding`]). So the search
    /// costs a step for each glob read of a holder, each holder and each
    /// step of finding the dominators, once for each item, name and name
    /// space asked, and each step is charged as an entry of a set of modules
    /// ([`SUPPLY_STEP_BYTES`]) before it is made.
    ///
    /// The names of each view that hold a companion, and for each module
    /// those of the views its globs read, are listed as temporaries that are
    /// not charged: each holds at most one entry for each name and name
    /// space of a view, which the views are charged for, the list of a
    /// module is dropped before the next module's is made, and the list of
    /// the views before the record grows. So are the readers of each module,
    /// one entry for each glob read, and the modules that bind each name
    /// asked, one entry for each binding of the scope tables, which both are
    /// charged for.
    pub(crate) fn record_drops(&mut self, meter: &Meter) -> Result<usize, LoadError> {
        let Some(&first) = self.glob_imports.first() else {
            return Ok(0);
        };
        let decl = &self.uses[first];
        let (file, line) = (decl.file.clone(), decl.line);
        let companions: Vec<Vec<(SharedStr, usize)>> = self
            .globs
            .views
            .iter()
            .map(|view| {
                view.names
                    .iter()
                    .flat_map(|(name, lists)| {
                        (0..3)
                            .filter(|at| lists[*at].iter().any(|entry| entry.1.is_companion()))
                            .map(|at| (name.clone(), at))
                    })
                    .collect()
            })
            .collect();
        let mut supply: Option<Supply> = None;
        let mut due: Vec<(usize, usize, SharedStr, usize, Level)> = Vec::new();
        for module in 0..self.globs.reads.len() {
            let reads = self.reads_of(module);
            if reads.len() < 2 {
                continue;
            }
            let names: BTreeSet<&(SharedStr, usize)> = reads
                .iter()
                .flat_map(|read| &companions[read.view])
                .collect();
            for (name, at) in names {
                let found = self.read_lists(module, name, *at);
                let supply = supply.get_or_insert_with(|| self.supply(&companions));
                let charge =
                    |steps: usize| meter.items(steps * SUPPLY_STEP_BYTES, &file, Some(line));
                let redelivered = self.redelivered(module, &found, name, *at, &mut |from, id| {
                    self.supplied(supply, &charge, from, module, name, *at, id)
                });
                if let Some(refused) = supply.refused.take() {
                    return Err(refused);
                }
                for ((read, _), redelivered) in found.iter().zip(redelivered) {
                    let Some(level) = redelivered else {
                        continue;
                    };
                    let known = self.drop_level(read, name, *at);
                    if known.is_none_or(|known| self.width(level) < self.width(known)) {
                        due.push((read.glob, read.target, name.clone(), *at, level));
                    }
                }
            }
        }
        drop(supply);
        drop(companions);
        let mut recorded = 0;
        for (glob, target, name, at, level) in due {
            let mut bytes = 0;
            if self.globs.drops.is_empty() {
                bytes += btree_map::<(usize, usize), BTreeMap<SharedStr, [Option<Level>; 3]>>();
            }
            let names = self.globs.drops.get(&(glob, target));
            if names.is_none() {
                bytes += btree_entry::<(usize, usize), BTreeMap<SharedStr, [Option<Level>; 3]>>()
                    + btree_map::<SharedStr, [Option<Level>; 3]>();
            }
            if names.and_then(|names| names.get(name.as_str())).is_none() {
                // The record keeps the text of the name after the views
                // that shared it are dropped.
                bytes += btree_entry::<SharedStr, [Option<Level>; 3]>() + shared_text(name.len());
            }
            meter.items(bytes, &file, Some(line))?;
            self.globs
                .drops
                .entry((glob, target))
                .or_default()
                .entry(name)
                .or_default()[at] = Some(level);
            self.globs.dropped += 1;
            recorded += 1;
        }
        Ok(recorded)
    }

    /// The readers of each module and the modules that bind each name of
    /// `companions` themselves, from the settled tables.
    fn supply(&self, companions: &[Vec<(SharedStr, usize)>]) -> Supply {
        let mut readers: Vec<Vec<(usize, usize)>> = vec![Vec::new(); self.modules.len()];
        for (module, reads) in self.globs.reads.iter().enumerate() {
            for (place, read) in reads.iter().enumerate() {
                readers[read.target].push((module, place));
            }
        }
        let asked: BTreeSet<SharedStr> = companions
            .iter()
            .flatten()
            .map(|(name, _)| name.clone())
            .collect();
        let mut binders: BTreeMap<(SharedStr, usize), Vec<usize>> = BTreeMap::new();
        for (module, scope) in self.scopes.iter().enumerate() {
            for (name, entry) in scope {
                let Some(name) = asked.get(name.as_str()) else {
                    continue;
                };
                for (at, defs) in entry.defs.iter().enumerate() {
                    if !defs.is_empty() {
                        binders.entry((name.clone(), at)).or_default().push(module);
                    }
                }
            }
        }
        Supply {
            readers,
            binders,
            ..Supply::default()
        }
    }

    /// True when `from` holds `id` as itself under `name` in the name space
    /// at `at` apart from the reading module `reader` (see
    /// [`Index::record_drops`]): it holds the item, and the reading module
    /// does not dominate it among the holders (see [`Holding`]), so the
    /// item reaches it by a way that does not pass through the reading
    /// module. A refusal by the budget answers false and is kept in
    /// `supply`.
    #[expect(
        clippy::too_many_arguments,
        reason = "one question, the search state and the budget it is charged to"
    )]
    fn supplied(
        &self,
        supply: &mut Supply,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
        from: usize,
        reader: usize,
        name: &SharedStr,
        at: usize,
        id: ItemId,
    ) -> bool {
        if supply.refused.is_some() || from == reader {
            return false;
        }
        let key = (id, name.clone(), at);
        if !supply.holdings.contains_key(&key) {
            match self.holding(supply, charge, name, at, id) {
                Ok(holding) => {
                    supply.holdings.insert(key.clone(), holding);
                }
                Err(refused) => {
                    supply.refused = Some(refused);
                    return false;
                }
            }
        }
        let holding = &supply.holdings[&key];
        // Decisive, not defensive: a holder that the item reaches only
        // through the reading module can still deliver it without the
        // reading module's companions, when a support that only importers
        // inside the reading module can see drops them from the view such a
        // holder reads. Its delivery would then widen the drop to every
        // importer; this check keeps it to the support's own level
        // (`a_drop_justified_by_a_holder_that_reads_the_item_back_from_the_reader_stays_within_the_reader_in_every_order`).
        holding.holds(from) && !holding.dominates(reader, from)
    }

    /// The holders of `id` under `name` in the name space at `at` and their
    /// dominators (see [`Holding`]): the holders are found from the modules
    /// that bind the name with the item, along the glob reads whose views
    /// hold it, and the dominators by the algorithm of Lengauer and Tarjan,
    /// with path compression. Each glob read looked at, each holder and each
    /// step of the algorithm is charged before it is made.
    fn holding(
        &self,
        supply: &Supply,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
        name: &SharedStr,
        at: usize,
        id: ItemId,
    ) -> Result<Holding, LoadError> {
        let mut step = |count: usize| {
            tally::add(Work::SupplySteps, count);
            charge(count)
        };
        // The graph: node 0 is the root, above each module that binds the
        // name with the item; an edge leads from a holder to a module that
        // a glob read of a view that holds the item brings it in to.
        let mut nodes: Vec<usize> = vec![NONE];
        let mut place: BTreeMap<usize, usize> = BTreeMap::new();
        let mut successors: Vec<Vec<usize>> = vec![Vec::new()];
        for &module in supply
            .binders
            .get(&(name.clone(), at))
            .into_iter()
            .flatten()
        {
            if self.scopes[module]
                .get(name.as_str())
                .is_some_and(|entry| entry.defs[at].get(id).is_some())
                && !place.contains_key(&module)
            {
                step(1)?;
                place.insert(module, nodes.len());
                successors[0].push(nodes.len());
                nodes.push(module);
                successors.push(Vec::new());
            }
        }
        let mut next = 1;
        while next < nodes.len() {
            let held = nodes[next];
            for &(module, at_read) in &supply.readers[held] {
                step(1)?;
                if self.binds_itself(module, name, at)
                    || !self.holds_firm(self.globs.reads[module][at_read].view, name, at, id)
                {
                    continue;
                }
                let node = if let Some(node) = place.get(&module) {
                    *node
                } else {
                    step(1)?;
                    place.insert(module, nodes.len());
                    nodes.push(module);
                    successors.push(Vec::new());
                    nodes.len() - 1
                };
                successors[next].push(node);
            }
            next += 1;
        }
        let dominator = dominators(&successors, &mut step)?;
        // Each holder's span in a walk of the tree of dominators.
        let mut children: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
        for (node, parent) in dominator.iter().enumerate().skip(1) {
            if *parent != NONE {
                children[*parent].push(node);
            }
        }
        let mut span = vec![(0, 0); nodes.len()];
        let mut clock = 0;
        let mut walk: Vec<(usize, usize)> = vec![(0, 0)];
        while let Some((node, child)) = walk.pop() {
            if child == 0 {
                span[node].0 = clock;
                clock += 1;
            }
            if let Some(&next) = children[node].get(child) {
                step(1)?;
                walk.push((node, child + 1));
                walk.push((next, 0));
            } else {
                span[node].1 = clock;
                clock += 1;
            }
        }
        Ok(Holding {
            span: place
                .into_iter()
                .map(|(module, node)| (module, span[node]))
                .collect(),
        })
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
        for (read, held) in self.delivered(target, name, at, Some(class)) {
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
            } else if !hidden.is_empty() && !held.is_empty() {
                // Nothing the class can access arrives, and what it already
                // reads under the name is now ambiguous with the items that
                // arrive, unless the read they arrive through drops them as
                // companions: the table of the module is read again for the
                // name (see [`Index::delivered`]).
                self.globs.views[view].hidden = true;
                self.export_ids(module, &class, name, at)
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

/// The immediate dominator of each node of the graph `successors`, whose
/// root is node 0 and whose every node the root reaches, by the algorithm of
/// Lengauer and Tarjan with path compression; [`NONE`] for the root. Each
/// edge looked at and each node numbered is counted to `step` before.
fn dominators(
    successors: &[Vec<usize>],
    step: &mut dyn FnMut(usize) -> Result<(), LoadError>,
) -> Result<Vec<usize>, LoadError> {
    let count = successors.len();
    let mut predecessors: Vec<Vec<usize>> = vec![Vec::new(); count];
    for (node, next) in successors.iter().enumerate() {
        for &to in next {
            step(1)?;
            predecessors[to].push(node);
        }
    }
    // Number the nodes in a depth-first walk from the root.
    let mut number = vec![NONE; count];
    let mut vertex: Vec<usize> = Vec::with_capacity(count);
    let mut parent = vec![NONE; count];
    let mut walk: Vec<(usize, usize)> = vec![(0, NONE)];
    while let Some((node, from)) = walk.pop() {
        if number[node] != NONE {
            continue;
        }
        step(1)?;
        number[node] = vertex.len();
        vertex.push(node);
        parent[node] = from;
        for &to in successors[node].iter().rev() {
            if number[to] == NONE {
                walk.push((to, node));
            }
        }
    }
    let mut semi: Vec<usize> = number.clone();
    let mut ancestor = vec![NONE; count];
    let mut label: Vec<usize> = (0..count).collect();
    let mut dominator = vec![NONE; count];
    let mut bucket: Vec<Vec<usize>> = vec![Vec::new(); count];
    let mut path: Vec<usize> = Vec::new();
    let mut eval =
        |node: usize, ancestor: &mut Vec<usize>, label: &mut Vec<usize>, semi: &[usize]| {
            if ancestor[node] == NONE {
                return node;
            }
            let mut at = node;
            while ancestor[ancestor[at]] != NONE {
                path.push(at);
                at = ancestor[at];
            }
            while let Some(at) = path.pop() {
                let up = ancestor[at];
                if semi[label[up]] < semi[label[at]] {
                    label[at] = label[up];
                }
                ancestor[at] = ancestor[up];
            }
            label[node]
        };
    for &node in vertex.iter().skip(1).rev() {
        for &from in &predecessors[node] {
            step(1)?;
            if number[from] == NONE {
                continue;
            }
            let least = eval(from, &mut ancestor, &mut label, &semi);
            if semi[least] < semi[node] {
                semi[node] = semi[least];
            }
        }
        bucket[vertex[semi[node]]].push(node);
        let up = parent[node];
        ancestor[node] = up;
        for waiting in std::mem::take(&mut bucket[up]) {
            step(1)?;
            let least = eval(waiting, &mut ancestor, &mut label, &semi);
            dominator[waiting] = if semi[least] < semi[waiting] {
                least
            } else {
                up
            };
        }
    }
    for &node in vertex.iter().skip(1) {
        if dominator[node] != vertex[semi[node]] {
            dominator[node] = dominator[dominator[node]];
        }
    }
    Ok(dominator)
}

#[cfg(test)]
mod tests {
    //! The search for the modules that hold an item apart from a reading
    //! module grows in step with the globs it searches, whatever the shape
    //! of the chain, and does not run for a read that holds no companion.

    use std::fmt::Write as _;

    use crate::work::{Work, measure_kind};
    use crate::{MemoryTree, Namespace, Workspace};

    /// The length of the shortest chain.
    const UNIT: usize = 40;

    /// The most the search may count while one crate loads: a search that
    /// runs once for each reader over the whole chain passes it at once.
    const CEILING: usize = 400 * UNIT;

    const DEMO: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";

    /// The modules every input starts with: `d` defines a struct `X`, `p`
    /// defines another, and `a` globs `d` publicly and `p` privately, so a
    /// reader of `a` reads `p::X` only as a companion of `d::X`.
    const BASE: &str = "pub mod d { pub struct X {} }\npub mod p { pub struct X {} }\n\
                        pub mod a { pub use crate::d::*; use crate::p::*; }\n";

    /// A chain of `n` pass-through globs of `d`, `c0` to `c{n-1}`, `c0`
    /// globbing `d` and the modules `extra` names, and `n` readers that each
    /// glob `a`, the end of the chain and what `more` names for the reader.
    fn chain(n: usize, extra: &str, more: &dyn Fn(usize) -> String) -> String {
        let mut lib = String::from(BASE);
        writeln!(lib, "pub mod c0 {{ pub use crate::d::*; {extra} }}").expect("write");
        for i in 1..n {
            writeln!(lib, "pub mod c{i} {{ pub use crate::c{}::*; }}", i - 1).expect("write");
        }
        for i in 0..n {
            writeln!(
                lib,
                "pub mod r{i} {{ pub use crate::a::*; pub use crate::c{}::*; {} }}",
                n - 1,
                more(i)
            )
            .expect("write");
        }
        lib
    }

    /// The chain, each reader holding the companion.
    fn plain(n: usize) -> String {
        chain(n, "", &|_| String::new())
    }

    /// The chain whose start also globs every reader: each reader holds
    /// the item through the chain and back from the readers, so each lies
    /// on a way to the others.
    fn fan_in(n: usize) -> String {
        let mut readers = String::new();
        for i in 0..n {
            write!(readers, "pub use crate::r{i}::*; ").expect("write");
        }
        chain(n, &readers, &|_| String::new())
    }

    /// The chain whose readers each also glob the one before: a second
    /// chain, of the readers, nested on the first.
    fn nested(n: usize) -> String {
        chain(n, "", &|i| {
            if i == 0 {
                String::new()
            } else {
                format!("pub use crate::r{}::*;", i - 1)
            }
        })
    }

    /// What loading `lib` counts of the search, and what `self::X` is read
    /// as from `demo::r0` in the type name space.
    fn counted(lib: &str) -> (usize, String) {
        let tree: MemoryTree = [("Cargo.toml", DEMO), ("src/lib.rs", lib)]
            .into_iter()
            .collect();
        let (ws, tally) = measure_kind(Work::SupplySteps, CEILING, || {
            Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"))
        });
        let found = ws
            .resolve_from_in("demo::r0", "self::X", Namespace::Type)
            .map_or_else(|e| format!("{e:?}"), |found| found.path);
        (tally.get(Work::SupplySteps), found)
    }

    /// The counts of the search for `build` at one, two and three units,
    /// each reader reading `d::X` alone, and the asserted growth: the count
    /// is nonzero and the third unit adds no more than the second.
    fn assert_linear(build: &dyn Fn(usize) -> String) {
        let counts = [1, 2, 3].map(|units| {
            let (count, found) = counted(&build(units * UNIT));
            assert_eq!(found, "demo::d::X");
            count
        });
        let [one, two, three] = counts;
        assert!(one > 0, "the search counted nothing: {counts:?}");
        assert!(
            three + one <= 2 * two,
            "the search grows faster than the chain: {counts:?}"
        );
    }

    #[test]
    fn the_holders_of_an_item_apart_from_each_reader_of_a_chain_of_globs_are_found_in_steps_linear_in_the_chain()
     {
        // The holders of `d::X` and their dominators are found once for the
        // load: searching the chain again for each reader grows with the
        // square of the chain.
        assert_linear(&plain);
    }

    #[test]
    fn the_holders_of_an_item_apart_from_readers_the_start_of_the_chain_globs_back_are_found_in_steps_linear_in_the_chain()
     {
        // Each reader lies on a way to the item for every other reader, so
        // a search without each reader in turn grows with the square of the
        // readers; the dominators answer for all of them at once.
        assert_linear(&fan_in);
    }

    #[test]
    fn the_holders_of_an_item_apart_from_readers_that_form_a_chain_of_their_own_are_found_in_steps_linear_in_the_chains()
     {
        assert_linear(&nested);
    }

    #[test]
    fn a_read_that_holds_no_companion_starts_no_search_for_the_holders_of_its_items() {
        // Each reader reads `X` as `d::X` beside a companion through `a`,
        // and as `e::X` through `e1` and `e2`, which both carry it from `e`:
        // only the read of `a` holds a companion, and it shares no item with
        // the other reads, so nothing is searched, as large as the chain is.
        let mut lib = String::from(BASE);
        lib.push_str(
            "pub mod e { pub struct X {} }\npub mod e1 { pub use crate::e::*; }\n\
             pub mod e2 { pub use crate::e::*; }\n",
        );
        for i in 0..3 * UNIT {
            writeln!(
                lib,
                "pub mod r{i} {{ pub use crate::a::*; pub use crate::e1::*; pub use crate::e2::*; }}"
            )
            .expect("write");
        }
        let (count, found) = counted(&lib);
        assert_eq!(count, 0);
        assert!(found.starts_with("Ambiguous"), "{found}");
    }

    /// Whether `ruler` dominates `node` by the immediate dominators
    /// `dominator`: it is `node` or one of the dominators above it.
    fn dominates_by(dominator: &[usize], ruler: usize, node: usize) -> bool {
        let mut at = node;
        loop {
            if at == ruler {
                return true;
            }
            if at == 0 || dominator[at] == super::NONE {
                return false;
            }
            at = dominator[at];
        }
    }

    /// Whether the root reaches `node` in `successors` with `left_out` left
    /// out, by a plain search.
    fn reached_without(successors: &[Vec<usize>], left_out: Option<usize>, node: usize) -> bool {
        let mut seen = vec![false; successors.len()];
        let mut work = vec![0];
        while let Some(at) = work.pop() {
            if Some(at) == left_out || seen[at] {
                continue;
            }
            seen[at] = true;
            work.extend(successors[at].iter().copied());
        }
        seen[node]
    }

    /// Check the dominators of `successors` against their definition: a
    /// node the root reaches is dominated by another exactly when leaving
    /// that one out leaves it unreached, and by itself and the root.
    fn check_dominators(successors: &[Vec<usize>]) -> Vec<usize> {
        let dominator =
            super::dominators(successors, &mut |_| Ok(())).unwrap_or_else(|e| panic!("{e}"));
        for node in 0..successors.len() {
            if !reached_without(successors, None, node) {
                assert_eq!(dominator[node], super::NONE, "{successors:?}: {node}");
                continue;
            }
            for ruler in 0..successors.len() {
                let expected = ruler == node || !reached_without(successors, Some(ruler), node);
                assert_eq!(
                    dominates_by(&dominator, ruler, node),
                    expected,
                    "{successors:?}: does {ruler} dominate {node}"
                );
            }
        }
        dominator
    }

    #[test]
    fn the_dominators_of_small_holder_graphs_follow_their_definition() {
        // A diamond: neither side dominates the join.
        let dominator = check_dominators(&[vec![1, 2], vec![3], vec![3], vec![]]);
        assert_eq!(dominator[3], 0);
        // A cycle entered once: its entry dominates the rest of it.
        let dominator = check_dominators(&[vec![1], vec![2], vec![1, 3], vec![]]);
        assert_eq!((dominator[2], dominator[3]), (1, 2));
        // A node that leads to itself.
        let dominator = check_dominators(&[vec![1], vec![1, 2], vec![]]);
        assert_eq!(dominator[2], 1);
        // A node the root does not reach.
        let dominator = check_dominators(&[vec![1], vec![], vec![1]]);
        assert_eq!(dominator[2], super::NONE);
        // Repeated edges.
        let dominator = check_dominators(&[vec![1, 1], vec![2, 2], vec![]]);
        assert_eq!(dominator[2], 1);
        // Nodes reached only through one node, beside one reached apart.
        let dominator = check_dominators(&[vec![1, 4], vec![2], vec![3], vec![1], vec![3]]);
        assert_eq!((dominator[2], dominator[3]), (1, 0));
    }

    #[test]
    fn the_dominators_of_seeded_random_graphs_follow_their_definition() {
        // A fixed generator, so every run checks the same graphs.
        let mut state: u64 = 0x5eed_0f00_d000_0014;
        let mut next = move |bound: usize| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            usize::try_from(state % u64::try_from(bound).expect("small")).expect("small")
        };
        for _ in 0..400 {
            let count = 2 + next(8);
            let edges = next(3 * count);
            let mut successors: Vec<Vec<usize>> = vec![Vec::new(); count];
            for _ in 0..edges {
                let from = next(count);
                let to = 1 + next(count - 1);
                successors[from].push(to);
            }
            check_dominators(&successors);
        }
    }
}
