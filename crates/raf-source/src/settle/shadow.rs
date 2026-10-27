// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Actual outward cuts retain a named shadow that hypothetical absence omits.

use super::{
    BTreeMap, BTreeSet, Fail, ImportReadState, Index, ItemId, LoadError, Meter, Namespace,
    ShadowCut, Source, Steps, UseKind, btree_entry, btree_map, read_components, slot,
    storage_failure,
};

#[derive(Default)]
struct Routes {
    keys: BTreeMap<(usize, usize, usize), BTreeMap<String, usize>>,
    nodes: Vec<(Source, usize)>,
    edges: Vec<Vec<usize>>,
}

impl Routes {
    fn node(
        &mut self,
        source: &Source,
        viewer: usize,
        steps: Steps<'_>,
    ) -> Result<usize, LoadError> {
        let key = (source.module, slot(source.ns), viewer);
        steps.storage_work(1)?;
        if let Some(node) = self
            .keys
            .get(&key)
            .and_then(|names| names.get(&source.name))
        {
            return Ok(*node);
        }
        steps.storage_work(1)?;
        if !self.keys.contains_key(&key) {
            steps.reserve(
                btree_entry::<(usize, usize, usize), BTreeMap<String, usize>>()
                    + btree_map::<String, usize>(),
            )?;
        }
        steps.reserve(btree_entry::<String, usize>() + 2 * source.name.len())?;
        steps.storage_work(2 * source.name.len())?;
        crate::glob_scope::reserve_next(&mut self.nodes, steps)?;
        crate::glob_scope::reserve_next(&mut self.edges, steps)?;
        let node = self.nodes.len();
        self.nodes.push((source.clone(), viewer));
        self.edges.push(Vec::new());
        self.keys
            .entry(key)
            .or_default()
            .insert(source.name.clone(), node);
        Ok(node)
    }
}

impl Index {
    /// Marked cuts preserve their real shadow and repeat the actual proof.
    #[cfg(debug_assertions)]
    pub(super) fn check_preserved_shadows(&self) {
        for (position, states) in self.use_read_state.iter().enumerate() {
            for (at, state) in states.iter().enumerate() {
                if *state != ImportReadState::Shadowed {
                    continue;
                }
                assert!(self.use_settled[position] && self.use_decided[position][at]);
                assert!(
                    !self.use_bound[position][at]
                        && !self.use_empty[position][at]
                        && !self.use_free[position][at]
                );
                let meter = Meter::new(&crate::options::LoadOptions::default());
                assert!(
                    self.preserved_shadow(position, at, &meter, Steps::Load)
                        .expect("a resident actual-cut proof fits its load budget")
                );
            }
        }
    }

    /// A bound peer may finish a missing namespace behind an actual private
    /// cut, while its own named shadow remains in place.
    pub(super) fn preserved_shadow(
        &self,
        position: usize,
        at: usize,
        meter: &Meter,
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        if at == slot(Namespace::Macro)
            || self.use_bound[position][at]
            || self.use_free[position][at]
            || self.use_empty[position][at]
            || !self.use_bound[position].iter().any(|bound| *bound)
            || (0..3).any(|other| other != at && !self.use_decided[position][other])
        {
            return Ok(false);
        }
        let memory = steps.temporary();
        let steps = memory.as_ref().map_or(steps, |memory| steps.at(memory));
        let decl = &self.uses[position];
        let UseKind::Named { segs, .. } = &decl.kind else {
            return Ok(false);
        };
        let Some(last) = segs.last() else {
            return Ok(false);
        };
        let (walks, _, _) = self.import_paths(position, decl, segs, steps)?;
        let sources = walks.into_iter().find_map(|(ns, walk)| {
            if slot(ns) == at
                && let Err(Fail::Missing(sources)) = walk
            {
                Some(sources)
            } else {
                None
            }
        });
        let Some(sources) = sources.filter(|sources| {
            !sources.is_empty()
                && sources
                    .iter()
                    .all(|source| slot(source.ns) == at && source.name == *last)
        }) else {
            return Ok(false);
        };
        self.actual_shadow_routes(position, at, &sources, meter, steps)
    }

