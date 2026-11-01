// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Checked Rustdoc fragments, leaving intra-doc URL resolution to Rustdoc.

use super::{DiagramBundle, DiagramError, cargo::Library, documentation::static_fragment};
use crate::{Workplan, digest::Digest, graph::BindingKind};
use purrdf_core::ContentDigest;
use purrdf_lex::allocation::{Memory, OrderedTable, Resident};
use purrdf_rdf::projections::escape_xml_text;
use raf_source::syn;
use raf_source::{
    ExportOptions, ExportedItem, Exports, Item, ItemId, Namespace, SymbolKind, Target, TargetId,
    TargetKind, Workspace,
};
use std::fmt::Write as _;
use std::path::Path;
use syn::ext::IdentExt as _;

/// The exact selected library and documented module receiving a fragment.
#[derive(Debug, Clone, Copy)]
pub(super) struct Context<'a> {
    /// Load-local library target identity, preserving package selection.
    pub(super) target: TargetId,
    /// Full crate-root or publicly exported module path in that library.
    pub(super) module: &'a str,
    /// Capability whose code bindings supply the required API references.
    pub(super) capability: &'a str,
    /// Exact declared binding spellings retained by the assembly request.
    pub(super) required: &'a [String],
    /// Library identity from the actual captured Cargo selection.
    pub(super) library: &'a Library,
    /// Canonical workspace root of that selection.
    pub(super) root: &'a Path,
    /// Native content digest of the captured Cargo metadata.
    pub(super) metadata: &'a str,
}

/// Add checked API references and source identity to one unchanged bundle.
///
/// The returned Markdown is suitable for `#[doc = include_str!(...)]` in the
/// selected crate or public module. Every code binding of the explicit capability
/// is required. A binding must select one indexed item in the selected library;
/// private, hidden, external, conditionally ambiguous and unindexed targets refuse.
/// Public re-exports retain their actual exported spelling. An internal binding
/// with several public aliases must instead name the intended exported path.
///
/// Links use ordinary disambiguated `crate::` intra-doc destinations. Rustdoc
/// chooses their actual HTML files and anchors, including associated items;
/// generation neither predicts those URLs nor depends on private Rustdoc HTML.
/// The native static fragment remains the same script-free presentation.
/// Crate-root identity uses native unframed content SHA-256 over the exact loaded
/// root-file bytes. Indexed modules and API references retain the existing framed
/// symbol SHA-256 identity; these separate digest domains are labelled explicitly.
///
/// # Errors
/// Refuses absent/invalid scope, a target other than a library, missing native
/// Cargo-observed package/version identity, unresolved or ambiguous required bindings,
/// unavailable public export paths and unsupported item kinds. Original native
/// resolution, export, digest and escaping errors retain their error sources.
/// More than 4096 API bindings refuse. A refusal returns no partial fragment.
pub(super) fn fragment(
    plan: &Workplan<'_>,
    source: &Workspace,
    bundle: &DiagramBundle,
    context: Context<'_>,
) -> Result<String, DiagramError> {
    let target = source
        .target(context.target)
        .ok_or_else(|| refused("the selected library target is absent from this source load"))?;
    if target.kind != TargetKind::Lib {
        return Err(refused(
            "select an actual library target for Rustdoc include fragments",
        ));
    }
    // Retain hidden paths so an explicitly hidden alias cannot disappear and
    // accidentally be replaced by a visible alias of the same definition.
    let mut export_options = ExportOptions::default();
    export_options.include_hidden = true;
    let exports = source
        .exported_items_with(context.target, &export_options)
        .map_err(DiagramError::data)?;
    let public = indexed_exports(&exports)?;
    let receiving = resolve_context(source, target, context.target, &public, context.module)?;
    let requested = checked_bindings(plan, context.capability, context.required)?;
    let mut references = String::new();
    let mut count = 0;
    for binding in plan
        .graph()
        .bindings_of(context.capability)
        .filter(|binding| binding.kind() == BindingKind::Code)
    {
        count += 1;
        if count > 4096 {
            return Err(refused(
                "the capability exceeds 4096 required API references",
            ));
        }
        // The document model and compiler-selected index have separate ItemIds.
        // Resolve the authored spelling again in the actual compiler context.
        let resolution = source.resolve(binding.path()).map_err(DiagramError::data)?;
        let [id] = resolution.sites.as_slice() else {
            return Err(refused(&format!(
                "binding `{}` must resolve to exactly one indexed source definition",
                binding.path()
            )));
        };
        let item = source
            .item(*id)
            .ok_or_else(|| refused("a required binding's indexed source item is absent"))?;
        if item.target != context.target {
            return Err(refused(
                "a required binding belongs to another library target",
            ));
        }
        if !item.documented {
            return Err(refused("a required API binding has no documentation"));
        }
        let (canonical, _) = spelling(binding.path())?;
        let exported = exported_path(&public, &canonical, item)?;
        let written = if exported == canonical {
            requested
                .get(&canonical)
                .ok_or_else(|| refused("required binding spelling is absent"))?
                .clone()
        } else {
            exported_spelling(exported)?
        };
        reference(
            &mut references,
            source,
            target,
            item,
            binding.path(),
            exported,
            &written,
        )?;
    }
    if count == 0 {
        return Err(refused(
            "the selected capability has no required code bindings",
        ));
    }
    let mut output = preface(source, target, receiving, &context)?;
    output.push_str(&references);
    compose(bundle, &output)
}

