// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Explicit artifact inspection and invocation-scoped documentation compilation.

use crate::{
    cli::{ArtifactArgs, LibraryArgs, TrustedDocsArgs},
    exit::Failure,
    render::{Record, Report},
    result::Result as CommandResult,
    trusted_docs,
};
use raf_workplan::diagram::cargo::{self, CargoDocOptions, DocumentationArtifacts};
use std::{fs::File, io::Read as _, path::Path};

fn options(args: &LibraryArgs) -> Result<CargoDocOptions, Failure> {
    let packages: Vec<_> = args.packages.iter().map(String::as_str).collect();
    let features: Vec<_> = args.features.iter().map(String::as_str).collect();
    Ok(CargoDocOptions::new(
        &packages,
        &features,
        args.no_default_features,
        Some(&args.target_directory),
    )?)
}

fn read(path: &Path) -> Result<Vec<u8>, Failure> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(cargo::MAX_METADATA_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > cargo::MAX_METADATA_BYTES {
        return Err(Failure::new(format!(
            "{} exceeds the artifact carrier byte bound",
            path.display()
        )));
    }
    Ok(bytes)
}

pub(crate) fn artifacts(root: &Path, args: &ArtifactArgs) -> Result<Report, Failure> {
    let options = options(&args.libraries)?;
    let metadata = read(&args.metadata)?;
    let messages = read(&args.messages)?;
    let selection =
        cargo::read_metadata(root, &args.libraries.target_directory, &options, &metadata)?;
    let artifacts = cargo::read_artifacts(&selection, &messages)?;
    report(
        &artifacts,
        "imported artifact data; compiler execution is not verified",
    )
}

pub(crate) fn build(root: &Path, args: &TrustedDocsArgs) -> Result<Report, Failure> {
    let built = trusted_docs::build(&trusted_docs::BuildRequest {
        root: root.to_path_buf(),
        toolchain_directory: args.toolchain_directory.clone(),
        target_directory: args.libraries.target_directory.clone(),
        packages: args.libraries.packages.clone(),
        features: args.libraries.features.clone(),
        no_default_features: args.libraries.no_default_features,
        trust_docs: args.trust_docs,
    })?;
    let options = options(&args.libraries)?;
    let before = cargo::read_metadata(
        root,
        &args.libraries.target_directory,
        &options,
        &built.metadata_before,
    )?;
    let after = cargo::read_metadata(
        root,
        &args.libraries.target_directory,
        &options,
        &built.metadata_after,
    )?;
    if before != after {
        return Err(Failure::new(
            "documentation metadata changed during compilation; inspect the workspace before retrying",
        ));
    }
    let artifacts = cargo::read_artifacts(&after, &built.messages)?;
    let mut report = report(&artifacts, "trusted documentation compilation completed")?;
    report.note(raf_workplan::escape::shown_lines(&format!(
        "{}\n{}\n{}",
        built.cargo_version, built.rustc_version, built.rustdoc_version
    )));
    if !built.diagnostics.is_empty() {
        report.note(raf_workplan::escape::shown_lines(&String::from_utf8_lossy(
            &built.diagnostics,
        )));
    }
    Ok(report)
}

fn report(artifacts: &DocumentationArtifacts, message: &str) -> Result<Report, Failure> {
    let records = artifacts
        .artifacts()
        .iter()
        .map(|artifact| {
            let entry = artifact
                .reported_path()
                .to_str()
                .ok_or_else(|| Failure::new("artifact path must be UTF-8"))?;
            Ok(Record::outcome(
                CommandResult::DocumentationArtifact {
                    package: artifact.library().package.clone(),
                    entry: entry.to_owned(),
                    sha256: artifact.sha256().to_string(),
                    provenance: message.to_owned(),
                },
                raf_workplan::escape::shown_lines(&format!(
                    "{}: {entry} SHA-256 {}",
                    artifact.library().package,
                    artifact.sha256()
                )),
            ))
        })
        .collect::<Result<Vec<_>, Failure>>()?;
    let mut report = Report::of(records);
    report.note(message);
    Ok(report)
}
