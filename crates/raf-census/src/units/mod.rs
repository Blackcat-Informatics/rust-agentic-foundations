// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The units of a workspace: the functions, methods, constants and statics
//! whose bodies the census fingerprints.
//!
//! A unit is one source item. A file that several targets reach, a module
//! declared by both a library and a binary, a support module shared by two
//! test binaries, is one unit, whatever number of paths it is reached under.

mod scan;
mod sig;

use std::collections::BTreeSet;

use raf_source::{Item, RelPath, SymbolKind, Workspace};

use crate::normalize::{Level, TableKind};
use crate::{CensusOptions, Error, Fingerprint, Normalizer, Print, Slice};

/// What a unit is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum UnitKind {
    /// A free function.
    Fn,
    /// A method or associated function: inherent, of a trait implementation,
    /// or a trait method with a default body.
    Method,
    /// A constant, free or associated.
    Const,
    /// A static.
    Static,
}

/// A string literal of a unit with the line it starts on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StringLiteral {
    /// The decoded value.
    pub value: String,
    /// The 1-based line of the literal in its file.
    pub line: usize,
}

/// One fingerprinted source item.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Unit {
    /// The full path of the item. When several paths reach the item, the
    /// symbol is the one a shipping target reaches, then the smallest path.
    pub symbol: String,
    /// The last segment of the symbol.
    pub name: String,
    /// The package of the target the symbol belongs to.
    pub package: String,
    /// The file, relative to the workspace root.
    pub file: String,
    /// The 1-based line of the item's name.
    pub line: usize,
    /// What the unit is.
    pub kind: UnitKind,
    /// True for a method of a trait implementation.
    pub trait_impl: bool,
    /// True when the body calls through a path that starts with `Self`.
    pub via_self: bool,
    /// True when some target that ships reaches the item other than through
    /// test code.
    pub shipping: bool,
    /// The names the parameters bind, in signature order; empty for a
    /// constant or a static.
    pub params: Vec<String>,
    /// The fingerprints of a function body; `None` for a constant or a static.
    pub print: Option<Print>,
    /// The key a shim groups under: the `shim` fingerprint of the body,
    /// absent when the body has none, for a method of a trait implementation
    /// and for a forwarder that calls through `Self::`.
    pub shim_key: Option<Fingerprint>,
    /// Every integer literal of the body or of the initialiser, by value.
    pub constants: BTreeSet<u128>,
    /// The hex-digit tables the body or the initialiser holds.
    pub tables: BTreeSet<TableKind>,
    /// Every string literal of the body or of the initialiser, in source
    /// order.
    pub strings: Vec<StringLiteral>,
}

/// What a symbol kind makes of an item: its unit kind and whether it is a
/// method of a trait implementation. Every other kind yields no unit.
const fn unit_kind(kind: SymbolKind) -> Option<(UnitKind, bool)> {
    match kind {
        SymbolKind::Fn => Some((UnitKind::Fn, false)),
        SymbolKind::InherentMethod | SymbolKind::TraitDefaultMethod => {
            Some((UnitKind::Method, false))
        }
        SymbolKind::TraitImplMethod => Some((UnitKind::Method, true)),
        SymbolKind::Const | SymbolKind::AssocConst => Some((UnitKind::Const, false)),
        SymbolKind::Static => Some((UnitKind::Static, false)),
        _ => None,
    }
}

/// The units of `workspace`, in the order of file and position.
///
/// # Errors
///
/// [`Error::Source`] when a body does not lex, or a signature or an
/// initialiser does not parse.
pub(crate) fn collect(workspace: &Workspace, options: &CensusOptions) -> Result<Vec<Unit>, Error> {
    let normalizer = Normalizer::new(options);
    let mut units = Vec::new();
    for (_, _, ids) in workspace.items_by_source() {
        let mut reaches: Vec<&Item> = ids.iter().filter_map(|id| workspace.item(*id)).collect();
        // A shipping reach first, then the smallest path.
        reaches.sort_by(|a, b| {
            (!workspace.is_shipping(a), &a.path).cmp(&(!workspace.is_shipping(b), &b.path))
        });
        let Some(&chosen) = reaches.first() else {
            continue;
        };
        let Some((kind, trait_impl)) = unit_kind(chosen.kind) else {
            continue;
        };
        let shipping = reaches.iter().any(|item| workspace.is_shipping(item));
        if let Some(unit) = build(
            workspace,
            normalizer,
            options,
            chosen,
            (kind, trait_impl),
            shipping,
        )? {
            units.push(unit);
        }
    }
    Ok(units)
}

