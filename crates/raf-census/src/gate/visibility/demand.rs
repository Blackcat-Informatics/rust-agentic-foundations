// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Target-level evidence for public workspace library definitions.

use std::collections::{BTreeMap, BTreeSet};

use purrdf_jsonschema::ecma::MatchLimits;
use raf_rdf::{Builder, Graph, TermValue as Term};
use raf_source::{
    Item, LoadOptions, PackageId, RelPath, SymbolKind, Target, TargetId, TargetKind, TestRole,
    Tree, Workspace,
};

use super::{
    CrossPackagePaths, UseSite, definition, exported, match_limits, scan_paths, write_item,
};
use crate::gate::patterns::{charge, failure, paths::DemandPaths};
use crate::{
    Error, REPORT_FORM, ReportError,
    report::write::{class, num, put, text},
};

const LABEL: &str = "visibility-demand";

/// Report written cross-target demand for effective public workspace items.
/// Observations identify the consuming Cargo package, target and use context.
/// The report distinguishes observed demand from unresolved macro and method
/// coverage in the declared member source index.
///
/// # Errors
/// Refuses invalid workspace inputs, ambiguous path resolution and exhausted
/// source, matching or report resources.
pub fn visibility_demand(tree: &dyn Tree, root: &RelPath) -> Result<Graph, Error> {
    let options = LoadOptions::default();
    let workspace = Workspace::load_members(tree, root, &options)?;
    let mut limits = match_limits(&options);
    let mut paths = CrossPackagePaths::demand();
    scan_paths(&workspace, &mut paths, &mut limits)?;
    let retained = paths.retained();
    let demand = paths
        .demand
        .ok_or_else(|| failure(LABEL, "missing target evidence"))?;
    let coverage = Coverage::new(&workspace, &demand, &mut limits, retained)?;
    let mut graph = Builder::new();
    class(&mut graph, "", "WorkspaceVisibilityDemand")?;
    num(&mut graph, "", "reportForm", REPORT_FORM as usize)?;
    let mut index = 0;
    for (target, owner) in workspace.target_ids().zip(workspace.targets()) {
        if owner.kind != TargetKind::Lib {
            continue;
        }
        for id in exported(&workspace, target)? {
            let item = definition(&workspace, id)?;
            if item.target != target
                || item.test != TestRole::None
                || workspace.same_source(item).is_empty()
            {
                continue;
            }
            let subject = format!("#declaration/{index}");
            Declaration {
                workspace: &workspace,
                demand: &demand,
                coverage: &coverage,
            }
            .write(&mut graph, &subject, item, owner, &mut limits)?;
            put(&mut graph, "", "declaration", Term::iri(&subject))?;
            index += 1;
        }
    }
    Ok(graph.finish().map_err(ReportError::Ground)?)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Context {
    Unknown,
    Other,
    Test,
    Production,
}

impl Context {
    fn of(target: &Target, site: UseSite) -> Self {
        if matches!(target.kind, TargetKind::Example | TargetKind::Bench) {
            Self::Other
        } else if target.kind == TargetKind::Test || site.test {
            Self::Test
        } else if site.uncertain {
            Self::Unknown
        } else if target.shipping {
            Self::Production
        } else {
            Self::Unknown
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Production => "production",
            Self::Test => "test",
            Self::Other => "other-nonshipping",
            Self::Unknown => "unknown",
        }
    }
}

struct Declaration<'a> {
    workspace: &'a Workspace,
    demand: &'a DemandPaths,
    coverage: &'a Coverage,
}

impl Declaration<'_> {
    fn write(
        &self,
        graph: &mut Builder,
        subject: &str,
        item: &Item,
        owner: &Target,
        limits: &mut MatchLimits,
    ) -> Result<(), Error> {
        class(graph, subject, "PublicDemand")?;
        write_item(graph, subject, item)?;
        text(graph, subject, "kind", super::kind_name(item.kind)?)?;
        text(graph, subject, "package", &owner.package)?;
        text(graph, subject, "target", &owner.name)?;

        let mut contexts = BTreeSet::new();
        let mut uncertain = false;
        if let Some(uses) = self.demand.by_item.get(&item.id) {
            for (index, (&site, paths)) in uses.iter().enumerate() {
                let target = self
                    .workspace
                    .target(site.target)
                    .ok_or_else(|| failure(LABEL, "evidence names a missing consuming target"))?;
                let context = Context::of(target, site);
                uncertain |= site.uncertain || context == Context::Unknown;
                contexts.insert(context);
                write_use(graph, subject, index, target, context, paths)?;
            }
        }
        let demand = if contexts.contains(&Context::Production) {
            "production"
        } else if contexts.contains(&Context::Other) {
            "other-nonshipping"
        } else if contexts.contains(&Context::Test) {
            "test-only"
        } else {
            "no-known-use"
        };
        text(graph, subject, "demand", demand)?;
        for reason in self
            .coverage
            .reasons(self.demand, item, owner, limits)?
            .into_iter()
            .chain(uncertain.then_some("uncertain-source-context"))
        {
            text(graph, subject, "coverageGap", reason)?;
        }
        Ok(())
    }
}

