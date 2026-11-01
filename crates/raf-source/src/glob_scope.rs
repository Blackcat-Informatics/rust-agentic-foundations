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
//! reason are its companions, marked on their level. A returned binding can
//! lose its companions only when every route to its carrier passes through
//! the reading module and another route supplies the binding independently.
//! A separate glob of the same canonical item cannot shadow an inherited
//! ambiguity. What reads and views hold only grows, so a drop that the settled
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
//! globs from one table. A glob that reads a view another glob of its module
//! with the same visibility already reads passes nothing on, and what a
//! module exports is gathered from each view its globs read once, so a module
//! that globs one module many times over costs a read for each glob and not a
//! copy of the view. A view read through a glob whose level the class of the
//! view being gathered cannot access brings in only what that class cannot
//! access, so it is not gathered name by name, and a fan-in of private globs
//! onto one module costs what a fan-in of `pub` globs does.
//!
//! A glob whose level no view of its module can access, as the private glob
//! of a child of its parent that the parent globs, is a covert reader of its
//! view: everything it brings in is an item no importer of the module can
//! access, which matters only under a name a view of the module holds too,
//! where it stands beside what that view holds, and for whether the views
//! learn that the module's globs bring in what they cannot access. So such
//! a read is not given the view's names: it is given those a view of its
//! module holds too, and when its view gains a name it is looked at only
//! where a view of its module holds the name or does not know yet what it
//! cannot access, found from the views that hold the name and from the
//! view's open readers. It is listed among the covert readers before its
//! names arrive, so what its view gains through a cycle of globs while they
//! arrive reaches the module too. Every reader of a view keeps the turn it
//! came to read it in, and a gain is told to the active and the covert
//! readers in their turns, and the names of a view to a covert read in their
//! order, so the imports that wait on a name are queued again, and a name
//! with a companion is read again, in the order an active read of each would
//! have made: an import of a cycle can be decided by the order it is tried
//! in, and the reading again of a name with a companion by what the views
//! hold when it runs. A module with a `pub(in path)`
//! visibility, whose levels can change as the tables settle, reads no view
//! covertly. So the prelude pattern, a root that globs many children that
//! each glob their parent, costs what its names do.
//!
//! Under a name that no view holds a companion under, a view that holds the
//! name takes what arrives as the change it brings: an item it holds as
//! widely changes nothing, tested without a copy; an item it can access is
//! added; and one it cannot access, and does not hold, is exposed beside
//! what it holds. That is what reading the name again from the globs would
//! give: such a view holds everything the module's globs brought in under
//! the name that its class can access and nothing they brought in that it
//! cannot, which would have added a companion. An item the class can access
//! that first arrives under a name at a view that knows its module brings
//! in what it cannot access is taken as it comes too, where nothing of the
//! kind stands under the name: the view records the names under which only
//! such items reached it while it did not hold them, and the views its
//! module reads through a glob its class cannot access are looked at, while
//! they are fewer than the reads a reading again would look at. A name a
//! reading again set after a change to the module's bindings or imports of
//! it, and a module with a `pub(in path)` visibility, keep the reading
//! again. So a hub that many siblings glob and that globs them back, partly
//! privately, costs what its names do, as the resolver from before the two
//! visibility rules did, quadratic in the siblings.
//!
//! What the globs of a module bring in under a name is read again, where a
//! companion or an item a class cannot access arrives otherwise, only from
//! the reads of the module whose view holds the name: the tables keep the
//! views that hold each name and the reads of each module by the view they
//! read, and the views that hold the name are looked at to find those reads
//! while they are fewer than the reads of the module, else every read is.
//! Under a name that no view holds a companion under, each read delivers
//! what its view holds, so the lists are read where they are and not copied,
//! and the routes of the reads to each item are combined as they come. So a
//! fan-in of globs that each carry a companion costs a reading of one read
//! for each name at the module they fan into, found among the few views that
//! hold the name. One shape costs more than that: a hub that many siblings
//! glob and that globs them back, partly privately, where each two siblings
//! define an item of one name, so that every name carries a companion and
//! keeps the exact reading at each arrival (cubic, where the resolver from
//! before the two visibility rules was quadratic: 7.5 to 8.2 times the
//! instructions per doubling of the siblings where that resolver took 3.8
//! to 4.1, and 422 times its count at 800 siblings).
//!
//! Every view, each name and item it holds, each glob's read of a view, each
//! covert and open reader of a view, each view a view's module reads through
//! a glob its class cannot access, each name a view dropped what its class
//! cannot access under and each name a reading again set, and each gain while
//! it waits to be passed on is charged to the glob-bytes budget
//! ([`Budget::GlobBytes`](crate::Budget::GlobBytes)) as an upper bound of what
//! it allocates, derived from the layout of the containers that hold it (see
//! `crate::cost`). The budget bounds what is held at once: what a gain held is
//! released once it is passed on, and what the tables of one settle hold is
//! released when they are built again. Each charge is made before the
//! allocation it covers.
//!
//! The steps of that work are charged to the work budget
//! ([`Budget::Work`](crate::Budget::Work)) before they are made, and never
//! given back: each glob read listed and each name gathered when a view is
//! built, and each name probed in a view read through a glob the class cannot
//! access, each reader a change is passed on to (and each name a new reader
//! is given), each name of a view a covert read probes or looks at and each
//! it passes on, each name its module waits on, each covert reader and open
//! reader looked at when the view grows and each view that holds the name
//! looked at to find them, each item held and each new one looked at when a
//! change taken as it comes is exposed, the record of the dropped names and
//! each view read through a glob the class cannot access looked at when an
//! item first arrives, each glob read of a module looked at when what its
//! globs bring in under a name is read again or the drops of its reads are
//! recorded, and each view that holds the name looked at to find those reads,
//! and each step of the search for the modules that hold an item apart from a
//! reading module. So every shape is charged as its work grows, and refused
//! at the limit before it is settled to the end.
//!
//! The lookups of what the globs of a module bring in under a name
//! ([`Index::globbed`], [`Index::globbed_any`]) are charged too: each glob
//! read they look at and each view that holds the name looked at to find
//! those reads is a step. They are made by code that answers and cannot
//! refuse, so during the load they are counted on the index as they are
//! made and charged at the next checkpoint, after each import tried, each
//! node of the graph of the imports left undecided, each component of it
//! decided and each decision made, each glob read again and each import
//! walked again, and each impl attached: a load is refused there, by the
//! same budget, past its limit by at most the lookups of one such step. A
//! query after the load counts its own on a meter of its own, from zero,
//! against `max_query_work`.
//!
//! A few temporaries are not charged, each bounded by a constant factor of
//! what another budget already charges and dropped before the next is made:
//! the visibilities of each module, listed once for each item and `use`
//! declaration before their repeats are removed, the bindings with their
//! routes that a lookup of what the globs of a module bring in under one name
//! returns, and the routes it combines for each item, the lists of the places
//! of the reads of a module whose view holds a name, one for each such view,
//! listed for one reading, the names that hold companions, listed while the
//! drops the settled tables show are recorded, the names a covert read passes
//! on and the items of each, at most the entries of the views, and the covert
//! readers a gain is passed on to and the open readers it closes, at most the
//! readers of the view. The search for the modules that hold an item apart
//! from a reading module charges its work before each step. Its live graph
//! nodes, edges and dominator scratch are separately admitted to the glob
//! budget before allocation (`SUPPLY_NODE_BYTES`, `SUPPLY_EDGE_BYTES`). The
//! cached spans remain charged while replacement recording uses them.

use purrdf_hash::fixed::FixedState;
use purrdf_lex::allocation::{OrderedTable, StorageError};

mod cut;
mod ranked;
use purrdf_lex::walk::WorkList;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::mem::size_of;
use std::ops::Bound::{Excluded, Unbounded};

use crate::cost::{btree_entry, btree_map, hash_entry, hash_map, shared_text, vec_first, vec_slot};
use crate::error::LoadError;
use crate::facts::Vis;
use crate::index::{Index, UseDecl};
use crate::item::{ItemId, Namespace, SharedStr};
use crate::meter::{Meter, NativeAdmission, Steps, Transient};
use crate::path::RelPath;
use crate::resolve::{Fail, Heads};
use crate::settle::{Bound, Origin, vis_key};
use crate::work::{self as tally, Work};

/// How far a binding is accessible: everywhere, within one module (which is
/// the module the binding is in or one of its ancestors) and so by every
/// module inside it, or nowhere. A level also tells whether the item it is
/// held at is there only as a companion of an ambiguity (see
/// [`Index::exposed`]), which does not change how far it is accessible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Level(usize);

impl Level {
    /// The low bit marks a companion at any visibility.
    const COMPANION: usize = 1;
    /// Accessible everywhere.
    pub(crate) const PUBLIC: Self = Self(usize::MAX - 1);
    /// Accessible nowhere: a restriction that names no module.
    pub(crate) const NOBODY: Self = Self(usize::MAX - 3);

    /// Accessible within `module`.
    pub(crate) const fn within(module: usize) -> Self {
        // Real module indices come from a Vec<Module>, whose allocation
        // bound is smaller than this representation's reserved range.
        assert!(
            module < Self::NOBODY.0 >> 1,
            "module index does not fit a visibility level"
        );
        Self(module << 1)
    }

    /// The level of a companion that is accessible as far as `self` is.
    pub(crate) const fn companion(self) -> Self {
        Self(self.0 | Self::COMPANION)
    }

    /// The level without the mark of a companion.
    const fn plain(self) -> Self {
        Self(self.0 & !Self::COMPANION)
    }

    /// True when the item is held as a companion only.
    const fn is_companion(self) -> bool {
        self.0 & Self::COMPANION != 0
    }

    /// The module the level is within, if it is one.
    const fn module(self) -> Option<usize> {
        let plain = self.plain();
        if plain.0 >= Self::NOBODY.0 {
            None
        } else {
            Some(plain.0 >> 1)
        }
    }

    /// Preserve the visibility ordering independently of packed storage.
    fn order_key(self) -> (u8, usize) {
        match self.plain() {
            Self::PUBLIC => (if self.is_companion() { 3 } else { 5 }, 0),
            Self::NOBODY => (if self.is_companion() { 2 } else { 4 }, 0),
            plain => (u8::from(self.is_companion()), plain.0 >> 1),
        }
    }
}

impl PartialOrd for Level {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Level {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order_key().cmp(&other.order_key())
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

/// Reserve the next geometric growth before constructing a place entry.
pub(crate) fn reserve_next<T>(into: &mut Vec<T>, steps: Steps<'_>) -> Result<(), LoadError> {
    // Every buffer entering here was created by this path under the same
    // parent lease. Native growth owns admission, allocation and old-buffer
    // destruction, including the period when both capacities coexist.
    let live = size_of::<T>() * into.capacity();
    let mut admission = NativeAdmission::resume(steps, live);
    let result =
        purrdf_lex::allocation::Memory::resume(&mut admission, live).reserve_for_push(into);
    result.map_err(|error| admission.graph_error(purrdf_core::graph::GraphError::Storage(error)))
}

/// Push onto a native LIFO under the buffer's original physical grant.
pub(crate) fn push_work<T, const N: usize>(
    into: &mut WorkList<T, N>,
    value: T,
    steps: Steps<'_>,
) -> Result<(), LoadError> {
    let live = size_of::<T>() * into.heap_capacity();
    let mut admission = NativeAdmission::resume(steps, live);
    into.try_push_admitted(
        value,
        &mut purrdf_lex::allocation::Memory::resume(&mut admission, live),
    )
    .map_err(|error| admission.graph_error(error.into()))
}

/// Intern a graph place once; every graph expansion is charged first.
fn add_holder(
    place: &Place,
    nodes: &mut Vec<Place>,
    places: &mut BTreeMap<Place, usize>,
    successors: &mut Vec<Vec<usize>>,
    predecessors: &mut Vec<Vec<usize>>,
    step: &mut dyn FnMut(usize) -> Result<(), LoadError>,
    admit: &dyn Fn(usize) -> Result<(), LoadError>,
) -> Result<usize, LoadError> {
    if let Some(node) = places.get(place) {
        return Ok(*node);
    }
    step(1)?;
    admit(SUPPLY_NODE_BYTES)?;
    let node = nodes.len();
    places.insert(place.clone(), node);
    nodes.push(place.clone());
    successors.push(Vec::new());
    predecessors.push(Vec::new());
    Ok(node)
}

/// Work units for a comparison sort with its worst-case `n log n` growth.
fn sort_units(length: usize) -> usize {
    if length < 2 {
        0
    } else {
        length * (length.ilog2() as usize + 1)
    }
}

/// A comparison-tree lookup's bounded number of key comparisons.
fn search_units(length: usize) -> usize {
    1 + length.saturating_add(1).ilog2() as usize
}

/// Upper bound for simultaneous supporter and carrier lookup buffers.
fn redelivery_bytes(found: &[(&GlobRead, Cow<'_, [Held]>)]) -> usize {
    let entries = found.iter().map(|(_, held)| held.len()).sum::<usize>();
    let per_entry = btree_entry::<ItemId, Vec<(Level, usize)>>()
        + vec_first::<(Level, usize)>()
        + vec_slot::<(Level, usize)>()
        + btree_entry::<ItemId, Vec<usize>>()
        + vec_first::<usize>()
        + vec_slot::<usize>();
    let base = btree_map::<ItemId, Vec<(Level, usize)>>() + btree_map::<ItemId, Vec<usize>>();
    base.saturating_add(entries.saturating_mul(per_entry))
}

/// One carrier's dense selection offset and immutable existing selections.
struct SelectionRead<'a> {
    offset: usize,
    known: &'a [Held],
}

/// Query-scoped selections in the same carrier and identity order as the input.
struct Redelivery<'a> {
    selected: Vec<Option<Level>>,
    reads: Vec<SelectionRead<'a>>,
}

impl<'a> Redelivery<'a> {
    /// Admit both indexed buffers before reading their immutable known rows.
    fn new(
        index: &'a Index,
        found: &[(&GlobRead, Cow<'_, [Held]>)],
        name: &str,
        at: usize,
        memory: &Transient<'_>,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
    ) -> Result<Self, LoadError> {
        let decl = &index.uses[index.glob_imports[0]];
        let entries = found.iter().map(|(_, held)| held.len()).sum::<usize>();
        charge(found.len() + entries)?;
        let mut answer = Self {
            selected: Vec::new(),
            reads: Vec::new(),
        };
        // Both arrays remain under the query lease until their caller
        // consumes them; one native account includes their overlapping layouts.
        let mut admission = NativeAdmission::new(Steps::LoadAt {
            memory,
            file: &decl.file,
            line: decl.line,
        });
        let allocated = {
            let mut native = purrdf_lex::allocation::Memory::new(&mut admission);
            native
                .reserve(&mut answer.reads, found.len())
                .and_then(|()| native.reserve(&mut answer.selected, entries))
        };
        allocated.map_err(|error| admission.graph_error(error.into()))?;
        answer.selected.resize(entries, None);
        let mut offset = 0;
        for (read, held) in found {
            answer.reads.push(SelectionRead {
                offset,
                known: index.replacements(read, name, at),
            });
            offset += held.len();
        }
        Ok(answer)
    }
}

/// Canonical candidate/competitor keys and their carrier positions.
type SelectionTable = OrderedTable<(ItemId, Vec<ItemId>), Vec<(usize, usize)>>;

/// Equal-competitor carriers and their dense output positions.
#[derive(Default)]
struct SelectionGroups {
    table: SelectionTable,
    /// Original native grant for the arena, competitor keys and member buffers.
    live: usize,
}

impl SelectionGroups {
    /// Admit grouping work and preserve the original native grant across calls.
    fn add(
        &mut self,
        id: ItemId,
        held: &[Held],
        member: (usize, usize),
        memory: &Transient<'_>,
        decl: &UseDecl,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
    ) -> Result<(), LoadError> {
        // Each AVL search has at most twice the binary-log height;
        // a miss performs a second search to admit the new placement.
        charge(
            1 + held
                .len()
                .saturating_mul(search_units(self.table.len()).saturating_mul(4)),
        )?;
        let mut admission = NativeAdmission::resume(
            Steps::LoadAt {
                memory,
                file: &decl.file,
                line: decl.line,
            },
            self.live,
        );
        let result = {
            let mut native = purrdf_lex::allocation::Memory::resume(&mut admission, self.live);
            let result = self.insert(id, held, member, &mut native);
            self.live = native.admitted_bytes();
            result
        };
        result.map_err(|error| admission.graph_error(error.into()))
    }

    /// Allocate a competitor spelling only for the first carrier of its group.
    fn insert(
        &mut self,
        id: ItemId,
        held: &[Held],
        member: (usize, usize),
        storage: &mut purrdf_lex::allocation::Memory<'_, NativeAdmission<'_>>,
    ) -> Result<(), StorageError> {
        if let Some(members) = self.table.get_mut_by(|(candidate, competitors)| {
            id.cmp(candidate).then_with(|| {
                held.iter()
                    .filter_map(|entry| (entry.0 != id).then_some(entry.0))
                    .cmp(competitors.iter().copied())
            })
        }) {
            storage.reserve_for_push(members)?;
            members.push(member);
            return Ok(());
        }
        let mut competitors = Vec::new();
        storage.reserve(&mut competitors, held.len().saturating_sub(1))?;
        competitors.extend(
            held.iter()
                .filter_map(|entry| (entry.0 != id).then_some(entry.0)),
        );
        let mut members = Vec::new();
        storage.reserve_for_push(&mut members)?;
        members.push(member);
        // The borrowed query and owned key use exactly the same total ordering.
        // Exclusive access keeps this placement vacant after the missed lookup.
        let _previous = self.table.insert((id, competitors), members, storage)?;
        Ok(())
    }
}

/// The existing level of one sorted canonical selection.
fn replacement_level(known: &[Held], id: ItemId) -> Option<Level> {
    known
        .binary_search_by_key(&id, |entry| entry.0)
        .ok()
        .map(|place| known[place].1)
}

/// What a module exports under each name, in each name space.
type Exported = Vec<(String, [Vec<Held>; 3])>;

/// What a view gathers: what its module exports under each name, whether
/// its globs bring in an item its class cannot access, and the names under
/// which they bring in only such items (see [`Index::gather`]).
type Gathered = (Exported, bool, Vec<(SharedStr, usize)>);

/// The covert readers of a view that a gain can change, by module, each with
/// its first covert read of the view and its turn, and the open readers that
/// no longer are (see [`Index::covert_changed`]).
type Changed = (Vec<(usize, (usize, usize))>, Vec<usize>);

/// A covert reader told of a gain in its turn: the turn, the module, its
/// first covert read of the view, and whether the gain arrives through it,
/// or only what waits on the name in the module is told (see
/// [`Index::covert_told`]).
type Told = (usize, usize, usize, bool);

/// The glob reads whose companions are dropped: for each glob import and
/// module it reaches, the names whose companions the read does not deliver,
/// in each name space, to the importers that can access the widest level
/// given (see [`Index::delivered`]).
type DropNames = HashMap<SharedStr, [Vec<Held>; 3], FixedState>;
type Drops = BTreeMap<(usize, usize), DropNames>;

/// A read, its name/namespace, and the independently supported canonical id.
type Replacement = (usize, usize, SharedStr, usize, ItemId, Level);

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
    /// they bring in reaches: each glob's module, its place among the
    /// module's reads and its turn, in the order of their turns.
    active: Vec<(usize, usize, usize)>,
    /// The turn of the next glob that reads the view from a module with
    /// views: a gain of the view is told to its active and covert readers in
    /// the order of their turns, the order in which they came to read it so,
    /// which is the order in which the imports that wait on its names are
    /// queued again.
    turns: usize,
    /// The other globs that read the view, by module.
    passive: BTreeMap<usize, Vec<usize>>,
    /// The places among the reads of each module that reads the view of
    /// those reads, in the order they were made.
    places: BTreeMap<usize, Vec<usize>>,
    /// The glob import the view was made for: a refusal by the budget while
    /// the view grows names its file and line.
    site: usize,
    /// Whether the globs of the target have brought in an item the class
    /// cannot access. Only then can a name of the view stand for items the
    /// class cannot access beside those it can (see [`Index::exposed`]), so
    /// a view for which it is false gathers a name from what arrives alone.
    hidden: bool,
    /// What the view records of the globs that read it covertly and of
    /// what the globs of its target bring in that its class cannot access,
    /// made the first time it records one (see [`Unseen`]).
    unseen: Option<Box<Unseen>>,
}

/// What a view records of the globs that read it covertly and of what the
/// globs of its target bring in that its class cannot access: most views
/// record none of it, so it is made, and charged, the first time a view
/// records one.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Unseen {
    /// The globs that read the view from a module that has views, none of
    /// which can access what the glob brings in, by module: their places
    /// among the module's reads, each with its turn. Such a glob is not
    /// given the view's names, only those a view of its module holds, and it
    /// is looked at when the view grows only to tell the module what it
    /// cannot access.
    covert: BTreeMap<usize, Vec<(usize, usize)>>,
    /// The modules among the covert readers that had a view that did not
    /// yet know that its module's globs bring in what it cannot access,
    /// when they were last looked at: only these, and those with a view
    /// that holds a name the view gains, can be changed by the gain. A
    /// module that stops reading the view covertly leaves them.
    open: BTreeSet<usize>,
    /// The views the target reads through a glob whose level the class
    /// cannot access: every item they bring in is one it cannot access.
    hiding: BTreeSet<usize>,
    /// The names, each in a name space, under which the globs of the target
    /// brought in only items the class cannot access while the view did
    /// not hold the name, under a name that no view held a companion under:
    /// with the views of `hiding`, where an item the class cannot access
    /// under a name that the view does not hold can stand (see
    /// [`Index::hides_nothing`]).
    dropped: BTreeSet<(SharedStr, usize)>,
}

/// The record of a view that has recorded nothing (see [`Unseen`]).
static NO_UNSEEN: Unseen = Unseen {
    covert: BTreeMap::new(),
    open: BTreeSet::new(),
    hiding: BTreeSet::new(),
    dropped: BTreeSet::new(),
};

impl View {
    /// What the view records of its covert readers and of what its class
    /// cannot access.
    fn unseen(&self) -> &Unseen {
        self.unseen.as_deref().unwrap_or(&NO_UNSEEN)
    }

    /// That record, made empty the first time, which is charged before it
    /// is made (see [`View::unseen_bytes`]).
    fn unseen_mut(&mut self) -> &mut Unseen {
        self.unseen.get_or_insert_with(Box::default)
    }

    /// What making the view's record of what it cannot see allocates: its
    /// box, the first time.
    fn unseen_bytes(&self) -> usize {
        if self.unseen.is_some() {
            0
        } else {
            size_of::<Unseen>()
        }
    }
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
    strings: HashSet<SharedStr, FixedState>,
    /// What the tables were charged, released when they are cleared.
    charged: usize,
    /// The glob reads whose companions are dropped, decided from the
    /// settled tables of an earlier settle of the load and kept, with the
    /// depths, when the tables are cleared (see [`Index::record_drops`]).
    drops: Drops,
    /// Names with persisted canonical selections, active before any new
    /// companion arrives during rebuilding. Charged with the retained rows.
    replaced_names: HashMap<SharedStr, [bool; 3], FixedState>,
    /// How many names of glob reads, each in one name space, `drops` holds.
    dropped: usize,
    /// For each name, the name spaces in which a view has held one of its
    /// items as a companion since the tables were built: only under such a
    /// name can a glob read deliver less than its view holds (see
    /// [`Index::delivered`]).
    companions: HashMap<SharedStr, [bool; 3], FixedState>,
    /// For each name, the views that hold it in any name space, in the order
    /// they took it; a view never gives a name up.
    holding: HashMap<SharedStr, Vec<usize>, FixedState>,
    /// The views, names and name spaces whose items a reading again after a
    /// change to the module's bindings or imports of the name set from the
    /// globs (see [`Index::refresh`]): an arrival there is read again, never
    /// passed on as the change it brings (see [`Index::arrive`]).
    exact: BTreeSet<(usize, SharedStr, usize)>,
}

/// The changes waiting to be passed on, as [`Index::pass_on`] holds them:
/// the list and the room of the list charged so far.
struct Queue<'m> {
    work: WorkList<Grown<'m>, 1>,
    memory: Transient<'m>,
}

impl<'m> Queue<'m> {
    fn new(meter: &'m Meter) -> Self {
        Self {
            work: WorkList::new(),
            memory: Transient::new(meter),
        }
    }

    fn push(&mut self, grown: Grown<'m>, decl: &UseDecl) -> Result<(), LoadError> {
        let live = self.work.heap_capacity() * size_of::<Grown<'_>>();
        let mut admission = NativeAdmission::resume(
            Steps::LoadAt {
                memory: &self.memory,
                file: &decl.file,
                line: decl.line,
            },
            live,
        );
        self.work
            .try_push_admitted(
                grown,
                &mut purrdf_lex::allocation::Memory::resume(&mut admission, live),
            )
            .map_err(|error| admission.graph_error(error.into()))
    }
}

/// A returned view delta keeps its reservation through queued delivery.
struct Added<'m> {
    ids: Vec<Held>,
    _memory: Transient<'m>,
}

impl std::ops::Deref for Added<'_> {
    type Target = [Held];
    fn deref(&self) -> &[Held] {
        &self.ids
    }
}

/// The glob reads of a module that may bring in a name: every read, in the
/// order they were made, or those whose view holds the name, the reads of
/// each such view in the order they were made (see [`Index::reads_holding`]).
enum Holders<'a> {
    /// Every read of the module.
    All(std::slice::Iter<'a, GlobRead>),
    /// The reads of the module at the places of these lists.
    Places {
        /// The reads of the module.
        reads: &'a [GlobRead],
        /// The lists of places not yet begun.
        lists: std::vec::IntoIter<&'a [usize]>,
        /// The places left of the list begun.
        list: std::slice::Iter<'a, usize>,
        /// How many places are left in all.
        left: usize,
        /// How many views that hold the name were looked at to find the
        /// lists.
        scanned: usize,
    },
}

impl Holders<'_> {
    /// How many views that hold the name were looked at to find the reads:
    /// none when every read is taken.
    const fn scanned(&self) -> usize {
        match self {
            Self::All(_) => 0,
            Self::Places { scanned, .. } => *scanned,
        }
    }
}

impl<'a> Iterator for Holders<'a> {
    type Item = &'a GlobRead;

    fn next(&mut self) -> Option<&'a GlobRead> {
        match self {
            Self::All(reads) => reads.next(),
            Self::Places {
                reads,
                lists,
                list,
                left,
                ..
            } => loop {
                if let Some(place) = list.next() {
                    *left -= 1;
                    return Some(&reads[*place]);
                }
                *list = lists.next()?.iter();
            },
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::All(reads) => reads.size_hint(),
            Self::Places { left, .. } => (*left, Some(*left)),
        }
    }
}

impl ExactSizeIterator for Holders<'_> {}

/// The visibility a glob read gives an item it brings in: the glob's own, as
/// written, or the narrower level of the binding it reads (see
/// [`Index::globbed`]). Two routes are kept apart unless they are the same
/// written visibility or the same level, and turned into visibilities only
/// once for each item.
#[derive(Clone, Copy)]
enum RouteVis<'a> {
    /// The visibility written on the glob import.
    Written(&'a Vis),
    /// The level of the binding the glob reads.
    At(Level),
}

impl RouteVis<'_> {
    /// True when the two are kept as one visibility without turning them
    /// into visibilities.
    fn same(self, other: Self) -> bool {
        match (self, other) {
            (Self::Written(left), Self::Written(right)) => left == right,
            (Self::At(left), Self::At(right)) => left.plain() == right.plain(),
            _ => false,
        }
    }
}

/// The routes of the glob reads of a module to one item, combined as
/// [`Bound::offer`] combines them (see [`Index::globbed`]).
struct Routes<'a> {
    id: ItemId,
    /// The target of the route kept: the first read, in the order made, of
    /// the least target.
    target: usize,
    /// The glob import of the route kept.
    glob: usize,
    /// The visibility of the route kept.
    kept: RouteVis<'a>,
    /// Whether a route makes the item public.
    public: bool,
    /// The visibility of the first route.
    first: RouteVis<'a>,
    /// The visibilities of the other routes that differ from the first and
    /// from each other, as [`RouteVis::same`] tells.
    others: Vec<RouteVis<'a>>,
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
struct Grown<'m> {
    view: usize,
    name: SharedStr,
    at: usize,
    ids: Added<'m>,
}

impl<'m> Grown<'m> {
    /// A change of `view`.
    const fn new(view: usize, name: SharedStr, at: usize, ids: Added<'m>) -> Self {
        Self {
            view,
            name,
            at,
            ids,
        }
    }
}

