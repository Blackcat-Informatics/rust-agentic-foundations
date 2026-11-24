// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Scope evidence for an implicit standard-library root.

use purrdf_lex::allocation::{Admission, Memory, StorageError};
use purrdf_lex::walk::WorkList;
use std::collections::BTreeMap;
use std::mem::size_of;

use crate::facts::{AttrFact, has_expansion_attributes};
use crate::glob_scope::reserve_next;
use crate::index::{Index, NamedNamespaces, UseKind};
use crate::item::Namespace;
use crate::meter::{ItemAdmission, Meter, Steps};
use crate::settle::slot;
use crate::work::{self, Work};
use crate::workspace::SourceFile;
use crate::{LoadError, RelPath};

enum BuildError {
    Storage(StorageError),
    Read(LoadError),
}

impl Index {
    /// Refine standard-library imports before consumers read the
    /// settled scopes. Recheck each admitted import after its namespace can
    /// change other imports. A disproved refinement is removed permanently.
    pub(crate) fn settle_standard_types(
        &mut self,
        files: &BTreeMap<RelPath, SourceFile>,
        meter: &Meter,
    ) -> Result<(), LoadError> {
        let mut from = NamedNamespaces::All;
        loop {
            let changes = self.standard_type_changes(files, meter, from)?;
            let Some(&first) = changes.first() else {
                return Ok(());
            };
            let to = match from {
                NamedNamespaces::All => NamedNamespaces::StandardRc,
                _ => NamedNamespaces::All,
            };
            for position in changes {
                if let UseKind::Named { namespaces, .. } = &mut self.uses[position].kind {
                    *namespaces = to;
                }
            }
            let site = &self.uses[first];
            meter.items(self.settle_bytes(), &site.file, Some(site.line))?;
            self.release_globs(meter);
            self.forget_glob_replacements();
            self.build_scopes();
            self.use_ambiguous.fill(false);
            self.crates_open = false;
            self.settle_agreed(meter)?;
            from = NamedNamespaces::StandardRc;
        }
    }

    fn standard_type_changes(
        &self,
        files: &BTreeMap<RelPath, SourceFile>,
        meter: &Meter,
        from: NamedNamespaces,
    ) -> Result<Vec<usize>, LoadError> {
        let selected = self.uses.iter().enumerate().filter(|(_, decl)| {
            !decl.test
                && matches!(&decl.kind, UseKind::Named { namespaces, segs, .. }
                    if *namespaces == from && segs.as_slice() == ["std", "rc", "Rc"])
        });
        let mut selected = selected.peekable();
        let Some((_, site)) = selected.peek() else {
            return Ok(Vec::new());
        };
        let scope = StdScope::new(self, files, meter, &site.file)?;
        let was_proven = from == NamedNamespaces::StandardRc;
        let mut changes = Vec::new();
        for (position, decl) in selected {
            let steps = Steps::QueryItems {
                meter,
                file: &decl.file,
                line: Some(decl.line),
            };
            let proven = !self.use_ambiguous[position]
                && !self.use_bound[position].iter().any(|bound| *bound)
                && scope.imports_std(self, position, steps)?;
            if proven != was_proven {
                reserve_next(&mut changes, steps)?;
                changes.push(position);
            }
        }
        Ok(changes)
    }
}

impl From<StorageError> for BuildError {
    fn from(error: StorageError) -> Self {
        Self::Storage(error)
    }
}

/// Facts shared by queries that prove the implicit standard-library root.
#[derive(Debug)]
pub(crate) struct StdScope {
    pub(crate) disabled: Vec<bool>,
    pub(crate) uncertain_globs: Vec<bool>,
    opaque_routes: Vec<bool>,
}

fn disables(attrs: &[AttrFact]) -> bool {
    attrs.iter().any(|attr| {
        work::add(Work::ReachFormatBytes, attr.path.len() + attr.args.len());
        matches!(
            attr.path.as_str(),
            "no_implicit_prelude" | "no_std" | "no_core" | "macro_use"
        ) || (attr.path == "cfg_attr"
            && attr
                .args
                .split(|ch: char| !ch.is_alphanumeric() && ch != '_')
                .any(|word| {
                    matches!(
                        word,
                        "no_implicit_prelude" | "no_std" | "no_core" | "macro_use"
                    )
                }))
    })
}