fn write_use(
    graph: &mut Builder,
    declaration: &str,
    index: usize,
    target: &Target,
    context: Context,
    paths: &BTreeSet<String>,
) -> Result<(), Error> {
    let subject = format!("{declaration}/use/{index}");
    class(graph, &subject, "TargetUse")?;
    text(graph, &subject, "consumer", &target.package)?;
    text(graph, &subject, "consumerTarget", &target.name)?;
    let kind =
        crate::embedded::target_kind(target.kind).map_err(|reason| failure(LABEL, reason))?;
    text(graph, &subject, "consumerTargetKind", kind)?;
    text(graph, &subject, "context", context.as_str())?;
    for path in paths {
        text(graph, &subject, "writtenPath", path)?;
    }
    put(graph, declaration, "use", Term::iri(&subject))?;
    Ok(())
}

struct Coverage(BTreeMap<TargetId, BTreeSet<PackageId>>);

impl Coverage {
    fn new(
        workspace: &Workspace,
        demand: &DemandPaths,
        limits: &mut MatchLimits,
        mut retained: usize,
    ) -> Result<Self, Error> {
        let targets: BTreeSet<_> = demand
            .macros
            .iter()
            .chain(demand.methods.values().flatten())
            .copied()
            .collect();
        let libraries: BTreeMap<_, _> = workspace
            .target_ids()
            .zip(workspace.targets())
            .filter(|(_, target)| target.kind == TargetKind::Lib)
            .map(|(id, target)| (target.package_id, id))
            .collect();
        let mut scopes = BTreeMap::new();
        for target in targets {
            let mut packages = BTreeSet::new();
            let mut pending = vec![target];
            while let Some(target) = pending.pop() {
                let info = workspace
                    .target(target)
                    .ok_or_else(|| failure(LABEL, "coverage names a missing target"))?;
                include_package(&mut packages, info.package_id, &mut retained, limits)?;
                let candidates = workspace
                    .target_prelude_dependencies(target, true)
                    .ok_or_else(|| failure(LABEL, "coverage target has no extern prelude"))?;
                for package in candidates {
                    charge(LABEL, limits, 1)?;
                    if include_package(&mut packages, package, &mut retained, limits)?
                        && let Some(&library) = libraries.get(&package)
                    {
                        pending.push(library);
                    }
                }
            }
            scopes.insert(target, packages);
        }
        Ok(Self(scopes))
    }

    fn reaches(
        &self,
        item: &Item,
        owner: &Target,
        targets: &BTreeSet<TargetId>,
        limits: &mut MatchLimits,
    ) -> Result<bool, Error> {
        for target in targets {
            charge(LABEL, limits, 1)?;
            if *target != item.target && self.0[target].contains(&owner.package_id) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn reasons(
        &self,
        demand: &DemandPaths,
        item: &Item,
        owner: &Target,
        limits: &mut MatchLimits,
    ) -> Result<Vec<&'static str>, Error> {
        let mut gaps = Vec::new();
        if self.reaches(item, owner, &demand.macros, limits)? {
            gaps.push("macro-expansion");
        }
        if matches!(
            item.kind,
            SymbolKind::InherentMethod
                | SymbolKind::TraitDefaultMethod
                | SymbolKind::TraitRequiredMethod
        ) && let Some(targets) = demand.methods.get(item.name.as_str())
            && self.reaches(item, owner, targets, limits)?
        {
            gaps.push("implicit-method-resolution");
        }
        Ok(gaps)
    }
}

fn include_package(
    packages: &mut BTreeSet<PackageId>,
    package: PackageId,
    retained: &mut usize,
    limits: &MatchLimits,
) -> Result<bool, Error> {
    if packages.contains(&package) {
        return Ok(false);
    }
    if *retained >= limits.states {
        return Err(failure(LABEL, "coverage state limit exhausted"));
    }
    packages.insert(package);
    *retained += 1;
    Ok(true)
}
