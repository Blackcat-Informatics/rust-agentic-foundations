// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Whole-import determinacy of the actual routes carrying a selected item.

use std::collections::BTreeSet;

use super::{Fail, Heads, Index, ItemId, LoadError, Namespace, ResolveError, Steps};
use crate::cost::{btree_entry, btree_map};
use crate::glob_scope::{Level, reserve_next};
use crate::index::UseKind;
use crate::meter::{NativeAdmission, Transient};
use crate::settle::{ImportReadState, slot};

#[cfg(test)]
mod tests;

/// A selected binding retains the context under which it was read. Names
/// borrow only stable index storage, never a temporary walk's Sources.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Selected<'a> {
    module: usize,
    ns: Namespace,
    name: &'a str,
    id: ItemId,
    own: Option<usize>,
    viewer: Option<usize>,
}

struct Proof<'a, 'm> {
    index: &'a Index,
    steps: Steps<'m>,
    seen: BTreeSet<Selected<'a>>,
    pending: purrdf_lex::walk::WorkList<Selected<'a>, 8>,
    imports: BTreeSet<usize>,
}

impl<'a, 'm> Proof<'a, 'm> {
    fn new(index: &'a Index, steps: Steps<'m>) -> Self {
        Self {
            index,
            steps,
            seen: BTreeSet::new(),
            pending: purrdf_lex::walk::WorkList::new(),
            imports: BTreeSet::new(),
        }
    }

    fn add(&mut self, selected: Selected<'a>) -> Result<(), LoadError> {
        // Comparison costs include identifier text, including an unsuccessful
        // membership test. Native vector growth admits coexisting buffers.
        self.steps.storage_work(
            2 * (selected.name.len() + 1)
                .saturating_mul(self.seen.len().max(1).ilog2() as usize + 1),
        )?;
        if !self.seen.contains(&selected) {
            self.steps.reserve(
                btree_entry::<Selected<'_>, ()>()
                    + if self.seen.is_empty() {
                        btree_map::<Selected<'_>, ()>()
                    } else {
                        0
                    },
            )?;
            let live = self.pending.heap_capacity() * size_of::<Selected<'_>>();
            let mut admission = NativeAdmission::resume(self.steps, live);
            self.pending
                .try_push_admitted(
                    selected,
                    &mut purrdf_lex::allocation::Memory::resume(&mut admission, live),
                )
                .map_err(|error| admission.graph_error(error.into()))?;
            self.seen.insert(selected);
        }
        Ok(())
    }