pub(super) fn compose(bundle: &DiagramBundle, body: &str) -> Result<String, DiagramError> {
    let mut output = static_fragment(bundle).map_err(DiagramError::data)?;
    output.push_str("\n\n");
    output.push_str(body);
    Ok(output)
}

fn preface(
    source: &Workspace,
    target: &Target,
    receiving: ResolvedContext<'_>,
    context: &Context<'_>,
) -> Result<String, DiagramError> {
    let package = source
        .package(target.package_id)
        .ok_or_else(|| refused("the selected library's package manifest is absent"))?;
    let library = context.library;
    if library.package != target.package
        || library.name != target.name
        || context
            .root
            .join(package.manifest.as_str())
            .canonicalize()
            .map_err(DiagramError::data)?
            != library
                .manifest
                .canonicalize()
                .map_err(DiagramError::data)?
        || context
            .root
            .join(target.root.as_str())
            .canonicalize()
            .map_err(DiagramError::data)?
            != library.source.canonicalize().map_err(DiagramError::data)?
    {
        return Err(refused(
            "Cargo-observed identity disagrees with the selected source target",
        ));
    }
    let mut output = String::new();
    write!(
        output,
        "API references for <code>{}</code>, Cargo-observed package <code>{}</code>, version <code>{}</code>.\n\n\
         Manifest <code>{}</code>, library root <code>{}</code>.\n\n",
        escaped(context.module)?,
        escaped(&library.package)?,
        escaped(&library.version)?,
        escaped(package.manifest.as_str())?,
        escaped(target.root.as_str())?,
    )
    .expect("writing to String cannot fail");
    match receiving {
        ResolvedContext::CrateRoot { target, text } => {
            let digest = ContentDigest::of(text.as_bytes());
            writeln!(
                output,
                "Crate root <code>{}</code>, file <code>{}</code>, unframed content SHA-256 <code>{digest}</code> over exact loaded bytes.\n",
                escaped(&target.crate_path)?,
                escaped(target.root.as_str())?,
            )
            .expect("writing to String cannot fail");
        }
        ResolvedContext::Module(module) => {
            let digest =
                Digest::symbol(source, module.path.as_str()).map_err(DiagramError::data)?;
            writeln!(
                output,
                "Source module <code>{}</code>, file <code>{}</code>, framed symbol SHA-256 <code>{digest}</code>.\n",
                escaped(module.path.as_str())?,
                escaped(module.file.as_str())?,
            )
            .expect("writing to String cannot fail");
        }
    }
    writeln!(
        output,
        "Cargo metadata content SHA-256 <code>{}</code>.\n",
        escaped(context.metadata)?
    )
    .expect("writing to String cannot fail");
    Ok(output)
}

type ExportIndex<'a> = OrderedTable<ItemId, Vec<&'a ExportedItem>>;

fn indexed_exports(exports: &Exports) -> Result<ExportIndex<'_>, DiagramError> {
    let mut index = ExportIndex::default();
    let mut resident = Resident;
    for export in &exports.items {
        if let Some(paths) = index.get_mut(&export.item) {
            paths.push(export);
        } else {
            index
                .insert(export.item, vec![export], &mut Memory::new(&mut resident))
                .map_err(DiagramError::data)?;
        }
    }
    Ok(index)
}

