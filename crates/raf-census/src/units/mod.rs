// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The units of a workspace: the functions, methods, constants and statics
//! whose bodies the census fingerprints.
//!
//! A unit is one source item. A file that several targets reach, a module
//! declared by both a library and a binary, a support module shared by two
//! test binaries, is one unit, whatever number of paths it is reached under.
//!
//! The parameter bindings of a function and the range of a constant's
//! initialiser come from the parse of `raf-source`, which runs on its own
//! budgeted stack. The census itself lexes bodies, which does not recurse, and
//! parses no signature and no item.

mod scan;

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

/// How a string literal is written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum StringKind {
    /// `"text"`, raw or not.
    Str,
    /// `b"bytes"`, raw or not.
    Bytes,
    /// `c"text"`, raw or not.
    CStr,
}

/// A string literal of a unit with the line it starts on.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct StringLiteral {
    /// The decoded value. The bytes of a byte string or a C string that are
    /// not UTF-8 are decoded lossily, each bad sequence as U+FFFD, and such a
    /// literal never names a hex table.
    pub value: String,
    /// How the literal is written.
    pub kind: StringKind,
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
    /// The byte offset where the source item starts.
    pub start_byte: usize,
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
/// The lexer of bodies keeps the text of every lexed slice in a table of the
/// thread. Collection owns a fresh thread; its teardown releases the table
/// on success, error and unwinding, without changing caller-owned spans.
///
/// # Errors
///
/// [`Error::Source`] when a body or an initialiser does not lex, the
/// workspace does not hold the text of an item, or the source index names an
/// item that does not resolve or lists a source item with no reach. Each names
/// the file; nothing is skipped. [`Error::CollectionSpawn`] when the owned
/// thread cannot start; a panic resumes on the caller with its original payload.
pub(crate) fn collect(workspace: &Workspace, options: &CensusOptions) -> Result<Vec<Unit>, Error> {
    on_thread(|| collect_units(workspace, options))?
}

/// Run only span-owning work on a fresh thread. Both the closure and its
/// result must be Send: a caller's proc-macro span cannot enter or escape.
fn on_thread<R: Send>(work: impl FnOnce() -> R + Send) -> Result<R, Error> {
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .name("raf-census-units".into())
            .stack_size(2 * 1024 * 1024)
            .spawn_scoped(scope, work)
            .map_err(Error::CollectionSpawn)?;
        match worker.join() {
            Ok(result) => Ok(result),
            Err(payload) => std::panic::resume_unwind(payload),
        }
    })
}

