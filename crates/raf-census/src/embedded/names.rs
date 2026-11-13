// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Rust identifier canonical equivalence through the native Unicode kernels.
//!
//! Equality of NFD forms is equality of NFC forms. Comparing decomposed scalars
//! avoids manufacturing strings and preserves the original source spellings.

use purrdf_lex::unicode::{
    TaggedScalar, UNICODE_VERSION, decompose_tagged_preallocated, decomposed_len,
};

use super::resource::{Meter, Refusal};

pub(super) const PROFILE: &str = "rust-identifier-canonical-equivalence/unicode-17";
const MAX_CANONICAL_SCALARS: usize = 4;

pub(super) fn same(left: &str, right: &str, meter: &Meter) -> Result<bool, Refusal> {
    meter.work(left.len().max(right.len()))?;
    if left == right {
        return Ok(true);
    }
    meter.work(left.len())?;
    meter.work(right.len())?;
    if left.is_ascii() && right.is_ascii() {
        return Ok(false);
    }
    if UNICODE_VERSION != (17, 0, 0) {
        return Err(Refusal::Unsupported(
            "native Unicode tables do not match the declared identifier profile",
        ));
    }
    let first = Normalized::new(left, meter)?;
    let second = Normalized::new(right, meter)?;
    meter.work(first.values.len().max(second.values.len()))?;
    Ok(first
        .values
        .iter()
        .map(|scalar| scalar.value)
        .eq(second.values.iter().map(|scalar| scalar.value)))
}

struct Normalized<'m> {
    values: Vec<TaggedScalar<()>>,
    scratch: Vec<TaggedScalar<()>>,
    order: Vec<usize>,
    meter: &'m Meter,
}

impl<'m> Normalized<'m> {
    fn new(text: &str, meter: &'m Meter) -> Result<Self, Refusal> {
        let scan = text
            .len()
            .checked_mul(MAX_CANONICAL_SCALARS)
            .ok_or(Refusal::Invariant(
                "identifier decomposition work overflows",
            ))?;
        meter.work(scan)?;
        let mut owned = Self {
            values: Vec::new(),
            scratch: Vec::new(),
            order: Vec::new(),
            meter,
        };
        let count = text.chars().count();
        meter.reserve_retained(&mut owned.values, count)?;
        // The immutable input's exact scalar count admits every following push.
        for value in text.chars() {
            owned.values.push(TaggedScalar {
                value,
                metadata: (),
            });
        }
        let decomposed = decomposed_len::<false, _>(&owned.values).ok_or(Refusal::Invariant(
            "identifier decomposition extent overflows",
        ))?;
        let levels = decomposed.bit_width();
        let per_scalar = (levels as usize + 1)
            .checked_mul(16)
            .and_then(|work| work.checked_add(8))
            .ok_or(Refusal::Invariant("identifier ordering work overflows"))?;
        meter.work(
            decomposed
                .checked_mul(per_scalar)
                .ok_or(Refusal::Invariant("identifier ordering extent overflows"))?,
        )?;
        meter.reserve_retained(&mut owned.scratch, decomposed)?;
        meter.reserve_retained(&mut owned.order, decomposed)?;
        decompose_tagged_preallocated::<false, _>(
            &mut owned.values,
            &mut owned.scratch,
            &mut owned.order,
        );
        Ok(owned)
    }
}

impl Drop for Normalized<'_> {
    fn drop(&mut self) {
        self.meter.release_buffer(std::mem::take(&mut self.values));
        self.meter.release_buffer(std::mem::take(&mut self.scratch));
        self.meter.release_buffer(std::mem::take(&mut self.order));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_equivalence_uses_native_ordering_with_exact_work_and_storage_refusals() {
        let first = "café";
        let second = "cafe\u{301}";
        let measured = Meter::new(100_000, 100_000);
        assert!(same(first, second, &measured).unwrap());
        let (work, bytes) = measured.usage();
        assert!(same(first, second, &Meter::new(work, bytes)).unwrap());
        assert_eq!(
            same(first, second, &Meter::new(work - 1, bytes)),
            Err(Refusal::Work { limit: work - 1 })
        );
        assert_eq!(
            same(first, second, &Meter::new(work, bytes - 1)),
            Err(Refusal::Memory { limit: bytes - 1 })
        );
        let meter = Meter::new(100_000, 100_000);
        assert!(!same(first, "cafe\u{300}", &meter).unwrap());
        assert!(same("a\u{301}\u{323}", "a\u{323}\u{301}", &meter).unwrap());
        assert!(same("x\u{344}", "x\u{308}\u{301}", &meter).unwrap());
    }

    #[test]
    fn native_unicode_seventeen_decompositions_fit_the_admitted_scalar_work_certificate() {
        assert_eq!(UNICODE_VERSION, (17, 0, 0));
        for value in (0..=0x10_ffff).filter_map(char::from_u32) {
            let input = [TaggedScalar {
                value,
                metadata: (),
            }];
            assert!(decomposed_len::<false, _>(&input).unwrap() <= MAX_CANONICAL_SCALARS);
        }
    }
}
