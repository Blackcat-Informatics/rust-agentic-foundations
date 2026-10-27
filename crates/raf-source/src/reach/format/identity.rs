// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Positive, deliberately narrow provenance for the standard format macro.

use purrdf_lex::allocation::{Admission, Memory, StorageError};
use purrdf_lex::walk::WorkList;
use std::collections::{BTreeMap, BTreeSet};
use std::mem::size_of;

use super::{Call, Formats, Import, ScopeKind};
use crate::RelPath;
use crate::facts::AttrFact;
use crate::index::{Index, UseKind};
use crate::item::{Namespace, SymbolKind};
use crate::meter::{ItemAdmission, Meter, Steps};
use crate::reach::{ReachError, Unit};
use crate::resolve::Heads;
use crate::work::{self, Work};
use crate::workspace::SourceFile;

enum BuildError {
    Storage(StorageError),
    Read(crate::LoadError),
}

impl From<StorageError> for BuildError {
    fn from(error: StorageError) -> Self {
        Self::Storage(error)
    }
}

/// Context facts owned once by a reachability walk, not once per invocation.
#[derive(Debug)]
pub(in crate::reach) struct Resolver {
    disabled: Vec<bool>,
    uncertain_globs: Vec<bool>,
}

fn capture_macro(name: &str) -> bool {
    name == "format"
}

fn expression_macro(name: &str) -> bool {
    matches!(
        name,
        "format"
            | "format_args"
            | "print"
            | "println"
            | "eprint"
            | "eprintln"
            | "write"
            | "writeln"
            | "panic"
            | "assert"
            | "assert_eq"
            | "assert_ne"
            | "debug_assert"
            | "debug_assert_eq"
            | "debug_assert_ne"
            | "vec"
    )
}

