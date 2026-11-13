// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Join actual Source module declarations and body files into prelude policy.

use raf_source::{Item, RelPath, SourceFile, SymbolKind, TargetId, Workspace};

use super::analysis::{Analysis, SourceFailure};
use super::collect::Failure;
use super::names;
use super::native_spans;
use super::prelude::Prelude;
use super::resource::{Meter, Refusal};
use super::scopes::Scopes;

fn located(file: &RelPath, byte: usize, cause: Refusal) -> SourceFailure<'_> {
    SourceFailure::Located {
        file,
        cause: Failure { byte, cause },
    }
}

/// Every actual reach is evaluated independently. Shared source must agree on
/// its complete declared profile; a disabled prelude is never pooled by OR.
pub(super) fn for_file<'source>(
    mut analysis: Analysis<'source>,
    workspace: &'source Workspace,
    file: &'source SourceFile,
) -> Result<(Analysis<'source>, Prelude), SourceFailure<'source>> {
    let declared = Prelude::declared_for_file(workspace, file, analysis.meter())
        .map_err(|cause| located(&file.path, 0, cause))?;
    let mut selected = None;
    for reach in &file.reached {
        analysis
            .meter()
            .work(1)
            .map_err(|cause| located(&file.path, 0, cause))?;
        if reach.predicate.is_some() {
            return Err(located(
                &file.path,
                0,
                Refusal::Unsupported("a conditional source reach needs admitted active selection"),
            ));
        }
        let target = workspace.target(reach.target).ok_or_else(|| {
            located(
                &file.path,
                0,
                Refusal::Invariant("source reach lost its target"),
            )
        })?;
        let mut current = root_file(
            workspace,
            reach.target,
            &target.crate_path,
            analysis.meter(),
            &file.path,
        )?;
        let mut profile;
        let mut file_module = target.crate_path.as_str();
        (analysis, profile) = inspect(analysis, current, None, declared, file_module, true)?;
        let path = reach.module_path.as_str();
        let suffix = reach_suffix(path, &target.crate_path, analysis.meter())
            .map_err(|cause| located(&file.path, 0, cause))?;
        let mut offset = path.len() - suffix.len();
        while offset < path.len() {
            let tail = &path[offset + 2..];
            analysis
                .meter()
                .work(tail.len())
                .map_err(|cause| located(&file.path, 0, cause))?;
            let width = tail.find("::").unwrap_or(tail.len());
            if width == 0 {
                return Err(located(
                    &file.path,
                    0,
                    Refusal::Invariant("source reach has an empty module segment"),
                ));
            }
            offset += width + 2;
            let prefix = &path[..offset];
            let module = declaration(workspace, reach.target, prefix, analysis.meter())
                .map_err(|cause| located(&file.path, 0, cause))?;
            if module.file != current.path {
                return Err(located(
                    &module.file,
                    module.span.start_byte,
                    Refusal::Invariant(
                        "module declaration does not belong to its actual parent source",
                    ),
                ));
            }
            (analysis, profile) =
                inspect(analysis, current, Some(module), profile, file_module, false)?;
            if module.body.is_none() {
                current = body_file(workspace, reach.target, prefix, analysis.meter())
                    .map_err(|cause| located(&module.file, module.span.start_byte, cause))?;
                file_module = prefix;
                (analysis, profile) =
                    inspect(analysis, current, None, profile, file_module, false)?;
            }
        }
        if !std::ptr::eq(current, file) {
            return Err(located(
                &file.path,
                0,
                Refusal::Invariant("module ancestry ended at a different actual source file"),
            ));
        }
        if selected.is_some_and(|known| known != profile) {
            return Err(located(
                &file.path,
                0,
                Refusal::Unsupported("source reaches disagree on their inherited prelude policy"),
            ));
        }
        selected = Some(profile);
    }
    let profile = selected.ok_or_else(|| {
        located(
            &file.path,
            0,
            Refusal::Invariant("source file has no actual reach"),
        )
    })?;
    Ok((analysis, profile))
}

fn reach_suffix<'path>(path: &'path str, root: &str, meter: &Meter) -> Result<&'path str, Refusal> {
    meter.work(path.len().max(root.len()))?;
    path.strip_prefix(root)
        .filter(|suffix| suffix.is_empty() || suffix.starts_with("::"))
        .ok_or(Refusal::Invariant(
            "source reach is outside its target root",
        ))
}

fn root_file<'source>(
    workspace: &'source Workspace,
    target: TargetId,
    path: &str,
    meter: &Meter,
    file: &'source RelPath,
) -> Result<&'source SourceFile, SourceFailure<'source>> {
    let root =
        declaration(workspace, target, path, meter).map_err(|cause| located(file, 0, cause))?;
    let current = workspace.file(&root.file).ok_or_else(|| {
        located(
            &root.file,
            root.span.start_byte,
            Refusal::Invariant("root declaration has no loaded source"),
        )
    })?;
    let root_body =
        body_file(workspace, target, path, meter).map_err(|cause| located(&root.file, 0, cause))?;
    if !std::ptr::eq(current, root_body) {
        return Err(located(
            &root.file,
            0,
            Refusal::Invariant("root declaration and actual reach disagree"),
        ));
    }
    Ok(current)
}

