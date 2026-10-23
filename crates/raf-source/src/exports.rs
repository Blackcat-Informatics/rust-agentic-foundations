// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The exported items of a target: what another crate can name through its
//! crate root.
//!
//! The surface is read from the settled scope tables (see [`crate::settle`]),
//! breadth first from the crate root through the modules that a public
//! binding reaches. A binding is public when it has a public route back to a
//! public definition: each `use` declaration on the way, named or glob, is
//! `pub` and not test code, and the definition is `pub` (a macro is
//! `#[macro_export]`). That is rustc's rule: an import of a binding that is
//! less visible than the import is refused, and a glob brings each name in no
//! more visible than the binding it reads, so one hop that is not `pub` keeps
//! the name out. The route is searched over every glob that brings the item
//! in, not only the one the tables keep, so an item that a public glob and a
//! private one both bring in is exported.
//!
//! Every binding of the surface is checked as the resolver checks the last
//! segment of a path (see [`Index::binding_failure`]), and the modules of a
//! path are each reached through such a binding, so a name that is
//! ambiguous, that goes through a cycle of imports or that reads a glob whose
//! path names nothing fails the query with the error
//! [`Index::resolve`] gives for its path, as it fails a lookup.
//!
//! What the surface holds is charged to the item-bytes budget after what the
//! load spent, by the layout rules of `crate::cost`, before it is kept. A few
//! temporaries are not charged, each bounded by what the load already charged
//! and dropped before the next is made: the search for the public route of one
//! binding (its stack and the places it has seen, at most the bindings of the
//! tables) and the items one name binds. What carrying the re-exports of the
//! modules a reader's globs reach takes, its walk, its graph and tree of
//! dominators, its searches and their answers, is charged, the work of each
//! step included (see `Surface::carried`).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt;
use std::mem::size_of;
use std::rc::Rc;

use crate::cost::{btree_entry, btree_map, vec_first, vec_slot};
use crate::error::{Esc, LoadError};
use crate::facts::{SpanFact, TestRole, Vis};
use crate::index::{Index, UseDecl, UseKind};
use crate::item::{ItemId, Namespace, SymbolKind};
use crate::meter::Meter;
use crate::modtree::TargetId;
use crate::path::RelPath;
use crate::resolve::{Fail, ResolveError};
use crate::settle::slot;
use crate::work::{self as tally, Work};

/// What [`Workspace::exported_items_with`](crate::Workspace::exported_items_with)
/// lists.
///
/// The options are `#[non_exhaustive]`: a caller starts from
/// `ExportOptions::default()` and sets the fields it needs, so adding an
/// option is not a breaking change.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ExportOptions {
    /// List the items and re-exports that `#[doc(hidden)]` hides as well,
    /// each with [`ExportedItem::doc_hidden`] set. False by default, which
    /// leaves them out.
    pub include_hidden: bool,
}

/// One item of a target's public surface, under one path that another crate
/// can name it by.
///
/// The struct is `#[non_exhaustive]`: adding a field is not a breaking change,
/// so a pattern names the fields it reads and ends in `..`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ExportedItem {
    /// The path from the crate root, starting with the target's crate path:
    /// the exported name, which differs from the defining item's own path for
    /// a re-export (`demo::run` for `pub use inner::go as run;`).
    pub path: String,
    /// The defining item, so a re-export maps back to it.
    pub item: ItemId,
    /// What the defining item is.
    pub kind: SymbolKind,
    /// The file that defines the item.
    pub file: RelPath,
    /// The whole defining item, attributes included.
    pub span: SpanFact,
    /// True when `#[doc(hidden)]` hides the path: on the item, on a module of
    /// the path (on its declaration or as `#![doc(hidden)]` in its file), on
    /// the `use` declaration that exports it, or on the type or trait whose
    /// member it is.
    pub doc_hidden: bool,
}

/// A public re-export of a target that names nothing the target itself
/// defines as an item of the index: an item of another crate of the
/// workspace, a crate outside the workspace or one of its items, an enum
/// variant (variants are not items of the index), or a path the index cannot
/// follow, such as an item a macro generates. A public glob carries such a
/// re-export of a module it reaches into the module that reads it, under the
/// reader's path.
///
/// The struct is `#[non_exhaustive]`: adding a field is not a breaking change.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Reexport {
    /// The exported path from the crate root, starting with the target's
    /// crate path; `<module>::*` for a glob.
    pub path: String,
    /// The path the declaration names, as written (`serde::Serialize`,
    /// `serde::*`, or the crate of an `extern crate` item).
    pub written: String,
    /// The item it names, when the index sees one: an item of another crate
    /// of the workspace, or that crate's root. `None` for a crate outside the
    /// workspace and for a path the index cannot follow.
    pub item: Option<ItemId>,
    /// The file of the declaration.
    pub file: RelPath,
    /// The 1-based line of the declaration.
    pub line: usize,
    /// True when `#[doc(hidden)]` hides the re-export, on the declaration or
    /// on a module of its path.
    pub doc_hidden: bool,
}

/// The public surface of a target, as
/// [`Workspace::exported_items`](crate::Workspace::exported_items) returns it.
///
/// The struct is `#[non_exhaustive]`: adding a field is not a breaking change.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Exports {
    /// The items of the target that another crate can name, sorted by path,
    /// then by the order of [`Workspace::items`](crate::Workspace::items) of
    /// the defining item.
    pub items: Vec<ExportedItem>,
    /// The public re-exports that name nothing the target defines, sorted by
    /// path, file and line.
    pub reexports: Vec<Reexport>,
}

/// Why the public surface of a target could not be listed.
#[derive(Debug)]
#[non_exhaustive]
pub enum ExportError {
    /// The id names no library or binary target of the workspace: only
    /// those have a crate root that is a public surface.
    NotLibraryOrBinary(TargetId),
    /// What the surface holds passes the item-bytes budget left after the
    /// load: a `Budget` load error naming the budget, its limit and the file
    /// and line of the item whose entry passed it.
    Budget(LoadError),
    /// Deciding whether the public globs of a module of the surface carry
    /// the re-export at `path` from a module they reach passes the
    /// item-bytes budget left after the load, which charges the work of that
    /// decision: `error` is the `Budget` load error, naming the file and line
    /// of the carried `use` declaration.
    CarryBudget {
        /// The exported path of the re-export being decided.
        path: String,
        /// The `Budget` load error.
        error: LoadError,
    },
    /// A public name of the surface does not resolve: it is ambiguous, goes
    /// through a cycle of imports, or reads a glob whose path names nothing
    /// in the workspace, as [`Workspace::resolve`](crate::Workspace::resolve)
    /// reports it for that path.
    Resolve(ResolveError),
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotLibraryOrBinary(id) => write!(
                f,
                "target {} is not a library or a binary of the workspace",
                id.index()
            ),
            Self::Budget(error) => write!(f, "the public surface is over budget: {error}"),
            Self::CarryBudget { path, error } => write!(
                f,
                "deciding whether the public globs carry the re-export `{}` is over budget: {error}",
                Esc(path)
            ),
            Self::Resolve(error) => write!(f, "a public name does not resolve: {error}"),
        }
    }
}