/// The modules that hold an item under a name apart from a reading module,
/// found once for each item, name and name space over the settled tables
/// while their drops are recorded (see [`Index::record_drops`]).
type Place = (usize, SharedStr, usize);

/// A route's destination and its own effective visibility, with companion flag.
type SupplyRoute = (Place, Level);

/// An exact visibility-qualified holder query; modes share no other level.
type HoldingKey = (ItemId, SharedStr, usize, Level, bool);

/// The admitted node table and the two directions of the holder graph.
struct HolderGraph {
    nodes: Vec<Place>,
    place: BTreeMap<Place, usize>,
    successors: Vec<Vec<usize>>,
    predecessors: Vec<Vec<usize>>,
}

impl HolderGraph {
    /// Initial storage has already been covered by `SUPPLY_BASE_BYTES`.
    fn new(at: usize) -> Self {
        Self {
            nodes: vec![(NONE, SharedStr::from(""), at)],
            place: BTreeMap::new(),
            successors: vec![Vec::new()],
            predecessors: vec![Vec::new()],
        }
    }

    /// Admit a destination and its edge before either graph buffer grows.
    fn edge(
        &mut self,
        from: usize,
        destination: &Place,
        step: &mut dyn FnMut(usize) -> Result<(), LoadError>,
        admit: &dyn Fn(usize) -> Result<(), LoadError>,
    ) -> Result<(), LoadError> {
        let node = add_holder(
            destination,
            &mut self.nodes,
            &mut self.place,
            &mut self.successors,
            &mut self.predecessors,
            step,
            admit,
        )?;
        admit(SUPPLY_EDGE_BYTES)?;
        self.successors[from].push(node);
        self.predecessors[node].push(from);
        Ok(())
    }
}

struct Supply<'m> {
    /// For each module, the glob reads of other modules that read one of
    /// its views: the reading module and the place of the read among its
    /// reads.
    readers: Vec<Vec<(usize, usize, Level)>>,
    /// For each name and name space, the modules that bind the name there
    /// themselves.
    roots: BTreeMap<ItemId, Vec<SupplyRoute>>,
    /// Actual named-import dependencies, including renamed source names.
    named: BTreeMap<(ItemId, Place), Vec<SupplyRoute>>,
    /// For each item, name and name space, the modules that hold the item as
    /// itself under the name and which of them every way to each passes
    /// through (see [`Holding`]).
    holdings: BTreeMap<HoldingKey, Holding>,
    /// Discovery can ask dominance only from modules with multiple raw reads.
    cut_reader: Option<usize>,
    cuts: cut::Cuts,
    /// A refusal by the budget during a search, returned once it ends.
    refused: Option<LoadError>,
    memory: Transient<'m>,
}

/// A raw name row is unqueried, known absent or known present.
#[derive(Clone, Copy)]
enum NameRow<'i> {
    Unqueried,
    Known(Option<&'i [Vec<Held>; 3]>),
}

/// Dense raw-row scratch allocated once for immutable drop discovery.
struct RowScratch<'i, 'm> {
    rows: Vec<NameRow<'i>>,
    touched: Vec<usize>,
    name: Option<SharedStr>,
    memory: Transient<'m>,
}

impl<'i, 'm> RowScratch<'i, 'm> {
    fn new(
        views: usize,
        memory: Transient<'m>,
        decl: &UseDecl,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
    ) -> Result<Self, LoadError> {
        charge(views)?;
        let mut scratch = Self {
            rows: Vec::new(),
            touched: Vec::new(),
            name: None,
            memory,
        };
        let mut admission = NativeAdmission::new(Steps::LoadAt {
            memory: &scratch.memory,
            file: &decl.file,
            line: decl.line,
        });
        let result = {
            let mut native = purrdf_lex::allocation::Memory::new(&mut admission);
            native
                .reserve(&mut scratch.rows, views)
                .and_then(|()| native.reserve(&mut scratch.touched, views))
        };
        result.map_err(|error| admission.graph_error(error.into()))?;
        scratch.rows.resize(views, NameRow::Unqueried);
        Ok(scratch)
    }

    /// Insert an unqueried view once, within the pre-admitted capacity.
    /// Both callers of `fill_raw_row` have already matched `Unqueried`.
    fn insert(&mut self, view: usize, row: Option<&'i [Vec<Held>; 3]>) {
        self.touched.push(view);
        self.rows[view] = NameRow::Known(row);
    }
}

/// Raw immutable rows for the active spelling, retaining all three namespaces.
struct QueryRows<'q, 'i, 'm> {
    index: &'i Index,
    name: &'q str,
    scratch: &'q mut RowScratch<'i, 'm>,
}

impl<'q, 'i, 'm> QueryRows<'q, 'i, 'm> {
    fn new(
        index: &'i Index,
        name: &'q SharedStr,
        phase: &'q mut Option<RowScratch<'i, 'm>>,
        memory: &Transient<'m>,
        decl: &UseDecl,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
    ) -> Result<Self, LoadError> {
        if phase.is_none() {
            *phase = Some(RowScratch::new(
                index.globs.views.len(),
                memory.child(),
                decl,
                charge,
            )?);
        }
        let scratch = phase.as_mut().expect("the phase row scratch was admitted");
        charge(1)?;
        // Equal immutable slices can retain raw rows across namespaces. A
        // different allocation conservatively resets even for equal text.
        let same = scratch
            .name
            .as_ref()
            .is_some_and(|previous| std::ptr::eq(previous.as_str(), name.as_str()));
        if !same {
            charge(1 + scratch.touched.len())?;
            for view in scratch.touched.drain(..) {
                scratch.rows[view] = NameRow::Unqueried;
            }
            scratch.name = Some(name.clone());
        }
        Ok(Self {
            index,
            name,
            scratch,
        })
    }

    /// Whether the caller requests this cache's spelling.
    fn matches(&self, name: &str) -> bool {
        std::ptr::eq(name, self.name) || name == self.name
    }

    /// Read or populate one already-admitted raw name slot without allocating.
    fn raw_row(&mut self, view: usize) -> Option<&'i [Vec<Held>; 3]> {
        match self.scratch.rows[view] {
            NameRow::Known(row) => row,
            NameRow::Unqueried => self.fill_raw_row(view),
        }
    }

    /// Populate an unqueried slot; callers charge before this actual mutation.
    fn fill_raw_row(&mut self, view: usize) -> Option<&'i [Vec<Held>; 3]> {
        let row = self.index.globs.views[view]
            .names
            .get(self.name)
            .map(Box::as_ref);
        self.scratch.insert(view, row);
        row
    }

    fn level(
        &mut self,
        view: usize,
        name: &str,
        at: usize,
        id: ItemId,
        charge: &mut dyn FnMut(usize) -> Result<(), LoadError>,
    ) -> Result<Option<Level>, LoadError> {
        charge(1)?;
        if !std::ptr::eq(name, self.name) {
            charge(1 + name.len().min(self.name.len()))?;
            if name != self.name {
                return Ok(self.index.held_level(view, name, at, id));
            }
        }
        charge(1)?;
        let row = if let NameRow::Known(row) = self.scratch.rows[view] {
            row
        } else {
            charge(1)?;
            self.fill_raw_row(view)
        };
        Ok(row.and_then(|lists| {
            let held = &lists[at];
            held.binary_search_by_key(&id, |entry| entry.0)
                .ok()
                .map(|place| held[place].1)
        }))
    }
}

/// One discovery query context; all retained proofs remain owned by Supply.
struct Routing<'a, 'i, 'm> {
    index: &'i Index,
    supply: &'a mut Supply<'m>,
    reader: usize,
    name: &'a SharedStr,
    at: usize,
    charge: &'a dyn Fn(usize) -> Result<(), LoadError>,
    rows: QueryRows<'a, 'i, 'm>,
}

impl Routing<'_, '_, '_> {
    fn ask(&mut self, from: usize, id: ItemId, level: Level, returning: bool) -> bool {
        self.index.supplied(
            self.supply,
            self.charge,
            from,
            self.reader,
            self.name,
            self.at,
            id,
            level,
            returning,
            Some(&mut self.rows),
        )
    }

    /// Borrow one exact returning proof; errors still precede row publication.
    fn returning(
        &mut self,
        id: ItemId,
        required: Level,
        apply: impl FnOnce(cut::Returning<'_>) -> Result<(), LoadError>,
    ) -> Result<(), LoadError> {
        if self.supply.refused.is_some() || required == Level::NOBODY {
            return Ok(());
        }
        if self.supply.cut_reader == Some(self.reader) {
            let key = (id, self.name.clone(), self.at, required, true);
            let proof = self.index.retained_cut(
                self.supply,
                self.charge,
                self.reader,
                &key,
                Some(&mut self.rows),
            )?;
            return apply(cut::Returning::Cut(proof));
        }
        let other = (id, self.name.clone(), self.at, required, false);
        let key = if self
            .supply
            .holdings
            .get(&other)
            .is_some_and(|proof| proof.firm_equivalent)
        {
            other
        } else {
            (id, self.name.clone(), self.at, required, true)
        };
        let proof =
            self.index
                .retained_holding(self.supply, self.charge, &key, Some(&mut self.rows))?;
        let reader = proof.span.get(&self.reader).copied();
        apply(cut::Returning::Full(proof, reader))
    }
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
    /// Both graph modes have exactly these edges: no eligible companion edge
    /// was included or skipped while traversing the complete reachable graph.
    firm_equivalent: bool,
}

impl Holding {
    /// Retain only this requested name's spans after the admitted graph walk.
    fn retain(
        place: BTreeMap<Place, usize>,
        span: &[(usize, usize)],
        firm_equivalent: bool,
        name: &SharedStr,
        at: usize,
        memory: &Transient<'_>,
        decl: &UseDecl,
    ) -> Result<Self, LoadError> {
        let retained = place
            .keys()
            .filter(|(_, held_name, held_at)| held_name == name && *held_at == at)
            .count();
        memory.charge(
            btree_map::<usize, (usize, usize)>()
                + retained * btree_entry::<usize, (usize, usize)>(),
            &decl.file,
            Some(decl.line),
        )?;
        Ok(Self {
            firm_equivalent,
            span: place
                .into_iter()
                .filter(|((_, held_name, held_at), _)| held_name == name && *held_at == at)
                .map(|((module, _, _), node)| (module, span[node]))
                .collect(),
        })
    }
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

/// A graph node and all simultaneously live node-indexed dominator scratch:
/// the place table, four adjacency/bucket headers and their initial room,
/// numeric arrays, the span, and the iterative tree walk. Edges are separate.
const SUPPLY_NODE_BYTES: usize = btree_entry::<Place, usize>()
    + vec_slot::<Place>()
    + 4 * (vec_slot::<Vec<usize>>() + vec_first::<usize>())
    + 9 * vec_slot::<usize>()
    + vec_slot::<(usize, usize)>();

/// Successor/predecessor entries and a possible queued depth-first edge.
const SUPPLY_EDGE_BYTES: usize = 2 * vec_slot::<usize>() + vec_slot::<(usize, usize)>();

/// Initial container room, admitted before even the sentinel graph node.
const SUPPLY_BASE_BYTES: usize = btree_map::<Place, usize>()
    + vec_first::<Place>()
    + 4 * vec_first::<Vec<usize>>()
    + 9 * vec_first::<usize>()
    + vec_first::<(usize, usize)>()
    + SUPPLY_NODE_BYTES;

/// No node: the parent or ancestor of the root of the holder graph.
const NONE: usize = usize::MAX;

/// What a view allocates at most beyond its class and its names: its record,
/// its entry among the views of every module and class (with a copy of the
/// class), its place among its module's views, its table of names, its
/// lists of readers at their first room and its table of the places of its
/// readers' reads.
const VIEW_BYTES: usize = vec_slot::<View>()
    + btree_entry::<(usize, Box<[bool]>), usize>()
    + vec_slot::<usize>()
    + vec_first::<usize>()
    + btree_map::<SharedStr, Lists>()
    + vec_first::<(usize, usize, usize)>()
    + 2 * btree_map::<usize, Vec<usize>>();

/// What a name of a view allocates at most beyond its text: its entry and its
/// lists.
const NAME_BYTES: usize = btree_entry::<SharedStr, Lists>() + size_of::<[Vec<Held>; 3]>();

/// What a glob's read of a view allocates at most: its place among its
/// module's reads and among the view's readers, as
/// an active reader or a passive one, and again if it turns active, and its
/// entries in the records of the reads made.
const READ_BYTES: usize = vec_slot::<GlobRead>()
    + btree_entry::<(usize, usize), ()>()
    + btree_entry::<(usize, usize, usize), ()>()
    + 2 * vec_slot::<(usize, usize, usize)>()
    + btree_entry::<usize, Vec<usize>>()
    + vec_first::<usize>()
    + vec_slot::<usize>();

/// What the tables allocate at most once for a load whose modules are
/// `modules`, before their first view: the lists of each module, the table of
/// views, the table of names, and the tables of the names held with a
/// companion and of the views that hold each name.
const fn base_bytes(modules: usize) -> usize {
    modules * (vec_slot::<Vec<usize>>() + vec_slot::<Vec<GlobRead>>())
        + vec_first::<Vec<usize>>()
        + vec_first::<Vec<GlobRead>>()
        + vec_first::<View>()
        + btree_map::<(usize, Box<[bool]>), usize>()
        + btree_map::<(usize, usize), ()>()
        + btree_map::<(usize, usize, usize), ()>()
        + hash_map::<SharedStr>()
        + hash_map::<(SharedStr, [bool; 3])>()
        + hash_map::<(SharedStr, Vec<usize>)>()
}

/// What a name a view takes allocates in the table of the views that hold
/// each name: a slot of its list, and the entry and first room of the list
/// for a name no view held before.
const fn holder_bytes(first: bool) -> usize {
    vec_slot::<usize>()
        + if first {
            hash_entry::<(SharedStr, Vec<usize>)>() + vec_first::<usize>()
        } else {
            0
        }
}

/// What a glob read listed among the covert readers of a view allocates: its
/// place and turn in the list of its module, with the entry and first room
/// of that list for the module's first such read of the view, and the first
/// room of the table for the view's first covert reader.
const fn covert_bytes(first_of_module: bool, first_of_view: bool) -> usize {
    vec_slot::<(usize, usize)>()
        + if first_of_module {
            btree_entry::<usize, Vec<(usize, usize)>>() + vec_first::<(usize, usize)>()
        } else {
            0
        }
        + if first_of_view {
            btree_map::<usize, Vec<(usize, usize)>>()
        } else {
            0
        }
}

/// What an entry of a set of `T` of the tables allocates: its entry, and
/// the first room of the set for its first.
const fn set_bytes<T>(first: bool) -> usize {
    btree_entry::<T, ()>() + if first { btree_map::<T, ()>() } else { 0 }
}

/// What a waiting import's name is told: the imports that wait on a name of
/// a module are queued again when what the module's globs bring in under it
/// grows.
pub(crate) trait Waiters {
    /// Queue again what waits on `name` in `module`.
    fn wake(&mut self, module: usize, name: &str) -> Result<(), LoadError>;
    /// The modules where something waits on `name`, if any.
    fn waiting_on(&self, name: &str) -> Option<&BTreeSet<usize>>;
    /// The names of `module` that something waits on.
    fn waited_in(
        &self,
        module: usize,
        memory: &Transient<'_>,
        file: &RelPath,
        line: usize,
    ) -> Result<Vec<String>, LoadError>;
}

/// No waiters: the tables are built once the imports are settled.
pub(crate) struct Nobody;

impl Waiters for Nobody {
    fn wake(&mut self, _module: usize, _name: &str) -> Result<(), LoadError> {
        Ok(())
    }

    fn waiting_on(&self, _name: &str) -> Option<&BTreeSet<usize>> {
        None
    }

    fn waited_in(
        &self,
        _module: usize,
        _memory: &Transient<'_>,
        _file: &RelPath,
        _line: usize,
    ) -> Result<Vec<String>, LoadError> {
        Ok(Vec::new())
    }
}

/// The reference mode of the tests: a load made in [`reference::exact`] is
/// settled as the glob scope tables settle a name that a view holds a
/// companion under, for every name: what the globs of a module bring in
/// under a name is read from every glob read of the module and as each
/// read delivers it (see [`Index::delivered`]), and a view is gathered from
/// every view the globs of its module read, also those whose glob its class
/// cannot access. So the equivalence test can compare the answers of loads
/// made in that mode with those of loads made out of it, where a name
/// without a companion is read only from the reads whose view holds it and
/// as the view holds it. The mode counts the builds of the glob tables made
/// in each, so the test can tell that the loads it compares were made in
/// the mode it asked for.
#[cfg(test)]
pub(crate) mod reference {
    use std::cell::Cell;

    thread_local! {
        /// Whether the loads of this thread are made in the reference mode.
        static EXACT: Cell<bool> = const { Cell::new(false) };
        /// The builds of the glob tables of this thread, out of the
        /// reference mode and in it.
        static BUILDS: Cell<[usize; 2]> = const { Cell::new([0; 2]) };
        /// Complete successful discovery/publication passes of this thread.
        static DISCOVERIES: Cell<usize> = const { Cell::new(0) };
        /// Execute the preserved mixed fallback for operation comparison.
        static SINGLE_RETURNING: Cell<bool> = const { Cell::new(false) };
        /// Actual retained-proof retrieval calls, including cache hits.
        static HOLDER_RETRIEVALS: Cell<usize> = const { Cell::new(0) };
    }

    /// Puts the mode back as it was when a run in the reference mode ends,
    /// even by a panic.
    struct Restore(bool);

    impl Drop for Restore {
        fn drop(&mut self) {
            EXACT.set(self.0);
        }
    }

    /// Run `run` in the reference mode.
    pub(crate) fn exact<T>(run: impl FnOnce() -> T) -> T {
        let _restore = Restore(EXACT.replace(true));
        run()
    }

    /// True when the loads of this thread are made in the reference mode.
    pub(crate) fn is_exact() -> bool {
        EXACT.get()
    }

    /// Count a build of the glob tables in the mode of this thread.
    pub(crate) fn built() {
        let mut builds = BUILDS.get();
        builds[usize::from(EXACT.get())] += 1;
        BUILDS.set(builds);
    }

    /// The builds of the glob tables this thread has made, out of the
    /// reference mode and in it.
    pub(crate) fn builds() -> [usize; 2] {
        BUILDS.get()
    }

    /// Compare the unchanged per-carrier fallback against homogeneous batching.
    pub(crate) fn with_single_returning<T>(run: impl FnOnce() -> T) -> T {
        struct Restore(bool);
        impl Drop for Restore {
            fn drop(&mut self) {
                SINGLE_RETURNING.set(self.0);
            }
        }
        let _restore = Restore(SINGLE_RETURNING.replace(true));
        run()
    }

    pub(crate) fn single_returning() -> bool {
        SINGLE_RETURNING.get()
    }

    pub(crate) fn retrieved_holder() {
        HOLDER_RETRIEVALS.set(HOLDER_RETRIEVALS.get() + 1);
    }

    pub(crate) fn holder_retrievals() -> usize {
        HOLDER_RETRIEVALS.get()
    }

    /// Observe actual successful discovery, including a no-new-rows pass.
    pub(crate) fn discovered() {
        DISCOVERIES.set(DISCOVERIES.get() + 1);
    }

    /// The completed discovery/publication passes on this test thread.
    pub(crate) fn discoveries() -> usize {
        DISCOVERIES.get()
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
        let bytes = vec_first::<usize>()
            + vec_first::<bool>()
            + self.modules.len() * (vec_slot::<usize>() + vec_slot::<bool>());
        let decl = &self.uses[first];
        meter.globs(bytes, &decl.file, Some(decl.line))?;
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
        let replaced_names = std::mem::take(&mut self.globs.replaced_names);
        let dropped = self.globs.dropped;
        self.globs = GlobTables {
            depth,
            in_path,
            drops,
            replaced_names,
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
        #[cfg(test)]
        reference::built();
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

    /// Compare widths directly for equal levels and the reserved endpoints.
    fn width_order(&self, left: Level, right: Level) -> std::cmp::Ordering {
        let (left, right) = (left.plain(), right.plain());
        if left == right {
            std::cmp::Ordering::Equal
        } else {
            match (left, right) {
                (Level::PUBLIC, _) | (_, Level::NOBODY) => std::cmp::Ordering::Less,
                (_, Level::PUBLIC) | (Level::NOBODY, _) => std::cmp::Ordering::Greater,
                _ => self.width(left).cmp(&self.width(right)),
            }
        }
    }

    /// Whether an offered route covers every importer of the required level.
    /// Holder routes cross module branches, so depth alone cannot establish
    /// containment as it can for two levels on one ancestor chain.
    pub(crate) fn level_covers(
        &self,
        offered: Level,
        required: Level,
        charge: &mut dyn FnMut(usize) -> Result<(), LoadError>,
    ) -> Result<bool, LoadError> {
        let (offered, required) = (offered.plain(), required.plain());
        if offered == Level::NOBODY || required == Level::NOBODY {
            return Ok(false);
        }
        if offered == Level::PUBLIC || offered == required {
            return Ok(true);
        }
        let (Some(offered), Some(mut required)) = (offered.module(), required.module()) else {
            return Ok(false);
        };
        loop {
            charge(1)?;
            if offered == required {
                return Ok(true);
            }
            match self.modules[required].parent {
                Some(parent) => required = parent,
                None => return Ok(false),
            }
        }
    }

    /// Of two levels of one chain of ancestors the one more modules can
    /// access.
    pub(crate) fn wider(&self, left: Level, right: Level) -> Level {
        if self.width_order(right, left).is_lt() {
            right
        } else {
            left
        }
    }

    /// Of two levels of one chain of ancestors the one fewer modules can
    /// access: what a binding is accessible at when it is imported at one
    /// level a binding that is accessible at the other.
    pub(crate) fn meet(&self, left: Level, right: Level) -> Level {
        if self.width_order(right, left).is_gt() {
            right
        } else {
            left
        }
    }

    /// The level a binding with the visibility `vis` in `home` is accessible
    /// at: `pub(in path)` names an ancestor of `home` or `home` itself.
    pub(crate) fn level_of(&self, vis: &Vis, home: usize, steps: Steps<'_>) -> Level {
        self.read_level_of(vis, home, steps)
            .expect("nonretained visibility levels reserve no import storage")
    }

    pub(crate) fn read_level_of(
        &self,
        vis: &Vis,
        home: usize,
        steps: Steps<'_>,
    ) -> Result<Level, Fail> {
        Ok(match vis {
            Vis::Public => Level::PUBLIC,
            Vis::Inherited => Level::within(home),
            Vis::Restricted(text) => match text.as_str() {
                "crate" => Level::within(self.modules[home].root),
                "self" => Level::within(home),
                "super" => self.modules[home]
                    .parent
                    .map_or(Level::NOBODY, Level::within),
                other => {
                    if let Some(written) = other.strip_prefix("in ") {
                        self.level_in(written, home, steps)?
                    } else {
                        Level::NOBODY
                    }
                }
            },
        })
    }

    /// The level of `pub(in written)` in `home`.
    fn level_in(&self, written: &str, home: usize, steps: Steps<'_>) -> Result<Level, Fail> {
        let segs = Self::visibility_path(written, steps)?;
        let walked = match self.walk(home, &segs, false, Namespace::Type, Heads::Local, steps) {
            Ok(walked) => walked,
            Err(Fail::Storage(error)) => return Err(Fail::Storage(error)),
            Err(_) => return Ok(Level::NOBODY),
        };
        Ok(walked
            .modules
            .iter()
            .copied()
            .chain(walked.items.keys().flat_map(|id| self.modules_for(*id)))
            .filter(|ancestor| self.within(home, *ancestor))
            .map(Level::within)
            .reduce(|left, right| self.wider(left, right))
            .unwrap_or(Level::NOBODY))
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

    /// True when the load is made in the reference mode of the tests (see
    /// [`reference`]); never outside them.
    fn exact_mode() -> bool {
        #[cfg(test)]
        {
            reference::is_exact()
        }
        #[cfg(not(test))]
        {
            false
        }
    }

    /// True when a view has held an item under `name` in the name space at
    /// `at` as a companion since the tables were built, or in the reference
    /// mode of the tests. Under any other name no view holds a companion, so
    /// each glob read of a module delivers what its view holds (see
    /// [`Index::delivered`]).
    fn flagged(&self, name: &str, at: usize) -> bool {
        Self::exact_mode()
            || self.replaced(name, at)
            || (!self.globs.companions.is_empty()
                && self
                    .globs
                    .companions
                    .get(name)
                    .is_some_and(|spaces| spaces[at]))
    }

    /// A previous settle already selected canonical ids under this name.
    fn replaced(&self, name: &str, at: usize) -> bool {
        self.globs
            .replaced_names
            .get(name)
            .is_some_and(|spaces| spaces[at])
    }

    /// True when a glob of `module` at `level` is covert: the module has
    /// views and none of their classes can access the level, so every item
    /// the glob brings in is one that no importer of the module can access
    /// (what a glob brings in is no wider than the glob, and a class that
    /// cannot access a level cannot access a narrower one). Never for a
    /// module with a `pub(in path)` visibility, whose levels can change as
    /// the tables settle, nor in the reference mode of the tests.
    fn covert_level(&self, module: usize, level: Level) -> bool {
        !Self::exact_mode()
            && !self.exports_by_path(module)
            && !self.exports_of(module).is_empty()
            && self
                .exports_of(module)
                .iter()
                .all(|view| !self.allowed(module, &self.globs.views[*view].class, level))
    }

    /// The reads of the globs of `module` whose view holds `name` in some
    /// name space: found from the views that hold the name, when they are
    /// fewer than the reads of the module, each view looked at counted as a
    /// view scanned, and the reads of each such view in the order they were
    /// made; else every read of the module, in the order they were made; in
    /// the reference mode of the tests, every read. A read whose view does
    /// not hold the name brings nothing in under it, and a view never gives a
    /// name up, so a reading of what the globs of the module bring in under
    /// the name from these reads reads what it would from all of them, and
    /// looks at a read only where the name is. Only the order of reads of
    /// different views can differ from the order they were made, and no
    /// reading depends on it: the route a lookup keeps is the first read of
    /// the least target, and the reads of one target read one view.
    fn reads_holding(&self, module: usize, name: &str) -> Holders<'_> {
        let reads = self.reads_of(module);
        if Self::exact_mode() {
            return Holders::All(reads.iter());
        }
        let Some(views) = self.globs.holding.get(name) else {
            return Holders::All([].iter());
        };
        if views.len() >= reads.len() {
            return Holders::All(reads.iter());
        }
        tally::add(Work::HolderScans, views.len());
        let lists: Vec<&[usize]> = views
            .iter()
            .filter_map(|view| self.globs.views[*view].places.get(&module))
            .map(Vec::as_slice)
            .collect();
        let left = lists.iter().map(|list| list.len()).sum();
        Holders::Places {
            reads,
            lists: lists.into_iter(),
            list: [].iter(),
            left,
            scanned: views.len(),
        }
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
        self.held_level(view, name, at, id).is_some()
    }

    /// The actual candidate level, preserving its companion provenance.
    fn held_level(&self, view: usize, name: &str, at: usize, id: ItemId) -> Option<Level> {
        let held = &self.globs.views[view].names.get(name)?[at];
        held.binary_search_by_key(&id, |entry| entry.0)
            .ok()
            .map(|place| held[place].1)
    }

    /// Reuse only raw rows of the current immutable discovery query.
    fn routed_level(
        &self,
        view: usize,
        name: &str,
        at: usize,
        id: ItemId,
        rows: &mut Option<&mut QueryRows<'_, '_, '_>>,
        charge: &mut dyn FnMut(usize) -> Result<(), LoadError>,
    ) -> Result<Option<Level>, LoadError> {
        match rows {
            Some(rows) => rows.level(view, name, at, id, charge),
            None => Ok(self.held_level(view, name, at, id)),
        }
    }

    /// A hidden route can carry ambiguity at a wider level without becoming
    /// an independent route there. Only a settled companion view witnesses
    /// that widening; a plain route of the same identity cannot do so.
    fn promoted_companion(
        &self,
        place: &Place,
        id: ItemId,
        required: Level,
        charge: &mut dyn FnMut(usize) -> Result<(), LoadError>,
        rows: &mut Option<&mut QueryRows<'_, '_, '_>>,
    ) -> Result<bool, LoadError> {
        for view in self.exports_of(place.0) {
            charge(1)?;
            if let Some(level) = self.routed_level(*view, &place.1, place.2, id, rows, charge)?
                && level.is_companion()
                && self.level_covers(level, required, charge)?
            {
                return Ok(true);
            }
        }
        Ok(false)
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
    ///
    /// Under a name that no view holds a companion under, each read
    /// delivers what its view holds (see [`Index::delivered`]), and the
    /// routes of the reads to each item are combined as they come, as
    /// [`Bound::offer`] would combine them: the route kept is the first, in
    /// the order the reads were made, of the least target, with its
    /// visibility, or public when any route is, and the others are the
    /// distinct visibilities of the other routes, sorted. So one route is
    /// made for each item, and no list is copied.
    ///
    /// Each read looked at, and each view that holds the name looked at to
    /// find the reads, is a lookup step, counted where `steps` says.
    pub(crate) fn globbed(&self, module: usize, name: &str, at: usize, steps: Steps<'_>) -> Bound {
        self.read_globbed(module, name, at, steps)
            .expect("nonretained glob lookups reserve no import storage")
    }

    pub(crate) fn read_globbed(
        &self,
        module: usize,
        name: &str,
        at: usize,
        steps: Steps<'_>,
    ) -> Result<Bound, Fail> {
        let reads = self.reads_holding(module, name);
        self.count_lookups(steps, module, reads.scanned(), reads.len());
        if !self.flagged(name, at) {
            return self.globbed_plain(module, reads, name, at, steps);
        }
        let mut bound = Bound::new();
        for (read, held) in self.delivered(module, reads, name, at, None, Work::LookupReads) {
            let decl = &self.uses[read.glob];
            let own = self.read_level_of(&decl.vis, module, steps)?;
            for (id, level) in held.iter() {
                steps.storage_work(1 + bound.len()).map_err(Fail::Storage)?;
                tally::add(Work::LookupRoutes, 1);
                let met = self.meet(own, *level);
                let vis = if met == own {
                    decl.vis.clone()
                } else {
                    self.vis_at(met, module)
                };
                let mut origin = Self::glob_origin(decl, vis, read.target, at, name);
                origin.companion = level.is_companion();
                bound.offer_work(*id, &origin, steps)?;
                bound.offer(*id, origin);
            }
        }
        Ok(bound)
    }

    /// Whether an actually delivered, accessible route reads an ordinary
    /// glob family rather than a source's explicit binding of this name.
    /// Same-item route merging must not erase this namespace distinction.
    pub(crate) fn unnamed_glob_source(
        &self,
        module: usize,
        name: &str,
        at: usize,
        id: ItemId,
        viewer: usize,
        steps: Steps<'_>,
    ) -> Result<bool, Fail> {
        let mut found = false;
        self.delivered_item_sources(
            module,
            name,
            at,
            id,
            Some(viewer),
            steps,
            &mut |target, _| {
                if self.scopes[target]
                    .get(name)
                    .is_none_or(|entry| entry.defs[at].is_empty())
                {
                    found = true;
                }
                Ok(())
            },
        )?;
        Ok(found)
    }

    /// Visit the actual replacement-filtered routes carrying one selected
    /// item, preserving each declaration, target and effective visibility.
    #[expect(
        clippy::too_many_arguments,
        reason = "one selected binding and its reader identify the delivered routes"
    )]
    pub(crate) fn delivered_item_sources(
        &self,
        module: usize,
        name: &str,
        at: usize,
        id: ItemId,
        viewer: Option<usize>,
        steps: Steps<'_>,
        visit: &mut dyn FnMut(usize, usize) -> Result<(), LoadError>,
    ) -> Result<(), Fail> {
        let memory = steps.temporary();
        let steps = memory.as_ref().map_or(steps, |memory| steps.at(memory));
        self.reserve_glob_lookup(module, name, at, steps)?;
        let reads = self.reads_holding(module, name);
        self.count_lookups(steps, module, reads.scanned(), reads.len());
        for (read, held) in self.delivered(module, reads, name, at, None, Work::LookupReads) {
            steps.storage_work(1 + held.len()).map_err(Fail::Storage)?;
            let Ok(found) = held.binary_search_by_key(&id, |entry| entry.0) else {
                continue;
            };
            let own = self.read_level_of(&self.uses[read.glob].vis, module, steps)?;
            if let Some(viewer) = viewer
                && !self
                    .level_covers(
                        self.meet(own, held[found].1),
                        Level::within(viewer),
                        &mut |units| steps.storage_work(units),
                    )
                    .map_err(Fail::Storage)?
            {
                continue;
            }
            visit(read.target, read.glob).map_err(Fail::Storage)?;
        }
        Ok(())
    }

    /// Stable spelling for a bound place; temporary walk Sources can die
    /// without leaving borrowed names in a semantic proof's worklist.
    pub(crate) fn bound_name(&self, module: usize, name: &str) -> Option<&str> {
        self.scopes[module]
            .get_key_value(name)
            .map(|(key, _)| key.as_str())
            .or_else(|| self.globs.strings.get(name).map(AsRef::as_ref))
    }

    /// Admit the holder-place cursor before constructing its borrowed lists.
    fn admitted_holders(
        &self,
        module: usize,
        name: &str,
        steps: Steps<'_>,
    ) -> Result<Holders<'_>, Fail> {
        if let Some(views) = self.globs.holding.get(name)
            && views.len() < self.reads_of(module).len()
        {
            steps.storage_work(views.len()).map_err(Fail::Storage)?;
            steps
                .reserve(vec_first::<&[usize]>() + views.len() * vec_slot::<&[usize]>())
                .map_err(Fail::Storage)?;
        }
        Ok(self.reads_holding(module, name))
    }

