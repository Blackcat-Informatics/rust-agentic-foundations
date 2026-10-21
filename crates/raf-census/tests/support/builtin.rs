// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The workspace the `baseline` example reports when it is given no
//! directory, and the text it prints; the reports test runs the same code.
//!
//! The workspace has one body group (`alpha` and `beta`), one shim group
//! (`one` and `two`, forwarders to `helper`), a hexadecimal digit table, the
//! constant 65536 and one job whose home is `alpha`.

use raf_census::{Census, CensusOptions, Error, Fingerprint, Job, JobForbidden, JobReport};
use std::path::Path;

use raf_source::{DiskTree, MemoryTree, Tree};

const MANIFEST: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

const SOURCE: &str = "\
pub fn alpha(a: u32) -> u32 { a + a + a + a + a + a + a + a + a + a + a }
pub fn beta(b: u32) -> u32 { b + b + b + b + b + b + b + b + b + b + b }
pub fn helper(x: u32, y: u32) -> u32 { x + y }
pub fn one(a: u32) -> u32 { helper(a, 1) }
pub fn two(b: u32) -> u32 { helper(b, 1) }
pub fn digits() -> &'static str { \"0123456789abcdef\" }
pub const BIG: u32 = 65536;
";

/// The built-in workspace, in memory.
pub(crate) fn tree() -> MemoryTree {
    [("Cargo.toml", MANIFEST), ("src/lib.rs", SOURCE)]
        .into_iter()
        .collect()
}

/// The one job of the built-in workspace: `alpha` is the home, and the job
/// forbids the body group's fingerprint and the constant 65536 elsewhere.
pub(crate) fn jobs(census: &Census) -> Vec<Job> {
    let fingerprints: Vec<Fingerprint> = census
        .body_groups()
        .map(|group| group.fingerprint.clone())
        .collect();
    vec![Job::new("sum", "demo::alpha").forbidding(JobForbidden {
        fingerprints,
        constants: vec![0x1_0000],
    })]
}

/// The baseline and the index of `census` under `jobs`, as the example prints
/// them.
pub(crate) fn render(census: &Census, jobs: &[Job]) -> Result<String, Error> {
    let reports: Vec<JobReport> = census.match_jobs(jobs);
    Ok(format!(
        "baseline\n{}index\n{}",
        census.baseline().to_turtle()?,
        census.index(&reports)?.to_turtle()?
    ))
}

/// The output for the built-in workspace.
pub(crate) fn builtin() -> Result<String, Error> {
    let census = Census::load(&tree(), &["."], &CensusOptions::default())?;
    render(&census, &jobs(&census))
}

/// The output for the workspace under `root` of `tree`, with no jobs.
pub(crate) fn of(tree: &impl Tree, root: &str) -> Result<String, Error> {
    let census = Census::load(tree, &[root], &CensusOptions::default())?;
    render(&census, &[])
}

/// The output for the workspace under the directory `dir`, with no jobs.
///
/// A relative `dir` is a root of the tree at the current directory. An
/// absolute `dir` is read as its parent's tree with its file name as the root,
/// so `<abs>/crates` and `crates` from the directory `<abs>` report the same.
pub(crate) fn of_dir(dir: &str) -> Result<String, Error> {
    let path = Path::new(dir);
    let (base, root) = match (path.parent(), path.file_name().and_then(|n| n.to_str())) {
        (Some(parent), Some(name)) if path.is_absolute() => (parent, name),
        (_, _) if path.is_absolute() => (path, "."),
        _ => (Path::new("."), dir),
    };
    let tree = DiskTree::new(base).map_err(|e| Error::Source {
        file: dir.to_owned(),
        line: 1,
        reason: e.to_string(),
    })?;
    of(&tree, root)
}
