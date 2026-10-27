// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Admitted derived glob facts and the traversal that queries them.

use purrdf_lex::allocation::{Admission, Memory, OrderedTable, Shared, StorageError};
use purrdf_lex::walk::{WorkList, try_equal_range_by};

use super::{GlobFacts, GlobPath, Index, LoadError, Namespace, Source, Steps};
use crate::meter::{Meter, NativeAdmission, Transient};
use crate::path::RelPath;

/// Native resize runs before allocation/copy. New-buffer bytes bound the
/// existing payload copied plus initialization, even with overlapping buffers.
struct FactAdmission<'m> {
    native: NativeAdmission<'m>,
    steps: Steps<'m>,
    live: usize,
    failure: Option<LoadError>,
}

impl Admission for FactAdmission<'_> {
    fn resize(&mut self, live: usize) -> Result<(), StorageError> {
        if live > self.live {
            let words = (live - self.live).div_ceil(size_of::<usize>());
            if let Err(error) = self.steps.storage_work(words.saturating_mul(4)) {
                self.failure = Some(error);
                return Err(StorageError::AdmissionFailed);
            }
        }
        self.native.resize(live)?;
        self.live = live;
        Ok(())
    }
}

/// An AVL search and insertion have logarithmically bounded node visits.
/// This covers the search, ancestor updates and rotations before they run.
pub(super) fn table_work(len: usize, steps: Steps<'_>) -> Result<(), LoadError> {
    steps.storage_work(table_units(len))
}

pub(super) fn table_units(len: usize) -> usize {
    8 * (len.max(1).ilog2() as usize + 1)
}

/// One physical native account, resumed for each mutation of its buffers.
/// Scratch uses its caller's live grant; immutable index payload uses Items.
struct Account<'m> {
    steps: Steps<'m>,
    items: Option<(&'m Meter, &'m RelPath, Option<usize>)>,
    live: usize,
}

impl<'m> Account<'m> {
    fn scratch(steps: Steps<'m>) -> Self {
        Self {
            steps,
            items: None,
            live: 0,
        }
    }

    fn native<T>(
        &mut self,
        action: impl FnOnce(&mut Memory<'_, FactAdmission<'_>>) -> Result<T, StorageError>,
    ) -> Result<T, LoadError> {
        let native = self.items.map_or_else(
            || NativeAdmission::resume(self.steps, self.live),
            |(meter, file, line)| NativeAdmission::items(meter, file, line, self.live),
        );
        let mut admission = FactAdmission {
            native,
            steps: self.steps,
            live: self.live,
            failure: None,
        };
        let result = {
            let mut memory = Memory::resume(&mut admission, self.live);
            let result = action(&mut memory);
            self.live = memory.admitted_bytes();
            result
        };
        result.map_err(|error| {
            admission
                .failure
                .take()
                .unwrap_or_else(|| admission.native.graph_error(error.into()))
        })
    }

    fn insert<K: Ord, V>(
        &mut self,
        table: &mut OrderedTable<K, V>,
        key: K,
        value: V,
    ) -> Result<Option<V>, LoadError> {
        table_work(table.len(), self.steps)?;
        self.native(|memory| table.insert(key, value, memory))
    }

    fn push<T>(&mut self, into: &mut Vec<T>, value: T) -> Result<(), LoadError> {
        self.steps.storage_work(1)?;
        self.native(|memory| memory.push(into, value))
    }

    fn queue<T>(&mut self, into: &mut WorkList<T, 8>, value: T) -> Result<(), LoadError> {
        self.steps.storage_work(1)?;
        self.native(|memory| into.try_push_admitted(value, memory))
    }
}

