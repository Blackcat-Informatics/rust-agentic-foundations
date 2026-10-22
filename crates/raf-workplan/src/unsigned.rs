// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Bounded unsigned host counts through the native exact integer reader.

/// Rust's unsigned decimal spelling, including an optional `+` and leading
/// zeroes, narrowed to a host count. Validate the lexical subset and significant
/// width before native arbitrary-precision parsing.
pub(crate) fn count(text: &str) -> Option<usize> {
    let digits = text.strip_prefix('+').unwrap_or(text);
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let digits = digits.trim_start_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    if digits.len() > usize::MAX.ilog10() as usize + 1 {
        return None;
    }
    purrdf_xsd::numeric::parse_integer_typed(digits, purrdf_xsd::XsdDatatype::NonNegativeInteger)
        .ok()?
        .into_bigint()
        .unsigned_abs_u128()
        .and_then(|value| usize::try_from(value).ok())
}
