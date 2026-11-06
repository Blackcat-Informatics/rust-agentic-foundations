// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Source-derived admission for the native authoring reader.

use std::borrow::Cow;
use std::mem::size_of;

use purrdf_markdown::syntax::{
    AuthoringComment, AuthoringHeading, AuthoringLine, AuthoringLink, Block, ContentLine, Inline,
    Link, SourcePosition,
};
use purrdf_markdown::{CodeSpanLimits, SyntaxError, SyntaxLimits};

use crate::limits::{MAX_BLOCK_NESTING, MAX_TEXT_BYTES};

fn overflow() -> SyntaxError {
    SyntaxError::Storage {
        position: SourcePosition {
            byte: 0,
            line: 1,
            column: 1,
        },
        cause: purrdf_lex::allocation::StorageError::SizeOverflow,
    }
}

/// Native capacity growth admits both old and replacement buffers. The
/// source bounds every observation population and every mapped input; grants
/// admit storage, and do not allocate it in advance.
pub(super) fn limits(n: usize, origin: usize) -> Result<SyntaxLimits, SyntaxError> {
    let values = || -> Option<(usize, usize)> {
        let m = n.checked_add(2)?; // Synthetic final newline and offset sentinel.
        let d = MAX_BLOCK_NESTING;
        let u = size_of::<usize>();
        let records = size_of::<ContentLine>()
            + size_of::<Block>()
            + size_of::<Inline>()
            + size_of::<AuthoringLine<'_>>()
            + size_of::<AuthoringHeading<'_>>()
            + size_of::<AuthoringComment>()
            + 4 * size_of::<AuthoringLink<'_>>()
            + size_of::<Link<'_>>()
            + size_of::<bool>();
        let arrays = m.checked_add(4)?.checked_mul(3)?.checked_mul(records)?;
        // Five input families, each with text and two byte-offset arrays.
        let inputs = m
            .checked_add(8)?
            .checked_add(m.checked_add(4)?.checked_mul(2)?.checked_mul(u)?)?
            .checked_mul(15)?;
        let brackets = m.checked_add(4)?.checked_mul(3)?.checked_mul(2 * u)?;
        // Prefix markers fit in eight native words; list entries in two.
        let prefixes = d.checked_add(4)?.checked_mul(3)?.checked_mul(10 * u)?;
        // At most six source lengths of retained text and nine source lengths
        // of string owners; each buffer costs at most 3 * (length + 8).
        let strings = m.checked_mul(234)?;
        let memory = arrays
            .checked_add(inputs)?
            .checked_add(brackets)?
            .checked_add(prefixes)?
            .checked_add(strings)?;
        // Two line-local code passes cost at most twelve squared source
        // lengths. Destination/title retries and bracket visits add three.
        // The remaining term covers endpoint probes, fixed-depth ancestor
        // visits, prefix scans, native copies and the explicit sorting charge.
        let depth = d.checked_add(1)?;
        let linear = depth
            .checked_mul(depth)?
            .checked_add(depth)?
            .checked_add(512)?
            .checked_add(3 * usize::BITS as usize)?;
        let work = m
            .checked_mul(m)?
            .checked_mul(16)?
            .checked_add(linear.checked_mul(m)?)?;
        Some((memory, work))
    };
    let (auxiliary_bytes, work) = values().ok_or_else(overflow)?;
    Ok(SyntaxLimits {
        source_bytes: MAX_TEXT_BYTES.checked_sub(origin).ok_or_else(overflow)?,
        auxiliary_bytes,
        work: u64::try_from(work).map_err(|_| overflow())?,
        containers: MAX_BLOCK_NESTING,
        nesting: n.checked_add(2).ok_or_else(overflow)?,
        link_part_bytes: super::link::MAX_LINK_PART,
    })
}

/// Decoded scalars retain native code-span semantics and typed refusals.
pub(crate) fn scalar_prose(text: &str) -> Result<Cow<'_, str>, SyntaxError> {
    let values = || -> Option<(usize, usize)> {
        let n = text.len();
        let m = n.checked_add(1)?;
        let memory = n.checked_mul(3)?.checked_add(16)?;
        let work = m
            .checked_mul(m)?
            .checked_mul(6)?
            .checked_add(m.checked_mul(8)?)?;
        Some((memory, work))
    };
    let (auxiliary_bytes, work) = values().ok_or_else(overflow)?;
    purrdf_markdown::replace_code_spans(
        text,
        CodeSpanLimits {
            source_bytes: text.len(),
            auxiliary_bytes,
            work: u64::try_from(work).map_err(|_| overflow())?,
        },
    )
}
