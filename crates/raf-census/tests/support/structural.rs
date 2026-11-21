// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

/// A structural fixture supplies Rust as data and has no compiler receipt.
pub(crate) fn fixture_gate(root: impl AsRef<std::path::Path>) -> raf_census::Gate {
    raf_census::Gate::workspace(root).without_production_lints()
}