    /// Reserve a lookup's owned buffers from the routes it will actually
    /// visit. Per-route bounds cover Routes/RouteVis, up to three concurrent
    /// visibility buffers while widening/filtering, and the original and
    /// filtered Bounds. Written and generated visibility text are both
    /// included. This is a local allocation upper bound, not a reservation
    /// for every item in the index.
    pub(crate) fn reserve_glob_lookup(
        &self,
        module: usize,
        name: &str,
        at: usize,
        steps: Steps<'_>,
    ) -> Result<(), Fail> {
        let reads = self.admitted_holders(module, name, steps)?;
        steps.storage_work(reads.len()).map_err(Fail::Storage)?;
        let mut bytes = vec_first::<Routes<'_>>()
            + 2 * vec_first::<(ItemId, Origin)>()
            + vec_first::<(&GlobRead, Cow<'_, [Held]>)>();
        for read in reads {
            let Some(lists) = self.globs.views[read.view].names.get(name) else {
                continue;
            };
            let held = &lists[at];
            if held.is_empty() {
                continue;
            }
            steps.storage_work(held.len()).map_err(Fail::Storage)?;
            let written = match &self.uses[read.glob].vis {
                Vis::Restricted(text) => text.len(),
                _ => 0,
            };
            let text = written.max(9 + self.modules[module].path.len());
            let route = vec_slot::<Routes<'_>>()
                + vec_first::<RouteVis<'_>>()
                + vec_slot::<RouteVis<'_>>()
                + 3 * (vec_first::<Vis>() + vec_slot::<Vis>() + text)
                + 2 * (vec_slot::<(ItemId, Origin)>() + 2 * name.len() + text);
            bytes += held.len() * route + vec_slot::<(&GlobRead, Cow<'_, [Held]>)>();
            if !self.replacements(read, name, at).is_empty() {
                bytes += vec_first::<Held>() + held.len() * vec_slot::<Held>();
            }
        }
        steps.reserve(bytes).map_err(Fail::Storage)
    }

    /// [`Index::globbed`] under a name that no view holds a companion
    /// under, from the glob reads `reads` of `module`: the routes of the
    /// reads to each item combined in place, and one route made for each.
    fn globbed_plain(
        &self,
        module: usize,
        reads: Holders<'_>,
        name: &str,
        at: usize,
        steps: Steps<'_>,
    ) -> Result<Bound, Fail> {
        let mut items: Vec<Routes<'_>> = Vec::new();
        for read in reads {
            tally::add(Work::LookupReads, 1);
            let Some(held) = self.globs.views[read.view]
                .names
                .get(name)
                .map(|lists| &lists[at])
                .filter(|held| !held.is_empty())
            else {
                continue;
            };
            let decl = &self.uses[read.glob];
            let own = self.read_level_of(&decl.vis, module, steps)?;
            for (id, level) in held {
                steps.storage_work(1 + items.len()).map_err(Fail::Storage)?;
                let met = self.meet(own, *level);
                let vis = if met == own {
                    RouteVis::Written(&decl.vis)
                } else {
                    RouteVis::At(met)
                };
                let public = match vis {
                    RouteVis::Written(written) => *written == Vis::Public,
                    RouteVis::At(level) => level.plain() == Level::PUBLIC,
                };
                match items.binary_search_by_key(id, |routes| routes.id) {
                    Err(place) => items.insert(
                        place,
                        Routes {
                            id: *id,
                            target: read.target,
                            glob: read.glob,
                            kept: vis,
                            public,
                            first: vis,
                            others: Vec::new(),
                        },
                    ),
                    Ok(place) => {
                        let routes = &mut items[place];
                        steps
                            .storage_work(routes.others.len())
                            .map_err(Fail::Storage)?;
                        if read.target < routes.target {
                            routes.target = read.target;
                            routes.glob = read.glob;
                            routes.kept = vis;
                        }
                        routes.public |= public;
                        if !vis.same(routes.first)
                            && !routes.others.iter().any(|other| vis.same(*other))
                        {
                            routes.others.push(vis);
                        }
                    }
                }
            }
        }
        let mut bound = Bound::new();
        for routes in items {
            tally::add(Work::LookupRoutes, 1);
            let vis_of = |vis: RouteVis<'_>| match vis {
                RouteVis::Written(written) => written.clone(),
                RouteVis::At(level) => self.vis_at(level, module),
            };
            let kept = vis_of(routes.kept);
            let decl = &self.uses[routes.glob];
            let mut origin = Self::glob_origin(decl, kept, routes.target, at, name);
            if routes.public {
                origin.vis = Vis::Public;
            } else {
                let mut also: Vec<Vis> = Vec::new();
                for vis in std::iter::once(routes.first).chain(routes.others) {
                    steps.storage_work(1 + also.len()).map_err(Fail::Storage)?;
                    let vis = vis_of(vis);
                    if vis != origin.vis && !also.contains(&vis) {
                        also.push(vis);
                    }
                }
                if !also.is_empty() {
                    steps
                        .storage_work(also.len().saturating_mul(also.len().ilog2() as usize + 1))
                        .map_err(Fail::Storage)?;
                    also.sort_by(|left, right| vis_key(left).cmp(&vis_key(right)));
                    origin.also = Some(also.into_boxed_slice());
                }
            }
            bound.offer_work(routes.id, &origin, steps)?;
            bound.offer(routes.id, origin);
        }
        Ok(bound)
    }

    /// What each of the glob reads `reads` of `module`, those whose view
    /// holds `name` (see [`Index::reads_holding`]), delivers under the name
    /// in the name space at `at`: the items of the view it reads, less its
    /// companions where the read drops them. A read with no companion under
    /// the name delivers its view's list, so under a name that no view holds
    /// a companion under each read delivers what its view holds. A view holds the items of a name that its
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
    /// module holds it. Without, what the module itself reads. Drops require
    /// the settled route proof in [`Index::record_drops`]; an independent
    /// read of the same canonical item does not itself justify a drop. Each glob read
    /// looked at is counted as `visits`.
    fn delivered<'a>(
        &'a self,
        module: usize,
        reads: Holders<'a>,
        name: &str,
        at: usize,
        class: Option<&[bool]>,
        visits: Work,
    ) -> Vec<(&'a GlobRead, Cow<'a, [Held]>)> {
        let mut found = self.read_lists(reads, name, at, visits, None);
        for (read, held) in &mut found {
            self.filter_delivered((module, name, at), read, class, held, Work::ListCopies);
        }
        found
    }

    /// Apply the same persisted per-read selection to ordinary and proof inputs.
    fn filter_delivered(
        &self,
        (module, name, at): (usize, &str, usize),
        read: &GlobRead,
        class: Option<&[bool]>,
        held: &mut Cow<'_, [Held]>,
        copies: Work,
    ) {
        let replacements = self.replacements(read, name, at);
        self.filter_known_delivered(module, class, replacements, held, copies);
    }

    /// Apply a previously borrowed selection from the same frozen row batch.
    fn filter_known_delivered(
        &self,
        module: usize,
        class: Option<&[bool]>,
        replacements: &[Held],
        held: &mut Cow<'_, [Held]>,
        copies: Work,
    ) {
        let selected = |id| {
            replacements
                .binary_search_by_key(&id, |entry| entry.0)
                .ok()
                .map(|place| replacements[place].1)
                .filter(|level| class.is_none_or(|class| self.allowed(module, class, *level)))
        };
        if replacements
            .iter()
            .any(|entry| class.is_none_or(|class| self.allowed(module, class, entry.1)))
        {
            if held.iter().all(|entry| {
                selected(entry.0).is_some_and(|support| {
                    !entry.1.is_companion() && self.meet(entry.1, support) == entry.1
                })
            }) {
                return;
            }
            if matches!(held, Cow::Borrowed(_)) {
                tally::add(copies, 1);
            }
            let held = held.to_mut();
            held.retain(|entry| selected(entry.0).is_some());
            // A canonical replacement can have arrived as a companion.
            // It becomes firm only within its independent support's
            // visibility, and only if the carrier already holds it.
            for entry in held {
                entry.1 = self.meet(
                    entry.1.plain(),
                    selected(entry.0).expect("retained support"),
                );
            }
        }
    }