impl Error for ExportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NotLibraryOrBinary(_) => None,
            Self::Budget(error) | Self::CarryBudget { error, .. } => Some(error),
            Self::Resolve(error) => Some(error),
        }
    }
}

/// A view that the public globs of a module read: the module it exports, the
/// first public glob that reads it, and whether every public glob that reads
/// it is `#[doc(hidden)]`.
#[derive(Debug, Clone, Copy)]
struct Route {
    view: usize,
    target: usize,
    glob: usize,
    hidden: bool,
}

/// The public routes of a module, with the routes whose view holds each
/// name, so a name's routes are found without reading every route.
struct Routes<'a> {
    list: Vec<Route>,
    by_name: BTreeMap<&'a str, Vec<usize>>,
    /// The views that every glob of the module reads, public or not, that
    /// hold each name, once each: what the module's globs bring in under a
    /// name, read without going through every glob.
    every: BTreeMap<&'a str, Vec<usize>>,
}

impl Routes<'_> {
    /// The routes whose view holds `name`, in order of view.
    fn holding(&self, name: &str) -> impl Iterator<Item = &Route> + '_ {
        self.by_name
            .get(name)
            .into_iter()
            .flatten()
            .map(|at| &self.list[*at])
    }
}

/// What carrying the re-exports of the modules a reader's routes reach works
/// with, for one reader: the reach as a graph, the modules of the reach that
/// bind each name themselves (the reader included, sorted), the modules of
/// the reach that carry each name, and the answers decided by a search, one
/// for each name and carrying module. No search keeps what it walked.
struct Carry<'a> {
    graph: Reach,
    binders: BTreeMap<&'a str, Vec<usize>>,
    carriers: BTreeMap<&'a str, Vec<usize>>,
    answers: BTreeMap<(&'a str, usize, usize), bool>,
}

/// The modules a reader's public routes reach, as a graph: node 0 is the
/// reader, each other node a module of the reach in the order of the walk,
/// with the nodes its routes lead to, once over the routes that
/// `#[doc(hidden)]` does not hide ([`SHOWN`]: every route but those that leave
/// the reader by a hidden glob, as for an item) and once over every route
/// ([`EVERY`]), whether the shown routes reach each node, and, once a carried
/// name needs it, the tree of dominators of each (a node dominates another
/// when every route from the reader to the other passes it), as the interval
/// each node the routes reach spans in a walk of the tree.
struct Reach {
    modules: Vec<usize>,
    node_of: BTreeMap<usize, usize>,
    succ: [Vec<Vec<usize>>; 2],
    shown: Vec<bool>,
    spans: [Option<Spans>; 2],
}

/// The interval of each node in a walk of a tree of dominators, `None` for a
/// node the routes do not reach.
type Spans = Vec<Option<(usize, usize)>>;

/// The routes of a [`Reach`] that `#[doc(hidden)]` does not hide.
const SHOWN: usize = 0;
/// Every route of a [`Reach`].
const EVERY: usize = 1;

impl Reach {
    /// True when, over the routes of `mode`, the node `above` dominates the
    /// node `below`.
    fn dominates(&self, mode: usize, above: usize, below: usize) -> bool {
        match self.spans[mode]
            .as_ref()
            .map(|spans| (spans[above], spans[below]))
        {
            Some((Some(above), Some(below))) => above.0 <= below.0 && below.1 <= above.1,
            _ => false,
        }
    }
}

/// The interval of each node that node 0 reaches in a walk of the tree of
/// dominators of `succ` (`None` for a node it does not reach), by the
/// iterative algorithm of Cooper,
/// Harvey and Kennedy over a reverse postorder, and the work it did: a unit
/// for each node and each edge each pass reads, and for each step of the
/// intersections.
fn dominator_spans(succ: &[Vec<usize>]) -> (Spans, usize) {
    let count = succ.len();
    let mut work = 0;
    // A postorder from node 0, on a stack of its own.
    let mut post = vec![usize::MAX; count];
    let mut order = Vec::with_capacity(count);
    let mut stack = vec![(0, 0)];
    let mut entered = vec![false; count];
    entered[0] = true;
    while let Some((node, next)) = stack.pop() {
        work += 1;
        if let Some(&to) = succ[node].get(next) {
            stack.push((node, next + 1));
            if !entered[to] {
                entered[to] = true;
                stack.push((to, 0));
            }
        } else {
            post[node] = order.len();
            order.push(node);
        }
    }
    let mut preds: Vec<Vec<usize>> = vec![Vec::new(); count];
    for (from, tos) in succ.iter().enumerate() {
        if post[from] == usize::MAX {
            continue;
        }
        for &to in tos {
            preds[to].push(from);
        }
    }
    let mut idom = vec![usize::MAX; count];
    idom[0] = 0;
    let mut changed = true;
    while changed {
        changed = false;
        for &node in order.iter().rev().skip(1) {
            let mut new = usize::MAX;
            for &pred in &preds[node] {
                work += 1;
                if idom[pred] == usize::MAX {
                    continue;
                }
                if new == usize::MAX {
                    new = pred;
                    continue;
                }
                let (mut a, mut b) = (pred, new);
                while a != b {
                    while post[a] < post[b] {
                        a = idom[a];
                        work += 1;
                    }
                    while post[b] < post[a] {
                        b = idom[b];
                        work += 1;
                    }
                }
                new = a;
            }
            if idom[node] != new {
                idom[node] = new;
                changed = true;
            }
        }
    }
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); count];
    for node in 1..count {
        if post[node] != usize::MAX {
            children[idom[node]].push(node);
        }
    }
    let mut spans: Spans = vec![None; count];
    let mut clock = 0;
    let mut stack = vec![(0, 0)];
    while let Some((node, next)) = stack.pop() {
        work += 1;
        if next == 0 {
            spans[node] = Some((clock, clock));
            clock += 1;
        }
        if let Some(&child) = children[node].get(next) {
            stack.push((node, next + 1));
            stack.push((child, 0));
        } else if let Some(span) = spans[node].as_mut() {
            span.1 = clock;
            clock += 1;
        }
    }
    (spans, work)
}

