// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Retained fragment facts and their one original resource account.

use super::collect::{Facts, Failure};
use super::error::InspectionError;
use super::native_literals::Literals;
use super::resource::{Meter, Refusal, TerminalAccount, TerminalSlot};
use super::tokens::Tokens;
use raf_source::SyntaxBound;

#[cfg(test)]
use super::{collect, scopes::Scopes, tokens};

// Source copies at most 431 diagnostic bytes in one Display write. The named
// native String growth ceiling is at most twice that length, including its
// minimum allocation; the copied message can outlive the joined worker.
const SOURCE_ERROR_BYTES: usize = 862;

#[derive(Debug)]
pub(super) enum SourceFailure<'source> {
    Native {
        file: &'source raf_source::RelPath,
        cause: InspectionError,
    },
    Located {
        file: &'source raf_source::RelPath,
        cause: Failure,
    },
}

impl std::fmt::Display for SourceFailure<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Native { file, cause } => write!(formatter, "{file}: {cause}"),
            Self::Located { file, cause } => {
                write!(formatter, "{file}: byte {}: {:?}", cause.byte, cause.cause)
            }
        }
    }
}

/// Source is owned outside this analysis. No parser-owned value escapes.
pub(super) enum Origin<'source> {
    Source {
        workspace: &'source raf_source::Workspace,
        file: &'source raf_source::SourceFile,
        owners: Vec<&'source raf_source::Item>,
        profile: super::prelude::Prelude,
    },
    #[cfg(test)]
    Text,
}

impl<'source> Origin<'source> {
    pub(super) fn source(
        &self,
    ) -> Option<(
        &'source raf_source::SourceFile,
        &[&'source raf_source::Item],
    )> {
        match self {
            Self::Source { file, owners, .. } => {
                (!owners.is_empty()).then_some((*file, owners.as_slice()))
            }
            #[cfg(test)]
            Self::Text => None,
        }
    }
}

pub(super) struct Function<'source> {
    pub(super) source: &'source str,
    pub(super) origin: Origin<'source>,
    pub(super) facts: Facts<'source>,
    pub(super) tokens: Tokens<'source>,
}

/// All retained buffers precede their account, including during unwinding.
/// This owner stays on the caller while the bounded parser worker borrows it.
/// It is not shared across workers, and never borrows its own meter.
pub(super) struct Analysis<'source> {
    pub(super) functions: Vec<Function<'source>>,
    pub(super) excesses: Vec<SyntaxExcess<'source>>,
    pub(super) meter: Meter,
}

/// A parsed syntax tree that held more than the bytes prepaid for it.
///
/// The excess was charged to the same account. A record is evidence that the
/// per-node or per-byte term of the bound is too low for this input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SyntaxExcess<'source> {
    pub(super) file: &'source raf_source::RelPath,
    /// Native token-tree nodes of the lexed source.
    pub(super) nodes: usize,
    pub(super) text_bytes: usize,
    pub(super) prepaid: usize,
    pub(super) held: usize,
    /// The most numerous node kind of the tree and its count.
    pub(super) kind: &'static str,
    pub(super) kind_nodes: usize,
}

/// The prepaid syntax bound of one source, carried to the point just after the
/// parse. It borrows the receipt that refunds every prepaid envelope at the join.
pub(super) struct Reconciliation<'receipt, 'source> {
    file: &'source raf_source::SourceFile,
    nodes: usize,
    prepaid: usize,
    reserved: &'receipt mut usize,
}

impl<'source> Reconciliation<'_, 'source> {
    /// Compare the parsed tree with its prepaid bound, before any analysis.
    /// An excess is charged to the same account and recorded; a grant that
    /// fails refuses here with no partial result. The allocation that is
    /// reported has already happened; only the prepaid bound refuses in advance.
    pub(super) fn check(
        self,
        analysis: &mut Analysis<'source>,
        syntax: &syn::File,
    ) -> Result<(), Failure> {
        let held = raf_source::syntax_held(syntax);
        let Some(excess) = held.bytes().checked_sub(self.prepaid).filter(|n| *n > 0) else {
            return Ok(());
        };
        let (kind, kind_nodes) = held.dominant();
        let refusal = Refusal::SyntaxExcess {
            nodes: self.nodes,
            prepaid: self.prepaid,
            held: held.bytes(),
            kind,
        };
        let fail = |cause| Failure { byte: 0, cause };
        let exhausted = |cause| match cause {
            Refusal::Memory { .. } => refusal,
            other => other,
        };
        let combined = self.reserved.checked_add(excess).ok_or(Refusal::Storage(
            purrdf_lex::allocation::StorageError::SizeOverflow,
        ));
        let combined = combined.map_err(fail)?;
        analysis
            .meter
            .prepay_native(excess)
            .map_err(|cause| fail(exhausted(cause)))?;
        // Change the caller-owned receipt immediately after admission.
        *self.reserved = combined;
        analysis
            .meter
            .push_retained(
                &mut analysis.excesses,
                SyntaxExcess {
                    file: &self.file.path,
                    nodes: self.nodes,
                    text_bytes: self.file.text.len(),
                    prepaid: self.prepaid,
                    held: held.bytes(),
                    kind,
                    kind_nodes,
                },
            )
            .map_err(|cause| fail(exhausted(cause)))
    }
}

/// The slot must deallocate before the original account on every unwind path.
struct PreparedInspection<'source> {
    slot: TerminalSlot,
    analysis: Analysis<'source>,
    source: raf_source::PreparedSource<'source>,
    reserved: usize,
}

impl<'source> Analysis<'source> {
    pub(super) const fn new(work_limit: usize, memory_limit: usize) -> Self {
        Self {
            functions: Vec::new(),
            excesses: Vec::new(),
            meter: Meter::new(work_limit, memory_limit),
        }
    }

    /// Borrow the original owner; its caller discards it after a failed analysis.
    /// Enclosing scopes are established by the caller from actual parsed source,
    /// and die before this owner moves back out of the parser callback.
    #[cfg(test)]
    pub(super) fn with_function(
        &mut self,
        source: &'source str,
        signature: &syn::Signature,
        body: &syn::Block,
        parsed: (&syn::File, proc_macro2::TokenStream),
        profile: Option<super::prelude::Prelude>,
        context: impl FnOnce(&mut Scopes<'source, '_>) -> Result<(), Refusal>,
    ) -> Result<(), Failure> {
        let (syntax, original) = parsed;
        let coordinates =
            super::coordinates::Coordinates::new(source, syntax.shebang.as_deref(), &self.meter)
                .map_err(|cause| Failure { byte: 0, cause })?;
        let byte = coordinates
            .range(body.brace_token.span.open())
            .map_err(|cause| Failure { byte: 0, cause })?
            .start;
        let mut scopes = match profile {
            Some(profile) => Scopes::with_prelude(source, &self.meter, profile, &coordinates),
            None => Scopes::new(source, &self.meter, &coordinates),
        };
        context(&mut scopes).map_err(|cause| Failure { byte, cause })?;
        let facts = collect::function(source, signature, body, &mut scopes, &self.meter)?;
        drop(scopes);
        let original = tokens::inventory(original, &self.meter, &coordinates)
            .map_err(|cause| Failure { byte, cause })?;
        let tokens = tokens::join(source, &facts, original.as_slice(), &self.meter)
            .map_err(|cause| Failure { byte, cause })?;
        drop(original);
        drop(coordinates);
        // Facts payload buffers have already been admitted by this same meter.
        // A failed outer-vector growth ends the entire owner, not just this file.
        self.meter
            .push_retained(
                &mut self.functions,
                Function {
                    source,
                    origin: Origin::Text,
                    facts,
                    tokens,
                },
            )
            .map_err(|cause| Failure { byte, cause })?;
        Ok(())
    }

