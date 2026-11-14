// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Source-owned function collection. Every accepted physical body keeps all
//! of its actual indexed reaches, independently of its presentation symbol.

use raf_source::{Item, SourceFile, SymbolKind, Workspace};
use syn::punctuated::Pair;

use super::analysis::{Analysis, Function, Origin, SourceFailure};
use super::attributes;
use super::collect::{self, Failure};
use super::context;
use super::coordinates::Coordinates;
use super::names;
use super::native_spans;
use super::resource::{Meter, Refusal};
use super::scopes::Scopes;
use super::tokens;

#[derive(Clone, Copy)]
enum Kind {
    Free,
    Inherent,
    TraitImpl,
    TraitDefault,
}

impl Kind {
    const fn matches(self, symbol: SymbolKind) -> bool {
        matches!(
            (self, symbol),
            (Self::Free, SymbolKind::Fn)
                | (Self::Inherent, SymbolKind::InherentMethod)
                | (Self::TraitImpl, SymbolKind::TraitImplMethod)
                | (Self::TraitDefault, SymbolKind::TraitDefaultMethod)
        )
    }
}

/// Collect only original source data. Native parser/lexer storage still has
/// its separate envelope; these limits cover retained facts and metered work.
pub(super) fn collect(
    workspace: &Workspace,
    work_limit: usize,
    memory_limit: usize,
) -> Result<Analysis<'_>, SourceFailure<'_>> {
    let mut analysis = Analysis::new(work_limit, memory_limit);
    for file in workspace.files() {
        let profile;
        (analysis, profile) = context::for_file(analysis, workspace, file)?;
        (analysis, ()) =
            analysis.inspect_source_with_tokens(file, move |analysis, syntax, original| {
                let coordinates =
                    Coordinates::new(&file.text, syntax.shebang.as_deref(), &analysis.meter)
                        .map_err(|cause| Failure { byte: 0, cause })?;
                let mut failure = None;
                let result = (|| {
                    let mut crate_root = true;
                    for reach in &file.reached {
                        analysis.meter.work(1)?;
                        let target = workspace
                            .target(reach.target)
                            .ok_or(Refusal::Invariant("a source reach lost its actual target"))?;
                        analysis
                            .meter
                            .work(reach.module_path.len().max(target.crate_path.len()))?;
                        crate_root &= reach.module_path.as_str() == target.crate_path;
                    }
                    analysis.meter.work(file.text.len())?;
                    let original = tokens::inventory(original, &analysis.meter, &coordinates)?;
                    // Scopes borrow the same account as the lexical inventory;
                    // on every exit they must drop before that inventory.
                    let mut scopes =
                        Scopes::with_prelude(&file.text, &analysis.meter, profile, &coordinates);
                    scopes.enter_module(&syntax.items, &syntax.attrs, crate_root)?;
                    scopes.descendant_prelude()?;
                    let first = analysis.functions.len();
                    let mut walk = Walk {
                        workspace,
                        file,
                        functions: &mut analysis.functions,
                        meter: &analysis.meter,
                        original: original.as_slice(),
                        coordinates: &coordinates,
                        failure: None,
                    };
                    let result = walk
                        .items(&syntax.items, &mut scopes)
                        .and_then(|()| walk.closed(first));
                    failure = walk.failure;
                    result
                })();
                result.map_err(|cause| failure.unwrap_or(Failure { byte: 0, cause }))?;
                drop(coordinates);
                Ok(())
            })?;
    }
    Ok(analysis)
}

struct Walk<'source, 'borrow> {
    workspace: &'source Workspace,
    file: &'source SourceFile,
    functions: &'borrow mut Vec<Function<'source>>,
    meter: &'borrow Meter,
    original: &'borrow [tokens::Lexeme],
    coordinates: &'borrow Coordinates<'source, 'borrow>,
    failure: Option<Failure>,
}

impl<'source, 'meter> Walk<'source, 'meter> {
    fn items(
        &mut self,
        items: &[syn::Item],
        scopes: &mut Scopes<'source, 'meter>,
    ) -> Result<(), Refusal> {
        for item in items {
            let anchor = native_spans::statement::item_ends(item, self.coordinates)?
                .map_or_else(proc_macro2::Span::call_site, |ends| ends.0);
            let byte = self.coordinates.range(anchor)?.start;
            let result = (|| {
                self.meter.work(1)?;
                match item {
                    syn::Item::Fn(function) => self.function(
                        &function.sig,
                        &function.block,
                        &function.attrs,
                        native_spans::function(function, self.meter)?,
                        Kind::Free,
                        scopes,
                    )?,
                    syn::Item::Mod(module) if module.content.is_some() => {
                        scopes.enter_declared_module(module)?;
                        scopes.descendant_prelude()?;
                        if let Some((_, items)) = &module.content {
                            self.items(items, scopes)?;
                        }
                        scopes.leave()?;
                    }
                    syn::Item::Impl(block) => {
                        attributes::list(&block.attrs, self.meter, self.coordinates)?;
                        self.generics(&block.generics, scopes)?;
                        let kind = if block.trait_.is_some() {
                            Kind::TraitImpl
                        } else {
                            Kind::Inherent
                        };
                        for item in &block.items {
                            self.meter.work(1)?;
                            if let syn::ImplItem::Fn(function) = item {
                                self.function(
                                    &function.sig,
                                    &function.block,
                                    &function.attrs,
                                    native_spans::method(function, self.meter)?,
                                    kind,
                                    scopes,
                                )?;
                            }
                        }
                        scopes.leave()?;
                    }
                    syn::Item::Trait(block) => {
                        attributes::list(&block.attrs, self.meter, self.coordinates)?;
                        self.generics(&block.generics, scopes)?;
                        for item in &block.items {
                            self.meter.work(1)?;
                            if let syn::TraitItem::Fn(function) = item
                                && let Some(body) = &function.default
                            {
                                self.function(
                                    &function.sig,
                                    body,
                                    &function.attrs,
                                    native_spans::trait_function(function, self.meter)?,
                                    Kind::TraitDefault,
                                    scopes,
                                )?;
                            }
                        }
                        scopes.leave()?;
                    }
                    _ => {}
                }
                Ok(())
            })();
            if let Err(cause) = result {
                self.failure.get_or_insert(Failure { byte, cause });
                return Err(cause);
            }
        }
        Ok(())
    }