fn declaration<'source>(
    workspace: &'source Workspace,
    target: TargetId,
    path: &str,
    meter: &Meter,
) -> Result<&'source Item, Refusal> {
    let mut selected = None;
    for item in workspace.items() {
        meter.work(1)?;
        if item.target != target || item.kind != SymbolKind::Mod {
            continue;
        }
        meter.work(path.len().max(item.path.len()))?;
        if item.path.as_str() == path {
            if selected.is_some() {
                return Err(Refusal::Unsupported(
                    "a module has multiple actual declarations; select an unambiguous context",
                ));
            }
            selected = Some(item);
        }
    }
    selected.ok_or(Refusal::Invariant(
        "an actual module reach has no indexed declaration",
    ))
}

fn body_file<'source>(
    workspace: &'source Workspace,
    target: TargetId,
    path: &str,
    meter: &Meter,
) -> Result<&'source SourceFile, Refusal> {
    let mut selected: Option<&'source SourceFile> = None;
    for file in workspace.files() {
        meter.work(1)?;
        for reach in &file.reached {
            meter.work(1)?;
            if reach.target != target {
                continue;
            }
            meter.work(path.len().max(reach.module_path.len()))?;
            if reach.module_path.as_str() != path {
                continue;
            }
            if reach.predicate.is_some() {
                return Err(Refusal::Unsupported(
                    "a module body has conditional source identity",
                ));
            }
            if selected.is_some_and(|known| !std::ptr::eq(known, file)) {
                return Err(Refusal::Unsupported(
                    "a module has multiple actual body files",
                ));
            }
            selected = Some(file);
        }
    }
    selected.ok_or(Refusal::Invariant(
        "an outline module has no actual loaded body file",
    ))
}

fn inspect<'source>(
    analysis: Analysis<'source>,
    file: &'source SourceFile,
    module: Option<&'source Item>,
    profile: Prelude,
    file_module: &'source str,
    crate_root: bool,
) -> Result<(Analysis<'source>, Prelude), SourceFailure<'source>> {
    analysis.inspect_source(file, move |analysis, syntax| {
        let byte = module.map_or(0, |module| module.span.start_byte);
        let coordinates = super::coordinates::Coordinates::new(
            &file.text,
            syntax.shebang.as_deref(),
            analysis.meter(),
        )
        .map_err(|cause| Failure { byte, cause })?;
        let mut scopes = Scopes::with_prelude(&file.text, analysis.meter(), profile, &coordinates);
        let result = (|| {
            if let Some(item) = module {
                let module = syntax_module(
                    syntax,
                    &file.text,
                    item,
                    file_module,
                    analysis.meter(),
                    &coordinates,
                )?;
                scopes.enter_declared_module(module)?;
            } else {
                scopes.enter_module(&syntax.items, &syntax.attrs, crate_root)?;
            }
            let (disabled, no_std) = scopes.descendant_prelude()?;
            Ok(profile.with_flags(disabled, no_std))
        })();
        drop(scopes);
        drop(coordinates);
        let profile = result.map_err(|cause| Failure { byte, cause })?;
        Ok(profile)
    })
}

fn syntax_module<'syntax>(
    syntax: &'syntax syn::File,
    source: &str,
    item: &Item,
    file_module: &str,
    meter: &Meter,
    coordinates: &super::coordinates::Coordinates<'_, '_>,
) -> Result<&'syntax syn::ItemMod, Refusal> {
    meter.work(item.path.len().max(file_module.len()))?;
    // Source's canonical path is the declared module ancestry. Independently
    // follow that exact lexical nesting in the original AST, then require the
    // exact indexed span. No allocating resolver or filename inference enters.
    let suffix = item
        .path
        .strip_prefix(file_module)
        .and_then(|tail| tail.strip_prefix("::"))
        .ok_or(Refusal::Invariant(
            "module declaration is outside its actual file module",
        ))?;
    let mut pieces = suffix.split("::").peekable();
    let mut items = syntax.items.as_slice();
    while let Some(piece) = pieces.next() {
        let mut found = None;
        for node in items {
            meter.work(1)?;
            let syn::Item::Mod(module) = node else {
                continue;
            };
            let range = coordinates.range(module.ident.span())?;
            let name = source
                .get(range)
                .ok_or(Refusal::Invariant("module name has no original UTF-8 span"))?;
            if names::same(name.strip_prefix("r#").unwrap_or(name), piece, meter)? {
                if found.is_some() {
                    return Err(Refusal::Unsupported(
                        "the original module context has multiple declarations of one name",
                    ));
                }
                found = Some(module);
            }
        }
        let module = found.ok_or(Refusal::Invariant(
            "indexed module ancestry has no actual parsed declaration",
        ))?;
        if pieces.peek().is_none() {
            let (first, last) = native_spans::module(module, meter)?;
            let range = coordinates.between(first, last)?;
            if range.start != item.span.start_byte || range.end != item.span.end_byte {
                return Err(Refusal::Invariant(
                    "parsed module and indexed declaration spans disagree",
                ));
            }
            return Ok(module);
        }
        items = module
            .content
            .as_ref()
            .map(|(_, items)| items.as_slice())
            .ok_or(Refusal::Invariant(
                "lexical module ancestry crosses an outline declaration",
            ))?;
    }
    Err(Refusal::Invariant(
        "module declaration has an empty relative path",
    ))
}

#[cfg(test)]
#[path = "context_tests.rs"]
mod tests;