#[derive(Clone, Copy)]
enum ResolvedContext<'a> {
    CrateRoot { target: &'a Target, text: &'a str },
    Module(&'a Item),
}

fn resolve_context<'a>(
    source: &'a Workspace,
    target: &'a Target,
    selected: TargetId,
    exports: &ExportIndex<'_>,
    path: &str,
) -> Result<ResolvedContext<'a>, DiagramError> {
    let (canonical, written) = spelling(path)?;
    if canonical == target.crate_path {
        let text = source.source(&target.root).ok_or_else(|| {
            refused(&format!(
                "selected library root `{}` has no loaded source text; reload the complete selected target before generating its crate documentation",
                target.root
            ))
        })?;
        return Ok(ResolvedContext::CrateRoot { target, text });
    }
    canonical
        .strip_prefix(target.crate_path.as_str())
        .and_then(|rest| rest.strip_prefix("::"))
        .filter(|rest| !rest.is_empty())
        .ok_or_else(|| {
            refused("the receiving module must name a public path within the selected crate")
        })?;
    let (_, rust_suffix) = written
        .split_once("::")
        .ok_or_else(|| refused("the receiving module has no selected-crate path suffix"))?;
    let local = format!("crate::{rust_suffix}");
    // This native API resolves imports within the selected package, rather
    // than globally qualified export names.
    let resolution = source
        .resolve_from_package_id_in(
            target.package_id,
            &target.crate_path,
            &local,
            Namespace::Type,
        )
        .map_err(DiagramError::data)?;
    let [id] = resolution.sites.as_slice() else {
        return Err(refused(
            "the receiving Rustdoc module must have one indexed definition",
        ));
    };
    let item = source
        .item(*id)
        .ok_or_else(|| refused("the receiving module's indexed definition is absent"))?;
    if item.kind != SymbolKind::Mod || item.target != selected {
        return Err(refused(
            "the receiving Rustdoc context must be a crate root or module",
        ));
    }
    let exported = exported_path(exports, &canonical, item)?;
    if exported != canonical {
        return Err(refused(
            "the receiving module must name its actual public export path",
        ));
    }
    Ok(ResolvedContext::Module(item))
}

fn exported_path<'a>(
    exports: &ExportIndex<'a>,
    requested: &str,
    item: &Item,
) -> Result<&'a str, DiagramError> {
    if item.doc_hidden {
        return Err(refused(
            "required API references must be visible in the selected library",
        ));
    }
    let paths = exports.get(&item.id).ok_or_else(|| {
        refused(&format!(
            "binding `{requested}` has no public documented export; private and hidden API references are refused"
        ))
    })?;
    if let Some(exact) = paths.iter().find(|export| export.path == requested) {
        if exact.doc_hidden {
            return Err(refused(&format!(
                "binding `{requested}` explicitly names a hidden public export; bind a visible exported path instead"
            )));
        }
        return Ok(&exact.path);
    }
    let mut visible = paths.iter().filter(|export| !export.doc_hidden);
    let first = visible.next().ok_or_else(|| {
        refused(&format!(
            "binding `{requested}` has no visible public export; private and hidden API references are refused"
        ))
    })?;
    if visible.next().is_some() {
        return Err(refused(&format!(
            "binding `{requested}` has multiple public aliases; bind the intended exported path explicitly"
        )));
    }
    Ok(&first.path)
}

fn reference(
    output: &mut String,
    source: &Workspace,
    target: &Target,
    item: &Item,
    requested: &str,
    exported: &str,
    written: &str,
) -> Result<(), DiagramError> {
    let suffix = exported.strip_prefix(&target.crate_path).ok_or_else(|| {
        refused("a public export path is outside the selected library's crate root")
    })?;
    if !suffix.is_empty() && !suffix.starts_with("::") {
        return Err(refused(
            "a public export does not follow the selected crate path",
        ));
    }
    // Intra-doc destinations use Rustdoc's canonical names: a raw identifier's
    // `#` is an anchor delimiter there. Keep Rust spelling in the visible label.
    let digest = Digest::symbol(source, requested).map_err(DiagramError::data)?;
    writeln!(
        output,
        "- [<code>{}</code>]({}@crate{suffix}) — definition <code>{}</code>, file <code>{}</code>, line {}, framed symbol SHA-256 <code>{digest}</code>.",
        escaped(written)?,
        disambiguator(item.kind)?,
        escaped(item.path.as_str())?,
        escaped(item.file.as_str())?,
        item.name_line,
    )
    .expect("writing to String cannot fail");
    Ok(())
}