fn endpoint(path: &[String], allowed: fn(&str) -> bool) -> bool {
    path.len() == 2 && path[0] == "std" && allowed(&path[1])
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

impl Resolver {
    pub(in crate::reach) fn new(
        index: &Index,
        files: &BTreeMap<RelPath, SourceFile>,
        meter: &Meter,
        file: &RelPath,
    ) -> Result<Self, ReachError> {
        meter
            .items(size_of::<Self>(), file, None)
            .map_err(ReachError::Budget)?;
        let mut admission = ItemAdmission::new(meter, file);
        let steps = Steps::QueryItems {
            meter,
            file,
            line: None,
        };
        let built = Memory::new(&mut admission)
            .try_scope(|memory| Self::build(index, files, memory, steps));
        built.map_err(|error| match error {
            BuildError::Storage(error) => Self::construction_error(&mut admission, error),
            BuildError::Read(error) => error.into(),
        })
    }

    fn construction_error(admission: &mut ItemAdmission<'_>, error: StorageError) -> ReachError {
        let error = admission.error(error);
        if matches!(error.kind, crate::LoadErrorKind::Budget { .. }) {
            ReachError::Budget(error)
        } else {
            ReachError::Source(error)
        }
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
        Self::mark_disabled(index, files, &mut disabled, memory)?;
        let mut children = Vec::new();
        memory.reserve(&mut children, index.modules.len())?;
        children.resize_with(index.modules.len(), Vec::<usize>::new);
        for (at, module) in index.modules.iter().enumerate() {
            if let Some(parent) = module.parent {
                memory.push(&mut children[parent], at)?;
            }
        }
        Self::propagate(&mut disabled, &children, memory)?;
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
        Self::release_edges(readers, memory)?;
        Index::query_refusal(steps)
            .map_err(|refused| BuildError::Read(index.refused_load(refused)))?;
        Ok(Self {
            disabled,
            uncertain_globs,
        })
    }

    fn mark_disabled(
        index: &Index,
        files: &BTreeMap<RelPath, SourceFile>,
        disabled: &mut [bool],
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
                let mut pending = WorkList::<_, 8>::default();
                pending.try_push_admitted((module, &file.facts.body), memory)?;
                while let Some((module, body)) = pending.pop() {
                    work::add(Work::ReachFormatTokens, 1);
                    disabled[module] |= body.items.iter().any(|item| disables(&item.attrs));
                    for child in &body.mods {
                        disabled[module] |= child.attrs.iter().any(|attr| {
                            attr.path == "macro_use"
                                || (attr.path == "cfg_attr" && attr.args.contains("macro_use"))
                        });
                        let Some(child_module) = index.scopes[module]
                            .get(child.name.as_str())
                            .and_then(|entry| {
                                entry.defs[crate::settle::slot(Namespace::Type)]
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

    fn propagate(
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

    fn sysroot(
        &self,
        index: &Index,
        module: usize,
        test: bool,
        leading: bool,
        steps: Steps<'_>,
    ) -> Result<bool, ReachError> {
        if self.disabled[module]
            || index.no_std(module)
            || index.extern_name(module, "std", test).is_some()
        {
            return Ok(false);
        }
        if index.edition(module) < 2018 {
            return Ok(false);
        }
        let root = index.modules[module].root;
        let root_read = index.reach_outcome(
            index.read_lookup(root, Namespace::Type, "std", None, None, steps),
            steps,
        )?;
        if root_read
            .as_ref()
            .and_then(|read| read.as_ref())
            .is_some_and(|(bound, _)| {
                bound
                    .keys()
                    .any(|id| index.extern_names.get(id).is_some_and(|name| name != "std"))
            })
        {
            return Ok(false);
        }
        if leading {
            return Ok(true);
        }
        let local_read = index.reach_outcome(
            index.read_lookup(module, Namespace::Type, "std", None, None, steps),
            steps,
        )?;
        Ok(local_read.as_ref().and_then(|read| read.as_ref()).is_none()
            && !index.blocked(module, "std", Namespace::Type, None)
            && !self.uncertain_globs[module])
    }

    fn imported(
        &self,
        index: &Index,
        module: usize,
        test: bool,
        import: &Import,
        allowed: fn(&str) -> bool,
        steps: Steps<'_>,
    ) -> Result<bool, ReachError> {
        Ok(!import.uncertain
            && endpoint(&import.path, allowed)
            && self.sysroot(index, module, test, import.leading, steps)?)
    }

    fn glob_clear(
        &self,
        index: &Index,
        module: usize,
        test: bool,
        (name, namespace): (&str, Namespace),
        import: &Import,
        steps: Steps<'_>,
    ) -> Result<bool, ReachError> {
        if import.uncertain {
            return Ok(false);
        }
        let Some(walked) = index.reach_outcome(
            index.walk(
                module,
                &import.path,
                import.leading,
                Namespace::Type,
                Heads::Use { test, own: None },
                steps,
            ),
            steps,
        )?
        else {
            return Ok(false);
        };
        let mut any = false;
        for target in walked
            .modules
            .iter()
            .copied()
            .chain(walked.items.keys().flat_map(|id| index.modules_for(*id)))
        {
            any = true;
            let read = index.reach_outcome(
                index.read_lookup(target, namespace, name, None, None, steps),
                steps,
            )?;
            if self.uncertain_globs[target]
                || read.as_ref().and_then(|read| read.as_ref()).is_some()
                || index.blocked(target, name, namespace, None)
            {
                return Ok(false);
            }
        }
        Ok(any)
    }

    fn std_unbound(
        &self,
        formats: &Formats,
        mut at: usize,
        index: &Index,
        module: usize,
        test: bool,
        steps: Steps<'_>,
    ) -> Result<bool, ReachError> {
        loop {
            work::add(Work::ReachFormatTokens, 1);
            let scope = &formats.scopes[at];
            if scope.std_bound || scope.imports.contains_key("std") || scope.unknown {
                return Ok(false);
            }
            for import in &scope.globs {
                if !self.glob_clear(index, module, test, ("std", Namespace::Type), import, steps)? {
                    return Ok(false);
                }
            }
            if matches!(scope.kind, ScopeKind::Module) {
                return Ok(true);
            }
            let Some(parent) = scope.parent else {
                return Ok(true);
            };
            at = parent;
        }
    }

    pub(in crate::reach) fn standard(
        &self,
        formats: &Formats,
        call: &Call,
        index: &Index,
        unit: &Unit<'_>,
        steps: Steps<'_>,
    ) -> Result<bool, ReachError> {
        let answer = self.standard_counted(formats, call, index, unit, steps);
        Index::query_refusal(steps).map_err(|refused| index.refused_load(refused))?;
        answer
    }

    fn standard_counted(
        &self,
        formats: &Formats,
        call: &Call,
        index: &Index,
        unit: &Unit<'_>,
        steps: Steps<'_>,
    ) -> Result<bool, ReachError> {
        if !self.proven(formats, (call, capture_macro), index, unit, steps)? {
            return Ok(false);
        }
        let mut scope = Some(call.scope);
        while let Some(at) = scope {
            work::add(Work::ReachFormatTokens, 1);
            let here = &formats.scopes[at];
            if let Some(enclosing) = here.enclosing
                && !self.proven(
                    formats,
                    (&formats.calls[enclosing], expression_macro),
                    index,
                    unit,
                    steps,
                )?
            {
                return Ok(false);
            }
            scope = here.parent;
        }
        Ok(true)
    }

    fn proven(
        &self,
        formats: &Formats,
        (call, allowed): (&Call, fn(&str) -> bool),
        index: &Index,
        unit: &Unit<'_>,
        steps: Steps<'_>,
    ) -> Result<bool, ReachError> {
        let (module, test) = (unit.module, unit.test);
        if self.disabled[module] {
            return Ok(false);
        }
        let Some(name) = call.path.last() else {
            return Ok(false);
        };
        let qualified = endpoint(&call.path, allowed);
        if !qualified && (call.path.len() != 1 || call.leading) {
            return Ok(false);
        }
        if qualified
            && !call.leading
            && !self.std_unbound(formats, call.scope, index, module, test, steps)?
        {
            return Ok(false);
        }
        let mut scope = Some(call.scope);
        let mut imported = None;
        let mut lexical_macros = BTreeSet::new();
        let mut outside_module = false;
        while let Some(at) = scope {
            work::add(Work::ReachFormatTokens, 1);
            let here = &formats.scopes[at];
            if here.disabled || (!outside_module && here.unknown) {
                return Ok(false);
            }
            if !qualified {
                if let Some(declarations) = here.macros.get(name) {
                    if declarations.iter().any(|&start| start <= call.start) {
                        return Ok(false);
                    }
                    lexical_macros.extend(declarations.iter().copied());
                }
                if !outside_module
                    && imported.is_none()
                    && let Some(imports) = here.imports.get(name)
                {
                    if imports.len() != 1
                        || !self.imported(index, module, test, &imports[0], allowed, steps)?
                        || (!imports[0].leading
                            && !self.std_unbound(formats, at, index, module, test, steps)?)
                    {
                        return Ok(false);
                    }
                    imported = Some(true);
                }
                if !outside_module {
                    for import in &here.globs {
                        if !self.glob_clear(
                            index,
                            module,
                            test,
                            (name, Namespace::Macro),
                            import,
                            steps,
                        )? {
                            return Ok(false);
                        }
                    }
                }
            }
            outside_module |= matches!(here.kind, ScopeKind::Module);
            scope = here.parent;
        }
        if qualified {
            return self.sysroot(index, module, test, call.leading, steps);
        }
        if !Self::indexed_macros_clear(index, unit, name, &lexical_macros, steps)? {
            return Ok(false);
        }
        if imported.is_some() {
            return Ok(true);
        }
        let positions = index.imports_of(module, name);
        if positions.len() == 1 {
            let position = positions[0];
            let decl = &index.uses[position];
            if let UseKind::Named { segs, .. } = &decl.kind {
                return Ok(endpoint(segs, allowed)
                    && index
                        .reach_outcome(index.read_outside_crate(position, steps), steps)?
                        .flatten()
                        .as_deref()
                        == Some("std")
                    && self.sysroot(index, module, test, decl.leading_colon, steps)?);
            }
        }
        if !allowed(name)
            || !positions.is_empty()
            || self.uncertain_globs[module]
            || index.blocked(module, name, Namespace::Macro, None)
        {
            return Ok(false);
        }
        Ok(!index.no_std(module) && index.extern_name(module, "std", test).is_none())
    }

    // A settled textual definition later in the same lexical scope does not
    // shadow an earlier call. Other indexed macro definitions always do.
    fn indexed_macros_clear(
        index: &Index,
        unit: &Unit<'_>,
        name: &str,
        lexical_macros: &BTreeSet<usize>,
        steps: Steps<'_>,
    ) -> Result<bool, ReachError> {
        let (module, file) = (unit.module, unit.file);
        let read = index.reach_outcome(
            index.read_lookup(module, Namespace::Macro, name, None, None, steps),
            steps,
        )?;
        if let Some((bound, _)) = read.as_ref().and_then(|read| read.as_ref())
            && bound.keys().any(|id| {
                let item = &index.items[id.0];
                item.file != *file || !lexical_macros.contains(&item.span.end_byte)
            })
        {
            return Ok(false);
        }
        let mut ancestor = index.modules[module].parent;
        while let Some(parent) = ancestor {
            work::add(Work::ReachFormatTokens, 1);
            if index.modules[parent].defs.iter().any(|id| {
                let item = &index.items[id.0];
                item.kind == SymbolKind::MacroRules && item.name == name && item.file != *file
            }) {
                return Ok(false);
            }
            ancestor = index.modules[parent].parent;
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use purrdf_lex::allocation::{Memory, StorageError};

    use super::Resolver;
    use crate::meter::{ItemAdmission, Meter};
    use crate::reach::ReachError;
    use crate::work::{self, Work};
    use crate::{Budget, LoadErrorKind, LoadOptions, RelPath};

    #[test]
    fn native_propagation_visits_cycles_and_duplicate_edges_once_per_active_source() {
        let meter = Meter::new(&LoadOptions::default());
        let file = RelPath::new("src/lib.rs").expect("path");
        let mut admission = ItemAdmission::new(&meter, &file);
        let mut memory = Memory::new(&mut admission);
        let edges = [vec![1, 1], vec![2], vec![0], vec![3]];
        let mut marked = [true, false, false, false];
        let (result, tally) = work::measure_kind(Work::ReachFormatTokens, 4, || {
            Resolver::propagate(&mut marked, &edges, &mut memory)
        });
        result.expect("cycle closure");
        assert_eq!(marked, [true, true, true, false]);
        assert_eq!(tally.get(Work::ReachFormatTokens), 4);
        assert_eq!(memory.admitted_bytes(), 0);
    }

    #[test]
    fn native_propagation_spill_refuses_without_refunding_the_cumulative_grant() {
        let file = RelPath::new("src/lib.rs").expect("path");
        let run = |limit| {
            let meter = Meter::new(&LoadOptions {
                max_item_bytes: limit,
                ..LoadOptions::default()
            });
            let mut admission = ItemAdmission::new(&meter, &file);
            let (result, retained) = {
                let mut memory = Memory::new(&mut admission);
                let mut marked = [true; 24];
                let edges = std::array::from_fn::<_, 24, _>(|_| Vec::new());
                let result =
                    memory.scope(|memory| Resolver::propagate(&mut marked, &edges, memory));
                (result, memory.admitted_bytes())
            };
            if result.is_ok() {
                assert_eq!(retained, 0);
            } else {
                // Sticky admission preserves the grant until its owner dies;
                // a refused rollback must not replace the original limit error.
                assert!(retained > 0);
            }
            (
                result.map_err(|error| admission.error(error)),
                meter.usage().item_bytes,
            )
        };
        let (result, charged) = run(usize::MAX);
        result.expect("spill admitted");
        assert!(charged > 0);
        run(charged).0.expect("exact cumulative grant");
        let error = run(charged - 1).0.expect_err("one byte short");
        assert_eq!(error.file, file);
        assert!(matches!(error.kind, LoadErrorKind::Budget {
            budget: Budget::ItemBytes, limit,
        } if limit == charged - 1));
        // A native physical failure remains distinguishable from admission.
        let meter = Meter::new(&LoadOptions::default());
        let mut admission = ItemAdmission::new(&meter, &file);
        let error = Resolver::construction_error(&mut admission, StorageError::AllocationFailed);
        assert!(matches!(error, ReachError::Source(ref load)
            if matches!(load.kind, LoadErrorKind::Storage(StorageError::AllocationFailed))));
        assert!(!error.to_string().contains("over budget"));
        let meter = Meter::new(&LoadOptions {
            max_item_bytes: 0,
            ..LoadOptions::default()
        });
        let mut admission = ItemAdmission::new(&meter, &file);
        let native = purrdf_lex::allocation::Admission::resize(&mut admission, 1)
            .expect_err("admission refusal");
        let error = Resolver::construction_error(&mut admission, native);
        assert!(matches!(error, ReachError::Budget(ref load)
            if matches!(load.kind, LoadErrorKind::Budget { budget: Budget::ItemBytes, limit: 0 })));
        assert!(error.to_string().contains("over budget"));
    }
}
