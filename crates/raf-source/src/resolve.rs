// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Name resolution over the settled scope tables.
//!
//! A path is walked segment by segment through the tables built by
//! [`crate::settle`]. Visibility is not checked on a walk: the index answers
//! whether a path names an item, and a private item is still an item. Glob
//! imports are the one place visibility counts, because rustc imports only the
//! names the importing module can name.

use purrdf_lex::allocation::{Memory, OrderedTable, Shared};
use purrdf_lex::walk::WorkList;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use crate::budget::Budget;
use crate::cost::{btree_entry, btree_map, vec_first, vec_slot};
use crate::error::{Esc, LoadError};
use crate::glob_scope::push_work;
use crate::index::{Index, UseKind};
use crate::item::{ItemId, Namespace, SharedStr, SymbolKind};
use crate::meter::{Meter, NativeAdmission, Steps, Transient, over_budget};
use crate::modtree::TargetId;
use crate::path::RelPath;
use crate::settle::{Source, slot};
use crate::target::PackageId;
use crate::work::{self, Work};

mod determinacy;
mod glob_facts;
mod observation;
pub use observation::ResolutionAttempt;

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

/// One segment of a walk: its position in the path, its text, the name space
/// it is read in and whether it is the last segment of the path.
#[derive(Debug, Clone, Copy)]
struct Step<'s> {
    offset: usize,
    seg: &'s str,
    ns: Namespace,
    last: bool,
}

/// Add a route to an item, keeping the better route if it is already there.
fn merge(
    found: &mut BTreeMap<ItemId, Hit>,
    id: ItemId,
    hit: Hit,
    steps: Steps<'_>,
) -> Result<(), LoadError> {
    if !found.contains_key(&id) {
        steps.reserve(
            btree_entry::<ItemId, Hit>()
                + if found.is_empty() {
                    btree_map::<ItemId, Hit>()
                } else {
                    0
                },
        )?;
    }
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
    Ok(())
}

#[derive(Debug)]
pub(crate) enum Fail {
    /// A retained import read was refused before allocating its storage.
    Storage(LoadError),
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
    /// The lookups of a query after the load passed the limit of its work
    /// (see [`Index::query_refusal`]). Only a walk of a query fails so: the
    /// lookups of the load are refused at its checkpoints.
    Budget(Refused),
}

impl From<LoadError> for Fail {
    fn from(error: LoadError) -> Self {
        Self::Storage(error)
    }
}

/// The dependencies, ambiguous items and source reads of one named import.
#[derive(Default)]
struct ImportWalks {
    blockers: Vec<usize>,
    ambiguous: Vec<ItemId>,
    reads: Vec<Source>,
}

/// Why an import-backed place cannot provide a settled binding.
pub(crate) enum Unsettled {
    Cycle,
    LateImport(usize),
}

/// Why a walk of the load is never refused for its work: only a walk of a
/// query counts its lookups on a meter it is refused by, and the lookups of
/// the load are charged at its checkpoints.
pub(crate) const LOAD_WALK: &str = "a walk of the load is never refused for its work";

/// The refusal of a query whose lookups passed the limit of its work.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Refused {
    /// The limit of the work of one query.
    limit: usize,
    /// The module whose lookup passed it.
    module: usize,
}

/// The globs of the settled tables whose paths resolve, with the modules
/// they glob, and those that a lookup reading them fails for (see
/// [`ResolveError::UnresolvedGlob`]).
#[derive(Debug, Default)]
pub(crate) struct GlobFacts {
    reach: Vec<Vec<usize>>,
    broken: OrderedTable<usize, ()>,
}

/// Derived query state, excluded from the equality of the settled index.
///
/// Its facts are a pure function of the other index fields. Their admitted
/// construction finishes before the completed index is published; retaining
/// derived data does not change what a workspace represents.
#[derive(Debug, Clone, Default)]
pub(crate) struct GlobFactsCache(Option<Shared<GlobFacts>>);

impl GlobFactsCache {
    pub(crate) fn get(&self) -> Option<&GlobFacts> {
        self.0.as_deref()
    }

    pub(crate) fn replace(&mut self, facts: Shared<GlobFacts>) {
        self.0 = Some(facts);
    }
}

impl PartialEq for GlobFactsCache {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for GlobFactsCache {}

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
    /// Exact type-alias definitions traversed by an associated-member code query.
    /// These are loaded syntax identities; callers still audit configuration and
    /// unexpanded declaration attributes before using them as execution proof.
    pub type_aliases: Vec<ItemId>,
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
    /// An import initially found an inaccessible binding, then a later
    /// glob route made its path accessible. A settled lookup can read that
    /// path, but the earlier import cannot acquire the late binding.
    IndeterminateImport {
        /// The path the caller tried to resolve.
        path: String,
        /// The source file of the import that could not be determined.
        file: RelPath,
        /// Its one-based source line.
        line: usize,
    },
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
    /// Several loaded packages have the requested display name. Select a
    /// load-local package identity instead.
    AmbiguousPackageName(String),
    /// The package identity does not exist in this workspace.
    UnknownPackageId(PackageId),
    /// The package has only a structural index, without a declared active context.
    UnconfiguredPackage(PackageId),
    /// A resolved site has no admitted active definition context.
    UnconfiguredDefinition(ItemId),
    /// The module named to read a `use` path from is not a module of the
    /// workspace.
    UnknownModule(String),
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
    /// The query passed its lookup/proof work or transient proof-byte limit.
    /// Lookup and proof work share the limit of the work of one query,
    /// [`LoadOptions::max_query_work`](crate::LoadOptions::max_query_work):
    /// the glob reads its lookups of what the globs of a module bring in
    /// looked at, and the views that hold a name looked at to find them,
    /// counted from zero for each query. The answer is not given. This is
    /// the refusal a load gets from the same budget,
    /// [`LoadErrorKind::Budget`](crate::LoadErrorKind::Budget): `budget` is
    /// [`Budget::Work`], [`Budget::ItemBytes`] or [`Budget::GlobBytes`], `limit` the limit that applied, and `file` and
    /// `line` name the declaration of the module whose lookup passed it.
    Budget {
        /// The work, item-byte or glob-byte budget passed.
        budget: Budget,
        /// The limit that applied.
        limit: usize,
        /// The file of the declaration of the module whose lookup passed it.
        file: RelPath,
        /// The line of that declaration, when it has one.
        line: Option<usize>,
    },
    /// A query's native scratch allocation failed for a reason other than
    /// a configured budget. The original failure is retained for diagnosis.
    Storage {
        /// The source file of the binding being checked.
        file: RelPath,
        /// Its source line, when known.
        line: Option<usize>,
        /// The allocation failure.
        what: String,
    },
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnconfiguredPackage(id) => write!(
                f,
                "package {} has no declared configuration; use a configured locked load",
                id.index()
            ),
            Self::UnconfiguredDefinition(id) => write!(
                f,
                "definition {} has no admitted active context",
                id.index()
            ),
            Self::NotFound(path) => write!(f, "`{}` names nothing in the workspace", Esc(path)),
            Self::Ambiguous(ids) => write!(f, "the path names {} different symbols", ids.len()),
            Self::Cyclic(path) => {
                write!(f, "`{}` goes through imports that form a cycle", Esc(path))
            }
            Self::IndeterminateImport { path, file, line } => write!(
                f,
                "`{}` goes through an import at {}:{line} whose binding became accessible after it was determined absent",
                Esc(path),
                Esc(file.as_str())
            ),
            Self::Invalid(text) => write!(f, "`{}` is not a path", Esc(text)),
            Self::UnresolvedSelf(path) => write!(
                f,
                "`{}` goes through the self type of an inherent impl that the index cannot see",
                Esc(path)
            ),
            Self::UnknownPackage(package) => {
                write!(f, "`{}` is not a package of the workspace", Esc(package))
            }
            Self::AmbiguousPackageName(package) => write!(
                f,
                "several packages are named `{}`; select a PackageId from this workspace",
                Esc(package)
            ),
            Self::UnknownPackageId(id) => {
                write!(
                    f,
                    "package identity {} is not in this workspace",
                    id.index()
                )
            }
            Self::UnknownModule(module) => {
                write!(f, "`{}` is not a module of the workspace", Esc(module))
            }
            Self::UnresolvedGlob { path, glob, module } => write!(
                f,
                "`{}` reads the glob `{}::*` of module `{}`, whose path names nothing in the \
                 workspace",
                Esc(path),
                Esc(glob),
                Esc(module)
            ),
            Self::Budget {
                budget,
                limit,
                file,
                line,
            } => write!(
                f,
                "the query is over budget: {}",
                over_budget(*budget, *limit, file, *line)
            ),
            Self::Storage { file, line, what } => write!(
                f,
                "query storage failed at {}{}: {}",
                Esc(file.as_str()),
                line.map_or_else(String::new, |line| format!(":{line}")),
                Esc(what)
            ),
        }
    }
}

