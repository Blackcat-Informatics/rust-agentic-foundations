// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Declaration, export and written-use evidence over one member index.

use std::collections::{BTreeMap, BTreeSet};

use purrdf_jsonschema::ecma::MatchLimits;
use raf_rdf::{Builder, Graph, TermValue as Term};
use raf_source::{
    CodeResolver, ExportOptions, Item, ItemId, LoadOptions, Namespace, PackageId, RelPath,
    SymbolKind, TargetId, TargetKind, TestRole, Tree, Vis, Workspace,
};

use super::patterns::charge;
use super::patterns::paths::{CrossPackagePaths, UseSite, visibility_resolution, visit};
use crate::{
    Error, REPORT_FORM, ReportError, VisibilityError,
    report::write::{class, num, put, text},
    syntax::path_text,
};

type Consumers = BTreeMap<PackageId, BTreeSet<String>>;

mod demand;
pub use demand::visibility_demand;

struct Library<'w> {
    target: TargetId,
    public: Vec<&'w Item>,
    uses: Vec<(&'w Item, Consumers)>,
}

/// Build the canonical RDF visibility census for a selected Cargo workspace.
///
/// The report uses declared members and the ordinary structural Source index.
/// Normal library dependencies determine the tiers. Written member paths count
/// against their defining library after lexical and indexed resolution.
///
/// # Examples
///
/// ```
/// use raf_rdf::Format;
/// use raf_source::{MemoryTree, RelPath};
///
/// let tree = MemoryTree::from([
///     ("Cargo.toml", "[package]\nname = \"model\"\nversion = \"0.1.0\"\n"),
///     ("src/lib.rs", "pub struct Data;"),
/// ]);
/// let report = raf_census::visibility(&tree, &RelPath::root())?;
/// let turtle = report.to_text(Format::Turtle)?;
/// assert!(turtle.contains("WorkspaceVisibility"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
/// Refuses invalid workspace inputs, ambiguous source resolution, a cyclic
/// normal library graph, and exhausted source, matching or report resources.
pub fn visibility(tree: &dyn Tree, root: &RelPath) -> Result<Graph, Error> {
    let options = LoadOptions::default();
    let workspace = Workspace::load_members(tree, root, &options)?;
    let mut libraries = libraries(&workspace);
    let tiers = dependency_tiers(&workspace, &libraries)?;
    let mut paths = CrossPackagePaths::default();
    scan_paths(&workspace, &mut paths, &mut match_limits(&options))?;
    for (id, consumers) in paths.by_item {
        let item = definition(&workspace, id)?;
        let target = workspace.target(item.target).ok_or_else(|| {
            Error::Options(format!("visibility: {} has no defining target", item.path))
        })?;
        libraries
            .get_mut(&target.package_id)
            .ok_or_else(|| Error::Options(format!("visibility: {} has no library", item.path)))?
            .uses
            .push((item, consumers));
    }
    let mut graph = Builder::new();
    class(&mut graph, "", "WorkspaceVisibility")?;
    num(&mut graph, "", "reportForm", REPORT_FORM as usize)?;
    for (index, (package, tier)) in tiers.into_iter().enumerate() {
        let library = &libraries[&package];
        let subject = format!("#library/{index}");
        library.write(&workspace, package, tier, &mut graph, &subject)?;
        put(&mut graph, "", "library", Term::iri(&subject))?;
    }
    Ok(graph.finish().map_err(ReportError::Ground)?)
}

fn match_limits(options: &LoadOptions) -> MatchLimits {
    MatchLimits {
        steps: u64::try_from(options.max_query_work).unwrap_or(u64::MAX),
        ..MatchLimits::default()
    }
}

fn scan_paths(
    workspace: &Workspace,
    paths: &mut CrossPackagePaths,
    limits: &mut MatchLimits,
) -> Result<(), Error> {
    let code = workspace.code_resolver()?;
    indexed_imports(workspace, &code, paths, limits)?;
    for file in workspace.files() {
        raf_source::parse_source(&file.path, &file.text, |syntax| {
            visit::references(workspace, &code, file, syntax, limits, paths)
        })?
        .map_err(|source| Error::Input {
            file: file.path.to_string(),
            source: Box::new(source),
        })?;
    }
    Ok(())
}

