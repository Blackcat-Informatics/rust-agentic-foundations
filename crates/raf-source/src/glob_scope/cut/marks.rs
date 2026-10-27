// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Membership storage bounded by distinct actually reached requested places.

use super::{LoadError, OrderedTable, Transient, UseDecl, admitted, search_units};

/// A dense byte costs at most this many bytes per actually reached module.
const DENSE_RATIO: usize = 64;

pub(super) enum Marks {
    Sparse {
        rows: OrderedTable<usize, u8>,
        live: usize,
    },
    Dense(Vec<u8>),
}

impl Default for Marks {
    fn default() -> Self {
        Self::Sparse {
            rows: OrderedTable::default(),
            live: 0,
        }
    }
}

impl Marks {
    pub(super) fn get(
        &self,
        module: usize,
        charge: &dyn Fn(usize) -> Result<(), LoadError>,
    ) -> Result<Option<u8>, LoadError> {
        Ok(match self {
            Self::Sparse { rows, .. } => {
                charge(2 * search_units(rows.len()))?;
                rows.get(&module).copied()
            }
            Self::Dense(rows) => {
                charge(1)?;
                rows.get(module).copied()
            }
        })
    }

    /// Return whether this is a new place, preserving its first reachability phase.
    pub(super) fn mark(
        &mut self,
        module: usize,
        modules: usize,
        returning: bool,
        memory: &Transient<'_>,
        decl: &UseDecl,
        step: &mut dyn FnMut(usize) -> Result<(), LoadError>,
    ) -> Result<bool, LoadError> {
        let value = if returning { 2 } else { 1 };
        match self {
            Self::Dense(rows) => {
                step(1)?;
                if rows[module] != 0 {
                    return Ok(false);
                }
                rows[module] = value;
            }
            Self::Sparse { rows, live } => {
                step(4 * search_units(rows.len()))?;
                if rows.get(&module).is_some() {
                    return Ok(false);
                }
                admitted(memory, decl, live, |native| {
                    rows.insert(module, value, native)
                })?;
                if rows.len() >= modules.div_ceil(DENSE_RATIO) {
                    self.promote(modules, memory, decl, step)?;
                }
            }
        }
        Ok(true)
    }

    /// Admit the overlap, copy every phase mark, then release the original arena.
    fn promote(
        &mut self,
        modules: usize,
        memory: &Transient<'_>,
        decl: &UseDecl,
        step: &mut dyn FnMut(usize) -> Result<(), LoadError>,
    ) -> Result<(), LoadError> {
        let Self::Sparse { rows, .. } = self else {
            unreachable!("only sparse membership promotes")
        };
        step(modules.saturating_add(rows.len()))?;
        let mut dense = Vec::new();
        admitted(memory, decl, &mut 0, |native| {
            native.reserve(&mut dense, modules)
        })?;
        dense.resize(modules, 0);
        for (&module, &mark) in rows.ordered() {
            dense[module] = mark;
        }
        let previous = std::mem::replace(self, Self::Dense(dense));
        let Self::Sparse { rows, mut live } = previous else {
            unreachable!("the original sparse account remains exclusive")
        };
        admitted(memory, decl, &mut live, |native| rows.release(native))
    }
}