/// The text of `range` of `file`, or the failure that the workspace does not
/// hold it.
fn text<'a>(
    workspace: &'a Workspace,
    file: &RelPath,
    range: std::ops::Range<usize>,
    line: usize,
) -> Result<&'a str, Error> {
    workspace
        .source(file)
        .and_then(|source| source.get(range))
        .ok_or_else(|| Error::Source {
            file: file.to_string(),
            line,
            reason: "the span of the item is not text of its file".to_owned(),
        })
}

/// The unit of `item`, or `None` for a function without a body or a
/// constant without a value.
fn build(
    workspace: &Workspace,
    normalizer: Normalizer,
    options: &CensusOptions,
    item: &Item,
    (kind, trait_impl): (UnitKind, bool),
    shipping: bool,
) -> Result<Option<Unit>, Error> {
    let file = item.file.to_string();
    let (params, slice_text, first_line) = match kind {
        UnitKind::Fn | UnitKind::Method => {
            let Some(body) = &item.body else {
                return Ok(None);
            };
            // The span of a body holds its braces; the slice is their contents.
            let inner =
                body.start_byte + 1..body.end_byte.saturating_sub(1).max(body.start_byte + 1);
            let signature = text(
                workspace,
                &item.file,
                item.span.start_byte..body.start_byte,
                item.span.lines.start,
            )?;
            let params = sig::parameters(&file, item.span.lines.start, item.kind, signature)?;
            let contents = text(workspace, &item.file, inner, body.lines.start)?;
            (params, contents, body.lines.start)
        }
        UnitKind::Const | UnitKind::Static => {
            let whole = text(
                workspace,
                &item.file,
                item.span.start_byte..item.span.end_byte,
                item.span.lines.start,
            )?;
            let Some(range) = sig::initializer(&file, item.span.lines.start, item.kind, whole)?
            else {
                return Ok(None);
            };
            let skipped = whole
                .get(..range.start)
                .map_or(0, |head| head.matches('\n').count());
            let contents = whole.get(range).unwrap_or_default();
            (Vec::new(), contents, item.span.lines.start + skipped)
        }
    };
    let package = workspace
        .target(item.target)
        .map(|target| target.package.clone())
        .ok_or_else(|| Error::Source {
            file: file.clone(),
            line: item.name_line,
            reason: "the item belongs to no target of the workspace".to_owned(),
        })?;
    let slice = Slice::new(&file, first_line, slice_text).params(params.iter().cloned());
    let tower = normalizer.tower(&slice)?;
    let facts = scan::facts(&file, tower.form(Level::Verbatim))?;
    let functionlike = matches!(kind, UnitKind::Fn | UnitKind::Method);
    let print = functionlike.then(|| tower.print(options.min_tokens));
    let via_self = functionlike && scan::calls_through_self(&tower.form(Level::Verbatim).nodes);
    let shim_key = print
        .as_ref()
        .and_then(|print| print.shim.as_ref())
        .filter(|_| !trait_impl && !via_self)
        .map(|shim| shim.fingerprint.clone());
    Ok(Some(Unit {
        symbol: item.path.to_string(),
        name: item.name.to_string(),
        package,
        file,
        line: item.name_line,
        kind,
        trait_impl,
        via_self,
        shipping,
        params,
        print,
        shim_key,
        constants: facts.constants,
        tables: facts.tables,
        strings: facts.strings,
    }))
}