fn indexed_imports(
    workspace: &Workspace,
    code: &CodeResolver<'_>,
    paths: &mut CrossPackagePaths,
    limits: &mut MatchLimits,
) -> Result<(), Error> {
    for import in code.indexed_imports() {
        let (target, file, line) = import.site();
        let consumer = paths.consumer(workspace, target)?;
        let (segments, absolute, glob, terminal_namespaces) = import.path();
        let uncertain = paths.demand.is_some()
            && import.is_conditional().map_err(|source| {
                Error::Visibility(VisibilityError::Path {
                    file: file.to_string(),
                    line,
                    path: path_text(segments, absolute),
                    source: Box::new(source),
                })
            })?;
        let mut bytes = usize::from(absolute) * 2;
        for (index, segment) in segments.iter().enumerate() {
            let length = index + 1;
            bytes = bytes.saturating_add(segment.len()).saturating_add(2);
            // A bare self/super prefix anchors a lookup and names no imported item.
            if matches!(segment.as_str(), "crate" | "self" | "super") {
                continue;
            }
            let namespaces = if length == segments.len() {
                terminal_namespaces
            } else {
                &[Namespace::Type][..]
            };
            charge(
                "visibility",
                limits,
                bytes
                    .saturating_mul(namespaces.len() + 1)
                    .try_into()
                    .unwrap_or(u64::MAX),
            )?;
            let written = path_text(&segments[..length], absolute);
            for &namespace in namespaces {
                match visibility_resolution(
                    workspace,
                    consumer,
                    import.resolve(&written, namespace),
                ) {
                    Ok(Some(resolution)) => {
                        let display = if glob && length == segments.len() {
                            format!("{written}::*")
                        } else {
                            written.clone()
                        };
                        for item in resolution.sites {
                            let site = UseSite {
                                target,
                                test: import.is_test(),
                                uncertain,
                            };
                            paths.record(workspace, site, item, &display, limits)?;
                        }
                    }
                    Ok(None) => {}
                    Err(source) => {
                        return Err(Error::Visibility(VisibilityError::Path {
                            file: file.to_string(),
                            line,
                            path: written,
                            source: Box::new(source),
                        }));
                    }
                }
            }
        }
    }
    Ok(())
}

fn libraries(workspace: &Workspace) -> BTreeMap<PackageId, Library<'_>> {
    let mut libraries: BTreeMap<_, _> = workspace
        .target_ids()
        .zip(workspace.targets())
        .filter(|(_, target)| target.kind == TargetKind::Lib)
        .map(|(id, target)| {
            (
                target.package_id,
                Library {
                    target: id,
                    public: Vec::new(),
                    uses: Vec::new(),
                },
            )
        })
        .collect();
    for item in workspace.items() {
        if item.visibility != Vis::Public
            || item.test != TestRole::None
            || workspace.same_source(item).is_empty()
        {
            continue;
        }
        if let Some(target) = workspace.target(item.target)
            && target.kind == TargetKind::Lib
            && let Some(library) = libraries.get_mut(&target.package_id)
        {
            library.public.push(item);
        }
    }
    libraries
}

fn definition(workspace: &Workspace, id: ItemId) -> Result<&Item, Error> {
    workspace
        .item(id)
        .ok_or_else(|| Error::Options("visibility: resolution names a missing source item".into()))
}

fn package_name(workspace: &Workspace, id: PackageId) -> Result<&str, Error> {
    workspace
        .package(id)
        .map(|package| package.name.as_str())
        .ok_or_else(|| {
            Error::Options("visibility: source graph names a missing workspace package".into())
        })
}

fn exported(workspace: &Workspace, target: TargetId) -> Result<BTreeSet<ItemId>, Error> {
    let target_info = workspace.target(target).ok_or_else(|| {
        Error::Options("visibility: report names a missing library target".into())
    })?;
    let manifest = workspace.package(target_info.package_id).ok_or_else(|| {
        Error::Options("visibility: report names a missing workspace package".into())
    })?;
    let mut options = ExportOptions::default();
    options.include_hidden = true;
    let exports = workspace
        .exported_items_with(target, &options)
        .map_err(|source| {
            Error::Visibility(VisibilityError::Export {
                manifest: manifest.manifest.to_string(),
                source: Box::new(source),
            })
        })?;
    Ok(exports.items.into_iter().map(|row| row.item).collect())
}

fn dependency_tiers(
    workspace: &Workspace,
    libraries: &BTreeMap<PackageId, Library<'_>>,
) -> Result<Vec<(PackageId, usize)>, Error> {
    let mut waiting = BTreeMap::new();
    let mut dependents: BTreeMap<_, Vec<_>> = BTreeMap::new();
    let mut ready = BTreeSet::new();
    for (&package, library) in libraries {
        let dependencies: BTreeSet<_> = workspace
            .library_prelude_dependencies(library.target)
            .ok_or_else(|| Error::Options("visibility: missing library prelude".into()))?
            .collect();
        if dependencies.is_empty() {
            ready.insert((0, package));
        }
        waiting.insert(package, (dependencies.len(), 0));
        for dependency in dependencies {
            dependents.entry(dependency).or_default().push(package);
        }
    }
    let mut result = Vec::new();
    while let Some((tier, package)) = ready.pop_first() {
        result.push((package, tier));
        for dependent in dependents.remove(&package).into_iter().flatten() {
            let (missing, next_tier) = waiting.get_mut(&dependent).ok_or_else(|| {
                Error::Options("visibility: library dependency has no consumer".into())
            })?;
            *missing -= 1;
            *next_tier = (*next_tier).max(tier + 1);
            if *missing == 0 {
                ready.insert((*next_tier, dependent));
            }
        }
    }
    if result.len() != libraries.len() {
        let unresolved = waiting
            .into_iter()
            .filter(|(_, (edges, _))| *edges != 0)
            .map(|(id, _)| package_name(workspace, id))
            .collect::<Result<Vec<_>, _>>()?;
        return Err(Error::Options(format!(
            "visibility: normal library dependency cycle among {}; resolve the cycle before computing tiers",
            unresolved.join(", ")
        )));
    }
    Ok(result)
}

