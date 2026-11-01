// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The compiled native consumer is also the exact source judged by the Gate.

pub(crate) fn digest(input: &[u8]) -> purrdf_hash::blake3::Hash {
    let mut state = purrdf_hash::blake3::Hasher::new();
    purrdf_hash::blake3::Hasher::update(&mut state, input);
    purrdf_hash::blake3::Hasher::finalize(&state)
}