/// What a step of a walk of the reach is charged: the binding it keeps of
/// the module it passes, an upper bound of what one step holds, so the work
/// of the walks is bounded with their memory by the item-bytes budget.
const WALK_STEP_BYTES: usize = btree_entry::<usize, bool>();

/// A binding the search for a public route reads: the item, and the name in a
/// module in the name space at a slot that binds it.
type Place = (ItemId, usize, usize, String);

/// A module waiting to be read, at the first path the search reached it by.
struct Visit {
    module: usize,
    path: String,
    hidden: bool,
}

/// What a binding of another crate names: an item the index sees (an item of
/// another crate of the workspace, or that crate's root), or a crate outside
/// the workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Other {
    Seen(ItemId),
    Outside,
}

impl Other {
    fn item(self) -> Option<ItemId> {
        match self {
            Self::Seen(id) => Some(id),
            Self::Outside => None,
        }
    }
}

/// What a public binding is bound by in its module: a definition or an
/// explicit import, or the module's globs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum By {
    Explicit,
    Glob,
}

/// What one listing of a surface works with.
pub(crate) struct Surface<'a> {
    index: &'a Index,
    target: TargetId,
    root: usize,
    crate_path: &'a str,
    options: &'a ExportOptions,
    meter: &'a Meter,
    /// The module paths of the target whose file has `#![doc(hidden)]`.
    hidden_files: &'a BTreeSet<&'a str>,
    /// The public routes of each module the search has read.
    routes: BTreeMap<usize, Rc<Routes<'a>>>,
    /// Whether each place a search for a public route decided has one.
    decided: BTreeMap<Place, bool>,
    items: Vec<ExportedItem>,
    reexports: Vec<Reexport>,
}

/// The path a `use` declaration names, as written.
fn written(decl: &UseDecl) -> String {
    let (segs, glob) = match &decl.kind {
        UseKind::Named { segs, .. } => (segs, false),
        UseKind::Glob { segs } => (segs, true),
    };
    let mut text = String::new();
    if decl.leading_colon {
        text.push_str("::");
    }
    text.push_str(&segs.join("::"));
    if glob {
        text.push_str("::*");
    }
    text
}

/// True when the `use` declaration can carry a name out of its crate: it is
/// `pub` and not test code.
fn exporting(decl: &UseDecl) -> bool {
    decl.vis == Vis::Public && !decl.test
}