impl Library<'_> {
    fn write(
        &self,
        workspace: &Workspace,
        package: PackageId,
        tier: usize,
        graph: &mut Builder,
        id: &str,
    ) -> Result<(), Error> {
        let manifest = workspace.package(package).ok_or_else(|| {
            Error::Options("visibility: report names a missing workspace package".into())
        })?;
        let exported = exported(workspace, self.target)?;
        let used: BTreeSet<_> = self.uses.iter().map(|(item, _)| item.id).collect();
        let written: BTreeSet<_> = self
            .uses
            .iter()
            .flat_map(|(_, consumers)| consumers.values().flatten())
            .collect();

        class(graph, id, "LibraryVisibility")?;
        text(graph, id, "package", &manifest.name)?;
        num(graph, id, "tier", tier)?;
        num(graph, id, "publicCount", self.public.len())?;
        num(graph, id, "exportedCount", exported.len())?;
        num(
            graph,
            id,
            "usedExportedCount",
            used.intersection(&exported).count(),
        )?;
        num(graph, id, "writtenPathCount", written.len())?;
        let mut kinds = BTreeMap::new();
        for (index, item) in self.public.iter().enumerate() {
            let declaration = format!("{id}/declaration/{index}");
            let kind = kind_name(item.kind)?;
            *kinds.entry(kind).or_insert(0) += 1;
            class(graph, &declaration, "PublicDeclaration")?;
            write_item(graph, &declaration, item)?;
            text(graph, &declaration, "kind", kind)?;
            put(graph, id, "declaration", Term::iri(&declaration))?;
            if !used.contains(&item.id) {
                put(graph, id, "unusedDeclaration", Term::iri(&declaration))?;
            }
        }
        for (kind, count) in kinds {
            let group = format!("{id}/kind/{kind}");
            class(graph, &group, "VisibilityKindCount")?;
            text(graph, &group, "kind", kind)?;
            num(graph, &group, "count", count)?;
            put(graph, id, "kindCount", Term::iri(&group))?;
        }
        for (index, (item, consumers)) in self.uses.iter().enumerate() {
            for (position, (&consumer, paths)) in consumers.iter().enumerate() {
                let usage = format!("{id}/use/{index}/{position}");
                class(graph, &usage, "LibraryUse")?;
                write_item(graph, &usage, item)?;
                text(
                    graph,
                    &usage,
                    "consumer",
                    package_name(workspace, consumer)?,
                )?;
                for path in paths {
                    text(graph, &usage, "writtenPath", path)?;
                }
                put(graph, id, "use", Term::iri(&usage))?;
            }
        }
        Ok(())
    }
}

fn write_item(graph: &mut Builder, id: &str, item: &Item) -> Result<(), ReportError> {
    text(graph, id, "symbol", item.path.as_str())?;
    text(graph, id, "file", &item.file.to_string())?;
    num(graph, id, "line", item.name_line)?;
    num(graph, id, "startByte", item.span.start_byte)
}

fn kind_name(kind: SymbolKind) -> Result<&'static str, Error> {
    Ok(match kind {
        SymbolKind::Fn => "function",
        SymbolKind::Struct => "struct",
        SymbolKind::Enum => "enum",
        SymbolKind::Union => "union",
        SymbolKind::Trait => "trait",
        SymbolKind::TypeAlias => "type-alias",
        SymbolKind::Const => "constant",
        SymbolKind::Static => "static",
        SymbolKind::Mod => "module",
        SymbolKind::MacroRules => "macro-rules",
        SymbolKind::ExternCrate => "extern-crate",
        SymbolKind::ForeignFn => "foreign-function",
        SymbolKind::ForeignStatic => "foreign-static",
        SymbolKind::ForeignType => "foreign-type",
        SymbolKind::InherentMethod => "inherent-method",
        SymbolKind::TraitImplMethod => "trait-impl-method",
        SymbolKind::TraitDefaultMethod => "trait-default-method",
        SymbolKind::TraitRequiredMethod => "trait-required-method",
        SymbolKind::AssocConst => "associated-constant",
        SymbolKind::AssocType => "associated-type",
        _ => {
            return Err(Error::Options(format!(
                "visibility: unsupported public item kind {kind:?}"
            )));
        }
    })
}
