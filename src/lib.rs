// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Root package of the `rust-agentic-foundations` workspace.
//!
//! This package hosts the repository's dogfood tests. It is not published.

use std::ffi::OsString;
use std::path::PathBuf;

/// The repository read root shared by its gates and runnable example.
///
/// `RAF_GATE_ROOT` selects an explicit checkout snapshot. Without it the
/// root is this package's manifest directory. The selected root is subjected
/// to the same gate rules in both cases.
#[must_use]
pub fn gate_root() -> PathBuf {
    gate_root_for(std::env::var_os("RAF_GATE_ROOT"))
}

fn gate_root_for(snapshot: Option<OsString>) -> PathBuf {
    snapshot.map_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")), PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::gate_root_for;
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[test]
    fn the_absent_gate_root_uses_the_manifest_directory() {
        assert_eq!(
            gate_root_for(None),
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        );
    }

    #[test]
    fn an_explicit_gate_root_is_used_without_falling_back() {
        assert_eq!(
            gate_root_for(Some(OsString::from("chosen-snapshot"))),
            PathBuf::from("chosen-snapshot")
        );
    }
}
