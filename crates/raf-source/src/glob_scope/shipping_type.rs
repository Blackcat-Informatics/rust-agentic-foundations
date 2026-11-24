// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Read proven standard Type names through the shipping part of settled scopes.

use std::borrow::Cow;
use std::collections::{BTreeMap, VecDeque};

use super::{Level, reserve_next};
use crate::LoadError;
use crate::cost::{btree_entry, btree_map, vec_first, vec_slot};
use crate::facts::TestRole;
use crate::index::{Index, NamedNamespaces, UseKind};
use crate::item::{ItemId, Namespace};
use crate::meter::Steps;
use crate::settle::{slot, storage_failure};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TypeCandidate {
    Indexed(ItemId),
    Standard,
}

type Held = (TypeCandidate, Level);
type Input = (usize, Level);

struct GlobInput {
    source: Input,
    replacements: Vec<Held>,
}

/// A glob reuses its resident class. A named import reads its settled
/// terminal module from the declaring module's class.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Reader {
    Local(usize),
    Glob(usize),
    Module { target: usize, importer: usize },
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Key<'a> {
    reader: Reader,
    name: &'a str,
}

#[derive(Default, PartialEq, Eq)]
struct Answer {
    cut: bool,
    held: Vec<Held>,
}

struct Row<'a> {
    key: Key<'a>,
    target: usize,
    class: Cow<'a, [bool]>,
    explicit: Answer,
    named: Vec<Input>,
    globs: Vec<GlobInput>,
    readers: Vec<usize>,
    answer: Answer,
    queued: bool,
}

impl Index {
    /// Recompute the Type candidates visible in the shipping reader, using
    /// the settled routes and visibility classes. A private candidate still
    /// needs a shipping visible seed to be exposed as an ambiguity companion.
    pub(crate) fn shipping_type_competitors(
        &self,
        module: usize,
        name: &str,
        steps: Steps<'_>,
    ) -> Result<Option<Vec<ItemId>>, LoadError> {
        let answer = self.project_shipping_types(module, name, true, steps)?;
        if candidate_level(&answer.held, TypeCandidate::Standard).is_none() {
            return Ok(None);
        }
        let mut candidates = Vec::new();
        steps.storage_work(answer.held.len())?;
        for (item, _) in answer.held {
            if let TypeCandidate::Indexed(id) = item {
                reserve_next(&mut candidates, steps)?;
                candidates.push(id);
            }
        }
        Ok(Some(candidates))
    }

    /// Whether explicit shipping declarations keep a Type from the globs.
    /// The root needs only its explicit answer for this query.
    pub(crate) fn shipping_type_cut(
        &self,
        module: usize,
        name: &str,
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        Ok(self.project_shipping_types(module, name, false, steps)?.cut)
    }

    fn project_shipping_types(
        &self,
        module: usize,
        name: &str,
        root_globs: bool,
        steps: Steps<'_>,
    ) -> Result<Answer, LoadError> {
        let mut rows = self.shipping_type_rows(module, name, root_globs, steps)?;
        loop {
            self.settle_shipping_rows(&mut rows, steps)?;
            if !self.drop_returning_shipping_types(&mut rows, steps)? {
                break;
            }
            steps.storage_work(rows.len())?;
            for row in &mut rows {
                row.answer = Answer::default();
            }
        }
        Self::query_refusal(steps).map_err(|refused| self.refused_load(refused))?;
        Ok(std::mem::take(&mut rows[0].answer))
    }

