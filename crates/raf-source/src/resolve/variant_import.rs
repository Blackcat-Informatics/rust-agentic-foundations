// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Named imports of enum members that have no ordinary indexed binding.

use std::collections::BTreeSet;

use purrdf_lex::walk::WorkList;

use super::{Fail, Heads, Index, Source, Steps, strip_raw, valid_code_segment};
use crate::cost::vec_slot;
use crate::facts::ConfigurationDependence;
use crate::glob_scope::{push_work, set_bytes};
use crate::index::UseKind;
use crate::{ItemId, Namespace, ResolveError, SymbolKind};

enum Prefix {
    Module(usize),
    Item(ItemId),
}

enum Leaf {
    Refused,
    Named(usize),
    Absent,
}

#[derive(Clone, Copy)]
struct ImportRoute<'a> {
    from: usize,
    segments: &'a [String],
    colon: bool,
    heads: Heads,
    module: Option<usize>,
}

#[derive(Clone, Copy)]
struct MemberQuery<'a> {
    path: &'a str,
    reader: usize,
    namespace: Namespace,
    steps: Steps<'a>,
}

type Candidate<'a> = (ItemId, &'a str);
type Read<'a> = (usize, usize, Option<usize>, &'a str);
type TypeRead<'a> = (usize, usize, Namespace, Option<usize>, &'a str);

impl Index {
    pub(crate) fn enum_member_import(
        &self,
        from: usize,
        path: &str,
        namespace: Namespace,
        heads: Heads,
        steps: Steps<'_>,
    ) -> Result<Option<Vec<(ItemId, String)>>, ResolveError> {
        if !matches!(namespace, Namespace::Type | Namespace::Value) {
            return Ok(Some(Vec::new()));
        }
        let (colon, written) = read_path(path, steps)?;
        let first = ImportRoute {
            from,
            segments: &written,
            colon,
            heads,
            module: None,
        };
        let query = MemberQuery {
            path,
            reader: from,
            namespace,
            steps,
        };
        let mut pending = WorkList::<ImportRoute<'_>, 8>::new();
        push_work(&mut pending, first, steps).map_err(ResolveError::storage)?;
        let mut visited = BTreeSet::<Read<'_>>::new();
        let mut candidates = BTreeSet::<Candidate<'_>>::new();
        while let Some(route) = pending.pop() {
            let Some((name, prefix)) = route.segments.split_last() else {
                return Ok(None);
            };
            let Some(scope) = self.enum_route_scope(route, name, prefix, path, steps)? else {
                return Ok(None);
            };
            let module = match scope {
                Prefix::Module(module) => module,
                Prefix::Item(id) => {
                    if route.heads.own().is_some()
                        && self.enum_member_candidate(id, name, namespace)
                    {
                        insert(&mut candidates, (id, name.as_str()), steps)?;
                    }
                    continue;
                }
            };
            let key = (module, route.from, route.heads.own(), name.as_str());
            if !insert(&mut visited, key, steps)? {
                continue;
            }
            match self.enum_import_leaf(
                (route.from, from),
                module,
                name,
                namespace,
                route.heads,
                steps,
            )? {
                Leaf::Refused => return Ok(None),
                Leaf::Named(position) => {
                    let decl = &self.uses[position];
                    let UseKind::Named { segs, .. } = &decl.kind else {
                        return Ok(None);
                    };
                    push_work(
                        &mut pending,
                        ImportRoute {
                            from: decl.module,
                            segments: segs,
                            colon: decl.leading_colon,
                            heads: decl.heads(position),
                            module: None,
                        },
                        steps,
                    )
                    .map_err(ResolveError::storage)?;
                }
                Leaf::Absent => {
                    if !self.follow_enum_globs(
                        (route, module, name),
                        query,
                        &mut pending,
                        &mut candidates,
                    )? {
                        return Ok(None);
                    }
                }
            }
        }
        steps
            .reserve(
                candidates
                    .len()
                    .saturating_mul(vec_slot::<(ItemId, String)>()),
            )
            .map_err(ResolveError::storage)?;
        let candidates = candidates
            .into_iter()
            .map(|(id, name)| variant_name(id, name, steps))
            .collect::<Result<_, _>>()?;
        Ok(Some(candidates))
    }