    fn generics(
        &self,
        generics: &syn::Generics,
        scopes: &mut Scopes<'source, 'meter>,
    ) -> Result<(), Refusal> {
        scopes.enter_values()?;
        for parameter in generics.params.pairs().map(Pair::into_value) {
            self.meter.work(1)?;
            let attrs = match parameter {
                syn::GenericParam::Const(parameter) => &parameter.attrs,
                syn::GenericParam::Type(parameter) => &parameter.attrs,
                syn::GenericParam::Lifetime(parameter) => &parameter.attrs,
            };
            attributes::list(attrs, self.meter, self.coordinates)?;
            if let syn::GenericParam::Const(parameter) = parameter {
                scopes.constant(scopes.ident(&parameter.ident)?)?;
            }
        }
        Ok(())
    }

    fn function(
        &mut self,
        signature: &syn::Signature,
        body: &syn::Block,
        attrs: &[syn::Attribute],
        native_span: native_spans::Ends,
        kind: Kind,
        scopes: &mut Scopes<'source, 'meter>,
    ) -> Result<(), Refusal> {
        // Attribute safety is unconditional, including empty functions whose
        // collection performs no value-name lookup.
        let span = self.coordinates.between(native_span.0, native_span.1)?;
        attributes::list(attrs, self.meter, self.coordinates).inspect_err(|&cause| {
            self.failure.get_or_insert(Failure {
                byte: span.start,
                cause,
            });
        })?;
        // Block::span prints all statements merely to recover this same
        // stored group span. Keep endpoint lookup independent of body size.
        let body_span = self.coordinates.range(body.brace_token.span.join())?;
        let name = scopes.ident(&signature.ident)?;
        let mut owners = Vec::new();
        for item in self.workspace.items_in(&self.file.path) {
            self.meter.work(1)?;
            if item.span.start_byte != span.start
                || item.span.end_byte != span.end
                || !eligible(item)
            {
                continue;
            }
            let actual_body = item
                .body
                .as_ref()
                .ok_or(Refusal::Invariant("eligible Source function has no body"))?;
            if !kind.matches(item.kind)
                || actual_body.start_byte != body_span.start
                || actual_body.end_byte != body_span.end
                || !names::same(name, item.name.as_str(), self.meter)?
            {
                return Err(Refusal::Invariant(
                    "parsed function and indexed source owner disagree",
                ));
            }
            self.meter.push_retained(&mut owners, item)?;
        }
        if owners.is_empty() {
            return Err(Refusal::Unsupported(
                "original function has no compatible indexed owner; source selection must agree across reaches",
            ));
        }
        let profile = scopes.profile()?;
        let facts = collect::function(&self.file.text, signature, body, scopes, self.meter)
            .map_err(|failure| {
                self.failure.get_or_insert(failure);
                failure.cause
            })?;
        let tokens = tokens::join(&self.file.text, &facts, self.original, self.meter)?;
        self.meter.push_retained(
            self.functions,
            Function {
                source: &self.file.text,
                origin: Origin::Source {
                    workspace: self.workspace,
                    file: self.file,
                    owners,
                    profile,
                },
                facts,
                tokens,
            },
        )
    }

    /// The indexed bodies and traversed bodies must account for each other.
    fn closed(&self, first: usize) -> Result<(), Refusal> {
        for item in self.workspace.items_in(&self.file.path) {
            self.meter.work(1)?;
            if !eligible(item) {
                continue;
            }
            let mut count = 0_usize;
            for function in &self.functions[first..] {
                self.meter.work(1)?;
                let (file, owners) = function.origin.source().ok_or(Refusal::Invariant(
                    "workspace function lost its actual source provenance",
                ))?;
                if !std::ptr::eq(file, self.file)
                    || !std::ptr::eq(function.source, self.file.text.as_str())
                {
                    return Err(Refusal::Invariant(
                        "workspace function source provenance changed",
                    ));
                }
                for owner in owners {
                    self.meter.work(1)?;
                    if std::ptr::eq(*owner, item) {
                        count += 1;
                    }
                }
            }
            if count != 1 {
                return Err(Refusal::Unsupported(
                    "indexed function selection is not accounted for exactly once by original source",
                ));
            }
        }
        Ok(())
    }
}

fn eligible(item: &Item) -> bool {
    item.body.is_some()
        && matches!(
            item.kind,
            SymbolKind::Fn
                | SymbolKind::InherentMethod
                | SymbolKind::TraitImplMethod
                | SymbolKind::TraitDefaultMethod
        )
}

#[cfg(test)]
#[path = "workspace_tests.rs"]
mod tests;
