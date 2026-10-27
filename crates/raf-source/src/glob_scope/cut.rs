// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Exact holder membership and dominance for one admitted discovery reader.

use super::{
    GlobRead, Holding, HoldingKey, Index, LoadError, NativeAdmission, OrderedTable, Place, Steps,
    StorageError, Supply, Transient, UseDecl, Work, WorkList, search_units, tally,
};

mod marks;
use marks::Marks;

/// Only a module with multiple raw reads can request discovery dominance.
pub(super) fn reader(reads: &[Vec<GlobRead>]) -> Option<usize> {
    let mut candidates = reads
        .iter()
        .enumerate()
        .filter(|(_, reads)| reads.len() >= 2);
    match (candidates.next(), candidates.next()) {
        (Some((reader, _)), None) => Some(reader),
        _ => None,
    }
}

/// A cut certificate never represents dominance by an arbitrary reader.
pub(super) struct CutHolding {
    /// Zero is absent; one avoids the reader; two is reached only through it.
    holders: Marks,
    firm_equivalent: bool,
}

impl CutHolding {
    pub(super) fn answer(
        &self,
        from: usize,
        returning: bool,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
    ) -> Result<bool, LoadError> {
        Ok(self
            .holders
            .get(from, charge)?
            .is_some_and(|mark| mark == if returning { 2 } else { 1 }))
    }
}

/// These caches have different proof domains and cannot exchange their payloads.
#[derive(Default)]
pub(super) struct Cuts {
    rows: OrderedTable<(usize, HoldingKey), CutHolding>,
    live: usize,
}

pub(super) enum Returning<'a> {
    Full(&'a Holding, Option<(usize, usize)>),
    Cut(&'a CutHolding),
}

impl Returning<'_> {
    pub(super) fn returns(
        &self,
        from: usize,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
    ) -> Result<bool, LoadError> {
        Ok(match self {
            Self::Full(proof, Some(outer)) => proof
                .span
                .get(&from)
                .is_some_and(|inner| outer.0 <= inner.0 && inner.1 <= outer.1),
            Self::Full(_, None) => false,
            Self::Cut(proof) => return proof.answer(from, true, charge),
        })
    }
}

/// Resume precisely this native allocation account, including failed growth.
fn admitted<T>(
    memory: &Transient<'_>,
    decl: &UseDecl,
    live: &mut usize,
    action: impl FnOnce(
        &mut purrdf_lex::allocation::Memory<'_, NativeAdmission<'_>>,
    ) -> Result<T, StorageError>,
) -> Result<T, LoadError> {
    let mut admission = NativeAdmission::resume(
        Steps::LoadAt {
            memory,
            file: &decl.file,
            line: decl.line,
        },
        *live,
    );
    let result = {
        let mut native = purrdf_lex::allocation::Memory::resume(&mut admission, *live);
        let result = action(&mut native);
        *live = native.admitted_bytes();
        result
    };
    result.map_err(|error| admission.graph_error(error.into()))
}

struct Walk<'s, 'm> {
    /// The requested name/namespace has exactly one place per dense module ID.
    holders: Marks,
    retained: &'s Transient<'m>,
    modules: usize,
    /// Only named renamings require an additional sparse place spelling.
    visited: OrderedTable<Place, ()>,
    pending: WorkList<Place, 8>,
    live: usize,
    cut: Place,
    reader_reachable: bool,
}

impl Walk<'_, '_> {
    fn visit(
        &mut self,
        place: &Place,
        returning: bool,
        memory: &Transient<'_>,
        decl: &UseDecl,
        step: &mut dyn FnMut(usize) -> Result<(), LoadError>,
    ) -> Result<(), LoadError> {
        step(1 + place.1.len())?;
        let requested = place.1 == self.cut.1 && place.2 == self.cut.2;
        // Remove exactly the queried name/namespace place, not all the
        // reader's places: a renamed named import is a different vertex.
        if !returning && requested && place.0 == self.cut.0 {
            self.reader_reachable = true;
            return Ok(());
        }
        if requested {
            if !self
                .holders
                .mark(place.0, self.modules, returning, self.retained, decl, step)?
            {
                return Ok(());
            }
        } else {
            step((6 + place.1.len()).saturating_mul(4 * search_units(self.visited.len())))?;
            if self.visited.get(place).is_some() {
                return Ok(());
            }
        }
        admitted(memory, decl, &mut self.live, |native| {
            if !requested {
                self.visited.insert(place.clone(), (), native)?;
            }
            self.pending.try_push_admitted(place.clone(), native)
        })
    }

    fn drain(
        &mut self,
        index: &Index,
        supply: &Supply<'_>,
        key: &HoldingKey,
        returning: bool,
        memory: &Transient<'_>,
        step: &mut dyn FnMut(usize) -> Result<(), LoadError>,
    ) -> Result<bool, LoadError> {
        let decl = &index.uses[index.glob_imports[0]];
        let mut firm_equivalent = true;
        while let Some(place) = self.pending.pop() {
            step(1)?;
            firm_equivalent &=
                index.visit_holder_routes(supply, key, &place, step, |destination, step| {
                    self.visit(destination, returning, memory, decl, step)
                })?;
        }
        Ok(firm_equivalent)
    }
}