    fn follow_enum_globs<'a>(
        &'a self,
        (route, module, name): (ImportRoute<'a>, usize, &'a str),
        query: MemberQuery<'_>,
        pending: &mut WorkList<ImportRoute<'a>, 8>,
        candidates: &mut BTreeSet<Candidate<'a>>,
    ) -> Result<bool, ResolveError> {
        let MemberQuery {
            path,
            reader,
            namespace,
            steps,
        } = query;
        let Some(globs) =
            self.enum_import_globs((route.from, reader), module, route.heads, steps)?
        else {
            return Ok(false);
        };
        for &position in globs.iter() {
            let decl = &self.uses[position];
            let UseKind::Glob { segs } = &decl.kind else {
                return Ok(false);
            };
            let heads = decl.heads(position);
            match self.enum_import_prefix(
                decl.module,
                segs,
                decl.leading_colon,
                heads,
                path,
                steps,
            )? {
                Some(Prefix::Module(module)) => push_work(
                    pending,
                    ImportRoute {
                        from: decl.module,
                        heads,
                        module: Some(module),
                        ..route
                    },
                    steps,
                )
                .map_err(ResolveError::storage)?,
                Some(Prefix::Item(id)) => {
                    let Some(present) = self.enum_glob_member(id, name, namespace, steps)? else {
                        return Ok(false);
                    };
                    if present {
                        insert(candidates, (id, name), steps)?;
                    }
                }
                None => return Ok(false),
            }
        }
        Ok(true)
    }

    fn enum_route_scope(
        &self,
        route: ImportRoute<'_>,
        name: &str,
        prefix: &[String],
        path: &str,
        steps: Steps<'_>,
    ) -> Result<Option<Prefix>, ResolveError> {
        if let Some(module) = route.module {
            return Ok(Some(Prefix::Module(module)));
        }
        if !prefix.is_empty() {
            return self.enum_import_prefix(
                route.from,
                prefix,
                route.colon,
                route.heads,
                path,
                steps,
            );
        }
        Ok(
            if self.crate_relative(route.from, name, route.colon, route.heads) {
                Some(Prefix::Module(self.modules[route.from].root))
            } else if route.colon || matches!(name, "self" | "super" | "crate") {
                None
            } else {
                Some(Prefix::Module(route.from))
            },
        )
    }

    fn enum_member_candidate(&self, id: ItemId, name: &str, namespace: Namespace) -> bool {
        self.items[id.index()].kind == SymbolKind::Enum
            && !self
                .assoc
                .get(&id)
                .and_then(|members| members.get(name))
                .is_some_and(|items| {
                    items
                        .iter()
                        .any(|id| self.items[id.index()].in_namespace(namespace))
                })
    }

    fn enum_glob_member(
        &self,
        id: ItemId,
        name: &str,
        namespace: Namespace,
        steps: Steps<'_>,
    ) -> Result<Option<bool>, ResolveError> {
        let Some(variants) = self.items[id.index()].written_enum_variants() else {
            return Ok(None);
        };
        for variant in variants {
            steps
                .storage_work(variant.name.len().max(name.len()).saturating_add(1))
                .map_err(ResolveError::storage)?;
            if variant.name == name
                && (namespace == Namespace::Type || variant.shape != crate::FieldShape::Named)
            {
                return Ok(Some(true));
            }
        }
        Ok(Some(false))
    }

    pub(crate) fn unconditional_type_path(
        &self,
        from: usize,
        path: &str,
        heads: Heads,
        steps: Steps<'_>,
    ) -> Result<Option<ItemId>, ResolveError> {
        let (colon, written) = read_path(path, steps)?;
        Ok(
            match self.enum_import_prefix(from, &written, colon, heads, path, steps)? {
                Some(Prefix::Item(id)) => Some(id),
                Some(Prefix::Module(_)) | None => None,
            },
        )
    }

    fn enum_import_leaf(
        &self,
        (viewer, reader): (usize, usize),
        module: usize,
        name: &str,
        namespace: Namespace,
        heads: Heads,
        steps: Steps<'_>,
    ) -> Result<Leaf, ResolveError> {
        if !self.module_is_unconditional(module, steps)? {
            return Ok(Leaf::Refused);
        }
        let bound = self.read_lookup(module, namespace, name, heads.own(), Some(viewer), steps);
        if enum_import_walk(self, bound)?.is_none_or(|lookup| lookup.as_ref().is_some()) {
            return Ok(Leaf::Refused);
        }
        let test = matches!(
            heads,
            Heads::Use { test: true, .. } | Heads::Code { test: true }
        );
        let mut found = None;
        for &position in self.imports_of(module, name) {
            let decl = &self.uses[position];
            steps.storage_work(1).map_err(ResolveError::storage)?;
            if heads.own() == Some(position)
                || !self.binds_name(position, module, namespace, name)
                || !test && decl.test
                || enum_import_walk(self, self.read_visibility(&decl.vis, module, viewer, steps))?
                    != Some(true)
            {
                continue;
            }
            if self.named_rhs_proves_enum_absence(position, namespace, steps)? {
                continue;
            }
            if decl.configuration == ConfigurationDependence::Conditional
                || found.replace(position).is_some()
                || viewer != reader
                    && enum_import_walk(
                        self,
                        self.read_visibility(&decl.vis, module, reader, steps),
                    )? != Some(true)
            {
                return Ok(Leaf::Refused);
            }
        }
        Ok(found.map_or(Leaf::Absent, Leaf::Named))
    }

    fn named_rhs_proves_enum_absence(
        &self,
        position: usize,
        namespace: Namespace,
        steps: Steps<'_>,
    ) -> Result<bool, ResolveError> {
        let at = crate::settle::slot(namespace);
        if !(self.use_free[position][at]
            && self.use_decided[position][at]
            && self.use_empty[position][at])
        {
            return Ok(false);
        }

        // Native empty slots describe indexed items. A forwarding import can
        // still carry an enum variant which has no item identity of its own.
        let decl = &self.uses[position];
        let UseKind::Named { segs, .. } = &decl.kind else {
            return Ok(false);
        };
        let Some((name, prefix)) = segs.split_last() else {
            return Ok(false);
        };
        let route = ImportRoute {
            from: decl.module,
            segments: segs,
            colon: decl.leading_colon,
            heads: decl.heads(position),
            module: None,
        };
        let Some(Prefix::Module(module)) =
            self.enum_route_scope(route, name, prefix, name, steps)?
        else {
            return Ok(false);
        };
        if !self.module_is_unconditional(module, steps)? || !self.globs_of(module).is_empty() {
            return Ok(false);
        }
        for &other in self.imports_of(module, name) {
            steps.storage_work(1).map_err(ResolveError::storage)?;
            if other != position && self.binds_name(other, module, namespace, name) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn enum_import_globs(
        &self,
        (viewer, reader): (usize, usize),
        module: usize,
        heads: Heads,
        steps: Steps<'_>,
    ) -> Result<Option<WorkList<usize, 8>>, ResolveError> {
        let test = matches!(
            heads,
            Heads::Use { test: true, .. } | Heads::Code { test: true }
        );
        let mut found = WorkList::new();
        for &position in self.globs_of(module) {
            let decl = &self.uses[position];
            steps.storage_work(1).map_err(ResolveError::storage)?;
            if heads.own() == Some(position)
                || !test && decl.test
                || enum_import_walk(self, self.read_visibility(&decl.vis, module, viewer, steps))?
                    != Some(true)
            {
                continue;
            }
            if decl.configuration == ConfigurationDependence::Conditional
                || viewer != reader
                    && enum_import_walk(
                        self,
                        self.read_visibility(&decl.vis, module, reader, steps),
                    )? != Some(true)
            {
                return Ok(None);
            }
            push_work(&mut found, position, steps).map_err(ResolveError::storage)?;
        }
        Ok(Some(found))
    }

    fn enum_import_prefix(
        &self,
        from: usize,
        prefix: &[String],
        colon: bool,
        heads: Heads,
        text: &str,
        steps: Steps<'_>,
    ) -> Result<Option<Prefix>, ResolveError> {
        let Some((scope, reads)) =
            self.enum_import_prefix_inputs(from, prefix, colon, heads, text, steps)?
        else {
            return Ok(None);
        };
        Ok(self
            .enum_import_prefix_reads(from, &reads, heads, steps)?
            .then_some(scope))
    }

    fn enum_import_prefix_inputs(
        &self,
        from: usize,
        prefix: &[String],
        colon: bool,
        heads: Heads,
        text: &str,
        steps: Steps<'_>,
    ) -> Result<Option<(Prefix, Vec<Source>)>, ResolveError> {
        let mut reads = Vec::new();
        let walk = self.walk_with_visibility(
            from,
            (prefix, colon, Namespace::Type),
            (heads, Some(from), Some(&mut reads)),
            steps,
        );
        let Some(walked) = enum_import_walk(self, walk)? else {
            return Ok(None);
        };
        if walked.items.len() + walked.modules.len() != 1 {
            return Ok(None);
        }
        self.path_determinacy(from, prefix, colon, Namespace::Type, heads, text, steps)?;
        if self
            .unresolved_glob_read(&reads, steps)
            .map_err(ResolveError::storage)?
            .is_some()
        {
            return Ok(None);
        }
        let module = if let Some(&module) = walked.modules.first() {
            module
        } else if let Some((&id, _)) = walked.items.first_key_value() {
            if self.items[id.index()].kind != SymbolKind::Mod {
                return Ok(Some((Prefix::Item(id), reads)));
            }
            let mut modules = self.modules_for(id);
            let Some(module) = modules.next().filter(|_| modules.next().is_none()) else {
                return Ok(None);
            };
            module
        } else {
            return Ok(None);
        };
        Ok(self
            .module_is_unconditional(module, steps)?
            .then_some((Prefix::Module(module), reads)))
    }

    fn enum_import_prefix_reads(
        &self,
        viewer: usize,
        reads: &[Source],
        heads: Heads,
        steps: Steps<'_>,
    ) -> Result<bool, ResolveError> {
        let mut pending = WorkList::<TypeRead<'_>, 8>::new();
        for source in reads {
            push_work(
                &mut pending,
                (
                    viewer,
                    source.module,
                    source.ns,
                    heads.own(),
                    source.name.as_str(),
                ),
                steps,
            )
            .map_err(ResolveError::storage)?;
        }
        let mut checked = BTreeSet::new();
        while let Some(read @ (viewer, module, namespace, own, name)) = pending.pop() {
            if !insert(&mut checked, read, steps)? {
                continue;
            }
            if !self.module_is_unconditional(module, steps)? {
                return Ok(false);
            }
            for &position in self.imports_of(module, name) {
                steps.storage_work(1).map_err(ResolveError::storage)?;
                if Some(position) != own
                    && self.uses[position].configuration == ConfigurationDependence::Conditional
                    && self.binds_name(position, module, namespace, name)
                {
                    return Ok(false);
                }
            }
            let bound = self.read_lookup(module, namespace, name, own, Some(viewer), steps);
            let Some(lookup) = enum_import_walk(self, bound)? else {
                return Ok(false);
            };
            let Some((bound, via_glob)) = lookup.as_ref() else {
                continue;
            };
            if bound.len() != 1 {
                return Ok(false);
            }
            if via_glob {
                let reading_heads =
                    own.map_or(heads, |position| self.uses[position].heads(position));
                if !self.audit_type_globs(read, reading_heads, &mut pending, steps)? {
                    return Ok(false);
                }
            } else if let Some((id, found)) = bound.iter().next() {
                if found.link.is_some() {
                    if !self.audit_named_type_import(read, *id, &mut pending, steps)? {
                        return Ok(false);
                    }
                } else if found.via_glob {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    fn audit_named_type_import<'a>(
        &'a self,
        (viewer, module, namespace, own, name): TypeRead<'a>,
        expected: ItemId,
        pending: &mut WorkList<TypeRead<'a>, 8>,
        steps: Steps<'_>,
    ) -> Result<bool, ResolveError> {
        let mut found = false;
        for &position in self.imports_of(module, name) {
            steps.storage_work(1).map_err(ResolveError::storage)?;
            let decl = &self.uses[position];
            if Some(position) == own
                || !self.use_bound[position][crate::settle::slot(namespace)]
                || enum_import_walk(self, self.read_visibility(&decl.vis, module, viewer, steps))?
                    != Some(true)
            {
                continue;
            }
            let UseKind::Named { segs, .. } = &decl.kind else {
                return Ok(false);
            };
            let heads = decl.heads(position);
            let mut reads = Vec::new();
            let walk = self.walk_with_visibility(
                module,
                (segs, decl.leading_colon, namespace),
                (heads, Some(module), Some(&mut reads)),
                steps,
            );
            let Some(walk) = enum_import_walk(self, walk)? else {
                return Ok(false);
            };
            let bindings = self
                .read_binds(namespace, Ok(walk), segs.len(), steps)
                .map_err(ResolveError::storage)?;
            if !bindings.iter().map(|(item, _)| *item).eq([expected]) {
                return Ok(false);
            }
            if !queue_import_reads((module, position), segs, reads, pending, steps)? {
                return Ok(false);
            }
            found = true;
        }
        Ok(found)
    }

    fn audit_type_globs<'a>(
        &'a self,
        (viewer, module, namespace, _, name): TypeRead<'a>,
        heads: Heads,
        pending: &mut WorkList<TypeRead<'a>, 8>,
        steps: Steps<'_>,
    ) -> Result<bool, ResolveError> {
        let Some(globs) = self.enum_import_globs((viewer, viewer), module, heads, steps)? else {
            return Ok(false);
        };
        if globs.is_empty() {
            return Ok(false);
        }
        for &position in globs.iter() {
            let decl = &self.uses[position];
            let UseKind::Glob { segs } = &decl.kind else {
                return Ok(false);
            };
            let Some((scope, reads)) = self.enum_import_prefix_inputs(
                module,
                segs,
                decl.leading_colon,
                decl.heads(position),
                name,
                steps,
            )?
            else {
                return Ok(false);
            };
            if !queue_import_reads((module, position), segs, reads, pending, steps)? {
                return Ok(false);
            }
            match scope {
                Prefix::Module(target) => push_work(
                    pending,
                    (module, target, namespace, Some(position), name),
                    steps,
                )
                .map_err(ResolveError::storage)?,
                Prefix::Item(id)
                    if self.enum_glob_member(id, name, namespace, steps)? == Some(false) => {}
                Prefix::Item(_) => return Ok(false),
            }
        }
        Ok(true)
    }

    pub(crate) fn module_is_unconditional(
        &self,
        mut module: usize,
        steps: Steps<'_>,
    ) -> Result<bool, ResolveError> {
        loop {
            steps.storage_work(1).map_err(ResolveError::storage)?;
            let current = &self.modules[module];
            if current.configuration == ConfigurationDependence::Conditional {
                return Ok(false);
            }
            let Some(parent) = current.parent else {
                return Ok(true);
            };
            module = parent;
        }
    }
}

fn queue_import_reads<'a>(
    (module, position): (usize, usize),
    segments: &'a [String],
    reads: Vec<Source>,
    pending: &mut WorkList<TypeRead<'a>, 8>,
    steps: Steps<'_>,
) -> Result<bool, ResolveError> {
    for read in reads {
        steps
            .storage_work(segments.len())
            .map_err(ResolveError::storage)?;
        let Some(segment) = segments
            .iter()
            .find(|segment| segment.as_str() == read.name)
        else {
            return Ok(false);
        };
        push_work(
            pending,
            (
                module,
                read.module,
                read.ns,
                Some(position),
                segment.as_str(),
            ),
            steps,
        )
        .map_err(ResolveError::storage)?;
    }
    Ok(true)
}

fn insert<T: Ord>(set: &mut BTreeSet<T>, value: T, steps: Steps<'_>) -> Result<bool, ResolveError> {
    steps
        .storage_work(set.len().bit_width() as usize + 1)
        .map_err(ResolveError::storage)?;
    if set.contains(&value) {
        return Ok(false);
    }
    steps
        .reserve(set_bytes::<T>(set.is_empty()))
        .map_err(ResolveError::storage)?;
    Ok(set.insert(value))
}

fn variant_name(
    id: ItemId,
    name: &str,
    steps: Steps<'_>,
) -> Result<(ItemId, String), ResolveError> {
    steps.reserve(name.len()).map_err(ResolveError::storage)?;
    Ok((id, name.to_owned()))
}

fn read_path(path: &str, steps: Steps<'_>) -> Result<(bool, Vec<String>), ResolveError> {
    let (colon, text) = path.strip_prefix("::").map_or((false, path), |p| (true, p));
    steps
        .storage_work(path.len())
        .map_err(ResolveError::storage)?;
    let count = text.match_indices("::").count() + 1;
    steps
        .reserve(
            path.len()
                .saturating_add(count.saturating_mul(size_of::<String>())),
        )
        .map_err(ResolveError::storage)?;
    let mut written = Vec::with_capacity(count);
    for part in text.split("::") {
        let name = strip_raw(part);
        if !valid_code_segment(part) {
            return Err(ResolveError::Invalid(path.to_owned()));
        }
        written.push(name.to_owned());
    }
    Ok((colon, written))
}

fn enum_import_walk<T>(index: &Index, result: Result<T, Fail>) -> Result<Option<T>, ResolveError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(Fail::Budget(error)) => Err(index.refused(error)),
        Err(Fail::Storage(error)) => Err(ResolveError::storage(error)),
        Err(
            Fail::Missing(_)
            | Fail::SuperAboveRoot
            | Fail::SeveralCrates { .. }
            | Fail::Ambiguous(_),
        ) => Ok(None),
    }
}
