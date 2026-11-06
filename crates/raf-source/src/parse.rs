// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Bounded parsing: a pre-scan, then `syn` on a dedicated thread.

use std::panic;
use std::thread;

use crate::budget::Budget;
use crate::error::{LoadError, LoadErrorKind};
use crate::meter::{Meter, NativeAdmission};
use crate::path::RelPath;
use crate::prescan::{Scan, prescan};
use crate::{LoadOptions, TreeError};

/// The stack of the thread that parses, in bytes: 64 MiB.
///
/// The caller's stack is not used, so a deep file costs this thread and not
/// the thread of a test or of a tool.
pub const PARSE_STACK_BYTES: usize = 64 * 1024 * 1024;

/// Parse `text` as a Rust file and run `visit` on the syntax tree.
///
/// The recursion budgets and the token budget, at its default limit
/// ([`Budget::FileTokens`](crate::Budget::FileTokens)), are checked on the
/// caller's thread first. The parse
/// and `visit` then run on a scoped thread named after `file`, with a stack of
/// [`PARSE_STACK_BYTES`]; a stack overflow in the runtime names that thread.
/// A panic in `visit` or in the parser is raised again on the caller's thread.
///
/// The tree is dropped when `visit` returns, so `visit` copies out what the
/// caller keeps.
///
/// # Errors
///
/// Returns a [`LoadError`] naming `file` when the text exceeds a budget
/// ([`LoadErrorKind::Budget`]), is not valid Rust ([`LoadErrorKind::Syntax`])
/// or the thread cannot start ([`LoadErrorKind::Spawn`]).
///
/// # Panics
///
/// Panics only when `visit` or the parser panics, with the original payload.
pub fn parse_source<R: Send>(
    file: &RelPath,
    text: &str,
    visit: impl FnOnce(&syn::File) -> R + Send,
) -> Result<R, LoadError> {
    parse_counted(file, text, Budget::FileTokens.limit(), visit).map(|(result, _)| result)
}

/// One explicitly selected source and its owned syntax tree.
///
/// The syntax and its spans stay on the worker that parsed them. A batch visitor
/// can move these records into its own analysis structures and return `Send`
/// report data through [`parse_sources`].
pub struct ParsedSource {
    /// The caller's source identity; no discovery or module selection is performed.
    pub path: RelPath,
    /// The original source text, moved from the input without copying.
    pub text: String,
    /// The syntax tree, owned by the visitor.
    pub syntax: syn::File,
}

/// Parse an explicit corpus once and give all its syntax trees to one visitor.
///
/// Input order and identities are preserved, including repeated identities.
/// Before parsing any file, the complete input is checked against
/// `options.max_file_bytes`, `options.max_total_bytes`, `options.max_file_tokens`
/// and the native recursion budgets. The result-vector allocation is charged to
/// `options.max_item_bytes`. This bounds source input and corpus-container storage;
/// it does not meter the aggregate heap of retained `syn` trees or allocations
/// made by `visit`. Each file retains the parser's per-file resource bounds.
///
/// Parsing, visiting and dropping the trees run on one scoped worker with
/// [`PARSE_STACK_BYTES`]. The visitor runs exactly once, after every file parses,
/// including for an empty corpus. Only its `Send` result leaves the worker.
/// A visitor returning a `Result` retains its own typed error in that result.
/// No filesystem, Cargo, target selection or duplicate policy is involved.
///
/// # Errors
///
/// Returns the native [`LoadError`] naming the input that exceeded a limit or
/// failed to parse. Worker startup and corpus-vector allocation failures are
/// also errors. No failed corpus reaches the visitor as a partial success.
///
/// # Panics
///
/// A parser or visitor panic resumes on the caller with its original payload.
pub fn parse_sources<R: Send>(
    inputs: Vec<(RelPath, String)>,
    options: &LoadOptions,
    visit: impl FnOnce(Vec<ParsedSource>) -> R + Send,
) -> Result<R, LoadError> {
    let meter = Meter::new(options);
    for (file, text) in &inputs {
        if text.len() as u64 > options.max_file_bytes {
            return Err(LoadError::new(
                file,
                None,
                LoadErrorKind::Tree(Box::new(TreeError::TooLarge {
                    path: file.to_string(),
                    limit: options.max_file_bytes,
                })),
            ));
        }
        meter.read(file, text)?;
        prescan(file, text, options.max_file_tokens)?;
    }
    let file = inputs
        .first()
        .map_or_else(RelPath::root, |(file, _)| file.clone());
    let worker_name = file.clone();
    on_worker(&worker_name, move || {
        let mut trees = Vec::new();
        let mut admission = NativeAdmission::items(&meter, &file, None, 0);
        purrdf_lex::allocation::Memory::new(&mut admission)
            .reserve(&mut trees, inputs.len())
            .map_err(|error| admission.graph_error(error.into()))?;
        for (path, text) in inputs {
            let syntax = parse_syntax(&path, &text)?;
            trees.push(ParsedSource { path, text, syntax });
        }
        Ok(visit(trees))
    })
}

/// [`parse_source`] with a token limit of `max_tokens`, also returning what
/// the pre-scan counted.
pub(crate) fn parse_counted<R: Send>(
    file: &RelPath,
    text: &str,
    max_tokens: usize,
    visit: impl FnOnce(&syn::File) -> R + Send,
) -> Result<(R, Scan), LoadError> {
    parse_counted_mut(file, text, max_tokens, |tree| visit(tree))
}

/// The same bounded worker, allowing declaration selection before fact extraction.
pub(crate) fn parse_counted_mut<R: Send>(
    file: &RelPath,
    text: &str,
    max_tokens: usize,
    visit: impl FnOnce(&mut syn::File) -> R + Send,
) -> Result<(R, Scan), LoadError> {
    let scan = prescan(file, text, max_tokens)?;
    on_worker(file, || {
        parse_syntax(file, text).map(|mut tree| (visit(&mut tree), scan))
    })
}

fn on_worker<R: Send>(
    file: &RelPath,
    work: impl FnOnce() -> Result<R, LoadError> + Send,
) -> Result<R, LoadError> {
    thread::scope(|scope| {
        let worker = thread::Builder::new()
            .name(file.to_string())
            .stack_size(PARSE_STACK_BYTES)
            .spawn_scoped(scope, work)
            .map_err(|source| LoadError::new(file, None, LoadErrorKind::Spawn(source)))?;
        match worker.join() {
            Ok(result) => result,
            Err(payload) => panic::resume_unwind(payload),
        }
    })
}

fn parse_syntax(file: &RelPath, text: &str) -> Result<syn::File, LoadError> {
    syn::parse_file(text).map_err(|error| {
        // Read thread-local span data before returning the error to the caller.
        let start = error.span().start();
        LoadError::new(
            file,
            Some(start.line),
            LoadErrorKind::Syntax {
                message: error.to_string(),
                column: start.column + 1,
            },
        )
    })
}