    /// Borrow a frozen, sorted row after applying its recorded support.
    /// No accessible support leaves the entire source unchanged. Otherwise
    /// only supported IDs survive, firm within the intersection of their
    /// carried visibility and the independently recorded support.
    fn known_delivered<'a>(
        &'a self,
        module: usize,
        class: &'a [bool],
        replacements: &'a [Held],
        held: &'a [Held],
    ) -> impl Iterator<Item = Held> + 'a {
        let active = replacements
            .iter()
            .any(|entry| self.allowed(module, class, entry.1));
        let mut position = 0;
        held.iter().copied().filter_map(move |entry| {
            if !active {
                return Some(entry);
            }
            while position < replacements.len() && replacements[position].0 < entry.0 {
                position += 1;
            }
            let &(id, support) = replacements.get(position)?;
            (id == entry.0 && self.allowed(module, class, support))
                .then(|| (entry.0, self.meet(entry.1.plain(), support)))
        })
    }

    /// The items of the view each of the glob reads `reads` of a module
    /// reads under `name` in the name space at `at`, for the reads whose view
    /// holds any. Each glob read looked at is counted as `visits`; the
    /// lists stay borrowed until an accessible recorded drop changes one.
    fn read_lists<'a>(
        &'a self,
        reads: Holders<'a>,
        name: &str,
        at: usize,
        visits: Work,
        rows: Option<&mut QueryRows<'_, 'a, '_>>,
    ) -> Vec<(&'a GlobRead, Cow<'a, [Held]>)> {
        let mut found = Vec::new();
        let mut previous = None;
        // Discovery prepays this match and each possible cache access. Other
        // callers retain the direct lookup and persisted delivery filtering.
        let mut rows = rows.filter(|rows| rows.matches(name));
        for read in reads {
            tally::add(visits, 1);
            // Holder places keep one view's reads together. Its immutable
            // name row is shared while each individual read is still visited.
            let lists = match previous {
                Some((view, lists)) if view == read.view => lists,
                _ => {
                    let lists = match &mut rows {
                        Some(rows) => rows.raw_row(read.view),
                        None => self.globs.views[read.view].names.get(name).map(Box::as_ref),
                    };
                    previous = Some((read.view, lists));
                    lists
                }
            };
            let Some(lists) = lists else {
                continue;
            };
            if !lists[at].is_empty() {
                found.push((read, Cow::Borrowed(lists[at].as_slice())));
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
    fn redelivered<'a>(
        &'a self,
        (module, name, at): (usize, &str, usize),
        found: &[(&GlobRead, Cow<'_, [Held]>)],
        memory: &Transient<'_>,
        routed: &mut Routing<'_, '_, '_>,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
    ) -> Result<Redelivery<'a>, LoadError> {
        let decl = &self.uses[self.glob_imports[0]];
        charge(found.len())?;
        memory.charge(redelivery_bytes(found), &decl.file, Some(decl.line))?;
        let mut answer = Redelivery::new(self, found, name, at, memory, charge)?;
        let mut supporters: BTreeMap<ItemId, Vec<(Level, usize)>> = BTreeMap::new();
        let mut carriers: BTreeMap<ItemId, Vec<usize>> = BTreeMap::new();
        // Group by candidate identity and competing identities. Equivalent
        // carriers share the independent-support search, including a carrier
        // whose returning canonical candidate is marked as a companion.
        for (this, (read, held)) in found.iter().enumerate() {
            charge(1 + held.len())?;
            let glob = {
                let visibility_memory = memory.child();
                self.read_level_of(
                    &self.uses[read.glob].vis,
                    module,
                    Steps::LoadAt {
                        memory: &visibility_memory,
                        file: &decl.file,
                        line: decl.line,
                    },
                )
                .map_err(crate::settle::storage_failure)?
            };
            for (id, level) in held.iter() {
                charge(search_units(carriers.len()) + search_units(supporters.len()))?;
                carriers.entry(*id).or_default().push(this);
                if !level.is_companion() {
                    supporters
                        .entry(*id)
                        .or_default()
                        .push((self.meet(glob, *level), this));
                }
            }
        }
        for list in supporters.values_mut() {
            charge(sort_units(list.len()))?;
            list.sort_by(|(left, left_other), (right, right_other)| {
                self.width_order(*left, *right)
                    .then_with(|| left_other.cmp(right_other))
            });
        }
        let groups_memory = memory.child();
        let groups = self.selection_groups(found, &answer, &supporters, &groups_memory, charge)?;
        if groups.table.is_empty() {
            return Ok(answer);
        }
        // Carrier indices are dense in found. Native-admitted generation
        // marks avoid rebuilding a comparison tree for every candidate group.
        let excluded_memory = memory.child();
        let mut excluded = Vec::new();
        let mut admission = NativeAdmission::new(Steps::LoadAt {
            memory: &excluded_memory,
            file: &decl.file,
            line: decl.line,
        });
        purrdf_lex::allocation::Memory::new(&mut admission)
            .reserve(&mut excluded, found.len())
            .map_err(|error| admission.graph_error(error.into()))?;
        charge(found.len())?;
        excluded.resize(found.len(), 0usize);
        for (group, ((id, competitors), members)) in groups.table.ordered().enumerate() {
            let id = *id;
            let generation = group + 1;
            for competing in competitors {
                charge(search_units(carriers.len()))?;
                if let Some(carriers) = carriers.get(competing) {
                    charge(1 + carriers.len())?;
                    for &carrier in carriers {
                        excluded[carrier] = generation;
                    }
                }
            }
            let best = Self::independent_supporters(
                found,
                supporters.get(&id).map_or(&[][..], Vec::as_slice),
                (&excluded, generation),
                id,
                routed,
                charge,
            )?;
            if self.confirm_returning_batch(
                found,
                members,
                id,
                best,
                module,
                routed,
                &mut answer.selected,
                charge,
            )? {
                continue;
            }
            for &(this, position) in members {
                charge(1)?;
                let selected = Self::carrier_support(best, found[this].0.view);
                if let Some(level) = selected
                    && self.confirms_carrier(&found[this], id, level, routed, charge)?
                {
                    answer.selected[position] = Some(level);
                }
            }
        }
        Ok(answer)
    }

    /// Group eligible carriers without allocating keys for repeated competitors.
    fn selection_groups(
        &self,
        found: &[(&GlobRead, Cow<'_, [Held]>)],
        answer: &Redelivery<'_>,
        supporters: &BTreeMap<ItemId, Vec<(Level, usize)>>,
        memory: &Transient<'_>,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
    ) -> Result<SelectionGroups, LoadError> {
        let decl = &self.uses[self.glob_imports[0]];
        let mut groups = SelectionGroups::default();
        for (this, (_, held)) in found.iter().enumerate() {
            charge(1 + held.len())?;
            if !held.iter().any(|entry| entry.1.is_companion()) {
                continue;
            }
            for (position, (id, _)) in held.iter().enumerate() {
                // A recorded public selection is already maximal. Keep
                // all its evidence above, but never repeat a routed proof
                // that cannot add an identity or widen this member.
                if Self::maximal_replacement(answer.reads[this].known, *id, charge)? {
                    continue;
                }
                if !self.possibly_supported(held, *id, supporters, charge)? {
                    continue;
                }
                groups.add(
                    *id,
                    held,
                    (this, answer.reads[this].offset + position),
                    memory,
                    decl,
                    charge,
                )?;
            }
        }
        Ok(groups)
    }

    /// Apply a homogeneous group through one retained proof, without new buffers.
    #[expect(
        clippy::too_many_arguments,
        reason = "the existing group and its borrowed selection destination"
    )]
    fn confirm_returning_batch(
        &self,
        found: &[(&GlobRead, Cow<'_, [Held]>)],
        members: &[(usize, usize)],
        id: ItemId,
        best: [Option<(Level, usize)>; 2],
        module: usize,
        routed: &mut Routing<'_, '_, '_>,
        selected: &mut [Option<Level>],
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
    ) -> Result<bool, LoadError> {
        let Some(required) =
            self.uniform_returning_level(found, members, id, best, module, charge)?
        else {
            return Ok(false);
        };
        routed.returning(id, required, |proof| {
            for &(this, position) in members {
                charge(1)?;
                let Some(level) = Self::carrier_support(best, found[this].0.view) else {
                    continue;
                };
                // Recompute eligibility without retaining a second buffer.
                // Both passes charge their actual scans.
                let Some(actual) = self.carrier_requirement(&found[this], id, level, charge)?
                else {
                    continue;
                };
                if actual != required || found[this].0.target == module {
                    continue;
                }
                if proof.returns(found[this].0.target, charge)? {
                    selected[position] = Some(level);
                }
            }
            Ok(())
        })?;
        Ok(true)
    }

    /// Preserve the actual support choice for this carrier's view.
    fn carrier_support(best: [Option<(Level, usize)>; 2], view: usize) -> Option<Level> {
        match best {
            [Some((level, other)), _] if other != view => Some(level),
            [_, Some((level, _))] => Some(level),
            _ => None,
        }
    }

    /// The existing firm guard and actual carrier-level meet, with no route lookup.
    fn carrier_requirement(
        &self,
        (_, held): &(&GlobRead, Cow<'_, [Held]>),
        id: ItemId,
        level: Level,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
    ) -> Result<Option<Level>, LoadError> {
        if !self.preserves_firm_competitors(held, id, level, charge)? {
            return Ok(None);
        }
        charge(held.len())?;
        // INVARIANT: group members were formed only from this held list
        // at the selected canonical identity; discovery has not mutated it.
        let carrier = held
            .iter()
            .find_map(|(candidate, level)| (*candidate == id).then_some(level.plain()))
            .expect("a carrier holds its selected identity");
        Ok(Some(self.meet(level.plain(), carrier).plain()))
    }

    /// A homogeneous group borrows one proof; mixed groups retain ordinary routing.
    fn uniform_returning_level(
        &self,
        found: &[(&GlobRead, Cow<'_, [Held]>)],
        members: &[(usize, usize)],
        id: ItemId,
        best: [Option<(Level, usize)>; 2],
        reader: usize,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
    ) -> Result<Option<Level>, LoadError> {
        #[cfg(test)]
        if reference::single_returning() {
            return Ok(None);
        }
        let mut uniform = None;
        for &(this, _) in members {
            charge(1)?;
            let Some(level) = Self::carrier_support(best, found[this].0.view) else {
                continue;
            };
            let Some(required) = self.carrier_requirement(&found[this], id, level, charge)? else {
                continue;
            };
            if found[this].0.target == reader || required == Level::NOBODY {
                continue;
            }
            if uniform.is_some_and(|previous| previous != required) {
                return Ok(None);
            }
            uniform = Some(required);
        }
        Ok(uniform)
    }

    /// The widest routed support from each of at most two distinct views.
    fn independent_supporters(
        found: &[(&GlobRead, Cow<'_, [Held]>)],
        supporters: &[(Level, usize)],
        (excluded, generation): (&[usize], usize),
        id: ItemId,
        routed: &mut Routing<'_, '_, '_>,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
    ) -> Result<[Option<(Level, usize)>; 2], LoadError> {
        let mut best = [None, None];
        for &(level, other) in supporters {
            charge(1)?;
            if level.plain() == Level::NOBODY
                || excluded[other] == generation
                || !routed.ask(found[other].0.target, id, level.plain(), false)
            {
                continue;
            }
            let view = found[other].0.view;
            match best {
                [None, _] => best[0] = Some((level, view)),
                [Some((_, first)), None] if first != view => {
                    best[1] = Some((level, view));
                    break;
                }
                _ => {}
            }
        }
        Ok(best)
    }

    /// Whether this exact carrier already records its widest possible level.
    fn maximal_replacement(
        known: &[Held],
        id: ItemId,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
    ) -> Result<bool, LoadError> {
        charge(search_units(known.len()))?;
        Ok(replacement_level(known, id).is_some_and(|level| level.plain() == Level::PUBLIC))
    }

    /// Reject carriers that even the widest optimistic support cannot keep.
    fn possibly_supported(
        &self,
        held: &[Held],
        id: ItemId,
        supporters: &BTreeMap<ItemId, Vec<(Level, usize)>>,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
    ) -> Result<bool, LoadError> {
        charge(search_units(supporters.len()))?;
        let Some(&(ceiling, _)) = supporters.get(&id).and_then(|list| list.first()) else {
            return Ok(false);
        };
        // This support may later fail exclusion or rootedness. Any real
        // choice is no wider, so the existing firm guard can reject early.
        self.preserves_firm_competitors(held, id, ceiling, charge)
    }

    /// A selection must not discard a wider noncompanion competitor.
    fn preserves_firm_competitors(
        &self,
        held: &[Held],
        id: ItemId,
        level: Level,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
    ) -> Result<bool, LoadError> {
        charge(held.len())?;
        Ok(!held.iter().any(|(other, original)| {
            *other != id && !original.is_companion() && self.width_order(*original, level).is_lt()
        }))
    }

    /// A selected supporter must preserve wider firm competitors and prove
    /// the returning route at the visibility its actual carrier delivers.
    fn confirms_carrier(
        &self,
        found: &(&GlobRead, Cow<'_, [Held]>),
        id: ItemId,
        level: Level,
        routed: &mut Routing<'_, '_, '_>,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
    ) -> Result<bool, LoadError> {
        let Some(required) = self.carrier_requirement(found, id, level, charge)? else {
            return Ok(false);
        };
        Ok(routed.ask(found.0.target, id, required, true))
    }

    /// True when `module` binds `name` itself in the name space at `at`, as
    /// a definition or an explicit import.
    fn binds_itself(&self, module: usize, name: &str, at: usize) -> bool {
        self.scopes[module]
            .get(name)
            .is_some_and(|entry| !entry.defs[at].is_empty())
    }

    /// The widest level of the importers to which the glob read `read`
    /// drops its companions under `name` in the name space at `at` from the
    /// start of a settle, if it drops them.
    fn replacements(&self, read: &GlobRead, name: &str, at: usize) -> &[Held] {
        self.globs
            .drops
            .get(&(read.glob, read.target))
            .and_then(|names| names.get(name))
            .map_or(&[], |spaces| spaces[at].as_slice())
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
    /// space asked. Node and edge storage, including dominator scratch, is
    /// separately admitted before allocation. The glob reads of a module
    /// looked at for a name, those whose view holds it, and each view that
    /// holds the name looked at to find them (see [`Index::reads_holding`]),
    /// are a step each, charged before they are read. Every step is charged
    /// to the first glob import of the load.
    ///
    /// Companion-name lists, per-module sets, candidate groups, live graph
    /// scratch and pending replacement rows hold overlapping glob-budget
    /// leases. Each admission precedes the allocation it bounds.
    pub(crate) fn record_drops(&mut self, meter: &Meter) -> Result<usize, LoadError> {
        let Some(&first) = self.glob_imports.first() else {
            return Ok(0);
        };
        let decl = &self.uses[first];
        let (file, line) = (decl.file.clone(), decl.line);
        let memory = Transient::new(meter);
        let steps = Steps::LoadAt {
            memory: &memory,
            file: &file,
            line,
        };
        let companions = self.companion_names(&memory, meter, first)?;
        let mut supply: Option<Supply<'_>> = None;
        let mut row_scratch = None;
        let mut due: Vec<Replacement> = Vec::new();
        for module in 0..self.globs.reads.len() {
            let reads = self.reads_of(module);
            if reads.len() < 2 {
                continue;
            }
            meter.work(reads.len(), &file, Some(line))?;
            let names_memory = memory.child();
            let names = self.companion_queries(reads, &companions, &names_memory, meter, first)?;
            for (name, at) in names {
                let query_memory = Transient::new(meter);
                let query_steps = Steps::LoadAt {
                    memory: &query_memory,
                    file: &file,
                    line,
                };
                let holders = self
                    .admitted_holders(module, name, query_steps)
                    .map_err(crate::settle::storage_failure)?;
                meter.work(holders.len(), &file, Some(line))?;
                query_steps.reserve(
                    vec_first::<(&GlobRead, Cow<'_, [Held]>)>()
                        + holders.len() * vec_slot::<(&GlobRead, Cow<'_, [Held]>)>(),
                )?;
                let charge = |steps: usize| meter.work(steps, &file, Some(line));
                let mut rows =
                    QueryRows::new(self, name, &mut row_scratch, &memory, decl, &charge)?;
                // These are the context's exact constructor inputs, so its
                // match is constant work. Slot lookup/insertion allocates
                // nothing; admit both operations per holder before the loop.
                charge(holders.len().saturating_mul(2).saturating_add(1))?;
                let found = self.read_lists(holders, name, *at, Work::DropReads, Some(&mut rows));
                if supply.is_none() {
                    supply = Some(self.supply(meter, first)?);
                }
                let supply = supply.as_mut().expect("the supply graph was constructed");
                let redelivered = self.redelivered(
                    (module, name, *at),
                    &found,
                    &query_memory,
                    &mut Routing {
                        index: self,
                        supply,
                        reader: module,
                        name,
                        at: *at,
                        charge: &charge,
                        rows,
                    },
                    &|units| self.candidate_steps(units, first, meter),
                )?;
                if let Some(refused) = supply.refused.take() {
                    return Err(refused);
                }
                meter.work(found.len() + redelivered.selected.len(), &file, Some(line))?;
                for ((read, held), selected) in found.iter().zip(&redelivered.reads) {
                    for (position, (id, _)) in held.iter().enumerate() {
                        let Some(level) = redelivered.selected[selected.offset + position] else {
                            continue;
                        };
                        meter.work(search_units(selected.known.len()), &file, Some(line))?;
                        let known = replacement_level(selected.known, *id);
                        if known.is_none_or(|known| self.width_order(level, known).is_lt()) {
                            reserve_next(&mut due, steps)?;
                            due.push((read.glob, read.target, name.clone(), *at, *id, level));
                        }
                    }
                }
            }
        }
        drop((supply, companions, row_scratch));
        self.record_replacements(due, meter, first)
    }

    /// Publish pending replacements in their original discovery order.
    fn record_replacements(
        &mut self,
        due: Vec<Replacement>,
        meter: &Meter,
        first: usize,
    ) -> Result<usize, LoadError> {
        let recorded = due.len();
        for replacement in due {
            self.record_replacement(replacement, meter, first)?;
        }
        #[cfg(test)]
        reference::discovered();
        Ok(recorded)
    }

    /// Collect distinct companion queries under the caller's retained lease.
    fn companion_queries<'a>(
        &self,
        reads: &[GlobRead],
        companions: &'a [Vec<(SharedStr, usize)>],
        names_memory: &Transient<'_>,
        meter: &Meter,
        first: usize,
    ) -> Result<Vec<&'a (SharedStr, usize)>, LoadError> {
        let decl = &self.uses[first];
        let (file, line) = (&decl.file, decl.line);
        let entries = reads
            .iter()
            .map(|read| companions[read.view].len())
            .sum::<usize>();
        names_memory.charge(
            hash_map::<(*const u8, usize)>()
                + vec_first::<&(SharedStr, usize)>()
                + entries
                    * (hash_entry::<(*const u8, usize)>() + vec_slot::<&(SharedStr, usize)>()),
            file,
            Some(line),
        )?;
        meter.work(entries, file, Some(line))?;
        // View keys share the canonical text in globs.strings. Its
        // allocation identity is therefore a constant-size membership
        // key; hashing it never walks the identifier text again. Keep
        // processing order in the vector, independent of hash iteration.
        let mut seen = HashSet::with_hasher(FixedState::new());
        let names: Vec<_> = reads
            .iter()
            .flat_map(|read| &companions[read.view])
            .filter(|name| seen.insert((name.0.as_ptr(), name.1)))
            .collect();
        Ok(names)
    }

    /// Admit the names of potentially replaceable views before collecting.
    fn companion_names(
        &self,
        memory: &Transient<'_>,
        meter: &Meter,
        first: usize,
    ) -> Result<Vec<Vec<(SharedStr, usize)>>, LoadError> {
        let decl = &self.uses[first];
        meter.work(self.globs.views.len(), &decl.file, Some(decl.line))?;
        let names = self
            .globs
            .views
            .iter()
            .map(|view| view.names.len())
            .sum::<usize>();
        meter.work(names.saturating_mul(3), &decl.file, Some(decl.line))?;
        let candidates = self
            .globs
            .views
            .iter()
            .flat_map(|view| view.names.values())
            .flat_map(|lists| lists.iter())
            .map(Vec::len)
            .sum::<usize>();
        meter.work(
            candidates + names.saturating_mul(3),
            &decl.file,
            Some(decl.line),
        )?;
        memory.charge(
            vec_first::<Vec<(SharedStr, usize)>>()
                + self.globs.views.len()
                    * (vec_slot::<Vec<(SharedStr, usize)>>() + vec_first::<(SharedStr, usize)>())
                + names.saturating_mul(3 * vec_slot::<(SharedStr, usize)>()),
            &decl.file,
            Some(decl.line),
        )?;
        Ok(self
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
            .collect())
    }

    /// Admit and persist one new id or wider support ceiling.
    fn record_replacement(
        &mut self,
        (glob, target, name, at, id, level): Replacement,
        meter: &Meter,
        first: usize,
    ) -> Result<(), LoadError> {
        let decl = &self.uses[first];
        let mut bytes = 0;
        if self.globs.replaced_names.is_empty() {
            bytes += hash_map::<(SharedStr, [bool; 3])>();
        }
        let recorded = self.globs.replaced_names.get(name.as_str());
        if recorded.is_none() {
            bytes += hash_entry::<(SharedStr, [bool; 3])>();
        }
        let publish_name = recorded.is_none_or(|spaces| !spaces[at]);
        if self.globs.drops.is_empty() {
            bytes += btree_map::<(usize, usize), DropNames>();
        }
        // Retain the placement while admitting the row. Publication then uses
        // the same search, without allocating before the complete grant.
        let placement = self.globs.drops.entry((glob, target));
        let names = match &placement {
            std::collections::btree_map::Entry::Occupied(entry) => Some(entry.get()),
            std::collections::btree_map::Entry::Vacant(_) => None,
        };
        if names.is_none() {
            bytes += btree_entry::<(usize, usize), DropNames>()
                + hash_map::<(SharedStr, [Vec<Held>; 3])>();
        }
        let spaces = names.and_then(|names| names.get(name.as_str()));
        if spaces.is_none() {
            // The record keeps the text of the name after the views
            // that shared it are dropped.
            bytes += hash_entry::<(SharedStr, [Vec<Held>; 3])>() + shared_text(name.len());
        }
        let known = spaces.map(|spaces| &spaces[at]);
        meter.work(
            known.map_or(1, |entries| 1 + entries.len()),
            &decl.file,
            Some(decl.line),
        )?;
        let position = known.map_or(Err(0), |entries| {
            entries.binary_search_by_key(&id, |entry| entry.0)
        });
        if position.is_err() {
            bytes += vec_slot::<Held>()
                + if known.is_none_or(Vec::is_empty) {
                    vec_first::<Held>()
                } else {
                    0
                };
        }
        meter.items(bytes, &decl.file, Some(decl.line))?;
        if publish_name {
            self.globs.replaced_names.entry(name.clone()).or_default()[at] = true;
        }
        let entries = &mut placement.or_default().entry(name).or_default()[at];
        match position {
            Ok(place) => entries[place].1 = level,
            Err(place) => {
                entries.reserve_exact(1);
                entries.insert(place, (id, level));
            }
        }
        self.globs.dropped += 1;
        Ok(())
    }

    /// Genuine definition roots at their own written visibility levels.
    fn definition_supply(
        &self,
        meter: &Meter,
        first: usize,
        steps: Steps<'_>,
    ) -> Result<BTreeMap<ItemId, Vec<SupplyRoute>>, LoadError> {
        steps.reserve(btree_map::<ItemId, Vec<SupplyRoute>>())?;
        let mut roots: BTreeMap<ItemId, Vec<SupplyRoute>> = BTreeMap::new();
        // Scope origins may combine this definition with named imports.
        // Only the actual definition's own visibility belongs to its root.
        for (module, record) in self.modules.iter().enumerate() {
            for id in &record.defs {
                let item = &self.items[id.0];
                steps.reserve(shared_text(item.name.len()))?;
                let name = SharedStr::from(item.name.as_str());
                let vis = if item.kind == crate::item::SymbolKind::MacroRules {
                    &Vis::Public
                } else {
                    &item.visibility
                };
                let visibility_memory = steps.temporary();
                let visibility_steps = visibility_memory.as_ref().map_or(steps, |m| steps.at(m));
                let level = self
                    .read_level_of(vis, module, visibility_steps)
                    .map_err(crate::settle::storage_failure)?;
                drop(visibility_memory);
                for (at, ns) in Namespace::ALL.into_iter().enumerate() {
                    self.candidate_steps(1, first, meter)?;
                    if !item.in_namespace(ns) {
                        continue;
                    }
                    if !roots.contains_key(id) {
                        steps.reserve(btree_entry::<ItemId, Vec<SupplyRoute>>())?;
                    }
                    let into = roots.entry(*id).or_default();
                    reserve_next(into, steps)?;
                    into.push(((module, name.clone(), at), level));
                }
            }
        }
        Ok(roots)
    }

    /// Every import of an empty namespace cycle has an actual answer in
    /// another namespace, either bound or independently rooted in its globs.
    pub(crate) fn grounded_cycle_imports(
        &self,
        imports: &BTreeMap<usize, Vec<usize>>,
        meter: &Meter,
        proof_steps: Steps<'_>,
    ) -> Result<Option<Vec<crate::settle::FinalFallback>>, LoadError> {
        let mut supply = None;
        let mut certified = Vec::new();
        for (&position, spaces) in imports {
            let decl = &self.uses[position];
            meter.work(1, &decl.file, Some(decl.line))?;
            tally::add(Work::SupplySteps, 1);
            if self.use_bound[position].iter().any(|bound| *bound) {
                continue;
            }
            let Some(&first) = self.glob_imports.first() else {
                return Ok(None);
            };
            let memory = Transient::new(meter);
            let steps = Steps::LoadAt {
                memory: &memory,
                file: &decl.file,
                line: decl.line,
            };
            let answers = self.fallback_supply(position, spaces, steps)?;
            self.drain_lookups(meter, &decl.file, Some(decl.line))?;
            if answers.is_empty() {
                return Ok(None);
            }
            if supply.is_none() {
                supply = Some(self.supply(meter, first)?);
            }
            let supply = supply.as_mut().expect("the supply graph was constructed");
            let crate::index::UseKind::Named { name, .. } = &decl.kind else {
                unreachable!("only named imports belong to a namespace cycle");
            };
            supply
                .memory
                .charge(shared_text(name.len()), &decl.file, Some(decl.line))?;
            let name = SharedStr::from(name.as_str());
            let charge = |units| meter.work(units, &decl.file, Some(decl.line));
            let mut grounded = None;
            for answer in answers {
                let at = crate::settle::slot(answer.ns);
                let holding = self.retained_holding(
                    supply,
                    &charge,
                    &(answer.id, name.clone(), at, answer.level, false),
                    None,
                )?;
                if !holding.holds(decl.module) {
                    continue;
                }
                if let Some(source) = answer.source {
                    supply.memory.charge(
                        shared_text(source.name.len()),
                        &decl.file,
                        Some(decl.line),
                    )?;
                    let source_name = SharedStr::from(source.name.as_str());
                    let holding = self.retained_holding(
                        supply,
                        &charge,
                        &(
                            answer.id,
                            source_name,
                            crate::settle::slot(source.ns),
                            answer.level,
                            false,
                        ),
                        None,
                    )?;
                    if !holding.holds(source.module) {
                        continue;
                    }
                }
                grounded = Some(crate::settle::FinalFallback {
                    position,
                    at,
                    id: answer.id,
                });
                break;
            }
            let Some(answer) = grounded else {
                return Ok(None);
            };
            reserve_next(&mut certified, proof_steps)?;
            certified.push(answer);
        }
        Ok(Some(certified))
    }

    /// Certify actual inaccessible cuts through independently rooted routes.
    pub(crate) fn grounded_shadow_cuts(
        &self,
        cuts: &[crate::settle::ShadowCut],
        meter: &Meter,
    ) -> Result<bool, LoadError> {
        if cuts.is_empty() {
            return Ok(false);
        }
        let Some(&first) = self.glob_imports.first() else {
            return Ok(false);
        };
        let mut supply = self.supply(meter, first)?;
        for cut in cuts {
            let decl = &self.uses[first];
            supply.memory.charge(
                shared_text(cut.source.name.len()),
                &decl.file,
                Some(decl.line),
            )?;
            let name = SharedStr::from(cut.source.name.as_str());
            let charge = |units| meter.work(units, &decl.file, Some(decl.line));
            let holding = self.retained_holding(
                &mut supply,
                &charge,
                &(
                    cut.id,
                    name,
                    crate::settle::slot(cut.source.ns),
                    Level::within(cut.source.module),
                    false,
                ),
                None,
            )?;
            if !holding.holds(cut.source.module) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Build the admitted supplier index once for all qualified holder queries.
    fn supply<'m>(&self, meter: &'m Meter, first: usize) -> Result<Supply<'m>, LoadError> {
        let memory = Transient::new(meter);
        let decl = &self.uses[first];
        let charge = |bytes| memory.charge(bytes, &decl.file, Some(decl.line));
        charge(
            vec_first::<Vec<(usize, usize, Level)>>()
                + self.modules.len()
                    * (vec_slot::<Vec<(usize, usize, Level)>>()
                        + vec_first::<(usize, usize, Level)>())
                + btree_map::<(ItemId, Place), Vec<SupplyRoute>>()
                + btree_map::<HoldingKey, Holding>(),
        )?;
        let mut readers: Vec<Vec<(usize, usize, Level)>> = vec![Vec::new(); self.modules.len()];
        self.candidate_steps(self.globs.reads.len(), first, meter)?;
        let cut_reader = cut::reader(&self.globs.reads);
        for (module, reads) in self.globs.reads.iter().enumerate() {
            for (place, read) in reads.iter().enumerate() {
                self.candidate_steps(1, first, meter)?;
                let visibility_memory = memory.child();
                let level = self
                    .read_level_of(
                        &self.uses[read.glob].vis,
                        module,
                        Steps::LoadAt {
                            memory: &visibility_memory,
                            file: &decl.file,
                            line: decl.line,
                        },
                    )
                    .map_err(crate::settle::storage_failure)?;
                drop(visibility_memory);
                let into = &mut readers[read.target];
                reserve_next(
                    into,
                    Steps::LoadAt {
                        memory: &memory,
                        file: &decl.file,
                        line: decl.line,
                    },
                )?;
                into.push((module, place, level));
            }
        }
        let steps = Steps::LoadAt {
            memory: &memory,
            file: &decl.file,
            line: decl.line,
        };
        let mut roots = self.definition_supply(meter, first, steps)?;
        let mut named: BTreeMap<(ItemId, Place), Vec<SupplyRoute>> = BTreeMap::new();
        for (position, import) in self.uses.iter().enumerate() {
            self.candidate_steps(1, first, meter)?;
            let crate::index::UseKind::Named { name, .. } = &import.kind else {
                continue;
            };
            let route_memory = memory.child();
            let routes = self.import_supply(position, steps.at(&route_memory))?;
            self.drain_lookups(meter, &decl.file, Some(decl.line))?;
            if routes.is_empty() {
                continue;
            }
            charge(shared_text(name.len()))?;
            let name = SharedStr::from(name.as_str());
            for route in routes {
                let destination = (import.module, name.clone(), crate::settle::slot(route.ns));
                if let Some(source) = route.source {
                    charge(shared_text(source.name.len()))?;
                    let source = (
                        source.module,
                        SharedStr::from(source.name.as_str()),
                        crate::settle::slot(source.ns),
                    );
                    let key = (route.id, source);
                    if !named.contains_key(&key) {
                        charge(btree_entry::<(ItemId, Place), Vec<SupplyRoute>>())?;
                    }
                    let into = named.entry(key).or_default();
                    reserve_next(into, steps)?;
                    into.push((destination, route.level));
                } else {
                    if !roots.contains_key(&route.id) {
                        charge(btree_entry::<ItemId, Vec<SupplyRoute>>())?;
                    }
                    let into = roots.entry(route.id).or_default();
                    reserve_next(into, steps)?;
                    into.push((destination, route.level));
                }
            }
        }
        Ok(Supply {
            readers,
            roots,
            named,
            holdings: BTreeMap::new(),
            cut_reader,
            cuts: cut::Cuts::default(),
            refused: None,
            memory,
        })
    }

    /// Whether `from` holds `id` under `name` in the name space at `at`
    /// through `reader` on every route (`returning`), or through a route
    /// independent of `reader` otherwise. The holder graph and its
    /// dominators prove both sides of a cycle replacement. A refusal by
    /// the budget answers false and is kept in `supply`, then propagated
    /// before a drop can be recorded.
    #[expect(
        clippy::too_many_arguments,
        reason = "one question, the search state and the budget it is charged to"
    )]
    fn supplied(
        &self,
        supply: &mut Supply<'_>,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
        from: usize,
        reader: usize,
        name: &SharedStr,
        at: usize,
        id: ItemId,
        required: Level,
        returning: bool,
        rows: Option<&mut QueryRows<'_, '_, '_>>,
    ) -> bool {
        let required = required.plain();
        if supply.refused.is_some() || from == reader || required == Level::NOBODY {
            return false;
        }
        if !returning
            && self.scopes[from].get(name.as_str()).is_some_and(|entry| {
                entry.defs[at]
                    .get(id)
                    .is_some_and(|origin| origin.link.is_none())
            })
            && let Some(roots) = supply.roots.get(&id)
        {
            let mut step = |units| {
                tally::add(Work::SupplySteps, units);
                charge(units)
            };
            if let Err(refused) = step(1 + roots.len()) {
                supply.refused = Some(refused);
                return false;
            }
            for (place, level) in roots {
                if place.0 == from && place.1 == *name && place.2 == at {
                    match self.level_covers(*level, required, &mut step) {
                        Ok(true) => return true,
                        Ok(false) => {}
                        Err(refused) => {
                            supply.refused = Some(refused);
                            return false;
                        }
                    }
                }
            }
        }
        if supply.cut_reader == Some(reader) {
            let key = (id, name.clone(), at, required, returning);
            return match self
                .retained_cut(supply, charge, reader, &key, rows)
                .and_then(|proof| proof.answer(from, returning, charge))
            {
                Ok(answer) => answer,
                Err(refused) => {
                    supply.refused = Some(refused);
                    false
                }
            };
        }
        let other = (id, name.clone(), at, required, !returning);
        let key = if supply
            .holdings
            .get(&other)
            .is_some_and(|holding| holding.firm_equivalent)
        {
            other
        } else {
            (id, name.clone(), at, required, returning)
        };
        let holding = match self.retained_holding(supply, charge, &key, rows) {
            Ok(holding) => holding,
            Err(refused) => {
                supply.refused = Some(refused);
                return false;
            }
        };
        // Decisive, not defensive: a holder that the item reaches only
        // through the reading module can still deliver it without the
        // reading module's companions, when a support that only importers
        // inside the reading module can see drops them from the view such a
        // holder reads. Its delivery would then widen the drop to every
        // importer; this check keeps it to the support's own level
        // (`a_drop_justified_by_a_holder_that_reads_the_item_back_from_the_reader_stays_within_the_reader_in_every_order`).
        holding.holds(from) && holding.dominates(reader, from) == returning
    }

    /// Admit and retain each exact visibility-qualified graph once.
    fn retained_holding<'s>(
        &self,
        supply: &'s mut Supply<'_>,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
        key: &HoldingKey,
        rows: Option<&mut QueryRows<'_, '_, '_>>,
    ) -> Result<&'s Holding, LoadError> {
        #[cfg(test)]
        reference::retrieved_holder();
        if !supply.holdings.contains_key(key) {
            let holding = self.holding(supply, charge, key, rows)?;
            let decl = &self.uses[self.glob_imports[0]];
            supply.memory.charge(
                btree_entry::<HoldingKey, Holding>(),
                &decl.file,
                Some(decl.line),
            )?;
            supply.holdings.insert(key.clone(), holding);
        }
        Ok(&supply.holdings[key])
    }

    /// The holders of `id` under `name` in the name space at `at` and their
    /// dominators (see [`Holding`]): the holders are found from the modules
    /// that bind the name with the item, along the glob reads whose views
    /// hold it, and the dominators by the algorithm of Lengauer and Tarjan,
    /// with path compression. Each glob read looked at, each holder and each
    /// step of the algorithm is charged before it is made, as bytes and as
    /// work.
    fn holding(
        &self,
        supply: &Supply<'_>,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
        query: &HoldingKey,
        mut rows: Option<&mut QueryRows<'_, '_, '_>>,
    ) -> Result<Holding, LoadError> {
        let (id, name, at, required, carried) = query;
        let (id, at, required, carried) = (*id, *at, *required, *carried);
        let memory = supply.memory.child();
        let decl = &self.uses[self.glob_imports[0]];
        let admit = |bytes| memory.charge(bytes, &decl.file, Some(decl.line));
        admit(SUPPLY_BASE_BYTES)?;
        let mut step = |count: usize| {
            tally::add(Work::SupplySteps, count);
            charge(count)
        };
        // A place is a module, name and namespace. Named imports are
        // dependency edges, including renamings, rather than independent
        // definition roots. A returning named route must pass the same
        // dominance proof as a returning glob route.
        step(1)?;
        let mut graph = HolderGraph::new(at);
        let mut firm_equivalent = true;
        for (root, level) in supply
            .roots
            .get(&id)
            .into_iter()
            .flatten()
            .filter(|(root, _)| root.2 == at)
        {
            step(1)?;
            if !self.level_covers(*level, required, &mut step)? {
                continue;
            }
            firm_equivalent &= !level.is_companion();
            if level.is_companion() && !carried {
                continue;
            }
            graph.edge(0, root, &mut step, &admit)?;
        }
        let mut next = 1;
        while let Some(held) = graph.nodes.get(next).cloned() {
            firm_equivalent &= self.visit_holder_routes(
                supply,
                query,
                &held,
                &mut step,
                &mut rows,
                |destination, step| graph.edge(next, destination, step, &admit),
            )?;
            next += 1;
        }
        let stack_steps = Steps::LoadAt {
            memory: &memory,
            file: &decl.file,
            line: decl.line,
        };
        let span = holder_spans(
            &dominators(
                &graph.successors,
                &graph.predecessors,
                &mut step,
                stack_steps,
            )?,
            &mut step,
            stack_steps,
        )?;
        Holding::retain(
            graph.place,
            &span,
            firm_equivalent,
            name,
            at,
            &supply.memory,
            decl,
        )
    }

    /// True when a glob of `module` brings in something under `name` in the
    /// name space at `at`. Each read looked at, up to the first that brings
    /// something in, and each view that holds the name looked at to find the
    /// reads, is a lookup step, counted where `steps` says.
    pub(crate) fn read_globbed_any(
        &self,
        module: usize,
        name: &str,
        at: usize,
        steps: Steps<'_>,
    ) -> Result<bool, Fail> {
        tally::add(Work::AnyLookups, 1);
        if self.replaced(name, at) {
            let memory = steps.temporary();
            let bounded = memory.as_ref().map_or(steps, |memory| steps.at(memory));
            self.reserve_glob_lookup(module, name, at, bounded)?;
            return Ok(!self.read_globbed(module, name, at, bounded)?.is_empty());
        }
        if let Some(views) = self.globs.holding.get(name)
            && views.len() < self.reads_of(module).len()
        {
            steps
                .reserve(vec_first::<&[usize]>() + views.len() * vec_slot::<&[usize]>())
                .map_err(Fail::Storage)?;
        }
        let mut reads = self.reads_holding(module, name);
        let mut visited = 0;
        let found = reads.any(|read| {
            tally::add(Work::LookupReads, 1);
            visited += 1;
            self.globs.views[read.view]
                .names
                .get(name)
                .is_some_and(|lists| !lists[at].is_empty())
        });
        self.count_lookups(steps, module, reads.scanned(), visited);
        Ok(found)
    }

    /// Count the lookup steps of a lookup in `module` where `steps` says:
    /// `reads` glob reads looked at and `scanned` views that hold the name
    /// looked at to find them. During the load they go to the index's
    /// count, which is drained into the work of the load at its next
    /// checkpoint (see [`Index::settle_lookups`]); after it, to the meter of
    /// the query.
    fn count_lookups(&self, steps: Steps<'_>, module: usize, scanned: usize, reads: usize) {
        tally::add(Work::LookupScans, scanned);
        let units = scanned + reads;
        if units == 0 {
            return;
        }
        match steps {
            Steps::Load | Steps::LoadAt { .. } => self.lookups.add(units),
            Steps::Query(meter) => {
                tally::add(Work::LookupCharges, units);
                meter.count_query(units, module);
            }
            Steps::Observed(_) | Steps::QueryAt { .. } | Steps::QueryItems { .. } => {
                tally::add(Work::LookupCharges, units);
                steps
                    .query_meter()
                    .expect("query proof has a meter")
                    .count_query(units, module);
            }
        }
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
        self.defs_held_iter(module, class, defs).collect()
    }

    /// The shared visible-definition transfer, without choosing output storage.
    fn defs_held_iter<'a>(
        &'a self,
        module: usize,
        class: &'a [bool],
        defs: &'a Bound,
    ) -> impl Iterator<Item = Held> + 'a {
        defs.iter().filter_map(move |(id, origin)| {
            origin
                .visibilities()
                .map(|vis| self.level_of(vis, module, Steps::Load))
                .filter(|level| self.allowed(module, class, *level))
                .reduce(|left, right| self.wider(left, right))
                .map(|level| {
                    (
                        *id,
                        if origin.companion {
                            level.companion()
                        } else {
                            level
                        },
                    )
                })
        })
    }

    /// Of the items `held`, sorted by item, with repeats, keep each item once
    /// at its widest level.
    fn widest(&self, held: &mut Vec<Held>, site: usize, meter: &Meter) -> Result<(), LoadError> {
        self.widest_charged(held, &|units| self.candidate_steps(units, site, meter))
    }

    /// Sort and merge candidate evidence under the caller's work account.
    fn widest_charged(
        &self,
        held: &mut Vec<Held>,
        charge: &impl Fn(usize) -> Result<(), LoadError>,
    ) -> Result<(), LoadError> {
        if held.len() < 2 {
            return Ok(());
        }
        charge(sort_units(held.len()) + held.len())?;
        held.sort_unstable_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| self.width_order(left.1, right.1))
        });
        held.dedup_by(|later, retained| {
            if later.0 != retained.0 {
                return false;
            }
            if !later.1.is_companion() {
                retained.1 = retained.1.plain();
            }
            true
        });
        Ok(())
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
        self.glob_held_iter(module, class, glob, held.iter().copied(), into);
    }

    /// The same exposure policy for an admitted borrowed selection.
    fn glob_held_iter(
        &self,
        module: usize,
        class: &[bool],
        glob: Level,
        held: impl Iterator<Item = Held>,
        into: (&mut Vec<Held>, &mut Vec<Held>),
    ) {
        let (seen, hidden) = into;
        for (id, level) in held {
            let mut met = self.meet(glob, level).plain();
            if level.is_companion() {
                met = met.companion();
            }
            if self.allowed(module, class, met) {
                seen.push((id, met));
            } else {
                hidden.push((id, met));
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
    fn exposed(
        &self,
        seen: Vec<Held>,
        hidden: Vec<Held>,
        site: usize,
        meter: &Meter,
    ) -> Result<Vec<Held>, LoadError> {
        self.exposed_charged(seen, hidden, &|units| {
            self.candidate_steps(units, site, meter)
        })
    }

    /// The shared exposure transfer with its caller's actual work category.
    fn exposed_charged(
        &self,
        mut seen: Vec<Held>,
        mut hidden: Vec<Held>,
        charge: &impl Fn(usize) -> Result<(), LoadError>,
    ) -> Result<Vec<Held>, LoadError> {
        self.expose_in_place(&mut seen, &mut hidden, charge)?;
        Ok(seen)
    }

    /// Shared exposure retaining the caller's admitted backing buffers.
    fn expose_in_place(
        &self,
        seen: &mut Vec<Held>,
        hidden: &mut Vec<Held>,
        charge: &impl Fn(usize) -> Result<(), LoadError>,
    ) -> Result<(), LoadError> {
        self.widest_charged(seen, charge)?;
        charge(seen.len() + hidden.len() + sort_units(hidden.len()))?;
        let Some(widest) = seen
            .iter()
            .map(|held| held.1)
            .reduce(|left, right| self.wider(left, right))
        else {
            return Ok(());
        };
        hidden.retain(|(id, _)| seen.binary_search_by_key(id, |held| held.0).is_err());
        hidden.sort_unstable_by_key(|held| held.0);
        hidden.dedup_by_key(|held| held.0);
        if !ambiguity_seen(seen.len() + hidden.len(), seen.len()) {
            return Ok(());
        }
        charge(hidden.len() + sort_units(seen.len() + hidden.len()))?;
        seen.extend(hidden.drain(..).map(|(id, _)| (id, widest.companion())));
        seen.sort_unstable_by_key(|held| held.0);
        Ok(())
    }

    /// What `target` exports under `name` in the name space at `at` to the
    /// importers of `class`: the bindings that shadow globs, when there are
    /// any, and those importers can access; else, when no explicit import
    /// they can name keeps the name from the globs, what the globs bring in
    /// (see [`Index::exposed`]). Sorted. Reading what the globs bring in reads
    /// again the glob reads of the target whose view holds the name (see
    /// [`Index::reads_holding`]), a step of work each, and each view that
    /// holds the name looked at to find them is a step too, charged for the
    /// glob import at `site` before they are made.
    fn export_ids(
        &self,
        target: usize,
        class: &[bool],
        name: &str,
        at: usize,
        site: usize,
        meter: &Meter,
    ) -> Result<Vec<Held>, LoadError> {
        Ok(self.exported(target, class, name, at, site, meter)?.0)
    }

    /// [`Index::export_ids`], and whether the globs of `target` deliver an
    /// item under `name` in the name space at `at`: false where the target
    /// binds the name itself or keeps it from its globs.
    fn exported(
        &self,
        target: usize,
        class: &[bool],
        name: &str,
        at: usize,
        site: usize,
        meter: &Meter,
    ) -> Result<(Vec<Held>, bool), LoadError> {
        if let Some(defs) = self.scopes[target]
            .get(name)
            .map(|entry| &entry.defs[at])
            .filter(|defs| !defs.is_empty())
        {
            return Ok((self.defs_held(target, class, defs), false));
        }
        if self.blocked_in(target, name, at) {
            return Ok((Vec::new(), false));
        }
        let reads = self.reads_holding(target, name);
        self.charge_steps(reads.scanned() + reads.len(), site, meter)?;
        tally::add(Work::Recomputations, 1);
        let (seen, hidden) = if self.flagged(name, at) {
            self.split_globs_delivered(target, reads, class, name, at)
        } else {
            self.split_globs_plain(target, reads, class, name, at)
        };
        let delivered = !seen.is_empty() || !hidden.is_empty();
        Ok((self.exposed(seen, hidden, site, meter)?, delivered))
    }

    /// What the glob reads `reads` of `target` bring in under `name`, as the
    /// items the importers of `class` can access and those they cannot,
    /// from what each glob delivers (see [`Index::delivered`]).
    fn split_globs_delivered(
        &self,
        target: usize,
        reads: Holders<'_>,
        class: &[bool],
        name: &str,
        at: usize,
    ) -> (Vec<Held>, Vec<Held>) {
        let (mut seen, mut hidden) = (Vec::new(), Vec::new());
        for (read, held) in
            self.delivered(target, reads, name, at, Some(class), Work::RecomputedReads)
        {
            let glob = self.level_of(&self.uses[read.glob].vis, target, Steps::Load);
            self.glob_held(target, class, glob, &held, (&mut seen, &mut hidden));
        }
        (seen, hidden)
    }

    /// [`Index::split_globs_delivered`] under a name that no view holds a
    /// companion under, where each glob read delivers what its view holds:
    /// each view's list is read where it is, and none is copied.
    fn split_globs_plain(
        &self,
        target: usize,
        reads: Holders<'_>,
        class: &[bool],
        name: &str,
        at: usize,
    ) -> (Vec<Held>, Vec<Held>) {
        let (mut seen, mut hidden) = (Vec::new(), Vec::new());
        for read in reads {
            tally::add(Work::RecomputedReads, 1);
            let Some(held) = self.globs.views[read.view]
                .names
                .get(name)
                .map(|lists| &lists[at])
                .filter(|held| !held.is_empty())
            else {
                continue;
            };
            let glob = self.level_of(&self.uses[read.glob].vis, target, Steps::Load);
            self.glob_held(target, class, glob, held, (&mut seen, &mut hidden));
        }
        (seen, hidden)
    }

    /// Add `ids`, sorted by item with one entry for each, to the items of
    /// `name` in the name space at `at` of `view`, charging what that
    /// allocates. An item already held is taken again only at a wider level.
    /// A name new to the view puts the view among the holders of the name
    /// (see [`Index::reads_holding`]), and an entry held as a companion flags
    /// the name in its name space (see [`Index::flagged`]). Returns the
    /// entries that were new or wider.
    fn add_to_view<'m>(
        &mut self,
        view: usize,
        name: &str,
        at: usize,
        ids: &[Held],
        meter: &'m Meter,
    ) -> Result<Added<'m>, LoadError> {
        let memory = Transient::new(meter);
        if ids.is_empty() {
            return Ok(Added {
                ids: Vec::new(),
                _memory: memory,
            });
        }
        let decl = &self.uses[self.globs.views[view].site];
        memory.charge(
            vec_first::<Held>() + ids.len() * vec_slot::<Held>(),
            &decl.file,
            Some(decl.line),
        )?;
        self.charge_steps(ids.len(), self.globs.views[view].site, meter)?;
        tally::add(Work::CandidateSteps, ids.len());
        let held = self.globs.views[view].names.get(name);
        let mut new = 0;
        let fresh: Vec<Held> = ids
            .iter()
            .copied()
            .filter(|(id, level)| {
                let known = held.and_then(|lists| {
                    lists[at]
                        .binary_search_by_key(id, |known| known.0)
                        .ok()
                        .map(|place| lists[at][place].1)
                });
                match known {
                    None => {
                        new += 1;
                        true
                    }
                    Some(known) => {
                        self.width_order(*level, known).is_lt()
                            || known.is_companion() && !level.is_companion()
                    }
                }
            })
            .collect();
        if fresh.is_empty() {
            return Ok(Added {
                ids: fresh,
                _memory: memory,
            });
        }
        let mut bytes = new * vec_slot::<Held>();
        if held.is_none_or(|lists| lists[at].is_empty()) {
            bytes += vec_first::<Held>();
        }
        // A name new to the view takes an entry, and its text the first time
        // any view holds it, and the view joins the holders of the name.
        let key = if held.is_some() {
            None
        } else {
            bytes += NAME_BYTES + holder_bytes(!self.globs.holding.contains_key(name));
            let known = self.globs.strings.get(name).cloned();
            if known.is_none() {
                bytes += shared_text(name.len()) + hash_entry::<SharedStr>();
            }
            let text = known.is_none();
            Some((known, text))
        };
        // A companion may be the first held under the name.
        if fresh.iter().any(|entry| entry.1.is_companion())
            && !self.globs.companions.contains_key(name)
        {
            bytes += hash_entry::<(SharedStr, [bool; 3])>();
        }
        let scratch = Transient::new(meter);
        if new < fresh.len() {
            scratch.charge(
                vec_first::<Held>() + fresh.len() * vec_slot::<Held>(),
                &decl.file,
                Some(decl.line),
            )?;
        }
        self.charge_tables(bytes, self.globs.views[view].site, meter)?;
        // Text construction belongs to the complete table/name grant too.
        let key = key.map(|(known, text)| (known.unwrap_or_else(|| SharedStr::from(name)), text));
        // Each entry as the view holds it once added: an item new to the
        // view as it comes, and one it holds as wide as the wider of the two
        // routes, a companion only if both are.
        let widened = (new < fresh.len()).then(|| self.widened_entries(view, name, at, &fresh));
        let finals = widened.as_deref().unwrap_or(&fresh);
        #[cfg(test)]
        self.assert_widened(view, name, at, finals);
        self.store_entries(view, name, at, key, finals);
        Ok(Added {
            ids: fresh,
            _memory: memory,
        })
    }

    /// The levels retained after incoming routes widen existing entries.
    fn widened_entries(&self, view: usize, name: &str, at: usize, fresh: &[Held]) -> Vec<Held> {
        let held = self.globs.views[view]
            .names
            .get(name)
            .map(|lists| &lists[at]);
        fresh
            .iter()
            .map(|entry| {
                let known = held.and_then(|held| {
                    held.binary_search_by_key(&entry.0, |known| known.0)
                        .ok()
                        .map(|place| held[place].1)
                });
                let Some(known) = known else { return *entry };
                let mut level = if self.width_order(entry.1, known).is_lt() {
                    entry.1.plain()
                } else {
                    known.plain()
                };
                if known.is_companion() && entry.1.is_companion() {
                    level = level.companion();
                }
                (entry.0, level)
            })
            .collect()
    }

    /// Store `finals`, charged, as the items of `name` in the name space at
    /// `at` of `view`: with `key`, the shared text of a name new to the
    /// view, which puts the view among the holders of the name and is
    /// stored as the load's copy of the text when that is new too, and a
    /// companion among them flags the name in its name space.
    fn store_entries(
        &mut self,
        view: usize,
        name: &str,
        at: usize,
        key: Option<(SharedStr, bool)>,
        finals: &[Held],
    ) {
        if finals.iter().any(|entry| entry.1.is_companion()) {
            let key = key
                .as_ref()
                .map(|(key, _)| key.clone())
                .or_else(|| self.globs.strings.get(name).cloned())
                .expect("a name a view holds is interned");
            self.globs.companions.entry(key).or_default()[at] = true;
        }
        let lists = match key {
            Some((key, text)) => {
                if text {
                    self.globs.strings.insert(key.clone());
                }
                self.globs
                    .holding
                    .entry(key.clone())
                    .or_default()
                    .push(view);
                self.globs.views[view].names.entry(key).or_default()
            }
            None => self.globs.views[view]
                .names
                .get_mut(name)
                .expect("the name is held"),
        };
        for entry in finals {
            match lists[at].binary_search_by_key(&entry.0, |known| known.0) {
                Ok(place) => lists[at][place] = *entry,
                Err(place) => lists[at].insert(place, *entry),
            }
        }
    }

    /// Fail unless each entry of `finals` is new to the items of `name` in
    /// the name space at `at` of `view`, or holds an item it already holds
    /// at least as widely, and as itself where it was: a view is only added
    /// to or widened, never narrowed or made a companion again.
    #[cfg(test)]
    fn assert_widened(&self, view: usize, name: &str, at: usize, finals: &[Held]) {
        let Some(lists) = self.globs.views[view].names.get(name) else {
            return;
        };
        for (id, level) in finals {
            if let Ok(place) = lists[at].binary_search_by_key(id, |known| known.0) {
                let known = lists[at][place].1;
                assert!(
                    self.width_order(*level, known).is_le()
                        && (known.is_companion() || !level.is_companion()),
                    "the view {view} narrows {id:?} under {name} in the name space {at}: \
                     {known:?} to {level:?}"
                );
            }
        }
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

    /// Charge `units` steps of glob table work, for the glob import at
    /// `site`, before they are made; they are never released.
    fn charge_steps(&self, units: usize, site: usize, meter: &Meter) -> Result<(), LoadError> {
        let decl = &self.uses[site];
        meter.work(units, &decl.file, Some(decl.line))
    }

    /// Charge candidate visits and sorting units before the operation.
    /// A visit includes its bounded logarithmic map/set lookup or binary
    /// search; sorting uses `n * (1 + floor(log2(n)))` units. These are
    /// algorithmic work units rather than individual CPU instructions.
    fn candidate_steps(&self, units: usize, site: usize, meter: &Meter) -> Result<(), LoadError> {
        self.charge_steps(units, site, meter)?;
        tally::add(Work::CandidateSteps, units);
        Ok(())
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
            turns: 0,
            passive: BTreeMap::new(),
            places: BTreeMap::new(),
            site,
            hidden: false,
            unseen: None,
        });
        self.globs.view_of.insert((target, class.into()), view);
        self.globs.exports[target].push(view);
        // The target's reads whose globs reach the view now pass on what
        // they bring in, and those no view of the target can access are
        // covert.
        self.turn_readers(target, site, meter)?;
        // What the target exports to the class, gathered in one pass over
        // its bindings and over the views its globs that reach the class
        // read, then added to the view. Every read of one view brings in the
        // same lists, so each view is gathered once, however many globs of
        // the target read it. What the gathering allocates is charged before
        // it is allocated, as a change waiting to be passed on, and released
        // once it is added. Each read listed and each name gathered is a
        // step of work. A view read through a glob whose level the class
        // cannot access brings in only items the class cannot access, so it
        // is not gathered name by name (see [`Index::gather`]); in the
        // reference mode of the tests it is.
        let reads = self.reads_of(target).len();
        let listed = vec_first::<(usize, Level)>() + reads * vec_slot::<(usize, Level)>();
        self.charge_work(listed, site, meter)?;
        self.charge_steps(reads, site, meter)?;
        let exact = Self::exact_mode();
        let covert = |glob: Level| !exact && !self.allowed(target, class, glob);
        let mut sources: Vec<(usize, Level)> = self
            .reads_of(target)
            .iter()
            .map(|read| {
                (
                    read.view,
                    self.level_of(&self.uses[read.glob].vis, target, Steps::Load),
                )
            })
            .collect();
        sources.sort_unstable_by_key(|(source, glob)| (covert(*glob), *source, *glob));
        sources.dedup();
        let split = sources.partition_point(|(_, glob)| !covert(*glob));
        let (shown, hidden) = sources.split_at(split);
        let gathered = self.gather_bytes(target, shown);
        self.charge_work(gathered, site, meter)?;
        let names = self.scopes[target].len()
            + shown
                .iter()
                .map(|(source, _)| self.globs.views[*source].names.len())
                .sum::<usize>();
        self.charge_steps(names, site, meter)?;
        let (content, any_hidden, dropped) =
            self.gather(target, class, shown, hidden, site, meter)?;
        let hiding: Vec<usize> = hidden.iter().map(|(source, _)| *source).collect();
        drop(sources);
        meter.release_globs(listed);
        self.globs.views[view].hidden = any_hidden;
        // A view that does not know that the target's globs bring in what it
        // cannot access makes the target an open reader of each view it
        // reads covertly (see [`Index::covert_told`]).
        if !any_hidden {
            for read in 0..self.globs.reads[target].len() {
                let read_view = self.globs.reads[target][read].view;
                if self.globs.views[read_view]
                    .unseen()
                    .covert
                    .contains_key(&target)
                {
                    self.note_open(read_view, target, site, meter)?;
                }
            }
        }
        for source in hiding {
            self.note_hiding(view, source, site, meter)?;
        }
        for (name, at) in dropped {
            self.note_dropped(view, name, at, site, meter)?;
        }
        for (name, lists) in content {
            for (at, ids) in lists.into_iter().enumerate() {
                self.add_to_view(view, &name, at, &ids, meter)?;
            }
        }
        meter.release_globs(gathered);
        Ok(view)
    }

    /// Turn the readers of the reads of `target` once it has a new view:
    /// a passive read becomes active where a view of the target can access
    /// what its glob brings in and covert where none can (see
    /// [`Index::covert_level`]), and a covert read becomes active where the
    /// new view can, leaving the covert and the open readers of its view.
    /// What a covert reader's entry allocates is charged for the glob
    /// import at `site`.
    fn turn_readers(&mut self, target: usize, site: usize, meter: &Meter) -> Result<(), LoadError> {
        for read in 0..self.globs.reads[target].len() {
            let read_view = self.globs.reads[target][read].view;
            let level = self.level_of(
                &self.uses[self.globs.reads[target][read].glob].vis,
                target,
                Steps::Load,
            );
            let shown = !self.covert_level(target, level);
            let readers = &self.globs.views[read_view];
            let passive = readers
                .passive
                .get(&target)
                .and_then(|list| list.iter().position(|known| *known == read));
            if let Some(at) = passive {
                if !shown {
                    let covert = &readers.unseen().covert;
                    let bytes = covert_bytes(!covert.contains_key(&target), covert.is_empty())
                        + readers.unseen_bytes();
                    self.charge_tables(bytes, site, meter)?;
                }
                let readers = &mut self.globs.views[read_view];
                let list = readers.passive.get_mut(&target).expect("listed");
                list.remove(at);
                if list.is_empty() {
                    readers.passive.remove(&target);
                }
                let turn = readers.turns;
                readers.turns += 1;
                if shown {
                    readers.active.push((target, read, turn));
                } else {
                    readers
                        .unseen_mut()
                        .covert
                        .entry(target)
                        .or_default()
                        .push((read, turn));
                }
            } else if shown
                && let Some(at) = readers
                    .unseen()
                    .covert
                    .get(&target)
                    .and_then(|list| list.iter().position(|known| known.0 == read))
            {
                // The read keeps its turn among the active readers.
                let readers = &mut self.globs.views[read_view];
                let unseen = readers.unseen_mut();
                let list = unseen.covert.get_mut(&target).expect("listed");
                let (_, turn) = list.remove(at);
                if list.is_empty() {
                    unseen.covert.remove(&target);
                    unseen.open.remove(&target);
                }
                let place = readers.active.partition_point(|known| known.2 < turn);
                readers.active.insert(place, (target, read, turn));
            }
        }
        Ok(())
    }

    /// Record `module` among the open covert readers of `view` (see
    /// [`Index::covert_told`]), charging the record for the glob import at
    /// `site` the first time.
    fn note_open(
        &mut self,
        view: usize,
        module: usize,
        site: usize,
        meter: &Meter,
    ) -> Result<(), LoadError> {
        let unseen = &self.globs.views[view];
        let open = &unseen.unseen().open;
        if open.contains(&module) {
            return Ok(());
        }
        let bytes = set_bytes::<usize>(open.is_empty()) + unseen.unseen_bytes();
        self.charge_tables(bytes, site, meter)?;
        self.globs.views[view].unseen_mut().open.insert(module);
        Ok(())
    }

    /// Record that the target of `view` reads `source` through a glob whose
    /// level the class of the view cannot access, charging the record for
    /// the glob import at `site` the first time.
    fn note_hiding(
        &mut self,
        view: usize,
        source: usize,
        site: usize,
        meter: &Meter,
    ) -> Result<(), LoadError> {
        let unseen = &self.globs.views[view];
        let hiding = &unseen.unseen().hiding;
        if hiding.contains(&source) {
            return Ok(());
        }
        let bytes = set_bytes::<usize>(hiding.is_empty()) + unseen.unseen_bytes();
        self.charge_tables(bytes, site, meter)?;
        self.globs.views[view].unseen_mut().hiding.insert(source);
        Ok(())
    }

    /// Record that the globs of the target of `view` brought in under `name`
    /// in the name space at `at` only items the class of the view cannot
    /// access while it did not hold the name, charging the record for the
    /// glob import at `site` the first time.
    fn note_dropped(
        &mut self,
        view: usize,
        name: SharedStr,
        at: usize,
        site: usize,
        meter: &Meter,
    ) -> Result<(), LoadError> {
        let unseen = &self.globs.views[view];
        let dropped = &unseen.unseen().dropped;
        let key = (name, at);
        if dropped.contains(&key) {
            return Ok(());
        }
        let bytes = set_bytes::<(SharedStr, usize)>(dropped.is_empty()) + unseen.unseen_bytes();
        self.charge_tables(bytes, site, meter)?;
        self.globs.views[view].unseen_mut().dropped.insert(key);
        Ok(())
    }

    /// What `target` exports to `class` under each name, as the tables
    /// stand: its own bindings that shadow globs, and what the views
    /// `shown` and `covert` hold, by [`Index::exposed`], and whether any of
    /// the items the globs bring in is one the class cannot access. A name
    /// read again from the globs is charged for the glob import at `site`.
    ///
    /// The views `covert` are read through globs whose level the class
    /// cannot access, so every item they bring in is one it cannot access:
    /// what a glob brings in is no wider than the glob, and a class that
    /// cannot access a level cannot access a narrower one, as the levels it
    /// can access are those of the ancestors its importers lie within. They
    /// are not gathered name by name. Their items matter only under a name the
    /// class reads an item of from `shown` (see [`Index::exposed`]), and are
    /// probed under those names, from the smaller side, one step each; and
    /// whether any is brought in at all is probed name by name until one is
    /// found that neither a binding of the target nor an import keeps from
    /// the globs, a step each, once nothing else has told. Under a name that
    /// no view holds a companion under, every glob read delivers what its
    /// view holds (see [`Index::delivered`]), so the exposure of what was
    /// gathered is what reading the name again from the globs gives; under
    /// any other the name is read again as it is read where a companion or
    /// an item the class cannot access arrives (see [`Index::export_ids`]).
    /// What the probes bring in is charged as a change waiting to be passed
    /// on, and released once gathered.
    fn gather<'s>(
        &'s self,
        target: usize,
        class: &[bool],
        shown: &[(usize, Level)],
        covert: &[(usize, Level)],
        site: usize,
        meter: &Meter,
    ) -> Result<Gathered, LoadError> {
        let mut content: BTreeMap<&'s str, [Vec<Held>; 3]> = BTreeMap::new();
        let mut outside: BTreeMap<(&'s str, usize), Vec<Held>> = BTreeMap::new();
        let mut blocked: BTreeMap<(&'s str, usize), bool> = BTreeMap::new();
        for (name, entry) in &self.scopes[target] {
            for (at, defs) in entry.defs.iter().enumerate() {
                let held = self.defs_held(target, class, defs);
                if !held.is_empty() {
                    content.entry(name.as_str()).or_default()[at] = held;
                }
            }
        }
        for (source, glob) in shown {
            for (name, lists) in &self.globs.views[*source].names {
                for (at, held) in lists.iter().enumerate() {
                    if held.is_empty()
                        || self.binds_itself(target, name, at)
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
        let (any_hidden, probed) =
            self.gather_covert(target, class, covert, &content, &mut outside, site, meter)?;
        let mut gathered: Exported = Vec::with_capacity(content.len());
        let mut dropped: Vec<(SharedStr, usize)> = Vec::new();
        for (name, mut lists) in content {
            for (at, ids) in lists.iter_mut().enumerate() {
                let hidden = outside.remove(&(name, at)).unwrap_or_default();
                let plain = !self.flagged(name, at);
                // Items the class cannot access under a name it reads
                // nothing of are dropped: the name is recorded where no view
                // holds a companion under it (see [`Index::hides_nothing`]).
                if plain && ids.is_empty() && !hidden.is_empty() {
                    let key = self
                        .globs
                        .strings
                        .get(name)
                        .expect("a name a view holds is interned");
                    dropped.push((key.clone(), at));
                }
                // Items the class cannot access, and companions, are read
                // again from the globs, as each delivers them (see
                // [`Index::delivered`]), under a name that a view holds a
                // companion under.
                *ids = if !self.replaced(name, at)
                    && (plain
                        || (hidden.is_empty() && !ids.iter().any(|held| held.1.is_companion())))
                {
                    self.exposed(std::mem::take(ids), hidden, site, meter)?
                } else {
                    self.export_ids(target, class, name, at, site, meter)?
                };
            }
            gathered.push((name.to_owned(), lists));
        }
        meter.release_globs(probed);
        Ok((gathered, any_hidden, dropped))
    }

    /// What the views `covert`, read by globs of `target` whose level
    /// `class` cannot access, add to what [`Index::gather`] gathered from
    /// the other views, `content` and `outside`: under each name and name
    /// space the class reads an item of, the items they hold, which the
    /// class cannot access, probed from the smaller side of the names
    /// gathered and the names of the view, a step each. And whether the
    /// globs of `target` bring in an item the class cannot access: already
    /// so when `outside` holds one, else probed name by name in the views
    /// until a name is found that neither a binding of the target nor an
    /// import keeps from the globs, a step each. Returns that, and the bytes
    /// charged for what the probes added to `outside`, as a change waiting
    /// to be passed on.
    #[expect(
        clippy::too_many_arguments,
        reason = "one gathering: the view's module and class, the views probed, what was gathered and the budget"
    )]
    fn gather_covert<'s>(
        &'s self,
        target: usize,
        class: &[bool],
        covert: &[(usize, Level)],
        content: &BTreeMap<&'s str, [Vec<Held>; 3]>,
        outside: &mut BTreeMap<(&'s str, usize), Vec<Held>>,
        site: usize,
        meter: &Meter,
    ) -> Result<(bool, usize), LoadError> {
        let mut any_hidden = !outside.is_empty();
        let mut probed = 0;
        for (source, glob) in covert {
            let names = &self.globs.views[*source].names;
            if !any_hidden {
                for (name, lists) in names {
                    self.charge_steps(1, site, meter)?;
                    if (0..3).any(|at| {
                        !lists[at].is_empty()
                            && !self.binds_itself(target, name, at)
                            && !self.blocked_in(target, name, at)
                    }) {
                        any_hidden = true;
                        break;
                    }
                }
            }
            self.charge_steps(content.len().min(names.len()), site, meter)?;
            let mut probe = |name: &'s str, lists: &Lists| -> Result<(), LoadError> {
                for (at, held) in lists.iter().enumerate() {
                    if held.is_empty()
                        || content[name][at].is_empty()
                        || self.binds_itself(target, name, at)
                    {
                        continue;
                    }
                    let bytes = btree_entry::<(&str, usize), Vec<Held>>()
                        + vec_first::<Held>()
                        + held.len() * vec_slot::<Held>();
                    self.charge_work(bytes, site, meter)?;
                    probed += bytes;
                    let hidden = outside.entry((name, at)).or_default();
                    let before = hidden.len();
                    self.glob_held(target, class, *glob, held, (&mut Vec::new(), hidden));
                    debug_assert_eq!(
                        hidden.len() - before,
                        held.len(),
                        "a glob the class cannot access brings in only what it cannot access"
                    );
                }
                Ok(())
            };
            if content.len() <= names.len() {
                for name in content.keys() {
                    if let Some(lists) = names.get(*name) {
                        probe(name, lists)?;
                    }
                }
            } else {
                for (name, lists) in names {
                    if content.contains_key(name.as_str()) {
                        probe(name.as_str(), lists)?;
                    }
                }
            }
        }
        Ok((any_hidden, probed))
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
    /// what the view holds is passed on to its module, a step of work for
    /// each name in each name space, unless no view of the module can access
    /// the glob, when the read is covert (see [`Index::read_covert`]). The
    /// views of the module whose class cannot access the glob record the
    /// view it reads. Reading the same view again does nothing.
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
        // The read's place among the module's reads of the view, and the
        // list of those places the first time.
        let mut bytes = READ_BYTES + vec_slot::<usize>();
        if self.globs.reads[module].is_empty() {
            bytes += vec_first::<GlobRead>();
        }
        if !self.globs.views[view].places.contains_key(&module) {
            bytes += btree_entry::<usize, Vec<usize>>() + vec_first::<usize>();
        }
        self.charge_tables(bytes, glob, meter)?;
        let read = self.globs.reads[module].len();
        self.globs.reads[module].push(GlobRead { glob, target, view });
        self.globs.views[view]
            .places
            .entry(module)
            .or_default()
            .push(read);
        self.globs.read_by.insert((glob, view));
        let level = self.level_of(&self.uses[glob].vis, module, Steps::Load);
        // The views of the module whose class cannot access the glob's level
        // learn that the view it reads brings in only what they cannot
        // access (see [`Index::hides_nothing`]).
        if !Self::exact_mode() && !self.exports_by_path(module) {
            for place in 0..self.exports_of(module).len() {
                let own = self.globs.exports[module][place];
                if !self.allowed(module, &self.globs.views[own].class, level) {
                    self.note_hiding(own, view, glob, meter)?;
                }
            }
        }
        // A glob of the module with the same level already reads the view:
        // this one passes on nothing more, so it is neither an active nor a
        // passive reader, and a module that reads one table through many
        // such globs passes on what the table holds once.
        let first = self.globs.passing.insert((module, view, level));
        let passes = first && !self.exports_of(module).is_empty();
        if passes && self.covert_level(module, level) {
            return self.read_covert(module, read, view, glob, waiters, meter);
        }
        if !passes {
            if first {
                self.globs.views[view]
                    .passive
                    .entry(module)
                    .or_default()
                    .push(read);
            }
            let memory = Transient::new(meter);
            let decl = &self.uses[glob];
            for name in waiters.waited_in(module, &memory, &decl.file, decl.line)? {
                if self.globs.views[view].names.contains_key(name.as_str()) {
                    waiters.wake(module, &name)?;
                }
            }
            return Ok(());
        }
        let turn = self.globs.views[view].turns;
        self.globs.views[view].turns += 1;
        self.globs.views[view].active.push((module, read, turn));
        // Each name of the view, one at a time: passing one on can add to
        // this view too, through a cycle of globs, and what it adds is passed
        // on then.
        let mut after: Option<SharedStr> = None;
        loop {
            let names = &self.globs.views[view].names;
            let next = match &after {
                None => names.iter().next(),
                Some(name) => names
                    .range::<SharedStr, _>((Excluded(name), Unbounded))
                    .next(),
            };
            let Some((name, lists)) = next.map(|(name, lists)| (name.clone(), (**lists).clone()))
            else {
                return Ok(());
            };
            for (at, ids) in lists.iter().enumerate() {
                if !ids.is_empty() {
                    self.charge_steps(1, glob, meter)?;
                    let mut work = Queue::new(meter);
                    self.arrive(module, read, &name, at, ids, &mut work, waiters, meter)?;
                    self.pass_on(work, waiters, meter)?;
                }
            }
            after = Some(name);
        }
    }

    /// Record the read at `read` of the globs of `module`, by the glob at
    /// `glob`, of `view` as covert (see [`Index::covert_level`]), and pass on
    /// what it can change. It is listed among the covert readers of the
    /// view before anything is passed on, so an item the view gains through
    /// a cycle of globs while its names arrive reaches the module, as every
    /// later gain of the view does (see [`Index::covert_told`]). What waits
    /// on a name of the view in the module is told, as for a passive read:
    /// a step for each name the module waits on. The views of the module
    /// learn that its globs bring in an item they cannot access once the
    /// view holds a name that the module neither binds nor keeps from the
    /// globs, probed name by name until one tells, a step each, while a view
    /// of the module does not know it yet. And each name that a view of the
    /// module holds and the view holds too, in the same name space, arrives
    /// as an active read would pass it on, a step each (see
    /// [`Index::covert_due`]): under any other name an arrival through the
    /// read finds nothing to add. That is what an active read of the view
    /// would change, without a step for each of the view's names.
    fn read_covert(
        &mut self,
        module: usize,
        read: usize,
        view: usize,
        glob: usize,
        waiters: &mut dyn Waiters,
        meter: &Meter,
    ) -> Result<(), LoadError> {
        let unseen = &self.globs.views[view];
        let covert = &unseen.unseen().covert;
        let bytes =
            covert_bytes(!covert.contains_key(&module), covert.is_empty()) + unseen.unseen_bytes();
        self.charge_tables(bytes, glob, meter)?;
        let readers = &mut self.globs.views[view];
        let turn = readers.turns;
        readers.turns += 1;
        readers
            .unseen_mut()
            .covert
            .entry(module)
            .or_default()
            .push((read, turn));
        let memory = Transient::new(meter);
        let decl = &self.uses[glob];
        let waited = waiters.waited_in(module, &memory, &decl.file, decl.line)?;
        self.covert_steps(waited.len(), glob, meter)?;
        if self
            .exports_of(module)
            .iter()
            .any(|own| !self.globs.views[*own].hidden)
        {
            let mut found = false;
            for (name, lists) in &self.globs.views[view].names {
                self.covert_steps(1, glob, meter)?;
                if (0..3).any(|at| {
                    !lists[at].is_empty()
                        && !self.binds_itself(module, name, at)
                        && !self.blocked_in(module, name, at)
                }) {
                    found = true;
                    break;
                }
            }
            if found {
                for place in 0..self.globs.exports[module].len() {
                    let own = self.globs.exports[module][place];
                    self.globs.views[own].hidden = true;
                }
            }
        }
        if self
            .exports_of(module)
            .iter()
            .any(|own| !self.globs.views[*own].hidden)
        {
            self.note_open(view, module, glob, meter)?;
        }
        // In the order of the view's names, as an active read gives them:
        // what waits on a name in the module is told before the name
        // arrives.
        let mut waits = waited.iter().peekable();
        for (name, at) in self.covert_due(module, view, glob, meter)? {
            while let Some(waiting) = waits.next_if(|waiting| waiting.as_str() <= name.as_str()) {
                if self.globs.views[view].names.contains_key(waiting.as_str()) {
                    waiters.wake(module, waiting)?;
                }
            }
            self.covert_steps(1, glob, meter)?;
            let ids = self.globs.views[view].names[name.as_str()][at].clone();
            let mut work = Queue::new(meter);
            self.arrive(module, read, &name, at, &ids, &mut work, waiters, meter)?;
            self.pass_on(work, waiters, meter)?;
        }
        for waiting in waits {
            if self.globs.views[view].names.contains_key(waiting.as_str()) {
                waiters.wake(module, waiting)?;
            }
        }
        Ok(())
    }

    /// The names, each in a name space, that a view of `module` holds and
    /// `view` holds too, and that the module neither binds nor keeps from
    /// the globs, in order: found from the smaller side, the names of the
    /// module's views or those of `view`, a step for each name looked at,
    /// charged for the glob at `glob`. An arrival through a covert read
    /// under any other name only tells the module's views that its globs
    /// bring in what they cannot access (see [`Index::arrive`]).
    fn covert_due(
        &self,
        module: usize,
        view: usize,
        glob: usize,
        meter: &Meter,
    ) -> Result<Vec<(SharedStr, usize)>, LoadError> {
        let own = self.exports_of(module);
        let source = &self.globs.views[view].names;
        let holds = |names: &BTreeMap<SharedStr, Lists>, name: &str, at: usize| {
            names.get(name).is_some_and(|lists| !lists[at].is_empty())
        };
        let open = |name: &str, at: usize| {
            !self.binds_itself(module, name, at) && !self.blocked_in(module, name, at)
        };
        let held: usize = own
            .iter()
            .map(|own| self.globs.views[*own].names.len())
            .sum();
        let mut due: Vec<(SharedStr, usize)> = Vec::new();
        if held <= source.len() {
            self.covert_steps(held, glob, meter)?;
            for own in own {
                for (name, lists) in &self.globs.views[*own].names {
                    for (at, ids) in lists.iter().enumerate() {
                        if !ids.is_empty() && holds(source, name, at) && open(name, at) {
                            due.push((name.clone(), at));
                        }
                    }
                }
            }
            due.sort_unstable();
            due.dedup();
        } else {
            self.covert_steps(source.len(), glob, meter)?;
            for (name, lists) in source {
                for (at, ids) in lists.iter().enumerate() {
                    if !ids.is_empty()
                        && own
                            .iter()
                            .any(|own| holds(&self.globs.views[*own].names, name, at))
                        && open(name, at)
                    {
                        due.push((name.clone(), at));
                    }
                }
            }
        }
        Ok(due)
    }

    /// Charge `units` steps of the covert reads of the glob tables (see
    /// [`Index::read_covert`]), for the glob import at `site`, and count them.
    fn covert_steps(&self, units: usize, site: usize, meter: &Meter) -> Result<(), LoadError> {
        tally::add(Work::CovertChecks, units);
        self.charge_steps(units, site, meter)
    }

    /// Recompute what `module` exports under `name` to each of its classes,
    /// after its bindings of the name or an import of the name changed, and
    /// pass on what that adds. Where the name is read again from the globs,
    /// neither bound by the module nor kept from its globs, and they deliver
    /// an item under it, the view, name and name space are recorded, charged
    /// as glob bytes, and every later arrival there is read as it is where a
    /// change is not taken as it comes (see [`Index::arrival`]): items the
    /// globs brought in while the module was kept from them did not tell the
    /// view that its class cannot access them. Where they deliver none, none
    /// came in, and what comes in later arrives as usual.
    pub(crate) fn refresh(
        &mut self,
        module: usize,
        name: &str,
        waiters: &mut dyn Waiters,
        meter: &Meter,
    ) -> Result<(), LoadError> {
        for place in 0..self.exports_of(module).len() {
            let view = self.globs.exports[module][place];
            let site = self.globs.views[view].site;
            for at in 0..3 {
                let class = &self.globs.views[view].class;
                let (ids, delivered) = self.exported(module, class, name, at, site, meter)?;
                if delivered {
                    self.mark_exact(view, name, at, site, meter)?;
                }
                let fresh = self.add_to_view(view, name, at, &ids, meter)?;
                if !fresh.is_empty() {
                    let grown = Grown::new(
                        view,
                        self.globs.strings.get(name).expect("interned").clone(),
                        at,
                        fresh,
                    );
                    let mut queue = Queue::new(meter);
                    queue.push(grown, &self.uses[self.globs.views[view].site])?;
                    self.pass_on(queue, waiters, meter)?;
                }
            }
        }
        Ok(())
    }

    /// Record that a reading again after a change to the bindings or imports
    /// of `name` set what `view` holds of it in the name space at `at`,
    /// charging the record, and the name's text the first time the tables
    /// hold it, for the glob import at `site`.
    fn mark_exact(
        &mut self,
        view: usize,
        name: &str,
        at: usize,
        site: usize,
        meter: &Meter,
    ) -> Result<(), LoadError> {
        let key = if let Some(key) = self.globs.strings.get(name) {
            if self.globs.exact.contains(&(view, key.clone(), at)) {
                return Ok(());
            }
            key.clone()
        } else {
            self.charge_tables(
                shared_text(name.len()) + hash_entry::<SharedStr>(),
                site,
                meter,
            )?;
            let key = SharedStr::from(name);
            self.globs.strings.insert(key.clone());
            key
        };
        self.charge_tables(
            set_bytes::<(usize, SharedStr, usize)>(self.globs.exact.is_empty()),
            site,
            meter,
        )?;
        self.globs.exact.insert((view, key, at));
        Ok(())
    }

    /// Pass on every change in `work`, and every change that passing one on
    /// makes, until none is left, charging what the list holds while it holds
    /// it, to the glob import at `site` for the changes it starts with and to
    /// the glob that passes each change on for those it makes. Each reader a
    /// change is passed on to is a step of work, charged to the reader's glob
    /// before the change arrives.
    fn pass_on<'m>(
        &mut self,
        mut queue: Queue<'m>,
        waiters: &mut dyn Waiters,
        meter: &'m Meter,
    ) -> Result<(), LoadError> {
        while let Some(grown) = queue.work.pop() {
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
                    waiters.wake(module, &grown.name)?;
                }
            }
            // Passing on adds names to views, never readers, so the lists of
            // readers stay as they are meanwhile. The covert readers the gain
            // can change, and those where something waits on the name, take
            // their turns among the active readers (see
            // [`Index::covert_told`]).
            let told = if self.globs.views[grown.view].unseen().covert.is_empty() {
                Vec::new()
            } else {
                self.covert_told(&grown, &*waiters, meter)?
            };
            let mut next = 0;
            for reader in 0..self.globs.views[grown.view].active.len() {
                let (module, read, turn) = self.globs.views[grown.view].active[reader];
                while let Some(&covert) = told.get(next).filter(|covert| covert.0 < turn) {
                    self.tell_covert(&grown, covert, &mut queue, waiters, meter)?;
                    next += 1;
                }
                let glob = self.globs.reads[module][read].glob;
                // Each reader the change is passed on to is a step of work.
                self.charge_steps(1, glob, meter)?;
                self.arrive(
                    module,
                    read,
                    &grown.name,
                    grown.at,
                    &grown.ids,
                    &mut queue,
                    waiters,
                    meter,
                )?;
            }
            for covert in &told[next..] {
                self.tell_covert(&grown, *covert, &mut queue, waiters, meter)?;
            }
        }
        Ok(())
    }

    /// The covert readers of the view of `grown` to tell of the gain (see
    /// [`Index::read_covert`]), in the order of their turns: each the gain
    /// can change arrives through its first covert read of the view (see
    /// [`Index::covert_changed`]), and of the others, each where something
    /// waits on the name is told so, found from the smaller side of the
    /// modules that wait on it and the covert readers, a step for each
    /// looked at, charged to the glob the view was made for. The open
    /// readers whose views all know now what they cannot access leave them.
    /// Told in their turns among the active readers, they are told in the
    /// order an active read of each would have been, so the imports that
    /// wait on the name are queued again in that order.
    fn covert_told(
        &mut self,
        grown: &Grown,
        waiters: &dyn Waiters,
        meter: &Meter,
    ) -> Result<Vec<Told>, LoadError> {
        let site = self.globs.views[grown.view].site;
        let (due, closed) = self.covert_changed(grown, site, meter)?;
        if let Some(unseen) = self.globs.views[grown.view].unseen.as_mut() {
            for module in closed {
                unseen.open.remove(&module);
            }
        }
        let readers = &self.globs.views[grown.view].unseen().covert;
        let mut told: Vec<Told> = due
            .iter()
            .map(|&(module, (read, turn))| (turn, module, read, true))
            .collect();
        if let Some(modules) = waiters.waiting_on(&grown.name) {
            self.covert_steps(modules.len().min(readers.len()), site, meter)?;
            let waiting: Vec<usize> = if modules.len() <= readers.len() {
                modules
                    .iter()
                    .copied()
                    .filter(|module| readers.contains_key(module))
                    .collect()
            } else {
                readers
                    .keys()
                    .copied()
                    .filter(|module| modules.contains(module))
                    .collect()
            };
            for module in waiting {
                if due.binary_search_by_key(&module, |entry| entry.0).is_err() {
                    let (read, turn) = readers[&module][0];
                    told.push((turn, module, read, false));
                }
            }
        }
        told.sort_unstable();
        Ok(told)
    }

    /// Tell the covert reader `told` of the gain `grown` (see
    /// [`Index::covert_told`]): the gain arrives through its read, queueing
    /// what that adds in `queue`, charged as [`Index::pass_on`] charges what
    /// an active reader adds; or what waits on the name in its module is
    /// told.
    fn tell_covert<'m>(
        &mut self,
        grown: &Grown,
        told: Told,
        queue: &mut Queue<'m>,
        waiters: &mut dyn Waiters,
        meter: &'m Meter,
    ) -> Result<(), LoadError> {
        let (_, module, read, arrives) = told;
        if !arrives {
            waiters.wake(module, &grown.name)?;
            return Ok(());
        }

        self.arrive(
            module,
            read,
            &grown.name,
            grown.at,
            &grown.ids,
            queue,
            waiters,
            meter,
        )
    }

    /// The covert readers of the view of `grown` that the gain can change,
    /// by module, each with its first covert read of the view and its turn,
    /// and the open readers whose views all know now that their globs bring
    /// in what they cannot access (see [`Index::covert_told`]). Only two
    /// kinds of covert reader can be changed: a module with a view that does
    /// not yet know what it cannot access, among the view's open readers,
    /// and a module with a view that holds the name, a target of the views
    /// that hold the name, found from the smaller side, those views or the
    /// covert readers. Each module found is looked at, and is changed unless
    /// the gain cannot change anything there (see [`Index::covert_inert`]);
    /// each view that holds the name and open reader looked at to find them
    /// is a step too, charged for the glob import at `site`.
    fn covert_changed(
        &self,
        grown: &Grown,
        site: usize,
        meter: &Meter,
    ) -> Result<Changed, LoadError> {
        let flagged = self.flagged(&grown.name, grown.at);
        let readers = self.globs.views[grown.view].unseen();
        let mut found: Vec<usize> = Vec::new();
        // The view that grew holds the name, so the views that hold it are
        // fewer than the covert readers only where those are two or more.
        let holders = if readers.covert.len() > 1 {
            self.globs
                .holding
                .get(grown.name.as_str())
                .map(Vec::as_slice)
                .filter(|holders| holders.len() < readers.covert.len())
        } else {
            None
        };
        if let Some(holders) = holders {
            self.covert_steps(holders.len(), site, meter)?;
            found.extend(
                holders
                    .iter()
                    .map(|view| self.globs.views[*view].target)
                    .filter(|module| readers.covert.contains_key(module)),
            );
        }
        self.covert_steps(readers.open.len(), site, meter)?;
        let mut closed: Vec<usize> = Vec::new();
        for module in &readers.open {
            if self
                .exports_of(*module)
                .iter()
                .any(|own| !self.globs.views[*own].hidden)
            {
                found.push(*module);
            } else {
                closed.push(*module);
            }
        }
        let mut due: Vec<(usize, (usize, usize))> = Vec::new();
        if holders.is_some() || !found.is_empty() {
            if holders.is_none() {
                found.extend(readers.covert.keys().copied());
            }
            found.sort_unstable();
            found.dedup();
            for module in found {
                self.covert_steps(1, site, meter)?;
                if !self.covert_inert(module, grown, flagged) {
                    due.push((module, readers.covert[&module][0]));
                }
            }
        } else {
            for (module, reads) in &readers.covert {
                self.covert_steps(1, site, meter)?;
                if !self.covert_inert(*module, grown, flagged) {
                    due.push((*module, reads[0]));
                }
            }
        }
        Ok((due, closed))
    }

    /// True when `grown` arriving at `module` through a covert read can
    /// change nothing: every view of the module already knows that its globs
    /// bring in what it cannot access, and either does not hold the name in
    /// the name space, so an item it cannot access has nothing to stand
    /// beside, or holds every item that arrives, under a name that no view
    /// holds a companion under and that a reading again did not set (see
    /// [`Index::arrive`]): an item the view cannot access and already holds
    /// adds no companion.
    fn covert_inert(&self, module: usize, grown: &Grown, flagged: bool) -> bool {
        self.exports_of(module).iter().all(|own| {
            let view = &self.globs.views[*own];
            view.hidden
                && view
                    .names
                    .get(grown.name.as_str())
                    .map(|lists| &lists[grown.at])
                    .filter(|held| !held.is_empty())
                    .is_none_or(|held| {
                        !flagged
                            && !self.is_exact(*own, &grown.name, grown.at)
                            && grown.ids.iter().all(|(id, _)| {
                                held.binary_search_by_key(id, |known| known.0).is_ok()
                            })
                    })
        })
    }

    /// True when a reading again after a change to the bindings or imports
    /// of `name` set what `view` holds of it in the name space at `at` from
    /// the globs (see [`Index::refresh`]).
    fn is_exact(&self, view: usize, name: &SharedStr, at: usize) -> bool {
        !self.globs.exact.is_empty() && self.globs.exact.contains(&(view, name.clone(), at))
    }

    /// Pass on what the read at `read` of the globs of `module` brought in,
    /// the items `ids` of `name` in the name space at `at`: tell what waits
    /// on the name there, and add the items to each view of the module that
    /// the glob reaches where nothing shadows the globs, queueing what that
    /// adds in `work`. A view takes them as the change they bring where it
    /// holds the name, under a name that no view holds a companion under
    /// (see [`Index::delta`]), and otherwise as [`Index::arrival`] tells.
    /// Reading the module's table again for the name is charged as work for
    /// each of its glob reads (see [`Index::export_ids`]).
    #[expect(
        clippy::too_many_arguments,
        reason = "one change passed on, the place it arrives at and the two sinks"
    )]
    fn arrive<'m>(
        &mut self,
        module: usize,
        read: usize,
        name: &SharedStr,
        at: usize,
        ids: &[Held],
        work: &mut Queue<'m>,
        waiters: &mut dyn Waiters,
        meter: &'m Meter,
    ) -> Result<(), LoadError> {
        waiters.wake(module, name)?;
        if self.binds_itself(module, name, at) {
            return Ok(());
        }
        let glob = self.globs.reads[module][read].glob;
        let level = self.level_of(&self.uses[glob].vis, module, Steps::Load);
        if self.blocked_in(module, name, at) {
            // The explicit import may settle in another namespace. The
            // arriving glob facts then become visible, even though no
            // delta was delivered here: a later arrival must read those
            // facts too, rather than assume this view consumed them.
            for place in 0..self.exports_of(module).len() {
                let view = self.globs.exports[module][place];
                self.mark_exact(view, name, at, glob, meter)?;
            }
            return Ok(());
        }
        // Under a name that no view holds a companion under, in a module
        // whose levels do not change as the tables settle, a view that holds
        // the name takes the change as it comes (see [`Index::delta`]).
        let plain = !self.flagged(name, at) && !self.exports_by_path(module);
        for place in 0..self.globs.exports[module].len() {
            let view = self.globs.exports[module][place];
            let exact = self.is_exact(view, name, at);
            let own = &self.globs.views[view];
            let held = own
                .names
                .get(name.as_str())
                .map(|lists| lists[at].as_slice())
                .filter(|held| !held.is_empty());
            if let Some(held) = held
                && !self.exports_by_path(module)
                && !exact
            {
                self.charge_steps(ids.len(), glob, meter)?;
                tally::add(Work::CandidateSteps, ids.len());
                if let Some(any_hidden) = self.unchanged(module, &own.class, level, held, ids) {
                    tally::add(Work::Deltas, 1);
                    if any_hidden {
                        self.globs.views[view].hidden = true;
                    }
                    continue;
                }
            }
            let passed = match held {
                Some(_) if plain && !exact => {
                    self.delta(module, view, name, at, level, ids, glob, meter)?
                }
                held => {
                    let holds = held.is_some();
                    self.arrival(
                        module, view, name, at, level, ids, holds, plain, exact, glob, meter,
                    )?
                }
            };
            let Some(passed) = passed else {
                continue;
            };
            let fresh = self.add_to_view(view, name, at, &passed, meter)?;
            if !fresh.is_empty() {
                work.push(Grown::new(view, name.clone(), at, fresh), &self.uses[glob])?;
            }
        }
        Ok(())
    }

    /// Whether the items `ids` that a glob at `level` of `module` brings in
    /// change nothing in a view of the module for `class` that holds `held`
    /// under their name: each is held at least as widely, and a visible
    /// plain arrival is already held plain. A companion cannot turn a plain
    /// entry back into a companion. Tested without
    /// a copy; `Some` with whether one of them is an item the class cannot
    /// access when they change nothing, `None` when they change something.
    fn unchanged(
        &self,
        module: usize,
        class: &[bool],
        level: Level,
        held: &[Held],
        ids: &[Held],
    ) -> Option<bool> {
        let mut any_hidden = false;
        for (id, item) in ids {
            let met = self.meet(level, *item).plain();
            let shown = self.allowed(module, class, met);
            any_hidden |= !shown;
            let kept = held
                .binary_search_by_key(id, |known| known.0)
                .is_ok_and(|place| {
                    self.width_order(held[place].1, met).is_le()
                        && (!shown || item.is_companion() || !held[place].1.is_companion())
                });
            if !kept {
                return None;
            }
        }
        Some(any_hidden)
    }

    /// What `view` of `module`, which holds `name` in the name space at
    /// `at`, a name that no view holds a companion under, takes of the items
    /// `ids` that a glob at `level` of the module brings in, which change
    /// something there (see [`Index::unchanged`]). Such a view holds
    /// everything the globs of the module brought in under the name that its
    /// class can access and nothing they brought in that it cannot, which
    /// would have added a companion: the view took the name from a gathering
    /// or a reading again of the globs, which expose every item, or from
    /// what arrived while no item it cannot access had (see
    /// [`Index::arrival`]), and a reading again after a change to the
    /// module's bindings or imports of the name keeps it out of this path.
    /// So an item it can access is added as it comes, and one it cannot
    /// access and does not hold makes the name ambiguous, exposed beside
    /// what the view holds, a step for each item held and each new one,
    /// charged for the glob at `glob`; either way the view learns whether
    /// an item it cannot access arrived.
    #[expect(
        clippy::too_many_arguments,
        reason = "one change arriving at one view, the glob it came through and the budget"
    )]
    fn delta(
        &mut self,
        module: usize,
        view: usize,
        name: &SharedStr,
        at: usize,
        level: Level,
        ids: &[Held],
        glob: usize,
        meter: &Meter,
    ) -> Result<Option<Vec<Held>>, LoadError> {
        let own = &self.globs.views[view];
        let held = &own.names[name.as_str()][at];
        let (mut seen, mut hidden) = (Vec::new(), Vec::new());
        self.glob_held(module, &own.class, level, ids, (&mut seen, &mut hidden));
        let newly: Vec<Held> = hidden
            .iter()
            .copied()
            .filter(|(id, _)| held.binary_search_by_key(id, |known| known.0).is_err())
            .collect();
        let passed = if newly.is_empty() {
            self.widest(&mut seen, glob, meter)?;
            seen
        } else {
            // Another read may already hold a visible route whose gain
            // has not reached this view yet. Exposing beside `held` would
            // manufacture a companion at a wider level than that route.
            tally::add(Work::Exposures, newly.len());
            self.charge_steps(newly.len(), glob, meter)?;
            self.export_ids(module, &own.class, name, at, glob, meter)?
        };
        if !hidden.is_empty() {
            self.globs.views[view].hidden = true;
        }
        Ok(Some(passed))
    }

    /// What `view` of `module`, which `holds` the name or not, takes of the
    /// items `ids` that a glob at `level` of the module brings in under
    /// `name` in the name space at `at`, where it does not take them as they
    /// come (see [`Index::delta`]), or `None` when it takes nothing; `plain`
    /// when no view holds a companion under the name and the module's
    /// levels do not change as the tables settle. Items the class can
    /// access arrive under a name the view does not hold: once the globs of
    /// the module have brought in an item it cannot access, the view takes
    /// them as they come where it can tell that no such item stands under
    /// the name (see [`Index::hides_nothing`]). Items the class can access
    /// arrive otherwise:
    /// where the globs of the module bring in items it cannot access, those
    /// join them, whichever glob they came through, so the table of the
    /// module is read again for the name, and so it is where a companion
    /// arrives, as another glob of the module may deliver the item it was
    /// carried with, which drops it (see [`Index::delivered`]). Where only
    /// items the class cannot access arrive and it already reads the name,
    /// that is now ambiguous with them unless the read they arrive through
    /// drops them as companions, so the table is read again too; where it
    /// does not read the name they have nothing to stand beside. Either way
    /// the view learns whether an item it cannot access arrived. A reading
    /// again is charged for the glob at `glob`.
    #[expect(
        clippy::too_many_arguments,
        reason = "one change arriving at one view, the glob it came through and the budget"
    )]
    fn arrival(
        &mut self,
        module: usize,
        view: usize,
        name: &SharedStr,
        at: usize,
        level: Level,
        ids: &[Held],
        holds: bool,
        plain: bool,
        exact: bool,
        glob: usize,
        meter: &Meter,
    ) -> Result<Option<Vec<Held>>, LoadError> {
        if exact || self.replaced(name, at) {
            let class = &self.globs.views[view].class;
            return self
                .export_ids(module, class, name, at, glob, meter)
                .map(Some);
        }
        let own = &self.globs.views[view];
        let class: &[bool] = &own.class;
        let (mut seen, mut hidden) = (Vec::new(), Vec::new());
        self.glob_held(module, class, level, ids, (&mut seen, &mut hidden));
        let fresh = plain && !holds;
        let (passed, hide) = if !seen.is_empty() {
            if own.hidden
                && hidden.is_empty()
                && fresh
                && self.hides_nothing(module, view, name, at, glob, meter)?
            {
                self.widest(&mut seen, glob, meter)?;
                (Some(seen), false)
            } else if own.hidden || !hidden.is_empty() {
                (
                    Some(self.export_ids(module, class, name, at, glob, meter)?),
                    true,
                )
            } else if seen.iter().any(|held| held.1.is_companion()) {
                (
                    Some(self.export_ids(module, class, name, at, glob, meter)?),
                    false,
                )
            } else {
                self.widest(&mut seen, glob, meter)?;
                (Some(seen), false)
            }
        } else if !hidden.is_empty() && holds {
            (
                Some(self.export_ids(module, class, name, at, glob, meter)?),
                true,
            )
        } else {
            if fresh && !hidden.is_empty() {
                let site = self.globs.views[view].site;
                self.note_dropped(view, name.clone(), at, site, meter)?;
            }
            (None, !hidden.is_empty())
        };
        if hide {
            self.globs.views[view].hidden = true;
        }
        Ok(passed)
    }

    /// True when no item that the class of `view` cannot access stands
    /// under `name` in the name space at `at` among what the globs of
    /// `module` have brought in, a name the view does not hold and that no
    /// view holds a companion under: none has been dropped there (see
    /// [`Unseen::dropped`]), and no view the module reads through a glob whose
    /// level the class cannot access holds the name there. Items that the
    /// globs bring in later arrive, and are passed on, as the view holds the
    /// name then (see [`Index::delta`]). A step for the record and each such
    /// view, charged for the glob at `glob`; false without a step where they
    /// are as many as the reads that a reading again of the name would look
    /// at (see [`Index::reads_holding`]).
    fn hides_nothing(
        &self,
        module: usize,
        view: usize,
        name: &SharedStr,
        at: usize,
        glob: usize,
        meter: &Meter,
    ) -> Result<bool, LoadError> {
        let own = self.globs.views[view].unseen();
        let again = self
            .globs
            .holding
            .get(name.as_str())
            .map_or(0, Vec::len)
            .min(self.reads_of(module).len());
        if own.hiding.len() >= again {
            return Ok(false);
        }
        let steps = 1 + own.hiding.len();
        tally::add(Work::HiddenChecks, steps);
        self.charge_steps(steps, glob, meter)?;
        Ok(!own.dropped.contains(&(name.clone(), at))
            && own.hiding.iter().all(|source| {
                self.globs.views[*source]
                    .names
                    .get(name.as_str())
                    .is_none_or(|lists| lists[at].is_empty())
            }))
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
        let memory = Transient::new(meter);
        let steps = Steps::LoadAt {
            memory: &memory,
            file: &decl.file,
            line: decl.line,
        };
        let mut work = WorkList::<_, 8>::new();
        push_work(&mut work, module, steps)?;
        while let Some(next) = work.pop() {
            if !std::mem::replace(&mut self.import_reach.reaches[next], true) {
                for &reader in &self.import_reach.globbed_by[next] {
                    push_work(&mut work, reader, steps)?;
                }
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

/// Number entry/exit spans in the already admitted dominator-tree scratch.
fn holder_spans(
    dominator: &[usize],
    step: &mut dyn FnMut(usize) -> Result<(), LoadError>,
    storage: Steps<'_>,
) -> Result<Vec<(usize, usize)>, LoadError> {
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); dominator.len()];
    for (node, parent) in dominator.iter().enumerate().skip(1) {
        if *parent != NONE {
            children[*parent].push(node);
        }
    }
    let mut span = vec![(0, 0); dominator.len()];
    let mut position = 0;
    let mut walk = WorkList::<_, 8>::new();
    push_work(&mut walk, (0, 0), storage)?;
    while let Some((node, child)) = walk.pop() {
        if child == 0 {
            span[node].0 = position;
            position += 1;
        }
        if let Some(&next) = children[node].get(child) {
            step(1)?;
            push_work(&mut walk, (node, child + 1), storage)?;
            push_work(&mut walk, (next, 0), storage)?;
        } else {
            span[node].1 = position;
            position += 1;
        }
    }
    Ok(span)
}

/// The immediate dominator of each node of the graph `successors`, whose
/// root is node 0 and whose every node the root reaches. Its predecessor
/// lists are built alongside its successors, avoiding another edge scan.
/// Uses the algorithm of
/// Lengauer and Tarjan with path compression; [`NONE`] for the root. Each
/// edge looked at and each node numbered is counted to `step` before.
fn dominators(
    successors: &[Vec<usize>],
    predecessors: &[Vec<usize>],
    step: &mut dyn FnMut(usize) -> Result<(), LoadError>,
    storage: Steps<'_>,
) -> Result<Vec<usize>, LoadError> {
    let count = successors.len();
    // Number the nodes in a depth-first walk from the root.
    let mut number = vec![NONE; count];
    let mut vertex: Vec<usize> = Vec::with_capacity(count);
    let mut parent = vec![NONE; count];
    step(1)?;
    let mut walk = WorkList::<_, 8>::new();
    push_work(&mut walk, (0, NONE), storage)?;
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
                step(1)?;
                push_work(&mut walk, (to, node), storage)?;
            }
        }
    }
    let mut semi: Vec<usize> = number.clone();
    let mut ancestor = vec![NONE; count];
    let mut label: Vec<usize> = (0..count).collect();
    let mut dominator = vec![NONE; count];
    let mut bucket: Vec<Vec<usize>> = vec![Vec::new(); count];
    let mut path = WorkList::<_, 8>::new();
    let mut eval = |node: usize,
                    ancestor: &mut Vec<usize>,
                    label: &mut Vec<usize>,
                    semi: &[usize],
                    step: &mut dyn FnMut(usize) -> Result<(), LoadError>|
     -> Result<usize, LoadError> {
        step(1)?;
        if ancestor[node] == NONE {
            return Ok(node);
        }
        let mut at = node;
        while ancestor[ancestor[at]] != NONE {
            step(1)?;
            push_work(&mut path, at, storage)?;
            at = ancestor[at];
        }
        while !path.is_empty() {
            step(1)?;
            let at = path.pop().expect("the compression path is nonempty");
            let up = ancestor[at];
            if semi[label[up]] < semi[label[at]] {
                label[at] = label[up];
            }
            ancestor[at] = ancestor[up];
        }
        Ok(label[node])
    };
    for &node in vertex.iter().skip(1).rev() {
        for &from in &predecessors[node] {
            step(1)?;
            if number[from] == NONE {
                continue;
            }
            let least = eval(from, &mut ancestor, &mut label, &semi, step)?;
            if semi[least] < semi[node] {
                semi[node] = semi[least];
            }
        }
        step(1)?;
        bucket[vertex[semi[node]]].push(node);
        let up = parent[node];
        ancestor[node] = up;
        for waiting in std::mem::take(&mut bucket[up]) {
            step(1)?;
            let least = eval(waiting, &mut ancestor, &mut label, &semi, step)?;
            dominator[waiting] = if semi[least] < semi[waiting] {
                least
            } else {
                up
            };
        }
    }
    for &node in vertex.iter().skip(1) {
        step(1)?;
        if dominator[node] != vertex[semi[node]] {
            dominator[node] = dominator[dominator[node]];
        }
    }
    Ok(dominator)
}