impl Index {
    pub(super) fn retained_cut<'s>(
        &self,
        supply: &'s mut Supply<'_>,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
        reader: usize,
        query: &HoldingKey,
    ) -> Result<&'s CutHolding, LoadError> {
        #[cfg(test)]
        super::reference::retrieved_holder();
        let mut other = query.clone();
        other.4 = !other.4;
        let other = (reader, other);
        charge(1 + (1 + query.1.len()).saturating_mul(8 * search_units(supply.cuts.rows.len())))?;
        let key = if supply
            .cuts
            .rows
            .get(&other)
            .is_some_and(|proof| proof.firm_equivalent)
        {
            other
        } else {
            (reader, query.clone())
        };
        if supply.cuts.rows.get(&key).is_none() {
            let proof = self.cut_holding(supply, charge, reader, &key.1)?;
            let decl = &self.uses[self.glob_imports[0]];
            admitted(&supply.memory, decl, &mut supply.cuts.live, |native| {
                supply.cuts.rows.insert(key.clone(), proof, native)
            })?;
        }
        Ok(supply
            .cuts
            .rows
            .get(&key)
            .expect("the exact reader-qualified proof was admitted"))
    }

    fn cut_holding(
        &self,
        supply: &Supply<'_>,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
        reader: usize,
        query: &HoldingKey,
    ) -> Result<CutHolding, LoadError> {
        let (id, name, at, required, carried) = query;
        let memory = supply.memory.child();
        let decl = &self.uses[self.glob_imports[0]];
        let mut step = |units| {
            tally::add(Work::SupplySteps, units);
            charge(units)
        };
        let mut walk = Walk {
            holders: Marks::default(),
            retained: &supply.memory,
            modules: self.modules.len(),
            visited: OrderedTable::default(),
            pending: WorkList::new(),
            live: 0,
            cut: (reader, name.clone(), *at),
            reader_reachable: false,
        };
        let mut firm_equivalent = true;
        for (root, level) in supply.roots.get(id).into_iter().flatten() {
            step(1)?;
            if root.2 != *at || !self.level_covers(*level, *required, &mut step)? {
                continue;
            }
            firm_equivalent &= !level.is_companion();
            if !level.is_companion() || *carried {
                walk.visit(root, false, &memory, decl, &mut step)?;
            }
        }
        firm_equivalent &= walk.drain(self, supply, query, false, &memory, &mut step)?;
        if walk.reader_reachable {
            let cut = walk.cut.clone();
            walk.visit(&cut, true, &memory, decl, &mut step)?;
            firm_equivalent &= walk.drain(self, supply, query, true, &memory, &mut step)?;
        }
        // Phase one contains exactly paths avoiding the reader. Phase two
        // adds precisely the remaining reachable places, each expanded once.
        // Both phases inspect every reachable eligible route, preserving the
        // full graph's certificate for sharing firm/carried modes.
        Ok(CutHolding {
            holders: walk.holders,
            firm_equivalent,
        })
    }

    /// One policy for actual outgoing edges, shared by both proof algorithms.
    pub(super) fn visit_holder_routes(
        &self,
        supply: &Supply<'_>,
        query: &HoldingKey,
        held: &Place,
        step: &mut dyn FnMut(usize) -> Result<(), LoadError>,
        mut visit: impl FnMut(
            &Place,
            &mut dyn FnMut(usize) -> Result<(), LoadError>,
        ) -> Result<(), LoadError>,
    ) -> Result<bool, LoadError> {
        let (id, _, _, required, carried) = query;
        let mut firm_equivalent = true;
        for (destination, level) in supply.named.get(&(*id, held.clone())).into_iter().flatten() {
            step(1)?;
            let visible = self.level_covers(*level, *required, step)?;
            let promoted =
                !visible && self.promoted_companion(destination, *id, *required, step)?;
            if visible || promoted {
                let companion = level.is_companion() || promoted;
                firm_equivalent &= !companion;
                if !companion || *carried {
                    visit(destination, step)?;
                }
            }
        }
        for &(module, at_read, glob_level) in &supply.readers[held.0] {
            step(1)?;
            if self.binds_itself(module, &held.1, held.2) {
                continue;
            }
            let view = self.globs.reads[module][at_read].view;
            let Some(level) = self.held_level(view, &held.1, held.2, *id) else {
                continue;
            };
            let destination = (module, held.1.clone(), held.2);
            let visible = self.level_covers(glob_level, *required, step)?
                && self.level_covers(level, *required, step)?;
            let promoted =
                !visible && self.promoted_companion(&destination, *id, *required, step)?;
            if visible || promoted {
                let companion = level.is_companion() || promoted;
                firm_equivalent &= !companion;
                if !companion || *carried {
                    visit(&destination, step)?;
                }
            }
        }
        Ok(firm_equivalent)
    }
}