impl Index {
    /// Each actual glob edge and duplicate visit is charged; sets and queue
    /// storage stay under the same enclosing load or query grant.
    pub(super) fn globs_read(
        &self,
        reach: &[Vec<usize>],
        module: usize,
        ns: Namespace,
        name: &str,
        steps: Steps<'_>,
    ) -> Result<OrderedTable<usize, ()>, LoadError> {
        let mut account = Account::scratch(steps);
        let mut seen = OrderedTable::default();
        account.insert(&mut seen, module, ())?;
        let mut work = WorkList::new();
        account.queue(&mut work, module)?;
        let mut out = OrderedTable::default();
        while let Some(at) = work.pop() {
            // Scope text lookup, definition lookup and named-import scan.
            steps.storage_work((name.len() + 1).saturating_mul(
                4 * (self.scopes[at].len().max(1).ilog2() as usize + 1)
                    + 4 * (self.named_imports.len().max(1).ilog2() as usize + 1),
            ))?;
            steps.storage_work(self.imports_of(at, name).len())?;
            if self.binds_explicitly(at, ns, name) {
                continue;
            }
            table_work(self.glob_imports.len(), steps)?;
            for glob in self.globs_of(at) {
                account.insert(&mut out, *glob, ())?;
                table_work(self.glob_imports.len(), steps)?;
                let key = (self.uses[*glob].module, *glob);
                let range = try_equal_range_by(&self.glob_imports, |position| {
                    Ok::<_, std::convert::Infallible>(
                        (self.uses[*position].module, *position).cmp(&key),
                    )
                })
                .expect("integer comparison cannot fail");
                assert!(
                    !range.is_empty(),
                    "a module glob belongs to the frozen registry"
                );
                for target in &reach[range.start] {
                    if account.insert(&mut seen, *target, ())?.is_none() {
                        account.queue(&mut work, *target)?;
                    }
                }
            }
        }
        Ok(out)
    }

    /// Inspect each distinct actual read place once, preserving lexical order.
    pub(super) fn unresolved_glob_read(
        &self,
        reads: &[Source],
        steps: Steps<'_>,
    ) -> Result<Option<usize>, LoadError> {
        let Some(facts) = self.glob_facts().filter(|facts| !facts.broken.is_empty()) else {
            return Ok(None);
        };
        let Some(source) = reads.first() else {
            return Ok(None);
        };
        let memory = steps.query_meter().map(Transient::new);
        let (file, line) = self.module_site(source.module);
        let steps = steps.with_query_storage(memory.as_ref(), (&file, line));
        let mut account = Account::scratch(steps);
        let mut places = OrderedTable::default();
        for source in reads {
            steps.storage_work(source.name.len().saturating_mul(table_units(places.len())))?;
            account.insert(
                &mut places,
                (source.module, source.ns, source.name.as_str()),
                (),
            )?;
        }
        let mut first = None;
        for ((module, ns, name), ()) in places.ordered() {
            let memory = steps.temporary();
            let read_steps = memory.as_ref().map_or(steps, |memory| steps.at(memory));
            let globs = self.globs_read(&facts.reach, *module, *ns, name, read_steps)?;
            for (glob, ()) in globs.ordered() {
                table_work(facts.broken.len(), steps)?;
                if facts.broken.get(glob).is_some() {
                    first = Some(first.map_or(*glob, |old: usize| old.min(*glob)));
                }
            }
        }
        Ok(first)
    }

    /// Build immutable index data under `ItemBytes`, with temporary paths,
    /// reverse edges and traversal queues under the load's `GlobBytes` grant.
    pub(crate) fn settle_glob_facts(&mut self, meter: &Meter) -> Result<(), LoadError> {
        let Some(&first) = self.glob_imports.first() else {
            self.glob_facts = super::GlobFactsCache::default();
            return Ok(());
        };
        let decl = &self.uses[first];
        let memory = Transient::new(meter);
        let steps = Steps::LoadAt {
            memory: &memory,
            file: &decl.file,
            line: decl.line,
        };
        let mut scratch = Account::scratch(steps);
        let mut retained = Account {
            steps,
            items: Some((meter, &decl.file, Some(decl.line))),
            live: 0,
        };
        let mut facts = GlobFacts::default();
        // One row per actual glob, in the existing frozen registry's order.
        // No module/use-sized auxiliary index or tree construction is needed.
        retained.native(|memory| memory.reserve(&mut facts.reach, self.glob_imports.len()))?;
        let mut misses = OrderedTable::<usize, Vec<Source>>::default();
        let mut unknown = WorkList::new();
        for &glob in &self.glob_imports {
            let decl = &self.uses[glob];
            let path_memory = memory.child();
            let path_steps = Steps::LoadAt {
                memory: &path_memory,
                file: &decl.file,
                line: decl.line,
            };
            scratch.steps = Steps::LoadAt {
                memory: &memory,
                file: &decl.file,
                line: decl.line,
            };
            retained.steps = scratch.steps;
            retained.items = Some((meter, &decl.file, Some(decl.line)));
            path_steps.storage_work(1)?;
            let path = self.glob_path(glob, path_steps)?;
            self.settle_lookups(meter, glob)?;
            let mut row = Vec::new();
            match path {
                GlobPath::Resolves(targets) => {
                    for target in targets {
                        retained.push(&mut row, target)?;
                    }
                }
                GlobPath::Misses(sources) => {
                    // Move the actual surviving buffers, without cloning their
                    // names or readmitting their original physical payloads.
                    let bytes = sources.capacity() * size_of::<Source>()
                        + sources
                            .iter()
                            .map(|source| source.name.capacity())
                            .sum::<usize>();
                    path_memory.transfer_to(&memory, bytes, &decl.file, Some(decl.line))?;
                    scratch.insert(&mut misses, glob, sources)?;
                }
                GlobPath::Unknown => {
                    scratch.queue(&mut unknown, glob)?;
                }
                GlobPath::Nothing => {}
            }
            retained.push(&mut facts.reach, row)?;
        }
        self.complete_glob_facts(
            &mut facts,
            &misses,
            &mut unknown,
            &mut scratch,
            &mut retained,
        )?;
        let decl = &self.uses[first];
        let bytes = Shared::<GlobFacts>::allocation_layout().size();
        meter.work(
            bytes.div_ceil(size_of::<usize>()) * 4,
            &decl.file,
            Some(decl.line),
        )?;
        meter.items(bytes, &decl.file, Some(decl.line))?;
        let facts = Shared::try_new(facts).map_err(|_| {
            LoadError::new(
                &decl.file,
                Some(decl.line),
                crate::LoadErrorKind::Storage(StorageError::AllocationFailed),
            )
        })?;
        self.glob_facts.replace(facts);
        Ok(())
    }

