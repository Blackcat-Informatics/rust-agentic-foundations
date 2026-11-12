// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The repository's Census policy, shared by its test and registered target.

use std::error::Error;
use std::path::Path;

use raf_census::{ArchiveCache, Gate, GateReport, LockedRegistry};
use raf_source::{RelPath, Tree};

pub(crate) fn gate(root: impl AsRef<Path>) -> Gate {
    Gate::workspace(root).require_rules(["serde-json", "rdf-text-writer", "rdf-term-model"])
}

/// The fuzz harness is its own Cargo workspace with its own lock file. Its
/// first-party row joins the root catalog, keeping only the dependencies the
/// root lock also holds; every shared package has the same version in both.
fn with_fuzz_row(root_lock: &str, fuzz_lock: &str) -> Result<String, Box<dyn Error>> {
    let marker = "[[package]]\nname = \"raf-source-fuzz\"\n";
    let start = fuzz_lock
        .find(marker)
        .ok_or("fuzz/Cargo.lock has no raf-source-fuzz row")?;
    let stanza = fuzz_lock[start..]
        .split("\n\n")
        .next()
        .ok_or("fuzz/Cargo.lock row is empty")?;
    let mut merged = format!("{root_lock}\n");
    for line in stanza.lines() {
        let dependency = line
            .trim()
            .strip_prefix('"')
            .and_then(|rest| rest.strip_suffix("\","));
        if dependency.is_none_or(|name| root_lock.contains(&format!("name = \"{name}\"\n"))) {
            merged.push_str(line);
            merged.push('\n');
        }
    }
    Ok(merged)
}

/// The registry packages whose sources the preferred-provider proofs read,
/// sealed from the locked archives with the root and fuzz lock files joined.
fn registry(tree: &impl Tree) -> Result<LockedRegistry, Box<dyn Error>> {
    let lock = with_fuzz_row(
        &tree.read(&RelPath::new("Cargo.lock")?, u64::MAX)?,
        &tree.read(&RelPath::new("fuzz/Cargo.lock")?, u64::MAX)?,
    )?;
    Ok(LockedRegistry::new(ArchiveCache::cargo_home()?)
        .seal("purrdf-*")
        .with_lock(lock))
}

/// Evaluate the repository through the locked check.
pub(crate) fn locked_report(gate: &Gate, tree: &impl Tree) -> Result<GateReport, Box<dyn Error>> {
    Ok(gate.check_locked_registry(tree, &registry(tree)?)?)
}

/// Record the repository's admissible debt through the locked evaluation.
#[allow(
    dead_code,
    reason = "the registered example publishes debt; the read-only test never does"
)]
pub(crate) fn write_locked_debt(
    gate: &Gate,
    tree: &impl Tree,
) -> Result<raf_census::DebtWrite, Box<dyn Error>> {
    Ok(gate.write_debt_registry(&registry(tree)?)?)
}