impl StdScope {
    pub(crate) fn new(
        index: &Index,
        files: &BTreeMap<RelPath, SourceFile>,
        meter: &Meter,
        file: &RelPath,
    ) -> Result<Self, LoadError> {
        meter.items(size_of::<Self>(), file, None)?;
        let mut admission = ItemAdmission::new(meter, file);
        let steps = Steps::QueryItems {
            meter,
            file,
            line: None,
        };
        let built = Memory::new(&mut admission)
            .try_scope(|memory| Self::build(index, files, memory, steps));
        built.map_err(|error| match error {
            BuildError::Storage(error) => admission.error(error),
            BuildError::Read(error) => error,
        })
    }

    fn build(
        index: &Index,
        files: &BTreeMap<RelPath, SourceFile>,
        memory: &mut Memory<'_, impl Admission>,
        steps: Steps<'_>,
    ) -> Result<Self, BuildError> {
        let mut disabled = Vec::new();
        memory.reserve(&mut disabled, index.modules.len())?;
        disabled.resize(index.modules.len(), false);
        let mut opaque_routes = Vec::new();
        memory.reserve(&mut opaque_routes, index.modules.len())?;
        opaque_routes.resize(index.modules.len(), false);
        Self::mark_modules(index, files, &mut disabled, &mut opaque_routes, memory)?;
        let mut children = Vec::new();
        memory.reserve(&mut children, index.modules.len())?;
        children.resize_with(index.modules.len(), Vec::<usize>::new);
        for (at, module) in index.modules.iter().enumerate() {
            if let Some(parent) = module.parent {
                memory.push(&mut children[parent], at)?;
            }
        }
        Self::propagate(&mut disabled, &children, memory)?;
        Self::propagate(&mut opaque_routes, &children, memory)?;
        Self::release_edges(children, memory)?;
        let mut uncertain_globs = Vec::new();
        memory.reserve(&mut uncertain_globs, index.modules.len())?;
        uncertain_globs.resize(index.modules.len(), false);
        let mut readers = Vec::new();
        memory.reserve(&mut readers, index.modules.len())?;
        readers.resize_with(index.modules.len(), Vec::<usize>::new);
        for (module, uncertain) in uncertain_globs.iter_mut().enumerate() {
            for &position in index.globs_of(module) {
                work::add(Work::ReachFormatTokens, 1);
                match index
                    .reach_outcome(index.glob_targets(position, steps), steps)
                    .map_err(BuildError::Read)?
                {
                    Some(targets) if !targets.is_empty() => {
                        for target in targets {
                            memory.push(&mut readers[target], module)?;
                        }
                    }
                    _ => *uncertain = true,
                }
            }
        }
        Self::propagate(&mut uncertain_globs, &readers, memory)?;
        Self::propagate(&mut opaque_routes, &readers, memory)?;
        Self::release_edges(readers, memory)?;
        Index::query_refusal(steps)
            .map_err(|refused| BuildError::Read(index.refused_load(refused)))?;
        Ok(Self {
            disabled,
            uncertain_globs,
            opaque_routes,
        })
    }

    fn mark_modules(
        index: &Index,
        files: &BTreeMap<RelPath, SourceFile>,
        disabled: &mut [bool],
        opaque: &mut [bool],
        memory: &mut Memory<'_, impl Admission>,
    ) -> Result<(), StorageError> {
        for file in files.values() {
            for reach in &file.reached {
                let root = index.target_roots[reach.target.index()];
                let Some(&module) = index.module_by_path.get(&(root, reach.module_path.clone()))
                else {
                    continue;
                };
                disabled[module] |= disables(&file.facts.attrs);
                opaque[module] |= has_expansion_attributes(&file.facts.attrs);
                let mut pending = WorkList::<_, 8>::default();
                pending.try_push_admitted((module, &file.facts.body), memory)?;
                while let Some((module, body)) = pending.pop() {
                    work::add(Work::ReachFormatTokens, 1);
                    disabled[module] |= body.items.iter().any(|item| disables(&item.attrs));
                    opaque[module] |= body.expansion_attributes
                        || !body.unexpanded.is_empty()
                        || body
                            .items
                            .iter()
                            .any(|item| has_expansion_attributes(&item.attrs));
                    for child in &body.mods {
                        opaque[module] |= has_expansion_attributes(&child.attrs);
                        disabled[module] |= child.attrs.iter().any(|attr| {
                            attr.path == "macro_use"
                                || (attr.path == "cfg_attr" && attr.args.contains("macro_use"))
                        });
                        let Some(child_module) = index.scopes[module]
                            .get(child.name.as_str())
                            .and_then(|entry| {
                                entry.defs[slot(Namespace::Type)]
                                    .keys()
                                    .filter_map(|id| index.module_of.get(id).copied())
                                    .find(|&at| {
                                        index.modules[at].parent == Some(module)
                                            && index.modules[at].root == root
                                            && index.modules[at].path.rsplit("::").next()
                                                == Some(child.name.as_str())
                                    })
                            })
                        else {
                            continue;
                        };
                        disabled[child_module] |= disables(&child.attrs);
                        if let Some(body) = &child.inline {
                            pending.try_push_admitted((child_module, body), memory)?;
                        }
                    }
                }
                pending.release_admitted(memory)?;
            }
        }
        Ok(())
    }