fn disambiguator(kind: SymbolKind) -> Result<&'static str, DiagramError> {
    Ok(match kind {
        SymbolKind::Fn
        | SymbolKind::ForeignFn
        | SymbolKind::InherentMethod
        | SymbolKind::TraitDefaultMethod
        | SymbolKind::TraitRequiredMethod => "fn",
        SymbolKind::Struct => "struct",
        SymbolKind::Enum => "enum",
        SymbolKind::Union => "union",
        SymbolKind::Trait => "trait",
        SymbolKind::TypeAlias | SymbolKind::AssocType => "type",
        SymbolKind::Const | SymbolKind::AssocConst => "const",
        SymbolKind::Static | SymbolKind::ForeignStatic => "static",
        SymbolKind::Mod => "mod",
        SymbolKind::MacroRules => "macro",
        _ => {
            return Err(refused(
                "the required item kind has no supported Rustdoc intra-doc target",
            ));
        }
    })
}

fn escaped(text: &str) -> Result<String, DiagramError> {
    escape_xml_text(text).map_err(DiagramError::data)
}

fn refused(reason: &str) -> DiagramError {
    DiagramError::Refused {
        reason: format!("Rustdoc fragment: {reason}"),
    }
}

pub(super) type ApiNames = OrderedTable<String, String>;

/// Validate modeled item paths without admitting recursive generic syntax.
pub(super) fn spelling(path: &str) -> Result<(String, String), DiagramError> {
    let mut canonical = String::with_capacity(path.len());
    let mut written = String::with_capacity(path.len());
    for segment in path.split("::") {
        let ident = syn::parse_str::<syn::Ident>(segment.trim()).map_err(DiagramError::data)?;
        if !canonical.is_empty() {
            canonical.push_str("::");
            written.push_str("::");
        }
        canonical.push_str(&ident.unraw().to_string());
        written.push_str(&ident.to_string());
    }
    Ok((canonical, written))
}

pub(super) fn api_names<'a>(
    paths: impl IntoIterator<Item = &'a str>,
) -> Result<ApiNames, DiagramError> {
    let mut names = ApiNames::default();
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    for path in paths {
        if names.len() == 4096 {
            return Err(refused(
                "the capability exceeds 4096 required API references",
            ));
        }
        let (canonical, written) = spelling(path)?;
        if names
            .insert(canonical, written, &mut memory)
            .map_err(DiagramError::data)?
            .is_some()
        {
            return Err(refused(
                "duplicate required API spelling after raw-identifier normalization",
            ));
        }
    }
    Ok(names)
}

pub(super) fn checked_bindings(
    plan: &Workplan<'_>,
    capability: &str,
    required: &[String],
) -> Result<ApiNames, DiagramError> {
    let mut caps = plan
        .capabilities()
        .iter()
        .filter(|found| found.value().header.id == capability);
    if caps.next().is_none() || caps.next().is_some() {
        return Err(refused(
            "name exactly one loaded capability for the required API references",
        ));
    }
    let declared = api_names(
        plan.graph()
            .bindings_of(capability)
            .filter(|binding| binding.kind() == BindingKind::Code)
            .map(crate::graph::Binding::path),
    )?;
    let requested = api_names(required.iter().map(String::as_str))?;
    if declared.is_empty()
        || declared.len() != requested.len()
        || declared
            .iter()
            .any(|(name, _)| requested.get(name).is_none())
    {
        return Err(refused(
            "required API paths must match every declared capability code binding",
        ));
    }
    Ok(requested)
}

fn exported_spelling(path: &str) -> Result<String, DiagramError> {
    let mut written = String::with_capacity(path.len());
    for segment in path.split("::") {
        let ident = syn::parse_str::<syn::Ident>(segment)
            .or_else(|_| syn::parse_str::<syn::Ident>(&format!("r#{segment}")))
            .map_err(DiagramError::data)?;
        if !written.is_empty() {
            written.push_str("::");
        }
        written.push_str(&ident.to_string());
    }
    Ok(written)
}
