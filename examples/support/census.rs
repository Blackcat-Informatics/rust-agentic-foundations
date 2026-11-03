// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The repository's Census policy, shared by its test and registered target.

use std::path::Path;

use raf_census::Gate;

pub(crate) fn gate(root: impl AsRef<Path>) -> Gate {
    Gate::workspace(root).require_rules(["serde-json", "rdf-text-writer", "rdf-term-model"])
}