impl Error for ResolveError {}

impl ResolveError {
    fn storage(error: LoadError) -> Self {
        let crate::error::LoadErrorKind::Budget { budget, limit } = error.kind else {
            return Self::Storage {
                what: error.kind.to_string(),
                file: error.file,
                line: error.line,
            };
        };
        Self::Budget {
            budget,
            limit,
            file: error.file,
            line: error.line,
        }
    }
}

/// The places a walk read, once each by module and name, in order.
pub(crate) fn places(reads: Vec<Source>) -> BTreeSet<(usize, String)> {
    reads
        .into_iter()
        .map(|source| (source.module, source.name))
        .collect()
}

/// Add the place `name` in `module` in `ns` to the places a walk read, when
/// they are noted.
fn note(
    reads: &mut Option<&mut Vec<Source>>,
    module: usize,
    ns: Namespace,
    name: &str,
    steps: Steps<'_>,
) -> Result<(), Fail> {
    if let Some(list) = reads.as_deref_mut() {
        push_source(list, module, ns, name, steps)?;
    }
    Ok(())
}

fn push_source(
    list: &mut Vec<Source>,
    module: usize,
    ns: Namespace,
    name: &str,
    steps: Steps<'_>,
) -> Result<(), Fail> {
    steps
        .reserve(
            vec_slot::<Source>()
                + name.len()
                + if list.is_empty() {
                    vec_first::<Source>()
                } else {
                    0
                },
        )
        .map_err(Fail::Storage)?;
    list.push(Source {
        module,
        ns,
        name: name.to_owned(),
    });
    Ok(())
}

fn strip_raw(segment: &str) -> &str {
    segment.strip_prefix("r#").map_or(segment, |plain| plain)
}

pub(crate) fn insert_module(
    modules: &mut BTreeSet<usize>,
    module: usize,
    steps: Steps<'_>,
) -> Result<(), LoadError> {
    if !modules.contains(&module) {
        steps.reserve(
            btree_entry::<usize, ()>()
                + if modules.is_empty() {
                    btree_map::<usize, ()>()
                } else {
                    0
                },
        )?;
        modules.insert(module);
    }
    Ok(())
}

fn reserve_ids(count: usize, steps: Steps<'_>) -> Result<(), Fail> {
    steps
        .reserve(vec_first::<ItemId>() + 2 * count * vec_slot::<ItemId>())
        .map_err(Fail::Storage)?;
    steps
        .storage_work(count.saturating_mul(count.max(1).ilog2() as usize + 1))
        .map_err(Fail::Storage)
}