    /// Rewalk adjacent prefixes, carrying only predecessors of the chosen
    /// next segment. Enumerate each actual container lookup: neither flat
    /// reads nor a preferred Hit can retain alternative same-item routes.
    fn path(
        &mut self,
        from: usize,
        segs: &[String],
        colon: bool,
        ns: Namespace,
        heads: Heads,
        selected: Option<ItemId>,
    ) -> Result<(), LoadError> {
        let path_memory = self.steps.temporary();
        let path_steps = path_memory
            .as_ref()
            .map_or(self.steps, |memory| self.steps.at(memory));
        let mut wanted = Vec::new();
        let mut previous = Vec::new();
        {
            let memory = path_steps.temporary();
            let steps = memory
                .as_ref()
                .map_or(path_steps, |memory| path_steps.at(memory));
            steps.storage_work(segs.len())?;
            let Some(walked) = self.walk(from, segs, colon, ns, heads, steps)? else {
                return Ok(());
            };
            for id in walked.items.keys() {
                steps.storage_work(1)?;
                if selected.is_none_or(|selected| *id == selected) {
                    reserve_next(&mut wanted, path_steps)?;
                    wanted.push(*id);
                }
            }
        }
        let viewer = matches!(heads, Heads::Use { .. }).then_some(from);
        for offset in (0..segs.len()).rev() {
            if wanted.is_empty() {
                break;
            }
            let memory = path_steps.temporary();
            let steps = memory
                .as_ref()
                .map_or(path_steps, |memory| path_steps.at(memory));
            let space = if offset + 1 == segs.len() {
                ns
            } else {
                Namespace::Type
            };
            let segment = &segs[offset];
            if offset == 0 {
                self.head(from, &segs[..1], colon, space, heads, &wanted, steps)?;
                break;
            }
            steps.storage_work(offset)?;
            let Some(prefix) =
                self.walk(from, &segs[..offset], colon, Namespace::Type, heads, steps)?
            else {
                break;
            };
            // Prefix items retain every selected predecessor id, even when
            // several containers deliver the same terminal id.
            previous.clear();
            for &owner in prefix.items.keys() {
                let mut contributes = false;
                for target in self.index.modules_for(owner) {
                    steps.storage_work(1)?;
                    contributes |= self.select_at(
                        target,
                        space,
                        segment,
                        heads.own(),
                        viewer,
                        &wanted,
                        steps,
                    )?;
                }
                if let Some(ids) = self
                    .index
                    .assoc
                    .get(&owner)
                    .and_then(|names| names.get(segment.as_str()))
                {
                    for id in ids {
                        steps.storage_work(1 + wanted.len())?;
                        contributes |= wanted.contains(id);
                    }
                }
                if contributes {
                    reserve_next(&mut previous, path_steps)?;
                    previous.push(owner);
                }
            }
            for target in prefix.modules {
                steps.storage_work(1)?;
                self.select_at(target, space, segment, heads.own(), viewer, &wanted, steps)?;
            }
            std::mem::swap(&mut wanted, &mut previous);
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the path head retains edition, visibility, own-import and selected-item context"
    )]
    fn head(
        &mut self,
        from: usize,
        segs: &[String],
        colon: bool,
        ns: Namespace,
        heads: Heads,
        wanted: &[ItemId],
        steps: Steps<'_>,
    ) -> Result<(), LoadError> {
        let mut reads = Vec::new();
        steps.storage_work(1)?;
        match self
            .index
            .walk_noting(from, segs, colon, ns, heads, Some(&mut reads), steps)
        {
            Ok(_) => {}
            Err(Fail::Storage(error)) => return Err(error),
            Err(Fail::Budget(refused)) => return Err(self.index.refused_load(refused)),
            Err(_) => return Ok(()),
        }
        let viewer = matches!(heads, Heads::Use { .. }).then_some(from);
        for source in reads {
            self.select_at(
                source.module,
                ns,
                &source.name,
                heads.own(),
                viewer,
                wanted,
                steps,
            )?;
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the exact selected lookup includes namespace, own-import and visibility context"
    )]
    fn select_at(
        &mut self,
        module: usize,
        ns: Namespace,
        name: &str,
        own: Option<usize>,
        viewer: Option<usize>,
        wanted: &[ItemId],
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        let read = self
            .index
            .read_lookup(module, ns, name, own, viewer, steps)
            .map_err(|failure| self.failure(failure))?;
        let Some((bound, _)) = read.as_ref() else {
            return Ok(false);
        };
        let mut found = false;
        for id in bound.keys() {
            steps.storage_work(1 + wanted.len())?;
            if wanted.contains(id) {
                let name = self
                    .index
                    .bound_name(module, name)
                    .expect("a selected binding has a stable spelling");
                self.add(Selected {
                    module,
                    ns,
                    name,
                    id: *id,
                    own,
                    viewer,
                })?;
                found = true;
            }
        }
        Ok(found)
    }

    fn walk(
        &self,
        from: usize,
        segs: &[String],
        colon: bool,
        ns: Namespace,
        heads: Heads,
        steps: Steps<'_>,
    ) -> Result<Option<super::Walked>, LoadError> {
        match self.index.walk(from, segs, colon, ns, heads, steps) {
            Ok(walked) => Ok(Some(walked)),
            Err(Fail::Storage(error)) => Err(error),
            Err(Fail::Budget(refused)) => Err(self.index.refused_load(refused)),
            Err(_) => Ok(None),
        }
    }