impl<'a> Surface<'a> {
    /// A listing of the surface of `target`, a library or a binary whose
    /// crate root is the module `root`.
    #[allow(
        clippy::too_many_arguments,
        reason = "a listing is set up from the index, the target and the caller's budget and options"
    )]
    pub(crate) fn new(
        index: &'a Index,
        target: TargetId,
        crate_path: &'a str,
        options: &'a ExportOptions,
        meter: &'a Meter,
        hidden_files: &'a BTreeSet<&'a str>,
    ) -> Self {
        Self {
            index,
            target,
            root: index.target_roots[target.index()],
            crate_path,
            options,
            meter,
            hidden_files,
            routes: BTreeMap::new(),
            decided: BTreeMap::new(),
            items: Vec::new(),
            reexports: Vec::new(),
        }
    }

    /// Charge `bytes` held for the item `id`.
    fn charge(&self, bytes: usize, id: ItemId) -> Result<(), ExportError> {
        let item = &self.index.items[id.0];
        self.meter
            .items(bytes, &item.file, Some(item.name_line))
            .map_err(ExportError::Budget)
    }

    /// The item that stands for `module`: its crate root or its `mod` item.
    fn module_item(&self, module: usize) -> ItemId {
        self.index
            .module_items(module)
            .first()
            .copied()
            .unwrap_or_else(|| self.index.root_items[&self.index.modules[module].root])
    }

    /// List the surface: every module a public binding reaches, read once at
    /// the first path that reaches it, breadth first, a path that
    /// `#[doc(hidden)]` does not hide before one that it does.
    pub(crate) fn list(mut self) -> Result<Exports, ExportError> {
        let index = self.index;
        let root_item = index.root_items[&self.root];
        self.charge(
            vec_first::<ExportedItem>()
                + vec_first::<Reexport>()
                + 2 * vec_first::<Visit>()
                + index.modules.len() * size_of::<bool>()
                + btree_map::<usize, Vec<Route>>(),
            root_item,
        )?;
        let mut visited = vec![false; index.modules.len()];
        let mut shown: VecDeque<Visit> = VecDeque::new();
        let mut hidden: VecDeque<Visit> = VecDeque::new();
        let start = Visit {
            module: self.root,
            path: self.crate_path.to_owned(),
            hidden: index.items[root_item.0].doc_hidden,
        };
        self.charge(vec_slot::<Visit>() + start.path.capacity(), root_item)?;
        if start.hidden {
            if self.options.include_hidden {
                hidden.push_back(start);
            }
        } else {
            visited[self.root] = true;
            shown.push_back(start);
        }
        loop {
            let visit = match shown.pop_front() {
                Some(visit) => visit,
                None => match hidden.pop_front() {
                    Some(visit) if visited[visit.module] => continue,
                    Some(visit) => {
                        visited[visit.module] = true;
                        visit
                    }
                    None => break,
                },
            };
            let found = self.module(&visit)?;
            for next in found {
                if visited[next.module] {
                    continue;
                }
                self.charge(
                    vec_slot::<Visit>() + next.path.capacity(),
                    self.module_item(next.module),
                )?;
                if next.hidden {
                    hidden.push_back(next);
                } else {
                    visited[next.module] = true;
                    shown.push_back(next);
                }
            }
        }
        let Self {
            mut items,
            mut reexports,
            ..
        } = self;
        items.sort_by(|a, b| {
            (a.path.as_str(), index.sort_key(a.item))
                .cmp(&(b.path.as_str(), index.sort_key(b.item)))
        });
        reexports.sort_by(|a, b| {
            (&a.path, &a.file, a.line, &a.written, a.item)
                .cmp(&(&b.path, &b.file, b.line, &b.written, b.item))
        });
        Ok(Exports { items, reexports })
    }

    /// The public routes of `module`: the views its `pub` globs that are not
    /// test code read, once each, in order of view, and for each name the
    /// routes whose view holds it. Made and charged once for the listing.
    fn routes(&mut self, module: usize) -> Result<Rc<Routes<'a>>, ExportError> {
        if let Some(known) = self.routes.get(&module) {
            return Ok(Rc::clone(known));
        }
        let index = self.index;
        let mut by_view: BTreeMap<usize, Route> = BTreeMap::new();
        for (glob, target, view) in index.glob_reads(module) {
            let decl = &index.uses[glob];
            if !exporting(decl) {
                continue;
            }
            by_view
                .entry(view)
                .and_modify(|route| {
                    route.glob = route.glob.min(glob);
                    route.hidden &= decl.doc_hidden;
                })
                .or_insert(Route {
                    view,
                    target,
                    glob,
                    hidden: decl.doc_hidden,
                });
        }
        let list: Vec<Route> = by_view.into_values().collect();
        let mut held = 0;
        let mut views: Vec<usize> = index.glob_reads(module).map(|(_, _, view)| view).collect();
        views.sort_unstable();
        views.dedup();
        let mut every: BTreeMap<&'a str, Vec<usize>> = BTreeMap::new();
        for &view in &views {
            for name in index.view_names(view) {
                every.entry(name).or_default().push(view);
                held += 1;
            }
        }
        let mut by_name: BTreeMap<&'a str, Vec<usize>> = BTreeMap::new();
        for (at, route) in list.iter().enumerate() {
            for name in index.view_names(route.view) {
                by_name.entry(name).or_default().push(at);
                held += 1;
            }
        }
        self.charge(
            btree_entry::<usize, Rc<Routes<'a>>>()
                + size_of::<Routes<'a>>()
                + 2 * size_of::<usize>()
                + list.capacity() * size_of::<Route>()
                + btree_map::<&str, Vec<usize>>()
                + 2 * btree_map::<&str, Vec<usize>>()
                + (by_name.len() + every.len())
                    * (btree_entry::<&str, Vec<usize>>() + vec_first::<usize>())
                + held * vec_slot::<usize>()
                + views.capacity() * size_of::<usize>(),
            self.module_item(module),
        )?;
        let routes = Rc::new(Routes {
            list,
            by_name,
            every,
        });
        self.routes.insert(module, Rc::clone(&routes));
        Ok(routes)
    }

    /// The modules that the public routes of `start` reach, directly or through
    /// the public routes of the modules they reach, each once, with whether
    /// `#[doc(hidden)]` hides every such route, as it hides an item: a route
    /// is hidden when the glob of `start` it leaves by is, whatever the
    /// globs after it, and a module that a shown route reaches is shown. Each module the walk passes, in either of its two
    /// passes, is charged a binding of the walk, so the walks of a listing
    /// are bounded by the item-bytes budget with what they hold.
    fn glob_reach(&mut self, start: usize) -> Result<Vec<(usize, bool)>, ExportError> {
        let mut seen: BTreeMap<usize, bool> = BTreeMap::new();
        let mut order = Vec::new();
        let at = self.module_item(start);
        // Shown routes first, then every route, so a module keeps the first
        // shown route that reaches it.
        for with_hidden in [false, true] {
            let mut work = VecDeque::from([(start, false)]);
            let mut entered = BTreeSet::from([start]);
            while let Some((module, hidden)) = work.pop_front() {
                tally::add(Work::WalkSteps, 1);
                self.charge(WALK_STEP_BYTES, at)?;
                if module != start
                    && let std::collections::btree_map::Entry::Vacant(slot) = seen.entry(module)
                {
                    slot.insert(hidden);
                    order.push(module);
                }
                for route in &self.routes(module)?.list {
                    // As for an item, only the reader's own globs hide what
                    // they bring in: a hidden glob further on does not.
                    let shown = !route.hidden || module != start;
                    if (with_hidden || shown) && entered.insert(route.target) {
                        work.push_back((route.target, hidden || !shown));
                    }
                }
            }
        }
        Ok(order
            .into_iter()
            .map(|module| (module, seen[&module]))
            .collect())
    }

    /// True when the item `id` is declared so that another crate can name
    /// it: `pub`, or `#[macro_export]` for a macro. That it is not test code
    /// is checked before its route is searched.
    fn declared_public(&self, id: ItemId) -> bool {
        let item = &self.index.items[id.0];
        match item.kind {
            SymbolKind::MacroRules => item.macro_export,
            _ => item.visibility == Vis::Public,
        }
    }

    /// The `pub` named imports of `name` in `module` that are not test code
    /// and bound something in the name space at `at`, in order of position.
    fn exporting_imports(&self, module: usize, name: &str, at: usize) -> Vec<usize> {
        let index = self.index;
        index
            .imports_of(module, name)
            .iter()
            .copied()
            .filter(|position| index.use_bound[*position][at] && exporting(&index.uses[*position]))
            .collect()
    }

    /// True when the binding of `id` under `name` in `module`, in the name
    /// space at `at`, has a public route: every `use` declaration from the
    /// binding back to the definition is `pub` and not test code, and the
    /// definition is public.
    ///
    /// The search keeps a stack and the places it has seen, each with the
    /// place it was reached from, of its own, so a cycle of globs ends it and
    /// its depth costs the caller's stack nothing. What it decides is kept for
    /// the listing: the places on the route it found have a public route, and
    /// when it finds none, no place it saw has one, as it saw every place they
    /// lead to. So a chain of re-exports is walked once for all its names.
    fn public_route(
        &mut self,
        module: usize,
        at: usize,
        name: &str,
        id: ItemId,
    ) -> Result<bool, ExportError> {
        let index = self.index;
        let start: Place = (id, module, at, name.to_owned());
        let mut from: BTreeMap<Place, Option<Place>> = BTreeMap::new();
        let mut work: Vec<(Place, Option<Place>)> = vec![(start, None)];
        let mut found = None;
        while let Some((place, parent)) = work.pop() {
            if from.contains_key(&place) {
                continue;
            }
            from.insert(place.clone(), parent);
            match self.decided.get(&place) {
                Some(true) => {
                    found = Some(place);
                    break;
                }
                Some(false) => continue,
                None => {}
            }
            let (_, module, at, name) = &place;
            let (module, at) = (*module, *at);
            let defs = index.scopes[module]
                .get(name)
                .map(|entry| &entry.defs[at])
                .filter(|defs| !defs.is_empty());
            if let Some(defs) = defs {
                let Some(origin) = defs.get(id) else {
                    continue;
                };
                let next = match &origin.link {
                    None => None,
                    Some(_) if self.exporting_imports(module, name, at).is_empty() => continue,
                    // A path that ends at a module and not at a name in it
                    // binds the module's item, as a definition does.
                    Some(link) => link.source.as_ref(),
                };
                match next {
                    Some(source) => {
                        let step = (id, source.module, slot(source.ns), source.name.clone());
                        work.push((step, Some(place)));
                    }
                    None if self.declared_public(id) => {
                        found = Some(place);
                        break;
                    }
                    None => {}
                }
            } else if !index.blocked(module, name, Namespace::ALL[at], None) {
                let routes = self.routes(module)?;
                for route in routes.holding(name) {
                    if index.view_holds(route.view, name, at, id) {
                        let step = (id, route.target, at, name.clone());
                        work.push((step, Some(place.clone())));
                    }
                }
            }
        }
        let decided: Vec<(Place, bool)> = match found {
            Some(mut place) => {
                let mut route = Vec::new();
                loop {
                    let parent = from.get(&place).cloned().flatten();
                    route.push((place, true));
                    match parent {
                        Some(next) => place = next,
                        None => break,
                    }
                }
                route
            }
            None => from.into_keys().map(|place| (place, false)).collect(),
        };
        let reached = !decided.is_empty() && decided[0].1;
        for (place, public) in decided {
            if let std::collections::btree_map::Entry::Vacant(slot) = self.decided.entry(place) {
                self.meter
                    .items(
                        btree_entry::<Place, bool>() + slot.key().3.capacity(),
                        &index.items[id.0].file,
                        Some(index.items[id.0].name_line),
                    )
                    .map_err(ExportError::Budget)?;
                slot.insert(public);
            }
        }
        Ok(reached)
    }

    /// The public routes of `module` that carry `id` under `name` in the name
    /// space at `at` out of the crate: those whose view holds it and whose
    /// target binds it with a public route of its own (see
    /// [`Surface::public_route`]). A glob whose view holds the item but whose
    /// target keeps it in, behind a binding that is not `pub` or an import
    /// that shadows it, does not export it, whatever its visibility.
    fn carrying_routes(
        &mut self,
        module: usize,
        at: usize,
        name: &str,
        id: ItemId,
    ) -> Result<Vec<Route>, ExportError> {
        let index = self.index;
        let routes = self.routes(module)?;
        let mut carrying = Vec::new();
        for route in routes.holding(name) {
            if index.view_holds(route.view, name, at, id)
                && self.public_route(route.target, at, name, id)?
            {
                carrying.push(*route);
            }
        }
        Ok(carrying)
    }

    /// True when the `use` declarations that export `id` under `name` in
    /// `module` are all `#[doc(hidden)]`.
    fn reexport_hidden(
        &mut self,
        module: usize,
        at: usize,
        name: &str,
        id: ItemId,
        by: By,
    ) -> Result<bool, ExportError> {
        let index = self.index;
        Ok(match by {
            By::Explicit => {
                let imported = index.scopes[module]
                    .get(name)
                    .and_then(|entry| entry.defs[at].get(id))
                    .is_some_and(|origin| origin.link.is_some());
                let imports = self.exporting_imports(module, name, at);
                imported
                    && !imports.is_empty()
                    && imports
                        .iter()
                        .all(|position| index.uses[*position].doc_hidden)
            }
            By::Glob => {
                let carrying = self.carrying_routes(module, at, name, id)?;
                !carrying.is_empty() && carrying.iter().all(|route| route.hidden)
            }
        })
    }

    /// The declaration that exports `id` under `name` in `module`, as written,
    /// with its file and line: the first `pub` import of the name, the first
    /// `pub` glob that brings the item in, or the item itself for an
    /// `extern crate` item.
    fn binding_site(
        &mut self,
        module: usize,
        at: usize,
        name: &str,
        id: ItemId,
        by: By,
    ) -> Result<(String, RelPath, usize), ExportError> {
        let index = self.index;
        let decl = match by {
            By::Explicit => self
                .exporting_imports(module, name, at)
                .first()
                .map(|position| &index.uses[*position]),
            By::Glob => self
                .carrying_routes(module, at, name, id)?
                .iter()
                .map(|route| route.glob)
                .min()
                .map(|glob| &index.uses[glob]),
        };
        if let Some(decl) = decl {
            return Ok((written(decl), decl.file.clone(), decl.line));
        }
        let item = &index.items[id.0];
        let crate_name = index
            .extern_names
            .get(&id)
            .map_or_else(|| item.name.to_string(), ToString::to_string);
        Ok((crate_name, item.file.clone(), item.name_line))
    }

    /// `Some` when the binding of `id` names another crate: an item of
    /// another target, or an `extern crate` item of a crate other than this
    /// one, which stands for that crate's root when the index sees it.
    fn other_crate(&self, id: ItemId) -> Option<Other> {
        let index = self.index;
        let item = &index.items[id.0];
        if item.target != self.target {
            return Some(Other::Seen(id));
        }
        if item.kind == SymbolKind::ExternCrate {
            return match index.crate_of.get(&id) {
                None => Some(Other::Outside),
                Some(roots) if roots.contains(&self.root) => None,
                Some(roots) => Some(
                    roots
                        .first()
                        .and_then(|root| index.root_items.get(root).copied())
                        .map_or(Other::Outside, Other::Seen),
                ),
            };
        }
        None
    }

    /// Keep an item of the surface, unless `#[doc(hidden)]` hides it and the
    /// options leave hidden items out.
    fn keep(&mut self, path: String, id: ItemId, doc_hidden: bool) -> Result<(), ExportError> {
        if doc_hidden && !self.options.include_hidden {
            return Ok(());
        }
        self.charge(vec_slot::<ExportedItem>() + path.capacity(), id)?;
        let item = &self.index.items[id.0];
        self.items.push(ExportedItem {
            path,
            item: id,
            kind: item.kind,
            file: item.file.clone(),
            span: item.span.clone(),
            doc_hidden,
        });
        Ok(())
    }

    /// Keep a re-export, unless `#[doc(hidden)]` hides it and the options
    /// leave hidden items out.
    #[allow(
        clippy::too_many_arguments,
        reason = "a re-export is built from its path, what it names and its declaration"
    )]
    fn keep_reexport(
        &mut self,
        path: String,
        written: String,
        item: Option<ItemId>,
        file: RelPath,
        line: usize,
        doc_hidden: bool,
    ) -> Result<(), ExportError> {
        if doc_hidden && !self.options.include_hidden {
            return Ok(());
        }
        self.meter
            .items(
                vec_slot::<Reexport>() + path.capacity() + written.capacity(),
                &file,
                Some(line),
            )
            .map_err(ExportError::Budget)?;
        self.reexports.push(Reexport {
            path,
            written,
            item,
            file,
            line,
            doc_hidden,
        });
        Ok(())
    }

    /// Read one module of the surface: its public re-exports of what the
    /// index cannot see, and each public binding of each of its names.
    /// Returns the modules of the target its bindings reach.
    fn module(&mut self, visit: &Visit) -> Result<Vec<Visit>, ExportError> {
        let index = self.index;
        let (module, path) = (visit.module, visit.path.as_str());
        self.unseen(visit)?;
        let routes = self.routes(module)?;
        let mut names: BTreeSet<&'a str> =
            index.scopes[module].keys().map(String::as_str).collect();
        names.extend(routes.by_name.keys().copied());
        self.charge(
            btree_map::<&str, ()>() + names.len() * btree_entry::<&str, ()>(),
            self.module_item(module),
        )?;
        let mut next = Vec::new();
        for name in names {
            // The items the name exports, each with the first name space it is
            // exported in and what binds it there.
            let mut found: BTreeMap<ItemId, (usize, By)> = BTreeMap::new();
            for ns in Namespace::ALL {
                let at = slot(ns);
                let Some((ids, from_glob)) = Self::bound(index, &routes, module, ns, name) else {
                    continue;
                };
                let mut exported = false;
                for &id in &ids {
                    if index.items[id.0].test == TestRole::None
                        && self.public_route(module, at, name, id)?
                    {
                        exported = true;
                        found
                            .entry(id)
                            .or_insert((at, if from_glob { By::Glob } else { By::Explicit }));
                    }
                }
                if exported
                    && let Some(error) =
                        index.binding_failure(module, ns, name, &ids, &format!("{path}::{name}"))
                {
                    return Err(ExportError::Resolve(error));
                }
            }
            for id in index.sorted(found.keys().copied()) {
                let (at, by) = found[&id];
                if let Some(child) = self.binding(visit, name, id, at, by)? {
                    next.push(child);
                }
            }
        }
        Ok(next)
    }

    /// The public re-exports of the module of `visit` of what the index cannot
    /// see, its own and those its public glob routes carry from the modules
    /// they reach, each under the path of `visit`.
    ///
    /// The modules the routes reach are walked once, into a graph. A carried
    /// import of a name is carried when some route from the reader to its
    /// module passes no other module that binds the name itself (see
    /// [`Surface::carried`]).
    fn unseen(&mut self, visit: &Visit) -> Result<(), ExportError> {
        let index = self.index;
        self.unseen_globs(visit, visit.module, false)?;
        self.unbound_imports(visit, visit.module, false, None)?;
        let reach = self.glob_reach(visit.module)?;
        if reach.is_empty() {
            return Ok(());
        }
        let mut carry = Carry {
            graph: self.reach_graph(visit.module, &reach)?,
            binders: BTreeMap::new(),
            carriers: BTreeMap::new(),
            answers: BTreeMap::new(),
        };
        for (node, &module) in carry.graph.modules.iter().enumerate() {
            let defined = index.scopes[module]
                .iter()
                .filter(|(_, entry)| entry.defs.iter().any(|bound| !bound.is_empty()))
                .map(|(name, _)| name.as_str());
            let mut imported: Vec<&'a str> = Vec::new();
            for position in index.named_of(module) {
                let decl = &index.uses[*position];
                if let UseKind::Named { name, .. } = &decl.kind {
                    imported.push(name.as_str());
                    if node > 0
                        && exporting(decl)
                        && !index.use_bound[*position].iter().any(|bound| *bound)
                    {
                        let carriers = carry.carriers.entry(name.as_str()).or_default();
                        if carriers.last() != Some(&node) {
                            carriers.push(node);
                        }
                    }
                }
            }
            for name in defined.chain(imported) {
                let binders = carry.binders.entry(name).or_default();
                if binders.last() != Some(&node) {
                    binders.push(node);
                }
            }
        }
        let held: usize = carry
            .binders
            .values()
            .chain(carry.carriers.values())
            .map(Vec::len)
            .sum();
        self.charge(
            2 * btree_map::<&str, Vec<usize>>()
                + (carry.binders.len() + carry.carriers.len())
                    * (btree_entry::<&str, Vec<usize>>() + vec_first::<usize>())
                + held * vec_slot::<usize>(),
            self.module_item(visit.module),
        )?;
        for (module, hidden) in reach {
            self.unseen_globs(visit, module, hidden)?;
            self.unbound_imports(visit, module, hidden, Some(&mut carry))?;
        }
        Ok(())
    }

    /// The graph of `reach`, the modules the public routes of `reader` reach,
    /// charged as it is kept.
    fn reach_graph(
        &mut self,
        reader: usize,
        reach: &[(usize, bool)],
    ) -> Result<Reach, ExportError> {
        let modules: Vec<usize> = std::iter::once(reader)
            .chain(reach.iter().map(|(module, _)| *module))
            .collect();
        let node_of: BTreeMap<usize, usize> = modules
            .iter()
            .enumerate()
            .map(|(node, module)| (*module, node))
            .collect();
        let shown: Vec<bool> = std::iter::once(true)
            .chain(reach.iter().map(|(_, hidden)| !hidden))
            .collect();
        let mut succ: [Vec<Vec<usize>>; 2] = [
            Vec::with_capacity(modules.len()),
            Vec::with_capacity(modules.len()),
        ];
        let mut edges = 0;
        for (node, &module) in modules.iter().enumerate() {
            let routes = self.routes(module)?;
            for (mode, list) in succ.iter_mut().enumerate() {
                // As for an item, only the reader's own hidden globs are left
                // out of the shown routes.
                let mut to: Vec<usize> = routes
                    .list
                    .iter()
                    .filter(|route| mode == EVERY || node != 0 || !route.hidden)
                    .filter_map(|route| node_of.get(&route.target).copied())
                    .collect();
                to.sort_unstable();
                to.dedup();
                edges += to.len();
                list.push(to);
            }
        }
        self.charge(
            modules.len()
                * (size_of::<usize>()
                    + size_of::<bool>()
                    + btree_entry::<usize, usize>()
                    + 2 * size_of::<Vec<usize>>())
                + btree_map::<usize, usize>()
                + edges * size_of::<usize>(),
            self.module_item(reader),
        )?;
        Ok(Reach {
            modules,
            node_of,
            succ,
            shown,
            spans: [None, None],
        })
    }

    /// Whether the public routes of the module of `visit` carry the name
    /// `name` of the import at `position` from `module`, which the walk
    /// reached: `Some(false)` when a route that `#[doc(hidden)]` does not
    /// hide carries it, `Some(true)` when only hidden routes do, and `None`
    /// when none does. A route carries it when it passes no module that binds
    /// the name itself, the reader included, but the module.
    fn carried(
        &mut self,
        visit: &Visit,
        position: usize,
        module: usize,
        name: &'a str,
        carry: &mut Carry<'a>,
    ) -> Result<Option<bool>, ExportError> {
        if self.carried_by(visit, position, module, name, carry, SHOWN)? {
            return Ok(Some(false));
        }
        Ok(self
            .carried_by(visit, position, module, name, carry, EVERY)?
            .then_some(true))
    }

    /// True when a route of `mode` carries the name `name` from `module` to
    /// the reader of `visit` (see [`Surface::carried`]).
    ///
    /// Only a binder that the routes of `mode` reach and that has routes of
    /// its own there can be passed on the way. With none, the name is
    /// carried. One that dominates the module (is on every such route to it)
    /// keeps the name out, and so does the reader; with one binder that does
    /// not, the name is carried. With several, none of which dominates, a
    /// search from the reader past the binders decides the name for every
    /// module that carries it at once, and keeps only those answers. The tree
    /// of dominators of each mode is made once for the reader, the first
    /// time a name needs it. The work of the tree and of each search, a unit
    /// for each module and route it reads, is charged to the item-bytes
    /// budget with what they hold; a listing that passes the budget so is
    /// [`ExportError::CarryBudget`], naming the re-export and its
    /// declaration.
    #[allow(
        clippy::too_many_arguments,
        reason = "a decision is made for one import of one module, for one reader and one mode"
    )]
    fn carried_by(
        &mut self,
        visit: &Visit,
        position: usize,
        module: usize,
        name: &'a str,
        carry: &mut Carry<'a>,
        mode: usize,
    ) -> Result<bool, ExportError> {
        let decl = &self.index.uses[position];
        let charge = |surface: &Self, bytes: usize| {
            surface
                .meter
                .items(bytes, &decl.file, Some(decl.line))
                .map_err(|error| ExportError::CarryBudget {
                    path: format!("{}::{name}", visit.path),
                    error,
                })
        };
        let target = carry.graph.node_of[&module];
        if mode == SHOWN && !carry.graph.shown[target] {
            return Ok(false);
        }
        let Some(binders) = carry.binders.get(name) else {
            return Ok(true);
        };
        if binders.first() == Some(&0) {
            return Ok(false);
        }
        let succ = &carry.graph.succ[mode];
        let passed: Vec<usize> = binders
            .iter()
            .copied()
            .filter(|node| {
                *node != target
                    && !succ[*node].is_empty()
                    && (mode == EVERY || carry.graph.shown[*node])
            })
            .collect();
        if passed.is_empty() {
            return Ok(true);
        }
        if carry.graph.spans[mode].is_none() {
            let (spans, work) = dominator_spans(succ);
            let edges: usize = succ.iter().map(Vec::len).sum();
            tally::add(Work::WalkSteps, work);
            charge(
                self,
                work * WALK_STEP_BYTES
                    + spans.len() * (size_of::<Option<(usize, usize)>>() + 6 * size_of::<usize>())
                    + edges * 2 * size_of::<usize>(),
            )?;
            carry.graph.spans[mode] = Some(spans);
        }
        if passed
            .iter()
            .any(|binder| carry.graph.dominates(mode, *binder, target))
        {
            return Ok(false);
        }
        if passed.len() == 1 {
            return Ok(true);
        }
        if let Some(known) = carry.answers.get(&(name, target, mode)) {
            return Ok(*known);
        }
        // One search for the name: from the reader, never past a module that
        // binds it, noting each module that carries it.
        let succ = &carry.graph.succ[mode];
        let count = carry.graph.modules.len();
        let mut entered = vec![false; count];
        entered[0] = true;
        let mut work = vec![0];
        let mut reached = BTreeSet::new();
        let mut steps = 0;
        while let Some(node) = work.pop() {
            steps += 1 + succ[node].len();
            if node != 0 {
                reached.insert(node);
                if binders.binary_search(&node).is_ok() {
                    continue;
                }
            }
            for &to in &succ[node] {
                if !entered[to] {
                    entered[to] = true;
                    work.push(to);
                }
            }
        }
        let carriers = carry.carriers.get(name).map_or(&[][..], Vec::as_slice);
        tally::add(Work::WalkSteps, steps);
        charge(
            self,
            steps * WALK_STEP_BYTES
                + count * size_of::<bool>()
                + carriers.len() * btree_entry::<(&str, usize, usize), bool>(),
        )?;
        for &carrier in carriers {
            carry
                .answers
                .insert((name, carrier, mode), reached.contains(&carrier));
        }
        Ok(reached.contains(&target))
    }

    /// The items `module` binds to `name` in `ns`, and whether its globs
    /// bring them in, as [`Index::lookup_in`] answers: the bindings that
    /// shadow globs when there are any, else, when no explicit import keeps
    /// the name from the globs, what every glob of the module brings in,
    /// read through the views that hold the name.
    fn bound(
        index: &Index,
        routes: &Routes<'_>,
        module: usize,
        ns: Namespace,
        name: &str,
    ) -> Option<(Vec<ItemId>, bool)> {
        let at = slot(ns);
        if let Some(defs) = index.scopes[module]
            .get(name)
            .map(|entry| &entry.defs[at])
            .filter(|defs| !defs.is_empty())
        {
            return Some((defs.keys().copied().collect(), false));
        }
        if index.blocked(module, name, ns, None) {
            return None;
        }
        let mut ids: Vec<ItemId> = routes
            .every
            .get(name)
            .into_iter()
            .flatten()
            .flat_map(|view| index.view_ids(*view, name, at))
            .collect();
        ids.sort_unstable();
        ids.dedup();
        (!ids.is_empty()).then_some((ids, true))
    }

    /// The public globs of `module`, read under the path of `visit` (its own
    /// module, or one its public glob routes reach, which `hidden` routes
    /// alone reach), whose path names nothing the index can look into: each
    /// is a re-export. One whose path names nothing in the workspace past its
    /// first segment fails the listing, as it fails every lookup that reads
    /// it, and so does one whose path rustc refuses.
    fn unseen_globs(
        &mut self,
        visit: &Visit,
        module: usize,
        hidden: bool,
    ) -> Result<(), ExportError> {
        let index = self.index;
        for &glob in index.globs_of(module) {
            let decl = &index.uses[glob];
            if !exporting(decl) {
                continue;
            }
            let path = format!("{}::*", visit.path);
            match index.glob_targets(glob) {
                Ok(_) => {}
                Err(Fail::Missing(_)) => {
                    if index.glob_broken(glob) {
                        return Err(ExportError::Resolve(index.glob_error(glob, &path)));
                    }
                    self.keep_reexport(
                        path,
                        written(decl),
                        None,
                        decl.file.clone(),
                        decl.line,
                        visit.hidden || hidden || decl.doc_hidden,
                    )?;
                }
                Err(Fail::SeveralCrates { roots: ids, .. } | Fail::Ambiguous(ids)) => {
                    return Err(ExportError::Resolve(ResolveError::Ambiguous(ids)));
                }
                // The load refuses a `super` above the crate root.
                Err(Fail::SuperAboveRoot) => {
                    return Err(ExportError::Resolve(ResolveError::Invalid(path)));
                }
            }
        }
        Ok(())
    }

    /// The public named imports of `module`, read under the path of `visit`
    /// as [`Surface::unseen_globs`] reads its globs, that bound nothing: a
    /// re-export of a crate outside the workspace, of one of its items, of an
    /// enum variant or of a path the index cannot follow, unless the module's
    /// globs answer for the name, or, for a module a glob route reaches, a
    /// module on every such route binds the name itself. A name that
    /// resolves to an error fails the listing.
    fn unbound_imports(
        &mut self,
        visit: &Visit,
        module: usize,
        hidden: bool,
        mut carry: Option<&mut Carry<'a>>,
    ) -> Result<(), ExportError> {
        let index = self.index;
        let at_path = &index.modules[module].path;
        let package = index.targets[index.modules[module].target.index()].package_id;
        for &position in index.named_of(module) {
            let decl = &index.uses[position];
            if !exporting(decl) || index.use_bound[position].iter().any(|bound| *bound) {
                continue;
            }
            let UseKind::Named { name, .. } = &decl.kind else {
                continue;
            };
            let hidden = match carry.as_deref_mut() {
                Some(carry) => match self.carried(visit, position, module, name.as_str(), carry)? {
                    Some(hidden) => hidden,
                    None => continue,
                },
                None => hidden,
            };
            match index.resolve(&format!("{at_path}::{name}"), None, Some(package)) {
                Err(ResolveError::NotFound(_)) => self.keep_reexport(
                    format!("{}::{name}", visit.path),
                    written(decl),
                    None,
                    decl.file.clone(),
                    decl.line,
                    visit.hidden || hidden || decl.doc_hidden,
                )?,
                Err(other) => return Err(ExportError::Resolve(other)),
                Ok(_) => {}
            }
        }
        Ok(())
    }

    /// List the public binding of `id` under `name` in the module of `visit`:
    /// a re-export when it names another crate, else the item and, for a type
    /// or a trait, its public members. Returns the module of the target that
    /// a `mod` item stands for, to be read under the binding's path.
    fn binding(
        &mut self,
        visit: &Visit,
        name: &str,
        id: ItemId,
        at: usize,
        by: By,
    ) -> Result<Option<Visit>, ExportError> {
        let index = self.index;
        let module = visit.module;
        let path = format!("{}::{name}", visit.path);
        let item = &index.items[id.0];
        let child = (item.kind == SymbolKind::Mod)
            .then(|| index.module_of.get(&id).copied())
            .flatten();
        let doc_hidden = visit.hidden
            || item.doc_hidden
            || child.is_some_and(|child| {
                self.hidden_files
                    .contains(index.modules[child].path.as_str())
            })
            || self.reexport_hidden(module, at, name, id, by)?;
        if let Some(named) = self.other_crate(id) {
            let (written, file, line) = self.binding_site(module, at, name, id, by)?;
            self.keep_reexport(path, written, named.item(), file, line, doc_hidden)?;
            return Ok(None);
        }
        self.keep(path.clone(), id, doc_hidden)?;
        if doc_hidden && !self.options.include_hidden {
            return Ok(None);
        }
        match item.kind {
            SymbolKind::Struct | SymbolKind::Enum | SymbolKind::Union | SymbolKind::Trait => {
                self.members(&path, id, doc_hidden)?;
                Ok(None)
            }
            _ => Ok(child.map(|child| Visit {
                module: child,
                path,
                hidden: doc_hidden,
            })),
        }
    }

    /// List the members of the type or trait `owner`, exported under
    /// `owner_path`: every member of a trait, and the `pub` members of the
    /// inherent impls of a type. The members of trait impls carry no
    /// visibility of their own and are not listed: their interface is the
    /// trait's.
    fn members(
        &mut self,
        owner_path: &str,
        owner: ItemId,
        hidden: bool,
    ) -> Result<(), ExportError> {
        let index = self.index;
        let Some(table) = index.assoc.get(&owner) else {
            return Ok(());
        };
        let of_trait = index.items[owner.0].kind == SymbolKind::Trait;
        for (name, ids) in table {
            for id in index.sorted(ids.iter().copied()) {
                let member = &index.items[id.0];
                if member.test != TestRole::None
                    || member.target != self.target
                    || member.trait_path.is_some()
                    || (!of_trait && member.visibility != Vis::Public)
                {
                    continue;
                }
                self.keep(
                    format!("{owner_path}::{name}"),
                    id,
                    hidden || member.doc_hidden,
                )?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