impl Index {
    /// Refuse a step of a walk that would go on in the crate roots of several
    /// crates at once, with the segments from `rest` on.
    pub(crate) fn one_crate(
        &self,
        modules: &BTreeSet<usize>,
        rest: usize,
        steps: Steps<'_>,
    ) -> Result<(), Fail> {
        steps
            .reserve(vec_first::<usize>() + modules.len() * vec_slot::<usize>())
            .map_err(Fail::Storage)?;
        steps.storage_work(modules.len()).map_err(Fail::Storage)?;
        let roots: Vec<usize> = modules
            .iter()
            .copied()
            .filter(|module| self.modules[*module].parent.is_none())
            .collect();
        if roots.len() > 1 {
            reserve_ids(roots.len(), steps)?;
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
    pub(crate) fn extern_roots(
        &self,
        module: usize,
        name: &str,
        heads: Heads,
        into: &mut BTreeSet<usize>,
        steps: Steps<'_>,
    ) -> Result<bool, Fail> {
        let before = into.len();
        for root in self.extern_root_candidates(module, name, heads) {
            insert_module(into, root, steps)?;
        }
        Ok(into.len() > before)
    }

    /// Borrow the same prelude and root declarations used by external resolution.
    pub(crate) fn extern_root_candidates<'a>(
        &'a self,
        module: usize,
        name: &'a str,
        heads: Heads,
    ) -> impl Iterator<Item = usize> + 'a {
        let test = match heads {
            Heads::Code { test } | Heads::Use { test, .. } => Some(test),
            Heads::Local => None,
        };
        let root = self.modules[module].root;
        test.into_iter().flat_map(move |test| {
            let prelude = self
                .extern_name(module, name, test)
                .into_iter()
                .flat_map(move |entry| entry.roots(test));
            let explicit = self.scopes[root]
                .get(name)
                .into_iter()
                .flat_map(|entry| entry.defs[slot(Namespace::Type)].keys())
                .filter(|id| self.items[id.0].kind == SymbolKind::ExternCrate)
                .flat_map(|id| self.modules_for(*id));
            prelude.chain(explicit)
        })
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
    pub(crate) fn read_outside_crate(
        &self,
        position: usize,
        steps: Steps<'_>,
    ) -> Result<Option<String>, Fail> {
        let decl = &self.uses[position];
        let UseKind::Named { segs, .. } = &decl.kind else {
            return Ok(None);
        };
        let Some(head) = segs.first() else {
            return Ok(None);
        };
        if matches!(head.as_str(), "crate" | "self" | "super") {
            return Ok(None);
        }
        let heads = Heads::Use {
            test: decl.test,
            own: Some(position),
        };
        if self.crate_relative(decl.module, head, decl.leading_colon, heads) {
            let root = self.modules[decl.module].root;
            let lookup = self.read_lookup(root, Namespace::Type, head, None, None, steps)?;
            return Ok(match lookup.as_ref() {
                // An `extern crate` item of a crate outside the index.
                Some((bound, _)) => {
                    if bound.len() == 1 {
                        let id = bound.keys().next().expect("one bound item");
                        if self.modules_for(*id).next().is_none() {
                            if let Some(name) = self.extern_names.get(id) {
                                steps.reserve(name.len()).map_err(Fail::Storage)?;
                                Some(name.to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                None => {
                    if head == self.injected_crate(decl.module) {
                        steps.reserve(head.len()).map_err(Fail::Storage)?;
                        Some(head.clone())
                    } else {
                        None
                    }
                }
            });
        }
        let unbound = decl.leading_colon
            || (self
                .read_lookup(
                    decl.module,
                    Namespace::Type,
                    head,
                    Some(position),
                    None,
                    steps,
                )?
                .as_ref()
                .is_none()
                && !self.blocked(decl.module, head, Namespace::Type, Some(position)));
        let mut roots = BTreeSet::new();
        let outside = unbound
            && self.in_use_prelude(decl.module, head, decl.test)
            && !self.extern_roots(decl.module, head, heads, &mut roots, steps)?;
        if outside {
            steps.reserve(head.len()).map_err(Fail::Storage)?;
        }
        Ok(outside.then(|| head.clone()))
    }

    /// The aliases of another crate that the globs of `from` bring in under
    /// `name`: named imports whose path is a single segment, a crate outside
    /// the index other than `name` (`pub use std as serde;`), that bind
    /// `name` in a module the globs reach, directly or through the globs of
    /// that module, and are visible from `from`. An import of an item of a
    /// crate outside the index is not one: the index cannot tell its name
    /// space, and only the type name space collides with a crate (`use
    /// quote::quote;` brings in a macro).
    pub(crate) fn globbed_outside(
        &self,
        from: usize,
        name: &str,
        steps: Steps<'_>,
    ) -> Result<Vec<usize>, LoadError> {
        self.read_globbed_outside(from, name, steps)
            .map_err(crate::settle::storage_failure)
    }

    fn read_globbed_outside(
        &self,
        from: usize,
        name: &str,
        steps: Steps<'_>,
    ) -> Result<Vec<usize>, Fail> {
        let mut seen = BTreeSet::new();
        insert_module(&mut seen, from, steps)?;
        let stack_memory = steps.query_meter().map(Transient::new);
        let (stack_file, stack_line) = self.module_site(from);
        let stack_steps =
            steps.with_query_storage(stack_memory.as_ref(), (&stack_file, stack_line));
        let mut work = WorkList::<_, 8>::new();
        push_work(&mut work, from, stack_steps).map_err(Fail::Storage)?;
        let mut found = Vec::new();
        while let Some(module) = work.pop() {
            for glob in self.globs_of(module) {
                steps.storage_work(1).map_err(Fail::Storage)?;
                let targets = match self.glob_targets(*glob, steps) {
                    Ok(targets) => targets,
                    Err(Fail::Storage(error)) => return Err(Fail::Storage(error)),
                    Err(_) => continue,
                };
                for target in targets {
                    if seen.contains(&target) {
                        continue;
                    }
                    insert_module(&mut seen, target, steps)?;
                    push_work(&mut work, target, stack_steps).map_err(Fail::Storage)?;
                    for position in self.imports_of(target, name) {
                        steps.storage_work(1).map_err(Fail::Storage)?;
                        let import = &self.uses[*position];
                        let crate_alias =
                            matches!(&import.kind, UseKind::Named { segs, .. } if segs.len() == 1);
                        if crate_alias
                            && !self.use_resolved[*position]
                            && self.read_visibility(&import.vis, target, from, steps)?
                            && self
                                .read_outside_crate(*position, steps)?
                                .is_some_and(|krate| krate != name)
                        {
                            steps
                                .reserve(
                                    vec_slot::<usize>()
                                        + if found.is_empty() {
                                            vec_first::<usize>()
                                        } else {
                                            0
                                        },
                                )
                                .map_err(Fail::Storage)?;
                            found.push(*position);
                        }
                    }
                }
            }
        }
        steps
            .storage_work(
                found
                    .len()
                    .saturating_mul(found.len().max(1).ilog2() as usize + 1),
            )
            .map_err(Fail::Storage)?;
        found.sort_unstable();
        Ok(found)
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
        self.read_glob_against_prelude(from, name, test, bound, Steps::Load)
            .expect("a nonretained prelude check reserves no import storage")
    }

    fn read_glob_against_prelude(
        &self,
        from: usize,
        name: &str,
        test: bool,
        bound: &crate::settle::Bound,
        steps: Steps<'_>,
    ) -> Result<Option<Vec<ItemId>>, Fail> {
        if self.edition(from) == 2015 || !self.in_use_prelude(from, name, test) {
            return Ok(None);
        }
        let mut roots = BTreeSet::new();
        self.extern_roots(from, name, Heads::Code { test }, &mut roots, steps)?;
        let names_the_crate = bound.keys().all(|id| {
            let item = &self.items[id.0];
            match item.kind {
                SymbolKind::ExternCrate => {
                    let count = self.modules_for(*id).count();
                    if count == 0 {
                        self.extern_names
                            .get(id)
                            .is_some_and(|real| real.as_ref() == name)
                    } else {
                        count == roots.len()
                            && self.modules_for(*id).all(|module| roots.contains(&module))
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
            return Ok(None);
        }
        let ids = bound.keys().copied().chain(
            roots
                .iter()
                .filter_map(|root| self.root_items.get(root).copied()),
        );
        reserve_ids(bound.len() + roots.len(), steps)?;
        Ok(Some(self.sorted(ids)))
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
        steps: Steps<'_>,
    ) -> Result<(), Fail> {
        let wait = || {
            steps
                .reserve(vec_first::<Source>() + vec_slot::<Source>() + first.len())
                .map_err(Fail::Storage)?;
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
        let pending = self.read_pending_at(from, first, slot(ns), None, heads.own(), steps)?;
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
        let carried = self.read_globbed_outside(from, first, steps)?;
        if carried.is_empty() {
            return rest();
        }
        let mut roots = BTreeSet::new();
        self.extern_roots(from, first, heads, &mut roots, steps)?;
        let ids = carried
            .iter()
            .flat_map(|position| self.module_items(self.uses[*position].module))
            .chain(
                roots
                    .iter()
                    .filter_map(|root| self.root_items.get(root).copied()),
            );
        reserve_ids(carried.len() + roots.len(), steps)?;
        Err(Fail::Ambiguous(self.sorted(ids)))
    }

    /// The state after the first segment of a path, and the index of the
    /// segment to read next.
    #[expect(
        clippy::too_many_arguments,
        reason = "the first segment is read as the walk reads it, noting what it read and counting its lookups"
    )]
    fn start(
        &self,
        from: usize,
        segs: &[String],
        leading_colon: bool,
        last_ns: Namespace,
        heads: Heads,
        reads: &mut Option<&mut Vec<Source>>,
        steps: Steps<'_>,
    ) -> Result<(Walked, usize), Fail> {
        let mut state = Walked::default();
        let Some(first) = segs.first() else {
            return Err(Fail::Missing(Vec::new()));
        };
        if self.crate_relative(from, first, leading_colon, heads) {
            // The walk reads the first segment in the crate root's scope,
            // which holds its items, its imports and its `extern crate` items.
            insert_module(&mut state.modules, self.modules[from].root, steps)?;
            return Ok((state, 0));
        }
        let mut next_index = 1;
        if leading_colon {
            if !self.extern_roots(from, first, heads, &mut state.modules, steps)? {
                return Err(Fail::Missing(Vec::new()));
            }
        } else {
            match first.as_str() {
                "crate" => {
                    insert_module(&mut state.modules, self.modules[from].root, steps)?;
                }
                "self" => {
                    insert_module(&mut state.modules, from, steps)?;
                }
                "super" => {
                    let mut module = from;
                    let mut at = 0;
                    while segs.get(at).is_some_and(|s| s == "super") {
                        module = self.modules[module].parent.ok_or(Fail::SuperAboveRoot)?;
                        at += 1;
                    }
                    insert_module(&mut state.modules, module, steps)?;
                    next_index = at;
                }
                _ => {
                    let ns = if segs.len() == 1 {
                        last_ns
                    } else {
                        Namespace::Type
                    };
                    note(reads, from, ns, first, steps)?;
                    let lookup = self.read_lookup(from, ns, first, heads.own(), None, steps)?;
                    if lookup.as_ref().is_none() {
                        self.unbound_use_head(from, ns, first, heads, steps)?;
                    }
                    if let Some((bound, from_glob)) = lookup.as_ref() {
                        if let (true, Namespace::Type, Heads::Use { test, .. }) =
                            (from_glob, ns, heads)
                            && let Some(ids) =
                                self.read_glob_against_prelude(from, first, test, bound, steps)?
                        {
                            return Err(Fail::Ambiguous(ids));
                        }
                        // As in the segment loop: a name two globs bring in
                        // as different items is ambiguous in the module, and
                        // a path cannot start with it, also where only one
                        // of the items has what the path goes on to name.
                        // The `cfg` alternatives of one module are one
                        // candidate, and so are definitions and imports of
                        // one name, which all count.
                        if from_glob
                            && segs.len() > 1
                            && self.several_symbols(bound.keys().copied())
                        {
                            reserve_ids(bound.len(), steps)?;
                            return Err(Fail::Ambiguous(self.sorted(bound.keys().copied())));
                        }
                        for (id, origin) in bound.iter() {
                            steps.reserve(first.len()).map_err(Fail::Storage)?;
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
                                steps,
                            )?;
                        }
                    } else if !self.extern_roots(from, first, heads, &mut state.modules, steps)? {
                        steps
                            .reserve(vec_first::<Source>() + vec_slot::<Source>() + first.len())
                            .map_err(Fail::Storage)?;
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
        steps: Steps<'_>,
    ) -> Result<Walked, Fail> {
        self.walk_noting(from, segs, leading_colon, last_ns, heads, None, steps)
    }

    /// [`Index::walk`], adding to `reads` every place whose binding the walk
    /// looked up, found or not: what its answer depends on while the tables
    /// settle.
    ///
    /// The lookup steps of the walk are counted where `steps` says. In a
    /// query after the load the walk is refused with [`Fail::Budget`] once
    /// they pass the limit of the query: after its first segment and after
    /// each segment it reads.
    #[expect(
        clippy::too_many_arguments,
        reason = "a walk reads one path from one module under one reading, noting what it read and counting its lookups"
    )]
    pub(crate) fn walk_noting(
        &self,
        from: usize,
        segs: &[String],
        leading_colon: bool,
        last_ns: Namespace,
        heads: Heads,
        mut reads: Option<&mut Vec<Source>>,
        steps: Steps<'_>,
    ) -> Result<Walked, Fail> {
        let (mut state, next_index) =
            self.start(from, segs, leading_colon, last_ns, heads, &mut reads, steps)?;
        Self::query_refusal(steps).map_err(Fail::Budget)?;
        self.one_crate(&state.modules, next_index, steps)?;
        // A `use` path reads each module as rustc resolves an import: only
        // what the importing module can access counts.
        let viewer = matches!(heads, Heads::Use { .. }).then_some(from);
        for (offset, seg) in segs.iter().enumerate().skip(next_index) {
            let last = offset + 1 == segs.len();
            let ns = if last { last_ns } else { Namespace::Type };
            state = self.step(
                state,
                Step {
                    offset,
                    seg,
                    ns,
                    last,
                },
                heads,
                viewer,
                &mut reads,
                steps,
            )?;
        }
        Ok(state)
    }

    /// One actual metered segment, shared by use/code walks and prefix reachability.
    fn step(
        &self,
        mut state: Walked,
        on: Step<'_>,
        heads: Heads,
        viewer: Option<usize>,
        reads: &mut Option<&mut Vec<Source>>,
        steps: Steps<'_>,
    ) -> Result<Walked, Fail> {
        let Step {
            offset,
            seg,
            ns,
            last,
        } = on;
        work::add(Work::SegmentReads, 1);
        if seg == "super" && state.items.is_empty() {
            let mut parents = BTreeSet::new();
            for module in &state.modules {
                insert_module(
                    &mut parents,
                    self.modules[*module].parent.ok_or(Fail::SuperAboveRoot)?,
                    steps,
                )?;
            }
            return Ok(Walked {
                modules: parents,
                items: BTreeMap::new(),
            });
        }
        let mut containers = std::mem::take(&mut state.modules);
        let carried = state.items.values().any(|hit| hit.via_glob);
        for id in state.items.keys() {
            for module in self.modules_for(*id) {
                insert_module(&mut containers, module, steps)?;
            }
        }
        self.one_crate(&containers, offset, steps)?;
        let mut found: BTreeMap<ItemId, Hit> = BTreeMap::new();
        let mut missing = Vec::new();
        for module in containers {
            note(reads, module, ns, seg, steps)?;
            let lookup = self.read_lookup(module, ns, seg, heads.own(), viewer, steps)?;
            if let Some((bound, from_glob)) = lookup.as_ref() {
                // A name two globs bring in as different items is
                // ambiguous in the module, and a path cannot go
                // through it, as in rustc: also where only one of the
                // items has what the path goes on to name. The
                // `cfg` alternatives of one module are one
                // candidate, and the definitions and imports of a
                // name all count.
                if from_glob && !last && self.several_symbols(bound.keys().copied()) {
                    reserve_ids(bound.len(), steps)?;
                    return Err(Fail::Ambiguous(self.sorted(bound.keys().copied())));
                }
                for (id, origin) in bound.iter() {
                    steps.reserve(seg.len()).map_err(Fail::Storage)?;
                    merge(
                        &mut found,
                        *id,
                        Hit {
                            via_glob: carried || from_glob || origin.via_glob,
                            last: Some(Source {
                                module,
                                ns,
                                name: seg.to_owned(),
                            }),
                        },
                        steps,
                    )?;
                }
            } else {
                push_source(&mut missing, module, ns, seg, steps)?;
            }
        }
        for (id, hit) in &state.items {
            if let Some(members) = self.assoc.get(id).and_then(|m| m.get(seg)) {
                for member in members {
                    if self.items[member.0].in_namespace(ns) {
                        merge(
                            &mut found,
                            *member,
                            Hit {
                                via_glob: hit.via_glob,
                                last: None,
                            },
                            steps,
                        )?;
                    }
                }
            }
        }
        Self::query_refusal(steps).map_err(Fail::Budget)?;
        if found.is_empty() {
            return Err(Fail::Missing(missing));
        }
        Ok(Walked {
            modules: BTreeSet::new(),
            items: found,
        })
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
    ///
    /// # Errors
    ///
    /// Preserves storage and work refusals from every namespace walk.
    fn import_walks(
        &self,
        position: usize,
        resolved_mode: bool,
        steps: Steps<'_>,
    ) -> Result<ImportWalks, ResolveError> {
        let decl = &self.uses[position];
        let UseKind::Named { segs, .. } = &decl.kind else {
            return Ok(ImportWalks::default());
        };
        let mut reads = Vec::new();
        let mut ambiguous = BTreeSet::new();
        for ns in Namespace::ALL {
            match self.walk_noting(
                decl.module,
                segs,
                decl.leading_colon,
                ns,
                Heads::Use {
                    test: decl.test,
                    own: Some(position),
                },
                Some(&mut reads),
                steps,
            ) {
                Err(Fail::Ambiguous(ids)) => ambiguous.extend(ids),
                Err(Fail::Storage(error)) => return Err(ResolveError::storage(error)),
                Err(Fail::Budget(refused)) => return Err(self.refused(refused)),
                Ok(_)
                | Err(Fail::Missing(_) | Fail::SuperAboveRoot | Fail::SeveralCrates { .. }) => {}
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
        Ok(ImportWalks {
            blockers: blockers.into_iter().collect(),
            ambiguous: ambiguous.into_iter().collect(),
            reads,
        })
    }

    /// True when what is bound at `source` was bound by a named import that
    /// never settled: one that bound its name in this name space and is left
    /// undecided in another, by a cycle there, or one that a cycle left to
    /// the globs there and that never decided (see [`Index::left_to_globs`]).
    /// rustc reports such an import as an error, that it cannot determine
    /// its resolution, whatever it bound, so a path through it does not
    /// resolve.
    pub(crate) fn bound_by_undecided(&self, module: usize, ns: Namespace, name: &str) -> bool {
        let at = slot(ns);
        let bound = self.scopes[module]
            .get(name)
            .is_some_and(|entry| !entry.defs[at].is_empty());
        self.imports_of(module, name).iter().any(|position| {
            !self.use_settled[*position]
                && (bound && self.use_bound[*position][at] || self.left_to_globs(*position, at))
        })
    }

    /// Why nothing is bound at `missing`, the places a walk found unbound:
    /// `Cyclic` when an import that binds one of them never settled, which
    /// the settling of the tables leaves only to the imports of a cycle and
    /// those that wait on one, or a glob may still bring the name in from a
    /// module where such an import keeps it (see [`Index::unsettled_at`]);
    /// else
    /// `Ambiguous` naming the items of every ambiguous first segment among
    /// the unresolved imports that bind them, or that those wait on; else
    /// `UnresolvedGlob` when a lookup of the walk, at `reads`, or of the paths
    /// of those imports reads a glob whose path names nothing past its first
    /// segment; else `NotFound`.
    ///
    /// A query whose lookups pass its limit on the way is refused with
    /// [`ResolveError::Budget`] instead.
    fn unbound(
        &self,
        missing: &[Source],
        reads: &[Source],
        text: &str,
        steps: Steps<'_>,
    ) -> ResolveError {
        if let Err(error) = self.record_failed_routes(missing.iter().chain(reads), steps) {
            return ResolveError::storage(error);
        }
        let mut unsettled = None;
        for source in missing {
            match self.unsettled_at(source, steps) {
                Ok(Some(reason)) => {
                    unsettled = Some(reason);
                    break;
                }
                Ok(None) => (),
                Err(error) => return ResolveError::storage(error),
            }
        }
        if let Err(refused) = Self::query_refusal(steps) {
            return self.refused(refused);
        }
        match unsettled {
            Some(Unsettled::Cycle) => return ResolveError::Cyclic(text.to_owned()),
            Some(Unsettled::LateImport(position)) => {
                let decl = &self.uses[position];
                return ResolveError::IndeterminateImport {
                    path: text.to_owned(),
                    file: decl.file.clone(),
                    line: decl.line,
                };
            }
            None => {}
        }
        // The imports on the way wait on no cycle, so the search ends; it
        // keeps a list of its own and visits each import once.
        let stack_memory = steps.query_meter().map(Transient::new);
        let Some(source) = missing.first().or_else(|| reads.first()) else {
            return ResolveError::NotFound(text.to_owned());
        };
        let (stack_file, stack_line) = self.module_site(source.module);
        let stack_steps =
            steps.with_query_storage(stack_memory.as_ref(), (&stack_file, stack_line));
        let mut seen: BTreeSet<usize> = BTreeSet::new();
        let mut work = WorkList::<_, 8>::new();
        for source in missing {
            for position in self.unresolved_uses(source.module, &source.name, true) {
                if let Err(error) = push_work(&mut work, position, stack_steps) {
                    return ResolveError::storage(error);
                }
            }
        }
        let mut ambiguous: BTreeSet<ItemId> = BTreeSet::new();
        let mut through: Vec<Source> = reads.to_vec();
        while let Some(position) = work.pop() {
            if !seen.insert(position) {
                continue;
            }
            let ImportWalks {
                blockers: following,
                ambiguous: items,
                reads,
            } = match self.import_walks(position, true, steps) {
                Ok(walks) => walks,
                Err(error) => return error,
            };
            if let Err(refused) = Self::query_refusal(steps) {
                return self.refused(refused);
            }
            ambiguous.extend(items);
            if let Err(error) = self.record_failed_routes(reads.iter(), steps) {
                return ResolveError::storage(error);
            }
            for position in following {
                if let Err(error) = push_work(&mut work, position, stack_steps) {
                    return ResolveError::storage(error);
                }
            }
            through.extend(reads);
        }
        if !ambiguous.is_empty() {
            return ResolveError::Ambiguous(self.sorted(ambiguous));
        }
        match self.unresolved_glob_read(&through, steps) {
            Ok(Some(glob)) => self.glob_error(glob, text),
            Ok(None) => ResolveError::NotFound(text.to_owned()),
            Err(error) => ResolveError::storage(error),
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

    /// What the path of the glob import at `position` names in the settled
    /// tables.
    fn glob_path(&self, position: usize, steps: Steps<'_>) -> Result<GlobPath, LoadError> {
        let decl = &self.uses[position];
        let UseKind::Glob { segs } = &decl.kind else {
            return Ok(GlobPath::Nothing);
        };
        let sources = match self.glob_targets(position, steps) {
            Ok(targets) => return Ok(GlobPath::Resolves(targets)),
            Err(Fail::Missing(sources)) => sources,
            Err(Fail::SuperAboveRoot | Fail::SeveralCrates { .. } | Fail::Ambiguous(_)) => {
                return Ok(GlobPath::Nothing);
            }
            Err(Fail::Budget(_)) => unreachable!("{LOAD_WALK}"),
            Err(Fail::Storage(error)) => return Err(error),
        };
        // A missing path with no known source cannot be diagnosed as a
        // missing local module. This also covers a glob with no segments.
        if sources.is_empty() {
            return Ok(GlobPath::Unknown);
        }
        let head = self.walk(
            decl.module,
            &segs[..1],
            decl.leading_colon,
            Namespace::Type,
            Heads::Use {
                test: decl.test,
                own: Some(position),
            },
            steps,
        );
        if let Err(Fail::Storage(error)) = head {
            return Err(error);
        }
        // A first segment that names nothing in the workspace, or a path
        // that goes on in a crate outside it, is a dependency.
        if head.is_err() {
            Ok(GlobPath::Unknown)
        } else {
            Ok(GlobPath::Misses(sources))
        }
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
        steps: Steps<'_>,
    ) -> Option<ResolveError> {
        if let Err(error) = self.binding_determinacy(module, ns, name, bound, text, steps) {
            return Some(error);
        }
        if self.bound_by_undecided(module, ns, name) {
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
        let source = Source {
            module,
            ns,
            name: name.to_owned(),
        };
        match self.unresolved_glob_read(std::slice::from_ref(&source), steps) {
            Ok(glob) => glob.map(|glob| self.glob_error(glob, text)),
            Err(error) => Some(ResolveError::storage(error)),
        }
    }

    /// The refusal of a query whose lookups passed the limit of its work,
    /// once they have: the walks of the query check it after each segment,
    /// and the query before it answers. Never for the load, whose lookups
    /// are refused at its checkpoints.
    pub(crate) fn query_refusal(steps: Steps<'_>) -> Result<(), Refused> {
        if let Some(meter) = steps.query_meter()
            && let Some(module) = meter.query_passed()
        {
            return Err(Refused {
                limit: meter.work_limit(),
                module,
            });
        }
        Ok(())
    }

    /// The answer of a query, or its refusal when its lookups passed the
    /// limit of its work on the way: the refusal comes first.
    fn query_checked(
        &self,
        steps: Steps<'_>,
        answer: Result<Resolution, ResolveError>,
    ) -> Result<Resolution, ResolveError> {
        match Self::query_refusal(steps) {
            Err(refused) => Err(self.refused(refused)),
            Ok(()) => answer,
        }
    }

    /// The [`ResolveError::Budget`] of a refused query.
    pub(crate) fn refused(&self, refused: Refused) -> ResolveError {
        let (file, line) = self.module_site(refused.module);
        ResolveError::Budget {
            budget: Budget::Work,
            limit: refused.limit,
            file,
            line,
        }
    }

    /// The [`LoadError`] of a refused query, as the public surface reports
    /// it ([`ExportError::Budget`](crate::ExportError::Budget)).
    pub(crate) fn refused_load(&self, refused: Refused) -> LoadError {
        let (file, line) = self.module_site(refused.module);
        over_budget(Budget::Work, refused.limit, &file, line)
    }

    /// The file and line that name `module`: its `mod` item, or for a crate
    /// root the root file of its target, with no line.
    pub(crate) fn module_site(&self, module: usize) -> (RelPath, Option<usize>) {
        if self.modules[module].parent.is_some()
            && let Some(id) = self.module_items(module).first()
        {
            let item = &self.items[id.0];
            return (item.file.clone(), Some(item.name_line));
        }
        let target = &self.targets[self.modules[module].target.index()];
        (target.root.clone(), None)
    }

    /// True when the path of the glob import at `position` names nothing in
    /// the workspace past its first segment, so a lookup that reads it fails
    /// with [`ResolveError::UnresolvedGlob`].
    pub(crate) fn glob_broken(&self, position: usize, meter: &Meter) -> Result<bool, LoadError> {
        let Some(facts) = self.glob_facts() else {
            return Ok(false);
        };
        let decl = &self.uses[position];
        meter.work(
            glob_facts::table_units(facts.broken.len()),
            &decl.file,
            Some(decl.line),
        )?;
        Ok(facts.broken.get(&position).is_some())
    }

    fn glob_facts(&self) -> Option<&GlobFacts> {
        let facts = self.glob_facts.get();
        assert!(
            facts.is_some() || self.glob_imports.is_empty(),
            "glob facts are built before a completed index is queried"
        );
        facts
    }

    pub(crate) fn origin_of(
        &self,
        source: &Source,
        id: ItemId,
        steps: Steps<'_>,
    ) -> Option<crate::settle::Origin> {
        let ns = slot(source.ns);
        let defs = self.scopes[source.module]
            .get(&source.name)
            .and_then(|entry| entry.defs[ns].get(id));
        match defs {
            Some(origin) => Some(origin.clone()),
            None => self
                .globbed(source.module, &source.name, ns, steps)
                .get(id)
                .cloned(),
        }
    }

    fn chain_for(&self, requested: &str, hit: &Hit, id: ItemId, steps: Steps<'_>) -> Vec<String> {
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
                .origin_of(&at, id, steps)
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
    fn package_of(&self, module: usize) -> PackageId {
        self.targets[self.modules[module].target.index()].package_id
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
        steps: Steps<'_>,
    ) -> Result<(), ResolveError> {
        let start = walks.reads.len();
        let walk = self.walk_noting(
            root,
            rest,
            false,
            ns,
            Heads::Local,
            Some(&mut walks.reads),
            steps,
        );
        if walk.is_ok()
            && walks.reads[start..]
                .iter()
                .any(|source| self.bound_by_undecided(source.module, source.ns, &source.name))
        {
            return Err(ResolveError::Cyclic(text.to_owned()));
        }
        match walk {
            Ok(walked) => {
                self.path_determinacy(root, rest, false, ns, Heads::Local, text, steps)?;
                for (id, hit) in walked.items {
                    merge(&mut walks.found, id, hit, steps).map_err(ResolveError::storage)?;
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
            Err(Fail::Budget(refused)) => return Err(self.refused(refused)),
            Err(Fail::Storage(error)) => return Err(ResolveError::storage(error)),
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
        package: Option<PackageId>,
        steps: Steps<'_>,
    ) -> Result<Resolution, ResolveError> {
        let mut walks = Walks::default();
        // The prefix grows by one segment a step, so the prefixes of a path
        // cost its length, not its length squared.
        let mut prefix = String::with_capacity(text.len());
        for (taken, seg) in (1..=segs.len()).zip(segs) {
            work::add(Work::SegmentReads, 1);
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
                .filter(|root| package.is_none_or(|id| self.package_of(*root) == id))
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
                    self.walk_from_root(root, &segs[taken..], ns, text, &mut walks, steps)?;
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
                        steps,
                    )
                    .map_err(ResolveError::storage)?;
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
                    && package.is_none_or(|id| self.targets[item.target.index()].package_id == id)
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
        self.conclude(&found, &missing, &reads, text, ns, from_keys, steps)
    }

    /// What a walk found, as the answer: the unbound error when it found
    /// nothing, ambiguity when it found several symbols, and the symbol
    /// otherwise. `from_keys` is true when the items come from the foreign
    /// keys and not from a walk.
    #[expect(
        clippy::too_many_arguments,
        reason = "the answer is concluded from what one walk found, read and missed, counting its lookups"
    )]
    fn conclude(
        &self,
        found: &BTreeMap<ItemId, Hit>,
        missing: &[Source],
        reads: &[Source],
        text: &str,
        ns: Namespace,
        from_keys: bool,
        steps: Steps<'_>,
    ) -> Result<Resolution, ResolveError> {
        if found.is_empty() {
            return Err(self.unbound(missing, reads, text, steps));
        }
        // One symbol is one path and kind in one crate. Items of two targets
        // at one path are two symbols, whatever order they were found in; the
        // members of trait impls on types that are not local are keyed by
        // the trait, so the impls of every crate share their key.
        let mut groups: BTreeSet<(Option<TargetId>, &str, SymbolKind)> = BTreeSet::new();
        for id in found.keys() {
            let item = &self.items[id.0];
            let target = if from_keys {
                item.trait_definition_target
            } else {
                Some(item.target)
            };
            groups.insert((target, item.path.as_str(), item.kind));
        }
        if groups.len() > 1 {
            return Err(ResolveError::Ambiguous(self.sorted(found.keys().copied())));
        }
        // A lookup on the way that read a glob whose path names nothing past
        // its first segment may have missed what the glob would bring in.
        if let Some(glob) = self
            .unresolved_glob_read(reads, steps)
            .map_err(ResolveError::storage)?
        {
            return Err(self.glob_error(glob, text));
        }
        let sites = self.sorted(found.keys().copied());
        let first = sites[0];
        let hit = &found[&first];
        Ok(Resolution {
            path: self.items[first.0].path.to_string(),
            namespace: ns,
            via_glob: hit.via_glob,
            chain: self.chain_for(text, hit, first, steps),
            type_aliases: Vec::new(),
            sites,
        })
    }

    /// True when `ids` are more than one symbol: a symbol is one path and
    /// kind in one crate, so the `cfg` alternatives of one module, which the
    /// index keeps as several items, are one.
    fn several_symbols(&self, ids: impl Iterator<Item = ItemId>) -> bool {
        let groups: BTreeSet<(TargetId, &str, SymbolKind)> = ids
            .map(|id| {
                let item = &self.items[id.0];
                (item.target, item.path.as_str(), item.kind)
            })
            .collect();
        groups.len() > 1
    }

    /// The module whose canonical path is `path`, such as `demo::inner`, in
    /// the crate of any target: the crate path that starts it is found by
    /// prefixes that grow a segment at a time, as in [`Index::resolve_in`],
    /// so `demo::test_crate::it::support` is a module of the test target
    /// `it` and `demo::bin_crate::tool::cli` one of the binary `tool`.
    ///
    /// A crate path that targets of several packages share (two libraries
    /// with one crate name) names the crates of all of them, and the module is
    /// not told apart without its package: it is
    /// [`ResolveError::Ambiguous`] between their crate roots, as
    /// [`Index::resolve`] answers for a path that starts with that crate path.
    /// A path that is a module of two crates, a module `test_crate::it` of the
    /// library beside the test target `it`, is ambiguous between the items
    /// that stand for the two modules.
    fn module_named(
        &self,
        path: &str,
        package: Option<PackageId>,
    ) -> Result<Option<usize>, ResolveError> {
        let key = SharedStr::from(path);
        let mut found = Vec::new();
        let mut prefix = String::with_capacity(path.len());
        for (taken, segment) in path.split("::").enumerate() {
            work::add(Work::SegmentReads, 1);
            if taken > 0 {
                prefix.push_str("::");
            }
            prefix.push_str(segment);
            let Some(roots) = self.crate_roots.get(&prefix) else {
                continue;
            };
            let roots = roots
                .iter()
                .copied()
                .filter(|root| package.is_none_or(|id| self.package_of(*root) == id));
            if roots.clone().count() > 1 {
                return Err(ResolveError::Ambiguous(self.sorted(
                    roots.filter_map(|root| self.root_items.get(&root).copied()),
                )));
            }
            found.extend(
                roots.filter_map(|root| self.module_by_path.get(&(root, key.clone())).copied()),
            );
        }
        match found.as_slice() {
            [] => Ok(None),
            [one] => Ok(Some(*one)),
            several => Err(ResolveError::Ambiguous(
                self.sorted(several.iter().flat_map(|module| self.module_items(*module))),
            )),
        }
    }

    /// Resolve the path of a `use` declaration written in the module
    /// `module`, in `ns` or in each name space in turn.
    pub(crate) fn resolve_from(
        &self,
        module: &str,
        path: &str,
        ns: Option<Namespace>,
        package: Option<PackageId>,
        meter: &Meter,
    ) -> Result<Resolution, ResolveError> {
        let steps = Steps::Query(meter);
        let answer = self.resolve_from_counted(module, path, ns, package, steps);
        self.query_checked(steps, answer)
    }

    /// [`Index::resolve_from`], its lookup steps counted where `steps` says.
    fn resolve_from_counted(
        &self,
        module: &str,
        path: &str,
        ns: Option<Namespace>,
        package: Option<PackageId>,
        steps: Steps<'_>,
    ) -> Result<Resolution, ResolveError> {
        let from = self
            .module_named(module, package)?
            .ok_or_else(|| ResolveError::UnknownModule(module.to_owned()))?;
        self.resolve_at(
            from,
            path,
            ns,
            Heads::Use {
                test: false,
                own: None,
            },
            steps,
        )
    }

    pub(crate) fn unconditional_method(
        &self,
        id: ItemId,
        steps: Steps<'_>,
    ) -> Result<bool, ResolveError> {
        let item = &self.items[id.index()];
        if item
            .detail
            .as_ref()
            .is_some_and(|detail| detail.method_requires_inference)
        {
            return Ok(false);
        }
        let Some(block) = self
            .impl_blocks
            .iter()
            .find(|block| block.members.contains(&id))
        else {
            return Ok(false);
        };
        let Some(arity) = block.unconditional_self else {
            return Ok(false);
        };
        let base = block
            .self_type
            .split('<')
            .next()
            .unwrap_or(&block.self_type);
        let resolution = match self.resolve_at(
            block.module_index,
            base,
            Some(Namespace::Type),
            Heads::Code {
                test: item.test != crate::TestRole::None,
            },
            steps,
        ) {
            Ok(resolution) => resolution,
            Err(error @ (ResolveError::Budget { .. } | ResolveError::Storage { .. })) => {
                return Err(error);
            }
            Err(_) => return Ok(false),
        };
        Ok(!resolution.sites.is_empty()
            && resolution.sites.iter().all(|site| {
                block.self_items.contains(site)
                    && self.items[site.index()]
                        .detail
                        .as_ref()
                        .and_then(|detail| detail.nominal_const_arity)
                        == Some(arity)
                    && matches!(
                        self.items[site.index()].kind,
                        SymbolKind::Struct | SymbolKind::Enum | SymbolKind::Union
                    )
            }))
    }

    fn prefer_inherent(
        &self,
        found: &mut BTreeMap<ItemId, Hit>,
        steps: Steps<'_>,
    ) -> Result<(), ResolveError> {
        let Some(&first) = found.keys().next() else {
            return Ok(());
        };
        let item = &self.items[first.index()];
        let storage = steps.query_meter().map(Transient::new);
        let steps = steps.with_query_storage(storage.as_ref(), (&item.file, Some(item.name_line)));
        let mut inherent = OrderedTable::default();
        let mut live = 0;
        for id in found.keys() {
            steps.storage_work(1).map_err(ResolveError::storage)?;
            let item = &self.items[id.index()];
            if item.kind != SymbolKind::InherentMethod || !self.unconditional_method(*id, steps)? {
                continue;
            }
            let mut admission = NativeAdmission::resume(steps, live);
            let result = {
                let mut memory = Memory::resume(&mut admission, live);
                let result = inherent.insert((item.target, item.path.as_str()), (), &mut memory);
                live = memory.admitted_bytes();
                result
            };
            result.map_err(|error| ResolveError::storage(admission.graph_error(error.into())))?;
        }
        if !inherent.is_empty() {
            steps
                .storage_work(found.len())
                .map_err(ResolveError::storage)?;
            found.retain(|id, _| {
                let item = &self.items[id.index()];
                item.kind != SymbolKind::TraitImplMethod
                    || inherent.get(&(item.target, item.path.as_str())).is_none()
            });
        }
        Ok(())
    }

    /// The shared traversal for a known reading module and explicit head rules.
    pub(crate) fn resolve_at(
        &self,
        from: usize,
        path: &str,
        ns: Option<Namespace>,
        heads: Heads,
        steps: Steps<'_>,
    ) -> Result<Resolution, ResolveError> {
        let answer = self.resolve_at_counted(from, path, ns, heads, steps);
        self.query_checked(steps, answer)
    }

    fn resolve_at_counted(
        &self,
        from: usize,
        path: &str,
        ns: Option<Namespace>,
        heads: Heads,
        steps: Steps<'_>,
    ) -> Result<Resolution, ResolveError> {
        let (leading_colon, written) = path.strip_prefix("::").map_or((false, path), |p| (true, p));
        let segs: Vec<String> = written
            .split("::")
            .map(|s| strip_raw(s).to_owned())
            .collect();
        let valid = |s: &String| {
            !s.is_empty() && s != "Self" && s.chars().all(|c| c.is_alphanumeric() || c == '_')
        };
        if !segs.iter().all(valid) {
            return Err(ResolveError::Invalid(path.to_owned()));
        }
        let text = segs.join("::");
        let spaces: Vec<Namespace> = ns.map_or_else(|| Namespace::ALL.to_vec(), |one| vec![one]);
        for space in spaces {
            let mut reads = Vec::new();
            let walk = self.walk_noting(
                from,
                &segs,
                leading_colon,
                space,
                heads,
                Some(&mut reads),
                steps,
            );
            if walk.is_ok()
                && reads
                    .iter()
                    .any(|source| self.bound_by_undecided(source.module, source.ns, &source.name))
            {
                return Err(ResolveError::Cyclic(text));
            }
            let outcome = match walk {
                Ok(mut walked) => {
                    self.path_determinacy(from, &segs, leading_colon, space, heads, &text, steps)?;
                    if space == Namespace::Value {
                        self.prefer_inherent(&mut walked.items, steps)?;
                    }
                    if space == Namespace::Type && walked.items.is_empty() {
                        for module in &walked.modules {
                            for definition in self.module_items(*module) {
                                merge(
                                    &mut walked.items,
                                    definition,
                                    Hit {
                                        via_glob: false,
                                        last: None,
                                    },
                                    steps,
                                )
                                .map_err(ResolveError::storage)?;
                            }
                        }
                    }
                    self.conclude(&walked.items, &[], &reads, &text, space, false, steps)
                }
                Err(Fail::Missing(missing)) => Err(self.unbound(&missing, &reads, &text, steps)),
                Err(Fail::SeveralCrates { roots: ids, .. } | Fail::Ambiguous(ids)) => {
                    Err(ResolveError::Ambiguous(ids))
                }
                Err(Fail::SuperAboveRoot) => Err(ResolveError::Invalid(path.to_owned())),
                Err(Fail::Budget(refused)) => Err(self.refused(refused)),
                Err(Fail::Storage(error)) => Err(ResolveError::storage(error)),
            };
            match outcome {
                Err(ResolveError::NotFound(_)) => {}
                other => return other,
            }
        }
        Err(ResolveError::NotFound(text))
    }

    /// The items that a run of path segments, written in code of `from`,
    /// names, each run read once segment by segment: every proper prefix in
    /// the type name space, the name space a path goes on through, and the
    /// whole run in each name space. `test` is true for test code, which also
    /// names the dev-dependencies of a library or a binary.
    ///
    /// A prefix counts as [`Index::resolve_from`] would answer it, read as
    /// code reads it ([`Heads::Code`]): it names its items when the walk
    /// reaches them, no import on the way is undecided, they are one symbol
    /// and no lookup on the way reads a glob whose path names nothing; a
    /// prefix that fails names nothing, and every longer prefix fails with it.
    /// A prefix of path keywords alone (`crate`, `self`, `super`) names a
    /// module and no item.
    pub(crate) fn reach_run(
        &self,
        from: usize,
        segs: &[String],
        leading_colon: bool,
        test: bool,
        steps: Steps<'_>,
    ) -> Result<BTreeSet<ItemId>, LoadError> {
        let heads = Heads::Code { test };
        let mut out = BTreeSet::new();
        match segs.len() {
            0 => {}
            1 => self.reach_single(from, segs, leading_colon, heads, &mut out, steps)?,
            _ => self.reach_chain(from, segs, leading_colon, heads, &mut out, steps)?,
        }
        Self::query_refusal(steps).map_err(|refused| self.refused_load(refused))?;
        Ok(out)
    }

    /// A decoded format capture is a value, without type/macro prefix edges.
    pub(crate) fn reach_value(
        &self,
        from: usize,
        name: &str,
        test: bool,
        steps: Steps<'_>,
    ) -> Result<BTreeSet<ItemId>, LoadError> {
        steps.reserve(name.len())?;
        let segs = [name.to_owned()];
        let mut reads = Vec::new();
        let mut out = BTreeSet::new();
        let outcome = self.walk_noting(
            from,
            &segs,
            false,
            Namespace::Value,
            Heads::Code { test },
            Some(&mut reads),
            steps,
        );
        if let Some(walked) = self.reach_outcome(outcome, steps)? {
            self.take(&walked, false, &reads, &mut out, steps)?;
        }
        Self::query_refusal(steps).map_err(|refused| self.refused_load(refused))?;
        Ok(out)
    }

    /// Only semantic nonresolution contributes no reachability edge.
    pub(crate) fn reach_outcome<T>(
        &self,
        outcome: Result<T, Fail>,
        steps: Steps<'_>,
    ) -> Result<Option<T>, LoadError> {
        Self::query_refusal(steps).map_err(|refused| self.refused_load(refused))?;
        match outcome {
            Ok(value) => Ok(Some(value)),
            Err(Fail::Storage(error)) => Err(error),
            Err(Fail::Budget(refused)) => Err(self.refused_load(refused)),
            Err(
                Fail::Missing(_)
                | Fail::SuperAboveRoot
                | Fail::SeveralCrates { .. }
                | Fail::Ambiguous(_),
            ) => Ok(None),
        }
    }

    /// Admit the owned prefix reused by another terminal namespace branch.
    fn fork_walk(state: &Walked, steps: Steps<'_>) -> Result<Walked, LoadError> {
        steps.storage_work(state.modules.len().saturating_add(state.items.len()))?;
        let mut branch = Walked::default();
        for module in &state.modules {
            insert_module(&mut branch.modules, *module, steps)?;
        }
        for (id, hit) in &state.items {
            if let Some(source) = &hit.last {
                steps.reserve(source.name.len())?;
            }
            merge(&mut branch.items, *id, hit.clone(), steps)?;
        }
        Ok(branch)
    }

    /// A single segment is read in each namespace under the same allowance.
    fn reach_single(
        &self,
        from: usize,
        segs: &[String],
        leading_colon: bool,
        heads: Heads,
        out: &mut BTreeSet<ItemId>,
        steps: Steps<'_>,
    ) -> Result<(), LoadError> {
        for ns in Namespace::ALL {
            let mut own = Vec::new();
            let started = self.start(
                from,
                segs,
                leading_colon,
                ns,
                heads,
                &mut Some(&mut own),
                steps,
            );
            let Some((first, next)) = self.reach_outcome(started, steps)? else {
                continue;
            };
            let walked = self.one_crate(&first.modules, next, steps).and_then(|()| {
                if next == 0 {
                    self.step(
                        first,
                        Step {
                            offset: 0,
                            seg: &segs[0],
                            ns,
                            last: true,
                        },
                        heads,
                        None,
                        &mut Some(&mut own),
                        steps,
                    )
                } else {
                    Ok(first)
                }
            });
            if let Some(walked) = self.reach_outcome(walked, steps)? {
                self.take(&walked, false, &own, out, steps)?;
            }
        }
        Ok(())
    }

    /// Walk each proper type prefix once, then fork only the terminal namespaces.
    fn reach_chain(
        &self,
        from: usize,
        segs: &[String],
        leading_colon: bool,
        heads: Heads,
        out: &mut BTreeSet<ItemId>,
        steps: Steps<'_>,
    ) -> Result<(), LoadError> {
        let mut reads = Vec::new();
        let started = self.start(
            from,
            segs,
            leading_colon,
            Namespace::Type,
            heads,
            &mut Some(&mut reads),
            steps,
        );
        let Some((mut state, next)) = self.reach_outcome(started, steps)? else {
            return Ok(());
        };
        if self
            .reach_outcome(self.one_crate(&state.modules, next, steps), steps)?
            .is_none()
        {
            return Ok(());
        }
        let mut tainted = self.taints(&reads, steps)?;
        if next == 1 {
            tainted |= self.take(&state, tainted, &reads, out, steps)?;
        }
        let last_offset = segs.len() - 1;
        for (offset, seg) in segs.iter().enumerate().skip(next) {
            if offset == last_offset {
                let mut retained = Some(state);
                for (branch, ns) in Namespace::ALL.into_iter().enumerate() {
                    let state = if branch + 1 == Namespace::ALL.len() {
                        retained
                            .take()
                            .expect("the final namespace owns the retained prefix")
                    } else {
                        Self::fork_walk(
                            retained
                                .as_ref()
                                .expect("earlier namespaces borrow the prefix"),
                            steps,
                        )?
                    };
                    let mut own = Vec::new();
                    let walked = self.step(
                        state,
                        Step {
                            offset,
                            seg,
                            ns,
                            last: true,
                        },
                        heads,
                        None,
                        &mut Some(&mut own),
                        steps,
                    );
                    if let Some(walked) = self.reach_outcome(walked, steps)? {
                        self.take(&walked, tainted, &own, out, steps)?;
                    }
                }
                return Ok(());
            }
            let mut own = Vec::new();
            let walked = self.step(
                state,
                Step {
                    offset,
                    seg,
                    ns: Namespace::Type,
                    last: false,
                },
                heads,
                None,
                &mut Some(&mut own),
                steps,
            );
            let Some(walked) = self.reach_outcome(walked, steps)? else {
                return Ok(());
            };
            state = walked;
            tainted |= self.take(&state, tainted, &own, out, steps)?;
        }
        Ok(())
    }

    fn taints(&self, reads: &[Source], steps: Steps<'_>) -> Result<bool, LoadError> {
        if reads
            .iter()
            .any(|source| self.bound_by_undecided(source.module, source.ns, &source.name))
        {
            return Ok(true);
        }
        let broken = self.unresolved_glob_read(reads, steps)?;
        Self::query_refusal(steps).map_err(|refused| self.refused_load(refused))?;
        Ok(broken.is_some())
    }

    /// Retain only untainted, unambiguous reached identities, before allocating.
    fn take(
        &self,
        walked: &Walked,
        tainted: bool,
        own: &[Source],
        out: &mut BTreeSet<ItemId>,
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        if tainted || self.taints(own, steps)? {
            return Ok(true);
        }
        if walked.items.is_empty() || self.several_symbols(walked.items.keys().copied()) {
            return Ok(false);
        }
        if !self.reach_binding_determinacy(walked, own, steps)? {
            return Ok(true);
        }
        steps.storage_work(walked.items.len())?;
        for id in walked.items.keys() {
            if !out.contains(id) {
                steps.reserve(
                    btree_entry::<ItemId, ()>()
                        + if out.is_empty() {
                            btree_map::<ItemId, ()>()
                        } else {
                            0
                        },
                )?;
                out.insert(*id);
            }
        }
        Ok(false)
    }

    /// True when a proper prefix of `segs` is the self path of an inherent
    /// impl that the index cannot see, of `package` when one is given.
    fn through_unresolved(&self, segs: &[String], package: Option<PackageId>) -> bool {
        // The prefix grows by one segment a step, as in [`Index::resolve_in`].
        let mut prefix = String::new();
        segs.iter().take(segs.len().saturating_sub(1)).any(|seg| {
            work::add(Work::SegmentReads, 1);
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
                    package.is_none_or(|id| self.targets[block.target.index()].package_id == id)
                })
        })
    }

    /// Resolve `path`, trying the type, value and macro name spaces in turn,
    /// among the crates of `package` when one is given. The lookup steps are
    /// counted on `meter`, the meter of the query: one that passes its limit
    /// is refused with [`ResolveError::Budget`], whatever it would answer.
    pub(crate) fn resolve(
        &self,
        path: &str,
        ns: Option<Namespace>,
        package: Option<PackageId>,
        meter: &Meter,
    ) -> Result<Resolution, ResolveError> {
        let steps = Steps::Query(meter);
        let answer = self.resolve_counted(path, ns, package, steps);
        self.query_checked(steps, answer)
    }

    /// [`Index::resolve`], its lookup steps counted where `steps` says.
    fn resolve_counted(
        &self,
        path: &str,
        ns: Option<Namespace>,
        package: Option<PackageId>,
        steps: Steps<'_>,
    ) -> Result<Resolution, ResolveError> {
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
            match self.resolve_in(&segs, space, &text, package, steps) {
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
    use crate::work::{self, Work};
    use crate::{MemoryTree, Workspace};

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
            ResolveError::AmbiguousPackageName(hostile()),
            ResolveError::UnknownPackageId(crate::PackageId(7)),
            ResolveError::UnconfiguredPackage(crate::PackageId(7)),
            ResolveError::UnconfiguredDefinition(ItemId(7)),
            ResolveError::UnknownModule(hostile()),
            ResolveError::IndeterminateImport {
                path: hostile(),
                file: crate::RelPath::new(&format!("src/{}.rs", hostile()))
                    .expect("a relative path"),
                line: 3,
            },
            ResolveError::UnresolvedGlob {
                path: hostile(),
                glob: hostile(),
                module: hostile(),
            },
            ResolveError::Budget {
                budget: crate::Budget::Work,
                limit: 7,
                file: crate::RelPath::new(&format!("src/{}.rs", hostile()))
                    .expect("a relative path"),
                line: Some(3),
            },
            ResolveError::Storage {
                file: crate::RelPath::new(&format!("src/{}.rs", hostile()))
                    .expect("a relative path"),
                line: Some(3),
                what: hostile(),
            },
        ];
        for error in &all {
            match error {
                ResolveError::NotFound(_)
                | ResolveError::Ambiguous(_)
                | ResolveError::Cyclic(_)
                | ResolveError::IndeterminateImport { .. }
                | ResolveError::Invalid(_)
                | ResolveError::UnresolvedSelf(_)
                | ResolveError::UnknownPackage(_)
                | ResolveError::AmbiguousPackageName(_)
                | ResolveError::UnknownPackageId(_)
                | ResolveError::UnconfiguredPackage(_)
                | ResolveError::UnconfiguredDefinition(_)
                | ResolveError::UnknownModule(_)
                | ResolveError::UnresolvedGlob { .. }
                | ResolveError::Budget { .. }
                | ResolveError::Storage { .. } => {}
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
                ResolveError::Ambiguous(_)
                | ResolveError::UnknownPackageId(_)
                | ResolveError::UnconfiguredPackage(_)
                | ResolveError::UnconfiguredDefinition(_) => 0,
                ResolveError::UnresolvedGlob { .. } => 3,
                ResolveError::IndeterminateImport { .. } | ResolveError::Storage { .. } => 2,
                _ => 1,
            };
            assert_eq!(
                text.matches(r"a\u{1b}[2J\u{202e}\r\nb").count(),
                fields,
                "{text}"
            );
        }
    }

    #[test]
    fn a_module_path_of_fifty_thousand_segments_is_read_once_to_find_its_crate() {
        const SEGMENTS: usize = 50_000;
        let manifest = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n";
        let tree: MemoryTree = [("Cargo.toml", manifest), ("src/lib.rs", "pub fn x() {}\n")]
            .into_iter()
            .collect();
        let ws = Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"));
        let module = format!("demo{}", "::a".repeat(SEGMENTS));
        let (answer, tally) = work::measure(SEGMENTS + 1, || ws.resolve_from(&module, "x"));
        assert_eq!(answer, Err(ResolveError::UnknownModule(module.clone())));
        // Each segment grows the crate path that is looked up once.
        assert_eq!(tally.get(Work::SegmentReads), SEGMENTS + 1);
        let (answer, tally) = work::measure(SEGMENTS + 1, || ws.resolve_from("demo", "x"));
        assert_eq!(answer.map(|found| found.path), Ok("demo::x".to_owned()));
        assert!(tally.get(Work::SegmentReads) >= 1);
    }

    #[test]
    fn a_path_of_fifty_thousand_segments_reads_each_segment_a_bounded_number_of_times() {
        const SEGMENTS: usize = 50_000;
        let manifest = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n";
        let tree: MemoryTree = [
            ("Cargo.toml", manifest),
            (
                "src/lib.rs",
                "pub mod m { pub use crate::m; pub fn end() {} }\n",
            ),
        ]
        .into_iter()
        .collect();
        let ws = Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"));
        let missing = format!("demo{}", "::a".repeat(SEGMENTS));
        let deep = format!("demo{}::end", "::m".repeat(SEGMENTS));
        // A path is walked in each name space and scanned once for a block it
        // cannot see into, so a segment is read a few times and not once for
        // each segment before it. The bound is also the ceiling, so a walk
        // that starts again from the crate root for each prefix stops at once.
        let segments = SEGMENTS + 2;
        let reads = 4 * segments;
        let lookups = 2 * segments;
        let (answer, tally) = work::measure(reads, || ws.resolve(&missing));
        assert_eq!(answer, Err(ResolveError::NotFound(missing.clone())));
        assert!(tally.get(Work::SegmentReads) > 0);
        assert!(tally.get(Work::SegmentReads) <= reads);
        assert!(tally.get(Work::Lookups) > 0);
        assert!(tally.get(Work::Lookups) <= lookups);
        let (answer, tally) = work::measure(reads, || ws.resolve(&deep));
        assert_eq!(answer.expect("the deep path resolves").path, "demo::m::end");
        assert!(tally.get(Work::SegmentReads) > 0);
        assert!(tally.get(Work::SegmentReads) <= reads);
        assert!(tally.get(Work::Lookups) > 0);
        assert!(tally.get(Work::Lookups) <= lookups);
    }
}
