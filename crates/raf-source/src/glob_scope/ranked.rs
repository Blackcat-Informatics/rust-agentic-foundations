// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Constructive terminal validation in the PUBLIC-level peer-glob interval.

use super::{
    Bound, Cow, GlobRead, Held, Index, Level, LoadError, Meter, NativeAdmission, Steps, Transient,
    Vis, Work, WorkList, push_work, search_units, size_of, sort_units, tally,
};

#[cfg(test)]
mod tests;

/// One validation invocation; resident payload and frozen rows remain borrowed.
struct Ranked<'a, 'm> {
    index: &'a Index,
    meter: &'m Meter,
    memory: Transient<'m>,
    site: usize,
}

/// Reusable transfer buffers; their grant remains live across all comparisons.
#[derive(Default)]
struct Scratch {
    seen: Vec<Held>,
    hidden: Vec<Held>,
    copied: Vec<Held>,
    #[cfg(test)]
    closure_calls: usize,
}

struct Row<'a> {
    held: &'a [Held],
    defs: Option<&'a Bound>,
}

struct Read<'a> {
    source: &'a GlobRead,
    known: &'a [Held],
    glob: Level,
}

/// Read structure and written visibility do not depend on a name or namespace.
struct Topology<'a> {
    reads: Vec<(&'a GlobRead, Level)>,
    offsets: Vec<usize>,
}

/// Borrow each textual lookup once while all three namespaces are certified.
struct NameInputs<'a> {
    rows: Vec<Resident<'a>>,
    known: Vec<Option<&'a [Vec<Held>; 3]>>,
    bottom: [bool; 3],
}

struct Resident<'a> {
    held: Option<&'a [Vec<Held>; 3]>,
    defs: Option<&'a [Bound; 3]>,
}

impl<'a> NameInputs<'a> {
    fn namespace<'b>(&'b self, topology: &'b Topology<'a>, at: usize) -> Inputs<'a, 'b> {
        Inputs {
            rows: &self.rows,
            reads: &topology.reads,
            known: &self.known,
            offsets: &topology.offsets,
            at,
            bottom: self.bottom[at],
        }
    }
}

/// Borrowed only while all definitions, resident payload and recorded rows freeze.
struct Inputs<'a, 'b> {
    rows: &'b [Resident<'a>],
    reads: &'b [(&'a GlobRead, Level)],
    known: &'b [Option<&'a [Vec<Held>; 3]>],
    offsets: &'b [usize],
    at: usize,
    bottom: bool,
}

impl<'a> Inputs<'a, '_> {
    fn row(&self, view: usize) -> Row<'a> {
        let Resident { held, defs } = self.rows[view];
        Row {
            held: held.map_or(&[], |spaces| spaces[self.at].as_slice()),
            defs: defs
                .map(|spaces| &spaces[self.at])
                .filter(|defs| !defs.is_empty()),
        }
    }

    fn reads(&self, module: usize) -> impl ExactSizeIterator<Item = Read<'a>> + Clone + '_ {
        let range = if self.bottom {
            0..0
        } else {
            self.offsets[module]..self.offsets[module + 1]
        };
        self.reads[range.clone()]
            .iter()
            .zip(&self.known[range])
            .map(|((source, glob), known)| Read {
                source,
                known: known.map_or(&[], |spaces| spaces[self.at].as_slice()),
                glob: *glob,
            })
    }
}

#[derive(Clone, Copy, Default)]
struct Mark {
    certified: bool,
    queued: bool,
    closed: bool,
}

impl Index {
    /// Validate a rooted terminal table without using its cyclic facts as roots.
    /// Auxiliary history must never seed resumed propagation after success.
    pub(crate) fn ranked_glob_tables(&self, meter: &Meter) -> Result<bool, LoadError> {
        if !self.fixed_glob_paths(meter)? {
            return Ok(false);
        }
        let Some(&site) = self.glob_imports.first() else {
            return Ok(true);
        };
        let proof = Ranked {
            index: self,
            meter,
            memory: Transient::new(meter),
            site,
        };
        proof.tables()
    }
}

