// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Admitted inverse coordinates for Source's logical lexical input.

use super::resource::{Grant, Meter, Refusal};

#[cfg(test)]
pub(super) mod fixture;

/// Buffers die before their original grant and before the analysis owner moves.
pub(super) struct Coordinates<'source, 'meter> {
    offsets: Vec<usize>,
    lines: Vec<usize>,
    source: &'source str,
    grant: Grant<'meter>,
}

impl<'source, 'meter> Coordinates<'source, 'meter> {
    pub(super) const fn meter(&self) -> &'meter Meter {
        self.grant.meter()
    }

    pub(super) fn work(&self, units: usize) -> Result<(), Refusal> {
        self.meter().work(units)
    }

    /// Inspect one original parsed opaque range. These bounds require Source's
    /// raw lexer and Syn's monotone verbatim range producers, not generated or
    /// caller-built streams. Keep their original owner borrowed through cleanup.
    pub(super) fn opaque_ends(
        &self,
        tokens: &proc_macro2::TokenStream,
    ) -> Result<Option<(proc_macro2::Span, proc_macro2::Span)>, Refusal> {
        self.work(1)?;
        if tokens.is_empty() {
            return Ok(None);
        }
        let overflow = Refusal::Invariant("opaque token admission envelope overflow");
        let count = self.source.len().checked_mul(2).ok_or(overflow)?;
        let spelling = self.source.len().checked_mul(50).ok_or(overflow)?;
        let bytes = count
            .checked_mul(size_of::<proc_macro2::TokenTree>())
            .and_then(|bytes| bytes.checked_add(spelling))
            .ok_or(overflow)?;
        // Source-level clone/copy, advance, span, reader, drop and allocation
        // events, not machine instructions or allocator execution. The fixed
        // allowance is eight native events, seven reader events and three
        // grant transactions; see the capability's endpoint accounting model.
        // Admission arithmetic/control plumbing is outside these event units.
        let work = count
            .checked_mul(10)
            .and_then(|units| units.checked_add(spelling))
            .and_then(|units| units.checked_add(18))
            .ok_or(overflow)?;
        self.work(work)?;
        let mut grant = Grant::new(self.meter());
        grant.reserve_native(bytes)?;

        // into_iter clones the shared root Vec and its atomic spellings. Its
        // size_hint is already too late to admit that storage. Borrowed tokens
        // retain each original Group, so clone destruction cannot become unique.
        let mut iterator = tokens.clone().into_iter();
        let mut first = None;
        let mut last = None;
        let mut seen = 0usize;
        for tree in iterator.by_ref() {
            seen = seen.checked_add(1).ok_or(overflow)?;
            if seen > count {
                return Err(Refusal::Invariant(
                    "opaque tokens exceed their original-source admission",
                ));
            }
            let span = tree.span();
            if first.is_none() {
                first = Some(span);
            }
            last = Some(span);
        }
        drop(iterator);
        drop(grant);
        first.zip(last).map(Some).ok_or(Refusal::Invariant(
            "nonempty opaque stream lost its original tokens",
        ))
    }

    pub(super) fn new(
        source: &'source str,
        shebang: Option<&str>,
        meter: &'meter Meter,
    ) -> Result<Self, Refusal> {
        // Admit the complete logical scan before decoding source scalars.
        meter.work(source.len())?;
        let mut own = Self {
            offsets: Vec::new(),
            lines: Vec::new(),
            source,
            grant: Grant::new(meter),
        };
        own.push(false, 0)?;
        let bom: usize = if source.starts_with('\u{feff}') { 3 } else { 0 };
        let header_end = if let Some(header) = shebang {
            let comparison = header.len().checked_mul(2).ok_or(Refusal::Invariant(
                "logical source header comparison overflow",
            ))?;
            meter.work(comparison)?;
            let end = bom.checked_add(header.len()).ok_or(Refusal::Invariant(
                "logical source header arithmetic overflow",
            ))?;
            if !header.starts_with("#!")
                || header.contains('\n')
                || source.get(bom..end) != Some(header)
            {
                return Err(Refusal::Invariant(
                    "Source's retained shebang is not its original source prefix",
                ));
            }
            for (offset, _) in source[..bom].char_indices() {
                own.scalar(offset, false)?;
            }
            // Source pads each original header byte with one ASCII space.
            // Intermediate entries can be inside an original UTF-8 scalar.
            for offset in bom..end {
                own.scalar(offset, false)?;
            }
            end
        } else {
            0
        };
        for (offset, ch) in source[header_end..].char_indices() {
            let offset = header_end.checked_add(offset).ok_or(Refusal::Invariant(
                "logical source offset arithmetic overflow",
            ))?;
            own.scalar(offset, ch == '\n')?;
        }
        own.push(true, source.len())?;
        Ok(own)
    }

    fn push(&mut self, offset: bool, value: usize) -> Result<(), Refusal> {
        let values = if offset {
            &mut self.offsets
        } else {
            &mut self.lines
        };
        let relocation = if values.len() == values.capacity() {
            values.len()
        } else {
            0
        };
        self.grant.meter().work(
            relocation
                .checked_add(1)
                .ok_or(Refusal::Invariant("coordinate insertion work overflow"))?,
        )?;
        self.grant.memory(|memory| Ok(memory.push(values, value)?))
    }

    fn scalar(&mut self, offset: usize, newline: bool) -> Result<(), Refusal> {
        self.grant.meter().work(1)?;
        self.push(true, offset)?;
        if newline {
            self.push(false, self.offsets.len())?;
        }
        Ok(())
    }

    fn byte(&self, position: proc_macro2::LineColumn) -> Result<usize, Refusal> {
        let invalid = Refusal::Invariant("native span has an invalid source coordinate");
        let line = position.line.checked_sub(1).ok_or(invalid)?;
        let start = *self.lines.get(line).ok_or(invalid)?;
        let next = line.checked_add(1).ok_or(invalid)?;
        let last = if let Some(&end) = self.lines.get(next) {
            end.checked_sub(1).ok_or(invalid)?
        } else {
            self.offsets.len().checked_sub(1).ok_or(invalid)?
        };
        let character = start.checked_add(position.column).ok_or(invalid)?;
        if character > last {
            return Err(invalid);
        }
        let byte = *self.offsets.get(character).ok_or(invalid)?;
        if !self.source.is_char_boundary(byte) {
            return Err(Refusal::Invariant(
                "native span endpoint splits original UTF-8",
            ));
        }
        Ok(byte)
    }

    pub(super) fn range(&self, span: proc_macro2::Span) -> Result<std::ops::Range<usize>, Refusal> {
        self.between(span, span)
    }

    /// Convert original endpoint spans directly, without a native `Span::join`
    /// search or printing the intervening syntax into another token stream.
    pub(super) fn between(
        &self,
        first: proc_macro2::Span,
        last: proc_macro2::Span,
    ) -> Result<std::ops::Range<usize>, Refusal> {
        // Each endpoint searches the source-file and line slices. The pinned
        // slice binary search makes at most usize::BITS + 1 comparisons.
        // File search uses at most four integer comparisons (including its
        // containment assertion); line search uses one. Charge both endpoints
        // before entering native code, plus the two containment comparisons
        // in each endpoint's line conversion. This is not a total native work
        // claim.
        self.grant
            .meter()
            .work(10 * (usize::BITS as usize + 1) + 4)?;
        let start = self.byte(first.start())?;
        let end = self.byte(last.end())?;
        if start > end {
            return Err(Refusal::Invariant(
                "native span has reversed source endpoints",
            ));
        }
        Ok(start..end)
    }
}
