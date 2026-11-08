// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Bounded parsing: a pre-scan, then `syn` on a dedicated thread.

use std::panic;
use std::thread;

use crate::budget::Budget;
use crate::error::{LoadError, LoadErrorKind};
use crate::meter::{Meter, NativeAdmission};
use crate::path::RelPath;
use crate::prescan::{Scan, prescan_file, shebang};
use crate::{LoadOptions, TreeError};

/// The stack of the thread that parses, in bytes: 64 MiB.
///
/// The caller's stack is not used, so a deep file costs this thread and not
/// the thread of a test or of a tool.
pub const PARSE_STACK_BYTES: usize = 64 * 1024 * 1024;

/// An immutable source admitted by the caller-thread guard scan.
///
/// Private fields bind the handle to its borrowed path and text. Reading counts
/// starts no worker or native lexer. Its consuming parse methods use that
/// same text without repeating the guard scan. Admission is not syntax
/// validation and does not establish a complete native heap or work bound.
#[derive(Debug)]
pub struct PreparedSource<'source> {
    file: &'source RelPath,
    text: &'source str,
    scan: Scan,
    native_bound: usize,
}

/// Check the default file-token and recursion guards without starting a worker.
///
/// The returned handle binds the counts and subsequent parsing to immutable
/// borrowed input. It performs no filesystem access or source mutation.
///
/// # Errors
/// Returns the located guard refusal, or storage-size overflow when the native
/// token ceiling cannot be represented. Syntax is checked only when parsing.
pub fn prepare_source<'source>(
    file: &'source RelPath,
    text: &'source str,
) -> Result<PreparedSource<'source>, LoadError> {
    PreparedSource::new(file, text, Budget::FileTokens.limit())
}

impl<'source> PreparedSource<'source> {
    fn new(
        file: &'source RelPath,
        text: &'source str,
        max_tokens: usize,
    ) -> Result<Self, LoadError> {
        let scan = prescan_file(file, text, max_tokens)?;
        let native_bound = scan
            .tokens
            .checked_mul(3)
            .and_then(|tokens| text.len().checked_mul(2).map(|bytes| tokens.min(bytes)))
            .ok_or_else(|| {
                LoadError::new(
                    file,
                    None,
                    LoadErrorKind::Storage(purrdf_lex::allocation::StorageError::SizeOverflow),
                )
            })?;
        Ok(Self {
            file,
            text,
            scan,
            native_bound,
        })
    }

    /// Units compared against the guard's file-token limit.
    ///
    /// Numeric guard runs can fuse native leaves; this is not a native-node count.
    #[must_use]
    pub const fn token_count(&self) -> usize {
        self.scan.tokens
    }

    /// The highest additive recursion counter reached by the guard scan.
    #[must_use]
    pub const fn recursion(&self) -> usize {
        self.scan.recursion
    }

    /// Upper bound on complete native token-tree nodes if lexing succeeds.
    ///
    /// At most three native nodes arise per guard unit; source bytes separately
    /// bound documentation lowering at twice their length. This is the smaller
    /// checked bound and does not bound native AST storage or parser work.
    #[must_use]
    pub const fn native_token_bound(&self) -> usize {
        self.native_bound
    }

    /// Parse the admitted text on its scoped worker, then join it.
    ///
    /// # Errors
    /// Returns the native storage, syntax or worker-start errors of [`parse_source`].
    ///
    /// # Panics
    /// Resumes parser or visitor panics on the caller with the original payload.
    pub fn parse<R: Send>(
        self,
        visit: impl FnOnce(&syn::File) -> R + Send,
    ) -> Result<R, LoadError> {
        self.parse_mut(|tree| visit(tree))
    }

    /// Parse once and lend the original native tokens to the worker visitor.
    ///
    /// # Errors
    /// Returns the native errors of [`parse_source_with_tokens`], without rescanning.
    ///
    /// # Panics
    /// Resumes parser or visitor panics on the caller with the original payload.
    pub fn parse_with_tokens<R: Send>(
        self,
        visit: impl FnOnce(&syn::File, proc_macro2::TokenStream) -> R + Send,
    ) -> Result<R, LoadError> {
        let Self { file, text, .. } = self;
        on_worker(file, move || {
            let (original, header) = lex_source(file, text)?;
            let tree = syntax(file, original.clone(), header)?;
            Ok(visit(&tree, original))
        })
    }

