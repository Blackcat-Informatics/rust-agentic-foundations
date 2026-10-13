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
        let Heads::Code { test } = heads else {
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
                    if let Some((bound, from_glob)) = self.lookup(from, ns, first) {
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
                match self.lookup(module, ns, seg) {
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
        self.uses
            .iter()
            .enumerate()
            .filter(|(position, decl)| {
                !(if resolved_mode {
                    self.use_resolved[*position]
                } else {
                    self.use_settled[*position]
                }) && decl.module == source.module
                    && matches!(&decl.kind, UseKind::Named { name, .. } if *name == source.name)
            })
            .map(|(position, _)| position)
            .collect()
    }

    /// The imports that the import at `position` waits on.
    pub(crate) fn blockers(&self, position: usize, resolved_mode: bool) -> Vec<usize> {
        let decl = &self.uses[position];
        let UseKind::Named { segs, .. } = &decl.kind else {
            return Vec::new();
        };
        let mut blockers = BTreeSet::new();
        for ns in Namespace::ALL {
            if let Err(Fail::Missing(sources)) = self.walk(
                decl.module,
                segs,
                decl.leading_colon,
                ns,
                Heads::Code { test: decl.test },
            ) {
                for source in sources {
                    blockers.extend(self.unresolved_uses(&source, resolved_mode));
                }
            }
        }
        blockers.into_iter().collect()
    }

    /// True when the unresolved imports reachable from `start` contain a cycle.
    fn depends_on_a_cycle(&self, start: usize) -> bool {
        let mut state: BTreeMap<usize, bool> = BTreeMap::new();
        let mut stack: Vec<(usize, Vec<usize>, usize)> =
            vec![(start, self.blockers(start, true), 0)];
        state.insert(start, true);
        while let Some((node, successors, at)) = stack.pop() {
            if at < successors.len() {
                let next = successors[at];
                stack.push((node, successors, at + 1));
                match state.get(&next) {
                    Some(true) => return true,
                    Some(false) => {}
                    None => {
                        state.insert(next, true);
                        let following = self.blockers(next, true);
                        stack.push((next, following, 0));
                    }
                }
            } else {
                state.insert(node, false);
            }
        }
        false
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
                        Err(Fail::SeveralCrates { roots, .. }) => {
                            return Err(ResolveError::Ambiguous(roots));
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
            for source in &missing {
                for position in self.unresolved_uses(source, true) {
                    if self.depends_on_a_cycle(position) {
                        return Err(ResolveError::Cyclic(text.to_owned()));
                    }
                }
            }
            return Err(ResolveError::NotFound(text.to_owned()));
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