    pub(crate) fn propagate(
        marked: &mut [bool],
        edges: &[Vec<usize>],
        memory: &mut Memory<'_, impl Admission>,
    ) -> Result<(), StorageError> {
        let mut pending = WorkList::<_, 8>::default();
        for (at, &active) in marked.iter().enumerate() {
            if active {
                pending.try_push_admitted(at, memory)?;
            }
        }
        // Each activated module scans its own edges once, including duplicates.
        while let Some(at) = pending.pop() {
            for &next in &edges[at] {
                work::add(Work::ReachFormatTokens, 1);
                if !marked[next] {
                    marked[next] = true;
                    pending.try_push_admitted(next, memory)?;
                }
            }
        }
        pending.release_admitted(memory)
    }

    fn release_edges(
        mut edges: Vec<Vec<usize>>,
        memory: &mut Memory<'_, impl Admission>,
    ) -> Result<(), StorageError> {
        for row in edges.drain(..) {
            memory.release_vec(row)?;
        }
        memory.release_vec(edges)
    }

    fn implicit_available(&self, index: &Index, module: usize, test: bool) -> bool {
        !self.disabled[module]
            && !index.no_std(module)
            && index.extern_name(module, "std", test).is_none()
    }

    pub(crate) fn sysroot(
        &self,
        index: &Index,
        module: usize,
        test: bool,
        leading: bool,
        own: Option<usize>,
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        if !self.implicit_available(index, module, test) || index.edition(module) < 2018 {
            return Ok(false);
        }
        let root = index.modules[module].root;
        let Some(root_read) = index.reach_outcome(
            index.read_lookup(root, Namespace::Type, "std", own, None, steps),
            steps,
        )?
        else {
            return Ok(false);
        };
        if root_read.as_ref().is_some_and(|(bound, _)| {
            bound
                .keys()
                .any(|id| index.extern_names.get(id).is_some_and(|name| name != "std"))
        }) {
            return Ok(false);
        }
        if leading {
            return Ok(true);
        }
        let Some(local_read) = index.reach_outcome(
            index.read_lookup(module, Namespace::Type, "std", own, None, steps),
            steps,
        )?
        else {
            return Ok(false);
        };
        Ok(local_read.as_ref().is_none()
            && !index.blocked(module, "std", Namespace::Type, own)
            && !self.uncertain_globs[module])
    }

    /// Confirm that a named import reads `std` through the implicit root.
    pub(crate) fn imports_std(
        &self,
        index: &Index,
        position: usize,
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        let decl = &index.uses[position];
        let module = decl.module;
        let root = index.modules[module].root;
        if self.opaque_routes[module] || self.opaque_routes[root] {
            return Ok(false);
        }
        let modern = index.edition(module) >= 2018;
        let implicit = if modern {
            self.sysroot(
                index,
                module,
                decl.test,
                decl.leading_colon,
                Some(position),
                steps,
            )?
        } else {
            self.implicit_available(index, module, decl.test)
                && !self.uncertain_globs[root]
                && !index.blocked(root, "std", Namespace::Type, Some(position))
                && index
                    .reach_outcome(
                        index.read_lookup(
                            root,
                            Namespace::Type,
                            "std",
                            Some(position),
                            None,
                            steps,
                        ),
                        steps,
                    )?
                    .is_some_and(|read| read.as_ref().is_none())
        };
        if !implicit {
            return Ok(false);
        }
        if !modern || !decl.leading_colon {
            let reader = if modern { module } else { root };
            let pending = index.reach_outcome(
                index.read_pending_at(reader, "std", slot(Namespace::Type), None, None, steps),
                steps,
            )?;
            if pending != Some(false) || !index.globbed_outside(reader, "std", steps)?.is_empty() {
                return Ok(false);
            }
        }
        Ok(index
            .reach_outcome(index.read_outside_crate(position, steps), steps)?
            .flatten()
            .as_deref()
            == Some("std"))
    }
}