    fn parse_mut<R: Send>(
        self,
        visit: impl FnOnce(&mut syn::File) -> R + Send,
    ) -> Result<R, LoadError> {
        let Self { file, text, .. } = self;
        on_worker(file, || {
            parse_syntax(file, text).map(|mut tree| visit(&mut tree))
        })
    }
}

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
    prepare_source(file, text)?.parse(visit)
}

/// Parse once and give the visitor the syntax and its original native tokens.
///
/// The same file token and recursion checks as [`parse_source`] run before the
/// worker starts. Both native values stay on that worker; only `Send` results
/// return. The AST is parsed from one shared stream clone; native parsing can
/// copy token payloads. The complete source is not lexed a second time, and
/// caller-thread spans are never invalidated.
///
/// Byte ranges index the original `text`, including after a leading BOM or
/// shebang. The BOM is handled by the native lexer. Language-ignored shebang
/// bytes are replaced with same-length whitespace solely for lexical input;
/// the original header is retained in `syn::File::shebang`. No file is edited
/// and the header is never executed. Native parser/token/TLS heap and visitor
/// allocations retain their separate resource contracts.
///
/// # Errors
/// Returns the located native budget, storage, syntax or worker-start refusal.
/// The visitor is never called after a failed admission, lex or parse.
///
/// # Panics
/// Resumes parser or visitor panics on the caller with their original payload.
pub fn parse_source_with_tokens<R: Send>(
    file: &RelPath,
    text: &str,
    visit: impl FnOnce(&syn::File, proc_macro2::TokenStream) -> R + Send,
) -> Result<R, LoadError> {
    prepare_source(file, text)?.parse_with_tokens(visit)
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
        prescan_file(file, text, options.max_file_tokens)?;
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
    let prepared = PreparedSource::new(file, text, max_tokens)?;
    let scan = prepared.scan;
    prepared.parse_mut(visit).map(|result| (result, scan))
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
    let (original, header) = lex_source(file, text)?;
    syntax(file, original, header)
}

fn lex_source(
    file: &RelPath,
    text: &str,
) -> Result<(proc_macro2::TokenStream, Option<String>), LoadError> {
    use purrdf_lex::allocation::{Memory, Resident};
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    let mut header = None;
    let normalized = if let Some(range) = shebang(text) {
        let error = |source| LoadError::new(file, Some(1), LoadErrorKind::Storage(source));
        header = Some(memory.string(&text[range.clone()]).map_err(error)?);
        // Padding preserves byte positions and all newlines. There are no
        // syntax positions within the ignored first-line header itself.
        std::borrow::Cow::Owned(
            memory
                .format(&format_args!(
                    "{}{blank:width$}{}",
                    &text[..range.start],
                    &text[range.end..],
                    blank = "",
                    width = range.len()
                ))
                .map_err(error)?,
        )
    } else {
        std::borrow::Cow::Borrowed(text)
    };
    let original = normalized
        .parse::<proc_macro2::TokenStream>()
        .map_err(|error| located_syntax(file, error.span(), error.to_string()))?;
    Ok((original, header))
}

fn syntax(
    file: &RelPath,
    original: proc_macro2::TokenStream,
    header: Option<String>,
) -> Result<syn::File, LoadError> {
    let mut tree: syn::File = syn::parse2(original)
        .map_err(|error| located_syntax(file, error.span(), error.to_string()))?;
    tree.shebang = header;
    Ok(tree)
}

fn located_syntax(file: &RelPath, span: proc_macro2::Span, message: String) -> LoadError {
    // Read thread-local span data before returning the error to the caller.
    let start = span.start();
    LoadError::new(
        file,
        Some(start.line),
        LoadErrorKind::Syntax {
            message,
            column: start.column + 1,
        },
    )
}
