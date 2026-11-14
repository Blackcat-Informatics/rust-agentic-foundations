// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native Rust literal measurements for admission before syntax parsing.

use std::fmt::{self, Write};

use super::resource::Refusal;

#[derive(Default)]
pub(super) struct Literals {
    nodes: usize,
    count: usize,
    bytes: usize,
    numbers: usize,
    numeric_bytes: usize,
    numeric_squares: usize,
    minuses: usize,
}

fn overflow() -> Refusal {
    Refusal::Storage(purrdf_lex::allocation::StorageError::SizeOverflow)
}

impl Literals {
    /// Fund native iterator copies, spelling copies and measurement visits.
    /// The original parser-input copy envelope also covers their live storage;
    /// all measurement iterators die before that envelope is used by parsing.
    /// Logical cloned spellings total at most ten source bytes per input byte,
    /// including documentation escaping and synthesized `doc` identifiers.
    /// Their copy reads/writes fit twenty; the byte allowance retains four
    /// additional events. Native spare capacity is not copied by `String::clone`.
    /// Stream setup/cleanup uses at most 64 events per stream; token operations
    /// and measurement updates each use at most 32 per node. There are at most
    /// nodes + 1 streams; 64 root/result events give 128 * (nodes + 1).
    pub(super) fn measurement_work(bytes: usize, nodes: usize) -> Result<usize, Refusal> {
        let per_node = size_of::<proc_macro2::TokenTree>()
            .checked_mul(4)
            .and_then(|n| n.checked_add(128))
            .ok_or_else(overflow)?;
        nodes
            .checked_mul(per_node)
            .and_then(|n| bytes.checked_mul(24).and_then(|b| n.checked_add(b)))
            .and_then(|n| n.checked_add(128))
            .ok_or_else(overflow)
    }

    pub(super) fn measure(
        tokens: &proc_macro2::TokenStream,
        max_nodes: usize,
    ) -> Result<Self, Refusal> {
        let mut measured = Self::default();
        measured.walk(tokens.clone(), max_nodes, 0)?;
        Ok(measured)
    }