    fn settle_shipping_rows(
        &self,
        rows: &mut [Row<'_>],
        steps: Steps<'_>,
    ) -> Result<(), LoadError> {
        steps.storage_work(rows.len())?;
        steps.reserve(vec_first::<usize>() + rows.len() * vec_slot::<usize>())?;
        let mut pending = VecDeque::with_capacity(rows.len());
        for (at, row) in rows.iter_mut().enumerate().rev() {
            row.queued = true;
            pending.push_back(at);
        }
        while let Some(at) = pending.pop_front() {
            rows[at].queued = false;
            let answer = self.shipping_type_answer(rows, at, steps)?;
            if answer == rows[at].answer {
                continue;
            }
            rows[at].answer = answer;
            steps.storage_work(rows[at].readers.len())?;
            for offset in 0..rows[at].readers.len() {
                let reader = rows[at].readers[offset];
                if !rows[reader].queued {
                    rows[reader].queued = true;
                    pending.push_back(reader);
                }
            }
        }
        Ok(())
    }

    /// Expand the finite graph of requested scope and name pairs. A fixed
    /// explicit cut never needs globs of its own.
    fn shipping_type_rows<'a>(
        &'a self,
        module: usize,
        name: &'a str,
        root_globs: bool,
        steps: Steps<'_>,
    ) -> Result<Vec<Row<'a>>, LoadError> {
        let root = Key {
            reader: Reader::Local(module),
            name,
        };
        steps.reserve(btree_map::<Key<'_>, usize>())?;
        let mut nodes = BTreeMap::new();
        let mut rows = Vec::new();
        self.shipping_row(root, &mut rows, &mut nodes, steps)?;
        let mut at = 0;
        while at < rows.len() {
            steps.storage_work(1)?;
            let key = rows[at].key;
            let module = rows[at].target;
            self.add_shipping_definitions(module, at, &mut rows, &mut nodes, steps)?;
            if !rows[at].explicit.cut && (at != 0 || root_globs) {
                let reads = self.reads_of(module);
                steps.storage_work(reads.len())?;
                for read in reads {
                    let decl = &self.uses[read.glob];
                    if decl.test {
                        continue;
                    }
                    let level = self
                        .read_level_of(&decl.vis, module, steps)
                        .map_err(storage_failure)?;
                    let target = Key {
                        reader: Reader::Glob(read.view),
                        name: key.name,
                    };
                    let to = self.shipping_row(target, &mut rows, &mut nodes, steps)?;
                    reserve_next(&mut rows[at].globs, steps)?;
                    rows[at].globs.push(GlobInput {
                        source: (to, level),
                        replacements: Vec::new(),
                    });
                    reserve_next(&mut rows[to].readers, steps)?;
                    rows[to].readers.push(at);
                }
            }
            at += 1;
        }
        Ok(rows)
    }

