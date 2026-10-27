// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Unsigned fixture and environment scalars use the native integer parser.

pub(crate) fn unsigned<T: TryFrom<u128>>(text: &str) -> Option<T> {
    let digits = text.strip_prefix('+').unwrap_or(text);
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let significant = digits.trim_start_matches('0');
    let significant = if significant.is_empty() {
        "0"
    } else {
        significant
    };
    if significant.len() > 39 {
        return None;
    }
    let value = purrdf_xsd::numeric::parse_integer_typed(
        significant,
        purrdf_xsd::XsdDatatype::NonNegativeInteger,
    )
    .ok()?
    .into_bigint()
    .unsigned_abs_u128()?;
    T::try_from(value).ok()
}