    fn actual_shadow_routes(
        &self,
        position: usize,
        at: usize,
        sources: &[Source],
        meter: &Meter,
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        steps.reserve(btree_map::<(usize, usize, usize), BTreeMap<String, usize>>())?;
        let mut routes = Routes::default();
        let mut cuts = Vec::new();
        for source in sources {
            routes.node(source, self.uses[position].module, steps)?;
        }
        steps.reserve(btree_map::<usize, ()>() + btree_entry::<usize, ()>())?;
        let inside = BTreeSet::from([position]);
        let mut next = 0;
        while next < routes.nodes.len() {
            let memory = steps.temporary();
            let reads = memory.as_ref().map_or(steps, |memory| steps.at(memory));
            reads.reserve(routes.nodes[next].0.name.len())?;
            reads.storage_work(1 + routes.nodes[next].0.name.len())?;
            let (source, viewer) = routes.nodes[next].clone();
            if let Some(id) = self.actual_shadow_cut(&source, viewer, reads)? {
                if !self.fallback_reads_closed(position, at, Some(&source), reads)? {
                    return Ok(false);
                }
                crate::glob_scope::reserve_next(&mut cuts, steps)?;
                steps.reserve(source.name.len())?;
                cuts.push(ShadowCut { source, id });
            } else {
                let outward = self
                    .read_lookup(
                        source.module,
                        source.ns,
                        &source.name,
                        None,
                        Some(viewer),
                        reads,
                    )
                    .map_err(storage_failure)?;
                if outward.as_ref().is_some() {
                    return Ok(false);
                }
                if self.actual_shadow_blocked(&source, &inside, reads)? {
                    return Ok(false);
                }
                let targets = self.shadow_targets(&source, viewer, reads)?;
                let Some(targets) = targets else {
                    return Ok(false);
                };
                reads.reserve(source.name.len())?;
                reads.storage_work(source.name.len())?;
                if !self.may_bind_without(
                    (source.module, source.name.clone(), at),
                    viewer,
                    &inside,
                    reads,
                )? {
                    next += 1;
                    continue;
                }
                for target in targets {
                    reads.reserve(source.name.len())?;
                    reads.storage_work(source.name.len())?;
                    let target_source = Source {
                        module: target,
                        ns: source.ns,
                        name: source.name.clone(),
                    };
                    let node = routes.node(&target_source, source.module, steps)?;
                    crate::glob_scope::reserve_next(&mut routes.edges[next], steps)?;
                    routes.edges[next].push(node);
                }
                if routes.edges[next].is_empty() {
                    return Ok(false);
                }
            }
            next += 1;
        }
        for component in read_components(&routes.edges, steps)? {
            steps.storage_work(1 + component.len())?;
            if component.len() > 1 {
                return Ok(false);
            }
            if let Some(node) = component.first() {
                steps.storage_work(routes.edges[*node].len())?;
                if routes.edges[*node].contains(node) {
                    return Ok(false);
                }
            }
        }
        self.grounded_shadow_cuts(&cuts, meter)
    }

    fn actual_shadow_cut(
        &self,
        source: &Source,
        viewer: usize,
        steps: Steps<'_>,
    ) -> Result<Option<ItemId>, LoadError> {
        let local = self
            .read_lookup(source.module, source.ns, &source.name, None, None, steps)
            .map_err(storage_failure)?;
        let Some((bound, true)) = local.as_ref().filter(|(bound, _)| bound.len() == 1) else {
            return Ok(None);
        };
        let (id, origin) = bound.iter().next().expect("unique actual local candidate");
        if origin.companion
            || self
                .origin_visible(origin, source.module, viewer, steps)
                .map_err(storage_failure)?
        {
            return Ok(None);
        }
        let outward = self
            .read_lookup(
                source.module,
                source.ns,
                &source.name,
                None,
                Some(viewer),
                steps,
            )
            .map_err(storage_failure)?;
        Ok(outward.as_ref().is_none().then_some(*id))
    }

    fn actual_shadow_blocked(
        &self,
        source: &Source,
        inside: &BTreeSet<usize>,
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        for &other in self.imports_of(source.module, &source.name) {
            steps.storage_work(1)?;
            let at = slot(source.ns);
            if inside.contains(&other) && !self.use_free[other][at] {
                return Ok(true);
            }
            if !self.use_decided[other][at] {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn shadow_targets(
        &self,
        source: &Source,
        viewer: usize,
        steps: Steps<'_>,
    ) -> Result<Option<Vec<usize>>, LoadError> {
        let mut targets = Vec::new();
        for &glob in self.globs_of(source.module) {
            steps.storage_work(1)?;
            if source.module != viewer
                && !self
                    .read_visibility(&self.uses[glob].vis, source.module, viewer, steps)
                    .map_err(storage_failure)?
            {
                continue;
            }
            if !self.glob_closed[glob] {
                return Ok(None);
            }
            let found = match self.glob_targets(glob, steps) {
                Ok(found) => found,
                Err(Fail::Storage(error)) => return Err(error),
                Err(_) => return Ok(None),
            };
            for target in found {
                steps.storage_work(1)?;
                crate::glob_scope::reserve_next(&mut targets, steps)?;
                targets.push(target);
            }
        }
        Ok(Some(targets))
    }

    /// A marked shadow is a real blocker unless that import is omitted.
    pub(super) fn has_preserved_shadow(
        &self,
        module: usize,
        name: &str,
        at: usize,
        inside: &BTreeSet<usize>,
        steps: Steps<'_>,
    ) -> Result<bool, LoadError> {
        for position in self.imports_of(module, name) {
            steps.storage_work(1)?;
            if !inside.contains(position)
                && self.use_read_state[*position][at] == ImportReadState::Shadowed
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