    fn shipping_row<'a>(
        &'a self,
        key: Key<'a>,
        rows: &mut Vec<Row<'a>>,
        nodes: &mut BTreeMap<Key<'a>, usize>,
        steps: Steps<'_>,
    ) -> Result<usize, LoadError> {
        if let Some(&known) = nodes.get(&key) {
            return Ok(known);
        }
        let (target, class) = match key.reader {
            Reader::Glob(view) => {
                let view = &self.globs.views[view];
                (view.target, Cow::Borrowed(view.class.as_ref()))
            }
            Reader::Local(module) => {
                let depth = 1 + self.depth_of(module);
                steps.reserve(depth * size_of::<bool>())?;
                steps.storage_work(depth)?;
                (module, Cow::Owned(vec![true; depth]))
            }
            Reader::Module { target, importer } => (
                target,
                Cow::Owned(self.shipping_importer_class(target, importer, steps)?),
            ),
        };
        let next = rows.len();
        steps.reserve(btree_entry::<Key<'_>, usize>())?;
        nodes.insert(key, next);
        reserve_next(rows, steps)?;
        rows.push(Row {
            key,
            target,
            class,
            explicit: Answer::default(),
            named: Vec::new(),
            globs: Vec::new(),
            readers: Vec::new(),
            answer: Answer::default(),
            queued: false,
        });
        Ok(next)
    }

    fn shipping_importer_class(
        &self,
        mut target: usize,
        mut importer: usize,
        steps: Steps<'_>,
    ) -> Result<Vec<bool>, LoadError> {
        let mut at = self.depth_of(target);
        let mut from = self.depth_of(importer);
        steps.storage_work(1 + at + from)?;
        steps.reserve((at + 1) * size_of::<bool>())?;
        let mut class = vec![false; at + 1];
        if self.modules[target].root != self.modules[importer].root {
            return Ok(class);
        }
        while target != importer {
            if at >= from
                && let Some(parent) = self.modules[target].parent
            {
                target = parent;
                at -= 1;
            } else if let Some(parent) = self.modules[importer].parent {
                importer = parent;
                from -= 1;
            }
        }
        class[..=at].fill(true);
        Ok(class)
    }

    /// Seed declarations that ship and follow the settled terminal scope of
    /// each indexed named import. Its shipping source can bind a different
    /// Type, or bind only a Value after a test-only Type disappears.
    fn add_shipping_definitions<'a>(
        &'a self,
        module: usize,
        at: usize,
        rows: &mut Vec<Row<'a>>,
        nodes: &mut BTreeMap<Key<'a>, usize>,
        steps: Steps<'_>,
    ) -> Result<(), LoadError> {
        let space = slot(Namespace::Type);
        let name = rows[at].key.name;
        if let Some(entry) = self.scopes[module].get(name) {
            let defs = &entry.defs[space];
            steps.storage_work(defs.len())?;
            for (id, origin) in defs {
                if origin.link.is_none() && self.items[id.0].test == TestRole::None {
                    rows[at].explicit.cut = true;
                    let level = self
                        .read_level_of(&origin.vis, module, steps)
                        .map_err(storage_failure)?;
                    reserve_next(&mut rows[at].explicit.held, steps)?;
                    rows[at]
                        .explicit
                        .held
                        .push((TypeCandidate::Indexed(*id), level));
                }
            }
        }
        let imports = self.imports_of(module, name);
        steps.storage_work(imports.len())?;
        for &position in imports {
            let decl = &self.uses[position];
            if decl.test || self.use_free[position][space] {
                continue;
            }
            if matches!(
                decl.kind,
                UseKind::Named {
                    namespaces: NamedNamespaces::StandardRc,
                    ..
                }
            ) {
                rows[at].explicit.cut = true;
                let level = self
                    .read_level_of(&decl.vis, module, steps)
                    .map_err(storage_failure)?;
                reserve_next(&mut rows[at].explicit.held, steps)?;
                rows[at]
                    .explicit
                    .held
                    .push((TypeCandidate::Standard, level));
            } else if self.use_bound[position][space] {
                self.add_shipping_named_import(module, position, at, rows, nodes, steps)?;
            } else {
                rows[at].explicit.cut = true;
            }
        }
        Ok(())
    }

    fn add_shipping_named_import<'a>(
        &'a self,
        module: usize,
        position: usize,
        at: usize,
        rows: &mut Vec<Row<'a>>,
        nodes: &mut BTreeMap<Key<'a>, usize>,
        steps: Steps<'_>,
    ) -> Result<(), LoadError> {
        let visibility = self
            .read_level_of(&self.uses[position].vis, module, steps)
            .map_err(storage_failure)?;
        for supply in self.import_supply(position, steps)? {
            steps.storage_work(1)?;
            if supply.ns != Namespace::Type {
                continue;
            }
            if let Some(source) = supply.source {
                if let Some(name) = self.bound_name(source.module, &source.name) {
                    let key = Key {
                        reader: Reader::Module {
                            target: source.module,
                            importer: module,
                        },
                        name,
                    };
                    let to = self.shipping_row(key, rows, nodes, steps)?;
                    reserve_next(&mut rows[at].named, steps)?;
                    rows[at].named.push((to, visibility));
                    reserve_next(&mut rows[to].readers, steps)?;
                    rows[to].readers.push(at);
                }
            } else if self.items[supply.id.0].test == TestRole::None {
                rows[at].explicit.cut = true;
                reserve_next(&mut rows[at].explicit.held, steps)?;
                rows[at]
                    .explicit
                    .held
                    .push((TypeCandidate::Indexed(supply.id), supply.level));
            }
        }
        Ok(())
    }

    fn shipping_type_answer(
        &self,
        rows: &[Row<'_>],
        at: usize,
        steps: Steps<'_>,
    ) -> Result<Answer, LoadError> {
        let row = &rows[at];
        let target = row.target;
        let class = &row.class;
        let explicit = self.shipping_explicit_answer(target, class, rows, row, steps)?;
        if explicit.cut {
            return Ok(explicit);
        }
        steps.storage_work(row.globs.len())?;
        let count = row
            .globs
            .iter()
            .map(|read| rows[read.source.0].answer.held.len())
            .sum();
        if count == 0 {
            return Ok(explicit);
        }
        steps.reserve(2 * (vec_first::<Held>() + count * vec_slot::<Held>()))?;
        steps.storage_work(count)?;
        let mut seen = Vec::with_capacity(count);
        let mut hidden = Vec::with_capacity(count);
        for read in &row.globs {
            let (to, level) = read.source;
            let supplied = rows[to].answer.held.iter().filter_map(|&(item, offered)| {
                if read.replacements.is_empty() {
                    Some((item, offered))
                } else {
                    let support = candidate_level(&read.replacements, item)?;
                    Some((item, self.meet(offered.plain(), support)))
                }
            });
            self.glob_held_iter(target, class, level, supplied, &mut seen, &mut hidden);
        }
        self.expose_in_place(&mut seen, &mut hidden, &|units| steps.storage_work(units))?;
        Ok(Answer {
            cut: false,
            held: seen,
        })
    }

    fn shipping_explicit_answer(
        &self,
        target: usize,
        class: &[bool],
        rows: &[Row<'_>],
        row: &Row<'_>,
        steps: Steps<'_>,
    ) -> Result<Answer, LoadError> {
        let mut cut = row.explicit.cut;
        let mut held = Vec::new();
        steps.storage_work(row.explicit.held.len() + row.named.len())?;
        for &(item, level) in &row.explicit.held {
            if self.allowed(target, class, level) {
                reserve_next(&mut held, steps)?;
                held.push((item, level));
            }
        }
        for &(to, visibility) in &row.named {
            let supplied = &rows[to].answer.held;
            steps.storage_work(supplied.len())?;
            cut |= !supplied.is_empty();
            for &(item, source) in supplied {
                let level = self.meet(visibility, source);
                if matches!(item, TypeCandidate::Indexed(_)) && self.allowed(target, class, level) {
                    reserve_next(&mut held, steps)?;
                    held.push((item, level));
                }
            }
        }
        self.widest_charged(&mut held, &|units| steps.storage_work(units))?;
        Ok(Answer { cut, held })
    }

    /// Narrow returning reads only where another visible read independently
    /// supplies the same Type. An ordinary duplicate keeps any ambiguity
    /// that also has independent support at the carrier.
    fn drop_returning_shipping_types(
        &self,
        rows: &mut [Row<'_>],
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        let mut changes = Vec::new();
        for (at, row) in rows.iter().enumerate() {
            steps.storage_work(1)?;
            if row.answer.cut || row.answer.held.len() < 2 || row.globs.len() < 2 {
                continue;
            }
            for &(item, _) in &row.answer.held {
                let mut independent = None;
                let mut returning = Vec::new();
                steps.storage_work(row.globs.len())?;
                for (offset, read) in row.globs.iter().enumerate() {
                    let Some(level) = self.shipping_glob_level(read, rows, item) else {
                        continue;
                    };
                    if !self.allowed(row.target, &row.class, level) {
                        continue;
                    }
                    let level = level.plain();
                    let from = read.source.0;
                    if self.shipping_supply_apart(
                        rows,
                        from,
                        Some((row.target, row.key.name)),
                        (item, level),
                        steps,
                    )? {
                        independent =
                            Some(independent.map_or(level, |other| self.wider(level, other)));
                    } else if rows[from].answer.held.len() > 1
                        && self.shipping_supply_apart(rows, from, None, (item, level), steps)?
                    {
                        reserve_next(&mut returning, steps)?;
                        returning.push((offset, level));
                    }
                }
                if let Some(independent) = independent {
                    for (offset, offered) in returning {
                        let level = self.meet(independent, offered);
                        reserve_next(&mut changes, steps)?;
                        changes.push((at, offset, item, level));
                    }
                }
            }
        }
        let mut widened = false;
        for (at, offset, item, level) in changes {
            steps.storage_work(1)?;
            let known = &mut rows[at].globs[offset].replacements;
            match known.binary_search_by_key(&item, |&(candidate, _)| candidate) {
                Ok(place)
                    if self.wider(known[place].1, level) == level && known[place].1 != level =>
                {
                    known[place].1 = level;
                    widened = true;
                }
                Err(place) => {
                    steps.storage_work(known.len() - place)?;
                    reserve_next(known, steps)?;
                    known.insert(place, (item, level));
                    widened = true;
                }
                Ok(_) => {}
            }
        }
        Ok(widened)
    }

    fn shipping_glob_level(
        &self,
        read: &GlobInput,
        rows: &[Row<'_>],
        item: TypeCandidate,
    ) -> Option<Level> {
        let (to, own) = read.source;
        let offered = candidate_level(&rows[to].answer.held, item)?;
        if read.replacements.is_empty() {
            Some(self.meet(own, offered))
        } else {
            let support = candidate_level(&read.replacements, item)?;
            Some(self.meet(own, self.meet(offered.plain(), support)))
        }
    }

    /// Trace the candidate to a firm declaration at the visibility needed
    /// for the selection. A widened ambiguity companion is insufficient.
    fn shipping_supply_apart(
        &self,
        rows: &[Row<'_>],
        from: usize,
        avoid: Option<(usize, &str)>,
        (item, required): Held,
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        steps.reserve(rows.len() * size_of::<bool>())?;
        let mut visited = vec![false; rows.len()];
        let mut work = Vec::new();
        reserve_next(&mut work, steps)?;
        work.push(from);
        let covers =
            |offered| self.level_covers(offered, required, &mut |units| steps.storage_work(units));
        while let Some(at) = work.pop() {
            steps.storage_work(1)?;
            let row = &rows[at];
            if visited[at] || avoid == Some((row.target, row.key.name)) {
                continue;
            }
            visited[at] = true;
            let Some(level) = candidate_level(&row.answer.held, item) else {
                continue;
            };
            if !covers(level)? {
                continue;
            }
            if row.answer.cut {
                steps.storage_work(row.explicit.held.len() + row.named.len())?;
                for &(known, level) in &row.explicit.held {
                    if known == item
                        && self.allowed(row.target, &row.class, level)
                        && covers(level)?
                    {
                        return Ok(true);
                    }
                }
                if matches!(item, TypeCandidate::Indexed(_)) {
                    for &(to, level) in &row.named {
                        if covers(level)? {
                            reserve_next(&mut work, steps)?;
                            work.push(to);
                        }
                    }
                }
            } else {
                steps.storage_work(row.globs.len())?;
                for read in &row.globs {
                    if let Some(level) = self.shipping_glob_level(read, rows, item)
                        && covers(level)?
                    {
                        reserve_next(&mut work, steps)?;
                        work.push(read.source.0);
                    }
                }
            }
        }
        Ok(false)
    }
}

fn candidate_level(held: &[Held], item: TypeCandidate) -> Option<Level> {
    let at = held.binary_search_by_key(&item, |&(known, _)| known).ok()?;
    Some(held[at].1)
}