    fn walk(
        &mut self,
        tokens: proc_macro2::TokenStream,
        max_nodes: usize,
        depth: usize,
    ) -> Result<(), Refusal> {
        let limit = raf_source::Budget::Delimiters
            .limit()
            .checked_add(1)
            .ok_or_else(overflow)?;
        if depth > limit {
            return Err(Refusal::Invariant(
                "native token groups exceeded the guarded delimiter bound",
            ));
        }
        for token in tokens {
            self.nodes = self.nodes.checked_add(1).ok_or_else(overflow)?;
            if self.nodes > max_nodes {
                return Err(Refusal::Invariant(
                    "native token nodes exceeded the prepared source bound",
                ));
            }
            match token {
                proc_macro2::TokenTree::Group(group) => {
                    let depth = depth.checked_add(1).ok_or_else(overflow)?;
                    self.walk(group.stream(), max_nodes, depth)?;
                }
                proc_macro2::TokenTree::Literal(literal) => {
                    let mut spelling = Spelling::default();
                    if write!(&mut spelling, "{literal}").is_err() {
                        return Err(spelling.failure.unwrap_or(Refusal::Invariant(
                            "native literal formatting failed without a measurement refusal",
                        )));
                    }
                    self.count = self.count.checked_add(1).ok_or_else(overflow)?;
                    self.bytes = self
                        .bytes
                        .checked_add(spelling.bytes)
                        .ok_or_else(overflow)?;
                    if spelling.first.is_some_and(|byte| byte.is_ascii_digit()) {
                        self.numbers = self.numbers.checked_add(1).ok_or_else(overflow)?;
                        self.numeric_bytes = self
                            .numeric_bytes
                            .checked_add(spelling.bytes)
                            .ok_or_else(overflow)?;
                        let square = spelling
                            .bytes
                            .checked_mul(spelling.bytes)
                            .ok_or_else(overflow)?;
                        self.numeric_squares = self
                            .numeric_squares
                            .checked_add(square)
                            .ok_or_else(overflow)?;
                    }
                }
                proc_macro2::TokenTree::Punct(punct) if punct.as_char() == '-' => {
                    self.minuses = self.minuses.checked_add(1).ok_or_else(overflow)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// The exact native token-tree node count of the measured stream.
    pub(super) const fn nodes(&self) -> usize {
        self.nodes
    }

    pub(super) fn storage(&self) -> Result<usize, Refusal> {
        if self.count == 0 {
            return Ok(0);
        }
        let child_entries = self.numbers.checked_mul(2).ok_or_else(overflow)?;
        let visits = self
            .count
            .checked_mul(6)
            .and_then(|n| n.checked_add(child_entries))
            .ok_or_else(overflow)?;
        let representation = Self::representation_bytes()?;
        let visits = representation
            .checked_add(60)
            .and_then(|n| n.checked_mul(visits))
            .ok_or_else(overflow)?;
        let payload = self
            .bytes
            .checked_add(self.minuses)
            .and_then(|n| n.checked_mul(140))
            .ok_or_else(overflow)?;
        let index = self
            .bytes
            .checked_add(self.minuses.min(self.numbers))
            .and_then(|n| n.checked_mul(2))
            .and_then(|n| self.numbers.checked_mul(8).and_then(|m| n.checked_add(m)))
            .ok_or_else(overflow)?;
        payload
            .checked_add(visits)
            .and_then(|n| n.checked_add(index))
            .ok_or_else(overflow)
    }

    /// Radix arithmetic and decoder inspections, loops, UTF-8 and output events.
    /// Six decoder visits per original literal each fit 8 * spelling + 10.
    /// Numeric scans, predicates, decimal storage/output and float copies use
    /// the weighted spelling and radix-digit ceilings. Decoder relocation,
    /// search and terminal bytes and float validation use logical span units;
    /// Literal spelling copies and minus insertion count read/write events.
    /// Reconstruction scans count visits, decoded extents and cursor count spans.
    /// Span operations, construction metadata and kernel execution stay separate.
    pub(super) fn conversion_work(&self) -> Result<usize, Refusal> {
        if self.count == 0 {
            return Ok(0);
        }
        let spelling = self.bytes.checked_add(self.minuses).ok_or_else(overflow)?;
        let numeric_spelling = self
            .numeric_bytes
            .checked_add(self.minuses.min(self.numbers))
            .ok_or_else(overflow)?;
        let arithmetic = self
            .numeric_squares
            .checked_mul(14)
            .and_then(|n| {
                self.numeric_bytes
                    .checked_mul(28)
                    .and_then(|b| n.checked_add(b))
            })
            .ok_or_else(overflow)?;
        let decoding = self
            .bytes
            .checked_mul(48)
            .and_then(|n| self.count.checked_mul(60).and_then(|k| n.checked_add(k)))
            .ok_or_else(overflow)?;
        let visits = self
            .count
            .checked_mul(6)
            .and_then(|n| self.numbers.checked_mul(2).and_then(|j| n.checked_add(j)))
            .ok_or_else(overflow)?;
        // A failed negative probe can copy a nonnumeric literal in the float
        // converter, so scans use all spelling bytes, not just numeric bytes.
        let numeric_linear = spelling
            .checked_mul(112)
            .and_then(|n| {
                self.numeric_bytes
                    .checked_mul(252)
                    .and_then(|d| n.checked_add(d))
            })
            .and_then(|n| visits.checked_mul(17).and_then(|v| n.checked_add(v)))
            .ok_or_else(overflow)?;
        // Six decoder visits each fit 6 * spelling + 12 residual events.
        // One float-validation span per trial uses the weighted input bound.
        let kernels = self
            .bytes
            .checked_mul(36)
            .and_then(|n| self.count.checked_mul(72).and_then(|k| n.checked_add(k)))
            .and_then(|n| spelling.checked_mul(7).and_then(|b| n.checked_add(b)))
            .ok_or_else(overflow)?;
        // Each effective input fits 14 * length + 2 spelling/prefix events.
        // The outer tuple-index clone/display pair is additional and may carry
        // one reconstructed minus per original numeric literal.
        let copies = spelling
            .checked_mul(98)
            .and_then(|n| visits.checked_mul(2).and_then(|v| n.checked_add(v)))
            .and_then(|n| {
                numeric_spelling
                    .checked_mul(4)
                    .and_then(|b| n.checked_add(b))
            })
            .ok_or_else(overflow)?;
        // Reconstruction occurs only after numeric conversion succeeds.
        // Its scan inputs total at most 7 * numeric_spelling over 8 * numbers
        // invocations, each bounded by 16 * input length + 64 named events.
        let reconstruction = numeric_spelling
            .checked_mul(112)
            .and_then(|n| self.numbers.checked_mul(512).and_then(|v| n.checked_add(v)))
            .ok_or_else(overflow)?;
        arithmetic
            .checked_add(decoding)
            .and_then(|n| n.checked_add(numeric_linear))
            .and_then(|n| n.checked_add(kernels))
            .and_then(|n| n.checked_add(copies))
            .and_then(|n| n.checked_add(reconstruction))
            .ok_or_else(overflow)
    }

    fn representation_bytes() -> Result<usize, Refusal> {
        let alignment = align_of::<proc_macro2::Literal>().max(align_of::<Box<str>>());
        let padding = alignment
            .checked_sub(1)
            .and_then(|n| n.checked_mul(4))
            .ok_or_else(overflow)?;
        size_of::<Box<str>>()
            .checked_mul(2)
            .and_then(|n| n.checked_add(size_of::<proc_macro2::Literal>()))
            .and_then(|n| n.checked_add(padding))
            .ok_or_else(overflow)
    }
}

/// Classifies and measures the existing native Rust spelling without copying it.
#[derive(Default)]
struct Spelling {
    first: Option<u8>,
    bytes: usize,
    failure: Option<Refusal>,
}

impl Write for Spelling {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let Some(bytes) = self.bytes.checked_add(text.len()) else {
            self.failure = Some(overflow());
            return Err(fmt::Error);
        };
        if self.first.is_none() {
            self.first = text.as_bytes().first().copied();
        }
        self.bytes = bytes;
        Ok(())
    }
}