    pub(super) fn functions(&self) -> &[Function<'source>] {
        &self.functions
    }

    pub(super) const fn meter(&self) -> &Meter {
        &self.meter
    }

    /// Lend the owner to Source's bounded worker until its explicit join.
    /// Native failure and panic cleanup finish before this account can drop.
    pub(super) fn inspect_source<R: Send>(
        self,
        file: &'source raf_source::SourceFile,
        inspect: impl FnOnce(&mut Self, &syn::File) -> Result<R, Failure> + Send,
    ) -> Result<(Self, R), SourceFailure<'source>> {
        self.inspect_source_bounded(file, SyntaxBound::MEASURED, inspect)
    }

    /// [`Self::inspect_source`] under explicit syntax-bound terms; production
    /// passes the measured terms.
    pub(super) fn inspect_source_bounded<R: Send>(
        self,
        file: &'source raf_source::SourceFile,
        bound: SyntaxBound,
        inspect: impl FnOnce(&mut Self, &syn::File) -> Result<R, Failure> + Send,
    ) -> Result<(Self, R), SourceFailure<'source>> {
        self.inspect_lexed(file, bound, |analysis, lexed, reconcile| {
            lexed.parse(|syntax| {
                reconcile.check(analysis, syntax)?;
                inspect(analysis, syntax)
            })
        })
    }

    pub(super) fn inspect_source_with_tokens<R: Send>(
        self,
        file: &'source raf_source::SourceFile,
        inspect: impl FnOnce(&mut Self, &syn::File, proc_macro2::TokenStream) -> Result<R, Failure>
        + Send,
    ) -> Result<(Self, R), SourceFailure<'source>> {
        self.inspect_lexed(file, SyntaxBound::MEASURED, |analysis, lexed, reconcile| {
            lexed.parse_with_tokens(|syntax, tokens| {
                reconcile.check(analysis, syntax)?;
                inspect(analysis, syntax, tokens)
            })
        })
    }

    fn inspect_lexed<R: Send>(
        self,
        file: &'source raf_source::SourceFile,
        bound: SyntaxBound,
        inspect: impl FnOnce(
            &mut Self,
            raf_source::LexedSource<'source>,
            Reconciliation<'_, 'source>,
        ) -> Result<Result<R, Failure>, raf_source::LoadError>
        + Send,
    ) -> Result<(Self, R), SourceFailure<'source>> {
        let mut prepared = self.prepare_source(file)?;
        let max_nodes = prepared.source.native_token_bound();
        let result = prepared
            .source
            .with_lexed(|lexed| {
                let (nodes, prepaid) = match prepared.analysis.admit_literals(
                    lexed.tokens(),
                    max_nodes,
                    (file.text.len(), bound),
                    &mut prepared.reserved,
                ) {
                    Ok(admitted) => admitted,
                    Err(cause) => return Ok(Err(Failure { byte: 0, cause })),
                };
                let reconcile = Reconciliation {
                    file,
                    nodes,
                    prepaid,
                    reserved: &mut prepared.reserved,
                };
                inspect(&mut prepared.analysis, lexed, reconcile)
            })
            .and_then(std::convert::identity);
        prepared
            .analysis
            .located(file, result, prepared.reserved, prepared.slot)
    }

    /// Prepay the literal conversion storage and the syntax tree bound before
    /// the parser starts. Returns the exact node count and the tree bytes.
    fn admit_literals(
        &self,
        tokens: &proc_macro2::TokenStream,
        max_nodes: usize,
        (text_bytes, bound): (usize, SyntaxBound),
        reserved: &mut usize,
    ) -> Result<(usize, usize), Refusal> {
        let overflow = Refusal::Storage(purrdf_lex::allocation::StorageError::SizeOverflow);
        let measured = Literals::measure(tokens, max_nodes)?;
        let tree = bound.bytes(measured.nodes(), text_bytes).ok_or(overflow)?;
        let bytes = measured.storage()?.checked_add(tree).ok_or(overflow)?;
        let combined = reserved.checked_add(bytes).ok_or(overflow)?;
        self.meter.work(measured.conversion_work()?)?;
        self.meter.prepay_native(bytes)?;
        // Change the caller-owned receipt immediately after admission, before
        // parsing or another fallible operation can begin.
        *reserved = combined;
        Ok((measured.nodes(), tree))
    }

    /// Bind tracking and error storage to the exact source before native parsing.
    fn prepare_source(
        self,
        file: &'source raf_source::SourceFile,
    ) -> Result<PreparedInspection<'source>, SourceFailure<'source>> {
        let slot = self
            .meter
            .terminal_slot()
            .map_err(|cause| SourceFailure::Located {
                file: &file.path,
                cause: Failure { byte: 0, cause },
            })?;
        let original = self.prepay_source(file)?;
        let source = match raf_source::prepare_source(&file.path, &file.text) {
            Ok(source) => source,
            Err(cause) => {
                self.meter.refund_native(original);
                return Err(self.native_failure(&file.path, cause, slot));
            }
        };
        let reserve = || {
            let overflow = Refusal::Storage(purrdf_lex::allocation::StorageError::SizeOverflow);
            self.meter.work(Literals::measurement_work(
                file.text.len(),
                source.native_token_bound(),
            )?)?;
            let block = Self::tracking_block_bytes()?;
            let tracking = source
                .native_token_bound()
                .checked_mul(71)
                .and_then(|n| n.checked_add(4))
                .and_then(|n| n.checked_mul(block))
                .ok_or(overflow)?;
            let errors = Self::parser_error_bytes(source.native_token_bound())?;
            let extra = tracking
                .checked_add(errors)
                .and_then(|n| n.checked_add(SOURCE_ERROR_BYTES))
                .ok_or(overflow)?;
            let combined = original.checked_add(extra).ok_or(overflow)?;
            self.meter.prepay_native(extra)?;
            Ok(combined)
        };
        let reserved = reserve().map_err(|cause| SourceFailure::Located {
            file: &file.path,
            cause: Failure { byte: 0, cause },
        })?;
        Ok(PreparedInspection {
            slot,
            analysis: self,
            source,
            reserved,
        })
    }

    /// Fund source preparation, lexer buffers and parser-input storage.
    /// A possible header is deliberately broader than Source's trivia grammar.
    fn prepay_source(
        &self,
        file: &'source raf_source::SourceFile,
    ) -> Result<usize, SourceFailure<'source>> {
        let reserve = || {
            let overflow = Refusal::Storage(purrdf_lex::allocation::StorageError::SizeOverflow);
            let source = file.text.len();
            let (lines, minuses, digits) = self.source_counts(&file.text)?;
            // Initial line writes and geometric relocation share this ceiling.
            let slots = lines
                .checked_mul(3)
                .and_then(|n| n.checked_add(4))
                .ok_or(overflow)?;
            let root = slots
                .checked_mul(size_of::<usize>())
                .and_then(|bytes| bytes.checked_add(source))
                .ok_or(overflow)?;
            let lexer = Self::original_lexer_bytes(source)?;
            let parser_input = Self::parser_input_bytes(source, digits)?;
            let (registry, registry_work) = Self::source_registry(source, minuses, digits)?;
            // One diagnostic line search plus fixed search/span comparisons and
            // message-copy reads/writes. The two registry searches are below.
            let diagnostic_work = lines
                .checked_mul(5)
                .and_then(|n| n.checked_add(872))
                .ok_or(overflow)?;
            // Logical UTF-8 extent, text copy, character count/predicate events,
            // root line writes/relocation and registered numeric source payloads.
            // Allocator, stack/control and other native work remain separate.
            let work = source
                .checked_mul(4)
                .and_then(|n| n.checked_add(slots))
                .and_then(|n| n.checked_add(registry_work))
                .and_then(|n| n.checked_add(diagnostic_work))
                .ok_or(overflow)?;
            self.meter.work(work)?;
            // At most three BOM bytes and two header-introducer bytes are read.
            self.meter.work(5)?;
            let text = file.text.strip_prefix("\u{feff}").unwrap_or(&file.text);
            // Native formatting retains old/new buffers during geometric growth.
            // Header bytes, that overlap and logical copies fit 4 * b + 8.
            let header = if text.starts_with("#!") {
                source
                    .checked_mul(4)
                    .and_then(|n| n.checked_add(8))
                    .ok_or(overflow)?
            } else {
                0
            };
            self.meter.work(header)?;
            let bytes = root
                .checked_add(registry)
                .and_then(|n| n.checked_add(lexer))
                .and_then(|n| n.checked_add(parser_input))
                .and_then(|n| n.checked_add(header))
                .ok_or(overflow)?;
            self.meter.prepay_native(bytes)?;
            Ok(bytes)
        };
        reserve().map_err(|cause| SourceFailure::Located {
            file: &file.path,
            cause: Failure { byte: 0, cause },
        })
    }

    fn source_counts(&self, text: &str) -> Result<(usize, usize, usize), Refusal> {
        let overflow = Refusal::Storage(purrdf_lex::allocation::StorageError::SizeOverflow);
        // Three named byte predicates and three counter updates per byte.
        self.meter
            .work(text.len().checked_mul(6).ok_or(overflow)?)?;
        let (newlines, minuses, digits) = text.bytes().fold(
            (0usize, 0usize, 0usize),
            |(newlines, minuses, digits), byte| {
                // Each counter is at most the number of visited source bytes.
                (
                    newlines + usize::from(byte == b'\n'),
                    minuses + usize::from(byte == b'-'),
                    digits + usize::from(byte.is_ascii_digit()),
                )
            },
        );
        Ok((newlines.checked_add(1).ok_or(overflow)?, minuses, digits))
    }

    /// Root and parser-created registrations on one fresh inspection worker.
    fn source_registry(
        source: usize,
        minuses: usize,
        digits: usize,
    ) -> Result<(usize, usize), Refusal> {
        let overflow = Refusal::Storage(purrdf_lex::allocation::StorageError::SizeOverflow);
        // Six original negative visits; each Index part can register its stream
        // and one negative Literal. Index parts number at most twice digits.
        let extra = minuses
            .checked_mul(6)
            .and_then(|n| digits.checked_mul(4).and_then(|m| n.checked_add(m)))
            .ok_or(overflow)?;
        // Only numeric spellings enter these reparses; they are disjoint source
        // ranges. Documentation-generated string spellings are not numeric.
        let text = if extra == 0 {
            0
        } else {
            source
                .checked_add(minuses)
                .and_then(|n| n.checked_mul(8))
                .ok_or(overflow)?
        };
        let limit = usize::try_from(u32::MAX).map_err(|_| overflow)?;
        // Each registration also needs one position after its predecessor.
        source
            .checked_add(1)
            .and_then(|n| n.checked_add(text))
            .and_then(|n| n.checked_add(extra))
            .filter(|n| *n <= limit)
            .ok_or(Refusal::Count {
                resource: "native span positions",
                limit,
            })?;
        let records = extra.checked_add(2).ok_or(overflow)?;
        let record = Self::source_map_record_bytes()?;
        let registry = records
            .checked_mul(3)
            .and_then(|n| n.checked_add(4))
            .and_then(|n| n.checked_mul(record))
            .ok_or(overflow)?;
        // Numeric registrations have no newlines and keep one line offset.
        // Root text/lines are funded separately; include the dummy line here.
        let lines = extra
            .checked_add(1)
            .and_then(|n| n.checked_mul(size_of::<usize>()))
            .ok_or(overflow)?;
        let bytes = registry
            .checked_add(lines)
            .and_then(|n| n.checked_add(text))
            .ok_or(overflow)?;
        // Record construction/insertion and geometric relocation, dummy line,
        // numeric UTF-8 extent, copy, count/predicate events and line writes.
        let work = records
            .checked_mul(4)
            .and_then(|n| n.checked_mul(record))
            .and_then(|n| n.checked_add(1))
            .and_then(|n| text.checked_mul(4).and_then(|m| n.checked_add(m)))
            .and_then(|n| n.checked_add(extra))
            // Error::span joins once; Source's located error calls start once.
            // Their two registry searches need at most sixteen visits per row.
            .and_then(|n| records.checked_mul(16).and_then(|m| n.checked_add(m)))
            .ok_or(overflow)?;
        Ok((bytes, work))
    }

    /// Syn's root input and its one possible live numeric Index input.
    /// AST, error and other parser storage/work retain separate obligations.
    fn parser_input_bytes(source: usize, digits: usize) -> Result<usize, Refusal> {
        let overflow = Refusal::Storage(purrdf_lex::allocation::StorageError::SizeOverflow);
        let entry = Self::token_buffer_entry_bytes()?;
        // One entry per original node, one End per group and one root End.
        // N <= 2b, so E <= 4b+1. Include Vec growth and boxed-slice shrink overlap.
        let root = source
            .checked_mul(4)
            .and_then(|n| n.checked_add(1))
            .and_then(|n| n.checked_mul(3))
            .and_then(|n| n.checked_add(4))
            .and_then(|n| n.checked_mul(entry))
            .ok_or(overflow)?;
        // Conversion clones each shared original vector at most once, at its
        // exact length. Ident/String clones copy no more than the funded 30b
        // original spelling allowance, including documentation lowering.
        let copies = size_of::<proc_macro2::TokenTree>()
            .checked_mul(2)
            .and_then(|n| n.checked_add(30))
            .and_then(|n| n.checked_mul(source))
            .ok_or(overflow)?;
        let child = if digits == 0 {
            0
        } else {
            // Index parsing cannot recurse into another Index root. Numeric
            // parts have at most two leaves and no groups: at most three Entry
            // slots, four native vector slots, one Rc block and b+1 leaf bytes.
            let block = Self::stream_block_bytes()?;
            let native = size_of::<proc_macro2::TokenTree>()
                .checked_mul(4)
                .and_then(|n| n.checked_add(source))
                .and_then(|n| n.checked_add(1))
                .and_then(|n| n.checked_add(block))
                .ok_or(overflow)?;
            entry
                .checked_mul(13)
                .and_then(|n| n.checked_add(native))
                .ok_or(overflow)?
        };
        root.checked_add(copies)
            .and_then(|n| n.checked_add(child))
            .ok_or(overflow)
    }

    /// Bound the actual Entry variant fields and the inspected enum layout.
    fn token_buffer_entry_bytes() -> Result<usize, Refusal> {
        use proc_macro2::{Group, Ident, Literal, Punct};
        let overflow = Refusal::Storage(purrdf_lex::allocation::StorageError::SizeOverflow);
        let alignment = align_of::<Group>()
            .max(align_of::<Ident>())
            .max(align_of::<Punct>())
            .max(align_of::<Literal>())
            .max(align_of::<usize>())
            .max(align_of::<isize>());
        let payload = [
            size_of::<Group>()
                .checked_add(size_of::<usize>())
                .ok_or(overflow)?,
            size_of::<Ident>(),
            size_of::<Punct>(),
            size_of::<Literal>(),
            size_of::<isize>().checked_mul(2).ok_or(overflow)?,
        ]
        .into_iter()
        .fold(0, usize::max);
        // The named native compilers bound this five-variant tag by usize;
        // End(isize,isize) also bounds tag widening's first-field alignment.
        // Four gaps cover the two payload fields, tag prefix and final padding.
        let padding = alignment
            .checked_sub(1)
            .and_then(|n| n.checked_mul(4))
            .ok_or(overflow)?;
        payload
            .checked_add(size_of::<usize>())
            .and_then(|n| n.checked_add(padding))
            .ok_or(overflow)
    }

    /// Allocation for one native Rc<Vec<TokenTree>> under the inspected layout.
    fn stream_block_bytes() -> Result<usize, Refusal> {
        Self::rc_block_bytes(std::alloc::Layout::new::<Vec<proc_macro2::TokenTree>>())
    }

    /// One Cell<Unexpected> Rc allocation under the inspected native profiles.
    fn tracking_block_bytes() -> Result<usize, Refusal> {
        use proc_macro2::{Delimiter, Span};
        use std::alloc::Layout;
        use std::rc::Rc;
        let overflow = Refusal::Storage(purrdf_lex::allocation::StorageError::SizeOverflow);
        let alignment = align_of::<Span>()
            .max(align_of::<Delimiter>())
            .max(align_of::<Rc<()>>())
            .max(align_of::<usize>())
            .max(align_of::<isize>());
        let payload = size_of::<Span>()
            .checked_add(size_of::<Delimiter>())
            .ok_or(overflow)?
            .max(size_of::<Rc<()>>());
        // Unexpected has None, Some(Span, Delimiter) and Chain(Rc<Cell<_>>).
        // Its sized Rc handle bounds tag widening; Cell is transparent.
        let padding = alignment
            .checked_sub(1)
            .and_then(|n| n.checked_mul(4))
            .ok_or(overflow)?;
        let bytes = payload
            .checked_add(size_of::<usize>())
            .and_then(|n| n.checked_add(padding))
            .ok_or(overflow)?;
        let value = Layout::from_size_align(bytes, alignment)
            .map_err(|_| overflow)?
            .pad_to_align();
        Self::rc_block_bytes(value)
    }

    fn rc_block_bytes(value: std::alloc::Layout) -> Result<usize, Refusal> {
        use std::alloc::Layout;
        use std::cell::Cell;
        let overflow = Refusal::Storage(purrdf_lex::allocation::StorageError::SizeOverflow);
        // RcInner is repr(C, align(2)): two counters, then its value.
        // Compose public layouts without constructing a private-layout surrogate.
        let counter = Layout::new::<Cell<usize>>();
        let (counters, _) = counter.extend(counter).map_err(|_| overflow)?;
        let (block, _) = counters.extend(value).map_err(|_| overflow)?;
        Ok(block
            .align_to(2)
            .map_err(|_| overflow)?
            .pad_to_align()
            .size())
    }

    /// Live native Syn error records and strings; Source conversion is separate.
    fn parser_error_bytes(nodes: usize) -> Result<usize, Refusal> {
        use proc_macro2::Span;
        use std::thread::ThreadId;
        let overflow = Refusal::Storage(purrdf_lex::allocation::StorageError::SizeOverflow);
        let alignment = align_of::<Span>()
            .max(align_of::<ThreadId>())
            .max(align_of::<String>());
        // SpanRange, ThreadBound and ErrorMessage are two-field structures.
        // Their public fields bound six padding gaps, including final padding.
        let padding = alignment
            .checked_sub(1)
            .and_then(|n| n.checked_mul(6))
            .ok_or(overflow)?;
        let record = size_of::<Span>()
            .checked_mul(2)
            .and_then(|n| n.checked_add(size_of::<ThreadId>()))
            .and_then(|n| n.checked_add(size_of::<String>()))
            .and_then(|n| n.checked_add(padding))
            .ok_or(overflow)?;
        // Each scoped constructor uses an exact single-element vector. Failed
        // recursive probes retain at most one record per original anchor;
        // two local constructions also cover terminal replacement overlap.
        let records = nodes
            .checked_add(2)
            .and_then(|n| n.checked_mul(record))
            .ok_or(overflow)?;
        // Retained recovered strings have capacity <= 140. Two local formatting
        // and terminal construction allowances together need at most 6896.
        let strings = nodes
            .checked_mul(140)
            .and_then(|n| n.checked_add(6896))
            .ok_or(overflow)?;
        records.checked_add(strings).ok_or(overflow)
    }

    /// Original raw-source lexer payload only, under the inspected profiles.
    /// Later streams/clones, TLS, parser errors/AST and native work are separate.
    fn original_lexer_bytes(source: usize) -> Result<usize, Refusal> {
        type Tokens = Vec<proc_macro2::TokenTree>;
        type Iter = std::vec::IntoIter<proc_macro2::TokenTree>;

        let overflow = Refusal::Storage(purrdf_lex::allocation::StorageError::SizeOverflow);
        let block = Self::stream_block_bytes()?;
        // One completed stream per written/doc group, plus even an empty root.
        let references = source
            .checked_add(1)
            .and_then(|n| n.checked_mul(block))
            .ok_or(overflow)?;
        // Exact ordinary copies and escaped documentation spelling capacities,
        // including the single active old/new String growth, fit 30b.
        let spellings = source.checked_mul(30).ok_or(overflow)?;
        // Source-credited nodes include pending doc/group nodes and fit 2b.
        // Live Vec capacities fit 4 times those credits plus one old allocation.
        let vectors = source
            .checked_mul(10)
            .and_then(|n| n.checked_mul(size_of::<proc_macro2::TokenTree>()))
            .ok_or(overflow)?;

        let alignment = align_of::<Tokens>()
            .max(align_of::<proc_macro2::Delimiter>())
            .max(align_of::<u32>());
        let padding = alignment
            .checked_sub(1)
            .and_then(|n| n.checked_mul(6))
            .ok_or(overflow)?;
        // Frame: (u32, (Delimiter, TokenStreamBuilder)); two single-field
        // wrappers around Vec plus two tuples give six padding boundaries.
        let frame = [
            size_of::<Tokens>(),
            size_of::<proc_macro2::Delimiter>(),
            size_of::<u32>(),
            padding,
        ]
        .into_iter()
        .try_fold(0usize, |sum, n| sum.checked_add(n).ok_or(overflow))?;
        // Source enforces written delimiter depth before starting the worker.
        let depth = source.min(raf_source::Budget::Delimiters.limit());
        let frames = depth
            .checked_mul(3)
            .and_then(|n| n.checked_add(4))
            .and_then(|n| n.checked_mul(frame))
            .ok_or(overflow)?;
        // Drop's wrapper holds one public IntoIter. Documentation lowering can
        // add one nested group. Emptied nested streams allocate no drop stack.
        let iterator = size_of::<Iter>()
            .checked_add(align_of::<Iter>() - 1)
            .ok_or(overflow)?;
        let cleanup = depth
            .checked_add(1)
            .and_then(|n| n.checked_mul(3))
            .and_then(|n| n.checked_add(4))
            .and_then(|n| n.checked_mul(iterator))
            .ok_or(overflow)?;
        // Failed lexing can retain frames while already built groups drop.
        [references, spellings, vectors, frames, cleanup]
            .into_iter()
            .try_fold(0usize, |sum, n| sum.checked_add(n).ok_or(overflow))
    }

    /// Bound the pinned private record from its actual field types and padding.
    fn source_map_record_bytes() -> Result<usize, Refusal> {
        type Offsets = std::collections::BTreeMap<usize, usize>;
        let overflow = Refusal::Storage(purrdf_lex::allocation::StorageError::SizeOverflow);
        let alignment = align_of::<String>()
            .max(align_of::<Vec<usize>>())
            .max(align_of::<Offsets>())
            .max(align_of::<u32>());
        let padding = alignment
            .checked_sub(1)
            .and_then(|n| n.checked_mul(6))
            .ok_or(overflow)?;
        // Two boundaries for the inner two-u32 Span, four for outer FileInfo.
        [
            size_of::<String>(),
            size_of::<Vec<usize>>(),
            size_of::<Offsets>(),
            size_of::<u32>().checked_mul(2).ok_or(overflow)?,
            padding,
        ]
        .into_iter()
        .try_fold(0usize, |total, bytes| {
            total.checked_add(bytes).ok_or(overflow)
        })
    }

    fn located<R>(
        self,
        file: &'source raf_source::SourceFile,
        result: Result<Result<R, Failure>, raf_source::LoadError>,
        reserved: usize,
        slot: TerminalSlot,
    ) -> Result<(Self, R), SourceFailure<'source>> {
        let result = match result {
            Ok(result) => result,
            // Keep every admitted envelope with the escaped diagnostic. Dead
            // transient storage may conservatively remain in terminal state;
            // no admission or reset is possible after this ownership transfer.
            Err(cause) => return Err(self.native_failure(&file.path, cause, slot)),
        };
        drop(slot);
        self.meter.refund_native(size_of::<TerminalAccount>());
        self.meter.refund_native(reserved);
        result
            .map_err(|cause| SourceFailure::Located {
                file: &file.path,
                cause,
            })
            .map(|value| (self, value))
    }

    fn native_failure(
        mut self,
        file: &'source raf_source::RelPath,
        cause: raf_source::LoadError,
        slot: TerminalSlot,
    ) -> SourceFailure<'source> {
        // Keep the original owner and account alive even if buffer destruction
        // unwinds. Only the now-empty owner is dismantled by the infallible freeze.
        drop(std::mem::take(&mut self.functions));
        let account = Box::write(slot, self.meter.into_terminal());
        SourceFailure::Native {
            file,
            cause: InspectionError::new(cause, account),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::resource::Grant;
    use super::*;

    fn collect<'source>(
        mut analysis: Analysis<'source>,
        source: &'source str,
    ) -> Result<Analysis<'source>, Failure> {
        let path = raf_source::RelPath::new("owner.rs").unwrap();
        raf_source::parse_source_with_tokens(&path, source, |syntax, original| {
            let syn::Item::Fn(function) = &syntax.items[0] else {
                panic!("fixture function")
            };
            analysis.with_function(
                source,
                &function.sig,
                &function.block,
                (syntax, original),
                None,
                |scopes| {
                    // Positive closed context for these source-only fixtures.
                    // Ambient state does not grant a prelude profile.
                    scopes.enter_module(&syntax.items, &syntax.attrs, true)
                },
            )
        })
        .unwrap()?;
        Ok(analysis)
    }

    #[test]
    fn retained_functions_share_the_original_grant_across_parser_workers() {
        let first = "#![no_implicit_prelude]\nfn one(input: i32) { let x = input + 1; sink(x); }";
        let second = "#![no_implicit_prelude]\nfn two(other: i32) { let y = other + 1; sink(y); }";
        let one = collect(Analysis::new(1_000_000, 1_000_000), first).unwrap();
        let (work, peak) = one.meter().usage();
        let both = collect(one, second).unwrap();
        assert_eq!(both.functions().len(), 2);
        assert!(both.meter().usage().0 > work);
        assert!(both.meter().usage().1 > peak);
        for (function, original) in both.functions().iter().zip([first, second]) {
            assert!(std::ptr::eq(function.source, original));
            assert_eq!(
                function
                    .source
                    .get(function.facts.body.start..function.facts.body.end),
                original.get(original.find('{').unwrap()..),
            );
        }
        let exact = collect(Analysis::new(1_000_000, peak), first).unwrap();
        assert_eq!(exact.meter().usage().1, peak);
        let refused = collect(exact, second);
        assert!(
            matches!(refused, Err(Failure { cause: Refusal::Memory { limit }, .. }) if limit == peak)
        );
        let invalid = "#![no_implicit_prelude]\nfn invalid() { let x = async {}; }";
        assert!(matches!(
            collect(both, invalid),
            Err(Failure {
                cause: Refusal::Unsupported(_),
                ..
            })
        ));
    }

    fn preparation_workspace() -> raf_source::Workspace {
        let probes = format!(
            "fn probes() {{ let _ = -\"{}\"; let _ = c\"\\u{{1f980}}\"; let _ = cr#\"raw🦀\"#; let _ = br#\"raw\"#; }}",
            "1".repeat(256)
        );
        let dense = format!(
            "/// \0\u{200b}'\"\\\nfn dense() {{ {}{}{} }}",
            "{".repeat(16),
            ";".repeat(128),
            "}".repeat(16)
        );
        let tree = raf_source::MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
            ),
            (
                "src/lib.rs",
                "\u{feff}#!/not-an-executable 🦀\nfn helper() {}\nmod empty;\nmod dense;\nmod numbers;\nmod probes;\n",
            ),
            ("src/main.rs", "// 🦀 source data\n\nfn main() {}\n"),
            ("src/empty.rs", ""),
            ("src/dense.rs", dense.as_str()),
            ("src/probes.rs", probes.as_str()),
            (
                "src/numbers.rs",
                "fn numbers(value: ((u8, u8), u8)) {\nmatch 0 { -1 => {}, -1234567890 => {}, -1.25e+2f64 => {}, _ => {} };\nlet _ = value.0.0;\n}",
            ),
        ]);
        raf_source::Workspace::load(&tree, &["."]).expect("controlled source-only workspace")
    }

    #[test]
    fn source_preparation_grants_precede_parsing_and_survive_until_each_worker_joins() {
        let workspace = preparation_workspace();
        let path = raf_source::RelPath::new("src/lib.rs").expect("fixture path");
        let file = workspace.file(&path).expect("loaded header source");
        let (measured, ()) = Analysis::new(1_000_000, 1_000_000)
            .inspect_source(file, |_, syntax| {
                assert_eq!(syntax.shebang.as_deref(), Some("#!/not-an-executable 🦀"));
                Ok(())
            })
            .expect("header remains source data");
        let (work, peak) = measured.meter().usage();
        assert!(
            peak >= file.text.len(),
            "the padded source must fit its grant"
        );
        assert!(matches!(
            Analysis::new(work - 1, peak).inspect_source(file, |_, _| -> Result<(), Failure> {
                panic!("work refusal must precede the visitor")
            }),
            Err(SourceFailure::Located {
                cause: Failure {
                    cause: Refusal::Work { .. },
                    byte: 0
                },
                ..
            })
        ));
        assert!(matches!(
            Analysis::new(work, peak - 1).inspect_source_with_tokens(
                file,
                |_, _, _| -> Result<(), Failure> {
                    panic!("memory refusal must precede the visitor")
                }
            ),
            Err(SourceFailure::Located {
                cause: Failure {
                    cause: Refusal::Memory { .. },
                    byte: 0
                },
                ..
            })
        ));
        let (analysis, ()) = Analysis::new(work * 2, peak)
            .inspect_source(file, |analysis, _| {
                let mut scratch = Grant::new(analysis.meter());
                assert!(matches!(
                    scratch.reserve_native(1),
                    Err(Refusal::Memory { .. })
                ));
                Ok(())
            })
            .expect("exact header grant");
        let (analysis, ()) = analysis
            .inspect_source_with_tokens(file, |analysis, _, _| {
                let mut scratch = Grant::new(analysis.meter());
                assert!(matches!(
                    scratch.reserve_native(1),
                    Err(Refusal::Memory { .. })
                ));
                Ok(())
            })
            .expect("the previous joined worker refunded its grant");
        assert_eq!(analysis.meter().usage(), (work * 2, peak));

        let path = raf_source::RelPath::new("src/main.rs").expect("fixture path");
        let plain = workspace.file(&path).expect("loaded plain source");
        let (plain_analysis, ()) = Analysis::new(1_000_000, peak - 1)
            .inspect_source_with_tokens(plain, |_, syntax, _| {
                assert!(syntax.shebang.is_none());
                let syn::Item::Fn(function) = &syntax.items[0] else {
                    panic!("plain fixture function")
                };
                assert_eq!(function.sig.fn_token.span.start().line, 3);
                Ok(())
            })
            .expect("plain source fits without a header envelope");
        let lines = plain.text.bytes().filter(|byte| *byte == b'\n').count() + 1;
        assert!(
            plain_analysis.meter().usage().1 >= plain.text.len() + lines * size_of::<usize>(),
            "root source text and every line entry must remain funded"
        );
        assert_native_preparation_funding(&workspace);
        assert_numeric_registration_preparation(&workspace);
        assert_string_digits_keep_linear_probe_work(&workspace);
        assert_owned_source_diagnostic();
    }

    fn assert_string_digits_keep_linear_probe_work(workspace: &raf_source::Workspace) {
        let path = raf_source::RelPath::new("src/probes.rs").expect("fixture path");
        let file = workspace.file(&path).expect("loaded static probe data");
        // The source is intentionally not compiler-valid and is never executed.
        // Squaring its 256 string digits would exceed this work grant by itself.
        let (measured, ()) = Analysis::new(750_000, 16_000_000)
            .inspect_source(file, |_, syntax| {
                let syn::Item::Fn(function) = &syntax.items[0] else {
                    panic!("probe fixture function")
                };
                let syn::Stmt::Local(local) = &function.block.stmts[0] else {
                    panic!("probe fixture initializer")
                };
                let syn::Expr::Unary(unary) =
                    local.init.as_ref().expect("initializer").expr.as_ref()
                else {
                    panic!("negative literal probe retains unary syntax")
                };
                assert!(matches!(unary.op, syn::UnOp::Neg(_)));
                assert!(matches!(
                    unary.expr.as_ref(),
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(_),
                        ..
                    })
                ));
                for statement in &function.block.stmts[1..3] {
                    let syn::Stmt::Local(local) = statement else {
                        panic!("C-string fixture initializer")
                    };
                    assert!(matches!(
                        local.init.as_ref().expect("initializer").expr.as_ref(),
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::CStr(_),
                            ..
                        })
                    ));
                }
                Ok(())
            })
            .expect("string digits fund linear failed probes without numeric square work");
        let (work, peak) = measured.meter().usage();
        // The visitor charges no work: conversion admission is the last charge,
        // so this boundary refuses on the worker after lexing and before parsing.
        assert!(matches!(
            Analysis::new(work - 1, peak).inspect_source(file, |_, _| -> Result<(), Failure> {
                panic!("conversion work must be admitted before the syntax visitor")
            }),
            Err(SourceFailure::Located {
                cause: Failure {
                    cause: Refusal::Work { .. },
                    byte: 0
                },
                ..
            })
        ));
    }

    /// A crate whose one function body is `count` empty statements, so that
    /// the syntax tree term dominates its storage.
    fn statements_workspace(count: usize) -> raf_source::Workspace {
        let body = format!("fn body() {{ {} }}\n", ";".repeat(count));
        let tree = raf_source::MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
            ),
            ("src/lib.rs", body.as_str()),
        ]);
        raf_source::Workspace::load(&tree, &["."]).expect("controlled source-only workspace")
    }

    const AMPLE_WORK: usize = 1 << 31;
    const AMPLE_BYTES: usize = 1 << 30;

    fn library(workspace: &raf_source::Workspace) -> &raf_source::SourceFile {
        let path = raf_source::RelPath::new("src/lib.rs").expect("fixture path");
        workspace.file(&path).expect("loaded statement source")
    }

    /// The exact native node count, the source length and the prepaid bound.
    fn prepaid_for(file: &raf_source::SourceFile, bound: SyntaxBound) -> (usize, usize) {
        let nodes = raf_source::prepare_source(&file.path, &file.text)
            .expect("guard-admitted fixture")
            .with_lexed(|lexed| {
                Literals::measure(lexed.tokens(), usize::MAX)
                    .expect("measured fixture")
                    .nodes()
            })
            .expect("fixture lexes");
        let bytes = bound.bytes(nodes, file.text.len()).expect("the bound fits");
        (nodes, bytes)
    }

    /// INVARIANT: the syntax tree bound is charged before the parser runs, so
    /// a limit one byte under the peak refuses with a memory refusal and the
    /// visitor never sees a tree; the exact peak succeeds, and each joined
    /// worker refunds its envelope so the same account serves another file.
    #[test]
    fn syntax_tree_grant_is_prepaid_before_the_parser_runs() {
        let workspace = statements_workspace(4096);
        let file = library(&workspace);
        let (measured, ()) = Analysis::new(AMPLE_WORK, AMPLE_BYTES)
            .inspect_source(file, |_, _| Ok(()))
            .expect("ample limits admit the fixture");
        let (work, peak) = measured.meter().usage();
        let (_, tree) = prepaid_for(file, SyntaxBound::MEASURED);
        assert!(
            peak >= tree,
            "the peak {peak} holds the prepaid tree bound {tree}"
        );
        assert!(measured.excesses.is_empty(), "the measured terms suffice");
        assert!(matches!(
            Analysis::new(work, peak - 1).inspect_source(file, |_, _| -> Result<(), Failure> {
                panic!("the tree refusal must precede the visitor")
            }),
            Err(SourceFailure::Located {
                cause: Failure {
                    cause: Refusal::Memory { .. },
                    byte: 0
                },
                ..
            })
        ));
        let (first, ()) = Analysis::new(work * 2, peak)
            .inspect_source(file, |_, _| Ok(()))
            .expect("the exact peak is admitted");
        let (second, ()) = first
            .inspect_source(file, |_, _| Ok(()))
            .expect("the joined worker refunded its envelope");
        assert_eq!(second.meter().usage().1, peak);

        // The neighbour with half the statements fits the same limit and
        // prepays less.
        let half = statements_workspace(2048);
        let (neighbour, ()) = Analysis::new(work, peak)
            .inspect_source(library(&half), |_, _| Ok(()))
            .expect("half the statements fit the limit of the whole");
        assert!(neighbour.meter().usage().1 < peak);
        assert_eq!(neighbour.excesses.len(), 0);
    }

    /// INVARIANT: a parsed tree larger than its prepaid bound is never silent.
    /// The excess is charged to the same account and recorded; when it cannot
    /// be granted the run refuses before any analysis with the shape and the
    /// remedy; a tree within its bound records nothing and charges nothing more.
    #[test]
    fn embedded_excess_over_the_prepaid_bound_is_detected_charged_or_refused_and_reported() {
        let workspace = statements_workspace(4096);
        let file = library(&workspace);
        let low = SyntaxBound {
            per_node: 1,
            per_byte: 1,
        };
        let (nodes, prepaid) = prepaid_for(file, low);
        let (_, production) = prepaid_for(file, SyntaxBound::MEASURED);
        let (reference, held) = Analysis::new(AMPLE_WORK, AMPLE_BYTES)
            .inspect_source(
                file,
                |_, syntax| Ok(raf_source::syntax_held(syntax).bytes()),
            )
            .expect("the reference run");
        assert_eq!(reference.excesses.len(), 0);
        let base = reference.meter().usage().1 - production;
        assert!(held > prepaid, "the low terms are far under the tree");

        // Detected and charged.
        let (charged, ()) = Analysis::new(AMPLE_WORK, AMPLE_BYTES)
            .inspect_source_bounded(file, low, |_, _| Ok(()))
            .expect("an ample limit charges the excess");
        let [record] = charged.excesses.as_slice() else {
            panic!("exactly one excess record")
        };
        assert_eq!(record.file.as_str(), "src/lib.rs");
        assert_eq!(record.kind, "Stmt");
        assert_eq!(record.kind_nodes, 4096);
        assert_eq!(record.nodes, nodes);
        assert_eq!(record.text_bytes, file.text.len());
        assert_eq!((record.prepaid, record.held), (prepaid, held));
        let record_bytes = charged.excesses.capacity() * size_of::<SyntaxExcess<'_>>();
        let (work, peak) = charged.meter().usage();
        // The account peaks at everything else, the prepaid bound, the excess
        // over it, and the admitted record.
        assert_eq!(peak, base + prepaid + (held - prepaid) + record_bytes);

        // Refused and reported.
        let refused = Analysis::new(work, peak - 1).inspect_source_bounded(
            file,
            low,
            |_, _| -> Result<(), Failure> { panic!("the excess refusal must precede the visitor") },
        );
        let Err(failure) = refused else {
            panic!("a limit under the peak refuses the excess")
        };
        assert!(matches!(
            &failure,
            SourceFailure::Located {
                cause: Failure {
                    cause: Refusal::SyntaxExcess { kind: "Stmt", nodes: n, prepaid: p, held: h },
                    byte: 0,
                },
                ..
            } if (*n, *p, *h) == (nodes, prepaid, held)
        ));
        let message = super::super::Error::from(failure).to_string();
        for part in [
            "src/lib.rs",
            "SyntaxExcess",
            "Stmt",
            &nodes.to_string(),
            &prepaid.to_string(),
            &held.to_string(),
            "reduce input or increase the selected resource limit",
        ] {
            assert!(message.contains(part), "{part:?} missing from {message}");
        }
        let (exact, ()) = Analysis::new(work, peak)
            .inspect_source_bounded(file, low, |_, _| Ok(()))
            .expect("the exact peak admits the excess and its record");
        assert_eq!(exact.excesses.len(), 1);

        // Neighbour within the bound: nothing recorded, nothing beyond the
        // prepaid bound charged.
        let half = statements_workspace(2048);
        let half_file = library(&half);
        let (_, half_tree) = prepaid_for(half_file, SyntaxBound::MEASURED);
        let (within, half_held) = Analysis::new(AMPLE_WORK, AMPLE_BYTES)
            .inspect_source(half_file, |_, syntax| {
                Ok(raf_source::syntax_held(syntax).bytes())
            })
            .expect("the production terms hold the tree");
        assert_eq!(within.excesses.len(), 0);
        assert!(half_held <= half_tree);
        assert!(within.meter().usage().1 < reference.meter().usage().1);
    }

    /// INVARIANT: an embedded run refuses a syntax tree over the memory limit
    /// as a whole, with the remedy and the located shape, and returns no run;
    /// the neighbour at the limit of the complete run succeeds.
    #[test]
    fn embedded_run_refuses_a_syntax_tree_over_the_memory_limit_with_an_actionable_message() {
        use super::super::{Error, Options, RefusalKind, Run};
        let workspace = statements_workspace(4096);
        let file = library(&workspace);
        let (_, tree) = prepaid_for(file, SyntaxBound::MEASURED);
        let run = Run::analyze(&workspace, &Options::default()).expect("the default limits hold");
        let (_, peak) = run.usage();
        assert!(peak > tree);
        let options = Options {
            max_memory_bytes: tree,
            ..Options::default()
        };
        let Err(error) = Run::analyze(&workspace, &options) else {
            panic!("a limit under the tree bound refuses")
        };
        assert!(matches!(
            &error,
            Error::Collection {
                cause: RefusalKind::Memory { limit },
                byte: 0,
                ..
            } if *limit == tree
        ));
        let message = error.to_string();
        assert!(message.starts_with("src/lib.rs: byte 0: Memory { limit: "));
        assert!(message.ends_with("; reduce input or increase the selected resource limit"));
        let options = Options {
            max_memory_bytes: peak,
            ..Options::default()
        };
        assert!(Run::analyze(&workspace, &options).is_ok());
    }

    fn assert_owned_source_diagnostic() {
        let diagnostic = {
            let path = raf_source::RelPath::new("invalid.rs").expect("fixture path");
            let analysis = Analysis::new(1_000_000, 1_000_000);
            let slot = analysis
                .meter()
                .terminal_slot()
                .expect("terminal account slot");
            analysis
                .meter()
                .prepay_native(SOURCE_ERROR_BYTES)
                .expect("fund the escaped diagnostic before its construction");
            let cause = raf_source::parse_source(&path, "fn broken(", |_| {
                panic!("malformed source cannot reach the visitor")
            })
            .expect_err("real Source syntax refusal");
            super::super::Error::from(analysis.native_failure(&path, cause, slot))
        };
        let super::super::Error::Inspection(owned) = &diagnostic else {
            panic!("inspection failures retain their original account")
        };
        let native = owned.cause();
        assert_eq!(native.file.as_str(), "invalid.rs");
        assert!(matches!(
            &native.kind,
            raf_source::LoadErrorKind::Syntax { message, .. } if !message.is_empty()
        ));
        assert_eq!(diagnostic.to_string(), native.to_string());
        assert!(std::ptr::eq(
            std::error::Error::source(&diagnostic)
                .expect("preserved native source chain")
                .downcast_ref::<raf_source::LoadError>()
                .expect("the direct cause remains the native LoadError"),
            native
        ));
    }

    fn assert_native_preparation_funding(workspace: &raf_source::Workspace) {
        let path = raf_source::RelPath::new("src/dense.rs").expect("fixture path");
        let dense = workspace
            .file(&path)
            .expect("loaded punctuation-dense module");
        // The positive envelope includes all conservatively funded tracking
        // cells; exact boundary refusals below use the measured combined peak.
        let (analysis, ()) = Analysis::new(1_000_000, 8_000_000)
            .inspect_source_with_tokens(dense, |_, syntax, _| {
                let syn::Item::Fn(function) = &syntax.items[0] else {
                    panic!("dense fixture function")
                };
                let syn::Meta::NameValue(doc) = &function.attrs[0].meta else {
                    panic!("lowered documentation attribute")
                };
                let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(text),
                    ..
                }) = &doc.value
                else {
                    panic!("escaped documentation string")
                };
                assert_eq!(text.value(), " \0\u{200b}'\"\\");
                let mut block = function.block.as_ref();
                for _ in 0..16 {
                    let syn::Stmt::Expr(syn::Expr::Block(nested), _) = &block.stmts[0] else {
                        panic!("nested lexer group")
                    };
                    block = &nested.block;
                }
                assert_eq!(block.stmts.len(), 128);
                Ok(())
            })
            .expect("original lexer token vectors remain funded");
        assert!(
            analysis.meter().usage().1 >= 128 * size_of::<proc_macro2::TokenTree>(),
            "every written semicolon needs live native token-vector backing"
        );
        let path = raf_source::RelPath::new("src/empty.rs").expect("fixture path");
        let empty = workspace.file(&path).expect("loaded empty module");
        let (empty_analysis, ()) = Analysis::new(1_000_000, 1_000_000)
            .inspect_source(empty, |_, syntax| {
                assert!(syntax.items.is_empty());
                Ok(())
            })
            .expect("empty source still bootstraps native source storage");
        let fields = size_of::<String>()
            + size_of::<Vec<usize>>()
            + size_of::<std::collections::BTreeMap<usize, usize>>()
            + 2 * size_of::<u32>();
        assert!(
            empty_analysis.meter().usage().1 >= 2 * fields + 2 * size_of::<usize>(),
            "dummy and root records and both initial line entries must be funded"
        );
    }

    fn assert_numeric_registration_preparation(workspace: &raf_source::Workspace) {
        let path = raf_source::RelPath::new("src/numbers.rs").expect("fixture path");
        let file = workspace.file(&path).expect("loaded numeric module");
        // Expanded pattern syntax also increases the conservative native heap
        // envelopes. This case verifies reconstruction and coordinates; exact
        // memory refusal boundaries are exercised by the shared funding case.
        Analysis::new(1_000_000, 8_000_000)
            .inspect_source_with_tokens(file, |_, syntax, _| {
                let syn::Item::Fn(function) = &syntax.items[0] else {
                    panic!("numeric fixture function")
                };
                let syn::Stmt::Expr(syn::Expr::Match(patterns), _) = &function.block.stmts[0]
                else {
                    panic!("numeric pattern fixture")
                };
                let syn::Pat::Lit(pattern) = &patterns.arms[0].pat else {
                    panic!("negative literal pattern")
                };
                let syn::Lit::Int(integer) = &pattern.lit else {
                    panic!("negative integer reconstruction")
                };
                assert_eq!(integer.token().to_string(), "-1");
                let syn::Pat::Lit(pattern) = &patterns.arms[1].pat else {
                    panic!("negative literal growth fixture")
                };
                let syn::Lit::Int(integer) = &pattern.lit else {
                    panic!("reconstructed long integer spelling")
                };
                assert_eq!(integer.token().to_string(), "-1234567890");
                let syn::Pat::Lit(pattern) = &patterns.arms[2].pat else {
                    panic!("negative exponent fixture")
                };
                let syn::Lit::Float(float) = &pattern.lit else {
                    panic!("reconstructed exponent and suffix")
                };
                assert_eq!(float.token().to_string(), "-1.25e+2f64");
                let syn::Stmt::Local(local) = &function.block.stmts[1] else {
                    panic!("numeric index fixture")
                };
                let mut expression = local
                    .init
                    .as_ref()
                    .expect("field initializer")
                    .expr
                    .as_ref();
                for _ in 0..2 {
                    let syn::Expr::Field(field) = expression else {
                        panic!("native split index")
                    };
                    let syn::Member::Unnamed(index) = &field.member else {
                        panic!("numeric member")
                    };
                    assert_eq!(index.index, 0);
                    assert_eq!(index.span.start().line, 3);
                    expression = &field.base;
                }
                Ok(())
            })
            .expect("numeric registrations preserve original coordinates");
        // Exercise the fixed native address domain without allocating huge text.
        let limit = usize::try_from(u32::MAX).expect("native position limit");
        assert!(matches!(
            Analysis::source_registry(limit, 0, 0),
            Err(Refusal::Count {
                resource: "native span positions",
                limit: actual,
            }) if actual == limit
        ));
        assert!(Analysis::source_registry(limit - 1, 0, 0).is_ok());
    }
}
