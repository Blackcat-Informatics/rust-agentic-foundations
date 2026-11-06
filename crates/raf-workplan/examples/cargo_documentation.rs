// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Inspect existing documentation data without invoking Cargo or a compiler.

use raf_workplan::diagram::cargo::{self, CargoDocOptions};
use std::{
    error::Error,
    fs::File,
    io::{Read as _, Write as _},
    path::Path,
};

fn read(path: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(cargo::MAX_METADATA_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > cargo::MAX_METADATA_BYTES {
        return Err("artifact carrier exceeds 16 MiB".into());
    }
    Ok(bytes)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let [root, artifacts, package, metadata, messages] = args.as_slice() else {
        return Err(
            "usage: cargo_documentation ROOT ARTIFACT_ROOT PACKAGE METADATA_JSON MESSAGES_JSON"
                .into(),
        );
    };
    let package = package.to_str().ok_or("package must be UTF-8")?;
    let options = CargoDocOptions::new(&[package], &[], false, Some(Path::new(artifacts)))?;
    let selected = cargo::read_metadata(
        Path::new(root),
        Path::new(artifacts),
        &options,
        &read(Path::new(metadata))?,
    )?;
    let imported = cargo::read_artifacts(&selected, &read(Path::new(messages))?)?;
    let mut output = std::io::stdout().lock();
    for artifact in imported.artifacts() {
        writeln!(
            output,
            "imported {} {} SHA-256 {}",
            artifact.library().package.escape_debug(),
            artifact.reported_path().display(),
            artifact.sha256()
        )?;
    }
    Ok(())
}
