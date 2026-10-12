// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Bounded parsing: a pre-scan, then `syn` on a dedicated thread.

use std::panic;
use std::thread;

use crate::error::{LoadError, LoadErrorKind};
use crate::path::RelPath;
use crate::prescan::prescan;

/// The stack of the thread that parses, in bytes: 64 MiB.
///
/// The caller's stack is not used, so a deep file costs this thread and not
/// the thread of a test or of a tool.
pub const PARSE_STACK_BYTES: usize = 64 * 1024 * 1024;

/// Parse `text` as a Rust file and run `visit` on the syntax tree.
///
/// The recursion budgets are checked on the caller's thread first. The parse
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
    prescan(file, text)?;
    thread::scope(|scope| {
        let worker = thread::Builder::new()
            .name(file.to_string())
            .stack_size(PARSE_STACK_BYTES)
            .spawn_scoped(scope, move || {
                // Span data belongs to the thread that parsed, so the error's
                // message and position are read here and leave as plain data.
                syn::parse_file(text)
                    .map(|tree| visit(&tree))
                    .map_err(|error| {
                        let start = error.span().start();
                        SyntaxFailure {
                            message: error.to_string(),
                            line: start.line,
                            column: start.column + 1,
                        }
                    })
            })
            .map_err(|source| LoadError::new(file, None, LoadErrorKind::Spawn(source)))?;
        match worker.join() {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(failure)) => Err(LoadError::new(
                file,
                Some(failure.line),
                LoadErrorKind::Syntax {
                    message: failure.message,
                    column: failure.column,
                },
            )),
            Err(payload) => panic::resume_unwind(payload),
        }
    })
}

/// A parse error as plain data.
struct SyntaxFailure {
    message: String,
    line: usize,
    column: usize,
}
