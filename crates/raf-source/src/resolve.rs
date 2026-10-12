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
    /// items, two imports of one name, or two canonical targets.
    Ambiguous(Vec<ItemId>),
    /// The imports that the path goes through depend on each other.
    Cyclic(String),
    /// The text is not a path.
    Invalid(String),
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(path) => write!(f, "`{path}` names nothing in the workspace"),
            Self::Ambiguous(ids) => write!(f, "the path names {} different symbols", ids.len()),
            Self::Cyclic(path) => write!(f, "`{path}` goes through imports that form a cycle"),
            Self::Invalid(text) => write!(f, "`{text}` is not a path"),
        }
    }
}

impl Error for ResolveError {}

fn strip_raw(segment: &str) -> &str {
    segment.strip_prefix("r#").map_or(segment, |plain| plain)
}

impl Index {
    /// The state after the first segment of a path, and the index of the
    /// segment to read next.
    fn start(
        &self,
        from: usize,
        segs: &[String],
        leading_colon: bool,
        last_ns: Namespace,
        prelude: bool,
    ) -> Result<(Walked, usize), Fail> {
        let mut state = Walked::default();
        let Some(first) = segs.first() else {
            return Err(Fail::Missing(Vec::new()));
        };
        let mut next_index = 1;
        if leading_colon {
            let Some(&root) = self.libs.get(first.as_str()) else {
                return Err(Fail::Missing(Vec::new()));
            };
            state.modules.insert(root);
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
                    } else if let (true, Some(&root)) = (prelude, self.libs.get(first.as_str())) {
                        state.modules.insert(root);
                    } else {
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
        prelude: bool,
    ) -> Result<Walked, Fail> {
        let (mut state, next_index) = self.start(from, segs, leading_colon, last_ns, prelude)?;
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
                if let Some(module) = self.module_for(*id) {
                    containers.insert(module);
                }
            }
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
            if let Err(Fail::Missing(sources)) =
                self.walk(decl.module, segs, decl.leading_colon, ns, true)
            {
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
        entry.defs[ns]
            .get(&id)
            .or_else(|| entry.globbed[ns].get(&id))
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

    /// Resolve a canonical path in one name space.
    fn resolve_in(
        &self,
        segs: &[String],
        ns: Namespace,
        text: &str,
    ) -> Result<Resolution, ResolveError> {
        let mut found: BTreeMap<ItemId, Hit> = BTreeMap::new();
        let mut missing: Vec<Source> = Vec::new();
        for taken in 1..segs.len() {
            let prefix = segs[..taken].join("::");
            let Some(&root) = self.crate_roots.get(&prefix) else {
                continue;
            };
            match self.walk(root, &segs[taken..], false, ns, false) {
                Ok(walked) => {
                    for (id, hit) in walked.items {
                        merge(&mut found, id, hit);
                    }
                }
                Err(Fail::Missing(sources)) => missing.extend(sources),
                Err(Fail::SuperAboveRoot) => {}
            }
        }
        // A path that is exactly a crate path names the crate root, a module.
        // If a first-party item already has that path, the path names two
        // things and is ambiguous.
        if ns == Namespace::Type
            && let Some(root) = self
                .crate_roots
                .get(text)
                .and_then(|module| self.root_items.get(module))
        {
            if !found.is_empty() {
                let mut all: Vec<ItemId> = found.keys().copied().collect();
                all.push(*root);
                return Err(ResolveError::Ambiguous(self.sorted(all)));
            }
            found.insert(
                *root,
                Hit {
                    via_glob: false,
                    last: None,
                },
            );
        }
        if let (true, Some(ids)) = (found.is_empty(), self.foreign.get(text)) {
            for id in ids {
                if self.items[id.0].in_namespace(ns) {
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
        let mut groups: BTreeMap<(String, SymbolKind), Vec<ItemId>> = BTreeMap::new();
        for id in found.keys() {
            let item = &self.items[id.0];
            groups
                .entry((item.path.clone(), item.kind))
                .or_default()
                .push(*id);
        }
        if groups.len() > 1 {
            return Err(ResolveError::Ambiguous(self.sorted(found.keys().copied())));
        }
        let sites = self.sorted(found.keys().copied());
        if let Some(shadow) = self.shadowed_namespace(&sites) {
            let mut all = shadow;
            all.extend(sites);
            return Err(ResolveError::Ambiguous(self.sorted(all)));
        }
        let first = sites[0];
        let hit = &found[&first];
        Ok(Resolution {
            path: self.items[first.0].path.clone(),
            namespace: ns,
            via_glob: hit.via_glob,
            chain: self.chain_for(text, hit, first),
            sites,
        })
    }

    /// When the crate path of the target that holds `sites` runs through a
    /// module of another crate (`<pkg>::test_crate` is a module of the
    /// library and the start of every test crate path), the name stands for
    /// both, so the modules' declarations are returned.
    fn shadowed_namespace(&self, sites: &[ItemId]) -> Option<Vec<ItemId>> {
        let target = &self.targets[self.items[sites[0].0].target.index()];
        let mut shadow = Vec::new();
        let segments: Vec<&str> = target.crate_path.split("::").collect();
        for taken in 1..segments.len() {
            let prefix = segments[..taken].join("::");
            if self.crate_roots.contains_key(&prefix) {
                continue;
            }
            shadow.extend(
                self.items
                    .iter()
                    .filter(|item| item.kind == SymbolKind::Mod && item.path == prefix)
                    .map(|item| item.id),
            );
        }
        if shadow.is_empty() {
            None
        } else {
            Some(shadow)
        }
    }

    /// Resolve `path`, trying the type, value and macro name spaces in turn.
    pub(crate) fn resolve(
        &self,
        path: &str,
        ns: Option<Namespace>,
    ) -> Result<Resolution, ResolveError> {
        let segs: Vec<String> = path.split("::").map(|s| strip_raw(s).to_owned()).collect();
        let valid =
            |s: &String| !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_');
        if !segs.iter().all(valid) {
            return Err(ResolveError::Invalid(path.to_owned()));
        }
        let text = segs.join("::");
        let spaces: Vec<Namespace> = ns.map_or_else(|| Namespace::ALL.to_vec(), |one| vec![one]);
        let mut last = ResolveError::NotFound(text.clone());
        for space in spaces {
            match self.resolve_in(&segs, space, &text) {
                Ok(resolution) => return Ok(resolution),
                Err(ResolveError::NotFound(_)) => {}
                Err(other) => {
                    last = other;
                    break;
                }
            }
        }
        Err(last)
    }
}