impl<'a> Ranked<'a, '_> {
    fn tables(&self) -> Result<bool, LoadError> {
        if !self.public_peer_interval()? {
            return Ok(false);
        }
        let names = self.names()?;
        if names.is_empty() {
            return Ok(true);
        }
        let memory = self.memory.child();
        let topology = self.topology(&memory)?;
        let mut scratch = Scratch::default();
        for name in names {
            let name_memory = self.memory.child();
            let facts = self.name_inputs(name, &topology, &name_memory)?;
            for at in 0..3 {
                let inputs = facts.namespace(&topology, at);
                if !self.certify(&inputs, &mut scratch, &memory)? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    fn work(&self, kind: Work, units: usize) -> Result<(), LoadError> {
        let decl = &self.index.uses[self.site];
        self.meter.work(units, &decl.file, Some(decl.line))?;
        tally::add(kind, units);
        Ok(())
    }

    fn reserve<T>(
        &self,
        values: &mut Vec<T>,
        required: usize,
        memory: &Transient<'_>,
    ) -> Result<(), LoadError> {
        self.work(Work::ProofSteps, 1)?;
        if required <= values.capacity() {
            return Ok(());
        }
        self.work(Work::ProofCopies, values.len())?;
        let decl = &self.index.uses[self.site];
        let bytes = values.capacity() * size_of::<T>();
        let mut admission = NativeAdmission::resume(
            Steps::LoadAt {
                memory,
                file: &decl.file,
                line: decl.line,
            },
            bytes,
        );
        purrdf_lex::allocation::Memory::resume(&mut admission, bytes)
            .reserve(values, required)
            .map_err(|error| admission.graph_error(error.into()))
    }

    /// This is an interval proof, not a check of final candidate levels alone.
    fn public_peer_interval(&self) -> Result<bool, LoadError> {
        if !self.index.named_imports.is_empty() {
            return Ok(false);
        }
        self.work(Work::ProofReads, self.index.globs.reads.len())?;
        for (module, reads) in self.index.globs.reads.iter().enumerate() {
            for read in reads {
                self.work(Work::ProofReads, 1)?;
                let parent = self.index.modules[module].parent;
                if parent.is_none()
                    || module == read.target
                    || parent != self.index.modules[read.target].parent
                    || !matches!(self.index.uses[read.glob].vis, Vis::Public | Vis::Inherited)
                {
                    return Ok(false);
                }
            }
        }
        for view in &self.index.globs.views {
            self.work(Work::ProofReads, 1 + view.class.len())?;
            if view.class.last() != Some(&false)
                || !view.class[..view.class.len() - 1]
                    .iter()
                    .all(|inside| *inside)
            {
                return Ok(false);
            }
            for entry in self.index.scopes[view.target].values() {
                self.work(Work::ProofReads, 4)?;
                for defs in &entry.defs {
                    for (_, origin) in defs.iter() {
                        self.work(Work::ProofReads, 1)?;
                        if origin.link.is_some() || origin.companion {
                            return Ok(false);
                        }
                        for vis in origin.visibilities() {
                            self.work(Work::ProofReads, 1)?;
                            if *vis != Vis::Public {
                                return Ok(false);
                            }
                        }
                    }
                }
            }
            for (name, lists) in &view.names {
                self.work(Work::ProofReads, 1 + name.len())?;
                if !self.index.globs.strings.contains(name.as_str()) {
                    return Ok(false);
                }
                for held in lists.as_ref() {
                    self.work(Work::ProofReads, held.len())?;
                    if held.iter().any(|(_, level)| level.plain() != Level::PUBLIC) {
                        return Ok(false);
                    }
                }
            }
        }
        for ((glob, _), names) in &self.index.globs.drops {
            self.work(Work::ProofReads, 1)?;
            let local = Level::within(self.index.uses[*glob].module);
            self.work(Work::ProofReads, names.capacity())?;
            for spaces in names.values() {
                self.work(Work::ProofReads, 4)?;
                for held in spaces {
                    self.work(Work::ProofReads, held.len())?;
                    if held.iter().any(|(_, level)| {
                        level.is_companion()
                            || (level.plain() != Level::PUBLIC && level.plain() != local)
                    }) {
                        return Ok(false);
                    }
                }
            }
        }
        Ok(true)
    }

    /// All resident input names plus own-scope names, even when outputs omit them.
    fn names(&self) -> Result<Vec<&'a str>, LoadError> {
        self.work(Work::ProofReads, self.index.globs.views.len())?;
        let own = self.index.globs.views.iter().fold(0usize, |total, view| {
            total.saturating_add(self.index.scopes[view.target].len())
        });
        let mut names = Vec::new();
        self.reserve(
            &mut names,
            self.index.globs.strings.len().saturating_add(own),
            &self.memory,
        )?;
        self.work(Work::ProofReads, self.index.globs.strings.capacity())?;
        for name in &self.index.globs.strings {
            names.push(name.as_str());
        }
        for view in &self.index.globs.views {
            self.work(Work::ProofReads, 1)?;
            for name in self.index.scopes[view.target].keys() {
                self.work(Work::ProofReads, 1 + name.len())?;
                if !self.index.globs.strings.contains(name.as_str()) {
                    names.push(name.as_str());
                }
            }
        }
        self.work(Work::ProofReads, names.len())?;
        let width = names.iter().map(|name| name.len()).max().unwrap_or(0);
        self.work(
            Work::ProofSteps,
            names.len().saturating_add(
                sort_units(names.len()).saturating_mul(1usize.saturating_add(width)),
            ),
        )?;
        names.sort_unstable();
        self.work(Work::ProofReads, names.len())?;
        self.work(
            Work::ProofSteps,
            names.iter().fold(0usize, |total, name| {
                total.saturating_add(1usize.saturating_add(name.len()))
            }),
        )?;
        names.dedup();
        Ok(names)
    }

    fn topology(&self, memory: &Transient<'_>) -> Result<Topology<'a>, LoadError> {
        let mut topology = Topology {
            reads: Vec::new(),
            offsets: Vec::new(),
        };
        self.work(Work::ProofReads, self.index.globs.reads.len())?;
        let count = self
            .index
            .globs
            .reads
            .iter()
            .fold(0usize, |total, reads| total.saturating_add(reads.len()));
        self.reserve(&mut topology.reads, count, memory)?;
        self.reserve(
            &mut topology.offsets,
            self.index.modules.len().saturating_add(1),
            memory,
        )?;
        self.work(Work::ProofReads, self.index.modules.len())?;
        for module in 0..self.index.modules.len() {
            self.work(Work::ProofCopies, 1)?;
            topology.offsets.push(topology.reads.len());
            for source in self.index.reads_of(module) {
                self.work(Work::ProofReads, 1)?;
                let glob =
                    self.index
                        .level_of(&self.index.uses[source.glob].vis, module, Steps::Load);
                self.work(Work::ProofCopies, 1)?;
                topology.reads.push((source, glob));
            }
        }
        self.work(Work::ProofCopies, 1)?;
        topology.offsets.push(topology.reads.len());
        Ok(topology)
    }

    fn name_inputs(
        &self,
        name: &str,
        topology: &Topology<'a>,
        memory: &Transient<'_>,
    ) -> Result<NameInputs<'a>, LoadError> {
        let mut inputs = NameInputs {
            rows: Vec::new(),
            known: Vec::new(),
            bottom: [true; 3],
        };
        self.reserve(&mut inputs.rows, self.index.globs.views.len(), memory)?;
        for view in &self.index.globs.views {
            self.work(Work::ProofReads, 8)?;
            let held: Option<&[Vec<Held>; 3]> = view.names.get(name).map(AsRef::as_ref);
            let defs = self.index.scopes[view.target]
                .get(name)
                .map(|entry| &entry.defs);
            for (at, bottom) in inputs.bottom.iter_mut().enumerate() {
                *bottom &= held.is_none_or(|spaces| spaces[at].is_empty())
                    && defs.is_none_or(|spaces| spaces[at].is_empty());
            }
            self.work(Work::ProofCopies, 1)?;
            inputs.rows.push(Resident { held, defs });
        }
        if inputs.bottom.iter().all(|bottom| *bottom) {
            return Ok(inputs);
        }
        self.reserve(&mut inputs.known, topology.reads.len(), memory)?;
        for (source, _) in &topology.reads {
            self.work(Work::ProofReads, 1)?;
            let known = self
                .index
                .globs
                .drops
                .get(&(source.glob, source.target))
                .and_then(|names| names.get(name));
            self.work(Work::ProofCopies, 1)?;
            inputs.known.push(known);
        }
        Ok(inputs)
    }

    fn enqueue(
        &self,
        pending: &mut WorkList<usize, 8>,
        view: usize,
        memory: &Transient<'_>,
    ) -> Result<(), LoadError> {
        self.work(Work::ProofSteps, 1)?;
        let spill = pending.len().saturating_sub(8);
        if spill == pending.heap_capacity() {
            self.work(Work::ProofCopies, spill)?;
        }
        let decl = &self.index.uses[self.site];
        push_work(
            pending,
            view,
            Steps::LoadAt {
                memory,
                file: &decl.file,
                line: decl.line,
            },
        )
    }

    fn certify(
        &self,
        inputs: &Inputs<'_, '_>,
        scratch: &mut Scratch,
        transfer_memory: &Transient<'_>,
    ) -> Result<bool, LoadError> {
        let memory = self.memory.child();
        // No transfer can create an id from bottom. Own definitions must be
        // empty too: absent resident outputs alone are not a closure proof.
        if inputs.bottom {
            return Ok(true);
        }
        let count = inputs.rows.len();
        let mut marks = Vec::new();
        self.reserve(&mut marks, count, &memory)?;
        self.work(Work::ProofSteps, count)?;
        marks.resize(count, Mark::default());
        let mut pending = WorkList::<_, 8>::new();
        let mut remaining = 0;
        for view in (0..count).rev() {
            self.work(Work::ProofReads, 1)?;
            if inputs.row(view).held.is_empty() {
                marks[view].certified = true;
            } else {
                remaining += 1;
                marks[view].queued = true;
                self.enqueue(&mut pending, view, &memory)?;
            }
        }
        while let Some(view) = pending.pop() {
            self.work(Work::ProofSteps, 1)?;
            marks[view].queued = false;
            let complete = self.transfer(view, inputs, &marks, scratch, transfer_memory)?;
            if !self.same(&scratch.seen, inputs.row(view).held)? {
                continue;
            }
            marks[view].certified = true;
            marks[view].closed = complete;
            remaining -= 1;
            for &module in self.index.globs.views[view].places.keys() {
                self.work(Work::ProofReads, 1)?;
                for &other in self.index.exports_of(module) {
                    self.work(Work::ProofReads, 1)?;
                    if !marks[other].certified && !marks[other].queued {
                        marks[other].queued = true;
                        self.enqueue(&mut pending, other, &memory)?;
                    }
                }
            }
        }
        if remaining != 0 {
            return Ok(false);
        }
        self.closure(inputs, &marks, scratch, transfer_memory)
    }

    fn closure(
        &self,
        inputs: &Inputs<'_, '_>,
        marks: &[Mark],
        scratch: &mut Scratch,
        memory: &Transient<'_>,
    ) -> Result<bool, LoadError> {
        for (view, mark) in marks.iter().enumerate() {
            self.work(Work::ProofReads, 1)?;
            if mark.closed {
                continue;
            }
            #[cfg(test)]
            {
                scratch.closure_calls += 1;
            }
            self.transfer(view, inputs, marks, scratch, memory)?;
            if !self.same(&scratch.seen, inputs.row(view).held)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn same(&self, left: &[Held], right: &[Held]) -> Result<bool, LoadError> {
        self.work(Work::ProofSteps, 1 + left.len().min(right.len()))?;
        Ok(left == right)
    }

    fn transfer(
        &self,
        view: usize,
        inputs: &Inputs<'_, '_>,
        marks: &[Mark],
        scratch: &mut Scratch,
        memory: &Transient<'_>,
    ) -> Result<bool, LoadError> {
        let own = &self.index.globs.views[view];
        let module = own.target;
        self.work(Work::ProofSteps, 3)?;
        scratch.seen.clear();
        scratch.hidden.clear();
        scratch.copied.clear();
        if let Some(defs) = inputs.row(view).defs {
            self.reserve(&mut scratch.seen, defs.len(), memory)?;
            self.work(Work::ProofReads, defs.len())?;
            self.work(Work::ProofCopies, defs.len())?;
            scratch
                .seen
                .extend(self.index.defs_held_iter(module, &own.class, defs));
            return Ok(true);
        }
        let reads = inputs.reads(module);
        self.work(Work::ProofReads, reads.len())?;
        let mut complete = true;
        let cells = reads
            .clone()
            .filter(|read| {
                complete &= marks[read.source.view].certified;
                marks[read.source.view].certified
            })
            .fold(0usize, |total, read| {
                total.saturating_add(inputs.row(read.source.view).held.len())
            });
        self.reserve(&mut scratch.seen, cells, memory)?;
        self.reserve(&mut scratch.hidden, cells, memory)?;
        for read in reads {
            self.work(Work::ProofReads, 1)?;
            if !marks[read.source.view].certified {
                continue;
            }
            let source = inputs.row(read.source.view).held;
            self.work(Work::ProofReads, source.len())?;
            scratch.copied.clear();
            self.reserve(&mut scratch.copied, source.len(), memory)?;
            self.work(Work::ProofCopies, source.len())?;
            scratch.copied.extend_from_slice(source);
            let mut held = Cow::Owned(std::mem::take(&mut scratch.copied));
            let known = read.known;
            self.work(
                Work::ProofSteps,
                known.len().saturating_add(
                    source
                        .len()
                        .saturating_mul(3 * search_units(known.len()) + 3),
                ),
            )?;
            self.index.filter_known_delivered(
                module,
                Some(&own.class),
                known,
                &mut held,
                Work::ProofCopies,
            );
            self.work(Work::ProofCopies, held.len())?;
            self.work(Work::ProofSteps, held.len())?;
            self.index.glob_held(
                module,
                &own.class,
                read.glob,
                &held,
                (&mut scratch.seen, &mut scratch.hidden),
            );
            scratch.copied = held.into_owned();
        }
        self.index
            .expose_in_place(&mut scratch.seen, &mut scratch.hidden, &|units| {
                self.work(Work::ProofSteps, units)
            })?;
        Ok(complete)
    }
}