    fn import(&mut self, position: usize) -> Result<(), LoadError> {
        self.steps
            .storage_work(2 * (self.imports.len().max(1).ilog2() as usize + 1))?;
        if self.imports.contains(&position) {
            return Ok(());
        }
        self.steps.reserve(
            btree_entry::<usize, ()>()
                + if self.imports.is_empty() {
                    btree_map::<usize, ()>()
                } else {
                    0
                },
        )?;
        self.imports.insert(position);
        let decl = &self.index.uses[position];
        let (segs, spaces) = match &decl.kind {
            UseKind::Named { segs, .. } => (segs, Namespace::ALL.as_slice()),
            UseKind::Glob { segs } => (segs, std::slice::from_ref(&Namespace::Type)),
        };
        for &ns in spaces {
            self.path(
                decl.module,
                segs,
                decl.leading_colon,
                ns,
                Heads::Use {
                    test: decl.test,
                    own: Some(position),
                },
                None,
            )?;
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<Option<usize>, LoadError> {
        while let Some(place) = self.pending.pop() {
            self.steps.storage_work(1)?;
            let memory = self.steps.temporary();
            let steps = memory
                .as_ref()
                .map_or(self.steps, |memory| self.steps.at(memory));
            let read = self
                .index
                .read_lookup(
                    place.module,
                    place.ns,
                    place.name,
                    place.own,
                    place.viewer,
                    steps,
                )
                .map_err(|failure| self.failure(failure))?;
            let Some((bound, globbed)) = read.as_ref() else {
                continue;
            };
            if bound.get(place.id).is_none() {
                continue;
            }
            if globbed {
                self.glob(place, steps)?;
            } else {
                // Genuine definitions stop this derivation. No unrelated
                // same-name import or glob is consulted behind a definition.
                if bound
                    .get(place.id)
                    .is_some_and(|origin| origin.link.is_none())
                {
                    continue;
                }
                for &position in self.index.imports_of(place.module, place.name) {
                    steps.storage_work(1)?;
                    if place.own == Some(position) {
                        continue;
                    }
                    let supply = self.index.import_supply(position, steps)?;
                    let mut supplies = false;
                    for route in supply {
                        steps.storage_work(1)?;
                        if route.ns == place.ns
                            && route.id == place.id
                            && (place.viewer.is_none()
                                || self.index.level_covers(
                                    route.level,
                                    Level::within(place.viewer.expect("a viewer")),
                                    &mut |units| steps.storage_work(units),
                                )?)
                        {
                            supplies = true;
                        }
                    }
                    if !supplies {
                        continue;
                    }
                    if self.index.use_read_state[position].contains(&ImportReadState::Late) {
                        return Ok(Some(position));
                    }
                    self.import(position)?;
                }
            }
        }
        Ok(None)
    }

    fn glob(&mut self, place: Selected<'a>, steps: Steps<'_>) -> Result<(), LoadError> {
        // An outward read can reach globs behind an invisible named
        // binding only through this precise peer-export exception.
        // Its peer and hidden named suppliers are dependencies too.
        if let Some(viewer) = place.viewer.filter(|viewer| *viewer != place.module)
            && let Some(defs) = self.index.scopes[place.module]
                .get(place.name)
                .map(|entry| &entry.defs[slot(place.ns)])
                .filter(|defs| !defs.is_empty())
            && let Some((peer, id)) = self
                .index
                .forwarded_peer_export(
                    (place.module, place.ns, place.name),
                    defs,
                    place.own,
                    viewer,
                    steps,
                )
                .map_err(|failure| self.failure(failure))?
        {
            self.add(Selected {
                ns: peer,
                id,
                viewer: None,
                ..place
            })?;
            for id in defs.keys() {
                steps.storage_work(1)?;
                self.add(Selected {
                    id: *id,
                    viewer: None,
                    ..place
                })?;
            }
        }
        let mut routes = Vec::new();
        self.index
            .delivered_item_sources(
                place.module,
                place.name,
                slot(place.ns),
                place.id,
                place.viewer,
                steps,
                &mut |target, glob| {
                    reserve_next(&mut routes, steps)?;
                    routes.push((target, glob));
                    Ok(())
                },
            )
            .map_err(|failure| self.failure(failure))?;
        for (target, glob) in routes {
            self.import(glob)?;
            let name = self
                .index
                .bound_name(target, place.name)
                .expect("a delivered item has a stable spelling");
            self.add(Selected {
                module: target,
                name,
                own: None,
                viewer: Some(place.module),
                ..place
            })?;
        }
        Ok(())
    }

    fn failure(&self, failure: Fail) -> LoadError {
        match failure {
            Fail::Storage(error) => error,
            Fail::Budget(refused) => self.index.refused_load(refused),
            _ => unreachable!("a selected binding lookup only fails on storage or work admission"),
        }
    }
}

impl Index {
    fn late_error(&self, position: usize, text: &str) -> ResolveError {
        let decl = &self.uses[position];
        ResolveError::IndeterminateImport {
            path: text.to_owned(),
            file: decl.file.clone(),
            line: decl.line,
        }
    }

    /// Shared query/export proof with real transient admission and no
    /// persistent route cache or preferred-Origin shortcut.
    fn checked_determinacy<'a>(
        &'a self,
        module: usize,
        text: &str,
        steps: Steps<'_>,
        seed: impl FnOnce(&mut Proof<'a, '_>) -> Result<(), LoadError>,
    ) -> Result<(), ResolveError> {
        if !self.has_late_imports {
            return Ok(());
        }
        let meter = steps
            .query_meter()
            .expect("semantic determinacy is checked after loading");
        let memory = Transient::new(meter);
        let (file, line) = self.module_site(module);
        let steps = Steps::QueryAt {
            memory: &memory,
            file: &file,
            line,
        };
        let mut proof = Proof::new(self, steps);
        let result = seed(&mut proof).and_then(|()| proof.finish());
        // A lookup-budget refusal takes precedence over a found semantic
        // reason, as do fallible scratch/work reservations.
        Self::query_refusal(steps).map_err(|refused| self.refused(refused))?;
        match result.map_err(ResolveError::storage)? {
            Some(position) => Err(self.late_error(position, text)),
            None => Ok(()),
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the original path and lookup context seed a selected semantic proof"
    )]
    pub(super) fn path_determinacy(
        &self,
        from: usize,
        segs: &[String],
        colon: bool,
        ns: Namespace,
        heads: Heads,
        text: &str,
        steps: Steps<'_>,
    ) -> Result<(), ResolveError> {
        self.checked_determinacy(from, text, steps, |proof| {
            proof.path(from, segs, colon, ns, heads, None)
        })
    }

    pub(super) fn reach_binding_determinacy(
        &self,
        walked: &super::Walked,
        reads: &[super::Source],
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        if !self.has_late_imports {
            return Ok(true);
        }
        steps.reserve(walked.items.len().saturating_mul(size_of::<ItemId>()))?;
        let wanted: Vec<_> = walked.items.keys().copied().collect();
        let mut proof = Proof::new(self, steps);
        for source in reads {
            proof.select_at(
                source.module,
                source.ns,
                &source.name,
                None,
                None,
                &wanted,
                steps,
            )?;
        }
        let late = proof.finish()?;
        Self::query_refusal(steps).map_err(|refused| self.refused_load(refused))?;
        Ok(late.is_none())
    }

    pub(super) fn binding_determinacy(
        &self,
        module: usize,
        ns: Namespace,
        name: &str,
        ids: &[ItemId],
        text: &str,
        steps: Steps<'_>,
    ) -> Result<(), ResolveError> {
        self.checked_determinacy(module, text, steps, |proof| {
            let name = self
                .bound_name(module, name)
                .expect("a surface binding has a stable spelling");
            for &id in ids {
                proof.add(Selected {
                    module,
                    ns,
                    name,
                    id,
                    own: None,
                    viewer: None,
                })?;
            }
            Ok(())
        })
    }
}