/// Collect plain unit data while all temporary token spans belong to the worker.
fn collect_units(workspace: &Workspace, options: &CensusOptions) -> Result<Vec<Unit>, Error> {
    let normalizer = Normalizer::new(options);
    let mut units = Vec::new();
    for (file, offset, ids) in workspace.items_by_source() {
        let source_error = |reason: &str| Error::Source {
            file: file.to_string(),
            line: line_at(workspace, file, offset),
            reason: reason.to_owned(),
        };
        let mut reaches: Vec<&Item> = Vec::with_capacity(ids.len());
        for id in ids {
            reaches.push(
                workspace
                    .item(*id)
                    .ok_or_else(|| source_error("an item of the source index does not resolve"))?,
            );
        }
        // A shipping reach first, then the smallest path.
        reaches.sort_by(|a, b| {
            (!workspace.is_shipping(a), &a.path).cmp(&(!workspace.is_shipping(b), &b.path))
        });
        let Some(&chosen) = reaches.first() else {
            return Err(source_error("a source item of the index has no reach"));
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

/// The 1-based line of byte `offset` of `file`; 1 when the workspace does not
/// hold the text or the offset.
fn line_at(workspace: &Workspace, file: &RelPath, offset: usize) -> usize {
    workspace
        .source(file)
        .and_then(|source| source.get(..offset))
        .map_or(1, |head| head.bytes().filter(|b| *b == b'\n').count() + 1)
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
            let braced = text(
                workspace,
                &item.file,
                body.start_byte..body.end_byte,
                body.lines.start,
            )?;
            let contents = braced
                .strip_prefix('{')
                .and_then(|rest| rest.strip_suffix('}'))
                .ok_or_else(|| Error::Source {
                    file: file.clone(),
                    line: body.lines.start,
                    reason: "the body of the item is not a braced block".to_owned(),
                })?;
            // A receiver is `self`, which the body does not rename.
            let params = item
                .parameter_bindings()
                .iter()
                .filter(|name| *name != "self")
                .cloned()
                .collect();
            (params, contents, body.lines.start)
        }
        UnitKind::Const | UnitKind::Static => {
            let line = item.span.lines.start;
            let Some(range) = item.initializer() else {
                return Ok(None);
            };
            let before = text(
                workspace,
                &item.file,
                item.span.start_byte..range.start,
                line,
            )?;
            let contents = text(workspace, &item.file, range, line)?;
            (Vec::new(), contents, line + before.matches('\n').count())
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
    let via_self = functionlike && tower.calls_through_self();
    let shim_key = print
        .as_ref()
        .and_then(|print| print.shim.as_ref())
        .filter(|_| !trait_impl && !via_self)
        .cloned();
    Ok(Some(Unit {
        symbol: item.path.to_string(),
        name: item.name.to_string(),
        package,
        file,
        line: item.name_line,
        start_byte: item.span.start_byte,
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

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use super::on_thread;
    use crate::{CensusOptions, Error, MAX_NESTING, Normalizer, Slice};

    struct Teardown(Arc<AtomicUsize>);
    impl Drop for Teardown {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    thread_local! {
        static TEARDOWN: RefCell<Option<Teardown>> = const { RefCell::new(None) };
    }

    fn attach(count: &Arc<AtomicUsize>) {
        TEARDOWN.with(|slot| *slot.borrow_mut() = Some(Teardown(Arc::clone(count))));
    }

    #[test]
    fn collection_worker_tears_down_on_normalization_success_and_error_beside_a_preserved_caller_span()
     {
        let tokens: proc_macro2::TokenStream = "\n\ncaller".parse().expect("caller tokens");
        let span = tokens.into_iter().next().expect("caller token").span();
        let before = (span.start(), span.source_text(), span.file());
        let count = Arc::new(AtomicUsize::new(0));
        let options = CensusOptions::default();
        for (depth, succeeds) in [(MAX_NESTING, true), (MAX_NESTING + 1, false)] {
            let body = format!("{}1{}", "(".repeat(depth), ")".repeat(depth));
            let result = on_thread(|| {
                attach(&count);
                Normalizer::new(&options).tower(&Slice::new("body.rs", 7, &body))
            })
            .expect("worker starts");
            match result {
                Ok(_) => assert!(succeeds),
                Err(Error::Source { file, line, reason }) => {
                    assert!(!succeeds);
                    assert_eq!((file.as_str(), line), ("body.rs", 7));
                    assert!(reason.contains("delimiters nest deeper"));
                }
                Err(other) => panic!("unexpected normalization error: {other}"),
            }
            assert_eq!((span.start(), span.source_text(), span.file()), before);
        }
        assert_eq!(
            count.load(Ordering::SeqCst),
            2,
            "both owned workers completed TLS teardown"
        );
    }

    #[test]
    fn collection_worker_panic_keeps_its_payload_and_tears_down_without_invalidating_caller_spans()
    {
        let tokens: proc_macro2::TokenStream = "caller".parse().expect("caller tokens");
        let span = tokens.into_iter().next().expect("caller token").span();
        let before = (span.start(), span.source_text());
        let count = Arc::new(AtomicUsize::new(0));
        let caught = std::panic::catch_unwind(|| {
            on_thread::<()>(|| {
                attach(&count);
                let _: proc_macro2::TokenStream =
                    "owned_worker_tokens".parse().expect("worker tokens");
                std::panic::panic_any(37_u32);
            })
        })
        .expect_err("worker panic resumes on caller");
        assert_eq!(caught.downcast_ref::<u32>(), Some(&37));
        assert_eq!(count.load(Ordering::SeqCst), 1);
        assert_eq!((span.start(), span.source_text()), before);
    }
}
