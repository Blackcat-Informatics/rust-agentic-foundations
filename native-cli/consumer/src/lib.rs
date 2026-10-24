// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

pub fn run(input: &[u8]) -> native::blake3::Hash {
    let mut state = native::blake3::Hasher::new();
    native::blake3::Hasher::update(&mut state, input);
    native::blake3::Hasher::finalize(&state)
}