#[cfg(test)]
#[path = "../tests/support/shapes/globs.rs"]
#[allow(
    dead_code,
    reason = "the unit tests build the shapes; the examples also resolve their paths"
)]
mod shapes;

#[cfg(test)]
mod equivalence;

#[cfg(test)]
mod incremental;

#[cfg(test)]
mod lookups;

#[cfg(test)]
mod delivery;

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

    #[test]
    fn holder_visibility_uses_ancestry_instead_of_sibling_depth() {
        let tree: MemoryTree = [
            ("Cargo.toml", DEMO),
            ("src/lib.rs", "pub mod a { pub mod child {} } pub mod b {}"),
        ]
        .into_iter()
        .collect();
        let ws = Workspace::load(&tree, &["."]).expect("loads");
        let index = ws.index();
        let module = |path: &str| {
            index
                .modules
                .iter()
                .position(|module| module.path == path)
                .expect("module")
        };
        let a = super::Level::within(module("demo::a"));
        let child = super::Level::within(module("demo::a::child"));
        let b = super::Level::within(module("demo::b"));
        let covers = |offered, required| {
            index
                .level_covers(offered, required, &mut |_| Ok(()))
                .expect("unlimited work")
        };
        assert!(covers(a, child));
        assert!(!covers(child, a));
        assert!(!covers(a, b));
        assert!(!covers(b, child));
        assert!(covers(super::Level::PUBLIC, child));
        assert!(!covers(child, super::Level::PUBLIC));
        assert!(!covers(super::Level::NOBODY, child));
        assert!(!covers(a, super::Level::NOBODY));
        assert!(covers(a.companion(), child));
    }

    #[test]
    fn named_supplier_levels_stay_with_the_declaration_that_provides_them() {
        let tree: MemoryTree = [
            ("Cargo.toml", DEMO),
            (
                "src/lib.rs",
                "pub mod d { pub struct B {} }\n\
                 pub mod a { pub use crate::d::B; }\n\
                 pub mod b { pub use crate::d::B; }\n\
                 pub mod c {\n\
                   #[cfg(feature = \"first\")] use crate::a::B;\n\
                   #[cfg(not(feature = \"first\"))] pub use crate::b::B;\n\
                 }\n\
                 pub mod g { pub use crate::d::*; }\n",
            ),
        ]
        .into_iter()
        .collect();
        let ws = Workspace::load(&tree, &["."]).expect("loads both configuration alternatives");
        let index = ws.index();
        let module = |path: &str| {
            index
                .modules
                .iter()
                .position(|module| module.path == path)
                .expect("module")
        };
        let id = index
            .items
            .iter()
            .find(|item| item.path == "demo::d::B")
            .expect("defined item")
            .id;
        let meter = crate::meter::Meter::new(&crate::LoadOptions::default());
        let supply = index
            .supply(&meter, index.glob_imports[0])
            .expect("admitted routes");
        let c = module("demo::c");
        for (source, expected) in [
            ("demo::a", super::Level::within(c)),
            ("demo::b", super::Level::PUBLIC),
        ] {
            let routes = &supply.named[&(
                id,
                (
                    module(source),
                    "B".into(),
                    crate::settle::slot(Namespace::Type),
                ),
            )];
            assert_eq!(
                routes,
                &[(
                    (c, "B".into(), crate::settle::slot(Namespace::Type)),
                    expected
                )]
            );
        }
    }

    #[test]
    fn native_resumed_growth_admits_overlapping_buffers_before_mutation() {
        use crate::meter::{Meter, Steps, Transient};
        use crate::{Budget, LoadErrorKind, LoadOptions, RelPath};

        let file = RelPath::new("src/imports.rs").expect("path");
        let run = |limit| {
            let meter = Meter::new(&LoadOptions {
                max_glob_bytes: limit,
                ..LoadOptions::default()
            });
            let memory = Transient::new(&meter);
            memory.charge(7, &file, Some(2)).expect("unrelated payload");
            let steps = Steps::LoadAt {
                memory: &memory,
                file: &file,
                line: 13,
            };
            let mut values: Vec<usize> = Vec::new();
            let mut refused = false;
            for value in 0..5 {
                match super::reserve_next(&mut values, steps) {
                    Ok(()) => values.push(value),
                    Err(error) => {
                        assert_eq!(values, [0, 1, 2, 3]);
                        assert_eq!(
                            (error.file.as_str(), error.line),
                            ("src/imports.rs", Some(13))
                        );
                        assert!(matches!(error.kind, LoadErrorKind::Budget {
                            budget: Budget::GlobBytes, limit: found,
                        } if found == limit));
                        refused = true;
                        break;
                    }
                }
            }
            if !refused {
                assert_eq!(values, [0, 1, 2, 3, 4]);
            }
            (refused, meter.usage().glob_bytes)
        };
        let (refused, peak) = run(usize::MAX);
        assert!(!refused);
        assert!(!run(peak).0);
        assert!(run(peak - 1).0);
    }

    /// The most the search may count while one crate loads: a search that
    /// runs once for each reader over the whole chain passes it at once.
    const CEILING: usize = 400 * UNIT;

    const DEMO: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";

    /// The two genuinely defined candidate identities.
    const BASE: &str = "pub mod d { pub struct X {} }\npub mod p { pub struct X {} }\n";

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
                "pub mod a{i} {{ pub use crate::r{i}::*; use crate::p::*; }}"
            )
            .expect("write");
            writeln!(
                lib,
                "pub mod r{i} {{ pub use crate::a{i}::*; pub use crate::c{}::*; {} }}",
                n - 1,
                more(i)
            )
            .expect("write");
        }
        lib
    }

    #[test]
    fn independent_same_item_supply_preserves_the_carriers_ambiguity() {
        // Unlike a returning carrier, a's genuine d route bypasses r0.
        let lib = format!(
            "{BASE}pub mod a {{ pub use crate::d::*; use crate::p::*; }}\n\
             pub mod c {{ pub use crate::d::*; }}\n\
             pub mod r0 {{ pub use crate::a::*; pub use crate::c::*; }}\n"
        );
        let tree: MemoryTree = [("Cargo.toml", DEMO), ("src/lib.rs", lib.as_str())]
            .into_iter()
            .collect();
        let ws = Workspace::load(&tree, &["."]).expect("loads");
        let Err(crate::ResolveError::Ambiguous(ids)) =
            ws.resolve_from_in("demo::r0", "self::X", Namespace::Type)
        else {
            panic!("the independently supplied carrier must remain ambiguous");
        };
        let mut paths: Vec<_> = ids
            .iter()
            .map(|id| ws.index().items[id.0].path.as_str())
            .collect();
        paths.sort_unstable();
        assert_eq!(paths, ["demo::d::X", "demo::p::X"]);
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
    /// as from `demo::r0` in the type name space. Each step of the search is
    /// a step of glob table work too, so the work of the load counts at least
    /// the steps of the search.
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
        let steps = tally.get(Work::SupplySteps);
        assert!(
            ws.usage().work >= steps,
            "{} steps of work for {steps} of the search",
            ws.usage().work
        );
        (steps, found)
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
            "pub mod a { pub use crate::d::*; use crate::p::*; }\n\
             pub mod e { pub struct X {} }\npub mod e1 { pub use crate::e::*; }\n\
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
        let mut work = purrdf_lex::walk::WorkList::<_, 8>::new();
        super::push_work(&mut work, 0, super::Steps::Load).expect("test stack");
        while let Some(at) = work.pop() {
            if Some(at) == left_out || seen[at] {
                continue;
            }
            seen[at] = true;
            for &next in &successors[at] {
                super::push_work(&mut work, next, super::Steps::Load).expect("test stack");
            }
        }
        seen[node]
    }

    /// Check the dominators of `successors` against their definition: a
    /// node the root reaches is dominated by another exactly when leaving
    /// that one out leaves it unreached, and by itself and the root.
    fn check_dominators(successors: &[Vec<usize>]) -> Vec<usize> {
        let mut predecessors = vec![Vec::new(); successors.len()];
        for (node, next) in successors.iter().enumerate() {
            for &to in next {
                predecessors[to].push(node);
            }
        }
        let dominator = super::dominators(
            successors,
            &predecessors,
            &mut |_| Ok(()),
            super::Steps::Load,
        )
        .unwrap_or_else(|e| panic!("{e}"));
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

#[cfg(test)]
mod work_budget {
    //! Import work includes table reads, candidates, queues, subscriptions
    //! and graph traversal. Operation laws, actual growth and exact refusal
    //! boundaries judge those populations without timing a load.

    use std::fmt::Write as _;

    use super::shapes::{
        MANIFEST as DEMO, ambiguous_hub, chain, companion_fan_in, holder_scan, hub, lookups,
        many_items, prelude, private_fan_in, pub_fan_in,
    };
    use crate::meter::charges;
    use crate::{Budget, LoadError, LoadErrorKind, LoadOptions, MemoryTree, Workspace};

    fn load(lib: &str, max_work: usize) -> Result<Workspace, LoadError> {
        let tree: MemoryTree = [("Cargo.toml", DEMO), ("src/lib.rs", lib)]
            .into_iter()
            .collect();
        let options = LoadOptions {
            max_work,
            ..LoadOptions::default()
        };
        Workspace::load_with(&tree, &["."], &options)
    }

    /// The glob table work of loading `lib` at the default limit.
    fn work(lib: &str) -> usize {
        load(lib, Budget::Work.limit())
            .unwrap_or_else(|error| panic!("loads: {error}"))
            .usage()
            .work
    }

    #[test]
    fn an_export_read_charges_each_visited_read_and_holder_before_returning_its_answer() {
        use crate::meter::Meter;
        use crate::work::{Work, measure};

        // One read takes the all-read branch. Unrelated reads make the
        // holder index smaller than the read list in the second fixture.
        for unrelated in [0, 8] {
            let mut lib = String::from("pub mod d { pub struct X {} }\n");
            for i in 0..unrelated {
                writeln!(lib, "pub mod e{i} {{ pub struct Y{i} {{}} }}").expect("write");
            }
            lib.push_str("pub mod t { pub use crate::d::*;\n");
            for i in 0..unrelated {
                writeln!(lib, "pub use crate::e{i}::*;").expect("write");
            }
            lib.push_str("}\n");
            let workspace = load(&lib, Budget::Work.limit()).expect("loads");
            let index = workspace.index();
            let target = index
                .modules
                .iter()
                .position(|m| m.path == "demo::t")
                .expect("target");
            let root = index
                .modules
                .iter()
                .position(|m| m.path == "demo")
                .expect("root");
            let class = index.class_for(target, root);
            let site = index.reads_of(target)[0].glob;
            let read = |limit| {
                let meter = Meter::new(&LoadOptions {
                    max_work: limit,
                    ..LoadOptions::default()
                });
                let (answer, tally) = measure(usize::MAX, || {
                    index.export_ids(target, &class, "X", 0, site, &meter)
                });
                (answer, tally, meter.usage().work)
            };
            let (answer, tally, spent) = read(usize::MAX);
            let answer = answer.expect("export succeeds");
            assert_eq!(answer.len(), 1);
            assert_eq!(index.items[answer[0].0.0].path, "demo::d::X");
            assert!(tally.get(Work::RecomputedReads) > 0);
            assert_eq!(tally.get(Work::HolderScans) > 0, unrelated > 0);
            assert_eq!(
                spent,
                tally.get(Work::RecomputedReads)
                    + tally.get(Work::HolderScans)
                    + tally.get(Work::CandidateSteps)
            );
            assert_eq!(read(spent).0.expect("exact budget"), answer);
            let error = read(spent - 1).0.expect_err("one step less");
            assert!(
                matches!(error.kind, LoadErrorKind::Budget {
                budget: Budget::Work, limit
            } if limit == spent - 1),
                "{error}"
            );
        }
    }

    #[test]
    fn a_new_view_charges_raw_reads_and_unique_source_names_before_gathering() {
        use crate::meter::Meter;
        use crate::work::{Work, measure};

        let workspace = load(
            "pub mod d { pub struct X {} pub struct Z {} }\n\
             pub mod t { pub fn own() {} pub use crate::d::*; pub use crate::d::*; }\n",
            Budget::Work.limit(),
        )
        .expect("loads");
        let mut index = workspace.index().clone();
        let target = index
            .modules
            .iter()
            .position(|m| m.path == "demo::t")
            .expect("target");
        let root = index
            .modules
            .iter()
            .position(|m| m.path == "demo")
            .expect("root");
        assert_eq!(index.exports_of(target), []);
        let class = index.class_for(target, root);
        let reads = index.reads_of(target);
        assert_eq!(reads.len(), 2);
        assert_eq!(reads[0].view, reads[1].view);
        let site = reads[0].glob;
        // Both declarations are listed; the one shared source is gathered once.
        let listed = reads.len();
        let names = index.scopes[target].len() + index.globs.views[reads[0].view].names.len();
        assert_eq!((listed, names), (2, 3));
        let meter = Meter::new(&LoadOptions::default());
        let (view, tally) = measure(usize::MAX, || index.make_view(target, &class, site, &meter));
        let view = &index.globs.views[view.expect("new view")];
        assert_eq!(view.names.len(), 3);
        for view in &index.globs.views {
            for name in view.names.keys() {
                let canonical = index
                    .globs
                    .strings
                    .get(name.as_ref())
                    .expect("interned name");
                assert!(std::ptr::eq(name.as_ref(), canonical.as_ref()));
            }
        }
        for (name, expected) in [
            ("X", "demo::d::X"),
            ("Z", "demo::d::Z"),
            ("own", "demo::t::own"),
        ] {
            let held = view.names[name].iter().flatten().collect::<Vec<_>>();
            assert_eq!(held.len(), 1);
            assert_eq!(index.items[held[0].0.0].path, expected);
        }
        assert_eq!(
            meter.usage().work,
            listed + names + tally.get(Work::CandidateSteps)
        );
    }

    /// Each source family and its existing upper growth bound per doubling,
    /// in tenths. These bounds judge real loads, including import bookkeeping.
    type Shape = (&'static str, fn(usize) -> String, usize);

    const SHAPES: [Shape; 10] = [
        ("chain", chain, 20),
        ("many items", many_items, 20),
        ("fan-in of pub globs", pub_fan_in, 20),
        ("fan-in of private globs", private_fan_in, 20),
        ("companion fan-in", companion_fan_in, 20),
        ("prelude", prelude, 34),
        ("hub", hub, 43),
        ("ambiguous hub", ambiguous_hub, 43),
        ("lookups", lookups, 44),
        ("holder scan", holder_scan, 43),
    ];

    #[test]
    fn each_shape_loads_at_its_measured_work_and_refuses_one_step_less() {
        // Whole-load work includes queue and graph operations, so the old
        // table-only polynomials are not laws of this population. The export,
        // lookup and settling tests separately judge executed operation counts.
        for (shape, build, _) in SHAPES {
            for n in [8, 16, 24, 32] {
                let lib = build(n);
                let workspace = load(&lib, Budget::Work.limit()).expect("loads");
                let spent = workspace.usage().work;
                assert!(spent > 0, "{shape} of {n}");
                let exact = load(&lib, spent).expect("exact budget");
                assert_eq!(exact.usage(), workspace.usage(), "{shape} of {n}");
                let error = load(&lib, spent - 1).expect_err("one step less");
                assert!(
                    matches!(error.kind, LoadErrorKind::Budget {
                    budget: Budget::Work, limit
                } if limit == spent - 1),
                    "{shape} of {n}: {error}"
                );
                assert_eq!(error.file.as_str(), "src/lib.rs");
                let line = error.line.expect("responsible import");
                assert!(
                    lib.lines()
                        .nth(line - 1)
                        .is_some_and(|text| text.contains("use ")),
                    "{shape} of {n}: {error}"
                );
            }
        }
    }

    #[test]
    fn doubling_a_shape_keeps_actual_work_within_its_growth_bound() {
        let mut failures = Vec::new();
        for (shape, build, bound) in SHAPES {
            let measured =
                [32, 64, 128].map(|n| crate::work::measure(usize::MAX, || work(&build(n))));
            let [one, two, four] = measured.map(|(work, _)| work);
            assert!(
                one > 0 && two > one && four > two,
                "{shape}: {one}, {two}, {four}"
            );
            let (small, large) = (two - one, four - two);
            if large * 10 > small * bound {
                failures.push(format!(
                    "{shape}: {small} then {large}, upper bound {bound}/10; {measured:?}"
                ));
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    #[test]
    fn actual_work_from_100_to_800_stays_within_each_shapes_doubling_bound() {
        // Load the source at every size. Evaluating a polynomial would say
        // nothing about the algorithm that is actually running.
        for (shape, build, bound) in SHAPES {
            // This source family contains quadratically many declarations;
            // 400 and 800 exceed default parsing admission, checked below.
            let sizes = if shape == "holder scan" {
                [25, 50, 100, 200]
            } else {
                [100, 200, 400, 800]
            };
            let counts = sizes.map(|n| work(&build(n)));
            assert!(
                counts[0] > 0 && counts.windows(2).all(|pair| pair[1] > pair[0]),
                "{shape}: {counts:?}"
            );
            // Doubling increments cancel fixed startup work analytically,
            // including negative offsets in otherwise linear populations.
            let increments: Vec<_> = counts.windows(2).map(|pair| pair[1] - pair[0]).collect();
            for pair in increments.windows(2) {
                assert!(
                    pair[1] * 10 <= pair[0] * bound,
                    "{shape}: {counts:?}, upper bound {bound}/10"
                );
            }
        }
    }

    #[test]
    fn holder_scan_sources_beyond_the_admitted_growth_sizes_are_refused() {
        let tokens = load(&holder_scan(400), Budget::Work.limit()).expect_err("token budget");
        assert!(
            matches!(tokens.kind, LoadErrorKind::Budget {
            budget: Budget::FileTokens, limit
        } if limit == Budget::FileTokens.limit()),
            "{tokens}"
        );
        let bytes = load(&holder_scan(800), Budget::Work.limit()).expect_err("file byte budget");
        assert!(
            matches!(bytes.kind, LoadErrorKind::Tree(ref source)
            if matches!(source.as_ref(), crate::TreeError::TooLarge { limit, .. }
                if *limit == crate::DEFAULT_MAX_FILE_BYTES)),
            "{bytes}"
        );
    }

    #[test]
    fn a_settle_heavy_shape_is_refused_at_the_charge_that_passes_a_small_limit() {
        for (shape, lib) in [
            ("fan-in", private_fan_in(32)),
            ("prelude", prelude(32)),
            ("hub", hub(32)),
            ("holder scan", holder_scan(16)),
        ] {
            charges::take();
            let total = work(&lib);
            let all = charges::take();
            assert_eq!(all.iter().sum::<usize>(), total, "{shape}");
            // The limit is what the charges before one that counts a step
            // add up to, once they reach a tenth of the load's work.
            let (mut limit, mut place) = (0, 0);
            while limit < total / 10 || all[place] == 0 {
                limit += all[place];
                place += 1;
            }
            let error = load(&lib, limit).expect_err("over the limit");
            assert!(
                matches!(
                    error.kind,
                    LoadErrorKind::Budget {
                        budget: Budget::Work,
                        limit: refused
                    } if refused == limit
                ),
                "{shape}: {error}"
            );
            assert_eq!(error.file.as_str(), "src/lib.rs", "{shape}");
            let line = error.line.expect("the line of a glob");
            assert!(
                lib.lines()
                    .nth(line - 1)
                    .is_some_and(|text| text.contains("::*;")),
                "{shape}: {line}"
            );
            // The load stopped at the charge that passed the limit: the count
            // at the refusal is the limit and that charge, below the work of
            // the whole load.
            let made = charges::take();
            assert_eq!(made, all[..=place], "{shape}");
            let spent: usize = made.iter().sum();
            assert_eq!(spent, limit + all[place], "{shape}");
            assert!(spent < total, "{shape}: {spent} of {total}");
        }
    }
}

#[cfg(test)]
mod settling {
    //! What settling counts: exact operation pins for unchanged algorithms,
    //! topology and growth laws for canonical candidate delivery. Each reading
    //! again of what the globs of a
    //! module bring in under a name and the glob reads it looks at, the
    //! glob reads looked at while the drops are recorded, the lists of the
    //! views copied, the glob reads that the lookups of what the globs
    //! bring in look at and the routes they make, the views that hold a
    //! name looked at to find the reads whose view holds it, the steps of
    //! the reads that no view of their module can access, the arrivals
    //! taken as the change they bring and the items such an arrival
    //! exposes, and the records looked at when an item first arrives under
    //! a name at a view that knows its module brings in what it cannot
    //! access. Independent metering laws also judge the actual reads and
    //! scans, so growth alone cannot hide an omitted charge.

    use super::shapes::{
        MANIFEST, ambiguous_hub, chain, companion_fan_in, holder_scan, hub, lookups, many_items,
        prelude, private_fan_in, pub_fan_in,
    };
    use crate::work::{Work, measure};
    use crate::{MemoryTree, Workspace};

    /// The smallest size each shape is counted at, and so the step between
    /// the sizes: an even size, as the hubs need.
    const N: usize = 8;

    /// The kinds of work pinned, in the order of a [`Pin`].
    const KINDS: [Work; 11] = [
        Work::Recomputations,
        Work::RecomputedReads,
        Work::DropReads,
        Work::ListCopies,
        Work::LookupReads,
        Work::LookupRoutes,
        Work::HolderScans,
        Work::CovertChecks,
        Work::Deltas,
        Work::Exposures,
        Work::HiddenChecks,
    ];

    /// The count of each kind of [`KINDS`] at a size.
    type Pin = [fn(usize) -> usize; 11];

    /// Nothing counted.
    const fn none(_n: usize) -> usize {
        0
    }

    /// What loading `lib` counts of each kind of [`KINDS`].
    fn counted(lib: &str) -> [usize; 11] {
        let tree: MemoryTree = [("Cargo.toml", MANIFEST), ("src/lib.rs", lib)]
            .into_iter()
            .collect();
        let (_, tally) = measure(usize::MAX, || {
            Workspace::load(&tree, &["."]).unwrap_or_else(|error| panic!("loads: {error}"))
        });
        KINDS.map(|work| tally.get(work))
    }

    /// Assert that the shape `build` counts what `pin` gives at n, 2n and
    /// 3n, and return the counts, by size and kind.
    fn assert_pinned(shape: &str, build: fn(usize) -> String, pin: Pin) -> [[usize; 11]; 3] {
        [1, 2, 3].map(|units| {
            let n = units * N;
            let counts = counted(&build(n));
            let expected = pin.map(|count| count(n));
            assert_eq!(counts, expected, "{shape} of {n}: {KINDS:?}");
            counts
        })
    }

    /// Preserve the original two-construction operation pins while proving
    /// exactly one real replay is absent from fixed-path production loads.
    fn assert_one_replay_omitted(
        shape: &str,
        build: fn(usize) -> String,
        pin: Pin,
    ) -> [[usize; 11]; 3] {
        [1, 2, 3].map(|units| {
            let n = units * N;
            let lib = build(n);
            let tree: MemoryTree = [("Cargo.toml", MANIFEST), ("src/lib.rs", lib.as_str())]
                .into_iter()
                .collect();
            let (ws, actual) = measure(usize::MAX, || {
                Workspace::load(&tree, &["."]).expect("fixed-path load")
            });
            let mut index = ws.index().clone();
            index.lookups.built(false);
            let meter = crate::meter::Meter::new(&crate::LoadOptions::default());
            let decl = &index.uses[index.glob_imports[0]];
            // The clone owns resident tables. Admit their actual retained
            // charge before a replay releases and replaces those tables.
            meter
                .globs(index.globs.charged, &decl.file, Some(decl.line))
                .expect("resident cloned tables fit");
            let ((), replay) = measure(usize::MAX, || {
                index
                    .replay_globs_for_measurement(&meter)
                    .expect("one actual replay");
            });
            let counts = KINDS.map(|kind| actual.get(kind));
            let with_replay = KINDS.map(|kind| actual.get(kind) + replay.get(kind));
            assert_eq!(
                with_replay,
                pin.map(|count| count(n)),
                "{shape} of {n}: actual + one replay; {KINDS:?}"
            );
            assert!(
                replay.get(Work::CovertChecks) > 0,
                "the omitted replay has real work"
            );
            counts
        })
    }

    /// Of counts at n, 2n and 3n, the second difference: zero when they
    /// grow linearly.
    fn second_difference([one, two, three]: [usize; 3]) -> isize {
        let [one, two, three] =
            [one, two, three].map(|count| isize::try_from(count).expect("a small count"));
        three - 2 * two + one
    }

    /// The counts of one kind, the place of `work` in [`KINDS`], at n, 2n
    /// and 3n.
    fn of(counts: &[[usize; 11]; 3], work: Work) -> [usize; 3] {
        let place = KINDS
            .iter()
            .position(|kind| *kind == work)
            .expect("a pinned kind");
        counts.map(|at| at[place])
    }

    #[test]
    fn the_settling_of_a_chain_of_globs_counts_its_pinned_polynomials_at_n_2n_and_3n() {
        // Each link passes on what it brings in: nothing is read again or
        // copied, and nothing is looked up through the globs.
        assert_pinned("chain", chain, [none; 11]);
    }

    #[test]
    fn the_settling_of_a_module_of_many_items_globbed_from_many_modules_counts_its_pinned_polynomials_at_n_2n_and_3n()
     {
        assert_pinned("many items", many_items, [none; 11]);
    }

    #[test]
    fn the_settling_of_a_fan_in_of_pub_globs_counts_its_pinned_polynomials_at_n_2n_and_3n() {
        assert_pinned("fan-in of pub globs", pub_fan_in, [none; 11]);
    }

    #[test]
    fn the_settling_of_a_fan_in_of_private_globs_counts_its_pinned_polynomials_at_n_2n_and_3n() {
        // The view of `t` for `u` is read through private globs that `u`
        // cannot access: nothing of what they bring in is gathered name by
        // name, so nothing is read again or copied. The reads are listed
        // among the covert readers when the view is made, and their views
        // never grow after it.
        assert_pinned("fan-in of private globs", private_fan_in, [none; 11]);
    }

    #[test]
    fn a_companion_fan_in_keeps_each_operation_population_linear_without_list_copies() {
        let counts = [1, 2, 3].map(|units| counted(&companion_fan_in(units * N)));
        for work in KINDS {
            assert_eq!(second_difference(of(&counts, work)), 0, "{work:?}");
        }
        for work in [
            Work::Recomputations,
            Work::RecomputedReads,
            Work::DropReads,
            Work::HolderScans,
            Work::CovertChecks,
            Work::Exposures,
        ] {
            assert!(of(&counts, work).iter().all(|count| *count > 0), "{work:?}");
        }
        assert_eq!(of(&counts, Work::ListCopies), [0; 3]);
    }

    #[test]
    fn a_reading_again_looks_at_the_reads_whose_view_holds_the_name_whatever_the_reads_of_the_module()
     {
        // Each X<i> is confined to its a/d/p branch and the shared t/u
        // route. At most two reads of any one target can hold that name,
        // however many unrelated reads t has. At most four views hold it.
        for n in [100, 200, 400] {
            let private = counted(&private_fan_in(n));
            assert_eq!(
                [private[0], private[1]],
                [0, 0],
                "fan-in of private globs of {n}"
            );
            let carried = counted(&companion_fan_in(n));
            let [recomputed, reads, drops, _, _, _, scans, ..] = carried;
            assert!(recomputed > 0 && reads > 0 && scans > 0);
            assert!(reads <= 2 * recomputed, "{n}: {carried:?}");
            assert!(scans <= 4 * (recomputed + drops), "{n}: {carried:?}");
        }
    }

    #[test]
    fn the_settling_of_the_prelude_pattern_counts_its_pinned_polynomials_at_n_2n_and_3n() {
        // The view of each child for the root is read through the child's
        // private glob of the root, which the root cannot access: it is not
        // gathered name by name, nothing is read again or copied, and the
        // child's read of the root's view is covert. Each gain of the
        // root's view looks at the views that hold the gained name, not at
        // every child that reads the view covertly, so the covert steps
        // grow linearly.
        let counts = assert_one_replay_omitted(
            "prelude",
            prelude,
            [
                none,
                none,
                none,
                none,
                none,
                none,
                none,
                |n| 8 * n - 5,
                none,
                none,
                none,
            ],
        );
        assert_eq!(second_difference(of(&counts, Work::CovertChecks)), 0);
    }

    #[test]
    fn the_settling_of_a_hub_counts_its_pinned_polynomials_at_n_2n_and_3n() {
        // Nothing is read again: the hub's private globs of the odd
        // siblings are covert, and each gain of an odd sibling's view looks
        // at the hub once, finding that it holds what arrives; each name
        // that comes back to the hub through an even sibling arrives at a
        // view that holds it and changes nothing, tested without a copy.
        // Both grow with the square of the siblings, as the names do.
        let counts = assert_one_replay_omitted(
            "hub",
            hub,
            [
                none,
                none,
                none,
                none,
                none,
                none,
                none,
                |n| n * n,
                |n| n * n - 2 * n,
                none,
                none,
            ],
        );
        for work in [Work::CovertChecks, Work::Deltas] {
            assert_eq!(
                2 * second_difference(of(&counts, work)),
                2 * 8 * 8,
                "{work:?}"
            );
        }
    }

    #[test]
    fn a_companion_hub_keeps_local_definitions_and_reads_without_copying_candidate_lists() {
        for n in [8, 16, 24] {
            let lib = ambiguous_hub(n);
            let tree: MemoryTree = [("Cargo.toml", MANIFEST), ("src/lib.rs", lib.as_str())]
                .into_iter()
                .collect();
            let (workspace, tally) = measure(usize::MAX, || {
                Workspace::load(&tree, &["."]).expect("loads")
            });
            for sibling in 0..n {
                let path = format!("demo::s{sibling}::T{}", sibling / 2);
                assert_eq!(
                    workspace.resolve(&path).expect("local definition").path,
                    path
                );
            }
            assert!(tally.get(Work::RecomputedReads) > 0);
            assert!(tally.get(Work::DropReads) > 0);
            assert_eq!(tally.get(Work::ListCopies), 0);
        }
        // The separate combined growth guard covers all rereads, candidate,
        // carrier graph and newly metered settling work, including total work.
    }

    #[test]
    fn a_companion_hub_bounds_delivery_and_carrier_graph_growth_together() {
        let kinds = [
            Work::Recomputations,
            Work::RecomputedReads,
            Work::DropReads,
            Work::ListCopies,
            Work::HolderScans,
            Work::CovertChecks,
            Work::Deltas,
            Work::SupplySteps,
            Work::CandidateSteps,
            Work::SettleSteps,
            Work::ProofReads,
            Work::ProofCopies,
            Work::ProofSteps,
        ];
        let counts = [8, 16, 24, 32].map(|n| {
            let lib = ambiguous_hub(n);
            let tree: MemoryTree = [("Cargo.toml", MANIFEST), ("src/lib.rs", lib.as_str())]
                .into_iter()
                .collect();
            let (workspace, tally) = measure(usize::MAX, || {
                Workspace::load(&tree, &["."]).expect("loads")
            });
            let counts = kinds.map(|kind| tally.get(kind));
            eprintln!(
                "n = {n}: {kinds:?} = {counts:?}; charged work = {}",
                workspace.usage().work
            );
            (counts, workspace.usage().work)
        });
        // Include carrier graph construction and all newly metered queue /
        // decision work; reducing rereads alone cannot establish this bound.
        for (at, kind) in kinds.into_iter().enumerate() {
            if matches!(
                kind,
                Work::ProofReads | Work::ProofCopies | Work::ProofSteps
            ) {
                assert!(counts[1].0[at] > 0, "actual constructive work: {kind:?}");
            }
            assert!(
                counts[3].0[at] * 10 <= counts[1].0[at] * 43,
                "{kind:?}: {counts:?}"
            );
        }
        assert!(
            counts[3].1 * 10 <= counts[1].1 * 43,
            "whole charged work: {counts:?}"
        );
    }

    #[test]
    fn named_import_lookup_routes_stay_linear_and_read_work_stays_quadratic() {
        let sizes = [8, 16, 32];
        let counts = sizes.map(|n| counted(&lookups(n)));
        for (n, counts) in sizes.into_iter().zip(counts) {
            for (kind, count) in KINDS.into_iter().zip(counts) {
                match kind {
                    // Privacy guards also revisit missing Macro paths, so
                    // read counts include more than the successful routes.
                    Work::LookupReads => assert!(count > 0),
                    Work::LookupRoutes => {
                        let routes = if cfg!(debug_assertions) { 17 } else { 15 };
                        assert_eq!(count, routes * n);
                    }
                    _ => assert_eq!(count, 0, "{kind:?}"),
                }
            }
        }
        let reads = counts.map(|counts| counts[4]);
        assert!(reads[0] > 0 && reads[1] > reads[0] && reads[2] > reads[1]);
        assert!(
            (reads[2] - reads[1]) * 10 <= (reads[1] - reads[0]) * 44,
            "{reads:?}"
        );
    }

    #[test]
    fn the_settling_of_a_module_whose_names_many_views_hold_counts_its_pinned_polynomials_at_n_2n_and_3n()
     {
        // Each `X<k>` first arrives at the view of each `t<i>` for `u<i>`,
        // which knows that `t<i>`'s private glob of `h` brings in an item
        // `u<i>` cannot access, once in the retained build: 2n^2 first arrivals.
        // Each is taken as it comes once the record of the names the view
        // dropped and `h`'s view, read through a glob `u<i>` cannot access,
        // show that no such item stands under `X<k>`: two steps each, and
        // nothing is read again, so no view that holds the name is looked
        // at to find the reads whose view holds it.
        let counts = assert_one_replay_omitted(
            "module whose names many views hold",
            holder_scan,
            [
                none,
                none,
                none,
                none,
                none,
                none,
                none,
                |n| 2 * n,
                none,
                none,
                |n| 8 * n * n,
            ],
        );
        assert_eq!(
            2 * second_difference(of(&counts, Work::HiddenChecks)),
            8 * 2 * 8 * 8
        );
    }
}

#[cfg(test)]
mod monotone {
    //! A view is only added to or widened: an entry held as a companion
    //! stays one until it is added plain, and is widened as it is added
    //! again, and an entry that would narrow one is refused.

    use super::{Held, Index, Level};
    use crate::item::ItemId;
    use crate::meter::Meter;
    use crate::options::LoadOptions;
    use crate::{MemoryTree, Workspace};

    /// An index of a crate with a glob import, its tables cleared, and a view
    /// of the root in it: the view, the root and the module `d`.
    fn viewed() -> (Index, Meter, usize) {
        let tree: MemoryTree = [
            (
                "Cargo.toml",
                "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            ),
            (
                "src/lib.rs",
                "pub mod d { pub struct X; }\npub mod a { pub use crate::d::*; }\n",
            ),
        ]
        .into_iter()
        .collect();
        let mut index = Workspace::load(&tree, &["."])
            .expect("loads")
            .index()
            .clone();
        let meter = Meter::new(&LoadOptions::default());
        index.clear_globs();
        index.globs.exports = vec![Vec::new(); index.modules.len()];
        index.globs.reads = vec![Vec::new(); index.modules.len()];
        let site = *index.glob_imports.first().expect("a glob import");
        let view = index
            .make_view(0, &[true], site, &meter)
            .expect("the view is made");
        (index, meter, view)
    }

    /// What `view` holds of `item` under `X` in the type name space.
    fn held(index: &Index, view: usize, item: ItemId) -> Option<Held> {
        let lists = index.globs.views[view].names.get("X")?;
        lists[0].iter().copied().find(|entry| entry.0 == item)
    }

    #[test]
    fn an_entry_held_as_a_companion_is_widened_as_it_is_added_again_and_made_plain_only_by_a_plain_entry()
     {
        let (mut index, meter, view) = viewed();
        let item = ItemId(1);
        // Added as a companion within `d`, then as a companion within the
        // root, which is wider: it stays a companion and is widened.
        let narrow = Level::within(1);
        let wide = Level::within(0);
        assert!(index.width(wide) < index.width(narrow));
        for (level, expected) in [
            (narrow.companion(), narrow.companion()),
            (wide.companion(), wide.companion()),
            // Again at the same width, and at a narrower one: nothing
            // changes and nothing is added.
            (wide.companion(), wide.companion()),
            (narrow.companion(), wide.companion()),
        ] {
            index
                .add_to_view(view, "X", 0, &[(item, level)], &meter)
                .expect("added");
            assert_eq!(held(&index, view, item), Some((item, expected)));
        }
        // A plain entry takes the place of the companion, at the wider of
        // the two levels.
        let fresh = index
            .add_to_view(view, "X", 0, &[(item, wide)], &meter)
            .expect("added");
        assert_eq!(fresh.ids, vec![(item, wide)]);
        assert_eq!(held(&index, view, item), Some((item, wide)));
        // And a companion does not make it one again.
        index
            .add_to_view(view, "X", 0, &[(item, Level::PUBLIC.companion())], &meter)
            .expect("added");
        assert_eq!(held(&index, view, item), Some((item, Level::PUBLIC)));
    }

    #[test]
    #[should_panic(expected = "narrows")]
    fn an_entry_that_narrows_the_level_an_item_is_held_at_is_refused() {
        let (mut index, meter, view) = viewed();
        let item = ItemId(1);
        index
            .add_to_view(view, "X", 0, &[(item, Level::PUBLIC.companion())], &meter)
            .expect("added");
        index.assert_widened(view, "X", 0, &[(item, Level::within(1).companion())]);
    }

    #[test]
    #[should_panic(expected = "narrows")]
    fn an_entry_that_makes_a_plain_item_a_companion_again_is_refused() {
        let (mut index, meter, view) = viewed();
        let item = ItemId(1);
        index
            .add_to_view(view, "X", 0, &[(item, Level::within(0))], &meter)
            .expect("added");
        index.assert_widened(view, "X", 0, &[(item, Level::within(0).companion())]);
    }
}

#[cfg(test)]
mod table_bytes {
    //! What the tables that find the glob reads whose view holds a name are
    //! charged to the glob bytes: the first room of the tables of the views
    //! that hold each name and of the names held with a companion, and of
    //! the places of each view's readers, a slot among the holders of a name
    //! for each name a view takes, an entry for a name no view held and for
    //! the first companion under a name, and the place of each read among
    //! the reads of its module of its view. Each is counted exactly on a
    //! small crate, from the layout rules and, on a 64-bit target, as bytes.

    use std::mem::size_of;

    use super::{Held, Index, Level, Nobody, Unseen, covert_bytes, set_bytes};
    use crate::cost::{
        btree_entry, btree_map, hash_entry, hash_map, shared_text, vec_first, vec_slot,
    };
    use crate::item::{ItemId, SharedStr};
    use crate::meter::Meter;
    use crate::options::LoadOptions;
    use crate::{MemoryTree, Workspace};

    /// What one name a view takes is charged whatever else it is: its
    /// entry and lists, its list's first room and the slot of its item.
    const fn name_taken() -> usize {
        super::NAME_BYTES + vec_first::<Held>() + vec_slot::<Held>()
    }

    /// An index of a crate whose module `a` globs the empty module `d`
    /// publicly and privately, its tables built again empty, with a meter,
    /// so that no view holds a name until the test adds it: the index,
    /// the meter, the modules `a` and `d`, the two glob imports of `a`, and
    /// what building the tables again charged.
    fn emptied() -> (Index, Meter, usize, usize, [usize; 2], usize) {
        let tree: MemoryTree = [
            (
                "Cargo.toml",
                "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            ),
            (
                "src/lib.rs",
                "pub mod d {}\npub mod a { pub use crate::d::*; use crate::d::*; }\n",
            ),
        ]
        .into_iter()
        .collect();
        let mut index = Workspace::load(&tree, &["."])
            .expect("loads")
            .index()
            .clone();
        let meter = Meter::new(&LoadOptions::default());
        index.clear_globs();
        index
            .reset_globs(&meter)
            .expect("the tables are built again");
        let module = |path: &str| {
            index
                .modules
                .iter()
                .position(|module| module.path == path)
                .expect("a module of the crate")
        };
        let (a, d) = (module("demo::a"), module("demo::d"));
        let globs = [index.glob_imports[0], index.glob_imports[1]];
        let base = index.globs.charged;
        (index, meter, a, d, globs, base)
    }

    #[test]
    fn the_tables_of_the_views_that_hold_each_name_and_of_the_names_with_a_companion_are_charged_their_first_room()
     {
        let (index, .., base) = emptied();
        let modules = index.modules.len();
        let expected = modules * (vec_slot::<Vec<usize>>() + vec_slot::<Vec<super::GlobRead>>())
            + vec_first::<Vec<usize>>()
            + vec_first::<Vec<super::GlobRead>>()
            + vec_first::<super::View>()
            + btree_map::<(usize, Box<[bool]>), usize>()
            + btree_map::<(usize, usize), ()>()
            + btree_map::<(usize, usize, usize), ()>()
            + hash_map::<SharedStr>()
            + hash_map::<(SharedStr, [bool; 3])>()
            + hash_map::<(SharedStr, Vec<usize>)>();
        assert_eq!(base, expected, "{modules} modules");
        #[cfg(target_pointer_width = "64")]
        assert_eq!((modules, base), (3, BASE_BYTES));
    }

    #[test]
    fn each_name_a_view_takes_is_charged_its_slot_among_the_holders_of_the_name_and_the_first_companion_its_entry()
     {
        let (mut index, meter, a, d, globs, _) = emptied();
        let class = index.class_for(d, a);
        let first = index.make_view(d, &class, globs[0], &meter).expect("made");
        let second = index
            .make_view(a, &[true, false], globs[0], &meter)
            .expect("made");
        let item = ItemId(1);
        let take = |index: &mut Index, view: usize, name: &str, level: Level| {
            let before = index.globs.charged;
            index
                .add_to_view(view, name, 0, &[(item, level)], &meter)
                .expect("added");
            index.globs.charged - before
        };
        // A name no view held: its text and its entry among the strings,
        // and its entry among the holders with the first room of its list
        // and the view's slot in it.
        let text = shared_text(1) + hash_entry::<SharedStr>();
        let holders = hash_entry::<(SharedStr, Vec<usize>)>() + vec_first::<usize>();
        let slot = vec_slot::<usize>();
        let companion = hash_entry::<(SharedStr, [bool; 3])>();
        let charged = [
            take(&mut index, first, "X", Level::PUBLIC),
            // The same name taken by another view: its slot only.
            take(&mut index, second, "X", Level::PUBLIC),
            // A companion under a name no view held: and the entry of the
            // name among the names held with a companion.
            take(&mut index, first, "Y", Level::PUBLIC.companion()),
            // Another companion under that name: no second entry.
            take(&mut index, second, "Y", Level::PUBLIC.companion()),
        ];
        let expected = [
            name_taken() + text + holders + slot,
            name_taken() + slot,
            name_taken() + text + holders + slot + companion,
            name_taken() + slot,
        ];
        assert_eq!(charged, expected);
        #[cfg(target_pointer_width = "64")]
        assert_eq!(charged, TAKEN_BYTES);
        assert_eq!(index.globs.holding.get("X"), Some(&vec![first, second]));
        assert_eq!(index.globs.holding.get("Y"), Some(&vec![first, second]));
        assert_eq!(index.globs.companions.get("Y"), Some(&[true, false, false]));
        assert_eq!(index.globs.companions.get("X"), None);
    }

    #[test]
    fn each_read_of_a_view_is_charged_its_place_among_the_reads_of_its_module_of_that_view() {
        let (mut index, meter, a, d, globs, _) = emptied();
        let class = index.class_for(d, a);
        let before = index.globs.charged;
        let view = index.make_view(d, &class, globs[0], &meter).expect("made");
        let made = index.globs.charged - before;
        let read = |index: &mut Index, glob: usize| {
            let before = index.globs.charged;
            index.read_glob(glob, d, &mut Nobody, &meter).expect("read");
            index.globs.charged - before
        };
        let charged = [read(&mut index, globs[0]), read(&mut index, globs[1])];
        // The view: its record, its tables of names and of readers, and the
        // two tables of the places of its readers' reads, first room each.
        assert_eq!(
            made,
            super::VIEW_BYTES + 2 * class.len(),
            "the view of `d` for `a`"
        );
        assert_eq!(
            super::VIEW_BYTES,
            vec_slot::<super::View>()
                + btree_entry::<(usize, Box<[bool]>), usize>()
                + vec_slot::<usize>()
                + vec_first::<usize>()
                + btree_map::<SharedStr, super::Lists>()
                + vec_first::<(usize, usize, usize)>()
                + 2 * btree_map::<usize, Vec<usize>>()
        );
        // The first read of `a` of the view: its record, its place, and the
        // entry of `a` among the modules that read the view with the first
        // room of its places; the second: its record and its place.
        let place = vec_slot::<usize>();
        let first_places = btree_entry::<usize, Vec<usize>>() + vec_first::<usize>();
        assert_eq!(
            charged,
            [
                super::READ_BYTES + vec_first::<super::GlobRead>() + place + first_places,
                super::READ_BYTES + place
            ]
        );
        #[cfg(target_pointer_width = "64")]
        assert_eq!((made, charged), READ_VIEW_BYTES);
        assert_eq!(index.globs.views[view].places.get(&a), Some(&vec![0, 1]));
    }

    /// The first room of the tables of a load of three modules.
    #[cfg(target_pointer_width = "64")]
    const BASE_BYTES: usize = 2688;

    /// What the four names of the test are charged.
    #[cfg(target_pointer_width = "64")]
    const TAKEN_BYTES: [usize; 4] = [520, 290, 595, 290];

    /// What the view and its two reads are charged.
    #[cfg(target_pointer_width = "64")]
    const READ_VIEW_BYTES: (usize, [usize; 2]) = (1627, [789, 526]);

    /// The tables of [`emptied`] with a view of `a` for importers outside
    /// it, such as `d`, which can access `a`'s `pub` glob of `d` but not its
    /// private one: the index, the meter, `a`, `d`, the two glob imports of
    /// `a`, the view and its class.
    fn viewed_from_outside() -> (Index, Meter, usize, usize, [usize; 2], usize, Box<[bool]>) {
        let (mut index, meter, a, d, globs, _) = emptied();
        let outside = index.class_for(a, d);
        let view = index
            .make_view(a, &outside, globs[0], &meter)
            .expect("made");
        (index, meter, a, d, globs, view, outside)
    }

    /// What running `run` on `index` charges to the glob tables.
    fn charged(index: &mut Index, run: &dyn Fn(&mut Index)) -> usize {
        let before = index.globs.charged;
        run(index);
        index.globs.charged - before
    }

    #[test]
    fn each_covert_reader_open_reader_and_view_read_through_a_glob_a_class_cannot_access_is_charged_its_entry_and_the_first_room_of_its_table()
     {
        let (mut index, meter, a, d, globs, view, outside) = viewed_from_outside();
        let metered = &meter;
        let read = |glob: usize| {
            move |index: &mut Index| {
                index
                    .read_glob(glob, d, &mut Nobody, metered)
                    .expect("read");
            }
        };
        // The `pub` glob makes the view of `d` for `a` and reads it, as an
        // active reader; the private glob reads it covertly. Besides its
        // record and its place, the covert read is charged its entry among
        // the views `a`'s view cannot access through a glob, its entry among
        // the covert readers of `d`'s view, and, as `d`'s view holds nothing
        // yet, so `a`'s view does not know what it cannot access, its entry
        // among the open readers, each with the first room of its table,
        // and the record of each of the two views where it is first made.
        let public = charged(&mut index, &read(globs[0]));
        let private = charged(&mut index, &read(globs[1]));
        let d_view = index.globs.exports[d][0];
        assert!(index.globs.views[d_view].unseen().covert.contains_key(&a));
        assert!(index.globs.views[d_view].unseen().open.contains(&a));
        assert_eq!(
            index.globs.views[view]
                .unseen()
                .hiding
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            vec![d_view]
        );
        let made = super::VIEW_BYTES + 2 * outside.len();
        let place = vec_slot::<usize>();
        let first_places = btree_entry::<usize, Vec<usize>>() + vec_first::<usize>();
        let expected_public =
            made + super::READ_BYTES + vec_first::<super::GlobRead>() + place + first_places;
        let expected_private = super::READ_BYTES
            + place
            + set_bytes::<usize>(true)
            + covert_bytes(true, true)
            + set_bytes::<usize>(true)
            + 2 * size_of::<Unseen>();
        assert_eq!([public, private], [expected_public, expected_private]);
        assert_eq!(
            covert_bytes(true, true),
            vec_slot::<(usize, usize)>()
                + btree_entry::<usize, Vec<(usize, usize)>>()
                + vec_first::<(usize, usize)>()
                + btree_map::<usize, Vec<(usize, usize)>>()
        );
        assert_eq!(
            set_bytes::<usize>(true),
            btree_entry::<usize, ()>() + btree_map::<usize, ()>()
        );
        #[cfg(target_pointer_width = "64")]
        assert_eq!([public, private], COVERT_READ_BYTES);
    }

    #[test]
    fn each_name_a_view_dropped_and_each_name_a_reading_again_set_is_charged_its_entry_and_the_first_room_of_its_record()
     {
        let (mut index, meter, _, _, globs, view, _) = viewed_from_outside();
        let site = globs[0];
        let x = SharedStr::from("X");
        // The names that a view dropped what its class cannot access under:
        // the first with the first room of the set and the view's record,
        // made then, another its entry, and one recorded again nothing.
        let drop_at = |at: usize| {
            let (x, meter) = (x.clone(), &meter);
            move |index: &mut Index| {
                index
                    .note_dropped(view, x.clone(), at, site, meter)
                    .expect("noted");
            }
        };
        let dropped = [0, 1, 1].map(|at| charged(&mut index, &drop_at(at)));
        assert_eq!(
            dropped,
            [
                set_bytes::<(SharedStr, usize)>(true) + size_of::<Unseen>(),
                set_bytes::<(SharedStr, usize)>(false),
                0
            ]
        );
        // The names a reading again set from the globs: one the tables never
        // held is charged its text too, the first entry the first room of the
        // record, and one recorded again nothing.
        let mark_at = |at: usize| {
            let meter = &meter;
            move |index: &mut Index| {
                index
                    .mark_exact(view, "Q", at, site, meter)
                    .expect("marked");
            }
        };
        let exact = [0, 1, 1].map(|at| charged(&mut index, &mark_at(at)));
        assert_eq!(
            exact,
            [
                shared_text(1)
                    + hash_entry::<SharedStr>()
                    + set_bytes::<(usize, SharedStr, usize)>(true),
                set_bytes::<(usize, SharedStr, usize)>(false),
                0
            ]
        );
        #[cfg(target_pointer_width = "64")]
        assert_eq!((dropped, exact), RECORD_BYTES);
    }

    /// What the two reads of `d` by `a` are charged.
    #[cfg(target_pointer_width = "64")]
    const COVERT_READ_BYTES: [usize; 2] = [2416, 1647];

    /// What the dropped names and the names read again are charged.
    #[cfg(target_pointer_width = "64")]
    const RECORD_BYTES: ([usize; 3], [usize; 3]) = ([490, 106, 0], [586, 135, 0]);
}