    fn complete_glob_facts<'a>(
        &'a self,
        facts: &mut GlobFacts,
        misses: &OrderedTable<usize, Vec<Source>>,
        unknown: &mut WorkList<usize, 8>,
        scratch: &mut Account<'a>,
        retained: &mut Account<'a>,
    ) -> Result<(), LoadError> {
        let mut readers = OrderedTable::<usize, Vec<usize>>::default();
        for (glob, sources) in misses.ordered() {
            let decl = &self.uses[*glob];
            let Steps::LoadAt { memory, .. } = scratch.steps else {
                unreachable!("construction owns a load lease")
            };
            scratch.steps = Steps::LoadAt {
                memory,
                file: &decl.file,
                line: decl.line,
            };
            let steps = scratch.steps;
            for source in sources {
                steps
                    .storage_work((source.name.len() + 1).saturating_mul(
                        4 * (self.named_imports.len().max(1).ilog2() as usize + 1),
                    ))?;
                if !self.imports_of(source.module, &source.name).is_empty() {
                    scratch.queue(unknown, *glob)?;
                }
                let memory = steps.temporary();
                let read_steps = memory.as_ref().map_or(steps, |memory| steps.at(memory));
                let read = self.globs_read(
                    &facts.reach,
                    source.module,
                    source.ns,
                    &source.name,
                    read_steps,
                )?;
                for (other, ()) in read.ordered() {
                    if other == glob {
                        continue;
                    }
                    table_work(readers.len(), steps)?;
                    if readers.get(other).is_none() {
                        scratch.insert(&mut readers, *other, Vec::new())?;
                    }
                    table_work(readers.len(), steps)?;
                    let row = readers.get_mut(other).expect("the reader row was admitted");
                    scratch.push(row, *glob)?;
                }
            }
        }
        let mut opaque = OrderedTable::default();
        while let Some(glob) = unknown.pop() {
            let decl = &self.uses[glob];
            let Steps::LoadAt { memory, .. } = scratch.steps else {
                unreachable!("construction owns a load lease")
            };
            scratch.steps = Steps::LoadAt {
                memory,
                file: &decl.file,
                line: decl.line,
            };
            if scratch.insert(&mut opaque, glob, ())?.is_none() {
                table_work(readers.len(), scratch.steps)?;
                for &reader in readers.get(&glob).into_iter().flatten() {
                    scratch.queue(unknown, reader)?;
                }
            }
        }
        for (glob, _) in misses.ordered() {
            let decl = &self.uses[*glob];
            let Steps::LoadAt { memory, .. } = scratch.steps else {
                unreachable!("construction owns a load lease")
            };
            retained.steps = Steps::LoadAt {
                memory,
                file: &decl.file,
                line: decl.line,
            };
            retained.items = Some((
                retained.items.expect("retained index admission").0,
                &decl.file,
                Some(decl.line),
            ));
            table_work(opaque.len(), scratch.steps)?;
            if opaque.get(glob).is_none() {
                retained.insert(&mut facts.broken, *glob, ())?;
            }
        }
        Ok(())
    }
}
